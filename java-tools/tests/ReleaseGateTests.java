package mpk.java2vir;

import java.lang.reflect.InvocationTargetException;
import java.net.URLClassLoader;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;
import javax.tools.ToolProvider;

/** T09 private differential/fuzz executor. It is never packaged or installed. */
public final class ReleaseGateTests {
    private ReleaseGateTests() {}

    private static final Path FIXTURES = Path.of("/mpk/tests/release");
    private static final String SOURCE_PATH = "src/vector/Case.java";
    private static final String CONTRACT_PATH = "contracts/f.json";
    private static final String METHOD = "vector.Case::f(int)->int";
    private static final Selection SELECTION = new Selection("fuzz", List.of(SOURCE_PATH),
            List.of(CONTRACT_PATH), List.of(METHOD));
    private static final String VALID_SOURCE = "package vector;\npublic interface Case {\n"
            + "    public static int f(int x) { return x; }\n}\n";
    private static int assertions;

    @FunctionalInterface
    private interface Operation { void run() throws Exception; }

    private record Differential(String id, int caseIndex, String caseId, String method,
            String arguments, String outcome, String expected) {}

    private record FuzzProfile(String id, String seed, int iterations, String executor,
            String sequenceSha256) {}

    private static void check(boolean condition, String label) {
        assertions++;
        if (!condition) throw new AssertionError(label);
    }

    public static void main(String[] arguments) throws Exception {
        check(arguments.length == 0, "no private runtime options");
        var differential = differential();
        var fuzz = fuzz();
        var upgrades = Files.readAllLines(FIXTURES.resolve("upgrade-ids.txt"), StandardCharsets.US_ASCII);
        check(upgrades.size() == 12 && upgrades.stream().distinct().count() == 12, "upgrade case ownership");
        var report = Map.of("schema", "mpk.java.release_gate_tests.v0", "differential", differential,
                "fuzz", fuzz, "upgrade_case_ids", upgrades, "network_access", false,
                "production_source_execution", false, "assertions", assertions);
        System.out.write((Protocol.json(report) + "\n").getBytes(StandardCharsets.UTF_8));
    }

    private static List<Map<String, Object>> differential() throws Exception {
        var rows = new ArrayList<Map<String, Object>>();
        var records = Files.readAllLines(FIXTURES.resolve("differential.tsv"), StandardCharsets.US_ASCII)
                .stream().map(line -> {
                    String[] field = line.split("\t", -1);
                    check(field.length == 7, "differential fixture shape");
                    return new Differential(field[0], Integer.parseInt(field[1]), field[2], field[3],
                            field[4], field[5], field[6]);
                }).toList();
        int previous = -1;
        URLClassLoader loader = null;
        try {
            for (Differential record : records) {
                if (record.caseIndex() != previous) {
                    if (loader != null) loader.close();
                    Path sourceRoot = FIXTURES.resolve("differential/%03d".formatted(record.caseIndex()));
                    Path output = Path.of("/work/differential/%03d".formatted(record.caseIndex()));
                    Files.createDirectories(output);
                    compile(sourceRoot, output);
                    loader = new URLClassLoader(new java.net.URL[]{output.toUri().toURL()},
                            ClassLoader.getPlatformClassLoader());
                    previous = record.caseIndex();
                }
                rows.add(invoke(record, loader));
            }
        } finally {
            if (loader != null) loader.close();
        }
        check(rows.size() == 102, "complete differential corpus");
        return rows;
    }

