package main

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"
)

const updateGoVIRCorpusEnv = "MPK_UPDATE_GO_VIR_CORPUS"

type activeFrontendCase struct {
	ID                string
	SourceRoot        string
	SourcePath        string
	ImportPath        string
	SelectedFunction  string
	Contracts         []string
	ExpectedFunctions int
	ExamplePath       string
}

type activeNegativeCase struct {
	ID           string
	SourceRoot   string
	SourcePath   string
	ImportPath   string
	Function     string
	Contracts    []string
	ExpectedCode string
}

type activeFrontendIndex struct {
	Schema              string                      `json:"schema"`
	UpdateCommand       string                      `json:"update_command"`
	DeterministicRuns   int                         `json:"deterministic_runs"`
	AlphaFunctionCount  int                         `json:"alpha_function_count"`
	PositiveSourceCount int                         `json:"positive_source_count"`
	Cases               []activeFrontendIndexEntry  `json:"cases"`
	NegativeCases       []activeNegativeIndexEntry  `json:"negative_cases"`
	SemanticVector      activeSemanticVectorSummary `json:"semantic_vector"`
}

type activeFrontendIndexEntry struct {
	ID             string                `json:"id"`
	SourceRoot     string                `json:"source_root"`
	SourcePath     string                `json:"source_path"`
	Selection      goSelection           `json:"selection"`
	FunctionCount  int                   `json:"function_count"`
	FrontendStatus string                `json:"frontend_status"`
	Artifacts      []activeArtifactIndex `json:"artifacts"`
	ExamplePath    string                `json:"example_path,omitempty"`
}

type activeNegativeIndexEntry struct {
	ID           string `json:"id"`
	SourceRoot   string `json:"source_root"`
	SourcePath   string `json:"source_path"`
	ExpectedCode string `json:"expected_code"`
	ActualCode   string `json:"actual_code"`
	Outcome      string `json:"outcome"`
}

type activeArtifactIndex struct {
	Kind   string `json:"kind"`
	Path   string `json:"path"`
	SHA256 string `json:"sha256"`
	Bytes  int    `json:"bytes"`
}

type activeSemanticVectorSummary struct {
	Path            string `json:"path"`
	AcceptedCases   int    `json:"accepted_cases"`
	RejectedCases   int    `json:"rejected_cases"`
	RuntimeChecks   int    `json:"runtime_checks"`
	Loops           int    `json:"loops"`
	Conversions     int    `json:"conversions"`
	Calls           int    `json:"calls"`
	Contracts       int    `json:"contracts"`
	UnresolvedCases int    `json:"unresolved_cases"`
}

type activeFrontendArtifacts struct {
	Envelope []byte
	VIR      []byte
	Map      []byte
	Manifest []byte
	Function int
}

