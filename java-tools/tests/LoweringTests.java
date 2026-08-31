package mpk.java2vir;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Predicate;
import java.util.function.UnaryOperator;
import static mpk.java2vir.JavaIr.*;

/** Private executor compiled outside the candidate JAR. No public source-processing entrypoint. */
public final class LoweringTests {
    private LoweringTests() {}
    private static final Path FIXTURES = Path.of("/mpk/tests");
    private static final List<String> COUNTERS = List.of("instructions_per_method", "instructions_per_closure",
            "cfg_blocks_per_method", "cfg_blocks_per_closure", "frontend_stdout", "frontend_stderr",
            "vir_canonical_bytes", "source_map_canonical_bytes", "source_manifest_canonical_bytes");
    private static int assertions;
    private static JavaEmission.Identity identity;
    private record Fixture(CapturedSnapshot snapshot, Program program) {}
    @FunctionalInterface private interface Operation { void run(); }
    private static void check(boolean condition, String label) { assertions++; if (!condition) throw new AssertionError(label); }
    private static FrontendFailure expect(String code, Operation operation) {
        try { operation.run(); }
        catch (FrontendFailure failure) { check(failure.code().equals(code), code + ": got " + failure.code()); return failure; }
        throw new AssertionError("missing failure " + code);
    }
    private static Map<String, Object> failure(String id, Selection selection, FrontendFailure failure) {
        return Map.of("id", id, "code", failure.code(), "status", failure.status(), "phase", failure.phase(), "exit", failure.exitCode(),
                "envelope", new String(Protocol.failure(selection, failure), StandardCharsets.UTF_8));
    }

