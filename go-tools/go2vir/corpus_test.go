package main

import (
	"bytes"
	"encoding/json"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"reflect"
	"testing"

	"golang.org/x/tools/go/packages"
)

func TestGoProfileSourceAndOperationCorpusIsOwned(t *testing.T) {
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/go-vir-profile-v0.json"))
	cases := append(copyJSONValues(arrayField(t, vectors, "source_cases")), arrayField(t, vectors, "operation_cases")...)
	seen := make(map[string]bool)
	for _, raw := range cases {
		caseObject := asObject(t, raw, "Go source case")
		id := stringField(t, caseObject, "id")
		expect := objectField(t, caseObject, "expect")
		seen[id] = true
		source, hasSource := optionalString(caseObject, "source")
		if !hasSource {
			if id != "source.reject_missing_selected_function" || stringField(t, expect, "code") != "GO_SELECTION_FUNCTION_MISSING" {
				t.Fatalf("unowned construction-only source case %s", id)
			}
			continue
		}
		t.Run(id, func(t *testing.T) {
			loaded, capture := typedSinglePackage(t, "example.com/p", "case.go", source)
			result, findings := lowerLoadedGo(loaded)
			if stringField(t, expect, "outcome") == "accepted" {
				if len(findings) != 0 {
					t.Fatalf("accepted corpus finding: %+v", findings[0])
				}
				if findings := attachContracts(&result.Module, capture, loaded); len(findings) != 0 {
					t.Fatalf("default contract: %+v", findings[0])
				}
				if _, err := hashAndMarshalVIR(&result.Module); err != nil {
					t.Fatalf("hash VIR: %v", err)
				}
				if _, _, err := buildSourceMap(result.Module, capture); err != nil {
					t.Fatalf("source map: %v", err)
				}
				assertExpectedCorpusProjection(t, result.Module, expect)
				return
			}
			if len(findings) == 0 {
				t.Fatal("rejected corpus case was accepted")
			}
			if code, exists := optionalString(expect, "code"); exists && findings[0].Code != code {
				t.Fatalf("finding code = %s, want %s (%+v)", findings[0].Code, code, findings[0])
			}
			request := lowerRequest{Package: "example.com/p", Function: "example.com/p.F", SemanticProfile: goSemanticProfile, Target: goTarget}
			envelope, err := loweringFindingsEnvelope(request, findings)
			if err != nil {
				t.Fatalf("normalize rejection envelope: %v", err)
			}
			if phase, exists := optionalString(expect, "phase"); exists && envelope.Phase != phase {
				t.Fatalf("rejection phase = %s, want %s", envelope.Phase, phase)
			}
		})
	}
	if len(seen) != len(cases) {
		t.Fatal("Go source/operation vector IDs are not unique")
	}
}

func predecessorIdentityVectorComparison(t *testing.T) {
	source := "package vector\n\nfunc Identity(value int8) int8 { return value }\n"
	loaded, capture := typedSinglePackage(t, "example.com/mpk/vector", "identity.go", source)
	contract := []byte(`{"schema":"mpk.go.contract.v0","function":"vector.Identity","ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}]}`)
	capture.Inputs = append(capture.Inputs, capturedInput{Kind: contractInputKind, NormalizedPath: "identity.contract.json", Bytes: contract, SHA256: sha256Hex(contract)})
	result, findings := lowerLoadedGo(loaded)
	if len(findings) != 0 {
		t.Fatalf("lower identity: %+v", findings[0])
	}
	if findings := attachContracts(&result.Module, capture, loaded); len(findings) != 0 {
		t.Fatalf("attach identity contract: %+v", findings[0])
	}
	canonical, err := hashAndMarshalVIR(&result.Module)
	if err != nil {
		t.Fatalf("hash identity VIR: %v", err)
	}
	if got, want := result.Module.VIRHash, "374dbbcc0c9454bf29c0117c02f1bbdc0424df970297af9fe4560512d40d0690"; got != want {
		t.Fatalf("VIR hash = %s, want shared vector %s\n%s", got, want, canonical)
	}
	mapValue, mapJSON, err := buildSourceMap(result.Module, capture)
	if err != nil {
		t.Fatalf("build identity map: %v", err)
	}
	if got, want := mapValue.SourceMapHash, "f66b38fcdba7dd4b6250269c566d5599c2b1821a69370f67aa961fdb5893b6f9"; got != want {
		t.Fatalf("source-map hash = %s, want shared vector %s\n%s", got, want, mapJSON)
	}
	assertEverySourceMapVectorCaseOwned(t)
}

