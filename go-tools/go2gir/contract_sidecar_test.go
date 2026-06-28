package main

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

func TestRunAttachesContractSidecarToGIR(t *testing.T) {
	result := runSuccessfulPackage(t, "./testdata/contractsample")
	if result.GIR == nil {
		t.Fatal("GIR missing")
	}

	function := findGIRFunction(*result.GIR, "github.com/finitefield-org/mpk/go-tools/go2gir/testdata/contractsample", "Max64")
	if function == nil {
		t.Fatalf("GIR missing Max64: %+v", result.GIR)
	}
	if len(function.Contracts.Requires) != 0 {
		t.Fatalf("requires = %+v, want empty", function.Contracts.Requires)
	}
	if len(function.Contracts.Modifies) != 0 {
		t.Fatalf("modifies = %+v, want empty", function.Contracts.Modifies)
	}
	if len(function.Contracts.Loops) != 0 {
		t.Fatalf("loops = %+v, want empty", function.Contracts.Loops)
	}
	if len(function.Contracts.Ensures) != 3 {
		t.Fatalf("ensures = %+v, want 3", function.Contracts.Ensures)
	}

	first := function.Contracts.Ensures[0]
	if first.Op != "signed_ge" || first.LHS == nil || first.RHS == nil {
		t.Fatalf("first ensure = %+v, want signed_ge lhs/rhs", first)
	}
	if first.LHS.Result == nil || *first.LHS.Result != 0 {
		t.Fatalf("first lhs = %+v, want result 0", first.LHS)
	}
	if first.RHS.Var != "a" {
		t.Fatalf("first rhs = %+v, want var a", first.RHS)
	}

	third := function.Contracts.Ensures[2]
	if third.Op != "or" || len(third.Args) != 2 {
		t.Fatalf("third ensure = %+v, want two-arg or", third)
	}
	if third.Args[0].Op != "eq" || third.Args[1].Op != "eq" {
		t.Fatalf("or args = %+v, want eq args", third.Args)
	}
}

func TestRunRejectsInvalidContractSidecars(t *testing.T) {
	tests := []struct {
		name   string
		path   string
		reason string
	}{
		{
			name:   "malformed json",
			path:   "./testdata/contract_malformed",
			reason: "invalid contract sidecar JSON",
		},
		{
			name:   "unknown function",
			path:   "./testdata/contract_unknown_function",
			reason: `contract function "unknowncontract.Missing" does not resolve to a lowered GIR function`,
		},
		{
			name:   "unsupported operator",
			path:   "./testdata/contract_unsupported_operator",
			reason: `unsupported contract expression operator "float_eq"`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var stdout bytes.Buffer
			var stderr bytes.Buffer

			exitCode := run([]string{tt.path}, &stdout, &stderr)
			if exitCode != 1 {
				t.Fatalf("exit code = %d, want 1; stdout=%s stderr=%s", exitCode, stdout.String(), stderr.String())
			}
			if stderr.Len() != 0 {
				t.Fatalf("stderr = %q, want empty", stderr.String())
			}

			var result cliResult
			if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
				t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
			}
			if result.Status != "rejected" {
				t.Fatalf("status = %q, want rejected", result.Status)
			}
			if result.GIR != nil || result.GIREmission != nil || result.SourceManifest != nil {
				t.Fatalf("GIR output = %+v / %+v / %+v, want nil for rejected package", result.GIR, result.GIREmission, result.SourceManifest)
			}
			if !hasRejectedFeatureReasonContaining(result.RejectedFeatures, "contract sidecar", tt.reason) {
				t.Fatalf("rejected features = %+v, want contract sidecar reason containing %q", result.RejectedFeatures, tt.reason)
			}
		})
	}
}

