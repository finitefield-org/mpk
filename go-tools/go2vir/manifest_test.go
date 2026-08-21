package main

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"testing"
)

func TestManifestMatchesSharedFrontendStageVector(t *testing.T) {
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/source-manifest-v0.json"))
	caseObject := findCase(t, arrayField(t, vectors, "manifest_cases"), "manifest.valid_go_frontend_stage")
	expectedValue := objectField(t, caseObject, "input")
	var expected sourceManifest
	unmarshalJSONValue(t, expectedValue, &expected)

	fixtureBytes := make(map[string][]byte)
	for _, value := range arrayField(t, vectors, "fixture_inputs") {
		fixture := asObject(t, value, "fixture input")
		decoded, err := base64.StdEncoding.DecodeString(stringField(t, fixture, "base64"))
		if err != nil {
			t.Fatalf("decode fixture input: %v", err)
		}
		path := stringField(t, fixture, "normalized_path")
		if int64(len(decoded)) != intField(t, fixture, "size_bytes") || sha256Hex(decoded) != stringField(t, fixture, "sha256") {
			t.Fatalf("fixture input %s has stale bytes", path)
		}
		fixtureBytes[path] = decoded
	}
	capturedInputs := make([]capturedInput, 0, len(expected.Inputs))
	var sourcePaths []string
	for _, input := range expected.Inputs {
		content, exists := fixtureBytes[input.NormalizedPath]
		if !exists {
			t.Fatalf("missing bytes for expected manifest input %s", input.NormalizedPath)
		}
		capturedInputs = append(capturedInputs, capturedInput{
			Kind: input.Kind, NormalizedPath: input.NormalizedPath, Bytes: content, SHA256: sha256Hex(content),
		})
		if input.Kind == sourceInputKind {
			sourcePaths = append(sourcePaths, input.NormalizedPath)
		}
	}
	loaded := packageLoadResult{Packages: []loadedPackage{{
		PackagePath:     expected.Units[0].Identity,
		Name:            expected.Units[0].Name,
		GoFiles:         sourcePaths,
		CompiledGoFiles: sourcePaths,
	}}}
	request := lowerRequest{
		Package:                     expected.Selection.Package,
		Function:                    expected.Selection.Function,
		SemanticProfile:             expected.SemanticProfile,
		Target:                      expected.SemanticParameters.TargetID,
		FrontendBundleID:            expected.Frontend.BundleID,
		FrontendSHA256:              expected.Frontend.BinarySHA256,
		ReleaseRegistryID:           expected.ReleaseRegistry.ID,
		ReleaseRegistrySHA256:       expected.ReleaseRegistry.RegistrySHA256,
		ToolchainBundleID:           expected.Toolchain.BundleID,
		ToolchainDistributionSHA256: expected.Toolchain.DistributionSHA256,
	}
	selection := validatedLauncherSelection{
		Registry: expected.ReleaseRegistry, Frontend: expected.Frontend, Toolchain: expected.Toolchain,
		Target: expected.Target, LimitProfileID: expected.LimitProfile,
	}
	capture := sourceCapture{ModulePath: expected.Selection.Package, SelectedPackage: expected.Selection.Package, Inputs: capturedInputs}
	actual, canonical, err := buildSourceManifest(request, capture, loaded, selection, expected.VIRHash, expected.SourceMapHash)
	if err != nil {
		t.Fatalf("build shared manifest vector: %v", err)
	}
	expectedCanonical, err := canonicalJSONValue(expectedValue)
	if err != nil {
		t.Fatalf("canonicalize expected manifest: %v", err)
	}
	if !bytes.Equal(canonical, expectedCanonical) {
		t.Fatalf("manifest canonical bytes differ from shared vector:\n got: %s\nwant: %s", canonical, expectedCanonical)
	}
	if actual.InputSetHash != expected.InputSetHash || actual.SourceManifestHash != expected.SourceManifestHash {
		t.Fatalf("manifest hashes = %s/%s, want %s/%s", actual.InputSetHash, actual.SourceManifestHash, expected.InputSetHash, expected.SourceManifestHash)
	}
}

func TestManifestHashExcludesOnlyTheRootSelfHash(t *testing.T) {
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/source-manifest-v0.json"))
	caseObject := findCase(t, arrayField(t, vectors, "manifest_cases"), "manifest.valid_go_frontend_stage")
	value := cloneJSONValue(t, objectField(t, caseObject, "input"))
	payload, err := withoutRootField(value, "source_manifest_hash")
	if err != nil {
		t.Fatalf("remove source manifest self hash: %v", err)
	}
	digest, err := hashCanonicalJSON(sourceManifestDomain, payload)
	if err != nil {
		t.Fatalf("hash source manifest payload: %v", err)
	}
	if want := stringField(t, asObject(t, value, "manifest"), "source_manifest_hash"); digest != want {
		t.Fatalf("source manifest hash = %s, want %s", digest, want)
	}
}

func unmarshalJSONValue(t *testing.T, value jsonValue, destination any) {
	t.Helper()
	canonical, err := canonicalJSONValue(value)
	if err != nil {
		t.Fatalf("canonicalize typed test value: %v", err)
	}
	if err := json.Unmarshal(canonical, destination); err != nil {
		t.Fatalf("unmarshal typed test value: %v", err)
	}
}
