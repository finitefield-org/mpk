package main

import (
	"fmt"
	"go/ast"
	"go/constant"
	"go/token"
	"go/types"
	"sort"
	"strings"

	"golang.org/x/tools/go/packages"
)

type virLoweringResult struct {
	Module virModule
	Calls  map[string][]string
}

type packageLowerer struct {
	pkg      *packages.Package
	paths    map[string]string
	structs  map[string]*types.Struct
	findings []loweringFinding
}

type functionLowerer struct {
	owner          *packageLowerer
	declaration    *ast.FuncDecl
	function       virFunction
	blocks         []virBlock
	current        int
	bindings       map[types.Object]string
	localTypes     map[string]virType
	loopHeaders    []string
	loopParameters map[string]map[string]string
	environment    map[string]virValue
	namedResults   []string
}

func lowerLoadedGo(loaded packageLoadResult) (virLoweringResult, []loweringFinding) {
	if findings := detectGoFeatures(loaded); len(findings) > 0 {
		return virLoweringResult{}, findings
	}
	if err := validateSSAAttribution(loaded); err != nil {
		return virLoweringResult{}, []loweringFinding{{Code: "GO_LOWER_PATTERN", Message: "typed source has no closed SSA attribution"}}
	}
	units := make([]virUnit, 0, len(loaded.Packages))
	allFindings := make([]loweringFinding, 0)
	calls := make(map[string][]string)
	structs := loadedStructTypes(loaded)
	for _, loadedPackage := range loaded.Packages {
		lowerer := packageLowerer{pkg: loadedPackage.packageValue, paths: packageSourcePaths(loadedPackage), structs: structs}
		unit := lowerer.lowerUnit()
		if len(unit.Functions) == 0 {
			lowerer.findings = append(lowerer.findings, loweringFinding{Code: "GO_SUBSET_IMPORT", Message: "every captured package must declare at least one function"})
		}
		units = append(units, unit)
		allFindings = append(allFindings, lowerer.findings...)
		for _, function := range unit.Functions {
			for _, block := range function.Blocks {
				for _, instruction := range block.Instructions {
					if instruction.Kind == "CallStatic" {
						calls[function.ID] = append(calls[function.ID], instruction.Function)
					}
				}
			}
		}
	}
	if len(allFindings) > 0 {
		sortLoweringFindings(allFindings)
		return virLoweringResult{}, allFindings
	}
	module := virModule{
		Schema: virSchema, SourceLanguage: "go", SemanticProfile: goSemanticProfile,
		SemanticParameters: semanticParameters{TargetID: goTarget, PointerWidth: goPointerWidth},
		Units:              units, VIRHash: zeroSHA256(),
	}
	if finding := canonicalizeFunctionOrder(&module, calls); finding != nil {
		return virLoweringResult{}, []loweringFinding{*finding}
	}
	return virLoweringResult{Module: module, Calls: calls}, nil
}

func (l *packageLowerer) lowerUnit() virUnit {
	unit := virUnit{
		ID: l.pkg.PkgPath, Name: l.pkg.Name,
		TypeDecls: []virTypeDecl{}, ConstDecls: []virConstDecl{}, Functions: []virFunction{},
	}
	for _, file := range l.pkg.Syntax {
		for _, declaration := range file.Decls {
			switch declaration := declaration.(type) {
			case *ast.GenDecl:
				switch declaration.Tok {
				case token.TYPE:
					for _, raw := range declaration.Specs {
						spec := raw.(*ast.TypeSpec)
						object, _ := l.pkg.TypesInfo.Defs[spec.Name].(*types.TypeName)
						named, _ := object.Type().(*types.Named)
						decl, ok := virStructDecl(named)
						if !ok {
							l.reject(spec, "GO_LOWER_TYPE", "struct declaration cannot be normalized")
							continue
						}
						unit.TypeDecls = append(unit.TypeDecls, decl)
					}
				case token.CONST:
					for _, raw := range declaration.Specs {
						spec := raw.(*ast.ValueSpec)
						if len(spec.Names) != 1 {
							continue
						}
						object, _ := l.pkg.TypesInfo.Defs[spec.Names[0]].(*types.Const)
						if object == nil {
							l.reject(spec, "GO_LOWER_CONSTANT", "constant lacks compiler identity")
							continue
						}
						typ, ok := virTypeFromGo(object.Type())
						if !ok {
							l.reject(spec, "GO_LOWER_TYPE", "constant type cannot be normalized")
							continue
						}
						literal, err := literalFromConstant(object.Val(), typ)
						if err != nil {
							l.reject(spec, "GO_LOWER_CONSTANT", "constant value cannot be normalized")
							continue
						}
						unit.ConstDecls = append(unit.ConstDecls, virConstDecl{ID: l.pkg.PkgPath + "." + object.Name(), Name: object.Name(), Type: typ, Value: literal})
					}
				}
			case *ast.FuncDecl:
				function, ok := l.lowerFunction(declaration)
				if ok {
					unit.Functions = append(unit.Functions, function)
				}
			}
		}
	}
	sort.Slice(unit.TypeDecls, func(i, j int) bool { return unit.TypeDecls[i].ID < unit.TypeDecls[j].ID })
	sort.Slice(unit.ConstDecls, func(i, j int) bool { return unit.ConstDecls[i].ID < unit.ConstDecls[j].ID })
	sort.Slice(unit.Functions, func(i, j int) bool { return unit.Functions[i].ID < unit.Functions[j].ID })
	return unit
}

