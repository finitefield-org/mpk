package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const contractSidecarSchema = "mpk.go.contract.v0"

type contractSidecar struct {
	Path     string
	Function string
	Requires []json.RawMessage
	Ensures  []json.RawMessage
	Modifies []string
	Loops    []contractLoopSidecar
}

type contractSidecarJSON struct {
	Schema   string                `json:"schema"`
	Function string                `json:"function"`
	Requires []json.RawMessage     `json:"requires,omitempty"`
	Ensures  []json.RawMessage     `json:"ensures,omitempty"`
	Modifies []string              `json:"modifies,omitempty"`
	Loops    []contractLoopSidecar `json:"loops,omitempty"`
}

type contractLoopSidecar struct {
	BlockID    string            `json:"block_id,omitempty"`
	Location   string            `json:"location,omitempty"`
	Invariants []json.RawMessage `json:"invariants,omitempty"`
	Decreases  []json.RawMessage `json:"decreases,omitempty"`
}

type contractExprJSON struct {
	Op     string            `json:"op,omitempty"`
	Args   []json.RawMessage `json:"args,omitempty"`
	LHS    *json.RawMessage  `json:"lhs,omitempty"`
	RHS    *json.RawMessage  `json:"rhs,omitempty"`
	Value  *json.RawMessage  `json:"value,omitempty"`
	Type   *girType          `json:"type,omitempty"`
	Var    *string           `json:"var,omitempty"`
	Result *int              `json:"result,omitempty"`
	Bool   *bool             `json:"bool,omitempty"`
	Int    *contractIntJSON  `json:"int,omitempty"`
}

type contractTarget struct {
	PackageIndex  int
	FunctionIndex int
}

type contractValidationContext struct {
	Variables  map[string]struct{}
	ResultLen  int
	BlockLabel map[string]struct{}
}

type contractIntJSON struct {
	Value  string `json:"value"`
	Width  int    `json:"width"`
	Signed *bool  `json:"signed"`
}

func attachContractSidecars(module *girModule, loaded packageLoadResult) []rejectedFeature {
	sidecars, findings := loadContractSidecars(loaded)
	if len(findings) > 0 || len(sidecars) == 0 {
		return findings
	}

	targets, targetFindings := contractTargets(module)
	findings = append(findings, targetFindings...)
	if len(targetFindings) > 0 {
		return findings
	}

	attached := make(map[contractTarget]string)
	for _, sidecar := range sidecars {
		targetsForIdentity := targets[sidecar.Function]
		if len(targetsForIdentity) == 0 {
			findings = append(findings, contractFinding(loaded.BaseDir, sidecar.Path, fmt.Sprintf("contract function %q does not resolve to a lowered GIR function", sidecar.Function)))
			continue
		}
		if len(targetsForIdentity) > 1 {
			findings = append(findings, contractFinding(loaded.BaseDir, sidecar.Path, fmt.Sprintf("contract function %q is ambiguous", sidecar.Function)))
			continue
		}

		target := targetsForIdentity[0]
		if previousPath, ok := attached[target]; ok {
			findings = append(findings, contractFinding(loaded.BaseDir, sidecar.Path, fmt.Sprintf("duplicate contract for %q; first seen in %s", sidecar.Function, normalizePath(loaded.BaseDir, previousPath))))
			continue
		}

		function := module.Packages[target.PackageIndex].Functions[target.FunctionIndex]
		contracts, validationFindings := validateContractSidecar(loaded.BaseDir, sidecar, function)
		if len(validationFindings) > 0 {
			findings = append(findings, validationFindings...)
			continue
		}
		module.Packages[target.PackageIndex].Functions[target.FunctionIndex].Contracts = contracts
		attached[target] = sidecar.Path
	}
	return findings
}

func loadContractSidecars(loaded packageLoadResult) ([]contractSidecar, []rejectedFeature) {
	paths, findings := discoverContractSidecarPaths(loaded)
	if len(findings) > 0 {
		return nil, findings
	}
	sidecars := make([]contractSidecar, 0, len(paths))
	for _, path := range paths {
		content, err := os.ReadFile(path)
		if err != nil {
			findings = append(findings, contractFinding(loaded.BaseDir, path, fmt.Sprintf("read contract sidecar: %v", err)))
			continue
		}
		sidecar, err := parseContractSidecar(path, content)
		if err != nil {
			findings = append(findings, contractFinding(loaded.BaseDir, path, err.Error()))
			continue
		}
		sidecars = append(sidecars, sidecar)
	}
	return sidecars, findings
}