    public static void main(String[] arguments) throws Exception {
        check(arguments.length == 0, "no test arguments");
        String jarHash = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(Path.of("/work/java2vir.jar"))));
        identity = new JavaEmission.Identity("0".repeat(64), "test.java.toolchain", JavaEmission.JDK_ARCHIVE_SHA256, "test.java.frontend", jarHash);
        var cases = new ArrayList<Map<String, Object>>();
        var fixtures = new HashMap<String, Fixture>();
        for (String line : Files.readAllLines(FIXTURES.resolve("lowering-cases.tsv"))) {
            String[] fields = line.split("\t", -1);
            String id = fields[0], code = fields[3], phase = fields[4];
            Path root = FIXTURES.resolve("lowering/" + fields[1]);
            Selection selection = selection(root.resolve("selection.json"));
            CapturedSnapshot snapshot = CapturedSnapshot.capture(root.resolve("snapshot"), selection);
            Program program;
            byte[] bytes;
            try {
                program = JavaLowering.lower(JavaAdmission.analyze(snapshot));
                if (id.equals("precedence/contract_before_lowering")) {
                    // This downstream defect must never get precedence over the
                    // invalid sidecar. Use the same mutation tested below.
                    program = rawShiftCount(program, 1);
                }
                bytes = JavaEmission.emit(snapshot, program, identity);
            } catch (FrontendFailure failure) {
                check(failure.code().equals(code) && failure.phase().equals(phase), id + ": " + failure.code() + "/" + failure.phase());
                JavaFrontend.Result pipeline = JavaFrontend.process(snapshot, identity);
                check(pipeline.exitCode() == failure.exitCode() && Arrays.equals(pipeline.stdout(), Protocol.failure(selection, failure)), "failure pipeline " + id);
                cases.add(failure(id, selection, failure));
                continue;
            }
            check(code.equals("ir-lowered"), "unexpected success " + id);
            JavaFrontend.Result repeated = JavaFrontend.process(snapshot, identity);
            check(repeated.exitCode() == 0 && Arrays.equals(bytes, repeated.stdout()), "fresh-session exact bytes " + id);
            byte[] detached = repeated.stdout(); detached[0] = 0;
            check(repeated.stdout()[0] == '{', "immutable completed result");
            cases.add(Map.of("id", id, "exit", 0, "envelope", new String(bytes, StandardCharsets.UTF_8),
                    "repeat_sha256", HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(repeated.stdout()))));
            if (Set.of("accepted/int.identity", "accepted/int.division", "accepted/int.wrap_add", "accepted/int.shift_unsigned_right",
                    "accepted/control.ternary", "extra/live-prefix", "extra/live-call-arguments", "extra/map-unicode").contains(id))
                fixtures.put(id, new Fixture(snapshot, program));
            if (id.startsWith("link/")) fixtures.put(id, new Fixture(snapshot, program));
        }
        var mutations = mutations(fixtures);
        var maps = maps();
        counterBoundaries();
        canonicalBytes();
        System.out.write(CanonicalJson.encode(Map.of("schema", "mpk.java.lowering_tests.v0", "cases", cases,
                "mutations", mutations, "maps", maps, "counter_boundaries", COUNTERS, "assertions", assertions), "frontend_stdout", true));
    }

    private static List<Map<String, Object>> mutations(Map<String, Fixture> fixtures) {
        var results = new ArrayList<Map<String, Object>>();
        Fixture divide = fixtures.get("accepted/int.division"), add = fixtures.get("accepted/int.wrap_add");
        Fixture shift = fixtures.get("accepted/int.shift_unsigned_right"), plain = fixtures.get("accepted/int.identity");
        Predicate<Instruction> division = instruction -> "bv_sdiv".equals(instruction.op());
        results.add(artifactFailure("lowering.div_missing", divide, change(divide.program(), division, instruction -> checks(instruction, List.of())), "JAVA_LOWERING_CHECK_MISSING"));
        results.add(artifactFailure("lowering.div_extra", divide, change(divide.program(), division, instruction -> checks(instruction,
                List.of("divisor_nonzero", "signed_divrem_representable"))), "JAVA_LOWERING_CHECK_EXTRA"));
        results.add(artifactFailure("lowering.overflow", add, change(add.program(), instruction -> "bv_add".equals(instruction.op()),
                instruction -> checks(instruction, List.of("integer_no_overflow"))), "JAVA_LOWERING_CHECK_EXTRA"));
        results.add(artifactFailure("lowering.shift_unmasked", shift, rawShiftCount(shift.program(), 1), "JAVA_LOWERING_SHIFT_PATTERN"));
        results.add(artifactFailure("lowering.shift_unlinked", shift, change(shift.program(), instruction -> "bv_lshr".equals(instruction.op()),
                instruction -> instruction.rewrite(instruction.result(), List.of(instruction.operands().getFirst(), new Value("t0", Type.I32)))), "JAVA_LOWERING_SHIFT_PATTERN"));
        results.add(artifactFailure("lowering.shift_wrong_mask", shift, change(shift.program(), instruction -> "31".equals(instruction.literal()),
                instruction -> new Instruction(instruction.result(), instruction.kind(), instruction.op(), instruction.operands(), "63",
                        instruction.target(), instruction.function(), instruction.contractHash(), instruction.checks(), instruction.origin())), "JAVA_LOWERING_SHIFT_PATTERN"));
        Value unsigned = shift.program().functions().getFirst().blocks().getFirst().instructions().stream()
                .filter(instruction -> "bv_lshr".equals(instruction.op())).findFirst().orElseThrow().result();
        results.add(artifactFailure("lowering.unsigned_escape", shift, changeTerminator(shift.program(), end -> end.kind().equals("Return"),
                end -> new Terminator(end.kind(), null, List.of(unsigned), end.edges(), end.origin())), "JAVA_LOWERING_SHIFT_PATTERN"));
        results.add(artifactFailure("extra/duplicate-check", divide, change(divide.program(), division, instruction -> checks(instruction,
                List.of("divisor_nonzero", "divisor_nonzero"))), "JAVA_LOWERING_CHECK_EXTRA"));
        results.add(artifactFailure("extra/swapped-mask", shift, change(shift.program(), instruction -> "bv_and".equals(instruction.op()),
                instruction -> instruction.rewrite(instruction.result(), instruction.operands().reversed())), "JAVA_LOWERING_SHIFT_PATTERN"));
        results.add(artifactFailure("extra/unlisted-operation", add, change(add.program(), instruction -> "bv_add".equals(instruction.op()),
                instruction -> new Instruction(instruction.result(), instruction.kind(), "bv_udiv", instruction.operands(), instruction.literal(),
                        instruction.target(), instruction.function(), instruction.contractHash(), instruction.checks(), instruction.origin())), "JAVA_LOWERING_OPERATION"));
        Fixture calls = fixtures.get("extra/live-call-arguments");
        results.add(artifactFailure("extra/callee-hash", calls, change(calls.program(), instruction -> instruction.kind().equals("CallStatic"),
                instruction -> new Instruction(instruction.result(), instruction.kind(), instruction.op(), instruction.operands(), instruction.literal(),
                        instruction.target(), instruction.function(), "f".repeat(64), instruction.checks(), instruction.origin())), "JAVA_LOWERING_OPERATION"));
        Fixture live = fixtures.get("extra/live-prefix");
        Value external = live.program().functions().getFirst().blocks().getFirst().instructions().getLast().result();
        results.add(artifactFailure("extra/cross-block-value", live, change(live.program(), instruction -> "bv_add".equals(instruction.op())
                && instruction.operands().stream().anyMatch(value -> value.id().startsWith("p")), instruction ->
                instruction.rewrite(instruction.result(), List.of(external, instruction.operands().get(1)))), "JAVA_LOWERING_CFG"));
        Fixture branch = fixtures.get("accepted/control.ternary");
        results.add(artifactFailure("extra/cycle", branch, changeTerminator(branch.program(), end -> end.kind().equals("Jump"),
                end -> new Terminator("Jump", null, List.of(), List.of(new Edge("bb0", List.of())), end.origin())), "JAVA_LOWERING_CFG"));

        Origin original = plain.program().functions().getFirst().origin();
        results.add(artifactFailure("precedence/map_failure_prevents_partial_output", plain,
                functionOrigin(plain.program(), new Origin(original.source(), original.tree(), 0, 0)), "JAVA_SOURCE_MAP_RANGE"));
        results.add(artifactFailure("extra/map-reversed", plain,
                functionOrigin(plain.program(), new Origin(original.source(), original.tree(), original.end(), original.start())), "JAVA_SOURCE_MAP_RANGE"));
        results.add(artifactFailure("extra/map-absent", plain,
                functionOrigin(plain.program(), new Origin(original.source(), original.tree(), -1, -1)), "JAVA_SOURCE_MAP_RANGE"));
        results.add(artifactFailure("extra/map-moved", plain,
                functionOrigin(plain.program(), new Origin(original.source(), original.tree(), original.start() + 1, original.end())), "JAVA_SOURCE_MAP_RANGE"));
        var externalSource = new SourceText(original.source().path(), original.source().text().getBytes(StandardCharsets.UTF_8));
        results.add(artifactFailure("extra/map-external", plain,
                functionOrigin(plain.program(), new Origin(externalSource, original.tree(), original.start(), original.end())), "JAVA_SOURCE_MAP_EXTERNAL"));
        results.add(artifactFailure("extra/map-unowned-tree", plain,
                functionOrigin(plain.program(), new Origin(original.source(), null, original.start(), original.end())), "JAVA_SOURCE_MAP_EXTERNAL"));
        Fixture unicode = fixtures.get("extra/map-unicode");
        Origin unicodeOrigin = unicode.program().functions().getFirst().origin();
        int low = unicodeOrigin.source().text().indexOf("😀") + 1;
        results.add(artifactFailure("extra/map-split-surrogate", unicode,
                functionOrigin(unicode.program(), new Origin(unicodeOrigin.source(), unicodeOrigin.tree(), low, low + 1)), "JAVA_SOURCE_MAP_UTF16"));
        var sourceMethod = plain.program().admitted().closure().methods().getFirst();
        results.add(artifactFailure("extra/map-function-role", plain, functionOrigin(plain.program(),
                Origin.of(plain.program().admitted().closure().origins(), sourceMethod.declaration().getBody())), "JAVA_SOURCE_MAP_RANGE"));
        results.add(artifactFailure("extra/map-return-role", plain, changeTerminator(plain.program(), end -> end.kind().equals("Return"),
                end -> new Terminator(end.kind(), null, end.values(), end.edges(), Origin.of(plain.program().admitted().closure().origins(),
                        ((com.sun.source.tree.ReturnTree) end.origin().tree()).getExpression()))), "JAVA_SOURCE_MAP_RANGE"));
        Origin callee = calls.program().functions().getFirst().origin();
        results.add(artifactFailure("extra/map-other-method", calls, change(calls.program(), instruction -> instruction.kind().equals("CallStatic"),
                instruction -> new Instruction(instruction.result(), instruction.kind(), instruction.op(), instruction.operands(), instruction.literal(),
                        instruction.target(), instruction.function(), instruction.contractHash(), instruction.checks(), callee)), "JAVA_SOURCE_MAP_RANGE"));
        for (String id : List.of("link/changed-source", "link/changed-sidecar-bytes")) {
            Fixture changed = fixtures.get(id);
            check(changed.snapshot().selection().equals(plain.snapshot().selection()), "same selection for raw-byte crossing");
            results.add(artifactFailure(id, changed, plain.program(), "JAVA_FRONTEND_INTERNAL"));
        }
        var supplied = new JavaEmission.Identity("a".repeat(64), "test.supplied.toolchain", "b".repeat(64),
                "test.supplied.frontend", identity.frontendSha256());
        String suppliedOutput = new String(JavaEmission.emit(plain.snapshot(), plain.program(), supplied), StandardCharsets.UTF_8);
        check(suppliedOutput.contains("\"distribution_sha256\":\"" + "b".repeat(64) + "\"")
                && suppliedOutput.contains("\"registry_sha256\":\"" + "a".repeat(64) + "\""), "release distribution identity is supplied, not guessed from JDK archive");
        try { plain.program().functions().clear(); throw new AssertionError("mutable functions"); }
        catch (UnsupportedOperationException expected) { assertions++; }
        try { plain.program().functions().getFirst().blocks().clear(); throw new AssertionError("mutable blocks"); }
        catch (UnsupportedOperationException expected) { assertions++; }
        return results;
    }

    private static Map<String, Object> artifactFailure(String id, Fixture fixture, Program program, String code) {
        var published = new ByteArrayOutputStream();
        FrontendFailure rejected = expect(code, () -> published.writeBytes(JavaEmission.emit(fixture.snapshot(), program, identity)));
        check(published.size() == 0, "atomic failure " + id);
        var result = new java.util.TreeMap<String, Object>(failure(id, fixture.snapshot().selection(), rejected));
        result.put("published_bytes", published.size());
        return result;
    }
    private static Instruction checks(Instruction instruction, List<String> checks) {
        return new Instruction(instruction.result(), instruction.kind(), instruction.op(), instruction.operands(), instruction.literal(),
                instruction.target(), instruction.function(), instruction.contractHash(), checks, instruction.origin());
    }
    private static Program rawShiftCount(Program program, int index) {
        return change(program, instruction -> "bv_lshr".equals(instruction.op()), instruction ->
                instruction.rewrite(instruction.result(), List.of(instruction.operands().getFirst(), new Value("arg" + index, Type.I32))));
    }
    private static Function blocks(Function original, List<Block> blocks) {
        return new Function(original.id(), original.name(), original.parameters(), original.result(), original.locals(), blocks,
                original.contracts(), original.features(), original.origin());
    }
    private static Program change(Program program, Predicate<Instruction> matches, UnaryOperator<Instruction> mutation) {
        boolean[] changed = {false};
        var functions = program.functions().stream().map(function -> blocks(function, function.blocks().stream().map(block ->
                new Block(block.label(), block.parameters(), block.instructions().stream().map(instruction -> {
                    if (changed[0] || !matches.test(instruction)) return instruction;
                    changed[0] = true; return mutation.apply(instruction);
                }).toList(), block.terminator())).toList())).toList();
        check(changed[0], "mutation target exists");
        return new Program(program.admitted(), functions);
    }
    private static Program changeTerminator(Program program, Predicate<Terminator> matches, UnaryOperator<Terminator> mutation) {
        boolean[] changed = {false};
        var functions = program.functions().stream().map(function -> blocks(function, function.blocks().stream().map(block -> {
            Terminator end = block.terminator();
            if (!changed[0] && matches.test(end)) { end = mutation.apply(end); changed[0] = true; }
            return new Block(block.label(), block.parameters(), block.instructions(), end);
        }).toList())).toList();
        check(changed[0], "terminator mutation target exists");
        return new Program(program.admitted(), functions);
    }
    private static Program functionOrigin(Program program, Origin origin) {
        var functions = new ArrayList<>(program.functions());
        Function original = functions.getFirst();
        functions.set(0, new Function(original.id(), original.name(), original.parameters(), original.result(), original.locals(),
                original.blocks(), original.contracts(), original.features(), origin));
        return new Program(program.admitted(), functions);
    }

    private static List<Map<String, Object>> maps() throws Exception {
        var result = new ArrayList<Map<String, Object>>();
        for (String line : Files.readAllLines(FIXTURES.resolve("source-maps.tsv"))) {
            String[] fields = line.split("\t");
            SourceText source = new SourceText("src/vector/Case.java", Files.readAllBytes(FIXTURES.resolve("maps/" + fields[0] + ".txt")));
            long start = Long.parseLong(fields[1]), end = Long.parseLong(fields[2]);
            if (fields[3].equals("accept")) {
                Map<String, Object> range = JavaSourceMaps.range(source, start, end);
                result.add(Map.of("id", fields[0], "range", List.of(range.get("start"), range.get("end"))));
            } else {
                FrontendFailure failure = expect(fields[3], () -> JavaSourceMaps.range(source, start, end));
                check(failure.phase().equals("emission") && failure.exitCode() == 1, "map failure phase");
                result.add(Map.of("id", fields[0], "code", failure.code()));
            }
        }
        return result;
    }

    private static void counterBoundaries() {
        var instructions = new ClosureCounter();
        var first = new MethodCounter(instructions);
        for (int n = 0; n < 100000; n++) first.instruction();
        expect("JAVA_LIMIT_INSTRUCTIONS_PER_METHOD", first::instruction);
        check(first.instructions() == 100000 && instructions.instructions() == 100000, "instruction method excess not retained");
        var second = new MethodCounter(instructions);
        for (int n = 0; n < 100000; n++) second.instruction();
        var third = new MethodCounter(instructions);
        for (int n = 0; n < 50000; n++) third.instruction();
        expect("JAVA_LIMIT_INSTRUCTIONS_PER_CLOSURE", third::instruction);
        check(third.instructions() == 50000 && instructions.instructions() == 250000, "instruction closure excess not retained");
        var blocks = new ClosureCounter();
        var one = new MethodCounter(blocks);
        for (int n = 0; n < 1024; n++) one.block();
        expect("JAVA_LIMIT_CFG_BLOCKS_PER_METHOD", one::block);
        check(one.blocks() == 1024 && blocks.blocks() == 1024, "block method excess not retained");
        for (int method = 0; method < 7; method++) {
            var next = new MethodCounter(blocks);
            for (int n = 0; n < 1024; n++) next.block();
        }
        var last = new MethodCounter(blocks);
        expect("JAVA_LIMIT_CFG_BLOCKS_PER_CLOSURE", last::block);
        check(last.blocks() == 0 && blocks.blocks() == 8192, "block closure excess not retained");
        // These are the production serializer's counters, tested without
        // allocating 256 MiB outputs or claiming a maximal source is admissible.
        for (String name : COUNTERS.subList(4, COUNTERS.size())) {
            var counter = new CanonicalJson.ByteCounter(name);
            var definition = FrontendLimits.DEFINITIONS.get(name);
            counter.append(definition.maximum() - 1); counter.append(1);
            expect(definition.code(), () -> counter.append(1));
            check(counter.bytes() == definition.maximum(), "byte excess not retained " + name);
        }
        JavaIr.parameterCount(4096);
        expect("JAVA_LOWERING_CFG", () -> JavaIr.parameterCount(4097));
        JavaLoweringValidation.checks(List.of(), List.of());
        expect("JAVA_LOWERING_CHECK_ORDER", () -> JavaLoweringValidation.checks(List.of("b", "a"), List.of("a", "b")));
    }
    private static void canonicalBytes() {
        Map<String, Object> value = Map.of("😀", "tab\tquote\"slash\\\n", "あ", List.of(-1, true, "é"), "a", "\u0001");
        byte[] wanted = (Protocol.json(value) + "\n").getBytes(StandardCharsets.UTF_8);
        byte[] actual = CanonicalJson.encode(value, "frontend_stdout", true);
        check(Arrays.equals(wanted, actual), "JCS escapes and UTF-16 key ordering");
        check(CanonicalJson.measure(value, "frontend_stdout", true) == actual.length, "exact UTF-8 count");
        check(CanonicalJson.measure(value, "frontend_stdout", false) == actual.length - 1, "LF counted only for stdout");
        check(CanonicalJson.hash("MPK-TEST", value, "frontend_stdout").equals(JavaContracts.typedHash("MPK-TEST", value)), "domain-NUL canonical hash");
        expect("JAVA_FRONTEND_INTERNAL", () -> CanonicalJson.encode("\ud800", "frontend_stdout", true));
    }
    private static Selection selection(Path path) throws Exception {
        var root = StrictJson.validate(Files.readAllBytes(path)).exact(Set.of("schema", "value"));
        check(root.get("schema").string().equals("mpk.selection.java_methods.v0"), "selection schema");
        var value = root.get("value").exact(Set.of("compilation", "sources", "contracts", "methods"));
        return new Selection(value.get("compilation").string(), strings(value.get("sources")), strings(value.get("contracts")), strings(value.get("methods")));
    }
    private static List<String> strings(StrictJson.Value value) {
        return value.elements(256, "JAVA_FRONTEND_INTERNAL").stream().map(StrictJson.Value::string).toList();
    }
}