func (l *packageLowerer) lowerFunction(declaration *ast.FuncDecl) (virFunction, bool) {
	object, _ := l.pkg.TypesInfo.Defs[declaration.Name].(*types.Func)
	if object == nil {
		l.reject(declaration, "GO_LOWER_PATTERN", "function lacks compiler identity")
		return virFunction{}, false
	}
	signature, _ := object.Type().(*types.Signature)
	if signature == nil || declaration.Body == nil {
		l.reject(declaration, "GO_LOWER_PATTERN", "function lacks typed body")
		return virFunction{}, false
	}
	id := canonicalFunctionID(l.pkg, declaration)
	function := virFunction{
		ID: id, UnitID: l.pkg.PkgPath, Name: declaration.Name.Name,
		Params: []virBinding{}, Results: []virBinding{}, Locals: []virBinding{}, Blocks: []virBlock{},
		Contracts: defaultVIRContract(l.pkg.PkgPath, id), FeaturesUsed: []string{},
		origin: originForNode(l.pkg.Fset, l.paths, declaration), loopHeaders: []string{},
		loopParameters: make(map[string]map[string]string),
	}
	lowerer := functionLowerer{
		owner: l, declaration: declaration, function: function, current: -1,
		bindings: make(map[types.Object]string), localTypes: make(map[string]virType),
		loopParameters: make(map[string]map[string]string), environment: make(map[string]virValue),
	}
	if receiver := signature.Recv(); receiver != nil {
		if !lowerer.addParameter(receiver) {
			return virFunction{}, false
		}
	}
	for index := 0; index < signature.Params().Len(); index++ {
		if !lowerer.addParameter(signature.Params().At(index)) {
			return virFunction{}, false
		}
	}
	for index := 0; index < signature.Results().Len(); index++ {
		result := signature.Results().At(index)
		typ, ok := virTypeFromGo(result.Type())
		if !ok {
			l.reject(declaration, "GO_LOWER_TYPE", "result type cannot be normalized")
			return virFunction{}, false
		}
		lowerer.function.Results = append(lowerer.function.Results, virBinding{ID: fmt.Sprintf("result%d", index), Type: typ})
		if result.Name() != "" {
			lowerer.namedResults = append(lowerer.namedResults, lowerer.addLocal(result, typ))
		}
	}
	lowerer.collectLocals(declaration.Body)
	lowerer.current = lowerer.newBlock()
	for _, localID := range lowerer.namedResults {
		value, ok := lowerer.lowerZero(lowerer.localTypes[localID], declaration.Type.Results)
		if !ok {
			return virFunction{}, false
		}
		lowerer.emitCopy(localID, value, lowerer.localTypes[localID], declaration.Type.Results)
	}
	lowerer.lowerStatements(declaration.Body.List)
	if !lowerer.terminated() {
		if len(lowerer.function.Results) != 0 {
			l.reject(declaration.Body, "GO_LOWER_PATTERN", "value-returning function falls through")
			return virFunction{}, false
		}
		lowerer.terminate(virTerminator{Kind: "Return", Values: []virValue{}, origin: sourceOrigin{Kind: "synthetic", Reason: "go.implicit_return"}})
	}
	lowerer.function.Blocks = lowerer.blocks
	lowerer.function.loopHeaders = append([]string{}, lowerer.loopHeaders...)
	lowerer.function.loopParameters = lowerer.loopParameters
	canonicalizeFunction(&lowerer.function)
	lowerer.function.FeaturesUsed = deriveFeatures(lowerer.function)
	return lowerer.function, true
}

func (l *functionLowerer) addParameter(variable *types.Var) bool {
	typ, ok := virTypeFromGo(variable.Type())
	if !ok {
		l.owner.reject(l.declaration, "GO_LOWER_TYPE", "parameter type cannot be normalized")
		return false
	}
	id := fmt.Sprintf("arg%d", len(l.function.Params))
	l.function.Params = append(l.function.Params, virBinding{ID: id, Type: typ})
	if variable.Name() != "" && variable.Name() != "_" {
		l.bindings[variable] = id
	}
	return true
}

func (l *functionLowerer) collectLocals(body *ast.BlockStmt) {
	seen := make(map[*types.Var]struct{})
	for object := range l.bindings {
		if variable, ok := object.(*types.Var); ok {
			seen[variable] = struct{}{}
		}
	}
	ast.Inspect(body, func(node ast.Node) bool {
		if _, nested := node.(*ast.FuncLit); nested {
			return false
		}
		var names []*ast.Ident
		switch value := node.(type) {
		case *ast.ValueSpec:
			names = value.Names
		case *ast.AssignStmt:
			if value.Tok == token.DEFINE {
				for _, raw := range value.Lhs {
					if name, ok := raw.(*ast.Ident); ok {
						names = append(names, name)
					}
				}
			}
		}
		for _, name := range names {
			variable, _ := l.owner.pkg.TypesInfo.Defs[name].(*types.Var)
			if variable == nil || variable.Parent() == l.owner.pkg.Types.Scope() {
				continue
			}
			if _, exists := seen[variable]; exists {
				continue
			}
			seen[variable] = struct{}{}
			typ, ok := virTypeFromGo(variable.Type())
			if !ok {
				l.owner.reject(name, "GO_LOWER_TYPE", "local type cannot be normalized")
				continue
			}
			l.addLocal(variable, typ)
		}
		return true
	})
}

func (l *functionLowerer) addLocal(variable *types.Var, typ virType) string {
	id := fmt.Sprintf("local%d", len(l.function.Locals))
	l.function.Locals = append(l.function.Locals, virBinding{ID: id, Type: typ})
	l.bindings[variable] = id
	l.localTypes[id] = typ
	return id
}

func (l *functionLowerer) lowerStatements(statements []ast.Stmt) {
	for index, statement := range statements {
		if l.terminated() {
			return
		}
		switch value := statement.(type) {
		case *ast.IfStmt:
			l.lowerIf(value, statements[index+1:])
			return
		case *ast.ForStmt:
			l.lowerFor(value, statements[index+1:])
			return
		default:
			l.lowerStatement(statement)
		}
	}
}

func (l *functionLowerer) lowerStatement(statement ast.Stmt) {
	switch value := statement.(type) {
	case *ast.AssignStmt:
		l.lowerAssignment(value)
	case *ast.DeclStmt:
		l.lowerDeclaration(value)
	case *ast.ReturnStmt:
		l.lowerReturn(value)
	case *ast.ExprStmt:
		if _, ok := l.lowerExpr(value.X, nil); !ok {
			return
		}
	case *ast.BlockStmt:
		l.lowerStatements(value.List)
	case *ast.EmptyStmt:
	default:
		l.owner.reject(statement, "GO_LOWER_PATTERN", "statement has no accepted lowering")
	}
}

func (l *functionLowerer) lowerDeclaration(statement *ast.DeclStmt) {
	declaration := statement.Decl.(*ast.GenDecl)
	spec := declaration.Specs[0].(*ast.ValueSpec)
	name := spec.Names[0]
	variable, _ := l.owner.pkg.TypesInfo.Defs[name].(*types.Var)
	target := l.bindings[variable]
	typ := l.localTypes[target]
	if len(spec.Values) == 0 {
		value, ok := l.lowerZero(typ, statement)
		if !ok {
			return
		}
		l.emitCopy(target, value, typ, statement)
		return
	}
	if shortCircuitRoot(spec.Values[0]) {
		l.lowerShortCircuitCopy(spec.Values[0], target, typ, statement)
		return
	}
	value, ok := l.lowerExpr(spec.Values[0], &typ)
	if ok {
		l.emitCopy(target, value, typ, statement)
	}
}

func (l *functionLowerer) lowerAssignment(statement *ast.AssignStmt) {
	name := statement.Lhs[0].(*ast.Ident)
	if name.Name == "_" {
		_, _ = l.lowerExpr(statement.Rhs[0], nil)
		return
	}
	var object types.Object = l.owner.pkg.TypesInfo.Uses[name]
	if statement.Tok == token.DEFINE {
		object = l.owner.pkg.TypesInfo.Defs[name]
	}
	target := l.bindings[object]
	typ := l.localTypes[target]
	if shortCircuitRoot(statement.Rhs[0]) {
		l.lowerShortCircuitCopy(statement.Rhs[0], target, typ, statement)
		return
	}
	value, ok := l.lowerExpr(statement.Rhs[0], &typ)
	if ok {
		l.emitCopy(target, value, typ, statement)
	}
}

