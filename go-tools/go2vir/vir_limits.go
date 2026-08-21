package main

const (
	maximumVIRUnits                   = 256
	maximumVIRTypeDecls               = 4_096
	maximumVIRConstDecls              = 65_536
	maximumVIRFunctions               = 8_192
	maximumVIRParams                  = 256
	maximumVIRResults                 = 16
	maximumVIRLocals                  = 65_536
	maximumVIRBlocksPerFunction       = 8_192
	maximumVIRBlocksPerModule         = 65_536
	maximumVIRBlockParameters         = 4_096
	maximumVIRInstructionsPerBlock    = 100_000
	maximumVIRInstructionsPerFunction = 100_000
	maximumVIRInstructionsPerModule   = 250_000
	maximumVIREdgesPerFunction        = 16_000
	maximumVIRCallArgs                = 256
	maximumVIRContractClauses         = 64
	maximumVIRContractNodesFunction   = 1_024
	maximumVIRContractNodesModule     = 8_192
	maximumVIRContractNesting         = 32
	maximumVIRLoops                   = 1_024
	maximumVIRLoopInvariants          = 64
	maximumVIRLoopDecreases           = 64
	maximumVIRIdentifierBytes         = 1_024
)

func validateGeneratedVIRLimits(module virModule) *loweringFinding {
	if len(module.Units) > maximumVIRUnits {
		return virLimitFinding("VIR_LIMIT_UNITS", "VIR unit count exceeds the shared limit", "")
	}
	typeDecls, constDecls, functions := 0, 0, 0
	blocks, instructions, contractNodes, loops := 0, 0, 0, 0
	for _, unit := range module.Units {
		if len(unit.ID) > maximumVIRIdentifierBytes {
			return virLimitFinding("VIR_LIMIT_IDENTIFIER_BYTES", "VIR unit identifier exceeds the shared limit", "")
		}
		typeDecls += len(unit.TypeDecls)
		constDecls += len(unit.ConstDecls)
		functions += len(unit.Functions)
		for _, declaration := range unit.TypeDecls {
			if len(declaration.ID) > maximumVIRIdentifierBytes {
				return virLimitFinding("VIR_LIMIT_IDENTIFIER_BYTES", "VIR type identifier exceeds the shared limit", "")
			}
			if len(declaration.Fields) > 64 {
				return virLimitFinding("VIR_LIMIT_STRUCT_FIELDS", "VIR struct field count exceeds the shared limit", "")
			}
		}
		for _, declaration := range unit.ConstDecls {
			if len(declaration.ID) > maximumVIRIdentifierBytes {
				return virLimitFinding("VIR_LIMIT_IDENTIFIER_BYTES", "VIR constant identifier exceeds the shared limit", "")
			}
		}
		for _, function := range unit.Functions {
			if finding := validateGeneratedFunctionLimits(function); finding != nil {
				return finding
			}
			blocks += len(function.Blocks)
			loops += len(function.Contracts.Loops)
			for _, block := range function.Blocks {
				instructions += len(block.Instructions)
			}
			nodes, _ := contractExpressionSize(function.Contracts)
			contractNodes += nodes
		}
	}
	for _, total := range []struct {
		value   int
		maximum int
		code    string
		message string
	}{
		{typeDecls, maximumVIRTypeDecls, "VIR_LIMIT_TYPE_DECLS", "VIR type declaration count exceeds the shared limit"},
		{constDecls, maximumVIRConstDecls, "VIR_LIMIT_CONST_DECLS", "VIR constant declaration count exceeds the shared limit"},
		{functions, maximumVIRFunctions, "VIR_LIMIT_FUNCTIONS", "VIR function count exceeds the shared limit"},
		{blocks, maximumVIRBlocksPerModule, "VIR_LIMIT_BLOCKS_PER_MODULE", "VIR module block count exceeds the shared limit"},
		{instructions, maximumVIRInstructionsPerModule, "VIR_LIMIT_INSTRUCTIONS_PER_MODULE", "VIR module instruction count exceeds the shared limit"},
		{contractNodes, maximumVIRContractNodesModule, "VIR_LIMIT_CONTRACT_EXPR_NODES_PER_MODULE", "VIR module contract expression count exceeds the shared limit"},
		{loops, maximumVIRLoops, "VIR_LIMIT_LOOPS", "VIR loop count exceeds the shared limit"},
	} {
		if total.value > total.maximum {
			return virLimitFinding(total.code, total.message, "")
		}
	}
	return nil
}

