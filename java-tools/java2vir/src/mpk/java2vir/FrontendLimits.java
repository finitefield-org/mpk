package mpk.java2vir;

import java.util.Map;

/** Inclusive counters; test the addition before retaining the first excess item. */
final class FrontendLimits {
    private FrontendLimits() {}
    record Limit(long maximum, String code) {}
    static final Map<String, Limit> DEFINITIONS = Map.ofEntries(
            Map.entry("source_files", new Limit(256L, "JAVA_LIMIT_SOURCE_FILES")),
            Map.entry("source_file_bytes", new Limit(1048576L, "JAVA_LIMIT_SOURCE_FILE_BYTES")),
            Map.entry("source_total_bytes", new Limit(16777216L, "JAVA_LIMIT_SOURCE_TOTAL_BYTES")),
            Map.entry("contract_files", new Limit(128L, "JAVA_LIMIT_CONTRACT_FILES")),
            Map.entry("contract_file_bytes", new Limit(1048576L, "JAVA_LIMIT_CONTRACT_FILE_BYTES")),
            Map.entry("contract_total_bytes", new Limit(8388608L, "JAVA_LIMIT_CONTRACT_TOTAL_BYTES")),
            Map.entry("snapshot_entries", new Limit(512L, "JAVA_LIMIT_SNAPSHOT_ENTRIES")),
            Map.entry("snapshot_total_bytes", new Limit(33554432L, "JAVA_LIMIT_SNAPSHOT_TOTAL_BYTES")),
            Map.entry("normalized_path_bytes", new Limit(1024L, "JAVA_LIMIT_NORMALIZED_PATH_BYTES")),
            Map.entry("canonical_method_id_bytes", new Limit(1024L, "JAVA_LIMIT_CANONICAL_METHOD_ID_BYTES")),
            Map.entry("selected_methods", new Limit(32L, "JAVA_LIMIT_SELECTED_METHODS")),
            Map.entry("method_closure", new Limit(128L, "JAVA_LIMIT_METHOD_CLOSURE")),
            Map.entry("syntax_nodes", new Limit(250000L, "JAVA_LIMIT_SYNTAX_NODES")),
            Map.entry("syntax_depth", new Limit(256L, "JAVA_LIMIT_SYNTAX_DEPTH")),
            Map.entry("instructions_per_method", new Limit(100000L, "JAVA_LIMIT_INSTRUCTIONS_PER_METHOD")),
            Map.entry("instructions_per_closure", new Limit(250000L, "JAVA_LIMIT_INSTRUCTIONS_PER_CLOSURE")),
            Map.entry("cfg_blocks_per_method", new Limit(1024L, "JAVA_LIMIT_CFG_BLOCKS_PER_METHOD")),
            Map.entry("cfg_blocks_per_closure", new Limit(8192L, "JAVA_LIMIT_CFG_BLOCKS_PER_CLOSURE")),
            Map.entry("contract_clauses", new Limit(64L, "JAVA_LIMIT_CONTRACT_CLAUSES")),
            Map.entry("contract_nodes_per_method", new Limit(1024L, "JAVA_LIMIT_CONTRACT_NODES_PER_METHOD")),
            Map.entry("contract_nodes_per_closure", new Limit(8192L, "JAVA_LIMIT_CONTRACT_NODES_PER_CLOSURE")),
            Map.entry("contract_depth", new Limit(32L, "JAVA_LIMIT_CONTRACT_DEPTH")),
            Map.entry("normalized_issues", new Limit(1024L, "JAVA_FRONTEND_DIAGNOSTIC_BUDGET")),
            Map.entry("diagnostic_message_bytes", new Limit(4096L, "JAVA_FRONTEND_DIAGNOSTIC_BUDGET")),
            Map.entry("diagnostic_total_message_bytes", new Limit(2097152L, "JAVA_FRONTEND_DIAGNOSTIC_BUDGET")),
            Map.entry("frontend_argument_bytes", new Limit(131072L, "JAVA_LIMIT_FRONTEND_ARGUMENT_BYTES")),
            Map.entry("frontend_stdout", new Limit(268435456L, "JAVA_FRONTEND_OUTPUT_LIMIT")),
            Map.entry("frontend_stderr", new Limit(2097152L, "JAVA_FRONTEND_OUTPUT_LIMIT")),
            Map.entry("vir_canonical_bytes", new Limit(201326592L, "JAVA_LIMIT_VIR_CANONICAL_BYTES")),
            Map.entry("source_map_canonical_bytes", new Limit(33554432L, "JAVA_LIMIT_SOURCE_MAP_CANONICAL_BYTES")),
            Map.entry("source_manifest_canonical_bytes", new Limit(4194304L, "JAVA_LIMIT_SOURCE_MANIFEST_CANONICAL_BYTES")),
            Map.entry("parameter_slots", new Limit(255L, "JAVA_LIMIT_PARAMETER_SLOTS")));

    static long add(String name, long current, long increment, String phase) {
        Limit limit = DEFINITIONS.get(name);
        if (limit == null) throw new IllegalArgumentException("unknown Java counter");
        if (current < 0 || increment < 0 || current > limit.maximum()
                || increment > limit.maximum() - current) throw FrontendFailure.of(limit.code(), phase);
        return current + increment;
    }

    static void check(String name, long count, String phase) { add(name, 0, count, phase); }

    static void arguments(String[] arguments) {
        long bytes = 0;
        for (String argument : arguments) {
            bytes = add("frontend_argument_bytes", bytes, 1, "capture"); // terminating NUL
            for (int i = 0; i < argument.length();) {
                int scalar = argument.codePointAt(i);
                if (scalar >= 0xd800 && scalar <= 0xdfff)
                    throw FrontendFailure.of("JAVA_CAPTURE_PATH", "capture");
                bytes = add("frontend_argument_bytes", bytes,
                        scalar < 0x80 ? 1 : scalar < 0x800 ? 2 : scalar < 0x10000 ? 3 : 4, "capture");
                i += Character.charCount(scalar);
            }
        }
    }
}
