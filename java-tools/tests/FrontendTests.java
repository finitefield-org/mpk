package mpk.java2vir;

import com.sun.source.tree.*;
import com.sun.source.util.JavacTask;
import java.lang.annotation.Annotation;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import javax.lang.model.element.AnnotationMirror;
import javax.lang.model.type.TypeKind;
import javax.lang.model.type.TypeMirror;
import javax.lang.model.type.TypeVisitor;
import javax.tools.Diagnostic;
import javax.tools.JavaFileManager;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.StandardLocation;
import javax.tools.ToolProvider;

/** Private executable tests, compiled separately and never packaged in java2vir.jar. */
public final class FrontendTests {
    private static final Path FIXTURES = Path.of("/mpk/tests");
    private static final String SOURCE = "src/vector/Case.java";
    private static final String CONTRACT = "contracts/f.json";
    private static final Selection SELECTION = new Selection("vector", List.of(SOURCE), List.of(CONTRACT), List.of("vector.Case::f(int)->int"));
    private static final Selection PROBE_SELECTION = new Selection("probe", List.of("src/demo/Probe.java"), List.of("contracts/probe.json"), List.of("demo.Probe::f()->int"));
    private static final List<Map<String, Object>> FAILURES = new ArrayList<>();
    private static final List<String> CHECKS = new ArrayList<>();
    private static byte[] baseline;
    private FrontendTests() {}

    @FunctionalInterface private interface Operation { void run() throws Exception; }
    private static void check(boolean condition, String name) {
        if (!condition) throw new AssertionError(name);
        CHECKS.add(name);
    }
    private static FrontendFailure expect(String id, String code, String phase, Operation operation) throws Exception {
        return expect(id, code, phase, SELECTION, operation);
    }
    private static FrontendFailure expect(String id, String code, String phase, Selection selection, Operation operation) throws Exception {
        try { operation.run(); }
        catch (FrontendFailure failure) {
            check(failure.code().equals(code) && failure.phase().equals(phase), id + ":" + failure.code() + ":" + failure.phase());
            failure(id, selection, failure);
            return failure;
        }
        throw new AssertionError("failure missing: " + id);
    }
    private static Map<String, Object> failure(String id, Selection selection, FrontendFailure failure) {
        var row = Map.<String, Object>of("id", id, "code", failure.code(), "phase", failure.phase(),
                "status", failure.status(), "exit", failure.exitCode(), "issues", failure.issues().size(),
                "envelope", new String(Protocol.failure(selection, failure), StandardCharsets.UTF_8));
        FAILURES.add(row);
        return row;
    }
    private static SourceText source(String text) { return new SourceText(SOURCE, text.getBytes(StandardCharsets.UTF_8)); }

    public static void main(String[] arguments) throws Exception {
        check(arguments.length == 0, "no private runtime options");
        baseline = Files.readAllBytes(FIXTURES.resolve("baseline.java.txt"));
        Locale.setDefault(Locale.JAPAN); // The adapter's explicit US locale must win.
        var observations = observations();
        multipleSources();
        var boundary = fileManager();
        transport();
        capture();
        diagnostics();
        var limits = limits();
        precedence();
        var definitions = new ArrayList<Map<String, Object>>();
        for (var entry : new TreeMap<>(DiagnosticRegistry.DEFINITIONS).entrySet()) {
            var definition = entry.getValue();
            definitions.add(Map.of("code", entry.getKey(), "status", definition.status(), "phase", definition.phase(),
                    "exit", definition.exitCode(), "message", definition.message()));
        }
        check(System.getProperty("mpk.java.test.processor") == null, "planted processor never loaded");
        var report = Map.of("schema", "mpk.java.frontend_tests.v0", "observations", observations,
                "file_manager_boundary_checks", boundary, "failures", FAILURES, "checks", CHECKS,
                "limits", limits, "diagnostic_registry", definitions, "compiler_options", CompilerSession.OPTIONS,
                "compiler_codes", CompilerDiagnostics.CODES.stream().sorted().toList());
        System.out.write((Protocol.json(report) + "\n").getBytes(StandardCharsets.UTF_8));
    }