func discoverContractSidecarPaths(loaded packageLoadResult) ([]string, []rejectedFeature) {
	dirs := packageSourceDirs(loaded)
	paths := make([]string, 0)
	var findings []rejectedFeature
	for _, dir := range dirs {
		entries, err := os.ReadDir(dir)
		if err != nil {
			findings = append(findings, contractFinding(loaded.BaseDir, dir, fmt.Sprintf("scan contract sidecars: %v", err)))
			continue
		}
		for _, entry := range entries {
			if entry.IsDir() || !isContractSidecarName(entry.Name()) {
				continue
			}
			paths = append(paths, filepath.Join(dir, entry.Name()))
		}
	}
	sort.Strings(paths)
	return paths, findings
}

func packageSourceDirs(loaded packageLoadResult) []string {
	seen := make(map[string]struct{})
	for _, pkg := range loaded.Packages {
		for _, sourcePath := range manifestPackageSourceFiles(pkg) {
			path := sourcePath
			if !filepath.IsAbs(path) {
				path = filepath.Join(loaded.BaseDir, path)
			}
			dir := filepath.Clean(filepath.Dir(path))
			seen[dir] = struct{}{}
		}
	}
	dirs := make([]string, 0, len(seen))
	for dir := range seen {
		dirs = append(dirs, dir)
	}
	sort.Strings(dirs)
	return dirs
}

func isContractSidecarName(name string) bool {
	lower := strings.ToLower(name)
	return lower == "contract.json" ||
		strings.HasSuffix(lower, ".contract.json") ||
		strings.HasSuffix(lower, "_contract.json")
}

func parseContractSidecar(path string, content []byte) (contractSidecar, error) {
	var input contractSidecarJSON
	if err := strictDecodeJSON(content, &input); err != nil {
		return contractSidecar{}, fmt.Errorf("invalid contract sidecar JSON: %w", err)
	}
	if input.Schema != contractSidecarSchema {
		return contractSidecar{}, fmt.Errorf("contract sidecar schema = %q, want %q", input.Schema, contractSidecarSchema)
	}
	function := strings.TrimSpace(input.Function)
	if function == "" {
		return contractSidecar{}, fmt.Errorf("contract sidecar requires non-empty function identity")
	}
	return contractSidecar{
		Path:     path,
		Function: function,
		Requires: input.Requires,
		Ensures:  input.Ensures,
		Modifies: input.Modifies,
		Loops:    input.Loops,
	}, nil
}

func strictDecodeJSON(content []byte, output any) error {
	decoder := json.NewDecoder(bytes.NewReader(content))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(output); err != nil {
		return err
	}
	var trailing any
	if err := decoder.Decode(&trailing); err != io.EOF {
		if err == nil {
			return fmt.Errorf("unexpected trailing JSON value")
		}
		return err
	}
	return nil
}

func contractTargets(module *girModule) (map[string][]contractTarget, []rejectedFeature) {
	targets := make(map[string][]contractTarget)
	for packageIndex := range module.Packages {
		pkg := module.Packages[packageIndex]
		for functionIndex := range pkg.Functions {
			function := pkg.Functions[functionIndex]
			target := contractTarget{
				PackageIndex:  packageIndex,
				FunctionIndex: functionIndex,
			}
			identities := []string{function.ID}
			if strings.HasPrefix(function.ID, pkg.PackagePath+".") {
				identities = append(identities, pkg.Name+strings.TrimPrefix(function.ID, pkg.PackagePath))
			}
			sort.Strings(identities)
			for _, identity := range identities {
				if identity == "" {
					continue
				}
				targets[identity] = appendUniqueContractTarget(targets[identity], target)
			}
		}
	}
	return targets, nil
}

func appendUniqueContractTarget(targets []contractTarget, target contractTarget) []contractTarget {
	for _, existing := range targets {
		if existing == target {
			return targets
		}
	}
	return append(targets, target)
}

