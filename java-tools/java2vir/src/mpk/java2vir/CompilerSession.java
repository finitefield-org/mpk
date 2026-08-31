package mpk.java2vir;

import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.util.JavacTask;
import com.sun.source.util.Trees;
import java.io.IOException;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import javax.lang.model.util.Elements;
import javax.lang.model.util.Types;
import javax.tools.StandardJavaFileManager;
import javax.tools.ToolProvider;

/** Fresh parse/analyze-only compiler session; no selected classes are generated or loaded. */
final class CompilerSession implements AutoCloseable {
    static final List<String> OPTIONS = List.of("--release", "25", "-encoding", "UTF-8",
            "-proc:none", "-implicit:none", "-Xlint:none", "-Xmaxerrs", "1025", "-Xmaxwarns", "1025");
    private final JavacTask task;
    private final Trees trees;
    private final List<CompilationUnitTree> units;
    private final TreeInventory before;
    private final TreeInventory after;
    private final ClosedFileManager manager;

    private CompilerSession(JavacTask task, List<CompilationUnitTree> units,
                            TreeInventory before, TreeInventory after, ClosedFileManager manager) {
        this.task = task;
        this.trees = Trees.instance(task);
        this.units = List.copyOf(units);
        this.before = before;
        this.after = after;
        this.manager = manager;
    }

    static CompilerSession analyze(CapturedSnapshot snapshot) {
        return analyzeSources(snapshot.sources());
    }

    // Source objects are immutable and already transport-validated. No runtime options
    // or file-manager injection is exposed by the production entrypoint.
    static CompilerSession analyzeSources(List<SourceText> sources) {
        if (sources.isEmpty()) throw FrontendFailure.of("JAVA_CAPTURE_INVENTORY", "capture");
        FrontendLimits.check("source_files", sources.size(), "capture");
        sources = List.copyOf(sources);
        String previous = "";
        long bytes = 0;
        for (SourceText source : sources) {
            if (previous.compareTo(source.path()) >= 0) throw FrontendFailure.of("JAVA_CAPTURE_PATH", "capture");
            bytes = FrontendLimits.add("source_total_bytes", bytes, source.byteLength(), "capture");
            previous = source.path();
        }
        if (!BuildIdentity.matches()) throw FrontendFailure.of("JAVA_TOOLCHAIN_COMPILER", "metadata");
        var diagnostics = new CompilerDiagnostics(sources);
        StandardJavaFileManager base = null;
        ClosedFileManager manager = null;
        String phase = "metadata";
        boolean transferred = false;
        try {
            var compiler = ToolProvider.getSystemJavaCompiler();
            if (compiler == null) throw FrontendFailure.of("JAVA_TOOLCHAIN_COMPILER", phase);
            base = compiler.getStandardFileManager(diagnostics, Locale.US, StandardCharsets.UTF_8);
            manager = new ClosedFileManager(base, sources);
            var writer = new RejectingWriter();
            var compilation = compiler.getTask(writer, manager, diagnostics, OPTIONS, null, sources);
            if (!(compilation instanceof JavacTask task)) throw FrontendFailure.of("JAVA_TOOLCHAIN_COMPILER", phase);
            task.setLocale(Locale.US);
            phase = "source";
            writer.phase = phase;
            manager.phase(phase);
            var units = new ArrayList<CompilationUnitTree>();
            for (CompilationUnitTree unit : task.parse()) {
                FrontendLimits.check("source_files", (long) units.size() + 1, phase);
                units.add(unit);
            }
            diagnostics.finishPhase();
            if (units.size() != sources.size()) throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
            Trees trees = Trees.instance(task);
            var origins = new TreeInventory.Origins(units, sources);
            TreeInventory before = TreeInventory.snapshot(trees, units, origins, false);
            phase = "typecheck";
            writer.phase = phase;
            manager.phase(phase);
            diagnostics.phase(phase);
            // Force completion of attribution. Do not call generate() or CompilationTask.call().
            for (var ignored : task.analyze()) { /* javac completes analysis while iterating */ }
            diagnostics.finishPhase();
            TreeInventory after = TreeInventory.snapshot(trees, units, origins, true);
            // T05 runs raw parent-first subset gates before requireUnchanged on admitted subtrees.
            // In particular, synthesized children of a rejected class are not adapter failures.
            CompilerSession result = new CompilerSession(task, units, before, after, manager);
            transferred = true;
            return result;
        } catch (FrontendFailure failure) {
            throw failure;
        } catch (IOException | RuntimeException | Error error) {
            throw FrontendFailure.compilerFailure(error, phase);
        } finally {
            if (!transferred && base != null) {
                try { if (manager == null) base.close(); else manager.close(); }
                catch (IOException error) { throw FrontendFailure.of("JAVA_TOOLCHAIN_FILE_MANAGER", phase); }
            }
        }
    }

    Trees trees() { ensureOpen(); return trees; }
    Elements elements() { ensureOpen(); return task.getElements(); }
    Types types() { ensureOpen(); return task.getTypes(); }
    List<CompilationUnitTree> units() { ensureOpen(); return units; }
    TreeInventory before() { ensureOpen(); return before; }
    TreeInventory after() { ensureOpen(); return after; }
    ClosedFileManager manager() { return manager; }
    private void ensureOpen() {
        if (manager.closed()) throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", "typecheck");
    }
    @Override public void close() {
        try { manager.close(); }
        catch (IOException error) { throw FrontendFailure.of("JAVA_TOOLCHAIN_FILE_MANAGER", "typecheck"); }
    }

    private static final class RejectingWriter extends Writer {
        private String phase = "metadata";
        @Override public void write(char[] chars, int offset, int length) {
            if (length != 0) throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
        }
        @Override public void flush() {}
        @Override public void close() {}
    }
}