    private static void compile(Path sourceRoot, Path output) throws Exception {
        var compiler = ToolProvider.getSystemJavaCompiler();
        check(compiler != null, "pinned system compiler available");
        List<Path> sources;
        try (var paths = Files.walk(sourceRoot)) {
            sources = paths.filter(path -> path.toString().endsWith(".java")).sorted().toList();
        }
        check(!sources.isEmpty(), "differential source inventory");
        var diagnostics = new ArrayList<Diagnostic<? extends JavaFileObject>>();
        try (var manager = compiler.getStandardFileManager(diagnostics::add, java.util.Locale.US,
                StandardCharsets.UTF_8)) {
            var options = List.of("--release", "25", "-encoding", "UTF-8", "-g:none", "-proc:none",
                    "-implicit:none", "-Xlint:all", "-Werror", "--class-path", "/work/empty",
                    "--source-path", "/work/empty", "--processor-path", "/work/empty",
                    "--module-path", "/work/empty", "-d", output.toString());
            boolean compiled = compiler.getTask(null, manager, diagnostics::add, options, null,
                    manager.getJavaFileObjectsFromPaths(sources)).call();
            check(compiled && diagnostics.isEmpty(), "independent Java compilation");
        }
    }

    private static Map<String, Object> invoke(Differential record, ClassLoader loader) throws Exception {
        int separator = record.method().indexOf("::");
        int open = record.method().indexOf('(', separator);
        int close = record.method().indexOf(')', open);
        String className = record.method().substring(0, separator);
        String methodName = record.method().substring(separator + 2, open);
        String parameterText = record.method().substring(open + 1, close);
        String[] names = parameterText.isEmpty() ? new String[0] : parameterText.split(",", -1);
        String[] values = record.arguments().isEmpty() ? new String[0] : record.arguments().split(",", -1);
        check(names.length == values.length, "differential argument arity");
        Class<?>[] types = new Class<?>[names.length];
        Object[] arguments = new Object[names.length];
        for (int index = 0; index < names.length; index++) {
            switch (names[index]) {
                case "boolean" -> { types[index] = boolean.class; arguments[index] = Boolean.parseBoolean(values[index]); }
                case "int" -> { types[index] = int.class; arguments[index] = Integer.parseInt(values[index]); }
                case "long" -> { types[index] = long.class; arguments[index] = Long.parseLong(values[index]); }
                default -> throw new AssertionError("uncompiled parameter type " + names[index]);
            }
        }
        var row = new TreeMap<String, Object>();
        row.put("id", record.id()); row.put("case_id", record.caseId()); row.put("method", record.method());
        try {
            Object result = loader.loadClass(className).getMethod(methodName, types).invoke(null, arguments);
            check(record.outcome().equals("result") && String.valueOf(result).equals(record.expected()),
                    "Java result " + record.id());
            row.put("result", String.valueOf(result));
        } catch (InvocationTargetException error) {
            String type = error.getCause().getClass().getName();
            check(record.outcome().equals("trap") && type.equals(record.expected()), "Java trap " + record.id());
            row.put("trap", type);
        }
        return row;
    }

    private static List<Map<String, Object>> fuzz() throws Exception {
        var profiles = Files.readAllLines(FIXTURES.resolve("fuzz.tsv"), StandardCharsets.US_ASCII)
                .stream().map(line -> {
                    String[] field = line.split("\t", -1);
                    check(field.length == 5, "fuzz fixture shape");
                    return new FuzzProfile(field[0], field[1], Integer.parseInt(field[2]), field[3], field[4]);
                }).toList();
        var rows = new ArrayList<Map<String, Object>>();
        for (FuzzProfile profile : profiles) {
            long[] states = states(profile);
            int rejected = switch (profile.id()) {
                case "source_decoder_parser" -> fuzzSource(states);
                case "contract_parser" -> fuzzContract(states);
                case "diagnostic_normalizer" -> fuzzDiagnostics(states);
                case "resource_capture" -> fuzzCapture(states);
                case "frontend_protocol" -> 0;
                default -> throw new AssertionError("unknown fuzz profile " + profile.id());
            };
            int cases = profile.executor().equals("rust_parent_validator") ? 0 : profile.iterations();
            check(rejected == cases, "closed fuzz profile " + profile.id());
            rows.add(Map.of("id", profile.id(), "seed", profile.seed(), "iterations", profile.iterations(),
                    "executor", profile.executor(), "sequence_sha256", profile.sequenceSha256(),
                    "cases", cases, "rejections", rejected));
        }
        return rows;
    }