func validateContractSidecar(baseDir string, sidecar contractSidecar, function girFunction) (girContracts, []rejectedFeature) {
	var findings []rejectedFeature
	context := contractValidationContextForFunction(function)
	contracts := emptyGIRContracts()

	if len(sidecar.Ensures) == 0 {
		findings = append(findings, contractFinding(baseDir, sidecar.Path, "contract sidecar requires at least one postcondition"))
	}
	for _, raw := range sidecar.Requires {
		expr, err := parseContractExpr(raw, context)
		if err != nil {
			findings = append(findings, contractFinding(baseDir, sidecar.Path, err.Error()))
			continue
		}
		contracts.Requires = append(contracts.Requires, expr)
	}
	for _, raw := range sidecar.Ensures {
		expr, err := parseContractExpr(raw, context)
		if err != nil {
			findings = append(findings, contractFinding(baseDir, sidecar.Path, err.Error()))
			continue
		}
		contracts.Ensures = append(contracts.Ensures, expr)
	}
	if len(sidecar.Modifies) > 0 {
		findings = append(findings, contractFinding(baseDir, sidecar.Path, "non-empty modifies are rejected by Go subset v0"))
	} else if sidecar.Modifies != nil {
		contracts.Modifies = append([]string{}, sidecar.Modifies...)
	}
	for _, loop := range sidecar.Loops {
		contract, err := parseLoopContract(loop, context)
		if err != nil {
			findings = append(findings, contractFinding(baseDir, sidecar.Path, err.Error()))
			continue
		}
		contracts.Loops = append(contracts.Loops, contract)
	}

	return contracts, findings
}

func contractValidationContextForFunction(function girFunction) contractValidationContext {
	variables := make(map[string]struct{})
	for _, binding := range function.Params {
		if binding.Name != "" && binding.Name != "_" {
			variables[binding.Name] = struct{}{}
		}
	}
	for _, binding := range function.Locals {
		if binding.Name != "" && binding.Name != "_" {
			variables[binding.Name] = struct{}{}
		}
	}
	blockLabels := make(map[string]struct{})
	for _, block := range function.Blocks {
		if block.Label != "" {
			blockLabels[block.Label] = struct{}{}
		}
	}
	return contractValidationContext{
		Variables:  variables,
		ResultLen:  len(function.Results),
		BlockLabel: blockLabels,
	}
}

func parseLoopContract(input contractLoopSidecar, context contractValidationContext) (girLoopContract, error) {
	blockID := strings.TrimSpace(input.BlockID)
	if blockID == "" {
		return girLoopContract{}, fmt.Errorf("loop contracts require block_id")
	}
	if _, ok := context.BlockLabel[blockID]; !ok {
		return girLoopContract{}, fmt.Errorf("loop contract block_id %q does not resolve to a lowered GIR block", blockID)
	}
	if len(input.Invariants) == 0 {
		return girLoopContract{}, fmt.Errorf("loop contracts require at least one invariant")
	}

	invariants := make([]girContractExpr, 0, len(input.Invariants))
	for _, raw := range input.Invariants {
		expr, err := parseContractExpr(raw, context)
		if err != nil {
			return girLoopContract{}, err
		}
		invariants = append(invariants, expr)
	}
	decreases := make([]girContractExpr, 0, len(input.Decreases))
	for _, raw := range input.Decreases {
		expr, err := parseContractExpr(raw, context)
		if err != nil {
			return girLoopContract{}, err
		}
		decreases = append(decreases, expr)
	}
	return girLoopContract{
		BlockID:    blockID,
		Location:   strings.TrimSpace(input.Location),
		Invariants: invariants,
		Decreases:  decreases,
	}, nil
}

func parseContractExpr(raw json.RawMessage, context contractValidationContext) (girContractExpr, error) {
	var input contractExprJSON
	if err := strictDecodeJSON(raw, &input); err != nil {
		return girContractExpr{}, fmt.Errorf("invalid contract expression JSON: %w", err)
	}
	if input.Op == "" {
		return parseContractAtom(input, context)
	}
	if hasContractAtom(input) {
		return girContractExpr{}, fmt.Errorf("contract operator %q cannot also contain atom fields", input.Op)
	}
	switch input.Op {
	case "not", "bv_neg", "bv_not":
		value, err := parseUnaryContractExpr(input, context, false)
		if err != nil {
			return girContractExpr{}, err
		}
		return girContractExpr{Op: input.Op, Value: &value}, nil
	case "and", "or":
		args, err := parseVariadicContractExpr(input, context)
		if err != nil {
			return girContractExpr{}, err
		}
		return girContractExpr{Op: input.Op, Args: args}, nil
	case "convert":
		value, err := parseUnaryContractExpr(input, context, true)
		if err != nil {
			return girContractExpr{}, err
		}
		if input.Type == nil {
			return girContractExpr{}, fmt.Errorf("contract convert expressions require type")
		}
		if err := validateContractTargetType(*input.Type); err != nil {
			return girContractExpr{}, err
		}
		return girContractExpr{Op: input.Op, Value: &value, Type: input.Type}, nil
	default:
		if !isBinaryContractOp(input.Op) {
			return girContractExpr{}, fmt.Errorf("unsupported contract expression operator %q", input.Op)
		}
		lhs, rhs, err := parseBinaryContractExpr(input, context)
		if err != nil {
			return girContractExpr{}, err
		}
		return girContractExpr{Op: input.Op, LHS: &lhs, RHS: &rhs}, nil
	}
}

