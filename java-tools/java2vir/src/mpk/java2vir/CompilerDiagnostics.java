package mpk.java2vir;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Set;
import javax.tools.Diagnostic;
import javax.tools.DiagnosticListener;
import javax.tools.JavaFileObject;

/** Counts every callback before retention, normalizes provenance, never requests compiler prose. */
final class CompilerDiagnostics implements DiagnosticListener<JavaFileObject> {
    static final Set<String> CODES = Set.of("compiler.err.cant.resolve.location",
            "compiler.err.doesnt.exist", "compiler.err.int.number.too.large",
            "compiler.err.premature.eof", "compiler.err.prob.found.req",
            "compiler.err.var.might.not.have.been.initialized");
    record Raw(String path, int start, int end, String code, Diagnostic.Kind kind, long ordinal,
               FrontendFailure.Span span) {}
    private final IdentityHashMap<JavaFileObject, SourceText> sources = new IdentityHashMap<>();
    private final List<Raw> values = new ArrayList<>();
    private String phase = "source";
    private long callbacks;

    CompilerDiagnostics(List<SourceText> sources) {
        for (SourceText source : sources) this.sources.put(source, source);
    }

    void phase(String phase) {
        if (!phase.equals("source") && !phase.equals("typecheck")) throw new IllegalArgumentException("diagnostic phase");
        this.phase = phase;
    }

    @Override public void report(Diagnostic<? extends JavaFileObject> diagnostic) {
        callbacks = FrontendLimits.add("normalized_issues", callbacks, 1, phase);
        if (diagnostic == null) throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
        String code = diagnostic.getCode();
        Diagnostic.Kind kind = diagnostic.getKind();
        if (code == null || !CODES.contains(code) || kind != Diagnostic.Kind.ERROR)
            throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
        SourceText source = sources.get(diagnostic.getSource());
        if (source == null) throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
        long first = diagnostic.getStartPosition();
        long last = diagnostic.getEndPosition();
        var span = source.span(first, last, true, phase);
        int start = source.byteOffset(first);
        int end = source.byteOffset(last);
        values.add(new Raw(source.path(), start, end, code, kind, callbacks, span));
    }

    long callbacks() { return callbacks; }

    List<Raw> raw() {
        return values.stream().sorted(Comparator.comparing(Raw::path).thenComparingInt(Raw::start)
                .thenComparingInt(Raw::end).thenComparing(Raw::code)
                .thenComparingInt(r -> r.kind().ordinal()).thenComparingLong(Raw::ordinal)).toList();
    }

    void finishPhase() {
        if (values.isEmpty()) return;
        String code = phase.equals("source") ? "JAVA_SOURCE_PARSE" : "JAVA_SOURCE_DIAGNOSTIC";
        throw FrontendFailure.issues(code, phase,
                raw().stream().map(raw -> new FrontendFailure.Issue(code, raw.span())).toList());
    }
}