func (l *functionLowerer) lowerReturn(statement *ast.ReturnStmt) {
	if len(statement.Results) == 1 && shortCircuitRoot(statement.Results[0]) {
		trueBlock, falseBlock := l.newBlock(), l.newBlock()
		l.lowerBooleanBranch(statement.Results[0], trueBlock, falseBlock, statement)
		valueTrue, valueFalse := true, false
		l.current = trueBlock
		l.terminate(virTerminator{Kind: "Return", Values: []virValue{{Bool: &valueTrue}}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, statement)})
		l.current = falseBlock
		l.terminate(virTerminator{Kind: "Return", Values: []virValue{{Bool: &valueFalse}}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, statement)})
		return
	}
	values := make([]virValue, 0, len(statement.Results))
	if len(statement.Results) == 0 && len(l.function.Results) > 0 {
		for index := range l.function.Results {
			if index >= len(l.function.Locals) {
				l.owner.reject(statement, "GO_SUBSET_ASSIGNMENT", "naked return is outside the Go profile")
				return
			}
			values = append(values, virValue{Var: l.function.Locals[index].ID})
		}
	} else {
		for index, expression := range statement.Results {
			var expected *virType
			if index < len(l.function.Results) {
				expected = &l.function.Results[index].Type
			}
			value, ok := l.lowerExpr(expression, expected)
			if !ok {
				l.terminate(virTerminator{Kind: "Return", Values: []virValue{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, statement)})
				return
			}
			values = append(values, value)
		}
	}
	l.terminate(virTerminator{Kind: "Return", Values: values, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, statement)})
}

func (l *functionLowerer) lowerIf(statement *ast.IfStmt, rest []ast.Stmt) {
	if statement.Init != nil {
		l.lowerStatement(statement.Init)
		if l.terminated() {
			return
		}
	}
	elseBlock, thenBlock, joinBlock := l.newBlock(), l.newBlock(), l.newBlock()
	l.lowerBooleanBranch(statement.Cond, thenBlock, elseBlock, statement)
	baseEnvironment := l.localEnvironment()
	l.current = elseBlock
	l.environment = cloneEnvironment(baseEnvironment)
	if statement.Else != nil {
		l.lowerStatements(statement.Else.(*ast.BlockStmt).List)
	}
	elseExit := l.current
	elseFallsThrough := !l.terminated()
	elseEnvironment := cloneEnvironment(l.environment)
	if elseFallsThrough {
		l.terminate(virTerminator{Kind: "Jump", Label: l.blockName(joinBlock), Args: []virValue{}, origin: sourceOrigin{Kind: "synthetic", Reason: "go.control_flow_join"}})
	}
	l.current = thenBlock
	l.environment = cloneEnvironment(baseEnvironment)
	l.lowerStatements(statement.Body.List)
	thenExit := l.current
	thenFallsThrough := !l.terminated()
	thenEnvironment := cloneEnvironment(l.environment)
	if thenFallsThrough {
		l.terminate(virTerminator{Kind: "Jump", Label: l.blockName(joinBlock), Args: []virValue{}, origin: sourceOrigin{Kind: "synthetic", Reason: "go.control_flow_join"}})
	}
	l.current = joinBlock
	l.environment = l.localEnvironment()
	if elseFallsThrough && thenFallsThrough {
		live := l.liveLocals(rest)
		for _, local := range l.function.Locals {
			if !live[local.ID] || virValuesEqual(elseEnvironment[local.ID], thenEnvironment[local.ID]) {
				continue
			}
			parameter := l.addBlockParameter(joinBlock, local.ID, local.Type)
			l.blocks[elseExit].Terminator.Args = append(l.blocks[elseExit].Terminator.Args, elseEnvironment[local.ID])
			l.blocks[thenExit].Terminator.Args = append(l.blocks[thenExit].Terminator.Args, thenEnvironment[local.ID])
			l.environment[local.ID] = virValue{Var: parameter}
		}
	}
	l.lowerStatements(rest)
}

func (l *functionLowerer) lowerFor(statement *ast.ForStmt, rest []ast.Stmt) {
	if statement.Init != nil {
		l.lowerStatement(statement.Init)
		if l.terminated() {
			return
		}
	}
	header, exit, body := l.newBlock(), l.newBlock(), l.newBlock()
	entry := l.current
	entryEnvironment := cloneEnvironment(l.environment)
	l.loopHeaders = append(l.loopHeaders, l.blockName(header))
	carried := l.assignedLocals(append(append([]ast.Stmt{}, statement.Body.List...), statement.Post))
	headerParameters := make(map[string]string)
	entryArgs := make([]virValue, 0, len(carried))
	for _, local := range l.function.Locals {
		if !carried[local.ID] || !l.localDeclaredBefore(local.ID, statement.Cond.Pos()) {
			continue
		}
		parameter := l.addBlockParameter(header, local.ID, local.Type)
		headerParameters[local.ID] = parameter
		argument := entryEnvironment[local.ID]
		if argument.Var == "" && argument.Bool == nil && argument.Int == nil && argument.Const == "" {
			argument = virValue{Var: local.ID}
		}
		entryArgs = append(entryArgs, argument)
	}
	l.loopParameters[l.blockName(header)] = headerParameters
	l.current = entry
	l.terminate(virTerminator{Kind: "Jump", Label: l.blockName(header), Args: entryArgs, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, statement)})
	l.current = header
	l.environment = l.localEnvironment()
	for local, parameter := range headerParameters {
		l.environment[local] = virValue{Var: parameter}
	}
	l.lowerBooleanBranch(statement.Cond, body, exit, statement)
	l.current = body
	l.environment = l.localEnvironment()
	l.lowerStatements(statement.Body.List)
	if !l.terminated() && statement.Post != nil {
		l.lowerStatement(statement.Post)
	}
	if !l.terminated() {
		backedgeArgs := make([]virValue, 0, len(headerParameters))
		for _, local := range l.function.Locals {
			if _, carried := headerParameters[local.ID]; carried {
				value := l.environment[local.ID]
				if value.Var == "" && value.Bool == nil && value.Int == nil && value.Const == "" {
					value = virValue{Var: local.ID}
				}
				backedgeArgs = append(backedgeArgs, value)
			}
		}
		l.terminate(virTerminator{Kind: "Jump", Label: l.blockName(header), Args: backedgeArgs, origin: sourceOrigin{Kind: "synthetic", Reason: "go.loop_backedge"}})
	}
	l.current = exit
	l.environment = l.localEnvironment()
	l.lowerStatements(rest)
}

func (l *functionLowerer) lowerBooleanBranch(expression ast.Expr, thenBlock, elseBlock int, finalOrigin ast.Node) {
	for {
		parenthesized, ok := expression.(*ast.ParenExpr)
		if !ok {
			break
		}
		expression = parenthesized.X
	}
	if binary, ok := expression.(*ast.BinaryExpr); ok {
		switch binary.Op {
		case token.LAND:
			rhs := l.newBlock()
			l.lowerBooleanBranch(binary.X, rhs, elseBlock, binary)
			l.current = rhs
			l.environment = l.localEnvironment()
			l.lowerBooleanBranch(binary.Y, thenBlock, elseBlock, finalOrigin)
			return
		case token.LOR:
			rhs := l.newBlock()
			l.lowerBooleanBranch(binary.X, thenBlock, rhs, binary)
			l.current = rhs
			l.environment = l.localEnvironment()
			l.lowerBooleanBranch(binary.Y, thenBlock, elseBlock, finalOrigin)
			return
		}
	}
	condition, ok := l.lowerExpr(expression, &virType{Kind: "bool"})
	if !ok {
		return
	}
	l.terminate(virTerminator{
		Kind: "Branch", Cond: &condition,
		ThenLabel: l.blockName(thenBlock), ThenArgs: []virValue{},
		ElseLabel: l.blockName(elseBlock), ElseArgs: []virValue{},
		origin: originForNode(l.owner.pkg.Fset, l.owner.paths, finalOrigin),
	})
}

