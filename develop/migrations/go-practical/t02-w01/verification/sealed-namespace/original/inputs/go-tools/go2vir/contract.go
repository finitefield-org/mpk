package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"regexp"
	"sort"
	"strings"
)

const contractSchema = "mpk.go.contract.v0"

var contractOperatorPattern = regexp.MustCompile(`^[a-z][a-z0-9_]{0,63}$`)

type sourceContract struct {
	path     string
	function string
	requires []jsonValue
	ensures  []jsonValue
	modifies []string
	loops    []jsonValue
}

type contractContext struct {
	function     *virFunction
	names        map[string]string
	loopNames    map[string]string
	types        map[string]virType
	allowResults bool
	allowLocals  bool
}

func attachContracts(module *virModule, capture sourceCapture, loaded packageLoadResult) []loweringFinding {
	contexts, aliases := contractContexts(module, loaded)
	contracts := make([]sourceContract, 0)
	for _, input := range capture.Inputs {
		if input.Kind != contractInputKind {
			continue
		}
		contract, finding := parseSourceContract(input)
		if finding != nil {
			return []loweringFinding{*finding}
		}
		contracts = append(contracts, contract)
	}
	sort.Slice(contracts, func(i, j int) bool { return contracts[i].path < contracts[j].path })
	attached := make(map[string]string)
	for _, input := range contracts {
		id := strings.TrimSpace(input.function)
		if !validContractFunctionID(id) {
			return []loweringFinding{{Code: "GO_CONTRACT_FUNCTION", Message: "contract function spelling is invalid", Origin: contractOrigin(input.path)}}
		}
		context := contexts[id]
		if context == nil {
			resolved := aliases[id]
			if len(resolved) == 1 {
				id, context = resolved[0], contexts[resolved[0]]
			} else if len(resolved) > 1 {
				return []loweringFinding{{Code: "GO_CONTRACT_FUNCTION", Message: fmt.Sprintf("contract function %q is ambiguous", id), Origin: contractOrigin(input.path)}}
			}
		}
		if context == nil {
			return []loweringFinding{{Code: "GO_CONTRACT_FUNCTION", Message: fmt.Sprintf("contract function %q does not resolve to an included Go function", id), Origin: contractOrigin(input.path)}}
		}
		if _, exists := attached[id]; exists {
			return []loweringFinding{{Code: "GO_CONTRACT_DUPLICATE", Message: "multiple sidecars target one function", FunctionID: id, Origin: contractOrigin(input.path)}}
		}
		contract, finding := normalizeSourceContract(input, *context)
		if finding != nil {
			finding.FunctionID = id
			return []loweringFinding{*finding}
		}
		context.function.Contracts = contract
		attached[id] = input.path
	}
	for id, context := range contexts {
		if len(context.function.loopHeaders) > 0 {
			if _, exists := attached[id]; !exists {
				return []loweringFinding{{Code: "GO_SUBSET_LOOP", Message: "every loop requires an explicit sidecar contract", FunctionID: id, Origin: context.function.origin}}
			}
		}
	}
	if err := hashContractsAndBindCalls(module); err != nil {
		return []loweringFinding{{Code: "GO_FRONTEND_INTERNAL", Message: "normalized contracts cannot be hashed"}}
	}
	return nil
}

