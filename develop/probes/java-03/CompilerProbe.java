/* Disposable JAVA-03-T01 public compiler API measurement; never a frontend. */
import com.sun.source.tree.*;
import com.sun.source.util.*;
import java.io.*;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import javax.lang.model.element.Element;
import javax.tools.*;

public final class CompilerProbe {
    static final List<String> OPTIONS = List.of("--release", "25", "-encoding", "UTF-8",
        "-proc:none", "-implicit:none", "-Xlint:none", "-Xmaxerrs", "1025", "-Xmaxwarns", "1025");
    static final List<Object> CASES = new ArrayList<>();
    static final Map<String, Long> ACCESSES = new TreeMap<>();

    static Map<String, Object> obj(Object... pairs) {
        var m = new LinkedHashMap<String, Object>();
        for (int i = 0; i < pairs.length; i += 2) m.put((String)pairs[i], pairs[i + 1]);
        return m;
    }
    static final class Source extends SimpleJavaFileObject {
        final String content;
        Source(String path, String content) { super(URI.create("mem:///" + path), Kind.SOURCE); this.content = content; }
        @Override public CharSequence getCharContent(boolean ignoreEncodingErrors) { return content; }
    }
    static final class DiagnosticLimit extends RuntimeException {}
    static final class Listener implements DiagnosticListener<JavaFileObject> {
        final List<Object> values = new ArrayList<>();
        int seen;
        @Override public void report(Diagnostic<? extends JavaFileObject> d) {
            if (++seen > 1024) throw new DiagnosticLimit();
            values.add(obj("kind", d.getKind().name(), "code", d.getCode(),
                "source", d.getSource() == null ? null : d.getSource().toUri().toString(),
                "start_utf16", d.getStartPosition(), "end_utf16", d.getEndPosition(),
                "position_utf16", d.getPosition(), "line", d.getLineNumber(), "column", d.getColumnNumber()));
        }
        boolean errors() { return values.stream().anyMatch(v -> ((Map<?, ?>)v).get("kind").equals("ERROR")); }
    }

