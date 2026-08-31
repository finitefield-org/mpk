package mpk.java2vir;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/** Failure-only successor transport. T06 owns success/artifact emission. */
final class Protocol {
    private Protocol() {}
    private static final Map<String, Object> CONTEXT = Map.of(
            "profile_entry_sha256", "0d80d13f97c45557fa9978eccc2545ffdb3fc1b93a26856b365a9be200470301",
            "profile_registry", Map.of("id", "mpk.semantic_profile.registry.v1", "schema", "mpk.semantic_profile.registry.v1",
                    "revision", 3, "registry_sha256", "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557"),
            "semantic_parameters", Map.of("schema", "mpk.semantic_parameters.java_scalar.v0", "value", Map.of(
                    "annotation_processing", "none", "encoding", "UTF-8", "language_version", "25", "preview", false,
                    "release", "25", "target_id", "linux-x64")),
            "semantic_profile", "mpk.java.scalar.v0", "source_language", "java");

    static Map<String, Object> semanticContext() { return CONTEXT; }

    static byte[] failure(Selection selection, FrontendFailure failure) {
        List<Map<String, Object>> issues = failure.issues().stream().map(FrontendFailure.Issue::json).toList();
        StringBuilder out = new StringBuilder();
        out.append("{\"diagnostics\":");
        append(out, failure.status().equals("rejected") ? List.of() : issues);
        out.append(",\"phase\":"); append(out, failure.phase());
        out.append(",\"rejected_features\":");
        append(out, failure.status().equals("rejected") ? issues : List.of());
        out.append(",\"schema\":\"mpk.frontend.cli.v1\",\"selection\":"); append(out, selection.envelope());
        out.append(",\"semantic_context\":").append(json(CONTEXT));
        out.append(",\"status\":"); append(out, failure.status());
        out.append("}\n");
        byte[] bytes = out.toString().getBytes(StandardCharsets.UTF_8);
        FrontendLimits.check("frontend_stdout", bytes.length, failure.phase());
        return bytes;
    }

    static String json(Object value) {
        StringBuilder out = new StringBuilder();
        append(out, value);
        return out.toString();
    }

    private static void append(StringBuilder out, Object value) {
        if (value instanceof String text) {
            out.append('"');
            for (int i = 0; i < text.length(); i++) {
                char c = text.charAt(i);
                switch (c) {
                    case '"' -> out.append("\\\"");
                    case '\\' -> out.append("\\\\");
                    case '\b' -> out.append("\\b");
                    case '\t' -> out.append("\\t");
                    case '\n' -> out.append("\\n");
                    case '\f' -> out.append("\\f");
                    case '\r' -> out.append("\\r");
                    default -> {
                        if (c < 0x20) {
                            out.append("\\u00").append(Character.forDigit(c >>> 4, 16)).append(Character.forDigit(c & 15, 16));
                        } else if (Character.isSurrogate(c)) {
                            if (!Character.isHighSurrogate(c) || i + 1 >= text.length() || !Character.isLowSurrogate(text.charAt(i + 1)))
                                throw new IllegalArgumentException("invalid Unicode scalar");
                            out.append(c).append(text.charAt(++i));
                        } else out.append(c);
                    }
                }
            }
            out.append('"');
        } else if (value instanceof Boolean || value instanceof Integer || value instanceof Long) {
            out.append(value);
        } else if (value instanceof List<?> list) {
            out.append('[');
            boolean first = true;
            for (Object item : list) { if (!first) out.append(','); first = false; append(out, item); }
            out.append(']');
        } else if (value instanceof Map<?, ?> map) {
            var sorted = new TreeMap<String, Object>();
            for (var entry : map.entrySet()) {
                if (!(entry.getKey() instanceof String key)) throw new IllegalArgumentException("JSON key");
                sorted.put(key, entry.getValue());
            }
            out.append('{');
            boolean first = true;
            for (var entry : sorted.entrySet()) {
                if (!first) out.append(','); first = false;
                append(out, entry.getKey()); out.append(':'); append(out, entry.getValue());
            }
            out.append('}');
        } else if (value == null) out.append("null");
        else throw new IllegalArgumentException("unsupported JSON value");
    }
}
