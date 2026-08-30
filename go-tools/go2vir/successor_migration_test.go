package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"testing"
)

const updateSuccessorGoCorpusEnv = "MPK_UPDATE_GO_CORPUS"
const activeGoFrontendBundleID = "frontend.go.go2vir.candidate.v1"
const activeGoToolchainBundleID = "toolchain.go.go1.25.0.linux-amd64.candidate.v1"

type successorRegistryFixture struct {
	Schema          string                           `json:"schema"`
	ID              string                           `json:"id"`
	RegistrySHA256  string                           `json:"registry_sha256"`
	ProfileRegistry successorProfileRegistryIdentity `json:"profile_registry"`
	FrontendBundles []struct {
		BundleID string `json:"bundle_id"`
		Name     string `json:"name"`
		Version  string `json:"version"`
		Main     struct {
			BinarySHA256 string `json:"binary_sha256"`
		} `json:"main"`
		SubordinateBinaries []subordinateIdentity `json:"subordinate_binaries"`
	} `json:"frontend_bundles"`
	ToolchainBundles []struct {
		BundleID           string              `json:"bundle_id"`
		DistributionSHA256 string              `json:"distribution_sha256"`
		Components         []componentIdentity `json:"components"`
	} `json:"toolchain_bundles"`
	Tuples []struct {
		LimitProfileID    string `json:"limit_profile_id"`
		FrontendBundleID  string `json:"frontend_bundle_id"`
		ToolchainBundleID string `json:"toolchain_bundle_id"`
	} `json:"tuples"`
}

type successorNegativeResult struct {
	ID         string
	SourceRoot string
	SourcePath string
	Code       string
	Message    string
	Status     string
	Phase      string
	Envelope   []byte
}

