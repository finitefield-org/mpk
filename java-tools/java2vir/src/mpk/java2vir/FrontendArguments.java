package mpk.java2vir;

import java.util.ArrayList;
import java.util.List;

/** The frozen ordered argument grammar; no caller-selected runtime paths or JVM options. */
final class FrontendArguments {
    private FrontendArguments() {}
    record Request(Selection selection, JavaEmission.Identity identity, List<String> arguments) {
        Request { arguments = List.copyOf(arguments); }
    }
    static Request parse(String[] arguments) {
        FrontendLimits.arguments(arguments);
        var cursor = new Cursor(arguments);
        cursor.exact("lower"); cursor.exact("/mpk/source");
        cursor.exact("--semantic-profile"); cursor.exact("mpk.java.scalar.v0");
        cursor.exact("--target"); cursor.exact("linux-x64");
        String compilation = cursor.option("--compilation");
        List<String> sources = cursor.repeated("--source", 256);
        List<String> contracts = cursor.repeated("--contract", 128);
        List<String> methods = cursor.repeated("--method", 32);
        cursor.exact("--profile-registry-id"); cursor.exact("mpk.semantic_profile.registry.v1");
        cursor.exact("--profile-registry-revision"); cursor.exact("3");
        cursor.exact("--profile-registry-sha256"); cursor.exact("fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557");
        cursor.exact("--profile-entry-sha256"); cursor.exact("0d80d13f97c45557fa9978eccc2545ffdb3fc1b93a26856b365a9be200470301");
        cursor.exact("--frontend-bundle-id"); cursor.exact(JavaRelease.FRONTEND_ID);
        String frontendHash = cursor.hash("--frontend-sha256");
        cursor.exact("--release-registry-id"); cursor.exact("mpk.release.registry.v1");
        String registryHash = cursor.hash("--release-registry-sha256");
        cursor.exact("--toolchain-bundle-id"); cursor.exact(JavaRelease.TOOLCHAIN_ID);
        cursor.exact("--toolchain-root"); cursor.exact("/mpk/toolchain");
        cursor.exact("--toolchain-distribution-sha256"); cursor.exact(JavaRelease.DISTRIBUTION);
        if (cursor.position != arguments.length) throw invalid();
        return new Request(new Selection(compilation, sources, contracts, methods),
                new JavaEmission.Identity(registryHash, JavaRelease.TOOLCHAIN_ID, JavaRelease.DISTRIBUTION,
                        JavaRelease.FRONTEND_ID, frontendHash), List.of(arguments));
    }
    private static final class Cursor {
        final String[] values;
        int position;
        Cursor(String[] values) { this.values = values; }
        String next() {
            if (position == values.length) throw invalid();
            return values[position++];
        }
        void exact(String value) { if (!next().equals(value)) throw invalid(); }
        String option(String name) { exact(name); return next(); }
        String hash(String name) {
            String value = option(name);
            if (!value.matches("[0-9a-f]{64}")) throw invalid();
            return value;
        }
        List<String> repeated(String name, int maximum) {
            var result = new ArrayList<String>();
            while (position < values.length && values[position].equals(name)) {
                if (result.size() == maximum) throw invalid();
                position++; result.add(next());
            }
            if (result.isEmpty()) throw invalid();
            return result;
        }
    }
    private static IllegalArgumentException invalid() { return new IllegalArgumentException("closed Java argument grammar"); }
}