func parseSourceContract(input capturedInput) (sourceContract, *loweringFinding) {
	value, err := decodeStrictJSON(input.Bytes)
	if err != nil {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_JSON", Message: "invalid contract sidecar JSON", Origin: contractOrigin(input.NormalizedPath)}
	}
	if containsJSONNull(value) {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract contains explicit null", Origin: contractOrigin(input.NormalizedPath)}
	}
	root, ok := value.(map[string]jsonValue)
	if !ok || !exactJSONFields(root, []string{"schema", "function", "requires", "ensures", "modifies", "loops"}, []string{"schema"}) {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract root has invalid fields", Origin: contractOrigin(input.NormalizedPath)}
	}
	schema, schemaOK := root["schema"].(string)
	if !schemaOK || schema != contractSchema {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract schema fields are invalid", Origin: contractOrigin(input.NormalizedPath)}
	}
	functionRaw, functionExists := root["function"]
	if !functionExists {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_FUNCTION", Message: "contract function is missing", Origin: contractOrigin(input.NormalizedPath)}
	}
	function, functionOK := functionRaw.(string)
	if !functionOK {
		return sourceContract{}, contractSchemaFinding(input.NormalizedPath)
	}
	if strings.TrimSpace(function) == "" {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_FUNCTION", Message: "contract function is empty", Origin: contractOrigin(input.NormalizedPath)}
	}
	requires, ok := optionalJSONArray(root, "requires")
	if !ok {
		return sourceContract{}, contractSchemaFinding(input.NormalizedPath)
	}
	ensuresRaw, ensuresExists := root["ensures"]
	if !ensuresExists {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_ENSURES", Message: "contract ensures is missing", Origin: contractOrigin(input.NormalizedPath)}
	}
	ensures, ok := ensuresRaw.([]jsonValue)
	if !ok {
		return sourceContract{}, contractSchemaFinding(input.NormalizedPath)
	}
	if len(ensures) == 0 {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_ENSURES", Message: "contract ensures must be nonempty", Origin: contractOrigin(input.NormalizedPath)}
	}
	loops, ok := optionalJSONArray(root, "loops")
	if !ok {
		return sourceContract{}, contractSchemaFinding(input.NormalizedPath)
	}
	modifiesRaw, ok := optionalJSONArray(root, "modifies")
	if !ok {
		return sourceContract{}, contractSchemaFinding(input.NormalizedPath)
	}
	modifies := make([]string, 0, len(modifiesRaw))
	for _, raw := range modifiesRaw {
		value, ok := raw.(string)
		if !ok {
			return sourceContract{}, contractSchemaFinding(input.NormalizedPath)
		}
		modifies = append(modifies, value)
	}
	if len(modifies) != 0 {
		return sourceContract{}, &loweringFinding{Code: "GO_CONTRACT_MODIFIES", Message: "contract modifies must be empty", Origin: contractOrigin(input.NormalizedPath)}
	}
	return sourceContract{path: input.NormalizedPath, function: function, requires: requires, ensures: ensures, modifies: modifies, loops: loops}, nil
}

func normalizeSourceContract(input sourceContract, context contractContext) (virContract, *loweringFinding) {
	contract := defaultVIRContract(context.function.UnitID, context.function.ID)
	contract.Ensures = []virContractExpr{}
	for _, raw := range input.requires {
		requireContext := context
		requireContext.allowResults, requireContext.allowLocals = false, false
		expression, typ, finding := normalizeContractExpression(raw, requireContext)
		if finding != nil {
			finding.Origin = contractOrigin(input.path)
			return virContract{}, finding
		}
		if typ.Kind != "bool" {
			return virContract{}, contractTypeFinding(input.path)
		}
		contract.Requires = append(contract.Requires, expression)
	}
	for _, raw := range input.ensures {
		ensureContext := context
		ensureContext.allowResults, ensureContext.allowLocals = true, false
		expression, typ, finding := normalizeContractExpression(raw, ensureContext)
		if finding != nil {
			finding.Origin = contractOrigin(input.path)
			return virContract{}, finding
		}
		if typ.Kind != "bool" {
			return virContract{}, contractTypeFinding(input.path)
		}
		contract.Ensures = append(contract.Ensures, expression)
	}
	contract.Modifies = append([]string{}, input.modifies...)
	contract.Loops = []virLoopContract{}
	for _, raw := range input.loops {
		loop, finding := normalizeLoopContract(raw, input.path, context)
		if finding != nil {
			return virContract{}, finding
		}
		contract.Loops = append(contract.Loops, loop)
	}
	if len(contract.Loops) != len(context.function.loopHeaders) {
		return virContract{}, &loweringFinding{Code: "GO_CONTRACT_LOOP", Message: "sidecar loops do not match the complete source loop set", Origin: contractOrigin(input.path)}
	}
	for index := range contract.Loops {
		if contract.Loops[index].Header != context.function.loopHeaders[index] {
			return virContract{}, &loweringFinding{Code: "GO_CONTRACT_LOOP", Message: "loop headers are not in canonical order", Origin: contractOrigin(input.path)}
		}
	}
	if len(contract.Loops) > 0 {
		partial, total := false, false
		for _, loop := range contract.Loops {
			if len(loop.Decreases) == 0 {
				partial = true
			} else {
				total = true
			}
		}
		if partial && total {
			return virContract{}, &loweringFinding{Code: "GO_CONTRACT_LOOP", Message: "loop termination modes cannot be mixed", Origin: contractOrigin(input.path)}
		}
		if partial {
			contract.Termination = "partial"
		}
	}
	return contract, nil
}

