package mpk.java2vir;

import java.util.Map;
import java.util.Set;

/** Compiled closed definitions from the immutable Java profile; no runtime configuration. */
final class DiagnosticRegistry {
    private DiagnosticRegistry() {}
    record Definition(String status, String phase, int exitCode, String message) {}
    static final Set<String> PHASES = Set.of("capture", "source", "metadata", "typecheck", "subset", "lowering", "emission");
    static final Map<String, Definition> DEFINITIONS = Map.ofEntries(
            Map.entry("JAVA_CAPTURE_FILE_TYPE", new Definition("rejected", "capture", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CAPTURE_INVENTORY", new Definition("rejected", "capture", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CAPTURE_PATH", new Definition("rejected", "capture", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_DUPLICATE", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_HASH", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_IDENTITY", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_JSON", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_MISSING", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_OPERATOR", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_SHAPE", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_TYPE", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_CONTRACT_UNUSED", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_FRONTEND_DIAGNOSTIC_BUDGET", new Definition("frontend-error", "started_phase", 1, "Java frontend failed closed")),
            Map.entry("JAVA_FRONTEND_INTERNAL", new Definition("frontend-error", "started_phase", 1, "Java frontend failed closed")),
            Map.entry("JAVA_FRONTEND_OUTPUT_LIMIT", new Definition("frontend-error", "started_phase", 1, "Java frontend failed closed")),
            Map.entry("JAVA_FRONTEND_RESOURCE", new Definition("frontend-error", "started_phase", 1, "Java frontend failed closed")),
            Map.entry("JAVA_LIMIT_CANONICAL_METHOD_ID_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CFG_BLOCKS_PER_CLOSURE", new Definition("rejected", "lowering", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CFG_BLOCKS_PER_METHOD", new Definition("rejected", "lowering", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_CLAUSES", new Definition("rejected", "subset", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_DEPTH", new Definition("rejected", "subset", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_FILES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_FILE_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_NODES_PER_CLOSURE", new Definition("rejected", "subset", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_NODES_PER_METHOD", new Definition("rejected", "subset", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_CONTRACT_TOTAL_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_FRONTEND_ARGUMENT_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_INSTRUCTIONS_PER_CLOSURE", new Definition("rejected", "lowering", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_INSTRUCTIONS_PER_METHOD", new Definition("rejected", "lowering", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_METHOD_CLOSURE", new Definition("rejected", "subset", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_NORMALIZED_PATH_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_PARAMETER_SLOTS", new Definition("rejected", "subset", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SELECTED_METHODS", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SNAPSHOT_ENTRIES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SNAPSHOT_TOTAL_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SOURCE_FILES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SOURCE_FILE_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SOURCE_MANIFEST_CANONICAL_BYTES", new Definition("rejected", "emission", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SOURCE_MAP_CANONICAL_BYTES", new Definition("rejected", "emission", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SOURCE_TOTAL_BYTES", new Definition("rejected", "capture", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SYNTAX_DEPTH", new Definition("rejected", "source", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_SYNTAX_NODES", new Definition("rejected", "source", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LIMIT_VIR_CANONICAL_BYTES", new Definition("rejected", "emission", 3, "Java profile limit exceeded")),
            Map.entry("JAVA_LOWERING_CFG", new Definition("rejected", "lowering", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_LOWERING_CHECK_EXTRA", new Definition("rejected", "lowering", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_LOWERING_CHECK_MISSING", new Definition("rejected", "lowering", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_LOWERING_CHECK_ORDER", new Definition("rejected", "lowering", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_LOWERING_OPERATION", new Definition("rejected", "lowering", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_LOWERING_SHIFT_PATTERN", new Definition("rejected", "lowering", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SOURCE_DIAGNOSTIC", new Definition("source-error", "typecheck", 4, "Java source is invalid")),
            Map.entry("JAVA_SOURCE_ENCODING", new Definition("source-error", "source", 4, "Java source is invalid")),
            Map.entry("JAVA_SOURCE_MAP_EXTERNAL", new Definition("frontend-error", "emission", 1, "Java frontend failed closed")),
            Map.entry("JAVA_SOURCE_MAP_RANGE", new Definition("frontend-error", "emission", 1, "Java frontend failed closed")),
            Map.entry("JAVA_SOURCE_MAP_UTF16", new Definition("frontend-error", "emission", 1, "Java frontend failed closed")),
            Map.entry("JAVA_SOURCE_PARSE", new Definition("source-error", "source", 4, "Java source is invalid")),
            Map.entry("JAVA_SUBSET_ABRUPT", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_CALL", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_CONTROL_FLOW", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_CONVERSION", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_DECLARATION", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_IDENTIFIER", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_INITIALIZATION", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_LITERAL", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_OPERATION", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_PURITY", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_SUBSET_TYPE", new Definition("rejected", "subset", 3, "Java source is outside the frozen profile")),
            Map.entry("JAVA_TOOLCHAIN_ADAPTER", new Definition("frontend-error", "started_phase", 1, "Java frontend failed closed")),
            Map.entry("JAVA_TOOLCHAIN_ARCHIVE", new Definition("frontend-error", "metadata", 1, "Java frontend failed closed")),
            Map.entry("JAVA_TOOLCHAIN_COMPILER", new Definition("frontend-error", "metadata", 1, "Java frontend failed closed")),
            Map.entry("JAVA_TOOLCHAIN_FILE_MANAGER", new Definition("frontend-error", "started_phase", 1, "Java frontend failed closed")),
            Map.entry("JAVA_TOOLCHAIN_OPTIONS", new Definition("frontend-error", "metadata", 1, "Java frontend failed closed")),
            Map.entry("JAVA_TOOLCHAIN_REFERENCE", new Definition("frontend-error", "metadata", 1, "Java frontend failed closed")),
            Map.entry("JAVA_TOOLCHAIN_RUNTIME", new Definition("frontend-error", "metadata", 1, "Java frontend failed closed")));

    static Definition definition(String code) {
        Definition result = DEFINITIONS.get(code);
        if (result == null) throw new IllegalArgumentException("unknown Java diagnostic");
        return result;
    }

    static String phase(String code, String started) {
        if (!PHASES.contains(started)) throw new IllegalArgumentException("unknown Java phase");
        String owner = definition(code).phase();
        return owner.equals("started_phase") ? started : owner;
    }
}
