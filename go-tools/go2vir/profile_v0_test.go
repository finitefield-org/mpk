package main

import "testing"

func TestGoProfileV0StableDiagnosticTruncation(t *testing.T) {
	request := lowerRequest{Package: "example.com/p", Function: "example.com/p.F", SemanticProfile: goSemanticProfile, Target: goTarget}
	findings := make([]loweringFinding, maximumIssues+1)
	for index := range findings {
		findings[index] = loweringFinding{Code: "GO_SUBSET_SYNTAX", Message: "unsupported source syntax", FunctionID: request.Function}
	}
	envelope, err := loweringFindingsEnvelope(request, findings)
	if err != nil {
		t.Fatalf("normalize truncated diagnostics: %v", err)
	}
	if len(envelope.RejectedFeatures) != maximumIssues-1 {
		t.Fatalf("retained issues = %d, want %d", len(envelope.RejectedFeatures), maximumIssues-1)
	}
	markers := 0
	for _, value := range envelope.Diagnostics {
		if value.Code == "GO_LIMIT_DIAGNOSTICS_TRUNCATED" {
			if value.FunctionID != nil || value.Span != nil {
				t.Fatal("truncation marker must remain function- and span-free")
			}
			markers++
		}
	}
	if markers != 1 {
		t.Fatalf("truncation markers = %d, want 1", markers)
	}
}

func TestGoProfileV0IdentityIsClosed(t *testing.T) {
	if goSemanticProfile != "mpk.go.fixed.v0" || goTarget != "linux/amd64" || goPointerWidth != 64 || virSchema != "mpk.vir.v0" || sourceMapSchema != "mpk.source_map.v0" {
		t.Fatal("Go/VIR profile identity drifted")
	}
}

func TestGeneratedVIRLimitsFailBeforeEmission(t *testing.T) {
	function := virFunction{
		ID: "example.com/p.F", UnitID: "example.com/p", Name: "F",
		Params: make([]virBinding, maximumVIRParams+1), Results: []virBinding{}, Locals: []virBinding{},
		Blocks:    []virBlock{{Label: "bb0", Parameters: []virBinding{}, Instructions: []virInstruction{}, Terminator: virTerminator{Kind: "Return", Values: []virValue{}}}},
		Contracts: defaultVIRContract("example.com/p", "example.com/p.F"), FeaturesUsed: []string{},
	}
	module := virModule{Units: []virUnit{{ID: "example.com/p", Name: "p", TypeDecls: []virTypeDecl{}, ConstDecls: []virConstDecl{}, Functions: []virFunction{function}}}}
	if finding := validateGeneratedVIRLimits(module); finding == nil || finding.Code != "VIR_LIMIT_PARAMS" {
		t.Fatalf("generated VIR limit finding = %+v, want VIR_LIMIT_PARAMS", finding)
	}
}

func TestSourceMapBuildFailureRetainsStableCode(t *testing.T) {
	function := virFunction{
		ID: "example.com/p.F", UnitID: "example.com/p", Name: "F", origin: sourceOrigin{},
		Params: []virBinding{}, Results: []virBinding{}, Locals: []virBinding{},
		Blocks:    []virBlock{{Label: "bb0", Parameters: []virBinding{}, Instructions: []virInstruction{}, Terminator: virTerminator{Kind: "Return", Values: []virValue{}, origin: sourceOrigin{Kind: "synthetic", Reason: "go.implicit_return"}}}},
		Contracts: defaultVIRContract("example.com/p", "example.com/p.F"), FeaturesUsed: []string{},
	}
	module := virModule{Units: []virUnit{{ID: "example.com/p", Name: "p", TypeDecls: []virTypeDecl{}, ConstDecls: []virConstDecl{}, Functions: []virFunction{function}}}}
	_, _, err := buildSourceMap(module, sourceCapture{})
	if err == nil {
		t.Fatal("missing source origin was accepted")
	}
	finding, ok := artifactFinding(err)
	if !ok || finding.Code != "GO_SOURCE_MAP_ORIGIN" {
		t.Fatalf("source-map finding = %+v, %v", finding, err)
	}
}
