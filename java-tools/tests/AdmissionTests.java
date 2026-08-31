package mpk.java2vir;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

/** Private source/contract executor. This class is never packaged in the candidate. */
public final class AdmissionTests {
    private AdmissionTests() {}
    private static final Path FIXTURES = Path.of("/mpk/tests");
    private static int assertions;

    private static void check(boolean value, String label) {
        assertions++;
        if (!value) throw new AssertionError(label);
    }
    @FunctionalInterface private interface Operation { void run(); }
    private static FrontendFailure expect(String code, Operation operation) {
        try { operation.run(); }
        catch (FrontendFailure failure) {
            check(failure.code().equals(code), code + ": got " + failure.code());
            return failure;
        }
        throw new AssertionError("missing failure " + code);
    }
    private static Map<String, Object> failure(Selection selection, FrontendFailure failure) {
        return Map.of("code", failure.code(), "phase", failure.phase(), "status", failure.status(), "exit", failure.exitCode(),
                "envelope", new String(Protocol.failure(selection, failure), StandardCharsets.UTF_8));
    }

    public static void main(String[] arguments) throws Exception {
        check(arguments.length == 0, "no test runtime options");
        var cases = new ArrayList<Map<String, Object>>();
        CapturedSnapshot snapshotB = null;
        JavaAdmission.Program programA = null;
        for (String line : Files.readAllLines(FIXTURES.resolve("admission-cases.tsv"))) {
            String[] fields = line.split("\t", -1);
            String id = fields[0], group = fields[2], expected = fields[3], phase = fields[4];
            Path root = FIXTURES.resolve("admission/" + fields[1]);
            Selection selection = selection(root.resolve("selection.json"));
            JavaAdmission.Program program;
            CapturedSnapshot snapshot;
            try {
                snapshot = CapturedSnapshot.capture(root.resolve("snapshot"), selection);
                program = JavaAdmission.analyze(snapshot);
            } catch (FrontendFailure rejected) {
                check(rejected.code().equals(expected) && rejected.phase().equals(phase),
                        id + ": expected " + expected + "/" + phase + ", got " + rejected.code() + "/" + rejected.phase());
                var row = new java.util.TreeMap<String, Object>(failure(selection, rejected));
                row.put("id", id); row.put("group", group);
                cases.add(row);
                continue;
            }
            check(expected.equals("admitted"), id + ": missing " + expected);
            List<Map<String, Object>> methods = new ArrayList<>();
            for (JavaSubset.Method method : program.closure().methods()) {
                methods.add(Map.of("id", method.id(), "result", method.result().keyword, "callees", method.callees(),
                        "parameters", method.parameters().stream().map(p -> Map.of("name", p.name(), "type", p.type().keyword)).toList(),
                        "locals", method.locals().stream().map(p -> Map.of("name", p.name(), "type", p.type().keyword)).toList(),
                        "variable_bindings", method.variableBindings().entrySet().stream()
                                .sorted(java.util.Comparator.comparingLong(entry -> program.closure().origins().node(entry.getKey()).start()))
                                .map(entry -> {
                                    var node = program.closure().origins().node(entry.getKey());
                                    return Map.of("name", entry.getValue().name(), "type", entry.getValue().type().keyword,
                                            "parameter", entry.getValue().parameter(), "path", node.source().path(),
                                            "start", node.source().byteOffset(node.start()), "end", node.source().byteOffset(node.end()));
                                }).toList(),
                        "integer_literals", method.integers().values().stream().sorted().toList()));
            }
            List<Map<String, Object>> contracts = new ArrayList<>();
            for (JavaContracts.Attached attached : program.contracts().methods()) {
                contracts.add(Map.of("path", attached.path(), "raw_input_sha256", attached.rawInputSha256(),
                        "sidecar_sha256", attached.sidecarSha256(), "sidecar", attached.sidecar(),
                        "normalized", attached.normalized(), "nodes", attached.nodeCount()));
            }
            check(program.contracts().methods().size() == methods.size(), id + ": total attachment");
            check(program.contracts().nodeCount() == program.contracts().methods().stream().mapToLong(JavaContracts.Attached::nodeCount).sum(),
                    id + ": exact closure expression count");
            cases.add(Map.of("id", id, "group", group, "status", "admitted", "methods", methods,
                    "contracts", contracts, "selection_sha256", program.contracts().selectionSha256(),
                    "contract_nodes", program.contracts().nodeCount()));
            if (id.equals("link/source-a")) programA = program;
            if (id.equals("link/source-b")) snapshotB = snapshot;
        }
        if (programA == null || snapshotB == null) throw new AssertionError("missing link fixtures");
        JavaAdmission.Program previous = programA;
        CapturedSnapshot different = snapshotB;
        FrontendFailure link = expect("JAVA_CONTRACT_HASH", () -> JavaContracts.attach(different, previous.closure()));
        try { programA.closure().methods().clear(); throw new AssertionError("mutable methods"); }
        catch (UnsupportedOperationException expected) { assertions++; }
        try { programA.contracts().methods().getFirst().normalized().clear(); throw new AssertionError("mutable contract"); }
        catch (UnsupportedOperationException expected) { assertions++; }
        try { programA.closure().methods().getFirst().expressionTypes().clear(); throw new AssertionError("mutable facts"); }
        catch (UnsupportedOperationException expected) { assertions++; }
        counterBoundaries();
        var rules = new ArrayList<Map<String, Object>>();
        for (String line : Files.readAllLines(FIXTURES.resolve("conversion-rules.tsv"))) {
            String[] fields = line.split("\t");
            boolean accepted = ScalarType.conversion(ScalarType.keyword(fields[0]), ScalarType.keyword(fields[1]), fields[2]);
            check(accepted == Boolean.parseBoolean(fields[3]), "conversion " + line);
            rules.add(Map.of("source", fields[0], "target", fields[1], "context", fields[2], "accepted", accepted));
        }
        List<Map<String, Object>> mappings = new ArrayList<>();
        for (ScalarType type : ScalarType.values()) mappings.add(Map.of("source", type.keyword, "vir", type.vir()));
        System.out.write((Protocol.json(Map.of("schema", "mpk.java.admission_tests.v0", "cases", cases,
                "type_mappings", mappings, "conversion_rules", rules, "assertions", assertions,
                "counter_boundaries", List.of("contract_clauses", "contract_nodes_per_method", "contract_nodes_per_closure", "contract_depth"),
                "link_failure", failure(snapshotB.selection(), link))) + "\n").getBytes(StandardCharsets.UTF_8));
    }