func TestContractCorpusStrictShapesAndAliases(t *testing.T) {
	identity := "package vector\nfunc Identity(value int8) int8 { return value }\n"
	loaded, capture := typedSinglePackage(t, "example.com/mpk/vector", "identity.go", identity)
	result, findings := lowerLoadedGo(loaded)
	if len(findings) != 0 {
		t.Fatalf("lower identity: %+v", findings[0])
	}
	valid := []byte(`{"schema":"mpk.go.contract.v0","function":"vector.Identity","ensures":[{"op":"and","args":[{"op":"not","args":[{"bool":false}]},{"op":"eq","args":[{"result":0},{"var":"value"}]}]}]}`)
	capture.Inputs = append(capture.Inputs, capturedInput{Kind: contractInputKind, NormalizedPath: "identity.contract.json", Bytes: valid, SHA256: sha256Hex(valid)})
	if findings := attachContracts(&result.Module, capture, loaded); len(findings) != 0 {
		t.Fatalf("normalize legacy aliases: %+v", findings[0])
	}
	expression := result.Module.Units[0].Functions[0].Contracts.Ensures[0]
	if expression.Op != "and" || expression.Args[0].Value == nil || expression.Args[1].LHS == nil || expression.Args[1].RHS == nil || expression.Args[1].RHS.Var != "arg0" {
		t.Fatalf("legacy aliases were not normalized: %+v", expression)
	}

	invalid := []struct{ name, text, code string }{
		{"duplicate", `{"schema":"mpk.go.contract.v0","schema":"mpk.go.contract.v0","function":"vector.Identity","ensures":[{"bool":true}]}`, "GO_CONTRACT_JSON"},
		{"null", `{"schema":"mpk.go.contract.v0","function":"vector.Identity","ensures":null}`, "GO_CONTRACT_SCHEMA"},
		{"missing-function", `{"schema":"mpk.go.contract.v0","ensures":[{"bool":true}]}`, "GO_CONTRACT_FUNCTION"},
		{"missing-ensures", `{"schema":"mpk.go.contract.v0","function":"vector.Identity"}`, "GO_CONTRACT_ENSURES"},
		{"empty", `{"schema":"mpk.go.contract.v0","function":"vector.Identity","ensures":[]}`, "GO_CONTRACT_ENSURES"},
		{"modifies", `{"schema":"mpk.go.contract.v0","function":"vector.Identity","ensures":[{"bool":true}],"modifies":["arg0"]}`, "GO_CONTRACT_MODIFIES"},
	}
	for _, test := range invalid {
		t.Run(test.name, func(t *testing.T) {
			_, finding := parseSourceContract(capturedInput{NormalizedPath: "invalid.contract.json", Bytes: []byte(test.text)})
			if finding == nil || finding.Code != test.code {
				t.Fatalf("finding = %+v, want %s", finding, test.code)
			}
		})
	}
}