    private static List<Map<String, Object>> observations() throws Exception {
        var observations = new ArrayList<Map<String, Object>>();
        for (String id : Files.readAllLines(FIXTURES.resolve("observation-ids.txt"))) {
            byte[] bytes = Files.readAllBytes(FIXTURES.resolve("observations/" + id + ".java.txt"));
            CompilerSession session;
            try { session = CompilerSession.analyzeSources(List.of(new SourceText("src/demo/Probe.java", bytes))); }
            catch (FrontendFailure error) {
                observations.add(failure("observation/" + id, PROBE_SELECTION, error));
                continue;
            }
            var row = new TreeMap<String, Object>();
            try (session) {
                row.put("id", id); row.put("status", "analyzed");
                row.put("before_analysis", trees(session.before(), false));
                row.put("after_analysis", trees(session.after(), true));
                row.put("system_files_returned", session.manager().systemFiles());
                row.put("output_attempts", session.manager().outputAttempts());
                if (id.startsWith("excluded-")) {
                    // Admission belongs to T05. Inventorying a synthesized child must
                    // not turn an excluded parent into a premature adapter error.
                    check(session.after().nodes().size() > session.before().nodes().size(), id + " changes inventoried only");
                    expect("raw-comparison/" + id, "JAVA_TOOLCHAIN_ADAPTER", "typecheck",
                            () -> session.before().requireUnchanged(session.units().getFirst(), session.after()));
                } else {
                    session.before().requireUnchanged(session.units().getFirst(), session.after());
                }
                var original = session.before().nodes().getFirst().source();
                var forged = new SimpleJavaFileObject(original.toUri(), JavaFileObject.Kind.SOURCE) {
                    @Override public CharSequence getCharContent(boolean ignore) { return new String(original.text()); }
                };
                expect("tree-origin/" + id, "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () ->
                        new TreeInventory.Origins(List.of(new ForeignUnit(session.units().getFirst(), forged)), List.of(original)));
            }
            check(session.manager().closed(), id + " manager closed");
            expect("closed-session/" + id, "JAVA_TOOLCHAIN_ADAPTER", "typecheck", session::units);
            row.put("manager_closed", true);
            observations.add(row);
        }
        return observations;
    }

    private static List<Map<String, Object>> trees(TreeInventory inventory, boolean analyzed) {
        var rows = new ArrayList<Map<String, Object>>();
        for (var node : inventory.nodes()) {
            var row = new TreeMap<String, Object>();
            row.put("kind", node.kind().name()); row.put("start_utf16", node.start()); row.put("end_utf16", node.end());
            if (node.start() >= 0 && node.end() >= node.start() && node.end() <= node.source().text().length()) {
                row.put("spelling", node.source().text().substring((int) node.start(), (int) node.end()));
                row.put("start_utf8", node.source().byteOffset(node.start())); row.put("end_utf8", node.source().byteOffset(node.end()));
            }
            if (node.facts().containsKey("literal")) row.put("literal_value", node.facts().get("literal"));
            if (node.facts().containsKey("flags")) row.put("flags", node.facts().get("flags"));
            if (analyzed) {
                row.put("type", node.type() == null ? null : node.type().toString());
                row.put("element", node.element() == null ? null : node.element().toString());
                row.put("element_kind", node.element() == null ? null : node.element().getKind().name());
                row.put("element_modifiers", node.element() == null ? null : node.element().getModifiers().stream().map(Object::toString).sorted().toList());
            }
            rows.add(row);
        }
        return rows;
    }

    private static void multipleSources() throws Exception {
        var a = new SourceText("src/vector/A.java", "package vector; public interface A { static int f(int x) { return x; } }\n".getBytes(StandardCharsets.UTF_8));
        var b = new SourceText("src/vector/B.java", "package vector; public interface B { static int g(int x) { return A.f(x); } }\n".getBytes(StandardCharsets.UTF_8));
        try (var session = CompilerSession.analyzeSources(List.of(a, b))) {
            check(session.units().size() == 2, "all selected source units analyzed");
            check(session.before().node(session.units().get(0)).source() == a
                    && session.before().node(session.units().get(1)).source() == b, "selection source order preserved");
            for (var unit : session.units()) session.before().requireUnchanged(unit, session.after());
            check(session.elements().getTypeElement("mpk.java2vir.Main") == null, "frontend JAR not an analyzed dependency");
        }
        var selectedB = new Selection("multi", List.of(b.path()), List.of(CONTRACT), List.of("vector.B::g(int)->int"));
        expect("session/no-previous-source", "JAVA_SOURCE_DIAGNOSTIC", "typecheck", selectedB, () -> {
            try (var session = CompilerSession.analyzeSources(List.of(b))) { session.units(); }
        });
        expect("session/source-order", "JAVA_CAPTURE_PATH", "capture", () -> {
            try (var session = CompilerSession.analyzeSources(List.of(b, a))) { session.units(); }
        });
    }