func TestActiveGoCorpus(t *testing.T) {
	assertSuccessorArgumentContract(t)
	reference, registry := successorManifestReference(t)
	activeIndex := readActiveFrontendIndex(t)
	corpusCases := frontendCorpusCases()
	if len(activeIndex.Cases) != len(corpusCases) {
		t.Fatalf("active positive corpus has %d cases, want %d", len(activeIndex.Cases), len(corpusCases))
	}
	activeByID := make(map[string]activeFrontendIndexEntry, len(activeIndex.Cases))
	for _, entry := range activeIndex.Cases {
		if _, duplicate := activeByID[entry.ID]; duplicate {
			t.Fatalf("active positive corpus duplicates %s", entry.ID)
		}
		activeByID[entry.ID] = entry
	}

	first := make(map[string]activeFrontendArtifacts)
	second := make(map[string]activeFrontendArtifacts)
	expectedFixturePaths := make(map[string]struct{}, len(corpusCases)*4+len(activeIndex.NegativeCases)+1)
	indexCases := make([]jsonValue, 0, len(corpusCases))
	for _, corpusCase := range corpusCases {
		first[corpusCase.ID] = generateFrontendCorpusCase(t, corpusCase, reference)
		second[corpusCase.ID] = generateFrontendCorpusCase(t, corpusCase, reference)
		assertActiveFrontendEqual(t, corpusCase.ID, first[corpusCase.ID], second[corpusCase.ID])
		active, exists := activeByID[corpusCase.ID]
		if !exists {
			t.Fatalf("active corpus omitted %s", corpusCase.ID)
		}
		artifacts := first[corpusCase.ID]
		base := "frontend/" + corpusCase.ID
		files := []struct {
			kind    string
			name    string
			content []byte
		}{
			{"frontend_envelope", "frontend-envelope.json", artifacts.Envelope},
			{"vir", "vir.json", artifacts.VIR},
			{"source_map", "source-map.json", artifacts.Map},
			{"source_manifest_frontend", "source-manifest.frontend.json", artifacts.Manifest},
		}
		indexedArtifacts := make([]jsonValue, 0, len(files))
		for _, file := range files {
			path := base + "/" + file.name
			assertSuccessorFixture(t, expectedFixturePaths, path, file.content)
			indexedArtifacts = append(indexedArtifacts, map[string]jsonValue{
				"kind":   file.kind,
				"path":   path,
				"sha256": sha256Hex(file.content),
				"bytes":  int64(len(file.content)),
			})
			if corpusCase.ExamplePath != "" {
				assertFixtureAt(t, repoPath("examples/"+corpusCase.ExamplePath+"/"+file.name), file.content)
			}
		}
		activeArtifacts := activeArtifactBytes(t, active)
		sourceBehaviorEqual := bytes.Equal(
			virSemanticProjection(t, activeArtifacts["vir"]),
			virSemanticProjection(t, artifacts.VIR),
		)
		requiredChecksEqual := bytes.Equal(
			requiredCheckProjection(t, activeArtifacts["vir"]),
			requiredCheckProjection(t, artifacts.VIR),
		)
		vcInputIntentEqual := bytes.Equal(
			vcInputProjection(t, activeArtifacts["source_map"], activeArtifacts["source_manifest_frontend"]),
			vcInputProjection(t, artifacts.Map, artifacts.Manifest),
		)
		diagnosticsEqual := bytes.Equal(
			diagnosticProjection(t, activeArtifacts["frontend_envelope"]),
			diagnosticProjection(t, artifacts.Envelope),
		)
		if !sourceBehaviorEqual || !requiredChecksEqual || !vcInputIntentEqual || !diagnosticsEqual {
			t.Fatalf(
				"%s has a semantic migration difference: source=%t checks=%t vc=%t diagnostics=%t",
				corpusCase.ID,
				sourceBehaviorEqual,
				requiredChecksEqual,
				vcInputIntentEqual,
				diagnosticsEqual,
			)
		}
		selection := successorSelection(goSelection{Package: corpusCase.ImportPath, Function: corpusCase.SelectedFunction})
		entry := map[string]jsonValue{
			"id":              corpusCase.ID,
			"source_root":     corpusCase.SourceRoot,
			"source_path":     corpusCase.SourcePath,
			"selection":       selection,
			"function_count":  int64(artifacts.Function),
			"frontend_status": "ir-lowered",
			"artifacts":       indexedArtifacts,
		}
		if corpusCase.ExamplePath != "" {
			entry["example_path"] = "examples/" + corpusCase.ExamplePath
			contextBytes, err := canonicalJSON(fixedSuccessorSemanticContext())
			if err != nil {
				t.Fatalf("canonical semantic context for %s: %v", corpusCase.ID, err)
			}
			selectionBytes, err := canonicalJSON(selection)
			if err != nil {
				t.Fatalf("canonical selection for %s: %v", corpusCase.ID, err)
			}
			exampleRoot := "examples/" + corpusCase.ExamplePath + "/"
			assertFixtureAt(t, repoPath(exampleRoot+"mpk-semantic-context.json"), contextBytes)
			assertFixtureAt(t, repoPath(exampleRoot+"mpk-selection.json"), selectionBytes)
		}
		indexCases = append(indexCases, entry)
	}

	firstNegative := successorNegativeCorpus(t)
	secondNegative := successorNegativeCorpus(t)
	if !reflect.DeepEqual(firstNegative, secondNegative) {
		t.Fatal("negative successor corpus changed between two generations")
	}
	indexNegative := make([]jsonValue, 0, len(firstNegative))
	if len(activeIndex.NegativeCases) != len(firstNegative) {
		t.Fatalf("active negative corpus has %d cases, want %d", len(activeIndex.NegativeCases), len(firstNegative))
	}
	activeNegative := make(map[string]activeNegativeIndexEntry, len(activeIndex.NegativeCases))
	for _, entry := range activeIndex.NegativeCases {
		if _, duplicate := activeNegative[entry.ID]; duplicate {
			t.Fatalf("active negative corpus duplicates %s", entry.ID)
		}
		activeNegative[entry.ID] = entry
	}
	for _, result := range firstNegative {
		active, exists := activeNegative[result.ID]
		diagnosticsEqual := exists &&
			active.Code == result.Code && active.Outcome == "rejected" &&
			result.Status == "rejected" && result.Phase == "subset" && result.Message == activeNegativeMessage(result.Code)
		if !diagnosticsEqual {
			t.Fatalf("negative diagnostic changed for %s", result.ID)
		}
		path := "negative/" + result.ID + "/frontend-envelope.json"
		assertSuccessorFixture(t, expectedFixturePaths, path, result.Envelope)
		indexNegative = append(indexNegative, map[string]jsonValue{
			"id":          result.ID,
			"source_root": result.SourceRoot,
			"source_path": result.SourcePath,
			"outcome":     result.Status,
			"phase":       result.Phase,
			"code":        result.Code,
			"message":     result.Message,
			"artifact": map[string]jsonValue{
				"kind":   "frontend_envelope",
				"path":   path,
				"sha256": sha256Hex(result.Envelope),
				"bytes":  int64(len(result.Envelope)),
			},
		})
	}

	index := map[string]jsonValue{
		"schema":                "mpk.go_vir_frontend_corpus.v1",
		"update_command":        "MPK_UPDATE_GO_CORPUS=1 go test -count=1 -run TestActiveGoCorpus",
		"deterministic_runs":    int64(2),
		"semantic_context":      fixedSuccessorSemanticContext(),
		"release_registry":      releaseRegistryIdentity{Schema: registry.Schema, ID: registry.ID, RegistrySHA256: registry.RegistrySHA256},
		"positive_source_count": int64(len(indexCases)),
		"negative_source_count": int64(len(indexNegative)),
		"cases":                 indexCases,
		"negative_cases":        indexNegative,
	}
	indexBytes, err := canonicalJSON(index)
	if err != nil {
		t.Fatalf("canonical successor index: %v", err)
	}
	assertSuccessorFixture(t, expectedFixturePaths, "frontend-index.json", indexBytes)

	assertSuccessorFixtureSet(t, expectedFixturePaths)
}

