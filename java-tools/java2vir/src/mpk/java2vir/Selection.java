package mpk.java2vir;

import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeMap;
import java.util.regex.Pattern;

/** Typed selection received from the validated parent request; no discovery. */
record Selection(String compilation, List<String> sources, List<String> contracts, List<String> methods) {
    private static final String EXCLUDED = "abstract assert boolean break byte case catch char class const continue default do double else enum extends final finally float for goto if implements import instanceof int interface long native new package private protected public return short static strictfp super switch synchronized this throw throws transient try void volatile while _ true false null exports module non-sealed open opens permits provides record requires sealed to transitive uses var when with yield";
    private static final Pattern IDENTIFIER = Pattern.compile("[A-Za-z_][A-Za-z0-9_]*");
    private static final Pattern METHOD = Pattern.compile("([^:]+)::([^(:]+)\\(((?:boolean|int|long)(?:,(?:boolean|int|long))*)?\\)->(boolean|int|long)");

    Selection {
        if (compilation == null || compilation.length() > 64
                || !compilation.matches("[a-z][a-z0-9]*([._-][a-z0-9]+)*")) throw pathFailure();
        ordered(sources, "source_files");
        ordered(contracts, "contract_files");
        ordered(methods, "selected_methods");
        sources = List.copyOf(sources);
        contracts = List.copyOf(contracts);
        methods = List.copyOf(methods);
        for (String source : sources) {
            FrontendLimits.check("normalized_path_bytes", source.length(), "capture");
            if (!sourcePath(source)) throw pathFailure();
        }
        for (String contract : contracts) {
            FrontendLimits.check("normalized_path_bytes", contract.length(), "capture");
            if (!portablePath(contract) || !contract.startsWith("contracts/") || !contract.endsWith(".json")) throw pathFailure();
        }
        for (String method : methods) {
            FrontendLimits.check("canonical_method_id_bytes", method.length(), "capture");
            if (!methodId(method)) throw pathFailure();
        }
        expected(sources, contracts);
    }

    private static void ordered(List<String> values, String counter) {
        if (values == null) throw pathFailure();
        FrontendLimits.check(counter, values.size(), "capture");
        if (values.isEmpty()) throw pathFailure();
        String previous = "";
        for (String value : values) {
            if (value == null || previous.compareTo(value) >= 0) throw pathFailure();
            previous = value;
        }
    }

    static boolean identifier(String value) {
        return IDENTIFIER.matcher(value).matches() && !(" " + EXCLUDED + " ").contains(" " + value + " ");
    }

    static boolean methodId(String value) {
        if (value == null || value.length() > 1024) return false;
        var parsed = METHOD.matcher(value);
        if (!parsed.matches() || !identifier(parsed.group(2))) return false;
        int separator = parsed.group(1).lastIndexOf('.');
        return separator > 0 && packageName(parsed.group(1).substring(0, separator))
                && identifier(parsed.group(1).substring(separator + 1));
    }

    static boolean packageName(String value) {
        for (String part : value.split("\\.", -1)) if (!identifier(part)) return false;
        for (String prefix : List.of("java", "javax", "jdk", "sun", "com.sun"))
            if (value.equals(prefix) || value.startsWith(prefix + ".")) return false;
        return true;
    }

    static boolean sourcePath(String path) {
        if (!portablePath(path) || !path.startsWith("src/") || !path.endsWith(".java")) return false;
        String relative = path.substring(4, path.length() - 5);
        int separator = relative.lastIndexOf('/');
        return separator > 0 && packageName(relative.substring(0, separator).replace('/', '.'))
                && identifier(relative.substring(separator + 1))
                && List.of(relative.substring(0, separator).split("/", -1)).stream().allMatch(Selection::identifier);
    }

    static boolean portablePath(String path) {
        if (path == null || path.isEmpty() || path.length() > 1024) return false;
        for (String part : path.split("/", -1)) {
            if (part.isEmpty() || part.length() > 255 || part.endsWith(".")
                    || !part.matches("[A-Za-z0-9._-]+")) return false;
            String stem = part.split("\\.", -1)[0].toUpperCase(Locale.ROOT);
            if (List.of("CON", "PRN", "AUX", "NUL").contains(stem) || stem.matches("(COM|LPT)[1-9]")) return false;
        }
        return true;
    }

    Map<String, Boolean> expected() { return expected(sources, contracts); }

    private static Map<String, Boolean> expected(List<String> sources, List<String> contracts) {
        var entries = new TreeMap<String, Boolean>();
        var folded = new TreeMap<String, String>();
        for (List<String> paths : List.of(sources, contracts)) for (String path : paths) {
            boolean directory = false;
            for (;;) {
                Boolean old = entries.get(path);
                String alias = folded.get(path.toLowerCase(Locale.ROOT));
                if ((old != null && (!directory || !old)) || (alias != null && !alias.equals(path))) throw pathFailure();
                if (old == null) {
                    FrontendLimits.check("snapshot_entries", (long) entries.size() + 1, "capture");
                    entries.put(path, directory);
                    folded.put(path.toLowerCase(Locale.ROOT), path);
                }
                int slash = path.lastIndexOf('/');
                if (slash < 0) break;
                path = path.substring(0, slash);
                directory = true;
            }
        }
        return Map.copyOf(entries);
    }

    Map<String, Object> envelope() {
        return Map.of("schema", "mpk.selection.java_methods.v0", "value", Map.of(
                "compilation", compilation, "sources", sources, "contracts", contracts, "methods", methods));
    }

    private static FrontendFailure pathFailure() { return FrontendFailure.of("JAVA_CAPTURE_PATH", "capture"); }
}