    private static Map<String, Boolean> fileManager() throws Exception {
        var source = new SourceText("src/demo/Probe.java", "package demo; public interface Probe { static int f() {return 1;} }\n".getBytes(StandardCharsets.UTF_8));
        var compiler = ToolProvider.getSystemJavaCompiler();
        // Positive controls prove the poisoned dependencies really are available.
        for (String expression : List.of("Hidden.value()", "poison.Injected.value()")) {
            var errors = new ArrayList<Diagnostic<? extends JavaFileObject>>();
            try (var base = compiler.getStandardFileManager(errors::add, Locale.US, StandardCharsets.UTF_8)) {
                base.setLocationFromPaths(StandardLocation.CLASS_PATH, List.of(Path.of("/work/poison-classes")));
                base.setLocationFromPaths(StandardLocation.SOURCE_PATH, List.of(FIXTURES.resolve("poison-source")));
                var input = new SourceText("src/demo/Probe.java", ("package demo; public interface Probe { static int f() { return " + expression + "; } }\n").getBytes(StandardCharsets.UTF_8));
                var task = (JavacTask) compiler.getTask(new java.io.StringWriter(), base, errors::add, CompilerSession.OPTIONS, null, List.of(input));
                task.parse(); task.analyze();
                check(errors.isEmpty(), "poison control resolves " + expression);
            }
        }
        check(ClassLoader.getSystemResource("META-INF/services/javax.annotation.processing.Processor") != null, "processor service planted");
        var base = compiler.getStandardFileManager(null, Locale.US, StandardCharsets.UTF_8);
        base.setLocationFromPaths(StandardLocation.MODULE_PATH, List.of(Path.of("/work/poison-modules")));
        check(base.getLocationForModule(StandardLocation.MODULE_PATH, "external.poison") != null, "module control resolves");
        var result = new TreeMap<String, Boolean>();
        try (var manager = new ClosedFileManager(base, List.of(source))) {
            var app = StandardLocation.CLASS_PATH;
            var module = StandardLocation.MODULE_PATH;
            result.put("application_list_empty", !manager.list(app, "poison", Set.of(JavaFileObject.Kind.CLASS), true).iterator().hasNext());
            result.put("application_java_input_absent", manager.getJavaFileForInput(app, "poison.Injected", JavaFileObject.Kind.CLASS) == null);
            result.put("application_resource_input_absent", manager.getFileForInput(app, "poison", "injected.txt") == null);
            result.put("application_binary_name_absent", manager.inferBinaryName(app, source) == null);
            result.put("application_contains_false", !manager.contains(app, source));
            result.put("application_classloader_absent", manager.getClassLoader(app) == null);
            result.put("application_module_name_absent", manager.inferModuleName(module) == null);
            result.put("application_module_locations_empty", !manager.listLocationsForModules(module).iterator().hasNext());
            result.put("application_module_by_name_absent", manager.getLocationForModule(module, "external.poison") == null);
            result.put("application_module_by_file_absent", manager.getLocationForModule(module, source) == null);
            expect("manager/service", "JAVA_TOOLCHAIN_FILE_MANAGER", "metadata", () -> manager.getServiceLoader(app, javax.annotation.processing.Processor.class));
            result.put("service_loader_refused", true);
            var external = new SimpleJavaFileObject(URI.create("file:///unselected/Injected.class"), JavaFileObject.Kind.CLASS) {};
            expect("compiler.external_lookup", "JAVA_TOOLCHAIN_FILE_MANAGER", "metadata", () -> manager.verifySystem(external));
            for (var location : List.of(StandardLocation.SOURCE_PATH, StandardLocation.MODULE_SOURCE_PATH,
                    StandardLocation.ANNOTATION_PROCESSOR_PATH, StandardLocation.ANNOTATION_PROCESSOR_MODULE_PATH,
                    StandardLocation.UPGRADE_MODULE_PATH, StandardLocation.PATCH_MODULE_PATH)) {
                check(!manager.hasLocation(location) && !manager.list(location, "", Set.of(JavaFileObject.Kind.SOURCE), true).iterator().hasNext(), "closed " + location);
            }
            var unknown = new JavaFileManager.Location() {
                @Override public String getName() { return "SYSTEM_MODULES"; }
                @Override public boolean isOutputLocation() { return false; }
            };
            check(!manager.hasLocation(unknown), "location identity cannot be forged by name");
            check(!manager.handleOption("--class-path", List.of("/unselected").iterator()), "unknown options never delegated");
            expect("manager/options", "JAVA_TOOLCHAIN_OPTIONS", "metadata", () -> manager.handleOption("-encoding", List.of("UTF-16").iterator()));
            manager.phase("typecheck");
            expect("compiler.unexpected_output", "JAVA_TOOLCHAIN_FILE_MANAGER", "typecheck", () -> manager.getJavaFileForOutput(StandardLocation.CLASS_OUTPUT, "Probe", JavaFileObject.Kind.CLASS, source));
            result.put("java_output_refused", true);
            expect("manager/resource-output", "JAVA_TOOLCHAIN_FILE_MANAGER", "typecheck", () -> manager.getFileForOutput(StandardLocation.CLASS_OUTPUT, "", "output", source));
            result.put("resource_output_refused", true);
            expect("manager/originating-java-output", "JAVA_TOOLCHAIN_FILE_MANAGER", "typecheck", () -> manager.getJavaFileForOutputForOriginatingFiles(StandardLocation.CLASS_OUTPUT, "Probe", JavaFileObject.Kind.CLASS, source));
            result.put("originating_java_output_refused", true);
            expect("manager/originating-resource-output", "JAVA_TOOLCHAIN_FILE_MANAGER", "typecheck", () -> manager.getFileForOutputForOriginatingFiles(StandardLocation.CLASS_OUTPUT, "", "output", source));
            result.put("originating_resource_output_refused", true);
            check(manager.outputAttempts() == 4, "all output attempts counted");
        }
        check(result.values().stream().allMatch(Boolean::booleanValue), "all file manager boundaries closed");
        expect("module/not-discovered", "JAVA_SOURCE_DIAGNOSTIC", "typecheck", () -> {
            try (var session = CompilerSession.analyzeSources(List.of(source("package vector; public interface Case { static int f() { return poisonmodule.Api.value(); } }\n")))) { session.units(); }
        });
        return result;
    }