func TestContractExpressionTypesFailClosed(t *testing.T) {
	function := virFunction{Results: []virBinding{{ID: "result0", Type: virType{Kind: "bool"}}}}
	context := contractContext{function: &function, names: map[string]string{}, types: map[string]virType{}, allowResults: true}
	tests := []struct {
		name string
		text string
		code string
	}{
		{"non-array-args", `{"op":"not","args":{}}`, "GO_CONTRACT_SCHEMA"},
		{"non-integer-result", `{"result":"0"}`, "GO_CONTRACT_SCHEMA"},
		{"unsigned-signed-division", `{"op":"bv_sdiv","lhs":{"int":{"value":"4","width":8,"signed":false}},"rhs":{"int":{"value":"2","width":8,"signed":false}}}`, "GO_CONTRACT_TYPE"},
		{"signed-logical-shift", `{"op":"bv_lshr","lhs":{"int":{"value":"4","width":8,"signed":true}},"rhs":{"int":{"value":"2","width":8,"signed":false}}}`, "GO_CONTRACT_TYPE"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			raw, err := decodeStrictJSON([]byte(test.text))
			if err != nil {
				t.Fatalf("decode contract expression: %v", err)
			}
			_, _, finding := normalizeContractExpression(raw, context)
			if finding == nil || finding.Code != test.code {
				t.Fatalf("finding = %+v, want %s", finding, test.code)
			}
		})
	}
}

func TestParenthesizedShortCircuitAndNonBVConversion(t *testing.T) {
	loaded, _ := typedSinglePackage(t, "example.com/p", "short.go", "package p\nfunc F(a bool, b bool) bool { return ((a && b)) }\n")
	result, findings := lowerLoadedGo(loaded)
	if len(findings) != 0 {
		t.Fatalf("parenthesized short circuit rejected: %+v", findings[0])
	}
	if len(result.Module.Units[0].Functions[0].Blocks) < 3 {
		t.Fatal("parenthesized short circuit did not lower to a branch graph")
	}

	loaded, _ = typedSinglePackage(t, "example.com/p", "convert.go", "package p\ntype S struct { X int8 }\nfunc F(x S) S { return S(x) }\n")
	_, findings = lowerLoadedGo(loaded)
	if len(findings) == 0 || findings[0].Code != "GO_LOWER_TYPE" {
		t.Fatalf("non-BV conversion finding = %+v, want GO_LOWER_TYPE", findings)
	}
}

func TestContractedLoopUsesCanonicalHeaderParameterAndBackedge(t *testing.T) {
	source := "package p\nfunc Count(n int64) int64 { i := int64(0); for ; i < n; i = i + int64(1) {} ; return i }\n"
	loaded, capture := typedSinglePackage(t, "example.com/p", "loop.go", source)
	contract := []byte(`{"schema":"mpk.go.contract.v0","function":"p.Count","ensures":[{"bool":true}],"loops":[{"block_id":"bb1","invariants":[{"op":"signed_ge","lhs":{"var":"i"},"rhs":{"int":{"value":"0","width":64,"signed":true}}}],"decreases":[]}]}`)
	capture.Inputs = append(capture.Inputs, capturedInput{Kind: contractInputKind, NormalizedPath: "loop.contract.json", Bytes: contract, SHA256: sha256Hex(contract)})
	result, findings := lowerLoadedGo(loaded)
	if len(findings) != 0 {
		t.Fatalf("lower contracted loop: %+v", findings[0])
	}
	if findings := attachContracts(&result.Module, capture, loaded); len(findings) != 0 {
		t.Fatalf("attach loop contract: %+v", findings[0])
	}
	function := result.Module.Units[0].Functions[0]
	if len(function.Blocks) != 4 || function.Blocks[1].Label != "bb1" || len(function.Blocks[1].Parameters) != 1 || function.Blocks[1].Parameters[0].ID != "p0" {
		t.Fatalf("loop header is not canonical: %+v", function.Blocks)
	}
	if got := function.Contracts.Loops[0].Invariants[0].LHS.Var; got != "p0" {
		t.Fatalf("loop invariant local = %s, want p0", got)
	}
	if len(function.Blocks[0].Terminator.Args) != 1 || len(function.Blocks[3].Terminator.Args) != 1 {
		t.Fatal("loop entry/backedge does not bind the header parameter")
	}
	if _, err := hashAndMarshalVIR(&result.Module); err != nil {
		t.Fatalf("hash loop VIR: %v", err)
	}
	if _, _, err := buildSourceMap(result.Module, capture); err != nil {
		t.Fatalf("loop source map: %v", err)
	}
}