func normalizeLoopContract(raw jsonValue, path string, context contractContext) (virLoopContract, *loweringFinding) {
	object, ok := raw.(map[string]jsonValue)
	if !ok || !exactJSONFields(object, []string{"block_id", "location", "invariants", "decreases"}, []string{"block_id", "invariants"}) {
		return virLoopContract{}, contractLoopFinding(path)
	}
	header, ok := object["block_id"].(string)
	if !ok || strings.TrimSpace(header) == "" {
		return virLoopContract{}, contractLoopFinding(path)
	}
	if location, exists := object["location"]; exists {
		if _, ok := location.(string); !ok {
			return virLoopContract{}, contractLoopFinding(path)
		}
	}
	invariants, ok := optionalJSONArray(object, "invariants")
	if !ok || len(invariants) == 0 {
		return virLoopContract{}, contractLoopFinding(path)
	}
	decreases, ok := optionalJSONArray(object, "decreases")
	if !ok {
		return virLoopContract{}, contractLoopFinding(path)
	}
	result := virLoopContract{Header: strings.TrimSpace(header), Invariants: []virContractExpr{}, Decreases: []virContractExpr{}}
	context.allowResults, context.allowLocals = true, true
	context.names = cloneContractNames(context.names)
	for sourceName, id := range context.loopNames {
		context.names[sourceName] = id
	}
	if parameters := context.function.loopParameters[result.Header]; parameters != nil {
		for sourceName, id := range context.names {
			if parameter := parameters[id]; parameter != "" {
				context.names[sourceName] = parameter
			}
		}
	}
	for _, value := range invariants {
		expression, typ, finding := normalizeContractExpression(value, context)
		if finding != nil || typ.Kind != "bool" {
			return virLoopContract{}, contractLoopFinding(path)
		}
		result.Invariants = append(result.Invariants, expression)
	}
	for _, value := range decreases {
		expression, typ, finding := normalizeContractExpression(value, context)
		if finding != nil || typ.Kind != "bv" {
			return virLoopContract{}, contractLoopFinding(path)
		}
		result.Decreases = append(result.Decreases, expression)
	}
	return result, nil
}