    private static void transport() throws Exception {
        for (String id : Files.readAllLines(FIXTURES.resolve("encoding-ids.txt"))) {
            byte[] bytes = Files.readAllBytes(FIXTURES.resolve("encoding/" + id + ".bin"));
            expect(id, "JAVA_SOURCE_ENCODING", "source", () -> new SourceText(SOURCE, bytes));
        }
        for (byte[] bytes : List.of(new byte[0], new byte[]{(byte) 0xed, (byte) 0xa0, (byte) 0x80, 10},
                new byte[]{(byte) 0xf4, (byte) 0x90, (byte) 0x80, (byte) 0x80, 10}, new byte[]{(byte) 0xe2, 10}))
            expect("encoding/invalid-scalar-" + Arrays.toString(bytes), "JAVA_SOURCE_ENCODING", "source", () -> new SourceText(SOURCE, bytes));
        for (int scalar : List.of(0xfdd0, 0xfdef, 0xfffe, 0x1ffff, 0x10fffe, 0x10ffff))
            expect("encoding/noncharacter-" + scalar, "JAVA_SOURCE_ENCODING", "source", () -> source("// " + new String(Character.toChars(scalar)) + "\n"));
        expect("encoding/double-backslash-u", "JAVA_SOURCE_ENCODING", "source", () -> source("// " + "\\" + "\\" + "u0041\n"));
        var text = source("\t// あ😀\nx\n");
        check(text.byteOffset(0) == 0 && text.byteOffset(1) == 1 && text.byteOffset(5) == 7 && text.byteOffset(7) == 11, "UTF16 to original UTF8 with tabs BMP and nonBMP");
        expect("diagnostic/split-surrogate", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> text.span(6, 7, true, "typecheck"));
        for (long[] span : List.of(new long[]{-1, 1}, new long[]{2, 1}, new long[]{0, 100}, new long[]{Long.MAX_VALUE, Long.MAX_VALUE}))
            expect("diagnostic/range-" + span[0], "JAVA_TOOLCHAIN_ADAPTER", "source", () -> text.span(span[0], span[1], true, "source"));
        check(text.span(7, 7, true, "typecheck") == null, "zero length diagnostic validated and omitted");
        expect("diagnostic/nonempty-origin", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> text.span(7, 7, false, "typecheck"));
        byte[] mutable = baseline.clone();
        var captured = new SourceText(SOURCE, mutable); mutable[0] = 0;
        check(captured.text().startsWith("package"), "source text immutable after decode");
    }