    private static long[] states(FuzzProfile profile) throws Exception {
        long value = Long.parseUnsignedLong(profile.seed(), 16);
        long[] states = new long[profile.iterations()];
        var digest = MessageDigest.getInstance("SHA-256");
        for (int index = 0; index < states.length; index++) {
            value ^= value << 13;
            value ^= value >>> 7;
            value ^= value << 17;
            states[index] = value;
            digest.update(ByteBuffer.allocate(Long.BYTES).putLong(value).array());
        }
        check(HexFormat.of().formatHex(digest.digest()).equals(profile.sequenceSha256()),
                "fuzz sequence " + profile.id());
        return states;
    }

    private static int fuzzSource(long[] states) throws Exception {
        int rejected = 0;
        for (long state : states) {
            byte[] bytes;
            switch ((int) Long.remainderUnsigned(state, 4)) {
                case 0 -> bytes = ("\0" + VALID_SOURCE).getBytes(StandardCharsets.UTF_8);
                case 1 -> bytes = ("// " + "\\" + "u0041\n" + VALID_SOURCE).getBytes(StandardCharsets.UTF_8);
                case 2 -> bytes = VALID_SOURCE.substring(0, VALID_SOURCE.length() - 1).getBytes(StandardCharsets.UTF_8);
                default -> bytes = "package vector; public interface Case { static int f( {\n"
                        .getBytes(StandardCharsets.UTF_8);
            }
            try {
                var source = new SourceText(SOURCE_PATH, bytes);
                try (var session = CompilerSession.analyzeSources(List.of(source))) { session.units(); }
                throw new AssertionError("source fuzz admitted");
            } catch (FrontendFailure failure) {
                check(Set.of("source", "typecheck").contains(failure.phase()), "source fuzz phase");
                rejected++;
            }
        }
        return rejected;
    }

    private static int fuzzContract(long[] states) throws Exception {
        byte[] baseline = Files.readAllBytes(FIXTURES.resolve("fuzz-base/contract.json"));
        JavaAdmission.Program program;
        Path original = createSnapshot("contract-baseline", baseline);
        try { program = JavaAdmission.analyze(CapturedSnapshot.capture(original, SELECTION)); }
        finally { deleteTree(original); }
        int rejected = 0;
        for (int index = 0; index < states.length; index++) {
            long state = states[index];
            byte[] mutation;
            String text = new String(baseline, StandardCharsets.UTF_8);
            switch ((int) Long.remainderUnsigned(state, 4)) {
                case 0 -> mutation = "{\n".getBytes(StandardCharsets.UTF_8);
                case 1 -> mutation = ("{\"schema\":\"mpk.java.contract.v0\","
                        + "\"schema\":\"mpk.java.contract.v0\"}\n").getBytes(StandardCharsets.UTF_8);
                case 2 -> mutation = text.replace("mpk.java.scalar.v0", "mpk.java.scalar.v1")
                        .getBytes(StandardCharsets.UTF_8);
                default -> mutation = (text.substring(0, text.length() - 2) + ",\"fuzz\":"
                        + Long.toUnsignedString(state) + "}\n").getBytes(StandardCharsets.UTF_8);
            }
            Path root = createSnapshot("contract-" + index, mutation);
            try {
                var snapshot = CapturedSnapshot.capture(root, SELECTION);
                try { JavaContracts.attach(snapshot, program.closure()); throw new AssertionError("contract fuzz admitted"); }
                catch (FrontendFailure failure) { check(failure.phase().equals("subset"), "contract fuzz phase"); rejected++; }
            } finally { deleteTree(root); }
        }
        return rejected;
    }