func normalizeContractExpression(raw jsonValue, context contractContext) (virContractExpr, virType, *loweringFinding) {
	object, ok := raw.(map[string]jsonValue)
	if !ok {
		return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract expression must be an object"}
	}
	allowed := []string{"op", "args", "lhs", "rhs", "value", "type", "var", "result", "bool", "int"}
	if !exactJSONFields(object, allowed, nil) {
		return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract expression has an unknown field"}
	}
	if rawOp, hasOp := object["op"]; hasOp {
		op, ok := rawOp.(string)
		if !ok {
			return virContractExpr{}, virType{}, contractSchemaExpressionFinding()
		}
		if !contractOperatorPattern.MatchString(op) {
			return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract operator spelling is invalid"}
		}
		for _, atom := range []string{"var", "result", "bool", "int"} {
			if _, exists := object[atom]; exists {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract operator contains atom fields"}
			}
		}
		switch op {
		case "not", "bv_neg", "bv_not", "convert":
			value, finding := unaryContractValue(object, op)
			if finding != nil {
				return virContractExpr{}, virType{}, finding
			}
			normalized, inputType, finding := normalizeContractExpression(value, context)
			if finding != nil {
				return virContractExpr{}, virType{}, finding
			}
			if op == "convert" {
				target, finding := contractTargetType(object["type"])
				if finding != nil || inputType.Kind != "bv" {
					return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract conversion requires exact BV types"}
				}
				return virContractExpr{Op: op, Value: &normalized, Type: &target}, target, nil
			}
			if _, exists := object["type"]; exists {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "unary operator has an extra type"}
			}
			if op == "not" && inputType.Kind != "bool" || op != "not" && inputType.Kind != "bv" {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract unary operand type is invalid"}
			}
			return virContractExpr{Op: op, Value: &normalized}, inputType, nil
		case "and", "or":
			rawArgs, exists := object["args"]
			if !exists {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract Boolean operator arity is invalid"}
			}
			args, ok := rawArgs.([]jsonValue)
			if !ok {
				return virContractExpr{}, virType{}, contractSchemaExpressionFinding()
			}
			if len(args) < 2 || len(args) > 64 || len(object) != 2 {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract Boolean operator arity is invalid"}
			}
			normalized := make([]virContractExpr, 0, len(args))
			for _, rawArg := range args {
				arg, typ, finding := normalizeContractExpression(rawArg, context)
				if finding != nil {
					return virContractExpr{}, virType{}, finding
				}
				if typ.Kind != "bool" {
					return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract Boolean operand is not bool"}
				}
				normalized = append(normalized, arg)
			}
			return virContractExpr{Op: op, Args: normalized}, virType{Kind: "bool"}, nil
		default:
			if !binaryContractOperator(op) {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: fmt.Sprintf("unsupported contract expression operator %q", op)}
			}
			leftRaw, rightRaw, finding := binaryContractValues(object)
			if finding != nil {
				return virContractExpr{}, virType{}, finding
			}
			left, leftType, finding := normalizeContractExpression(leftRaw, context)
			if finding != nil {
				return virContractExpr{}, virType{}, finding
			}
			right, rightType, finding := normalizeContractExpression(rightRaw, context)
			if finding != nil {
				return virContractExpr{}, virType{}, finding
			}
			shift := op == "bv_shl" || op == "bv_ashr" || op == "bv_lshr"
			if shift {
				if leftType.Kind != "bv" || rightType.Kind != "bv" {
					return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract shift operands must be bitvectors"}
				}
			} else if !typeEqual(leftType, rightType) {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract binary operands differ in type"}
			}
			if strings.HasPrefix(op, "signed_") && (leftType.Kind != "bv" || leftType.Signed == nil || !*leftType.Signed) {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "signed contract comparison requires signed bitvectors"}
			}
			if strings.HasPrefix(op, "unsigned_") && (leftType.Kind != "bv" || leftType.Signed == nil || *leftType.Signed) {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "unsigned contract comparison requires unsigned bitvectors"}
			}
			if strings.HasPrefix(op, "bv_") && leftType.Kind != "bv" {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "bitvector contract operator requires bitvectors"}
			}
			if (op == "bv_sdiv" || op == "bv_srem" || op == "bv_ashr") && (leftType.Signed == nil || !*leftType.Signed) {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract operator requires signed bitvectors"}
			}
			if (op == "bv_udiv" || op == "bv_urem" || op == "bv_lshr") && (leftType.Signed == nil || *leftType.Signed) {
				return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract operator requires unsigned bitvectors"}
			}
			resultType := leftType
			if op == "eq" || op == "not_eq" || strings.HasSuffix(op, "_lt") || strings.HasSuffix(op, "_le") || strings.HasSuffix(op, "_gt") || strings.HasSuffix(op, "_ge") {
				resultType = virType{Kind: "bool"}
			}
			return virContractExpr{Op: op, LHS: &left, RHS: &right}, resultType, nil
		}
	}
	if len(object) != 1 {
		return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract atom must have exactly one field"}
	}
	if raw, exists := object["var"]; exists {
		name, ok := raw.(string)
		if !ok {
			return virContractExpr{}, virType{}, contractSchemaExpressionFinding()
		}
		id := context.names[strings.TrimSpace(name)]
		if strings.HasPrefix(id, "result") && !context.allowResults || strings.HasPrefix(id, "local") && !context.allowLocals {
			return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract variable is not visible in this clause"}
		}
		typ, found := context.types[id]
		if !found {
			return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract variable is not visible"}
		}
		return virContractExpr{Var: id}, typ, nil
	}
	if raw, exists := object["result"]; exists {
		index, ok := raw.(int64)
		if !ok {
			return virContractExpr{}, virType{}, contractSchemaExpressionFinding()
		}
		if !context.allowResults || index < 0 || index >= int64(len(context.function.Results)) {
			return virContractExpr{}, virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract result index is invalid"}
		}
		return virContractExpr{Result: &index}, context.function.Results[index].Type, nil
	}
	if raw, exists := object["bool"]; exists {
		value, ok := raw.(bool)
		if !ok {
			return virContractExpr{}, virType{}, contractSchemaExpressionFinding()
		}
		return virContractExpr{Bool: &value}, virType{Kind: "bool"}, nil
	}
	if raw, exists := object["int"]; exists {
		integer, finding := contractInteger(raw)
		if finding != nil {
			return virContractExpr{}, virType{}, finding
		}
		typ := virType{Kind: "bv", Width: integer.Width, Signed: boolPointer(integer.Signed)}
		return virContractExpr{Int: &integer}, typ, nil
	}
	return virContractExpr{}, virType{}, contractSchemaExpressionFinding()
}