func activeNegativeMessage(code string) string {
	switch code {
	case "GO_SUBSET_GENERICS":
		return "generic functions are outside the Go profile"
	case "GO_SUBSET_FLOAT", "GO_SUBSET_MAPS", "GO_SUBSET_POINTER", "GO_SUBSET_STRING":
		return "type is outside the fixed-width Go profile"
	case "GO_CONTRACT_ENSURES":
		return "contract ensures must be nonempty"
	default:
		return ""
	}
}

func assertSuccessorArgumentContract(t *testing.T) {
	t.Helper()
	if frontendCLISchema != "mpk.frontend.cli.v1" ||
		virSchema != "mpk.vir.v1" || virHashDomain != "MPK-VIR-1.0" ||
		contractDomain != "MPK-CONTRACT-1.0" ||
		sourceMapSchema != "mpk.source_map.v1" || sourceMapDomain != "MPK-SOURCE-MAP-1.0" ||
		sourceManifestSchema != "mpk.source_manifest.v1" || sourceManifestDomain != "MPK-SOURCE-MANIFEST-1.0" ||
		registryID != "mpk.release.registry.v1" ||
		registeredFrontendVersion != "go1.25.0-profile-v1-staging" {
		t.Fatal("staged binary retained a predecessor artifact or release identity")
	}
	arguments := []string{
		"lower", logicalSourceRoot,
		"--package", "example.com/mpk/vector",
		"--semantic-profile", goSemanticProfile,
		"--target", goTarget,
		"--function", "example.com/mpk/vector.Identity",
		"--profile-registry-id", successorProfileRegistryID,
		"--profile-registry-revision", successorProfileRegistryRevisionArgument,
		"--profile-registry-sha256", successorProfileRegistrySHA256,
		"--profile-entry-sha256", successorGoProfileEntrySHA256,
		"--frontend-bundle-id", "frontend.go.test.v1",
		"--frontend-sha256", zeroSHA256(),
		"--release-registry-id", registryID,
		"--release-registry-sha256", zeroSHA256(),
		"--toolchain-bundle-id", "toolchain.go.test.v1",
		"--toolchain-root", logicalToolchain,
		"--toolchain-distribution-sha256", zeroSHA256(),
	}
	request, err := parseLowerArguments(arguments)
	if err != nil {
		t.Fatalf("successor arguments rejected: %v", err)
	}
	if request.ProfileEntrySHA256 != successorGoProfileEntrySHA256 || request.ReleaseRegistryID != "mpk.release.registry.v1" {
		t.Fatal("successor arguments did not retain exact assertions")
	}
	mutated := append([]string(nil), arguments...)
	mutated[13] = "1"
	if _, err := parseLowerArguments(mutated); err == nil {
		t.Fatal("predecessor semantic-registry revision was accepted")
	}
}