    private static int fuzzDiagnostics(long[] states) throws Exception {
        var source = new SourceText(SOURCE_PATH, VALID_SOURCE.getBytes(StandardCharsets.UTF_8));
        int rejected = 0;
        for (long state : states) {
            var diagnostics = new CompilerDiagnostics(List.of(source));
            int selector = (int) Long.remainderUnsigned(state, 4);
            String code = selector == 0 ? "compiler.err.fuzz.unknown" : "compiler.err.premature.eof";
            Diagnostic.Kind kind = selector == 1 ? Diagnostic.Kind.WARNING : Diagnostic.Kind.ERROR;
            long start = selector == 2 ? Long.MAX_VALUE : 0;
            long end = selector == 2 ? Long.MAX_VALUE : 1;
            try {
                diagnostics.report(new FuzzDiagnostic(source, code, kind, start, end));
                diagnostics.finishPhase();
                throw new AssertionError("diagnostic fuzz admitted");
            } catch (FrontendFailure failure) {
                byte[] transport = Protocol.failure(SELECTION, failure);
                check(!new String(transport, StandardCharsets.UTF_8).contains("fuzz secret"),
                        "diagnostic prose redacted");
                rejected++;
            }
        }
        return rejected;
    }

    private static int fuzzCapture(long[] states) throws Exception {
        byte[] contract = Files.readAllBytes(FIXTURES.resolve("fuzz-base/contract.json"));
        int rejected = 0;
        for (int index = 0; index < states.length; index++) {
            Path root = createSnapshot("capture-" + index, contract);
            try {
                int selector = (int) Long.remainderUnsigned(states[index], 5);
                Path source = root.resolve(SOURCE_PATH);
                switch (selector) {
                    case 0 -> Files.writeString(root.resolve("extra.txt"), "extra\n");
                    case 1 -> Files.delete(root.resolve(CONTRACT_PATH));
                    case 2 -> {
                        Path target = root.resolve("target.java"); Files.writeString(target, VALID_SOURCE);
                        Files.delete(source); Files.createSymbolicLink(source, target);
                    }
                    case 3 -> Files.createLink(root.resolve("alias.java"), source);
                    default -> Files.write(root.resolve(CONTRACT_PATH), new byte[1_048_577]);
                }
                try { CapturedSnapshot.capture(root, SELECTION); throw new AssertionError("capture fuzz admitted"); }
                catch (FrontendFailure failure) { check(failure.phase().equals("capture"), "capture fuzz phase"); rejected++; }
            } finally { deleteTree(root); }
        }
        return rejected;
    }

    private static Path createSnapshot(String prefix, byte[] contract) throws Exception {
        Path root = Files.createTempDirectory(Path.of("/work"), "t09-" + prefix + "-");
        Path source = root.resolve(SOURCE_PATH), sidecar = root.resolve(CONTRACT_PATH);
        Files.createDirectories(source.getParent()); Files.createDirectories(sidecar.getParent());
        Files.writeString(source, VALID_SOURCE, StandardCharsets.UTF_8);
        Files.write(sidecar, contract);
        return root;
    }

    private static void deleteTree(Path root) throws Exception {
        if (!Files.exists(root)) return;
        try (var paths = Files.walk(root)) {
            for (Path path : paths.sorted(Comparator.reverseOrder()).toList()) Files.delete(path);
        }
    }

    private record FuzzDiagnostic(JavaFileObject source, String code, Diagnostic.Kind kind,
            long start, long end) implements Diagnostic<JavaFileObject> {
        @Override public JavaFileObject getSource() { return source; }
        @Override public long getPosition() { return start; }
        @Override public long getStartPosition() { return start; }
        @Override public long getEndPosition() { return end; }
        @Override public long getLineNumber() { return 1; }
        @Override public long getColumnNumber() { return 1; }
        @Override public String getCode() { return code; }
        @Override public Diagnostic.Kind getKind() { return kind; }
        @Override public String getMessage(java.util.Locale locale) { return "fuzz secret /host/path"; }
    }
}