func (l *functionLowerer) lowerShortCircuitCopy(expression ast.Expr, target string, typ virType, origin ast.Node) {
	trueBlock, falseBlock, joinBlock := l.newBlock(), l.newBlock(), l.newBlock()
	l.lowerBooleanBranch(expression, trueBlock, falseBlock, origin)
	baseEnvironment := l.localEnvironment()
	trueValue, falseValue := true, false
	l.current = falseBlock
	l.environment = cloneEnvironment(baseEnvironment)
	falseResult := l.emitCopy(target, virValue{Bool: &falseValue}, typ, origin)
	l.terminate(virTerminator{Kind: "Jump", Label: l.blockName(joinBlock), Args: []virValue{falseResult}, origin: sourceOrigin{Kind: "synthetic", Reason: "go.control_flow_join"}})
	l.current = trueBlock
	l.environment = cloneEnvironment(baseEnvironment)
	trueResult := l.emitCopy(target, virValue{Bool: &trueValue}, typ, origin)
	l.terminate(virTerminator{Kind: "Jump", Label: l.blockName(joinBlock), Args: []virValue{trueResult}, origin: sourceOrigin{Kind: "synthetic", Reason: "go.control_flow_join"}})
	l.current = joinBlock
	l.environment = l.localEnvironment()
	parameter := l.addBlockParameter(joinBlock, target, typ)
	l.environment[target] = virValue{Var: parameter}
}

func (l *functionLowerer) lowerExpr(expression ast.Expr, expected *virType) (virValue, bool) {
	if parenthesized, ok := expression.(*ast.ParenExpr); ok {
		return l.lowerExpr(parenthesized.X, expected)
	}
	if identifier, ok := expression.(*ast.Ident); ok {
		object := l.owner.pkg.TypesInfo.Uses[identifier]
		if constantValue, ok := object.(*types.Const); ok && constantValue.Pkg() != nil {
			return virValue{Const: constantValue.Pkg().Path() + "." + constantValue.Name()}, true
		}
	}
	if typeValue := l.owner.pkg.TypesInfo.Types[expression]; typeValue.Value != nil {
		typ, ok := virTypeFromGo(typeValue.Type)
		if !ok && expected != nil {
			typ, ok = *expected, true
		}
		if !ok {
			l.owner.reject(expression, "GO_LOWER_UNTYPED_INTEGER", "untyped integer has no accepted context")
			return virValue{}, false
		}
		literal, err := literalFromConstant(typeValue.Value, typ)
		if err != nil {
			l.owner.reject(expression, "GO_LOWER_CONSTANT", "constant expression cannot be normalized")
			return virValue{}, false
		}
		return valueFromLiteral(literal), true
	}
	switch value := expression.(type) {
	case *ast.Ident:
		object := l.owner.pkg.TypesInfo.Uses[value]
		if constantValue, ok := object.(*types.Const); ok {
			return virValue{Const: constantValue.Pkg().Path() + "." + constantValue.Name()}, true
		}
		id := l.bindings[object]
		if id == "" {
			l.owner.reject(value, "GO_LOWER_PATTERN", "identifier has no normalized binding")
			return virValue{}, false
		}
		if current, exists := l.environment[id]; exists {
			return current, true
		}
		return virValue{Var: id}, true
	case *ast.BinaryExpr:
		if value.Op == token.LAND || value.Op == token.LOR {
			l.owner.reject(value, "GO_LOWER_PATTERN", "short-circuit expression appears in a nested value context")
			return virValue{}, false
		}
		return l.lowerBinary(value)
	case *ast.UnaryExpr:
		return l.lowerUnary(value)
	case *ast.CallExpr:
		return l.lowerCall(value)
	case *ast.SelectorExpr:
		return l.lowerField(value)
	case *ast.IndexExpr:
		return l.lowerIndex(value)
	case *ast.CompositeLit:
		return l.lowerComposite(value)
	default:
		l.owner.reject(value, "GO_LOWER_PATTERN", "expression has no accepted lowering")
		return virValue{}, false
	}
}

func (l *functionLowerer) lowerBinary(expression *ast.BinaryExpr) (virValue, bool) {
	lhs, ok := l.lowerExpr(expression.X, nil)
	if !ok {
		return virValue{}, false
	}
	rhsType := l.owner.pkg.TypesInfo.TypeOf(expression.Y)
	var rhsExpected *virType
	if typ, ok := virTypeFromGo(rhsType); ok {
		rhsExpected = &typ
	}
	if expression.Op == token.SHL || expression.Op == token.SHR {
		if basic, ok := rhsType.Underlying().(*types.Basic); ok && (basic.Kind() == types.UntypedInt || basic.Kind() == types.UntypedRune) {
			typ := virType{Kind: "bv", Width: 64, Signed: boolPointer(false)}
			rhsExpected = &typ
		}
	}
	rhs, ok := l.lowerExpr(expression.Y, rhsExpected)
	if !ok {
		return virValue{}, false
	}
	op, checks, ok := l.binaryOperator(expression)
	if !ok {
		return virValue{}, false
	}
	typ, ok := virTypeFromGo(l.owner.pkg.TypesInfo.TypeOf(expression))
	if !ok {
		l.owner.reject(expression, "GO_LOWER_TYPE", "operation result type cannot be normalized")
		return virValue{}, false
	}
	instruction := virInstruction{Kind: "BinOp", Op: op, Type: typ, LHS: &lhs, RHS: &rhs, SafetyChecks: checks, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}
	return l.emitValue(instruction), true
}