func unaryContractValue(object map[string]jsonValue, op string) (jsonValue, *loweringFinding) {
	allowed := map[string]bool{"op": true, "value": true, "args": true}
	if op == "convert" {
		allowed["type"] = true
	}
	for key := range object {
		if !allowed[key] {
			return nil, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract unary operator has extra fields"}
		}
	}
	value, hasValue := object["value"]
	rawArgs, argsPresent := object["args"]
	args, hasArgs := rawArgs.([]jsonValue)
	if argsPresent && !hasArgs {
		return nil, contractSchemaExpressionFinding()
	}
	if hasValue == hasArgs || hasArgs && len(args) != 1 {
		return nil, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract unary operator arity is invalid"}
	}
	if hasArgs {
		value = args[0]
	}
	if op == "convert" {
		if _, exists := object["type"]; !exists {
			return nil, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract conversion lacks a target type"}
		}
	}
	return value, nil
}

func binaryContractValues(object map[string]jsonValue) (jsonValue, jsonValue, *loweringFinding) {
	for key := range object {
		if key != "op" && key != "lhs" && key != "rhs" && key != "args" {
			return nil, nil, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract binary operator has extra fields"}
		}
	}
	lhs, hasLHS := object["lhs"]
	rhs, hasRHS := object["rhs"]
	rawArgs, argsPresent := object["args"]
	args, hasArgs := rawArgs.([]jsonValue)
	if argsPresent && !hasArgs {
		return nil, nil, contractSchemaExpressionFinding()
	}
	if hasArgs {
		if hasLHS || hasRHS || len(args) != 2 {
			return nil, nil, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract binary operator arity is invalid"}
		}
		return args[0], args[1], nil
	}
	if !hasLHS || !hasRHS {
		return nil, nil, &loweringFinding{Code: "GO_CONTRACT_OPERATOR", Message: "contract binary operator requires lhs and rhs"}
	}
	return lhs, rhs, nil
}

func contractTargetType(raw jsonValue) (virType, *loweringFinding) {
	object, ok := raw.(map[string]jsonValue)
	if !ok || !exactJSONFields(object, []string{"kind", "width", "signed"}, []string{"kind", "width", "signed"}) {
		return virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract conversion target is not an exact BV type"}
	}
	kind, kindOK := object["kind"].(string)
	width, widthOK := object["width"].(int64)
	signed, signedOK := object["signed"].(bool)
	if !kindOK || kind != "bv" || !widthOK || !signedOK || width != 8 && width != 16 && width != 32 && width != 64 {
		return virType{}, &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract conversion target is not an accepted BV type"}
	}
	return virType{Kind: "bv", Width: width, Signed: boolPointer(signed)}, nil
}

func contractInteger(raw jsonValue) (virInteger, *loweringFinding) {
	object, ok := raw.(map[string]jsonValue)
	if !ok || !exactJSONFields(object, []string{"value", "width", "signed"}, []string{"value", "width", "signed"}) {
		return virInteger{}, contractTypeExpressionFinding()
	}
	value, valueOK := object["value"].(string)
	width, widthOK := object["width"].(int64)
	signed, signedOK := object["signed"].(bool)
	if !valueOK || !widthOK || !signedOK {
		return virInteger{}, contractTypeExpressionFinding()
	}
	integer, err := parseContractInteger(value, width, signed)
	if err != nil {
		return virInteger{}, contractTypeExpressionFinding()
	}
	return integer, nil
}