func successorManifestReference(t *testing.T) (sourceManifest, successorRegistryFixture) {
	t.Helper()
	path := repoPath("release/bundles/bundle-registry.json")
	raw := mustReadFile(t, path)
	var registry successorRegistryFixture
	if err := json.Unmarshal(raw, &registry); err != nil {
		t.Fatalf("decode successor Go registry: %v", err)
	}
	if registry.Schema != "mpk.release.bundle_registry.v1" || registry.ID != registryID || len(registry.FrontendBundles) != 3 || len(registry.ToolchainBundles) != 3 || len(registry.Tuples) != 4 {
		t.Fatal("active registry shape is not the complete successor release")
	}
	frontend := registry.FrontendBundles[0]
	for _, candidate := range registry.FrontendBundles {
		if candidate.BundleID == activeGoFrontendBundleID {
			frontend = candidate
		}
	}
	toolchain := registry.ToolchainBundles[0]
	for _, candidate := range registry.ToolchainBundles {
		if candidate.BundleID == activeGoToolchainBundleID {
			toolchain = candidate
		}
	}
	tuple := registry.Tuples[0]
	for _, candidate := range registry.Tuples {
		if candidate.FrontendBundleID == frontend.BundleID && candidate.ToolchainBundleID == toolchain.BundleID {
			tuple = candidate
		}
	}
	if tuple.FrontendBundleID != frontend.BundleID || tuple.ToolchainBundleID != toolchain.BundleID {
		t.Fatal("successor Go tuple is not linked to its candidate bundles")
	}
	return sourceManifest{
		ReleaseRegistry: releaseRegistryIdentity{Schema: registry.Schema, ID: registry.ID, RegistrySHA256: registry.RegistrySHA256},
		Frontend: frontendIdentity{
			BundleID:            frontend.BundleID,
			Name:                frontend.Name,
			Version:             frontend.Version,
			BinarySHA256:        frontend.Main.BinarySHA256,
			SubordinateBinaries: frontend.SubordinateBinaries,
		},
		Toolchain: toolchainIdentity{
			BundleID:           toolchain.BundleID,
			DistributionSHA256: toolchain.DistributionSHA256,
			Components:         toolchain.Components,
		},
		Target:       targetIdentity{ID: goTarget, PointerWidth: goPointerWidth, LanguageConfiguration: fixedGoConfiguration()},
		LimitProfile: tuple.LimitProfileID,
	}, registry
}

func readActiveFrontendIndex(t *testing.T) activeFrontendIndex {
	t.Helper()
	var index activeFrontendIndex
	if err := json.Unmarshal(mustReadFile(t, repoPath("fixtures/vir-go/frontend-index.json")), &index); err != nil {
		t.Fatalf("decode active frontend index: %v", err)
	}
	return index
}

func activeArtifactBytes(t *testing.T, entry activeFrontendIndexEntry) map[string][]byte {
	t.Helper()
	artifacts := make(map[string][]byte, len(entry.Artifacts))
	for _, artifact := range entry.Artifacts {
		content := mustReadFile(t, repoPath("fixtures/vir-go/"+artifact.Path))
		if os.Getenv(updateSuccessorGoCorpusEnv) == "" &&
			(len(content) != artifact.Bytes || sha256Hex(content) != artifact.SHA256) {
			t.Fatalf("active fixture identity changed: %s", artifact.Path)
		}
		artifacts[artifact.Kind] = content
	}
	return artifacts
}

