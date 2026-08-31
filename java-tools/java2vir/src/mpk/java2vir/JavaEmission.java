package mpk.java2vir;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import static mpk.java2vir.JavaIr.*;

/** Private artifact assembly. T07 must supply a validated release identity; Main stays inactive. */
final class JavaEmission {
    private JavaEmission() {}
    static final String JDK_ARCHIVE_SHA256 = "dbb698396d478e7fa2b1e50f4103324b2a99b90569ee27c33f2261f9215cf41e";
    private static final List<Map<String, Object>> COMPONENTS = List.of(
            content("hotspot", "5e0fb6e83a5676090f28ff0b453e9199df06af1ff90d473c1b9837e41096114c"),
            content("jdk-modules", "6b11db4c84e8ac3b9500397753e02ad03450e904fa8656eb8fd2a2197e536b57"),
            content("jdk-release", "44be64b383baa18668afefbe9a780ae3a9d730a066eaaa92500f77bd1e4b934c"));

    // No defaults, bundle discovery or fabricated installed-release tuple. The
    // private conformance harness explicitly injects test identities until T07.
    record Identity(String releaseRegistrySha256, String toolchainBundleId, String distributionSha256,
                    String frontendBundleId, String frontendSha256) {
        Identity {
            for (String hash : List.of(releaseRegistrySha256, distributionSha256, frontendSha256))
                if (!hash.matches("[0-9a-f]{64}")) throw internal();
            for (String id : List.of(toolchainBundleId, frontendBundleId))
                if (id.length() > 128 || !id.matches("[a-z][a-z0-9]*([._-][a-z0-9]+)*")) throw internal();
        }
    }

    static byte[] emit(CapturedSnapshot snapshot, Program program, Identity identity) {
        JavaLoweringValidation.validate(program);
        validateInputs(snapshot, program);
        String unit = snapshot.selection().compilation();
        Map<String, Object> vir = CanonicalJson.artifact(Map.of("schema", "mpk.vir.v1", "semantic_context", Protocol.semanticContext(),
                "units", List.of(Map.of("id", unit, "name", unit, "type_decls", List.of(), "const_decls", List.of(),
                        "functions", program.functions().stream().map(function -> function.json(unit)).toList()))),
                "vir_hash", "MPK-VIR-1.0", "vir_canonical_bytes");
        String virHash = (String) vir.get("vir_hash");
        Map<String, Object> map = JavaSourceMaps.emit(program, virHash);
        String mapHash = (String) map.get("source_map_hash");
        Map<String, Object> manifest = manifest(snapshot, identity, virHash, mapHash);
        // No stream or file is exposed until every graph, origin, binding and
        // canonical byte budget (including this final LF) has passed.
        return CanonicalJson.encode(Map.of("schema", "mpk.frontend.cli.v1", "status", "ir-lowered", "phase", "emission",
                "semantic_context", Protocol.semanticContext(), "selection", snapshot.selection().envelope(),
                "diagnostics", List.of(), "rejected_features", List.of(),
                "ir", Map.of("schema", "mpk.vir.v1", "sha256", virHash, "value", vir),
                "source_map", map, "source_manifest", manifest), "frontend_stdout", true);
    }

    private static void validateInputs(CapturedSnapshot snapshot, Program program) {
        var admitted = program.admitted();
        if (!snapshot.selection().equals(admitted.closure().selection())
                || !admitted.contracts().selectionSha256().equals(JavaContracts.typedHash("MPK-JAVA-SELECTION-0.1", snapshot.selection().envelope()))) throw internal();
        for (SourceText source : admitted.closure().sources()) {
            var input = snapshot.file(source.path());
            if (!input.source() || !Arrays.equals(input.bytes(), source.text().getBytes(StandardCharsets.UTF_8))) throw internal();
        }
        for (var attached : admitted.contracts().methods()) {
            var input = snapshot.file(attached.path());
            if (input.source() || !input.sha256().equals(attached.rawInputSha256())) throw internal();
        }
    }

    private static Map<String, Object> manifest(CapturedSnapshot snapshot, Identity identity, String virHash, String mapHash) {
        List<Map<String, Object>> inputs = snapshot.inputs().stream().map(input -> Map.<String, Object>of(
                "normalized_path", input.path(), "kind", input.source() ? "source" : "contract",
                "size_bytes", input.size(), "sha256", input.sha256())).toList();
        var manifest = new TreeMap<String, Object>();
        manifest.put("schema", "mpk.source_manifest.v1"); manifest.put("semantic_context", Protocol.semanticContext());
        manifest.put("selection", snapshot.selection().envelope()); manifest.put("limit_profile", "mpk.vir.limits.v0");
        manifest.put("release_registry", Map.of("schema", "mpk.release.bundle_registry.v1", "id", "mpk.release.registry.v1",
                "registry_sha256", identity.releaseRegistrySha256()));
        manifest.put("toolchain", Map.of("bundle_id", identity.toolchainBundleId(),
                "distribution_sha256", identity.distributionSha256(), "components", COMPONENTS));
        manifest.put("frontend", Map.of("bundle_id", identity.frontendBundleId(), "name", "java2vir", "version", "0.1.0",
                "binary_sha256", identity.frontendSha256(), "subordinate_binaries", List.of()));
        String unit = snapshot.selection().compilation();
        manifest.put("units", List.of(Map.of("identity", unit, "name", unit, "kind", "compilation")));
        manifest.put("target", Map.of("id", "linux-x64", "pointer_width", 64));
        manifest.put("inputs", inputs);
        manifest.put("input_set_hash", CanonicalJson.hash("MPK-INPUT-SET-0.1", inputs, "source_manifest_canonical_bytes"));
        manifest.put("vir_hash", virHash); manifest.put("source_map_hash", mapHash);
        return CanonicalJson.artifact(manifest, "source_manifest_hash", "MPK-SOURCE-MANIFEST-1.0", "source_manifest_canonical_bytes");
    }
    private static Map<String, Object> content(String name, String hash) {
        return Map.of("kind", "content", "name", name, "release", "25.0.4.1+1", "content_sha256", hash);
    }
    private static FrontendFailure internal() { return FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "emission"); }
}