    // Application lookup mediation. --release 25 separately installs a javac-owned
    // platform file manager: its system lookups do NOT traverse this wrapper.
    // Only SYSTEM_MODULES and explicitly obtained children may delegate here.
    // Initial in-memory compilation units are not discoverable by list.
    static final class ClosedManager extends ForwardingJavaFileManager<StandardJavaFileManager> {
        final Set<Location> system = Collections.newSetFromMap(new IdentityHashMap<>());
        final Map<String, Long> calls = new TreeMap<>();
        long systemFiles;
        boolean closed;
        int outputAttempts;
        ClosedManager(StandardJavaFileManager base) { super(base); system.add(StandardLocation.SYSTEM_MODULES); }
        void touch(String name, Location location) {
            if (closed) throw new IllegalStateException("closed probe file manager");
            String key = name + ":" + (location == null ? "-" : location.getName());
            calls.merge(key, 1L, Long::sum); ACCESSES.merge(key, 1L, Long::sum);
        }
        boolean allowed(Location location) { return system.contains(location); }
        JavaFileObject verify(JavaFileObject file) throws IOException {
            if (file == null) return null;
            String uri = file.toUri().toString();
            if (!uri.startsWith("jrt:/") && !uri.startsWith("jar:file:/mpk/toolchain/jdk/lib/ct.sym!/"))
                throw new IOException("non-system compiler input");
            systemFiles++; return file;
        }
        @Override public boolean hasLocation(Location location) {
            touch("hasLocation", location);
            return allowed(location) && fileManager.hasLocation(location);
        }
        @Override public Iterable<JavaFileObject> list(Location location, String pkg, Set<JavaFileObject.Kind> kinds, boolean recurse) throws IOException {
            touch("list", location);
            if (!allowed(location)) return List.of();
            var result = new ArrayList<JavaFileObject>();
            for (var f : fileManager.list(location, pkg, kinds, recurse)) result.add(verify(f));
            return result;
        }
        @Override public JavaFileObject getJavaFileForInput(Location location, String name, JavaFileObject.Kind kind) throws IOException {
            touch("getJavaFileForInput", location);
            return allowed(location) ? verify(fileManager.getJavaFileForInput(location, name, kind)) : null;
        }
        @Override public FileObject getFileForInput(Location location, String pkg, String name) throws IOException {
            touch("getFileForInput", location);
            // No non-Java resources are needed by the measured compiler session.
            return null;
        }
        @Override public String inferBinaryName(Location location, JavaFileObject file) {
            touch("inferBinaryName", location);
            return allowed(location) ? fileManager.inferBinaryName(location, file) : null;
        }
        @Override public boolean contains(Location location, FileObject file) throws IOException {
            touch("contains", location);
            return allowed(location) && fileManager.contains(location, file);
        }
        @Override public ClassLoader getClassLoader(Location location) { touch("getClassLoader", location); return null; }
        @Override public <S> ServiceLoader<S> getServiceLoader(Location location, Class<S> service) throws IOException {
            touch("getServiceLoader", location); throw new UnsupportedOperationException("service discovery refused");
        }
        @Override public String inferModuleName(Location location) throws IOException {
            touch("inferModuleName", location);
            return allowed(location) ? fileManager.inferModuleName(location) : null;
        }
        @Override public Iterable<Set<Location>> listLocationsForModules(Location location) throws IOException {
            touch("listLocationsForModules", location);
            if (!allowed(location)) return List.of();
            var result = new ArrayList<Set<Location>>();
            for (var group : fileManager.listLocationsForModules(location)) { system.addAll(group); result.add(group); }
            return result;
        }
        @Override public Location getLocationForModule(Location location, String module) throws IOException {
            touch("getLocationForModuleName", location);
            if (!allowed(location)) return null;
            Location result = fileManager.getLocationForModule(location, module);
            if (result != null) system.add(result);
            return result;
        }
        @Override public Location getLocationForModule(Location location, JavaFileObject file) throws IOException {
            touch("getLocationForModuleFile", location);
            if (!allowed(location)) return null;
            Location result = fileManager.getLocationForModule(location, file);
            if (result != null) system.add(result);
            return result;
        }
        @Override public JavaFileObject getJavaFileForOutput(Location location, String name, JavaFileObject.Kind kind, FileObject sibling) throws IOException {
            touch("getJavaFileForOutput", location); outputAttempts++; throw new IOException("output refused");
        }
        @Override public FileObject getFileForOutput(Location location, String pkg, String name, FileObject sibling) throws IOException {
            touch("getFileForOutput", location); outputAttempts++; throw new IOException("output refused");
        }
        @Override public JavaFileObject getJavaFileForOutputForOriginatingFiles(Location location, String name, JavaFileObject.Kind kind, FileObject... origins) throws IOException {
            touch("getJavaFileForOutputForOriginatingFiles", location); outputAttempts++; throw new IOException("output refused");
        }
        @Override public FileObject getFileForOutputForOriginatingFiles(Location location, String pkg, String name, FileObject... origins) throws IOException {
            touch("getFileForOutputForOriginatingFiles", location); outputAttempts++; throw new IOException("output refused");
        }
        @Override public boolean isSameFile(FileObject a, FileObject b) {
            touch("isSameFile", null);
            if (a instanceof Source || b instanceof Source) return a.toUri().equals(b.toUri());
            return fileManager.isSameFile(a, b);
        }
        @Override public boolean handleOption(String current, Iterator<String> remaining) {
            touch("handleOption", null);
            // javac passes the fixed options to the manager too; user options never enter.
            if (!current.equals("--multi-release") && !current.equals("-encoding")) return false;
            if (!remaining.hasNext()) throw new IllegalArgumentException("missing fixed file-manager option value");
            String value = remaining.next();
            String expected = current.equals("-encoding") ? "UTF-8" : "25";
            if (!expected.equals(value)) throw new IllegalArgumentException("unexpected fixed file-manager option value");
            return fileManager.handleOption(current, List.of(value).iterator());
        }
        @Override public int isSupportedOption(String option) {
            touch("isSupportedOption", null);
            return option.equals("--multi-release") || option.equals("-encoding") ? fileManager.isSupportedOption(option) : -1;
        }
        @Override public void close() throws IOException { if (!closed) { closed = true; fileManager.close(); } }
        @Override public void flush() throws IOException { touch("flush", null); fileManager.flush(); }
    }