func parseContractAtom(input contractExprJSON, context contractValidationContext) (girContractExpr, error) {
	count := 0
	if input.Var != nil {
		count++
	}
	if input.Result != nil {
		count++
	}
	if input.Bool != nil {
		count++
	}
	if input.Int != nil {
		count++
	}
	if count != 1 {
		return girContractExpr{}, fmt.Errorf("contract expressions require exactly one atom or operator")
	}
	if input.Args != nil || input.LHS != nil || input.RHS != nil || input.Value != nil || input.Type != nil {
		return girContractExpr{}, fmt.Errorf("contract atom cannot contain operator fields")
	}
	if input.Var != nil {
		name := strings.TrimSpace(*input.Var)
		if name == "" {
			return girContractExpr{}, fmt.Errorf("contract var atom requires a non-empty variable name")
		}
		if _, ok := context.Variables[name]; !ok {
			return girContractExpr{}, fmt.Errorf("contract var %q does not resolve to a parameter or local", name)
		}
		return girContractExpr{Var: name}, nil
	}
	if input.Result != nil {
		if *input.Result < 0 || *input.Result >= context.ResultLen {
			return girContractExpr{}, fmt.Errorf("contract result index %d is out of range", *input.Result)
		}
		result := *input.Result
		return girContractExpr{Result: &result}, nil
	}
	if input.Bool != nil {
		value := *input.Bool
		return girContractExpr{Bool: &value}, nil
	}
	lit, err := validateContractIntLiteral(*input.Int)
	if err != nil {
		return girContractExpr{}, err
	}
	return girContractExpr{Int: &lit}, nil
}

func hasContractAtom(input contractExprJSON) bool {
	return input.Var != nil || input.Result != nil || input.Bool != nil || input.Int != nil
}

func parseUnaryContractExpr(input contractExprJSON, context contractValidationContext, allowType bool) (girContractExpr, error) {
	if input.LHS != nil || input.RHS != nil {
		return girContractExpr{}, fmt.Errorf("contract operator %q expects one value", input.Op)
	}
	if input.Type != nil && !allowType {
		return girContractExpr{}, fmt.Errorf("contract operator %q does not accept type", input.Op)
	}
	if input.Value != nil {
		if len(input.Args) > 0 {
			return girContractExpr{}, fmt.Errorf("contract operator %q expects either value or one arg", input.Op)
		}
		return parseContractExpr(*input.Value, context)
	}
	if len(input.Args) != 1 {
		return girContractExpr{}, fmt.Errorf("contract operator %q requires one value", input.Op)
	}
	return parseContractExpr(input.Args[0], context)
}

func parseVariadicContractExpr(input contractExprJSON, context contractValidationContext) ([]girContractExpr, error) {
	if input.Value != nil || input.LHS != nil || input.RHS != nil || input.Type != nil {
		return nil, fmt.Errorf("contract operator %q expects args", input.Op)
	}
	if len(input.Args) < 2 {
		return nil, fmt.Errorf("contract operator %q requires at least two args", input.Op)
	}
	args := make([]girContractExpr, 0, len(input.Args))
	for _, raw := range input.Args {
		expr, err := parseContractExpr(raw, context)
		if err != nil {
			return nil, err
		}
		args = append(args, expr)
	}
	return args, nil
}