func TestRegenerateGoVIRFrontendCorpus(t *testing.T) {
	referenceManifest := frontendManifestReference(t)
	cases := frontendCorpusCases()
	first := make(map[string]activeFrontendArtifacts, len(cases))
	second := make(map[string]activeFrontendArtifacts, len(cases))
	for _, corpusCase := range cases {
		first[corpusCase.ID] = generateFrontendCorpusCase(t, corpusCase, referenceManifest)
		second[corpusCase.ID] = generateFrontendCorpusCase(t, corpusCase, referenceManifest)
		assertActiveFrontendEqual(t, corpusCase.ID, first[corpusCase.ID], second[corpusCase.ID])
	}

	index := activeFrontendIndex{
		Schema:              "mpk.go_vir_frontend_corpus.v0",
		UpdateCommand:       "MPK_UPDATE_GO_VIR_CORPUS=1 go test -count=1 -run TestRegenerateGoVIRFrontendCorpus",
		DeterministicRuns:   2,
		AlphaFunctionCount:  100,
		PositiveSourceCount: len(cases),
		SemanticVector:      semanticVectorSummary(t),
	}
	for _, corpusCase := range cases {
		artifacts := first[corpusCase.ID]
		base := "frontend/" + corpusCase.ID
		files := []struct {
			kind, name string
			bytes      []byte
		}{
			{"frontend_envelope", "frontend-envelope.json", artifacts.Envelope},
			{"vir", "vir.json", artifacts.VIR},
			{"source_map", "source-map.json", artifacts.Map},
			{"source_manifest_frontend", "source-manifest.frontend.json", artifacts.Manifest},
		}
		entry := activeFrontendIndexEntry{
			ID: corpusCase.ID, SourceRoot: corpusCase.SourceRoot, SourcePath: corpusCase.SourcePath,
			Selection:     goSelection{Package: corpusCase.ImportPath, Function: corpusCase.SelectedFunction},
			FunctionCount: artifacts.Function, FrontendStatus: "ir-lowered",
		}
		if corpusCase.ExamplePath != "" {
			entry.ExamplePath = "examples/" + corpusCase.ExamplePath
		}
		for _, file := range files {
			path := base + "/" + file.name
			assertCorpusFixture(t, path, file.bytes)
			entry.Artifacts = append(entry.Artifacts, activeArtifactIndex{
				Kind: file.kind, Path: path, SHA256: sha256Hex(file.bytes), Bytes: len(file.bytes),
			})
			if corpusCase.ExamplePath != "" {
				assertFixtureAt(t, repoPath("examples/"+corpusCase.ExamplePath+"/"+file.name), file.bytes)
			}
		}
		index.Cases = append(index.Cases, entry)
	}
	index.NegativeCases = auditNegativeCorpus(t)
	indexBytes, err := canonicalJSON(index)
	if err != nil {
		t.Fatalf("canonical frontend corpus index: %v", err)
	}
	assertCorpusFixture(t, "frontend-index.json", indexBytes)
	assertNoCorpusLeakage(t, indexBytes)
}

func frontendCorpusCases() []activeFrontendCase {
	return []activeFrontendCase{
		{ID: "alpha-arith", SourceRoot: "fixtures/go-alpha", SourcePath: "arith/arith.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-alpha/arith", SelectedFunction: "github.com/finitefield-org/mpk/fixtures/go-alpha/arith.Add64", ExpectedFunctions: 34},
		{ID: "alpha-array", SourceRoot: "fixtures/go-alpha", SourcePath: "array/array.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-alpha/array", SelectedFunction: "github.com/finitefield-org/mpk/fixtures/go-alpha/array.BuildPair64", ExpectedFunctions: 33},
		{ID: "alpha-branch", SourceRoot: "fixtures/go-alpha", SourcePath: "branch/branch.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-alpha/branch", SelectedFunction: "github.com/finitefield-org/mpk/fixtures/go-alpha/branch.Max64", ExpectedFunctions: 33},
		{ID: "basic-arith", SourceRoot: "fixtures/go-basic", SourcePath: "positive/arith/arith.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/positive/arith", SelectedFunction: "github.com/finitefield-org/mpk/fixtures/go-basic/positive/arith.Add64", ExpectedFunctions: 3},
		{ID: "basic-branch", SourceRoot: "fixtures/go-basic", SourcePath: "positive/branch/branch.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/positive/branch", SelectedFunction: "github.com/finitefield-org/mpk/fixtures/go-basic/positive/branch.SelectGE", ExpectedFunctions: 1},
		{ID: "basic-structarray", SourceRoot: "fixtures/go-basic", SourcePath: "positive/structarray/struct_array.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/positive/structarray", SelectedFunction: "github.com/finitefield-org/mpk/fixtures/go-basic/positive/structarray.SumPair", ExpectedFunctions: 4},
		{ID: "payment-discount", SourceRoot: "examples/payment_policies/discount", SourcePath: "policy.go", ImportPath: "example.com/payment/discount", SelectedFunction: "example.com/payment/discount.ApprovedDiscountCents", Contracts: []string{"policy_contract.json"}, ExpectedFunctions: 1, ExamplePath: "payment_policies/discount"},
		{ID: "payment-fee", SourceRoot: "examples/payment_policies/fee", SourcePath: "policy.go", ImportPath: "example.com/payment/fee", SelectedFunction: "example.com/payment/fee.AppliedPlatformFeeCents", Contracts: []string{"policy_contract.json"}, ExpectedFunctions: 1, ExamplePath: "payment_policies/fee"},
		{ID: "payment-points", SourceRoot: "examples/payment_policies/points", SourcePath: "policy.go", ImportPath: "example.com/payment/points", SelectedFunction: "example.com/payment/points.ApprovedRedemptionPoints", Contracts: []string{"policy_contract.json"}, ExpectedFunctions: 1, ExamplePath: "payment_policies/points"},
		{ID: "payment-refund", SourceRoot: "examples/payment_policies/refund", SourcePath: "policy.go", ImportPath: "example.com/payment/refund", SelectedFunction: "example.com/payment/refund.ApprovedRefundCents", Contracts: []string{"policy_contract.json"}, ExpectedFunctions: 1, ExamplePath: "payment_policies/refund"},
		{ID: "payment-reserve", SourceRoot: "examples/payment_policies/reserve", SourcePath: "policy.go", ImportPath: "example.com/payment/reserve", SelectedFunction: "example.com/payment/reserve.ApprovedReserveCents", Contracts: []string{"policy_contract.json"}, ExpectedFunctions: 1, ExamplePath: "payment_policies/reserve"},
		{ID: "example-max64", SourceRoot: "examples/max64", SourcePath: "max64.go", ImportPath: "example", SelectedFunction: "example.Max64", Contracts: []string{"max64_contract.json"}, ExpectedFunctions: 1, ExamplePath: "max64"},
		{ID: "example-order-policy", SourceRoot: "examples/order_policy", SourcePath: "policy.go", ImportPath: "example.com/orderpolicy", SelectedFunction: "example.com/orderpolicy.ApprovedReserveCents", Contracts: []string{"policy_contract.json"}, ExpectedFunctions: 1, ExamplePath: "order_policy"},
	}
}