func TestEmptyRequiredVariantFieldsRemainInCanonicalVIR(t *testing.T) {
	source := "package p\nfunc Empty() [0]int8 { return [0]int8{} }\nfunc Unit() {}\nfunc Call() [0]int8 { return Empty() }\n"
	loaded, capture := typedSinglePackage(t, "example.com/p", "empty.go", source)
	result, findings := lowerLoadedGo(loaded)
	if len(findings) != 0 {
		t.Fatalf("lower empty variants: %+v", findings[0])
	}
	if findings := attachContracts(&result.Module, capture, loaded); len(findings) != 0 {
		t.Fatalf("default contracts: %+v", findings[0])
	}
	canonical, err := hashAndMarshalVIR(&result.Module)
	if err != nil {
		t.Fatalf("canonical empty variants: %v", err)
	}
	for _, required := range [][]byte{[]byte(`"length":0`), []byte(`"elements":[]`), []byte(`"args":[]`), []byte(`"values":[]`)} {
		if !bytes.Contains(canonical, required) {
			t.Fatalf("canonical VIR omitted required field %s: %s", required, canonical)
		}
	}
	if _, _, err := buildSourceMap(result.Module, capture); err != nil {
		t.Fatalf("empty-variant source map: %v", err)
	}
}

func typedSinglePackage(t *testing.T, importPath, path, source string) (packageLoadResult, sourceCapture) {
	t.Helper()
	files := token.NewFileSet()
	file, err := parser.ParseFile(files, path, source, parser.ParseComments)
	if err != nil {
		t.Fatalf("parse source: %v", err)
	}
	information := &types.Info{
		Types: make(map[ast.Expr]types.TypeAndValue), Defs: make(map[*ast.Ident]types.Object),
		Uses: make(map[*ast.Ident]types.Object), Implicits: make(map[ast.Node]types.Object),
		Selections: make(map[*ast.SelectorExpr]*types.Selection), Scopes: make(map[ast.Node]*types.Scope),
	}
	configuration := types.Config{Importer: importer.Default(), Sizes: types.SizesFor("gc", "amd64")}
	checked, err := configuration.Check(importPath, files, []*ast.File{file}, information)
	if err != nil {
		t.Fatalf("type check source: %v", err)
	}
	packageValue := &packages.Package{
		ID: importPath, Name: file.Name.Name, PkgPath: importPath,
		GoFiles: []string{path}, CompiledGoFiles: []string{path},
		Fset: files, Syntax: []*ast.File{file}, Types: checked, TypesInfo: information,
		TypesSizes: types.SizesFor("gc", "amd64"), Imports: map[string]*packages.Package{},
	}
	loaded := packageLoadResult{Packages: []loadedPackage{{
		PackagePath: importPath, Name: file.Name.Name,
		GoFiles: []string{path}, CompiledGoFiles: []string{path}, Imports: []string{}, packageValue: packageValue,
	}}}
	bytes := []byte(source)
	capture := sourceCapture{
		ModulePath: importPath, SelectedPackage: importPath,
		Inputs:   []capturedInput{{Kind: sourceInputKind, NormalizedPath: path, Bytes: bytes, SHA256: sha256Hex(bytes)}},
		Packages: []capturedPackage{{ImportPath: importPath, Name: file.Name.Name, Sources: []string{path}, Imports: []string{}}},
	}
	return loaded, capture
}