func TestValidateContractSidecarAttachesRequiresAndLoops(t *testing.T) {
	sidecar := contractSidecar{
		Path:     "contract.json",
		Function: "example.Count",
		Requires: []json.RawMessage{
			json.RawMessage(`{"bool": true}`),
		},
		Ensures: []json.RawMessage{
			json.RawMessage(`{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}`),
		},
		Modifies: []string{},
		Loops: []contractLoopSidecar{
			{
				BlockID: "entry",
				Invariants: []json.RawMessage{
					json.RawMessage(`{"bool": true}`),
				},
				Decreases: []json.RawMessage{
					json.RawMessage(`{"int":{"value":"0","width":64,"signed":true}}`),
				},
			},
		},
	}
	function := girFunction{
		Params: []girBinding{
			{Name: "value", Type: girType{Kind: "bv", Width: 64, Signed: boolPtr(true)}},
		},
		Results: []girBinding{
			{Name: "result0", Type: girType{Kind: "bv", Width: 64, Signed: boolPtr(true)}},
		},
		Blocks: []girBlock{{Label: "entry"}},
	}

	contracts, findings := validateContractSidecar("", sidecar, function)
	if len(findings) != 0 {
		t.Fatalf("findings = %+v, want none", findings)
	}
	if len(contracts.Requires) != 1 || contracts.Requires[0].Bool == nil || !*contracts.Requires[0].Bool {
		t.Fatalf("requires = %+v, want true atom", contracts.Requires)
	}
	if len(contracts.Ensures) != 1 || contracts.Ensures[0].Op != "eq" {
		t.Fatalf("ensures = %+v, want eq", contracts.Ensures)
	}
	if len(contracts.Modifies) != 0 {
		t.Fatalf("modifies = %+v, want empty", contracts.Modifies)
	}
	if len(contracts.Loops) != 1 {
		t.Fatalf("loops = %+v, want one loop contract", contracts.Loops)
	}
	if contracts.Loops[0].BlockID != "entry" || len(contracts.Loops[0].Invariants) != 1 || len(contracts.Loops[0].Decreases) != 1 {
		t.Fatalf("loop = %+v, want entry with invariant and decreases", contracts.Loops[0])
	}
}

func TestValidateContractSidecarRejectsNonEmptyModifies(t *testing.T) {
	sidecar := contractSidecar{
		Path:     "contract.json",
		Function: "example.Identity",
		Modifies: []string{"global"},
	}

	_, findings := validateContractSidecar("", sidecar, girFunction{})
	if !hasRejectedFeatureReasonContaining(findings, "contract sidecar", "non-empty modifies are rejected by Go subset v0") {
		t.Fatalf("findings = %+v, want non-empty modifies rejection", findings)
	}
}

func TestContractExprRejectsDroppedUnaryTypeField(t *testing.T) {
	raw := json.RawMessage(`{
		"op": "not",
		"value": {"bool": true},
		"type": {"kind": "bool"}
	}`)

	_, err := parseContractExpr(raw, contractValidationContext{})
	if err == nil {
		t.Fatal("parseContractExpr accepted unary type field, want rejection")
	}
	if !strings.Contains(err.Error(), `contract operator "not" does not accept type`) {
		t.Fatalf("error = %v", err)
	}
}

func TestContractExprRequiresSignedIntegerLiteralTag(t *testing.T) {
	raw := json.RawMessage(`{"int":{"value":"0","width":64}}`)

	_, err := parseContractExpr(raw, contractValidationContext{})
	if err == nil {
		t.Fatal("parseContractExpr accepted missing signed tag, want rejection")
	}
	if !strings.Contains(err.Error(), "contract integer literals require signed") {
		t.Fatalf("error = %v", err)
	}
}

func TestContractExprRejectsExtraConvertTargetTypeFields(t *testing.T) {
	raw := json.RawMessage(`{
		"op": "convert",
		"value": {"bool": true},
		"type": {"kind": "bool", "width": 64}
	}`)

	_, err := parseContractExpr(raw, contractValidationContext{})
	if err == nil {
		t.Fatal("parseContractExpr accepted extra bool type fields, want rejection")
	}
	if !strings.Contains(err.Error(), "contract bool type contains unsupported fields") {
		t.Fatalf("error = %v", err)
	}
}

func TestContractExprAcceptsUnaryArgsForm(t *testing.T) {
	raw := json.RawMessage(`{"op":"not","args":[{"bool": false}]}`)

	expr, err := parseContractExpr(raw, contractValidationContext{})
	if err != nil {
		t.Fatalf("parseContractExpr: %v", err)
	}
	if expr.Op != "not" || expr.Value == nil || expr.Value.Bool == nil || *expr.Value.Bool {
		t.Fatalf("expr = %+v, want not false", expr)
	}
}

func hasRejectedFeatureReasonContaining(features []rejectedFeature, feature string, reason string) bool {
	for _, got := range features {
		if got.Feature == feature && got.Location != "" && strings.Contains(got.Reason, reason) {
			return true
		}
	}
	return false
}