func successorNegativeCorpus(t *testing.T) []successorNegativeResult {
	t.Helper()
	results := make([]successorNegativeResult, 0, len(negativeCorpusCases()))
	for _, corpusCase := range negativeCorpusCases() {
		source := mustReadFile(t, repoPath(corpusCase.SourceRoot+"/"+corpusCase.SourcePath))
		loaded, capture := typedSinglePackage(t, corpusCase.ImportPath, corpusCase.SourcePath, string(source))
		result, findings := lowerLoadedGo(loaded)
		if len(findings) == 0 {
			for _, path := range corpusCase.Contracts {
				content := mustReadFile(t, repoPath(corpusCase.SourceRoot+"/"+path))
				capture.Inputs = append(capture.Inputs, capturedInput{Kind: contractInputKind, NormalizedPath: path, Bytes: content, SHA256: sha256Hex(content)})
			}
			findings = attachContracts(&result.Module, capture, loaded)
		}
		if len(findings) == 0 {
			t.Fatalf("negative case %s was accepted", corpusCase.ID)
		}
		request := lowerRequest{Package: corpusCase.ImportPath, Function: corpusCase.Function, SemanticProfile: goSemanticProfile, Target: goTarget}
		envelope, err := loweringFindingsEnvelope(request, findings)
		if err != nil {
			t.Fatalf("normalize negative case %s: %v", corpusCase.ID, err)
		}
		encoded, err := canonicalJSON(envelope)
		if err != nil {
			t.Fatalf("encode negative case %s: %v", corpusCase.ID, err)
		}
		issues := envelope.RejectedFeatures
		if len(issues) == 0 {
			issues = envelope.Diagnostics
		}
		if len(issues) == 0 || issues[0].Code != corpusCase.ExpectedCode {
			t.Fatalf("negative case %s changed diagnostic", corpusCase.ID)
		}
		results = append(results, successorNegativeResult{
			ID:         corpusCase.ID,
			SourceRoot: corpusCase.SourceRoot,
			SourcePath: corpusCase.SourcePath,
			Code:       issues[0].Code,
			Message:    issues[0].Message,
			Status:     envelope.Status,
			Phase:      envelope.Phase,
			Envelope:   encoded,
		})
	}
	return results
}

func virSemanticProjection(t *testing.T, raw []byte) []byte {
	t.Helper()
	root := strictObjectBytes(t, raw)
	deleteIdentity(root)
	delete(root, "schema")
	delete(root, "vir_hash")
	for _, unitValue := range arrayValue(t, root["units"]) {
		unit := objectValue(t, unitValue)
		for _, functionValue := range arrayValue(t, unit["functions"]) {
			function := objectValue(t, functionValue)
			contract := objectValue(t, function["contracts"])
			deleteIdentity(contract)
			delete(contract, "contract_hash")
			for _, blockValue := range arrayValue(t, function["blocks"]) {
				block := objectValue(t, blockValue)
				for _, instructionValue := range arrayValue(t, block["instructions"]) {
					delete(objectValue(t, instructionValue), "contract_hash")
				}
			}
		}
	}
	return canonicalValue(t, root)
}

func requiredCheckProjection(t *testing.T, raw []byte) []byte {
	t.Helper()
	root := strictObjectBytes(t, raw)
	checks := make([]jsonValue, 0)
	for _, unitValue := range arrayValue(t, root["units"]) {
		unit := objectValue(t, unitValue)
		for _, functionValue := range arrayValue(t, unit["functions"]) {
			function := objectValue(t, functionValue)
			for _, blockValue := range arrayValue(t, function["blocks"]) {
				block := objectValue(t, blockValue)
				for _, instructionValue := range arrayValue(t, block["instructions"]) {
					instruction := objectValue(t, instructionValue)
					checks = append(checks, map[string]jsonValue{
						"function":      function["id"],
						"block":         block["label"],
						"instruction":   instruction["id"],
						"safety_checks": instruction["safety_checks"],
					})
				}
			}
		}
	}
	return canonicalValue(t, checks)
}

func vcInputProjection(t *testing.T, mapBytes, manifestBytes []byte) []byte {
	t.Helper()
	mapRoot := strictObjectBytes(t, mapBytes)
	manifest := strictObjectBytes(t, manifestBytes)
	selection := manifest["selection"]
	if envelope, ok := selection.(map[string]jsonValue); ok && envelope["value"] != nil {
		selection = envelope["value"]
	}
	target := objectValue(t, manifest["target"])
	delete(target, "language_configuration")
	projection := map[string]jsonValue{
		"entries":        mapRoot["entries"],
		"selection":      selection,
		"limit_profile":  manifest["limit_profile"],
		"units":          manifest["units"],
		"target":         target,
		"inputs":         manifest["inputs"],
		"input_set_hash": manifest["input_set_hash"],
	}
	return canonicalValue(t, projection)
}