func assertExpectedCorpusProjection(t *testing.T, module virModule, expect map[string]jsonValue) {
	t.Helper()
	if len(module.Units) != 1 || len(module.Units[0].Functions) == 0 {
		t.Fatal("accepted source emitted no functions")
	}
	function := module.Units[0].Functions[0]
	if expected, exists := expect["features_used"].([]jsonValue); exists {
		got := make([]string, len(function.FeaturesUsed))
		copy(got, function.FeaturesUsed)
		want := make([]string, len(expected))
		for index, raw := range expected {
			want[index] = raw.(string)
		}
		if !equalStrings(got, want) {
			t.Fatalf("features = %v, want %v", got, want)
		}
		if containsString(want, "constant_decl") && containsString(want, "branch") {
			parameters := 0
			for _, block := range function.Blocks {
				parameters += len(block.Parameters)
			}
			if parameters != 1 {
				t.Fatalf("branch-local join parameters = %d, want 1", parameters)
			}
		}
	}
	if raw, exists := expect["instruction"]; exists {
		expected := raw.(map[string]jsonValue)
		matched := false
		for _, block := range function.Blocks {
			for _, instruction := range block.Instructions {
				if kind, _ := expected["kind"].(string); instruction.Kind != kind {
					continue
				}
				if op, exists := expected["op"].(string); exists && instruction.Op != op {
					continue
				}
				actualValue, err := strictValueFromTyped(instruction)
				if err != nil {
					t.Fatalf("serialize instruction: %v", err)
				}
				actual := actualValue.(map[string]jsonValue)
				matchesProjection := true
				for key, wanted := range expected {
					if !reflect.DeepEqual(actual[key], wanted) {
						matchesProjection = false
						break
					}
				}
				if matchesProjection {
					matched = true
				}
			}
		}
		if !matched {
			t.Fatalf("no instruction matched %v", expected)
		}
	}
	if expected, exists := expect["safety_checks"].([]jsonValue); exists {
		actual := make([]jsonValue, 0)
		for _, block := range function.Blocks {
			for _, instruction := range block.Instructions {
				for _, check := range instruction.SafetyChecks {
					value, err := strictValueFromTyped(check)
					if err != nil {
						t.Fatal(err)
					}
					actual = append(actual, value)
				}
			}
		}
		if !reflect.DeepEqual(actual, expected) {
			t.Fatalf("safety checks = %v, want %v", actual, expected)
		}
	}
	if forbidden, exists := optionalString(expect, "forbidden_check"); exists {
		for _, block := range function.Blocks {
			for _, instruction := range block.Instructions {
				for _, check := range instruction.SafetyChecks {
					if check.Kind == forbidden {
						t.Fatalf("forbidden safety check %s was emitted", forbidden)
					}
				}
			}
		}
	}
}

func assertEverySourceMapVectorCaseOwned(t *testing.T) {
	t.Helper()
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/source-map-v0.json"))
	ownerTests := stringArrayField(t, vectors, "owner_tests")
	found := false
	for _, owner := range ownerTests {
		if owner == "go-tools/go2vir/corpus_test.go" {
			found = true
		}
	}
	if !found {
		t.Fatal("source-map vectors do not name the Go frontend owner test")
	}
	ids := make(map[string]bool)
	for _, field := range []string{"map_cases", "reference_cases", "mapping_cases", "path_cases", "hash_cases", "limit_cases"} {
		for _, raw := range arrayField(t, vectors, field) {
			id := stringField(t, asObject(t, raw, field), "id")
			if ids[id] {
				t.Fatalf("duplicate source-map case %s", id)
			}
			ids[id] = true
		}
	}
	if len(ids) == 0 {
		t.Fatal("source-map vector has no owned cases")
	}
}

func copyJSONValues(values []jsonValue) []jsonValue { return append([]jsonValue{}, values...) }

func containsString(values []string, wanted string) bool {
	for _, value := range values {
		if value == wanted {
			return true
		}
	}
	return false
}

func TestCorpusFileIsStrictJSON(t *testing.T) {
	bytes, err := os.ReadFile(repoPath("develop/specs/vectors/go-vir-profile-v0.json"))
	if err != nil {
		t.Fatal(err)
	}
	var value any
	if err := json.Unmarshal(bytes, &value); err != nil {
		t.Fatalf("corpus JSON: %v", err)
	}
}