func (l *functionLowerer) binaryOperator(expression *ast.BinaryExpr) (string, []virSafetyCheck, bool) {
	checks := []virSafetyCheck{}
	if expression.Op == token.EQL {
		return "eq", checks, true
	}
	if expression.Op == token.NEQ {
		return "not_eq", checks, true
	}
	_, signed, ok := fixedInteger(l.owner.pkg.TypesInfo.TypeOf(expression.X))
	if !ok {
		l.owner.reject(expression, "GO_LOWER_TYPE", "operator requires fixed-width operands")
		return "", nil, false
	}
	switch expression.Op {
	case token.ADD:
		return "bv_add", checks, true
	case token.SUB:
		return "bv_sub", checks, true
	case token.MUL:
		return "bv_mul", checks, true
	case token.QUO:
		checks = append(checks, virSafetyCheck{Kind: "divisor_nonzero"})
		if signed {
			return "bv_sdiv", checks, true
		}
		return "bv_udiv", checks, true
	case token.REM:
		checks = append(checks, virSafetyCheck{Kind: "divisor_nonzero"})
		if signed {
			return "bv_srem", checks, true
		}
		return "bv_urem", checks, true
	case token.AND:
		return "bv_and", checks, true
	case token.OR:
		return "bv_or", checks, true
	case token.XOR:
		return "bv_xor", checks, true
	case token.SHL, token.SHR:
		if _, rhsSigned, rhsOK := fixedInteger(l.owner.pkg.TypesInfo.TypeOf(expression.Y)); rhsOK && rhsSigned {
			checks = append(checks, virSafetyCheck{Kind: "shift_count_nonnegative"})
		}
		if expression.Op == token.SHL {
			return "bv_shl", checks, true
		}
		if signed {
			return "bv_ashr", checks, true
		}
		return "bv_lshr", checks, true
	case token.LSS:
		if signed {
			return "signed_lt", checks, true
		}
		return "unsigned_lt", checks, true
	case token.LEQ:
		if signed {
			return "signed_le", checks, true
		}
		return "unsigned_le", checks, true
	case token.GTR:
		if signed {
			return "signed_gt", checks, true
		}
		return "unsigned_gt", checks, true
	case token.GEQ:
		if signed {
			return "signed_ge", checks, true
		}
		return "unsigned_ge", checks, true
	default:
		l.owner.reject(expression, "GO_SUBSET_SYNTAX", "binary operator is outside the Go profile")
		return "", nil, false
	}
}