    private static Path snapshot() throws Exception {
        Path root = Files.createTempDirectory(Path.of("/work"), "snapshot-");
        Files.createDirectories(root.resolve("src/vector")); Files.createDirectory(root.resolve("contracts"));
        Files.write(root.resolve(SOURCE), baseline);
        Files.write(root.resolve(CONTRACT), Files.readAllBytes(FIXTURES.resolve("baseline-contract.json.txt")));
        return root;
    }
    private static void capture() throws Exception {
        Path valid = snapshot();
        var captured = CapturedSnapshot.capture(valid, SELECTION);
        check(captured.inputs().size() == 2 && captured.sources().size() == 1, "capture exact selected files");
        byte[] copy = captured.file(SOURCE).bytes(); copy[0] = 0;
        Files.write(valid.resolve(SOURCE), "changed\n".getBytes(StandardCharsets.UTF_8));
        check(Arrays.equals(captured.file(SOURCE).bytes(), baseline), "capture owns immutable bytes");
        check(captured.file(SOURCE).sha256().length() == 64, "raw source hash available");
        try (var session = CompilerSession.analyze(captured)) { check(!session.before().nodes().isEmpty(), "captured source compiles"); }
        Path symlink = snapshot(); Files.delete(symlink.resolve(SOURCE)); Files.createSymbolicLink(symlink.resolve(SOURCE), valid.resolve(SOURCE));
        expect("capture.symlink", "JAVA_CAPTURE_FILE_TYPE", "capture", () -> CapturedSnapshot.capture(symlink, SELECTION));
        Path hardlink = snapshot(); Files.createLink(hardlink.resolve("src/vector/Other.java"), hardlink.resolve(SOURCE));
        var aliases = new Selection("vector", List.of(SOURCE, "src/vector/Other.java"), List.of(CONTRACT), SELECTION.methods());
        expect("capture.hardlink", "JAVA_CAPTURE_FILE_TYPE", "capture", () -> CapturedSnapshot.capture(hardlink, aliases));
        Path unlisted = snapshot(); Files.write(unlisted.resolve("src/vector/Unlisted.java"), baseline);
        expect("capture.unlisted", "JAVA_CAPTURE_INVENTORY", "capture", () -> CapturedSnapshot.capture(unlisted, SELECTION));
        Path collision = snapshot(); Files.write(collision.resolve("src/vector/case.java"), baseline);
        expect("capture.case_collision", "JAVA_CAPTURE_PATH", "capture", () -> CapturedSnapshot.capture(collision, SELECTION));
        Path missing = snapshot(); Files.delete(missing.resolve(CONTRACT));
        expect("capture/missing", "JAVA_CAPTURE_INVENTORY", "capture", () -> CapturedSnapshot.capture(missing, SELECTION));
        Path ancestor = Path.of("/work/symlink-root"); Files.createSymbolicLink(ancestor, valid);
        expect("capture/ancestor-link", "JAVA_CAPTURE_FILE_TYPE", "capture", () -> CapturedSnapshot.capture(ancestor, SELECTION));
        Path special = snapshot(); Files.delete(special.resolve(SOURCE));
        var fifo = new ProcessBuilder("/usr/bin/mkfifo", special.resolve(SOURCE).toString()).start();
        check(fifo.waitFor() == 0, "fifo fixture created");
        expect("capture/fifo", "JAVA_CAPTURE_FILE_TYPE", "capture", () -> CapturedSnapshot.capture(special, SELECTION));
        Path oversized = snapshot();
        try (var file = new java.io.RandomAccessFile(oversized.resolve(SOURCE).toFile(), "rw")) { file.setLength(1048577); }
        expect("capture/source-byte-overflow", "JAVA_LIMIT_SOURCE_FILE_BYTES", "capture", () -> CapturedSnapshot.capture(oversized, SELECTION));
        for (String path : List.of("../Case.java", "src/Case.java", "src/java/Case.java", "src/com/sun/Case.java", "src/vector/CON.java", "src/vector/A B.java", "src/vector/_foo$/Case.java"))
            expect("selection/" + path, "JAVA_CAPTURE_PATH", "capture", () -> new Selection("vector", List.of(path), List.of(CONTRACT), SELECTION.methods()));
        expect("selection/duplicates", "JAVA_CAPTURE_PATH", "capture", () -> new Selection("vector", List.of(SOURCE, SOURCE), List.of(CONTRACT), SELECTION.methods()));
        var sources = java.util.stream.IntStream.range(0, 257).mapToObj(i -> "src/vector/Case" + String.format(Locale.ROOT, "%03d", i) + ".java").toList();
        new Selection("vector", sources.subList(0, 256), List.of(CONTRACT), SELECTION.methods());
        expect("selection/source-count", "JAVA_LIMIT_SOURCE_FILES", "capture", () -> new Selection("vector", sources, List.of(CONTRACT), SELECTION.methods()));
        var contracts = java.util.stream.IntStream.range(0, 129).mapToObj(i -> "contracts/f" + String.format(Locale.ROOT, "%03d", i) + ".json").toList();
        new Selection("vector", List.of(SOURCE), contracts.subList(0, 128), SELECTION.methods());
        expect("selection/contract-count", "JAVA_LIMIT_CONTRACT_FILES", "capture", () -> new Selection("vector", List.of(SOURCE), contracts, SELECTION.methods()));
        var methods = java.util.stream.IntStream.range(0, 33).mapToObj(i -> "vector.Case::f" + String.format(Locale.ROOT, "%02d", i) + "(int)->int").toList();
        new Selection("vector", List.of(SOURCE), List.of(CONTRACT), methods.subList(0, 32));
        expect("selection/method-count", "JAVA_LIMIT_SELECTED_METHODS", "capture", () -> new Selection("vector", List.of(SOURCE), List.of(CONTRACT), methods));
        for (boolean invalidFirst : List.of(true, false)) {
            Path excessive = snapshot();
            if (invalidFirst) Files.createFile(excessive.resolve("invalid name"));
            for (int i = 0; i < 512; i++) Files.createFile(excessive.resolve("entry" + i));
            if (!invalidFirst) Files.createFile(excessive.resolve("invalid name"));
            expect("capture/entry-limit-order-" + invalidFirst, "JAVA_LIMIT_SNAPSHOT_ENTRIES", "capture", () -> CapturedSnapshot.capture(excessive, SELECTION));
        }
    }

