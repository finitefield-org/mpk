package mpk.java2vir;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

/** Bounded, fixed-message failures. Compiler text and exception messages never escape. */
final class FrontendFailure extends RuntimeException {
    private static final long serialVersionUID = 1L;
    record Span(String path, int start, int end) {
        Span {
            if (!Selection.portablePath(path) || start < 0 || end <= start)
                throw new IllegalArgumentException("invalid issue span");
        }
        Map<String, Object> json() { return Map.of("normalized_path", path, "start", start, "end", end); }
    }
    record Issue(String code, Span span) {
        Issue { DiagnosticRegistry.definition(code); }
        String message() { return DiagnosticRegistry.definition(code).message(); }
        Map<String, Object> json() {
            return span == null ? Map.of("code", code, "message", message())
                    : Map.of("code", code, "message", message(), "span", span.json());
        }
    }
    static final Comparator<Issue> ISSUE_ORDER = Comparator
            .comparing((Issue i) -> i.span() == null ? "" : i.span().path())
            .thenComparingInt(i -> i.span() == null ? -1 : i.span().start())
            .thenComparing(Issue::code).thenComparing(Issue::message)
            .thenComparingInt(i -> i.span() == null ? -1 : i.span().end());

    private final String phase;
    private final transient List<Issue> issues;

    private FrontendFailure(String phase, List<Issue> issues) {
        super(issues.getFirst().code(), null, false, false);
        this.phase = phase;
        this.issues = List.copyOf(issues);
    }

    static FrontendFailure of(String code, String started) {
        return new FrontendFailure(DiagnosticRegistry.phase(code, started), List.of(new Issue(code, null)));
    }

    static FrontendFailure issues(String code, String started, List<Issue> values) {
        if (values.isEmpty()) throw new IllegalArgumentException("empty failure");
        String phase = DiagnosticRegistry.phase(code, started);
        var sorted = new ArrayList<Issue>();
        long bytes = 0;
        for (Issue value : values) {
            FrontendLimits.check("normalized_issues", (long) sorted.size() + 1, started);
            if (!value.code().equals(code)) throw new IllegalArgumentException("mixed diagnostic owners");
            int length = value.message().length(); // All frozen public messages are ASCII.
            FrontendLimits.check("diagnostic_message_bytes", length, started);
            bytes = FrontendLimits.add("diagnostic_total_message_bytes", bytes, length, started);
            sorted.add(value);
        }
        sorted.sort(ISSUE_ORDER);
        return new FrontendFailure(phase, sorted);
    }

    String code() { return issues.getFirst().code(); }
    String status() { return DiagnosticRegistry.definition(code()).status(); }
    String phase() { return phase; }
    int exitCode() { return DiagnosticRegistry.definition(code()).exitCode(); }
    List<Issue> issues() { return issues; }

    static FrontendFailure compilerFailure(Throwable error, String phase) {
        // javac wraps client callbacks. Keep our bounded failure; discard all other details.
        Throwable current = error;
        for (int depth = 0; current != null && depth < 16; depth++, current = current.getCause()) {
            if (current instanceof FrontendFailure failure) return failure;
            if (current instanceof VirtualMachineError) return of("JAVA_FRONTEND_RESOURCE", phase);
        }
        return of("JAVA_TOOLCHAIN_ADAPTER", phase);
    }
}