    static List<Object> trees(JavacTask task, List<CompilationUnitTree> units, boolean analyzed) {
        var rows = new ArrayList<Object>();
        Trees api = Trees.instance(task);
        for (var unit : units) {
            String source;
            try { source = unit.getSourceFile().getCharContent(false).toString(); }
            catch (IOException e) { throw new UncheckedIOException(e); }
            new TreePathScanner<Void, Void>() {
                @Override public Void scan(Tree tree, Void p) {
                    if (tree == null) return null;
                    long start = api.getSourcePositions().getStartPosition(unit, tree);
                    long end = api.getSourcePositions().getEndPosition(unit, tree);
                    TreePath path = getCurrentPath() == null ? new TreePath(unit) : new TreePath(getCurrentPath(), tree);
                    var row = obj("kind", tree.getKind().name(), "start_utf16", start, "end_utf16", end);
                    if (start >= 0 && end >= start && end <= source.length()) {
                        String spelling = source.substring((int)start, (int)end);
                        row.put("spelling", spelling);
                        row.put("start_utf8", source.substring(0, (int)start).getBytes(StandardCharsets.UTF_8).length);
                        row.put("end_utf8", source.substring(0, (int)end).getBytes(StandardCharsets.UTF_8).length);
                        row.put("line", unit.getLineMap().getLineNumber(start));
                        row.put("tab_expanded_column", unit.getLineMap().getColumnNumber(start));
                    }
                    if (tree instanceof LiteralTree literal) row.put("literal_value", String.valueOf(literal.getValue()));
                    if (tree instanceof ModifiersTree modifiers) row.put("flags", modifiers.getFlags().stream().map(Object::toString).sorted().toList());
                    if (analyzed) {
                        var type = api.getTypeMirror(path);
                        Element element = api.getElement(path);
                        row.put("type", type == null ? null : type.toString());
                        row.put("element", element == null ? null : element.toString());
                        row.put("element_kind", element == null ? null : element.getKind().name());
                        row.put("element_modifiers", element == null ? null : element.getModifiers().stream().map(Object::toString).sorted().toList());
                    }
                    rows.add(row);
                    return super.scan(tree, p);
                }
            }.scan(unit, null);
        }
        return rows;
    }
    static String source(String body) { return "package demo;\npublic interface Probe {\n" + body + "\n}\n"; }
    static List<Object> discoveryControls() throws Exception {
        var controls = new ArrayList<Object>();
        for (String expression : List.of("Hidden.value()", "poison.Injected.value()")) {
            JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
            Listener listener = new Listener();
            try (var base = compiler.getStandardFileManager(listener, Locale.US, StandardCharsets.UTF_8)) {
                base.setLocationFromPaths(StandardLocation.CLASS_PATH, List.of(Path.of("/work/poison.jar")));
                base.setLocationFromPaths(StandardLocation.SOURCE_PATH, List.of(Path.of("/work/poison-source")));
                base.setLocationFromPaths(StandardLocation.ANNOTATION_PROCESSOR_PATH, List.of(Path.of("/work/poison.jar")));
                JavacTask task = (JavacTask)compiler.getTask(new StringWriter(), base, listener, OPTIONS, null,
                    List.of(new Source("src/demo/Probe.java", source("public static int a() { return " + expression + "; }"))));
                task.setLocale(Locale.US);
                task.parse();
                var elements = new ArrayList<String>();
                task.analyze().forEach(element -> elements.add(element.toString()));
                if (listener.seen != 0) throw new AssertionError("planted discovery control did not resolve");
                controls.add(obj("expression", expression, "diagnostics", listener.values, "analyzed_elements", elements,
                    "purpose", "same planted dependency resolves with unwrapped standard manager and identical options"));
            }
        }
        return controls;
    }
    static Map<String, Object> boundaryChecks() throws Exception {
        var compiler = ToolProvider.getSystemJavaCompiler();
        var base = compiler.getStandardFileManager(null, Locale.US, StandardCharsets.UTF_8);
        var fm = new ClosedManager(base);
        var results = new LinkedHashMap<String, Object>();
        try {
            var loc = StandardLocation.CLASS_PATH;
            results.put("application_list_empty", !fm.list(loc, "poison", Set.of(JavaFileObject.Kind.CLASS), true).iterator().hasNext());
            results.put("application_java_input_absent", fm.getJavaFileForInput(loc, "poison.Injected", JavaFileObject.Kind.CLASS) == null);
            results.put("application_resource_input_absent", fm.getFileForInput(loc, "", "META-INF/services/javax.annotation.processing.Processor") == null);
            results.put("application_binary_name_absent", fm.inferBinaryName(loc, new Source("x.java", "")) == null);
            results.put("application_contains_false", !fm.contains(loc, new Source("x.java", "")));
            results.put("application_classloader_absent", fm.getClassLoader(loc) == null);
            results.put("application_module_name_absent", fm.inferModuleName(StandardLocation.MODULE_PATH) == null);
            results.put("application_module_locations_empty", !fm.listLocationsForModules(StandardLocation.MODULE_PATH).iterator().hasNext());
            results.put("application_module_by_name_absent", fm.getLocationForModule(StandardLocation.MODULE_PATH, "poison") == null);
            results.put("application_module_by_file_absent", fm.getLocationForModule(StandardLocation.MODULE_PATH, new Source("x.java", "")) == null);
            try { fm.getServiceLoader(loc, javax.annotation.processing.Processor.class); results.put("service_loader_refused", false); }
            catch (UnsupportedOperationException expected) { results.put("service_loader_refused", true); }
            try { fm.getJavaFileForOutput(StandardLocation.CLASS_OUTPUT, "X", JavaFileObject.Kind.CLASS, null); results.put("java_output_refused", false); }
            catch (IOException expected) { results.put("java_output_refused", true); }
            try { fm.getFileForOutput(StandardLocation.CLASS_OUTPUT, "", "x", null); results.put("resource_output_refused", false); }
            catch (IOException expected) { results.put("resource_output_refused", true); }
            try { fm.getJavaFileForOutputForOriginatingFiles(StandardLocation.CLASS_OUTPUT, "X", JavaFileObject.Kind.CLASS); results.put("originating_java_output_refused", false); }
            catch (IOException expected) { results.put("originating_java_output_refused", true); }
            try { fm.getFileForOutputForOriginatingFiles(StandardLocation.CLASS_OUTPUT, "", "x"); results.put("originating_resource_output_refused", false); }
            catch (IOException expected) { results.put("originating_resource_output_refused", true); }
        } finally { fm.close(); }
        if (results.values().stream().anyMatch(v -> !Boolean.TRUE.equals(v))) throw new AssertionError("file manager boundary assertion failed");
        return results;
    }
    static void run(String id, String content, boolean detail, boolean planted) throws Exception {
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        Listener diagnostics = new Listener();
        var base = compiler.getStandardFileManager(diagnostics, Locale.US, StandardCharsets.UTF_8);
        for (StandardLocation loc : List.of(StandardLocation.CLASS_PATH, StandardLocation.SOURCE_PATH,
                StandardLocation.MODULE_PATH, StandardLocation.UPGRADE_MODULE_PATH,
                StandardLocation.ANNOTATION_PROCESSOR_PATH, StandardLocation.ANNOTATION_PROCESSOR_MODULE_PATH))
            base.setLocationFromPaths(loc, List.of());
        if (planted) {
            base.setLocationFromPaths(StandardLocation.CLASS_PATH, List.of(Path.of("/work/poison.jar")));
            base.setLocationFromPaths(StandardLocation.SOURCE_PATH, List.of(Path.of("/work/poison-source")));
            base.setLocationFromPaths(StandardLocation.ANNOTATION_PROCESSOR_PATH, List.of(Path.of("/work/poison.jar")));
        }
        ClosedManager fm = new ClosedManager(base);
        var out = new StringWriter();
        JavacTask task = (JavacTask)compiler.getTask(out, fm, diagnostics, OPTIONS, null,
                List.of(new Source("src/demo/Probe.java", content)));
        task.setLocale(Locale.US);
        var row = obj("id", id, "source", content, "analyze_called", false);
        String phase = "parse";
        try {
            var units = new ArrayList<CompilationUnitTree>();
            task.parse().forEach(units::add);
            if (detail) row.put("before_analysis", trees(task, units, false));
            if (!diagnostics.errors()) {
                phase = "analyze"; row.put("analyze_called", true);
                var elements = new ArrayList<String>();
                task.analyze().forEach(element -> elements.add(element.toString()));
                row.put("analyzed_elements", elements);
                if (detail) {
                    List<Object> after = trees(task, units, true);
                    row.put("after_analysis", after);
                    row.put("public_tree_inventory_unchanged", structuralTrees((List<?>)row.get("before_analysis")).equals(structuralTrees(after)));
                }
            }
        } catch (Throwable failure) {
            var chain = new ArrayList<String>();
            for (Throwable cause = failure; cause != null && chain.size() < 8; cause = cause.getCause()) chain.add(cause.getClass().getName());
            row.put("thrown", chain);
        } finally {
            fm.close();
        }
        row.put("final_phase", phase);
        row.put("diagnostics_seen", diagnostics.seen);
        row.put("diagnostics", diagnostics.values);
        row.put("writer_characters", out.getBuffer().length());
        row.put("output_attempts", fm.outputAttempts);
        row.put("system_files_returned", fm.systemFiles);
        row.put("manager_closed", fm.closed);
        row.put("file_manager_calls", fm.calls);
        try { fm.list(StandardLocation.CLASS_PATH, "", Set.of(JavaFileObject.Kind.CLASS), false); row.put("after_close", "unexpected-success"); }
        catch (IllegalStateException expected) { row.put("after_close", expected.getClass().getName()); }
        if (fm.outputAttempts != 0) throw new AssertionError("parse/analyze attempted class output");
        CASES.add(row);
    }

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.US);
        run("negative-literals", source("public static int a() { return -1; }\npublic static int b() { return -2147483648; }\npublic static long c() { return -9223372036854775808L; }\npublic static int d() { return -(1); }\npublic static int e() { return - -2147483648; }"), true, false);
        run("constant-trees", source("public static int a(int x) { int y = 2 + 3; if (false) { y = demo.Probe.b(x); } return true ? y : x; }\npublic static int b(int x) { return (int)(long)x; }"), true, false);
        run("implicit-public", source("static int a() { return 1; }"), true, false);
        run("utf16-tab-bmp-nonbmp", source("\t/* é😀 */ public static int a(int x) { return x + 1; }"), true, false);
        run("syntax-eof", "package demo;\npublic interface Probe {\npublic static int a() { return 1;\n", true, false);
        run("attribution-unknown-name", source("public static int a() { return absent; }"), true, false);
        run("attribution-type", source("public static int a() { return true; }"), false, false);
        run("attribution-uninitialized-local", source("public static int a() { int y; return y; }"), false, false);
        run("positive-min-magnitude", source("public static int a() { return 2147483648; }"), false, false);
        run("parenthesized-min-magnitude", source("public static int a() { return -(2147483648); }"), false, false);
        run("positive-long-min-magnitude", source("public static long a() { return 9223372036854775808L; }"), false, false);
        run("planted-source", source("public static int a() { return Hidden.value(); }"), false, true);
        run("planted-class", source("public static int a() { return poison.Injected.value(); }"), false, true);
        run("planted-processor-service", source("public static int a() { return 1; }"), false, true);
        run("jdk-reference-view", source("public static int a() { java.lang.String x = null; java.nio.file.Path y = null; return 1; }"), true, false);
        run("excluded-class-default-constructor", "package demo;\npublic class Probe {\npublic static int a(int x) { return x; }\n}\n", true, false);
        run("excluded-var-inferred-type", source("public static int a(int x) { var y = x; return y; }"), true, false);
        String params127 = String.join(",", java.util.stream.IntStream.range(0, 127).mapToObj(i -> "long p" + i).toList());
        String params128 = params127 + ",long p127";
        run("parameter-slots-254", source("public static long a(" + params127 + ") { return p0; }"), false, false);
        run("parameter-slots-256", source("public static long a(" + params128 + ") { return p0; }"), false, false);
        StringBuilder many = new StringBuilder("public static int a() {\n");
        for (int i = 0; i < 1026; i++) many.append("int p").append(i).append(" = absent").append(i).append(";\n");
        many.append("return 0; }");
        run("diagnostic-listener-abort-1025", source(many.toString()), false, false);
        List<Object> controls = discoveryControls();
        Map<String, Object> boundaries = boundaryChecks();
        validateCases();
        Map<String, Object> result = obj("schema", "mpk.java.compiler_api_probe.v0",
            "compiler_session", obj("jdk_runtime_version", Runtime.version().toString(),
                "java_vendor", System.getProperty("java.vendor"), "compiler_provider", ToolProvider.getSystemJavaCompiler().getClass().getName(),
                "options", OPTIONS, "locale", "en-US", "source_encoding", "UTF-8",
                "phases", List.of("parse", "analyze"), "generate_called", false, "compilation_task_call_called", false,
                "fresh_task_per_case", true, "max_retained_diagnostics", 1024),
            "adapter_observations", obj("cases", CASES, "file_manager_calls", ACCESSES,
                "discovery_controls", controls, "file_manager_boundary_checks", boundaries,
                "boot_modules", ModuleLayer.boot().modules().stream().map(Module::getName).sorted().toList(),
                "native_mapped_paths", Files.readAllLines(Path.of("/proc/self/maps")).stream()
                    .map(line -> line.split("\\s+", 6)).filter(parts -> parts.length == 6)
                    .map(parts -> parts[5]).filter(path -> path.contains(".so") && (path.startsWith("/mpk/toolchain/jdk/") || path.startsWith("/usr/lib/x86_64-linux-gnu/") || path.startsWith("/lib/x86_64-linux-gnu/")))
                    .distinct().sorted().toList(),
                "planted_processor_executed", Files.exists(Path.of("/tmp/processor-executed")),
                "scope", "Public compiler API observations under Linux amd64 emulation; not proof of production sandbox compatibility"));
        System.out.println(json(result));
    }
    static void validateCases() {
        Map<String, String> errors = Map.of("syntax-eof", "compiler.err.premature.eof",
            "attribution-unknown-name", "compiler.err.cant.resolve.location", "attribution-type", "compiler.err.prob.found.req",
            "attribution-uninitialized-local", "compiler.err.var.might.not.have.been.initialized",
            "positive-min-magnitude", "compiler.err.int.number.too.large", "parenthesized-min-magnitude", "compiler.err.int.number.too.large",
            "positive-long-min-magnitude", "compiler.err.int.number.too.large", "planted-source", "compiler.err.cant.resolve.location",
            "planted-class", "compiler.err.doesnt.exist");
        for (Object entry : CASES) {
            var c = (Map<?, ?>)entry;
            String id = (String)c.get("id");
            var ds = (List<?>)c.get("diagnostics");
            if (!c.get("writer_characters").equals(0) || !c.get("output_attempts").equals(0) || !Boolean.TRUE.equals(c.get("manager_closed")))
                throw new AssertionError("unexpected compiler output or unclosed manager: " + id);
            if (Boolean.FALSE.equals(c.get("public_tree_inventory_unchanged")) &&
                !id.equals("excluded-class-default-constructor") && !id.equals("excluded-var-inferred-type"))
                throw new AssertionError("unexpected public tree rewrite: " + id);
            if (id.equals("excluded-class-default-constructor")) {
                boolean rawClass = ((List<?>)c.get("before_analysis")).stream().anyMatch(t ->
                    "CLASS".equals(((Map<?, ?>)t).get("kind")));
                boolean syntheticConstructor = ((List<?>)c.get("after_analysis")).stream().anyMatch(t ->
                    "METHOD".equals(((Map<?, ?>)t).get("kind")) && Long.valueOf(-1).equals(((Map<?, ?>)t).get("end_utf16")));
                if (!rawClass || !syntheticConstructor || !Boolean.FALSE.equals(c.get("public_tree_inventory_unchanged")))
                    throw new AssertionError("unexpected excluded-class constructor observation");
            }
            if (id.equals("excluded-var-inferred-type")) {
                boolean rawVar = ((List<?>)c.get("before_analysis")).stream().anyMatch(t ->
                    "VARIABLE".equals(((Map<?, ?>)t).get("kind")) && "var y = x;".equals(((Map<?, ?>)t).get("spelling")));
                boolean inferredInt = ((List<?>)c.get("after_analysis")).stream().anyMatch(t ->
                    "VARIABLE".equals(((Map<?, ?>)t).get("kind")) && "var y = x;".equals(((Map<?, ?>)t).get("spelling")) &&
                    "int".equals(((Map<?, ?>)t).get("type")));
                if (!rawVar || !inferredInt || !Boolean.FALSE.equals(c.get("public_tree_inventory_unchanged")))
                    throw new AssertionError("unexpected excluded-var inferred type observation");
            }
            if (id.equals("diagnostic-listener-abort-1025")) {
                if (!c.get("diagnostics_seen").equals(1025) || ds.size() != 1024 ||
                    !List.of("java.lang.RuntimeException", "CompilerProbe$DiagnosticLimit").equals(c.get("thrown")))
                    throw new AssertionError("unexpected diagnostic abort behavior");
            } else if (c.containsKey("thrown")) throw new AssertionError("unexpected compiler exception: " + id);
            else if (errors.containsKey(id)) {
                if (ds.size() != 1 || !errors.get(id).equals(((Map<?, ?>)ds.getFirst()).get("code")))
                    throw new AssertionError("unexpected diagnostic: " + id);
            } else if (!ds.isEmpty()) throw new AssertionError("unexpected diagnostic: " + id);
        }
        if (Files.exists(Path.of("/tmp/processor-executed"))) throw new AssertionError("processor unexpectedly executed");
    }
    static List<Object> structuralTrees(List<?> rows) {
        var structural = new ArrayList<Object>();
        for (Object entry : rows) {
            var tree = (Map<?, ?>)entry;
            var copy = new LinkedHashMap<String, Object>();
            for (String key : List.of("kind", "start_utf16", "end_utf16", "spelling", "literal_value", "flags"))
                if (tree.containsKey(key)) copy.put(key, tree.get(key));
            structural.add(copy);
        }
        return structural;
    }
    static String json(Object value) {
        if (value == null) return "null";
        if (value instanceof Boolean || value instanceof Number) return value.toString();
        if (value instanceof Map<?, ?> map) return "{" + String.join(",", map.entrySet().stream().map(e -> json(e.getKey().toString()) + ":" + json(e.getValue())).toList()) + "}";
        if (value instanceof Iterable<?> values) {
            var parts = new ArrayList<String>(); for (Object element : values) parts.add(json(element));
            return "[" + String.join(",", parts) + "]";
        }
        String s = value.toString(); var out = new StringBuilder("\"");
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '\\' -> out.append("\\\\"); case '"' -> out.append("\\\"");
                case '\n' -> out.append("\\n"); case '\r' -> out.append("\\r"); case '\t' -> out.append("\\t");
                default -> { if (c < 32) out.append(String.format(Locale.ROOT, "\\u%04x", (int)c)); else out.append(c); }
            }
        }
        return out.append('"').toString();
    }
}