    private record TestDiagnostic(JavaFileObject source, String code, Kind kind, long start, long end) implements Diagnostic<JavaFileObject> {
        @Override public JavaFileObject getSource() { return source; }
        @Override public String getCode() { return code; }
        @Override public Kind getKind() { return kind; }
        @Override public long getStartPosition() { return start; }
        @Override public long getEndPosition() { return end; }
        @Override public long getPosition() { throw new AssertionError("position not used"); }
        @Override public long getLineNumber() { throw new AssertionError("line not used"); }
        @Override public long getColumnNumber() { throw new AssertionError("column not used"); }
        @Override public String getMessage(Locale locale) { throw new AssertionError("compiler prose must never be requested"); }
    }
    private static void diagnostics() throws Exception {
        var source = new SourceText(SOURCE, baseline);
        var known = new TestDiagnostic(source, "compiler.err.prob.found.req", Diagnostic.Kind.ERROR, 0, 1);
        for (String code : CompilerDiagnostics.CODES.stream().sorted().toList()) for (var kind : Diagnostic.Kind.values()) {
            var collector = new CompilerDiagnostics(List.of(source));
            var diagnostic = new TestDiagnostic(source, code, kind, 0, 1);
            if (kind == Diagnostic.Kind.ERROR) {
                collector.report(diagnostic);
                expect("diagnostic/known/" + code, "JAVA_SOURCE_PARSE", "source", collector::finishPhase);
            } else expect("diagnostic/kind/" + code + "/" + kind, "JAVA_TOOLCHAIN_ADAPTER", "source", () -> collector.report(diagnostic));
        }
        var unknown = new CompilerDiagnostics(List.of(source));
        expect("compiler.unknown_diagnostic", "JAVA_TOOLCHAIN_ADAPTER", "source", () -> unknown.report(new TestDiagnostic(source, "compiler.err.unknown", Diagnostic.Kind.ERROR, 0, 1)));
        expect("diagnostic/null-code", "JAVA_TOOLCHAIN_ADAPTER", "source", () -> new CompilerDiagnostics(List.of(source)).report(new TestDiagnostic(source, null, Diagnostic.Kind.ERROR, 0, 1)));
        var provenance = new CompilerDiagnostics(List.of(source)); provenance.phase("typecheck");
        expect("compiler.external_source", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> provenance.report(new TestDiagnostic(new SourceText(SOURCE, baseline), known.code(), known.kind(), 0, 1)));
        expect("diagnostic/no-source", "JAVA_TOOLCHAIN_ADAPTER", "source", () -> new CompilerDiagnostics(List.of(source)).report(new TestDiagnostic(null, known.code(), known.kind(), 0, 1)));
        var ordered = new CompilerDiagnostics(List.of(source));
        ordered.report(new TestDiagnostic(source, known.code(), known.kind(), 8, 9));
        ordered.report(new TestDiagnostic(source, known.code(), known.kind(), 1, 3));
        ordered.report(new TestDiagnostic(source, "compiler.err.premature.eof", known.kind(), 1, 2));
        ordered.report(new TestDiagnostic(source, known.code(), known.kind(), 1, 3));
        ordered.report(new TestDiagnostic(source, known.code(), known.kind(), 10, 10));
        check(ordered.raw().stream().map(CompilerDiagnostics.Raw::ordinal).toList().equals(List.of(3L, 2L, 4L, 1L, 5L)), "raw diagnostic sort and arrival ordinal");
        var failure = expect("diagnostic/public-order", "JAVA_SOURCE_PARSE", "source", ordered::finishPhase);
        check(failure.issues().getFirst().span() == null && failure.issues().get(1).span().end() == 2, "public diagnostic sort absent sentinels");
        var budget = new CompilerDiagnostics(List.of(source));
        for (int i = 0; i < 1024; i++) budget.report(known);
        check(budget.raw().size() == 1024, "1024 callbacks retained");
        expect("precedence/diagnostic_overflow_beats_source", "JAVA_FRONTEND_DIAGNOSTIC_BUDGET", "source", () -> budget.report(known));
        check(budget.raw().size() == 1024, "1025th callback not retained");
        Tree unknownTree = new Tree() {
            @Override public Kind getKind() { return Kind.OTHER; }
            @Override public <R, D> R accept(TreeVisitor<R, D> visitor, D data) { return visitor.visitOther(this, data); }
        };
        expect("compiler.unknown_tree", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> TreeInventory.requireKnownKind(unknownTree));
        expect("compiler.error_type", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> TreeInventory.requireKnownType(new UnknownType()));
        expect("compiler/missing-type", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> TreeInventory.requireKnownType(null));
        var errors = new CompilerDiagnostics(List.of(source)); errors.phase("typecheck"); errors.report(known);
        expect("precedence/compiler_exception_beats_source", "JAVA_TOOLCHAIN_ADAPTER", "typecheck", () -> {
            // Same normalization path used by CompilerSession's catch before finishPhase.
            try { throw new IllegalStateException("unselected host path /secret/compiler detail"); }
            catch (RuntimeException error) { throw FrontendFailure.compilerFailure(error, "typecheck"); }
        });
        check(FrontendFailure.compilerFailure(new RuntimeException(FrontendFailure.of("JAVA_FRONTEND_DIAGNOSTIC_BUDGET", "typecheck")), "typecheck").code().equals("JAVA_FRONTEND_DIAGNOSTIC_BUDGET"), "wrapped callback budget preserved");
        check(FrontendFailure.compilerFailure(new OutOfMemoryError("private"), "source").code().equals("JAVA_FRONTEND_RESOURCE"), "VM resource failure normalized");
        check(FrontendFailure.compilerFailure(new AssertionError("private"), "source").code().equals("JAVA_TOOLCHAIN_ADAPTER"), "compiler assertion redacted");
    }