func negativeCorpusCases() []activeNegativeCase {
	return []activeNegativeCase{
		{ID: "basic-generic", SourceRoot: "fixtures/go-basic", SourcePath: "negative/generic/generic.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/generic", Function: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/generic.Identity", ExpectedCode: "GO_SUBSET_GENERICS"},
		{ID: "basic-map", SourceRoot: "fixtures/go-basic", SourcePath: "negative/map/map.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/map", Function: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/map.Lookup", ExpectedCode: "GO_SUBSET_MAPS"},
		{ID: "basic-pointer", SourceRoot: "fixtures/go-basic", SourcePath: "negative/pointer/pointer.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/pointer", Function: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/pointer.Deref", ExpectedCode: "GO_SUBSET_POINTER"},
		{ID: "basic-string", SourceRoot: "fixtures/go-basic", SourcePath: "negative/string/string.go", ImportPath: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/string", Function: "github.com/finitefield-org/mpk/fixtures/go-basic/negative/string.Equal", ExpectedCode: "GO_SUBSET_STRING"},
		{ID: "payment-float", SourceRoot: "examples/payment_policies/negative/float", SourcePath: "policy.go", ImportPath: "example.com/payment/negative/float", Function: "example.com/payment/negative/float.ApprovedRate", ExpectedCode: "GO_SUBSET_FLOAT"},
		{ID: "payment-map", SourceRoot: "examples/payment_policies/negative/map", SourcePath: "policy.go", ImportPath: "example.com/payment/negative/map", Function: "example.com/payment/negative/map.LookupReserve", ExpectedCode: "GO_SUBSET_MAPS"},
		{ID: "payment-pointer", SourceRoot: "examples/payment_policies/negative/pointer", SourcePath: "policy.go", ImportPath: "example.com/payment/negative/pointer", Function: "example.com/payment/negative/pointer.DereferenceReserve", ExpectedCode: "GO_SUBSET_POINTER"},
		{ID: "payment-missing-postconditions", SourceRoot: "examples/payment_policies/negative/missing_postconditions", SourcePath: "policy.go", ImportPath: "example.com/payment/negative/missingpost", Function: "example.com/payment/negative/missingpost.IdentityCents", Contracts: []string{"policy_contract.json"}, ExpectedCode: "GO_CONTRACT_ENSURES"},
	}
}

func frontendManifestReference(t *testing.T) sourceManifest {
	t.Helper()
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/source-manifest-v0.json"))
	caseObject := findCase(t, arrayField(t, vectors, "manifest_cases"), "manifest.valid_go_frontend_stage")
	var manifest sourceManifest
	unmarshalJSONValue(t, objectField(t, caseObject, "input"), &manifest)
	return manifest
}

func generateFrontendCorpusCase(t *testing.T, corpusCase activeFrontendCase, reference sourceManifest) activeFrontendArtifacts {
	t.Helper()
	source := mustReadFile(t, repoPath(corpusCase.SourceRoot+"/"+corpusCase.SourcePath))
	loaded, capture := typedSinglePackage(t, corpusCase.ImportPath, corpusCase.SourcePath, string(source))
	goMod := mustReadFile(t, repoPath(corpusCase.SourceRoot+"/go.mod"))
	capture.Inputs = append(capture.Inputs,
		capturedInput{Kind: buildManifestInputKind, NormalizedPath: "go.mod", Bytes: goMod, SHA256: sha256Hex(goMod)},
		capturedInput{Kind: lockfileInputKind, NormalizedPath: "go.sum", Bytes: []byte{}, SHA256: sha256Hex(nil)},
	)
	for _, path := range corpusCase.Contracts {
		content := mustReadFile(t, repoPath(corpusCase.SourceRoot+"/"+path))
		capture.Inputs = append(capture.Inputs, capturedInput{Kind: contractInputKind, NormalizedPath: path, Bytes: content, SHA256: sha256Hex(content)})
	}
	request := lowerRequest{
		SourceRoot: logicalSourceRoot, Package: corpusCase.ImportPath, Function: corpusCase.SelectedFunction,
		SemanticProfile: goSemanticProfile, Target: goTarget,
		FrontendBundleID: reference.Frontend.BundleID, FrontendSHA256: reference.Frontend.BinarySHA256,
		ReleaseRegistryID: reference.ReleaseRegistry.ID, ReleaseRegistrySHA256: reference.ReleaseRegistry.RegistrySHA256,
		ToolchainBundleID: reference.Toolchain.BundleID, ToolchainRoot: logicalToolchain,
		ToolchainDistributionSHA256: reference.Toolchain.DistributionSHA256, Contracts: copyStrings(corpusCase.Contracts),
	}
	selection := validatedLauncherSelection{
		Registry: reference.ReleaseRegistry, Frontend: reference.Frontend, Toolchain: reference.Toolchain,
		Target: reference.Target, LimitProfileID: reference.LimitProfile,
	}
	artifacts, findings, err := lowerPrivatePipeline(request, capture, loaded, selection)
	if err != nil {
		t.Fatalf("%s frontend generation: %v", corpusCase.ID, err)
	}
	if len(findings) != 0 {
		t.Fatalf("%s unexpectedly rejected: %+v", corpusCase.ID, findings[0])
	}
	functionCount := 0
	for _, unit := range artifacts.Module.Units {
		functionCount += len(unit.Functions)
	}
	if functionCount != corpusCase.ExpectedFunctions {
		t.Fatalf("%s function count = %d, want %d", corpusCase.ID, functionCount, corpusCase.ExpectedFunctions)
	}
	for _, content := range [][]byte{artifacts.EnvelopeJSON, artifacts.ModuleJSON, artifacts.MapJSON, artifacts.ManifestJSON} {
		assertNoCorpusLeakage(t, content)
	}
	return activeFrontendArtifacts{
		Envelope: copyBytes(artifacts.EnvelopeJSON), VIR: copyBytes(artifacts.ModuleJSON),
		Map: copyBytes(artifacts.MapJSON), Manifest: copyBytes(artifacts.ManifestJSON), Function: functionCount,
	}
}

func auditNegativeCorpus(t *testing.T) []activeNegativeIndexEntry {
	t.Helper()
	entries := make([]activeNegativeIndexEntry, 0, len(negativeCorpusCases()))
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
		actual := findings[0].Code
		if actual != corpusCase.ExpectedCode {
			t.Fatalf("negative case %s code = %s, want %s", corpusCase.ID, actual, corpusCase.ExpectedCode)
		}
		entries = append(entries, activeNegativeIndexEntry{
			ID: corpusCase.ID, SourceRoot: corpusCase.SourceRoot, SourcePath: corpusCase.SourcePath,
			ExpectedCode: corpusCase.ExpectedCode, ActualCode: actual, Outcome: "rejected",
		})
	}
	bytes, err := canonicalJSON(struct {
		Schema string                     `json:"schema"`
		Cases  []activeNegativeIndexEntry `json:"cases"`
	}{Schema: "mpk.go_vir_negative_audit.v0", Cases: entries})
	if err != nil {
		t.Fatal(err)
	}
	assertCorpusFixture(t, "negative-results.json", bytes)
	return entries
}

func semanticVectorSummary(t *testing.T) activeSemanticVectorSummary {
	t.Helper()
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/go-vir-profile-v0.json"))
	accepted, rejected, runtimeChecks, loops, conversions, calls, contracts := 0, 0, 0, 0, 0, 0, 0
	for _, field := range []string{"source_cases", "operation_cases", "contract_cases", "loop_call_cases"} {
		for _, raw := range arrayField(t, vectors, field) {
			object := asObject(t, raw, field)
			expect := objectField(t, object, "expect")
			if stringField(t, expect, "outcome") == "accepted" {
				accepted++
			} else {
				rejected++
			}
			id := stringField(t, object, "id")
			if containsFragment(id, "div") || containsFragment(id, "shift") || containsFragment(id, "index") {
				runtimeChecks++
			}
			if containsFragment(id, "loop") {
				loops++
			}
			if containsFragment(id, "convert") {
				conversions++
			}
			if containsFragment(id, "call") {
				calls++
			}
			if field == "contract_cases" {
				contracts++
			}
		}
	}
	return activeSemanticVectorSummary{
		Path: "develop/specs/vectors/go-vir-profile-v0.json", AcceptedCases: accepted, RejectedCases: rejected,
		RuntimeChecks: runtimeChecks, Loops: loops, Conversions: conversions, Calls: calls, Contracts: contracts,
		UnresolvedCases: 0,
	}
}

func assertActiveFrontendEqual(t *testing.T, id string, left, right activeFrontendArtifacts) {
	t.Helper()
	if left.Function != right.Function || !bytes.Equal(left.Envelope, right.Envelope) || !bytes.Equal(left.VIR, right.VIR) || !bytes.Equal(left.Map, right.Map) || !bytes.Equal(left.Manifest, right.Manifest) {
		t.Fatalf("%s changed between two clean frontend generations", id)
	}
}

func assertCorpusFixture(t *testing.T, relative string, content []byte) {
	t.Helper()
	assertFixtureAt(t, repoPath("fixtures/vir-go/"+relative), content)
}

func assertFixtureAt(t *testing.T, path string, content []byte) {
	t.Helper()
	if os.Getenv(updateGoVIRCorpusEnv) != "" {
		if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
			t.Fatalf("create fixture directory %s: %v", filepath.Dir(path), err)
		}
		if err := os.WriteFile(path, content, 0o644); err != nil {
			t.Fatalf("write fixture %s: %v", path, err)
		}
		return
	}
	want, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read fixture %s: %v", path, err)
	}
	if !bytes.Equal(content, want) {
		t.Fatalf("fixture %s is stale; rerun the explicit update command", path)
	}
}

func assertNoCorpusLeakage(t *testing.T, content []byte) {
	t.Helper()
	forbiddenValues := []string{
		repoPath(""), os.TempDir(),
		`"timestamp"`, `"generated_at"`, `"generatedAt"`, `"hostname"`,
	}
	if hostname, err := os.Hostname(); err == nil {
		forbiddenValues = append(forbiddenValues, hostname)
	}
	for _, forbidden := range forbiddenValues {
		if forbidden != "" && bytes.Contains(content, []byte(forbidden)) {
			t.Fatalf("generated corpus leaks forbidden value %q", forbidden)
		}
	}
}

func containsFragment(value, fragment string) bool {
	return len(value) >= len(fragment) && bytes.Contains([]byte(value), []byte(fragment))
}