func validateGeneratedFunctionLimits(function virFunction) *loweringFinding {
	if len(function.ID) > maximumVIRIdentifierBytes || len(function.UnitID) > maximumVIRIdentifierBytes {
		return virLimitFinding("VIR_LIMIT_IDENTIFIER_BYTES", "VIR function identifier exceeds the shared limit", function.ID)
	}
	checks := []struct {
		value   int
		maximum int
		code    string
		message string
	}{
		{len(function.Params), maximumVIRParams, "VIR_LIMIT_PARAMS", "VIR parameter count exceeds the shared limit"},
		{len(function.Results), maximumVIRResults, "VIR_LIMIT_RESULTS", "VIR result count exceeds the shared limit"},
		{len(function.Locals), maximumVIRLocals, "VIR_LIMIT_LOCALS", "VIR local count exceeds the shared limit"},
		{len(function.Blocks), maximumVIRBlocksPerFunction, "VIR_LIMIT_BLOCKS_PER_FUNCTION", "VIR function block count exceeds the shared limit"},
		{len(function.Contracts.Requires) + len(function.Contracts.Ensures), maximumVIRContractClauses, "VIR_LIMIT_CONTRACT_CLAUSES", "VIR contract clause count exceeds the shared limit"},
	}
	for _, check := range checks {
		if check.value > check.maximum {
			return virLimitFinding(check.code, check.message, function.ID)
		}
	}
	instructionCount, edgeCount := 0, 0
	for _, block := range function.Blocks {
		if len(block.Parameters) > maximumVIRBlockParameters {
			return virLimitFinding("VIR_LIMIT_BLOCK_PARAMETERS", "VIR block parameter count exceeds the shared limit", function.ID)
		}
		if len(block.Instructions) > maximumVIRInstructionsPerBlock {
			return virLimitFinding("VIR_LIMIT_INSTRUCTIONS_PER_BLOCK", "VIR block instruction count exceeds the shared limit", function.ID)
		}
		instructionCount += len(block.Instructions)
		for _, instruction := range block.Instructions {
			if instruction.Kind == "CallStatic" && len(instruction.Args) > maximumVIRCallArgs {
				return virLimitFinding("VIR_LIMIT_CALL_ARGS", "VIR call argument count exceeds the shared limit", function.ID)
			}
			if instruction.Kind == "MakeArray" && len(instruction.Elements) > 256 {
				return virLimitFinding("VIR_LIMIT_ARRAY_ELEMENTS", "VIR array element count exceeds the shared limit", function.ID)
			}
			if instruction.Kind == "MakeStruct" && len(instruction.Fields) > 64 {
				return virLimitFinding("VIR_LIMIT_STRUCT_FIELDS", "VIR struct field count exceeds the shared limit", function.ID)
			}
		}
		switch block.Terminator.Kind {
		case "Jump":
			edgeCount++
		case "Branch":
			edgeCount += 2
		}
	}
	if instructionCount > maximumVIRInstructionsPerFunction {
		return virLimitFinding("VIR_LIMIT_INSTRUCTIONS_PER_FUNCTION", "VIR function instruction count exceeds the shared limit", function.ID)
	}
	if edgeCount > maximumVIREdgesPerFunction {
		return virLimitFinding("VIR_LIMIT_CFG_EDGES_PER_FUNCTION", "VIR function edge count exceeds the shared limit", function.ID)
	}
	for _, loop := range function.Contracts.Loops {
		if len(loop.Invariants) > maximumVIRLoopInvariants {
			return virLimitFinding("VIR_LIMIT_LOOP_INVARIANTS", "VIR loop invariant count exceeds the shared limit", function.ID)
		}
		if len(loop.Decreases) > maximumVIRLoopDecreases {
			return virLimitFinding("VIR_LIMIT_LOOP_DECREASES", "VIR loop decreases count exceeds the shared limit", function.ID)
		}
	}
	nodes, depth := contractExpressionSize(function.Contracts)
	if nodes > maximumVIRContractNodesFunction {
		return virLimitFinding("VIR_LIMIT_CONTRACT_EXPR_NODES_PER_FUNCTION", "VIR function contract expression count exceeds the shared limit", function.ID)
	}
	if depth > maximumVIRContractNesting {
		return virLimitFinding("VIR_LIMIT_CONTRACT_EXPR_NESTING", "VIR contract expression nesting exceeds the shared limit", function.ID)
	}
	return nil
}

func contractExpressionSize(contract virContract) (int, int) {
	nodes, depth := 0, 0
	visit := func(expression virContractExpr) {
		count, nesting := contractNodeSize(expression)
		nodes += count
		if nesting > depth {
			depth = nesting
		}
	}
	for _, expression := range contract.Requires {
		visit(expression)
	}
	for _, expression := range contract.Ensures {
		visit(expression)
	}
	for _, loop := range contract.Loops {
		for _, expression := range loop.Invariants {
			visit(expression)
		}
		for _, expression := range loop.Decreases {
			visit(expression)
		}
	}
	return nodes, depth
}

func contractNodeSize(expression virContractExpr) (int, int) {
	nodes, depth := 1, 1
	children := append([]virContractExpr{}, expression.Args...)
	for _, child := range []*virContractExpr{expression.LHS, expression.RHS, expression.Value} {
		if child != nil {
			children = append(children, *child)
		}
	}
	for _, child := range children {
		childNodes, childDepth := contractNodeSize(child)
		nodes += childNodes
		if childDepth+1 > depth {
			depth = childDepth + 1
		}
	}
	return nodes, depth
}

func virLimitFinding(code, message, functionID string) *loweringFinding {
	return &loweringFinding{Code: code, Message: message, FunctionID: functionID}
}