    private static List<Map<String, Object>> limits() throws Exception {
        var rows = new ArrayList<Map<String, Object>>();
        for (String line : Files.readAllLines(FIXTURES.resolve("limits.tsv"))) {
            String[] fields = line.split("\t"); String id = fields[0], phase = fields[2], code = fields[3];
            long maximum = Long.parseLong(fields[1]);
            check(FrontendLimits.DEFINITIONS.get(id).maximum() == maximum, "frozen counter " + id);
            check(FrontendLimits.add(id, maximum - 1, 1, phase) == maximum, "counter boundary " + id);
            expect("limit/" + id, code, phase, () -> FrontendLimits.add(id, maximum, 1, phase));
            expect("limit/overflow-arithmetic/" + id, code, phase, () -> FrontendLimits.add(id, Long.MAX_VALUE, 1, phase));
            rows.add(Map.of("id", id, "maximum", maximum, "code", code, "phase", phase));
        }
        FrontendLimits.arguments(new String[]{"a".repeat(131071)});
        FrontendLimits.arguments(new String[]{"あ".repeat(43690), ""});
        expect("argument/overflow", "JAVA_LIMIT_FRONTEND_ARGUMENT_BYTES", "capture", () -> FrontendLimits.arguments(new String[]{"a".repeat(131072)}));
        expect("argument/utf8-overflow", "JAVA_LIMIT_FRONTEND_ARGUMENT_BYTES", "capture", () -> FrontendLimits.arguments(new String[]{"あ".repeat(43691)}));
        // Exercise the actual public-tree child collector, with already retained
        // siblings accounting for the rest of the inclusive node/depth budget.
        BlockTree block = new BlockTree() {
            @Override public boolean isStatic() { return false; }
            @Override public List<? extends StatementTree> getStatements() { return List.of(this); }
            @Override public Kind getKind() { return Kind.BLOCK; }
            @Override public <R, D> R accept(TreeVisitor<R, D> visitor, D data) { return visitor.visitBlock(this, data); }
        };
        check(TreeInventory.children(block, 249999, 255, "source").size() == 1, "syntax collector inclusive boundary");
        expect("syntax/aggregate-budget", "JAVA_LIMIT_SYNTAX_NODES", "source", () -> TreeInventory.children(block, 250000, 1, "source"));
        expect("syntax/depth-budget", "JAVA_LIMIT_SYNTAX_DEPTH", "source", () -> TreeInventory.children(block, 1, 256, "source"));
        expect("syntax/after-attribution-budget", "JAVA_LIMIT_SYNTAX_NODES", "source", () -> TreeInventory.children(block, 250000, 1, "typecheck"));
        return rows;
    }