func (l *functionLowerer) lowerUnary(expression *ast.UnaryExpr) (virValue, bool) {
	value, ok := l.lowerExpr(expression.X, nil)
	if !ok {
		return virValue{}, false
	}
	if expression.Op == token.ADD {
		return value, true
	}
	op := ""
	switch expression.Op {
	case token.NOT:
		op = "not"
	case token.SUB:
		op = "bv_neg"
	case token.XOR:
		op = "bv_not"
	}
	if op == "" {
		l.owner.reject(expression, "GO_SUBSET_SYNTAX", "unary operator is outside the Go profile")
		return virValue{}, false
	}
	typ, ok := virTypeFromGo(l.owner.pkg.TypesInfo.TypeOf(expression))
	if !ok {
		return virValue{}, false
	}
	return l.emitValue(virInstruction{Kind: "UnaryOp", Op: op, Type: typ, Value: &value, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
}

func (l *functionLowerer) lowerCall(expression *ast.CallExpr) (virValue, bool) {
	if identifier, ok := expression.Fun.(*ast.Ident); ok {
		if typeName, ok := l.owner.pkg.TypesInfo.Uses[identifier].(*types.TypeName); ok {
			if len(expression.Args) != 1 {
				l.owner.reject(expression, "GO_LOWER_PATTERN", "conversion requires one argument")
				return virValue{}, false
			}
			target, ok := virTypeFromGo(typeName.Type())
			if !ok || target.Kind != "bv" {
				l.owner.reject(expression, "GO_LOWER_TYPE", "conversion requires fixed-width bitvectors")
				return virValue{}, false
			}
			source, sourceOK := virTypeFromGo(l.owner.pkg.TypesInfo.TypeOf(expression.Args[0]))
			if !sourceOK || source.Kind != "bv" {
				l.owner.reject(expression, "GO_LOWER_TYPE", "conversion requires fixed-width bitvectors")
				return virValue{}, false
			}
			input, ok := l.lowerExpr(expression.Args[0], nil)
			if !ok {
				return virValue{}, false
			}
			return l.emitValue(virInstruction{Kind: "Convert", Type: target, Value: &input, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
		}
	}
	function, ok := resolvedStaticFunction(l.owner.pkg, expression.Fun)
	if !ok {
		return virValue{}, false
	}
	signature, _ := function.Type().(*types.Signature)
	if signature == nil || signature.Results().Len() != 1 {
		l.owner.reject(expression, "GO_SUBSET_CALL", "static callees must have exactly one result")
		return virValue{}, false
	}
	functionID := canonicalTypesFunctionID(function)
	if function.Pkg() == nil || function.Pkg().Path() != l.owner.pkg.PkgPath {
		for index := 0; index < signature.Params().Len(); index++ {
			if containsStructType(signature.Params().At(index).Type()) {
				l.owner.reject(expression, "GO_SUBSET_IMPORT", "cross-unit calls cannot expose struct types")
				return virValue{}, false
			}
		}
		if containsStructType(signature.Results().At(0).Type()) {
			l.owner.reject(expression, "GO_SUBSET_IMPORT", "cross-unit calls cannot expose struct types")
			return virValue{}, false
		}
	}
	args := make([]virValue, 0, len(expression.Args)+1)
	if selector, ok := expression.Fun.(*ast.SelectorExpr); ok {
		if selection := l.owner.pkg.TypesInfo.Selections[selector]; selection != nil && selection.Kind() == types.MethodVal {
			receiver, ok := l.lowerExpr(selector.X, nil)
			if !ok {
				return virValue{}, false
			}
			args = append(args, receiver)
		}
	}
	for _, argument := range expression.Args {
		value, ok := l.lowerExpr(argument, nil)
		if !ok {
			return virValue{}, false
		}
		args = append(args, value)
	}
	typ, ok := virTypeFromGo(signature.Results().At(0).Type())
	if !ok {
		return virValue{}, false
	}
	return l.emitValue(virInstruction{Kind: "CallStatic", Type: typ, Function: functionID, ContractHash: zeroSHA256(), Args: args, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
}

func (l *functionLowerer) lowerField(expression *ast.SelectorExpr) (virValue, bool) {
	selection := l.owner.pkg.TypesInfo.Selections[expression]
	if selection == nil || selection.Kind() != types.FieldVal || len(selection.Index()) != 1 {
		l.owner.reject(expression, "GO_SUBSET_AGGREGATE", "field selection is not direct")
		return virValue{}, false
	}
	base, ok := l.lowerExpr(expression.X, nil)
	if !ok {
		return virValue{}, false
	}
	typ, ok := virTypeFromGo(l.owner.pkg.TypesInfo.TypeOf(expression))
	if !ok {
		return virValue{}, false
	}
	return l.emitValue(virInstruction{Kind: "Field", Type: typ, Base: &base, Field: expression.Sel.Name, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
}

func (l *functionLowerer) lowerIndex(expression *ast.IndexExpr) (virValue, bool) {
	base, ok := l.lowerExpr(expression.X, nil)
	if !ok {
		return virValue{}, false
	}
	indexType, typed := virTypeFromGo(l.owner.pkg.TypesInfo.TypeOf(expression.Index))
	if !typed {
		indexType = virType{Kind: "bv", Width: 64, Signed: boolPointer(true)}
	}
	index, ok := l.lowerExpr(expression.Index, &indexType)
	if !ok {
		return virValue{}, false
	}
	typ, ok := virTypeFromGo(l.owner.pkg.TypesInfo.TypeOf(expression))
	if !ok {
		return virValue{}, false
	}
	return l.emitValue(virInstruction{Kind: "Index", Type: typ, Base: &base, Index: &index, SafetyChecks: []virSafetyCheck{{Kind: "index_in_bounds"}}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
}

func (l *functionLowerer) lowerComposite(expression *ast.CompositeLit) (virValue, bool) {
	typ := l.owner.pkg.TypesInfo.TypeOf(expression)
	virTyp, ok := virTypeFromGo(typ)
	if !ok {
		return virValue{}, false
	}
	switch underlying := typ.Underlying().(type) {
	case *types.Array:
		if int64(len(expression.Elts)) != underlying.Len() {
			l.owner.reject(expression, "GO_SUBSET_AGGREGATE", "array literal must be complete")
			return virValue{}, false
		}
		elements := make([]virValue, 0, len(expression.Elts))
		for _, element := range expression.Elts {
			if _, keyed := element.(*ast.KeyValueExpr); keyed {
				l.owner.reject(element, "GO_SUBSET_AGGREGATE", "keyed array literal is outside the profile")
				return virValue{}, false
			}
			value, ok := l.lowerExpr(element, virTyp.Element)
			if !ok {
				return virValue{}, false
			}
			elements = append(elements, value)
		}
		return l.emitValue(virInstruction{Kind: "MakeArray", Type: virTyp, Elements: elements, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
	case *types.Struct:
		fields := make([]virNamedValue, underlying.NumFields())
		seen := make([]bool, underlying.NumFields())
		keyed, positional := false, false
		for index, raw := range expression.Elts {
			fieldIndex, valueExpression := index, raw
			if pair, ok := raw.(*ast.KeyValueExpr); ok {
				keyed = true
				valueExpression = pair.Value
				name, ok := pair.Key.(*ast.Ident)
				if !ok {
					return virValue{}, false
				}
				fieldIndex = structField(underlying, name.Name)
			} else {
				positional = true
			}
			if fieldIndex < 0 || fieldIndex >= underlying.NumFields() || seen[fieldIndex] {
				l.owner.reject(raw, "GO_SUBSET_AGGREGATE", "struct literal fields are invalid")
				return virValue{}, false
			}
			fieldType, _ := virTypeFromGo(underlying.Field(fieldIndex).Type())
			fieldValue, ok := l.lowerExpr(valueExpression.(ast.Expr), &fieldType)
			if !ok {
				return virValue{}, false
			}
			fields[fieldIndex] = virNamedValue{Name: underlying.Field(fieldIndex).Name(), Value: fieldValue}
			seen[fieldIndex] = true
		}
		if keyed && positional {
			l.owner.reject(expression, "GO_SUBSET_AGGREGATE", "mixed struct literal is outside the profile")
			return virValue{}, false
		}
		for _, present := range seen {
			if !present {
				l.owner.reject(expression, "GO_SUBSET_AGGREGATE", "struct literal must supply every field")
				return virValue{}, false
			}
		}
		return l.emitValue(virInstruction{Kind: "MakeStruct", Type: virTyp, Fields: fields, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, expression)}), true
	default:
		return virValue{}, false
	}
}

func (l *functionLowerer) lowerZero(typ virType, origin ast.Node) (virValue, bool) {
	if value, ok := zeroLiteral(typ); ok {
		return value, true
	}
	switch typ.Kind {
	case "array":
		elements := make([]virValue, typ.Length)
		for index := range elements {
			value, ok := l.lowerZero(*typ.Element, origin)
			if !ok {
				return virValue{}, false
			}
			elements[index] = value
		}
		return l.emitValue(virInstruction{Kind: "MakeArray", Type: typ, Elements: elements, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, origin)}), true
	case "struct":
		if structType := l.owner.structs[typ.ID]; structType != nil {
			fields := make([]virNamedValue, structType.NumFields())
			for index := range fields {
				fieldType, _ := virTypeFromGo(structType.Field(index).Type())
				value, ok := l.lowerZero(fieldType, origin)
				if !ok {
					return virValue{}, false
				}
				fields[index] = virNamedValue{Name: structType.Field(index).Name(), Value: value}
			}
			return l.emitValue(virInstruction{Kind: "MakeStruct", Type: typ, Fields: fields, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, origin)}), true
		}
	}
	l.owner.reject(origin, "GO_LOWER_TYPE", "zero value cannot be materialized")
	return virValue{}, false
}

func loadedStructTypes(loaded packageLoadResult) map[string]*types.Struct {
	result := make(map[string]*types.Struct)
	for _, loadedPackage := range loaded.Packages {
		pkg := loadedPackage.packageValue
		if pkg == nil || pkg.Types == nil {
			continue
		}
		for _, name := range pkg.Types.Scope().Names() {
			typeName, ok := pkg.Types.Scope().Lookup(name).(*types.TypeName)
			if !ok {
				continue
			}
			named, ok := typeName.Type().(*types.Named)
			if !ok {
				continue
			}
			if structure, ok := named.Underlying().(*types.Struct); ok {
				result[pkg.PkgPath+"."+name] = structure
			}
		}
	}
	return result
}

func (l *functionLowerer) emitCopy(target string, value virValue, typ virType, node ast.Node) virValue {
	result := l.emitValue(virInstruction{Kind: "Copy", Type: typ, Target: target, Value: &value, SafetyChecks: []virSafetyCheck{}, origin: originForNode(l.owner.pkg.Fset, l.owner.paths, node)})
	l.environment[target] = result
	return result
}

func (l *functionLowerer) emitValue(instruction virInstruction) virValue {
	index := len(l.blocks[l.current].Instructions)
	instruction.ID = fmt.Sprintf("u%d_%d", l.current, index)
	l.blocks[l.current].Instructions = append(l.blocks[l.current].Instructions, instruction)
	return virValue{Var: instruction.ID}
}

func (l *functionLowerer) newBlock() int {
	index := len(l.blocks)
	l.blocks = append(l.blocks, virBlock{Label: fmt.Sprintf("n%d", index), Parameters: []virBinding{}, Instructions: []virInstruction{}})
	return index
}

func (l *functionLowerer) addBlockParameter(block int, local string, typ virType) string {
	id := fmt.Sprintf("q%d_%s", block, local)
	l.blocks[block].Parameters = append(l.blocks[block].Parameters, virBinding{ID: id, Type: typ})
	return id
}

func (l *functionLowerer) localEnvironment() map[string]virValue {
	values := make(map[string]virValue, len(l.function.Locals))
	for _, local := range l.function.Locals {
		values[local.ID] = virValue{Var: local.ID}
	}
	return values
}

func cloneEnvironment(source map[string]virValue) map[string]virValue {
	values := make(map[string]virValue, len(source))
	for key, value := range source {
		values[key] = value
	}
	return values
}

func virValuesEqual(left, right virValue) bool {
	if left.Var != right.Var || left.Const != right.Const || (left.Bool == nil) != (right.Bool == nil) || (left.Int == nil) != (right.Int == nil) {
		return false
	}
	if left.Bool != nil && *left.Bool != *right.Bool {
		return false
	}
	return left.Int == nil || *left.Int == *right.Int
}

func (l *functionLowerer) liveLocals(statements []ast.Stmt) map[string]bool {
	result := make(map[string]bool)
	for _, statement := range statements {
		ast.Inspect(statement, func(node ast.Node) bool {
			identifier, ok := node.(*ast.Ident)
			if !ok {
				return true
			}
			if id := l.bindings[l.owner.pkg.TypesInfo.Uses[identifier]]; strings.HasPrefix(id, "local") {
				result[id] = true
			}
			return true
		})
	}
	return result
}

func (l *functionLowerer) assignedLocals(statements []ast.Stmt) map[string]bool {
	result := make(map[string]bool)
	for _, statement := range statements {
		if statement == nil {
			continue
		}
		ast.Inspect(statement, func(node ast.Node) bool {
			assignment, ok := node.(*ast.AssignStmt)
			if !ok {
				return true
			}
			for _, raw := range assignment.Lhs {
				identifier, ok := raw.(*ast.Ident)
				if !ok {
					continue
				}
				object := l.owner.pkg.TypesInfo.Uses[identifier]
				if assignment.Tok == token.DEFINE {
					object = l.owner.pkg.TypesInfo.Defs[identifier]
				}
				if id := l.bindings[object]; strings.HasPrefix(id, "local") {
					result[id] = true
				}
			}
			return true
		})
	}
	return result
}

func (l *functionLowerer) localDeclaredBefore(localID string, position token.Pos) bool {
	for object, id := range l.bindings {
		if id == localID {
			return object.Pos().IsValid() && object.Pos() < position
		}
	}
	return false
}

func (l *functionLowerer) blockName(index int) string { return l.blocks[index].Label }
func (l *functionLowerer) terminated() bool {
	return l.current >= 0 && l.blocks[l.current].Terminator.Kind != ""
}
func (l *functionLowerer) terminate(value virTerminator) {
	if l.current >= 0 {
		l.blocks[l.current].Terminator = value
	}
}

func (l *packageLowerer) reject(node ast.Node, code, message string) {
	l.findings = append(l.findings, loweringFinding{Code: code, Message: message, FunctionID: enclosingFunctionID(l.pkg, node), Origin: originForNode(l.pkg.Fset, l.paths, node)})
}

func literalFromConstant(value constant.Value, typ virType) (virLiteral, error) {
	switch value.Kind() {
	case constant.Bool:
		boolean := constant.BoolVal(value)
		return virLiteral{Bool: &boolean}, nil
	case constant.Int:
		integer, err := integerFromConstant(value, typ)
		if err != nil {
			return virLiteral{}, err
		}
		return virLiteral{Int: &integer}, nil
	default:
		return virLiteral{}, fmt.Errorf("constant kind is outside the profile")
	}
}

func valueFromLiteral(value virLiteral) virValue { return virValue{Bool: value.Bool, Int: value.Int} }
func shortCircuitRoot(expression ast.Expr) bool {
	for {
		parenthesized, ok := expression.(*ast.ParenExpr)
		if !ok {
			break
		}
		expression = parenthesized.X
	}
	binary, ok := expression.(*ast.BinaryExpr)
	return ok && (binary.Op == token.LAND || binary.Op == token.LOR)
}
func structField(value *types.Struct, name string) int {
	for index := 0; index < value.NumFields(); index++ {
		if value.Field(index).Name() == name {
			return index
		}
	}
	return -1
}

func containsStructType(typ types.Type) bool {
	if typ == nil {
		return false
	}
	switch value := typ.Underlying().(type) {
	case *types.Struct:
		return true
	case *types.Array:
		return containsStructType(value.Elem())
	default:
		return false
	}
}

func resolvedStaticFunction(pkg *packages.Package, expression ast.Expr) (*types.Func, bool) {
	switch value := expression.(type) {
	case *ast.Ident:
		function, ok := pkg.TypesInfo.Uses[value].(*types.Func)
		return function, ok
	case *ast.SelectorExpr:
		if selection := pkg.TypesInfo.Selections[value]; selection != nil {
			function, ok := selection.Obj().(*types.Func)
			return function, ok
		}
		function, ok := pkg.TypesInfo.Uses[value.Sel].(*types.Func)
		return function, ok
	default:
		return nil, false
	}
}

func canonicalFunctionID(pkg *packages.Package, declaration *ast.FuncDecl) string {
	function, _ := pkg.TypesInfo.Defs[declaration.Name].(*types.Func)
	if function == nil {
		return pkg.PkgPath + "." + declaration.Name.Name
	}
	return canonicalTypesFunctionID(function)
}

func canonicalTypesFunctionID(function *types.Func) string {
	if function == nil || function.Pkg() == nil {
		return ""
	}
	signature, _ := function.Type().(*types.Signature)
	if signature != nil && signature.Recv() != nil {
		receiver := signature.Recv().Type()
		if pointer, ok := receiver.(*types.Pointer); ok {
			receiver = pointer.Elem()
		}
		if named, ok := receiver.(*types.Named); ok && named.Obj() != nil {
			return function.Pkg().Path() + "." + named.Obj().Name() + "." + function.Name()
		}
	}
	return function.Pkg().Path() + "." + function.Name()
}

func enclosingFunctionID(pkg *packages.Package, node ast.Node) string {
	if node == nil {
		return ""
	}
	for _, file := range pkg.Syntax {
		for _, declaration := range file.Decls {
			if function, ok := declaration.(*ast.FuncDecl); ok && node.Pos() >= function.Pos() && node.End() <= function.End() {
				return canonicalFunctionID(pkg, function)
			}
		}
	}
	return ""
}

func canonicalizeFunction(function *virFunction) {
	byLabel := make(map[string]int, len(function.Blocks))
	for index := range function.Blocks {
		byLabel[function.Blocks[index].Label] = index
	}
	order, seen, queue := make([]int, 0, len(function.Blocks)), make(map[int]bool), []int{0}
	seen[0] = true
	for len(queue) > 0 {
		index := queue[0]
		queue = queue[1:]
		order = append(order, index)
		term := function.Blocks[index].Terminator
		successors := []string{}
		switch term.Kind {
		case "Jump":
			successors = append(successors, term.Label)
		case "Branch":
			successors = append(successors, term.ElseLabel, term.ThenLabel)
		}
		for _, label := range successors {
			target, ok := byLabel[label]
			if ok && !seen[target] {
				seen[target] = true
				queue = append(queue, target)
			}
		}
	}
	renames := make(map[string]string, len(order))
	for index, old := range order {
		renames[function.Blocks[old].Label] = fmt.Sprintf("bb%d", index)
	}
	blocks := make([]virBlock, 0, len(order))
	temporaryRename := make(map[string]string)
	tempIndex, parameterIndex := 0, 0
	for _, old := range order {
		block := function.Blocks[old]
		block.Label = renames[block.Label]
		for index := range block.Parameters {
			oldID := block.Parameters[index].ID
			block.Parameters[index].ID = fmt.Sprintf("p%d", parameterIndex)
			temporaryRename[oldID] = block.Parameters[index].ID
			parameterIndex++
		}
		for index := range block.Instructions {
			oldID := block.Instructions[index].ID
			newID := fmt.Sprintf("t%d", tempIndex)
			tempIndex++
			block.Instructions[index].ID = newID
			temporaryRename[oldID] = newID
		}
		blocks = append(blocks, block)
	}
	function.Blocks = blocks
	for blockIndex := range function.Blocks {
		block := &function.Blocks[blockIndex]
		for instructionIndex := range block.Instructions {
			renameInstructionValues(&block.Instructions[instructionIndex], temporaryRename)
		}
		renameTerminator(&block.Terminator, renames, temporaryRename)
	}
	for index, header := range function.loopHeaders {
		function.loopHeaders[index] = renames[header]
	}
	canonicalLoopParameters := make(map[string]map[string]string, len(function.loopParameters))
	for header, parameters := range function.loopParameters {
		canonical := make(map[string]string, len(parameters))
		for local, parameter := range parameters {
			canonical[local] = temporaryRename[parameter]
		}
		canonicalLoopParameters[renames[header]] = canonical
	}
	function.loopParameters = canonicalLoopParameters
}

func renameInstructionValues(instruction *virInstruction, names map[string]string) {
	values := []*virValue{instruction.Value, instruction.Base, instruction.Index, instruction.LHS, instruction.RHS}
	for _, value := range values {
		renameValue(value, names)
	}
	for index := range instruction.Args {
		renameValue(&instruction.Args[index], names)
	}
	for index := range instruction.Elements {
		renameValue(&instruction.Elements[index], names)
	}
	for index := range instruction.Fields {
		renameValue(&instruction.Fields[index].Value, names)
	}
}
func renameTerminator(term *virTerminator, blocks, values map[string]string) {
	if replacement := blocks[term.Label]; replacement != "" {
		term.Label = replacement
	}
	if replacement := blocks[term.ThenLabel]; replacement != "" {
		term.ThenLabel = replacement
	}
	if replacement := blocks[term.ElseLabel]; replacement != "" {
		term.ElseLabel = replacement
	}
	if term.Cond != nil {
		renameValue(term.Cond, values)
	}
	for index := range term.Values {
		renameValue(&term.Values[index], values)
	}
	for index := range term.Args {
		renameValue(&term.Args[index], values)
	}
	for index := range term.ThenArgs {
		renameValue(&term.ThenArgs[index], values)
	}
	for index := range term.ElseArgs {
		renameValue(&term.ElseArgs[index], values)
	}
}
func renameValue(value *virValue, names map[string]string) {
	if value != nil {
		if replacement := names[value.Var]; replacement != "" {
			value.Var = replacement
		}
	}
}

func deriveFeatures(function virFunction) []string {
	set := make(map[string]struct{})
	collectType := func(typ virType) {
		switch typ.Kind {
		case "array":
			set["array"] = struct{}{}
		case "struct":
			set["struct"] = struct{}{}
		}
	}
	collectValue := func(value virValue) {
		if value.Const != "" {
			set["constant_decl"] = struct{}{}
		}
	}
	for _, binding := range append(append(append([]virBinding{}, function.Params...), function.Results...), function.Locals...) {
		collectType(binding.Type)
	}
	if len(function.Locals) > 0 {
		set["mutable_local"] = struct{}{}
	}
	if len(function.loopHeaders) > 0 {
		set["cyclic_cfg"] = struct{}{}
	}
	for _, block := range function.Blocks {
		if block.Terminator.Kind == "Branch" {
			set["branch"] = struct{}{}
		}
		for _, value := range block.Terminator.Values {
			collectValue(value)
		}
		for _, value := range block.Terminator.Args {
			collectValue(value)
		}
		for _, value := range block.Terminator.ThenArgs {
			collectValue(value)
		}
		for _, value := range block.Terminator.ElseArgs {
			collectValue(value)
		}
		if block.Terminator.Cond != nil {
			collectValue(*block.Terminator.Cond)
		}
		for _, instruction := range block.Instructions {
			collectType(instruction.Type)
			for _, value := range instructionValues(instruction) {
				collectValue(value)
			}
			switch instruction.Kind {
			case "Convert":
				set["conversion"] = struct{}{}
			case "MakeArray", "Index":
				set["array"] = struct{}{}
			case "MakeStruct", "Field":
				set["struct"] = struct{}{}
			case "CallStatic":
				set["call_static"] = struct{}{}
			}
		}
	}
	features := make([]string, 0, len(set))
	for feature := range set {
		features = append(features, feature)
	}
	sort.Strings(features)
	return features
}

func instructionValues(instruction virInstruction) []virValue {
	values := make([]virValue, 0)
	for _, value := range []*virValue{instruction.Value, instruction.Base, instruction.Index, instruction.LHS, instruction.RHS} {
		if value != nil {
			values = append(values, *value)
		}
	}
	values = append(values, instruction.Args...)
	values = append(values, instruction.Elements...)
	for _, field := range instruction.Fields {
		values = append(values, field.Value)
	}
	return values
}

func canonicalizeFunctionOrder(module *virModule, calls map[string][]string) *loweringFinding {
	functions := make(map[string]*virFunction)
	unitFor := make(map[string]int)
	for unitIndex := range module.Units {
		for functionIndex := range module.Units[unitIndex].Functions {
			function := &module.Units[unitIndex].Functions[functionIndex]
			functions[function.ID] = function
			unitFor[function.ID] = unitIndex
		}
	}
	state := make(map[string]int)
	orderByUnit := make(map[int][]string)
	var visit func(string) *loweringFinding
	visit = func(id string) *loweringFinding {
		if state[id] == 1 {
			return &loweringFinding{Code: "GO_SUBSET_CALL", Message: "recursive static calls are outside the Go profile", FunctionID: id}
		}
		if state[id] == 2 {
			return nil
		}
		if functions[id] == nil {
			return &loweringFinding{Code: "GO_SUBSET_CALL", Message: "static callee is outside the captured closure", FunctionID: id}
		}
		state[id] = 1
		callees := append([]string{}, calls[id]...)
		sort.Strings(callees)
		for _, callee := range callees {
			if finding := visit(callee); finding != nil {
				return finding
			}
		}
		state[id] = 2
		orderByUnit[unitFor[id]] = append(orderByUnit[unitFor[id]], id)
		return nil
	}
	ids := make([]string, 0, len(functions))
	for id := range functions {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	for _, id := range ids {
		if finding := visit(id); finding != nil {
			return finding
		}
	}
	for unitIndex := range module.Units {
		ordered := make([]virFunction, 0, len(orderByUnit[unitIndex]))
		for _, id := range orderByUnit[unitIndex] {
			ordered = append(ordered, *functions[id])
		}
		module.Units[unitIndex].Functions = ordered
	}
	sort.Slice(module.Units, func(i, j int) bool { return module.Units[i].ID < module.Units[j].ID })
	return nil
}

func sortLoweringFindings(findings []loweringFinding) {
	sort.Slice(findings, func(i, j int) bool {
		left, right := findings[i], findings[j]
		if left.Origin.NormalizedPath != right.Origin.NormalizedPath {
			return left.Origin.NormalizedPath < right.Origin.NormalizedPath
		}
		if left.Origin.Start != right.Origin.Start {
			return left.Origin.Start < right.Origin.Start
		}
		if left.Code != right.Code {
			return left.Code < right.Code
		}
		if left.Message != right.Message {
			return left.Message < right.Message
		}
		return strings.Compare(left.FunctionID, right.FunctionID) < 0
	})
}