func binaryContractOperator(op string) bool {
	switch op {
	case "eq", "not_eq", "signed_lt", "signed_le", "signed_gt", "signed_ge", "unsigned_lt", "unsigned_le", "unsigned_gt", "unsigned_ge", "bv_add", "bv_sub", "bv_mul", "bv_sdiv", "bv_udiv", "bv_srem", "bv_urem", "bv_and", "bv_or", "bv_xor", "bv_shl", "bv_ashr", "bv_lshr":
		return true
	default:
		return false
	}
}

func validContractFunctionID(value string) bool {
	if value == "" || len(value) > maximumVIRIdentifierBytes {
		return false
	}
	for index, character := range value {
		if character != '.' || index == 0 || index == len(value)-1 {
			continue
		}
		unitID, declaration := value[:index], value[index+1:]
		parts := strings.Split(declaration, ".")
		if !validGoUnitID(unitID) || len(parts) < 1 || len(parts) > 2 {
			continue
		}
		valid := true
		for _, part := range parts {
			valid = valid && validASCIIIdentifier(part)
		}
		if valid {
			return true
		}
	}
	parts := strings.Split(value, ".")
	return len(parts) == 2 && validASCIIIdentifier(parts[0]) && validASCIIIdentifier(parts[1])
}

func contractContexts(module *virModule, loaded packageLoadResult) (map[string]*contractContext, map[string][]string) {
	contexts := make(map[string]*contractContext)
	aliases := make(map[string][]string)
	for unitIndex := range module.Units {
		unit := &module.Units[unitIndex]
		for functionIndex := range unit.Functions {
			function := &unit.Functions[functionIndex]
			contexts[function.ID] = &contractContext{function: function, names: make(map[string]string), loopNames: make(map[string]string), types: make(map[string]virType)}
		}
	}
	for _, loadedPackage := range loaded.Packages {
		for _, file := range loadedPackage.packageValue.Syntax {
			for _, declaration := range file.Decls {
				functionDecl, ok := declaration.(*ast.FuncDecl)
				if !ok {
					continue
				}
				id := canonicalFunctionID(loadedPackage.packageValue, functionDecl)
				context := contexts[id]
				if context == nil {
					continue
				}
				object, _ := loadedPackage.packageValue.TypesInfo.Defs[functionDecl.Name].(*types.Func)
				signature, _ := object.Type().(*types.Signature)
				parameterIndex := 0
				if receiver := signature.Recv(); receiver != nil {
					context.types[fmt.Sprintf("arg%d", parameterIndex)] = context.function.Params[parameterIndex].Type
					if receiver.Name() != "" {
						context.names[receiver.Name()] = fmt.Sprintf("arg%d", parameterIndex)
					}
					parameterIndex++
				}
				for index := 0; index < signature.Params().Len(); index++ {
					variable := signature.Params().At(index)
					id := fmt.Sprintf("arg%d", parameterIndex)
					context.types[id] = context.function.Params[parameterIndex].Type
					if variable.Name() != "" {
						context.names[variable.Name()] = id
					}
					parameterIndex++
				}
				for index := 0; index < signature.Results().Len(); index++ {
					id := fmt.Sprintf("result%d", index)
					context.types[id] = context.function.Results[index].Type
					if name := signature.Results().At(index).Name(); name != "" {
						context.names[name] = id
						context.loopNames[name] = fmt.Sprintf("local%d", len(context.loopNames))
					}
				}
				for index, local := range context.function.Locals {
					context.types[local.ID] = local.Type
					_ = index
				}
				for _, block := range context.function.Blocks {
					for _, parameter := range block.Parameters {
						context.types[parameter.ID] = parameter.Type
					}
				}
				localIndex := 0
				for index := 0; index < signature.Results().Len(); index++ {
					if signature.Results().At(index).Name() != "" {
						localIndex++
					}
				}
				ast.Inspect(functionDecl.Body, func(node ast.Node) bool {
					var names []*ast.Ident
					switch value := node.(type) {
					case *ast.ValueSpec:
						names = value.Names
					case *ast.AssignStmt:
						if value.Tok.String() == ":=" {
							for _, raw := range value.Lhs {
								if name, ok := raw.(*ast.Ident); ok {
									names = append(names, name)
								}
							}
						}
					}
					for _, name := range names {
						if name.Name != "_" && localIndex < len(context.function.Locals) {
							localID := fmt.Sprintf("local%d", localIndex)
							context.names[name.Name] = localID
							context.loopNames[name.Name] = localID
							localIndex++
						}
					}
					return true
				})
				if functionDecl.Recv == nil {
					alias := loadedPackage.Name + "." + functionDecl.Name.Name
					aliases[alias] = append(aliases[alias], id)
				}
			}
		}
	}
	return contexts, aliases
}