    private static void precedence() throws Exception {
        Path root = snapshot(); Files.write(root.resolve(SOURCE), new byte[]{(byte) 0xff, 10}); Files.writeString(root.resolve("unlisted.txt"), "extra");
        expect("precedence/capture_before_encoding", "JAVA_CAPTURE_INVENTORY", "capture", () -> { try (var session = CompilerSession.analyze(CapturedSnapshot.capture(root, SELECTION))) { session.units(); } });
        expect("precedence/encoding_before_parse", "JAVA_SOURCE_ENCODING", "source", () -> new SourceText(SOURCE, new byte[]{(byte) 0xff, '{', '\n'}));
        expect("precedence/parse_before_attribution", "JAVA_SOURCE_PARSE", "source", () -> { try (var session = CompilerSession.analyzeSources(List.of(source("package vector; public interface Case { static int f() { return missing;\n")))) { session.units(); } });
        expect("precedence/attribution_before_subset", "JAVA_SOURCE_DIAGNOSTIC", "typecheck", () -> { try (var session = CompilerSession.analyzeSources(List.of(source("package vector; public interface Case { static int f() { while (true) { return missing; } } }\n")))) { session.units(); } });
        // T05/T06 own actual subset/contract/lowering/map outcomes. T04 proves
        // their fixed failure transport never adds any partial artifact member.
        for (String code : List.of("JAVA_SUBSET_INITIALIZATION", "JAVA_CONTRACT_JSON", "JAVA_SOURCE_MAP_RANGE"))
            failure("future-phase-transport/" + code, SELECTION, FrontendFailure.of(code, "emission"));
    }

    private static final class UnknownType implements TypeMirror {
        @Override public TypeKind getKind() { return TypeKind.ERROR; }
        @Override public <R, P> R accept(TypeVisitor<R, P> visitor, P value) { return visitor.visitUnknown(this, value); }
        @Override public List<? extends AnnotationMirror> getAnnotationMirrors() { return List.of(); }
        @Override public <A extends Annotation> A getAnnotation(Class<A> type) { return null; }
        @Override public <A extends Annotation> A[] getAnnotationsByType(Class<A> type) { throw new AssertionError("annotations not used"); }
    }
    private record ForeignUnit(CompilationUnitTree delegate, JavaFileObject source) implements CompilationUnitTree {
        @Override public JavaFileObject getSourceFile() { return source; }
        @Override public ModuleTree getModule() { return delegate.getModule(); }
        @Override public List<? extends AnnotationTree> getPackageAnnotations() { return delegate.getPackageAnnotations(); }
        @Override public ExpressionTree getPackageName() { return delegate.getPackageName(); }
        @Override public PackageTree getPackage() { return delegate.getPackage(); }
        @Override public List<? extends ImportTree> getImports() { return delegate.getImports(); }
        @Override public List<? extends Tree> getTypeDecls() { return delegate.getTypeDecls(); }
        @Override public LineMap getLineMap() { return delegate.getLineMap(); }
        @Override public Kind getKind() { return Kind.COMPILATION_UNIT; }
        @Override public <R, D> R accept(TreeVisitor<R, D> visitor, D data) { return visitor.visitCompilationUnit(this, data); }
    }
}