func parseBinaryContractExpr(input contractExprJSON, context contractValidationContext) (girContractExpr, girContractExpr, error) {
	if input.Value != nil || input.Type != nil {
		return girContractExpr{}, girContractExpr{}, fmt.Errorf("contract operator %q expects lhs/rhs or two args", input.Op)
	}
	if input.LHS != nil || input.RHS != nil {
		if input.LHS == nil || input.RHS == nil || len(input.Args) > 0 {
			return girContractExpr{}, girContractExpr{}, fmt.Errorf("contract operator %q expects both lhs and rhs", input.Op)
		}
		lhs, err := parseContractExpr(*input.LHS, context)
		if err != nil {
			return girContractExpr{}, girContractExpr{}, err
		}
		rhs, err := parseContractExpr(*input.RHS, context)
		if err != nil {
			return girContractExpr{}, girContractExpr{}, err
		}
		return lhs, rhs, nil
	}
	if len(input.Args) != 2 {
		return girContractExpr{}, girContractExpr{}, fmt.Errorf("contract operator %q requires exactly two args", input.Op)
	}
	lhs, err := parseContractExpr(input.Args[0], context)
	if err != nil {
		return girContractExpr{}, girContractExpr{}, err
	}
	rhs, err := parseContractExpr(input.Args[1], context)
	if err != nil {
		return girContractExpr{}, girContractExpr{}, err
	}
	return lhs, rhs, nil
}

func isBinaryContractOp(op string) bool {
	switch op {
	case "eq", "not_eq",
		"signed_lt", "signed_le", "signed_gt", "signed_ge",
		"unsigned_lt", "unsigned_le", "unsigned_gt", "unsigned_ge",
		"bv_add", "bv_sub", "bv_mul",
		"bv_sdiv", "bv_udiv", "bv_srem", "bv_urem",
		"bv_and", "bv_or", "bv_xor", "bv_shl", "bv_ashr", "bv_lshr":
		return true
	default:
		return false
	}
}

func validateContractTargetType(typ girType) error {
	switch typ.Kind {
	case "bool":
		if typ.Name != "" || typ.Width != 0 || typ.Signed != nil || typ.Length != 0 || typ.Element != nil || len(typ.Fields) != 0 {
			return fmt.Errorf("contract bool type contains unsupported fields")
		}
		return nil
	case "bv":
		if typ.Name != "" || typ.Length != 0 || typ.Element != nil || len(typ.Fields) != 0 {
			return fmt.Errorf("contract bitvector type contains unsupported fields")
		}
		if typ.Signed == nil {
			return fmt.Errorf("contract bitvector types require signed")
		}
		if !validContractBitWidth(typ.Width) {
			return fmt.Errorf("contract bitvector width %d is not supported", typ.Width)
		}
		return nil
	default:
		return fmt.Errorf("contract convert target type %q is not supported", typ.Kind)
	}
}

func validateContractIntLiteral(input contractIntJSON) (girIntLiteral, error) {
	if input.Signed == nil {
		return girIntLiteral{}, fmt.Errorf("contract integer literals require signed")
	}
	lit := girIntLiteral{
		Value:  input.Value,
		Width:  input.Width,
		Signed: *input.Signed,
	}
	if !validContractBitWidth(lit.Width) {
		return girIntLiteral{}, fmt.Errorf("contract integer literal width %d is not supported", lit.Width)
	}
	value, ok := new(big.Int).SetString(lit.Value, 0)
	if !ok {
		return girIntLiteral{}, fmt.Errorf("contract integer literal %q cannot be parsed", lit.Value)
	}
	if lit.Signed {
		min := new(big.Int).Lsh(big.NewInt(1), uint(lit.Width-1))
		min.Neg(min)
		max := new(big.Int).Lsh(big.NewInt(1), uint(lit.Width-1))
		max.Sub(max, big.NewInt(1))
		if value.Cmp(min) < 0 || value.Cmp(max) > 0 {
			return girIntLiteral{}, fmt.Errorf("contract integer literal %q does not fit signed %d-bit value", lit.Value, lit.Width)
		}
		return lit, nil
	}
	max := new(big.Int).Lsh(big.NewInt(1), uint(lit.Width))
	max.Sub(max, big.NewInt(1))
	if value.Sign() < 0 || value.Cmp(max) > 0 {
		return girIntLiteral{}, fmt.Errorf("contract integer literal %q does not fit unsigned %d-bit value", lit.Value, lit.Width)
	}
	return lit, nil
}

func validContractBitWidth(width int) bool {
	return width == 8 || width == 16 || width == 32 || width == 64
}

func contractFinding(baseDir string, path string, reason string) rejectedFeature {
	return rejectedFeature{
		Location: normalizePath(baseDir, path),
		Feature:  "contract sidecar",
		Reason:   reason,
	}
}