func diagnosticProjection(t *testing.T, raw []byte) []byte {
	t.Helper()
	root := strictObjectBytes(t, raw)
	return canonicalValue(t, map[string]jsonValue{
		"status":            root["status"],
		"phase":             root["phase"],
		"rejected_features": root["rejected_features"],
		"diagnostics":       root["diagnostics"],
	})
}

func deleteIdentity(object map[string]jsonValue) {
	delete(object, "source_language")
	delete(object, "semantic_profile")
	delete(object, "semantic_parameters")
	delete(object, "semantic_context")
}

func strictObjectBytes(t *testing.T, raw []byte) map[string]jsonValue {
	t.Helper()
	value, err := decodeStrictJSON(raw)
	if err != nil {
		t.Fatalf("decode artifact: %v", err)
	}
	return objectValue(t, value)
}

func objectValue(t *testing.T, value jsonValue) map[string]jsonValue {
	t.Helper()
	object, ok := value.(map[string]jsonValue)
	if !ok {
		t.Fatalf("value is not an object: %T", value)
	}
	return object
}

func arrayValue(t *testing.T, value jsonValue) []jsonValue {
	t.Helper()
	array, ok := value.([]jsonValue)
	if !ok {
		t.Fatalf("value is not an array: %T", value)
	}
	return array
}

func canonicalValue(t *testing.T, value jsonValue) []byte {
	t.Helper()
	encoded, err := canonicalJSONValue(value)
	if err != nil {
		t.Fatalf("canonicalize semantic projection: %v", err)
	}
	return encoded
}

func assertSuccessorFixture(t *testing.T, expected map[string]struct{}, relative string, content []byte) {
	t.Helper()
	if _, duplicate := expected[relative]; duplicate {
		t.Fatalf("duplicate successor fixture path %s", relative)
	}
	expected[relative] = struct{}{}
	path := repoPath("fixtures/vir-go/" + relative)
	if os.Getenv(updateSuccessorGoCorpusEnv) != "" {
		if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
			t.Fatalf("create successor fixture directory: %v", err)
		}
		if err := os.WriteFile(path, content, 0o644); err != nil {
			t.Fatalf("write successor fixture: %v", err)
		}
		return
	}
	want, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read successor fixture %s: %v", relative, err)
	}
	if !bytes.Equal(content, want) {
		t.Fatalf("successor fixture %s is stale", relative)
	}
}

func assertSuccessorFixtureSet(t *testing.T, expected map[string]struct{}) {
	t.Helper()
	root := repoPath("fixtures/vir-go")
	actual := make(map[string]struct{}, len(expected))
	actual["frontend-index.json"] = struct{}{}
	for _, subtree := range []string{"frontend", "negative"} {
		err := filepath.WalkDir(filepath.Join(root, subtree), func(path string, entry os.DirEntry, walkErr error) error {
			if walkErr != nil {
				return walkErr
			}
			if entry.IsDir() {
				return nil
			}
			if entry.Type()&os.ModeSymlink != 0 {
				t.Fatalf("successor fixture tree contains symlink %s", path)
			}
			info, err := entry.Info()
			if err != nil {
				return err
			}
			if !info.Mode().IsRegular() {
				t.Fatalf("successor fixture tree contains non-regular file %s", path)
			}
			relative, err := filepath.Rel(root, path)
			if err != nil {
				return err
			}
			relative = filepath.ToSlash(relative)
			if _, duplicate := actual[relative]; duplicate {
				t.Fatalf("duplicate successor fixture tree path %s", relative)
			}
			actual[relative] = struct{}{}
			return nil
		})
		if err != nil {
			t.Fatalf("walk successor fixture tree: %v", err)
		}
	}
	if !reflect.DeepEqual(actual, expected) {
		t.Fatalf("successor fixture tree is not exact: got %d files, want %d", len(actual), len(expected))
	}
}