func hashContractsAndBindCalls(module *virModule) error {
	hashes := make(map[string]string)
	for unitIndex := range module.Units {
		for functionIndex := range module.Units[unitIndex].Functions {
			contract := &module.Units[unitIndex].Functions[functionIndex].Contracts
			strict, err := strictValueFromTyped(*contract)
			if err != nil {
				return err
			}
			payload, err := withoutRootField(strict, "contract_hash")
			if err != nil {
				return err
			}
			digest, err := hashCanonicalJSON(contractDomain, payload)
			if err != nil {
				return err
			}
			contract.ContractHash = digest
			hashes[contract.FunctionID] = digest
		}
	}
	for unitIndex := range module.Units {
		for functionIndex := range module.Units[unitIndex].Functions {
			function := &module.Units[unitIndex].Functions[functionIndex]
			for blockIndex := range function.Blocks {
				for instructionIndex := range function.Blocks[blockIndex].Instructions {
					instruction := &function.Blocks[blockIndex].Instructions[instructionIndex]
					if instruction.Kind == "CallStatic" {
						digest := hashes[instruction.Function]
						if digest == "" {
							return fmt.Errorf("callee contract hash is unavailable")
						}
						instruction.ContractHash = digest
					}
				}
			}
		}
	}
	return nil
}

func containsJSONNull(value jsonValue) bool {
	switch value := value.(type) {
	case nil:
		return true
	case []jsonValue:
		for _, child := range value {
			if containsJSONNull(child) {
				return true
			}
		}
	case map[string]jsonValue:
		for _, child := range value {
			if containsJSONNull(child) {
				return true
			}
		}
	}
	return false
}

func cloneContractNames(source map[string]string) map[string]string {
	result := make(map[string]string, len(source))
	for name, id := range source {
		result[name] = id
	}
	return result
}
func exactJSONFields(object map[string]jsonValue, allowed, required []string) bool {
	set := make(map[string]bool)
	for _, field := range allowed {
		set[field] = true
	}
	for field := range object {
		if !set[field] {
			return false
		}
	}
	for _, field := range required {
		if _, exists := object[field]; !exists {
			return false
		}
	}
	return true
}
func optionalJSONArray(object map[string]jsonValue, field string) ([]jsonValue, bool) {
	raw, exists := object[field]
	if !exists {
		return []jsonValue{}, true
	}
	values, ok := raw.([]jsonValue)
	return values, ok
}
func contractOrigin(path string) sourceOrigin {
	return sourceOrigin{Kind: "source", InputKind: contractInputKind, NormalizedPath: path, Start: 0, End: 1}
}
func contractSchemaFinding(path string) *loweringFinding {
	return &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract field has the wrong JSON type", Origin: contractOrigin(path)}
}
func contractSchemaExpressionFinding() *loweringFinding {
	return &loweringFinding{Code: "GO_CONTRACT_SCHEMA", Message: "contract atom has the wrong JSON shape"}
}
func contractTypeFinding(path string) *loweringFinding {
	return &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract clause is not Boolean", Origin: contractOrigin(path)}
}
func contractTypeExpressionFinding() *loweringFinding {
	return &loweringFinding{Code: "GO_CONTRACT_TYPE", Message: "contract integer type is invalid"}
}
func contractLoopFinding(path string) *loweringFinding {
	return &loweringFinding{Code: "GO_CONTRACT_LOOP", Message: "contract loop metadata is invalid", Origin: contractOrigin(path)}
}