    private static void counterBoundaries() {
        var all = new JavaContracts.ClosureCounter();
        var first = new JavaContracts.MethodCounter(all);
        for (int i = 0; i < 64; i++) first.clause();
        expect("JAVA_LIMIT_CONTRACT_CLAUSES", first::clause);
        for (int i = 0; i < 1024; i++) first.node(32);
        expect("JAVA_LIMIT_CONTRACT_NODES_PER_METHOD", () -> first.node(1));
        check(first.nodes() == 1024 && all.nodes() == 1024, "method excess never retained");
        for (int method = 0; method < 7; method++) {
            var next = new JavaContracts.MethodCounter(all);
            for (int i = 0; i < 1024; i++) next.node(1);
        }
        var last = new JavaContracts.MethodCounter(all);
        expect("JAVA_LIMIT_CONTRACT_NODES_PER_CLOSURE", () -> last.node(1));
        check(last.nodes() == 0 && all.nodes() == 8192, "closure excess never retained");
        expect("JAVA_LIMIT_CONTRACT_DEPTH", () -> last.node(33));
        check(last.nodes() == 0 && all.nodes() == 8192, "depth checked before retention");
    }

    private static Selection selection(Path path) throws Exception {
        var root = StrictJson.validate(Files.readAllBytes(path)).exact(Set.of("schema", "value"));
        check(root.get("schema").string().equals("mpk.selection.java_methods.v0"), "fixture selection schema");
        var value = root.get("value").exact(Set.of("compilation", "sources", "contracts", "methods"));
        return new Selection(value.get("compilation").string(), strings(value.get("sources")), strings(value.get("contracts")), strings(value.get("methods")));
    }
    private static List<String> strings(StrictJson.Value value) {
        return value.elements(256, "JAVA_FRONTEND_INTERNAL").stream().map(StrictJson.Value::string).toList();
    }
}
