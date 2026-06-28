package main

import (
	"fmt"
	"go/ast"
	"go/constant"
	"go/token"
	"go/types"
	"sort"
	"strconv"

	"golang.org/x/tools/go/packages"
)

const girSchemaVersion = "mpk.gir.v0"

type girModule struct {
	SchemaVersion string       `json:"schema_version"`
	Packages      []girPackage `json:"packages"`
	GIRHash       string       `json:"gir_hash,omitempty"`
}

type girPackage struct {
	PackagePath string        `json:"package_path"`
	Name        string        `json:"name"`
	Functions   []girFunction `json:"functions"`
}

type girFunction struct {
	ID                string            `json:"id"`
	Package           string            `json:"package"`
	Name              string            `json:"name"`
	Params            []girBinding      `json:"params"`
	Results           []girBinding      `json:"results"`
	Locals            []girBinding      `json:"locals"`
	Blocks            []girBlock        `json:"blocks"`
	Contracts         girContracts      `json:"contracts"`
	SupportedFeatures []string          `json:"supported_features"`
	RejectedFeatures  []rejectedFeature `json:"rejected_features"`
}

type girContracts struct {
	Requires []girContractExpr `json:"requires"`
	Ensures  []girContractExpr `json:"ensures"`
	Modifies []string          `json:"modifies"`
	Loops    []girLoopContract `json:"loops"`
}

type girContractExpr struct{}

type girLoopContract struct{}

type girBinding struct {
	Name string  `json:"name"`
	Type girType `json:"type"`
}

type girType struct {
	Kind    string         `json:"kind"`
	Name    string         `json:"name,omitempty"`
	Width   int            `json:"width,omitempty"`
	Signed  *bool          `json:"signed,omitempty"`
	Length  int64          `json:"length,omitempty"`
	Element *girType       `json:"element,omitempty"`
	Fields  []girFieldType `json:"fields,omitempty"`
}

type girFieldType struct {
	Name string  `json:"name"`
	Type girType `json:"type"`
}

type girBlock struct {
	Label        string           `json:"label"`
	Parameters   []girBinding     `json:"parameters"`
	Instructions []girInstruction `json:"instructions"`
	Terminator   girTerminator    `json:"terminator"`
}

type girInstruction struct {
	ID       string     `json:"id"`
	Kind     string     `json:"kind"`
	Op       string     `json:"op,omitempty"`
	Type     girType    `json:"type"`
	Target   string     `json:"target,omitempty"`
	Value    *girValue  `json:"value,omitempty"`
	Base     *girValue  `json:"base,omitempty"`
	Index    *girValue  `json:"index,omitempty"`
	Field    string     `json:"field,omitempty"`
	Fields   []girField `json:"fields,omitempty"`
	Elements []girValue `json:"elements,omitempty"`
	LHS      *girValue  `json:"lhs,omitempty"`
	RHS      *girValue  `json:"rhs,omitempty"`
	Function string     `json:"function,omitempty"`
	Args     []girValue `json:"args,omitempty"`
}

type girField struct {
	Name  string   `json:"name"`
	Value girValue `json:"value"`
}

type girTerminator struct {
	Kind      string     `json:"kind"`
	Values    []girValue `json:"values,omitempty"`
	Cond      *girValue  `json:"cond,omitempty"`
	Label     string     `json:"label,omitempty"`
	ThenLabel string     `json:"then_label,omitempty"`
	ElseLabel string     `json:"else_label,omitempty"`
	Args      []girValue `json:"args,omitempty"`
	Reason    string     `json:"reason,omitempty"`
}

type girValue struct {
	Var  string         `json:"var,omitempty"`
	Int  *girIntLiteral `json:"int,omitempty"`
	Bool *bool          `json:"bool,omitempty"`
}

type girIntLiteral struct {
	Value  string `json:"value"`
	Width  int    `json:"width"`
	Signed bool   `json:"signed"`
}

type girPackageLowerer struct {
	baseDir  string
	pkg      *packages.Package
	fset     *token.FileSet
	findings []rejectedFeature
	seen     map[rejectedFeature]struct{}
}

type girFunctionLowerer struct {
	packageLowerer *girPackageLowerer
	function       girFunction
	current        *girBlock
	tempIndex      int
	blockIndex     int
}

func lowerToGIR(loaded packageLoadResult) (girModule, []rejectedFeature) {
	packagesLowered := make([]girPackage, 0, len(loaded.Packages))
	var findings []rejectedFeature
	for _, pkg := range loaded.Packages {
		lowerer := &girPackageLowerer{
			baseDir: loaded.BaseDir,
			pkg:     pkg,
			fset:    pkg.Fset,
			seen:    make(map[rejectedFeature]struct{}),
		}
		packagesLowered = append(packagesLowered, lowerer.lowerPackage())
		findings = append(findings, lowerer.findings...)
	}

	sort.Slice(packagesLowered, func(i, j int) bool {
		if packagesLowered[i].PackagePath != packagesLowered[j].PackagePath {
			return packagesLowered[i].PackagePath < packagesLowered[j].PackagePath
		}
		return packagesLowered[i].Name < packagesLowered[j].Name
	})
	sortRejectedFeatures(findings)
	if len(findings) > 0 {
		return girModule{}, findings
	}

	return girModule{
		SchemaVersion: girSchemaVersion,
		Packages:      packagesLowered,
	}, nil
}

func (l *girPackageLowerer) lowerPackage() girPackage {
	functions := make([]girFunction, 0)
	for _, file := range l.pkg.Syntax {
		for _, decl := range file.Decls {
			funcDecl, ok := decl.(*ast.FuncDecl)
			if !ok {
				continue
			}
			function, ok := l.lowerFunction(funcDecl)
			if ok {
				functions = append(functions, function)
			}
		}
	}

	sort.Slice(functions, func(i, j int) bool {
		return functions[i].ID < functions[j].ID
	})

	return girPackage{
		PackagePath: l.pkg.PkgPath,
		Name:        l.pkg.Name,
		Functions:   functions,
	}
}

func (l *girPackageLowerer) lowerFunction(decl *ast.FuncDecl) (girFunction, bool) {
	obj, ok := l.pkg.TypesInfo.Defs[decl.Name].(*types.Func)
	if !ok {
		l.reject(decl.Name.Pos(), "GIR lowering", fmt.Sprintf("function %s has no type information", decl.Name.Name))
		return girFunction{}, false
	}
	signature, ok := obj.Type().(*types.Signature)
	if !ok {
		l.reject(decl.Name.Pos(), "GIR lowering", fmt.Sprintf("function %s has no signature", decl.Name.Name))
		return girFunction{}, false
	}
	if decl.Body == nil {
		l.reject(decl.Name.Pos(), "GIR lowering", "function declarations without bodies cannot lower to GIR")
		return girFunction{}, false
	}

	params := l.lowerParams(decl, signature)
	results := l.lowerTupleBindings(decl.Pos(), signature.Results(), "result")
	locals := l.collectLocals(decl.Body)
	allBindings := append(append([]girBinding{}, params...), results...)
	allBindings = append(allBindings, locals...)
	l.rejectDuplicateBindings(decl.Pos(), allBindings)

	function := girFunction{
		ID:                functionID(l.pkg.PkgPath, decl.Name.Name),
		Package:           l.pkg.PkgPath,
		Name:              decl.Name.Name,
		Params:            params,
		Results:           results,
		Locals:            locals,
		Contracts:         emptyGIRContracts(),
		SupportedFeatures: []string{"params", "locals", "blocks", "binops", "if", "return", "structs", "fixed_arrays", "field", "index"},
		RejectedFeatures:  []rejectedFeature{},
	}

	functionLowerer := &girFunctionLowerer{
		packageLowerer: l,
		function:       function,
	}
	functionLowerer.current = functionLowerer.newBlock("entry")
	functionLowerer.lowerStatements(decl.Body.List)
	if !functionLowerer.currentTerminated() {
		if signature.Results().Len() == 0 {
			functionLowerer.terminate(girTerminator{Kind: "Return"})
		} else {
			l.reject(decl.Body.End(), "GIR lowering", fmt.Sprintf("function %s can fall through without return", decl.Name.Name))
		}
	}

	return functionLowerer.function, true
}

func (l *girPackageLowerer) lowerParams(decl *ast.FuncDecl, signature *types.Signature) []girBinding {
	params := make([]girBinding, 0, signature.Params().Len()+1)
	if signature.Recv() != nil {
		name := signature.Recv().Name()
		if name == "" {
			name = "receiver"
		}
		if binding, ok := l.lowerBinding(signature.Recv().Pos(), name, signature.Recv().Type()); ok {
			params = append(params, binding)
		}
	}

	for i := 0; i < signature.Params().Len(); i++ {
		param := signature.Params().At(i)
		name := param.Name()
		if name == "" {
			name = fmt.Sprintf("param%d", i)
			if decl.Type.Params != nil && i < len(decl.Type.Params.List) && len(decl.Type.Params.List[i].Names) > 0 {
				name = decl.Type.Params.List[i].Names[0].Name
			}
		}
		if binding, ok := l.lowerBinding(param.Pos(), name, param.Type()); ok {
			params = append(params, binding)
		}
	}
	return params
}

func (l *girPackageLowerer) collectLocals(body *ast.BlockStmt) []girBinding {
	locals := make([]girBinding, 0)
	seen := make(map[*types.Var]struct{})

	ast.Inspect(body, func(node ast.Node) bool {
		switch node := node.(type) {
		case *ast.FuncLit:
			return false
		case *ast.ValueSpec:
			for _, name := range node.Names {
				if variable, ok := l.pkg.TypesInfo.Defs[name].(*types.Var); ok && variable.Parent() != l.pkg.Types.Scope() {
					if _, ok := seen[variable]; !ok {
						seen[variable] = struct{}{}
						if binding, ok := l.lowerBinding(name.Pos(), variable.Name(), variable.Type()); ok {
							locals = append(locals, binding)
						}
					}
				}
			}
		case *ast.AssignStmt:
			if node.Tok != token.DEFINE {
				return true
			}
			for _, lhs := range node.Lhs {
				name, ok := lhs.(*ast.Ident)
				if !ok {
					continue
				}
				if variable, ok := l.pkg.TypesInfo.Defs[name].(*types.Var); ok {
					if _, ok := seen[variable]; !ok {
						seen[variable] = struct{}{}
						if binding, ok := l.lowerBinding(name.Pos(), variable.Name(), variable.Type()); ok {
							locals = append(locals, binding)
						}
					}
				}
			}
		}
		return true
	})

	sort.SliceStable(locals, func(i, j int) bool {
		return locals[i].Name < locals[j].Name
	})
	return locals
}

func (l *girPackageLowerer) rejectDuplicateBindings(pos token.Pos, bindings []girBinding) {
	seen := make(map[string]struct{}, len(bindings))
	for _, binding := range bindings {
		if binding.Name == "_" || binding.Name == "" {
			continue
		}
		if _, ok := seen[binding.Name]; ok {
			l.reject(pos, "GIR lowering", fmt.Sprintf("duplicate GIR variable name %q is rejected by GO-005", binding.Name))
			continue
		}
		seen[binding.Name] = struct{}{}
	}
}

func (l *girPackageLowerer) lowerTupleBindings(pos token.Pos, tuple *types.Tuple, prefix string) []girBinding {
	if tuple == nil {
		return nil
	}
	bindings := make([]girBinding, 0, tuple.Len())
	for i := 0; i < tuple.Len(); i++ {
		variable := tuple.At(i)
		name := variable.Name()
		if name == "" {
			name = fmt.Sprintf("%s%d", prefix, i)
		}
		bindingPos := variable.Pos()
		if !bindingPos.IsValid() {
			bindingPos = pos
		}
		if binding, ok := l.lowerBinding(bindingPos, name, variable.Type()); ok {
			bindings = append(bindings, binding)
		}
	}
	return bindings
}

func (l *girPackageLowerer) lowerBinding(pos token.Pos, name string, typ types.Type) (girBinding, bool) {
	girType, ok := girTypeFromGoType(typ)
	if !ok {
		l.reject(pos, "GIR lowering", fmt.Sprintf("type %s is not lowered by GO-005", typ.String()))
		return girBinding{}, false
	}
	return girBinding{Name: name, Type: girType}, true
}

func (l *girFunctionLowerer) lowerStatements(statements []ast.Stmt) {
	for index, stmt := range statements {
		if l.currentTerminated() {
			return
		}
		if ifStmt, ok := stmt.(*ast.IfStmt); ok {
			l.lowerIf(ifStmt, statements[index+1:])
			return
		}
		l.lowerStatement(stmt)
	}
}

func (l *girFunctionLowerer) lowerStatement(stmt ast.Stmt) {
	switch stmt := stmt.(type) {
	case *ast.AssignStmt:
		l.lowerAssignment(stmt)
	case *ast.BlockStmt:
		l.lowerStatements(stmt.List)
	case *ast.DeclStmt:
		l.lowerDeclStatement(stmt)
	case *ast.EmptyStmt:
	case *ast.ExprStmt:
		l.lowerExpr(stmt.X)
	case *ast.ReturnStmt:
		l.lowerReturn(stmt)
	default:
		l.reject(stmt.Pos(), "GIR lowering", fmt.Sprintf("%T statements are not lowered by GO-005", stmt))
	}
}

func (l *girFunctionLowerer) lowerIf(stmt *ast.IfStmt, rest []ast.Stmt) {
	if stmt.Init != nil {
		l.lowerStatement(stmt.Init)
		if l.currentTerminated() {
			return
		}
	}

	cond, ok := l.lowerExpr(stmt.Cond)
	if !ok {
		return
	}

	thenLabel := l.nextBlockLabel("if_then")
	elseLabel := l.nextBlockLabel("if_else")
	afterLabel := ""
	if len(rest) > 0 {
		afterLabel = l.nextBlockLabel("if_after")
		if stmt.Else == nil {
			elseLabel = afterLabel
		}
	}

	l.terminate(girTerminator{
		Kind:      "Branch",
		Cond:      &cond,
		ThenLabel: thenLabel,
		ElseLabel: elseLabel,
	})

	l.current = l.addBlock(thenLabel)
	l.lowerStatements(stmt.Body.List)
	if !l.currentTerminated() {
		if afterLabel != "" {
			l.terminate(girTerminator{Kind: "Jump", Label: afterLabel})
		} else {
			l.reject(stmt.Body.End(), "GIR lowering", "if branch can fall through without return")
		}
	}

	if stmt.Else != nil {
		l.current = l.addBlock(elseLabel)
		switch elseStmt := stmt.Else.(type) {
		case *ast.BlockStmt:
			l.lowerStatements(elseStmt.List)
		case *ast.IfStmt:
			l.reject(elseStmt.Pos(), "GIR lowering", "else-if lowering is not implemented in GO-005")
		default:
			l.reject(elseStmt.Pos(), "GIR lowering", fmt.Sprintf("%T else branch is not lowered by GO-005", elseStmt))
		}
		if !l.currentTerminated() {
			if afterLabel != "" {
				l.terminate(girTerminator{Kind: "Jump", Label: afterLabel})
			} else {
				l.reject(stmt.Else.End(), "GIR lowering", "else branch can fall through without return")
			}
		}
	}

	if afterLabel != "" {
		l.current = l.addBlock(afterLabel)
		l.lowerStatements(rest)
	}
}

func (l *girFunctionLowerer) lowerDeclStatement(stmt *ast.DeclStmt) {
	decl, ok := stmt.Decl.(*ast.GenDecl)
	if !ok || decl.Tok != token.VAR {
		l.reject(stmt.Pos(), "GIR lowering", "only local var declarations are lowered by GO-005")
		return
	}
	for _, spec := range decl.Specs {
		valueSpec, ok := spec.(*ast.ValueSpec)
		if !ok {
			continue
		}
		for index, name := range valueSpec.Names {
			if index >= len(valueSpec.Values) {
				continue
			}
			value, ok := l.lowerExpr(valueSpec.Values[index])
			if !ok {
				continue
			}
			typ, ok := l.typeOfIdent(name)
			if !ok {
				continue
			}
			l.emitCopy(name.Name, value, typ)
		}
	}
}

func (l *girFunctionLowerer) lowerAssignment(stmt *ast.AssignStmt) {
	if len(stmt.Lhs) != len(stmt.Rhs) {
		l.reject(stmt.Pos(), "GIR lowering", "multi-target assignments with mismatched RHS count are not lowered by GO-005")
		return
	}
	for index, lhs := range stmt.Lhs {
		name, ok := lhs.(*ast.Ident)
		if !ok {
			l.reject(lhs.Pos(), "GIR lowering", "only local variable assignments are lowered by GO-005")
			continue
		}
		value, ok := l.lowerExpr(stmt.Rhs[index])
		if !ok {
			continue
		}
		typ, ok := l.typeOfIdent(name)
		if !ok {
			continue
		}
		l.emitCopy(name.Name, value, typ)
	}
}

func (l *girFunctionLowerer) lowerReturn(stmt *ast.ReturnStmt) {
	values := make([]girValue, 0, len(stmt.Results))
	for _, expr := range stmt.Results {
		value, ok := l.lowerExpr(expr)
		if !ok {
			return
		}
		values = append(values, value)
	}
	l.terminate(girTerminator{
		Kind:   "Return",
		Values: values,
	})
}

func (l *girFunctionLowerer) lowerExpr(expr ast.Expr) (girValue, bool) {
	switch expr := expr.(type) {
	case *ast.BasicLit:
		return l.lowerBasicLit(expr)
	case *ast.BinaryExpr:
		return l.lowerBinaryExpr(expr)
	case *ast.CallExpr:
		return l.lowerCallExpr(expr)
	case *ast.CompositeLit:
		return l.lowerCompositeLit(expr)
	case *ast.Ident:
		return l.lowerIdent(expr)
	case *ast.IndexExpr:
		return l.lowerIndexExpr(expr)
	case *ast.ParenExpr:
		return l.lowerExpr(expr.X)
	case *ast.SelectorExpr:
		return l.lowerSelectorExpr(expr)
	case *ast.UnaryExpr:
		return l.lowerUnaryExpr(expr)
	default:
		l.reject(expr.Pos(), "GIR lowering", fmt.Sprintf("%T expressions are not lowered by GO-005", expr))
		return girValue{}, false
	}
}

func (l *girFunctionLowerer) lowerBasicLit(lit *ast.BasicLit) (girValue, bool) {
	if lit.Kind != token.INT {
		l.reject(lit.Pos(), "GIR lowering", fmt.Sprintf("%s literals are not lowered by GO-005", lit.Kind))
		return girValue{}, false
	}
	typ, ok := l.girTypeOf(lit)
	if !ok {
		return girValue{}, false
	}
	if typ.Kind != "bv" {
		l.reject(lit.Pos(), "GIR lowering", "integer literal does not have a fixed-width integer type")
		return girValue{}, false
	}
	value, err := strconv.ParseInt(lit.Value, 0, 64)
	if err != nil {
		l.reject(lit.Pos(), "GIR lowering", fmt.Sprintf("integer literal %q cannot be parsed: %v", lit.Value, err))
		return girValue{}, false
	}
	return girValue{Int: &girIntLiteral{
		Value:  strconv.FormatInt(value, 10),
		Width:  typ.Width,
		Signed: typ.Signed != nil && *typ.Signed,
	}}, true
}

func (l *girFunctionLowerer) lowerBinaryExpr(expr *ast.BinaryExpr) (girValue, bool) {
	lhs, ok := l.lowerExpr(expr.X)
	if !ok {
		return girValue{}, false
	}
	rhs, ok := l.lowerExpr(expr.Y)
	if !ok {
		return girValue{}, false
	}

	op, ok := l.binaryOp(expr)
	if !ok {
		return girValue{}, false
	}
	typ, ok := l.girTypeOf(expr)
	if !ok {
		return girValue{}, false
	}
	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:   id,
		Kind: "BinOp",
		Op:   op,
		Type: typ,
		LHS:  &lhs,
		RHS:  &rhs,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) lowerCallExpr(expr *ast.CallExpr) (girValue, bool) {
	functionID, ok := l.staticFunctionID(expr.Fun)
	if !ok {
		l.reject(expr.Pos(), "GIR lowering", "only static calls to named pure functions are lowered by GO-005")
		return girValue{}, false
	}
	args := make([]girValue, 0, len(expr.Args))
	for _, arg := range expr.Args {
		value, ok := l.lowerExpr(arg)
		if !ok {
			return girValue{}, false
		}
		args = append(args, value)
	}
	typ, ok := l.girTypeOf(expr)
	if !ok {
		return girValue{}, false
	}
	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:       id,
		Kind:     "CallStatic",
		Type:     typ,
		Function: functionID,
		Args:     args,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) lowerIdent(ident *ast.Ident) (girValue, bool) {
	if ident.Name == "true" {
		value := true
		return girValue{Bool: &value}, true
	}
	if ident.Name == "false" {
		value := false
		return girValue{Bool: &value}, true
	}
	if ident.Name == "_" {
		l.reject(ident.Pos(), "GIR lowering", "blank identifier cannot be used as a GIR value")
		return girValue{}, false
	}
	if obj := l.packageLowerer.pkg.TypesInfo.Uses[ident]; obj != nil {
		if constant, ok := obj.(*types.Const); ok {
			return l.lowerConstant(ident.Pos(), constant)
		}
	}
	return girValue{Var: ident.Name}, true
}

func (l *girFunctionLowerer) lowerUnaryExpr(expr *ast.UnaryExpr) (girValue, bool) {
	value, ok := l.lowerExpr(expr.X)
	if !ok {
		return girValue{}, false
	}
	switch expr.Op {
	case token.ADD:
		return value, true
	case token.NOT:
		return l.emitUnary(expr, "not", value)
	case token.SUB:
		return l.emitUnary(expr, "bv_neg", value)
	case token.XOR:
		return l.emitUnary(expr, "bv_not", value)
	default:
		l.reject(expr.Pos(), "GIR lowering", fmt.Sprintf("unary operator %s is not lowered by GO-005", expr.Op))
		return girValue{}, false
	}
}

func (l *girFunctionLowerer) lowerConstant(pos token.Pos, constantValue *types.Const) (girValue, bool) {
	switch constantValue.Val().Kind() {
	case constant.Bool:
		value := constant.BoolVal(constantValue.Val())
		return girValue{Bool: &value}, true
	case constant.Int:
		typ, ok := girTypeFromGoType(constantValue.Type())
		if !ok || typ.Kind != "bv" {
			l.reject(pos, "GIR lowering", "integer constant does not have a fixed-width integer type")
			return girValue{}, false
		}
		value, exact := constant.Int64Val(constantValue.Val())
		if !exact {
			l.reject(pos, "GIR lowering", "integer constant cannot be represented as int64")
			return girValue{}, false
		}
		return girValue{Int: &girIntLiteral{
			Value:  strconv.FormatInt(value, 10),
			Width:  typ.Width,
			Signed: typ.Signed != nil && *typ.Signed,
		}}, true
	default:
		l.reject(pos, "GIR lowering", fmt.Sprintf("constant kind %s is not lowered by GO-005", constantValue.Val().Kind()))
		return girValue{}, false
	}
}

func (l *girFunctionLowerer) emitUnary(expr ast.Expr, op string, value girValue) (girValue, bool) {
	typ, ok := l.girTypeOf(expr)
	if !ok {
		return girValue{}, false
	}
	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:    id,
		Kind:  "UnaryOp",
		Op:    op,
		Type:  typ,
		Value: &value,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) emitCopy(target string, value girValue, typ girType) {
	if target == "_" {
		return
	}
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:     l.nextTemp(),
		Kind:   "Copy",
		Type:   typ,
		Target: target,
		Value:  &value,
	})
}

func (l *girFunctionLowerer) binaryOp(expr *ast.BinaryExpr) (string, bool) {
	switch expr.Op {
	case token.LAND:
		return "and", true
	case token.LOR:
		return "or", true
	case token.EQL:
		return "eq", true
	case token.NEQ:
		return "not_eq", true
	}

	operandType := l.packageLowerer.pkg.TypesInfo.TypeOf(expr.X)
	_, signed, ok := fixedWidthInteger(operandType)
	if !ok {
		l.reject(expr.Pos(), "GIR lowering", fmt.Sprintf("operator %s requires fixed-width integer operands", expr.Op))
		return "", false
	}

	switch expr.Op {
	case token.ADD:
		return "bv_add", true
	case token.SUB:
		return "bv_sub", true
	case token.MUL:
		return "bv_mul", true
	case token.QUO:
		if signed {
			return "bv_sdiv", true
		}
		return "bv_udiv", true
	case token.REM:
		if signed {
			return "bv_srem", true
		}
		return "bv_urem", true
	case token.AND:
		return "bv_and", true
	case token.OR:
		return "bv_or", true
	case token.XOR:
		return "bv_xor", true
	case token.SHL:
		return "bv_shl", true
	case token.SHR:
		if signed {
			return "bv_ashr", true
		}
		return "bv_lshr", true
	case token.LSS:
		return signedComparisonOp("lt", signed), true
	case token.LEQ:
		return signedComparisonOp("le", signed), true
	case token.GTR:
		return signedComparisonOp("gt", signed), true
	case token.GEQ:
		return signedComparisonOp("ge", signed), true
	default:
		l.reject(expr.Pos(), "GIR lowering", fmt.Sprintf("binary operator %s is not lowered by GO-005", expr.Op))
		return "", false
	}
}

func signedComparisonOp(op string, signed bool) string {
	if signed {
		return "signed_" + op
	}
	return "unsigned_" + op
}

func (l *girFunctionLowerer) staticFunctionID(expr ast.Expr) (string, bool) {
	switch expr := expr.(type) {
	case *ast.Ident:
		if function, ok := l.packageLowerer.pkg.TypesInfo.Uses[expr].(*types.Func); ok {
			return function.FullName(), true
		}
	case *ast.SelectorExpr:
		if selection := l.packageLowerer.pkg.TypesInfo.Selections[expr]; selection != nil {
			return selection.Obj().(*types.Func).FullName(), true
		}
		if function, ok := l.packageLowerer.pkg.TypesInfo.Uses[expr.Sel].(*types.Func); ok {
			return function.FullName(), true
		}
	}
	return "", false
}

func (l *girFunctionLowerer) typeOfIdent(ident *ast.Ident) (girType, bool) {
	if obj := l.packageLowerer.pkg.TypesInfo.Defs[ident]; obj != nil {
		return l.girTypeOfObject(ident.Pos(), obj.Type())
	}
	if obj := l.packageLowerer.pkg.TypesInfo.Uses[ident]; obj != nil {
		return l.girTypeOfObject(ident.Pos(), obj.Type())
	}
	l.reject(ident.Pos(), "GIR lowering", fmt.Sprintf("identifier %s has no type information", ident.Name))
	return girType{}, false
}

func (l *girFunctionLowerer) girTypeOfObject(pos token.Pos, typ types.Type) (girType, bool) {
	typLowered, ok := girTypeFromGoType(typ)
	if !ok {
		l.reject(pos, "GIR lowering", fmt.Sprintf("type %s is not lowered by GO-005", typ.String()))
		return girType{}, false
	}
	return typLowered, true
}

func (l *girFunctionLowerer) girTypeOf(expr ast.Expr) (girType, bool) {
	typ := l.packageLowerer.pkg.TypesInfo.TypeOf(expr)
	if typ == nil {
		l.reject(expr.Pos(), "GIR lowering", fmt.Sprintf("%T expression has no type information", expr))
		return girType{}, false
	}
	typLowered, ok := girTypeFromGoType(typ)
	if !ok {
		l.reject(expr.Pos(), "GIR lowering", fmt.Sprintf("type %s is not lowered by GO-005", typ.String()))
		return girType{}, false
	}
	return typLowered, true
}

func (l *girFunctionLowerer) newBlock(label string) *girBlock {
	block := girBlock{
		Label:        label,
		Parameters:   []girBinding{},
		Instructions: []girInstruction{},
	}
	l.function.Blocks = append(l.function.Blocks, block)
	return &l.function.Blocks[len(l.function.Blocks)-1]
}

func (l *girFunctionLowerer) addBlock(label string) *girBlock {
	return l.newBlock(label)
}

func (l *girFunctionLowerer) nextBlockLabel(prefix string) string {
	label := fmt.Sprintf("%s_%d", prefix, l.blockIndex)
	l.blockIndex++
	return label
}

func (l *girFunctionLowerer) nextTemp() string {
	id := fmt.Sprintf("t%d", l.tempIndex)
	l.tempIndex++
	return id
}

func (l *girFunctionLowerer) currentTerminated() bool {
	return l.current != nil && l.current.Terminator.Kind != ""
}

func (l *girFunctionLowerer) terminate(terminator girTerminator) {
	if l.current == nil {
		return
	}
	l.current.Terminator = terminator
}

func (l *girFunctionLowerer) reject(pos token.Pos, feature string, reason string) {
	l.packageLowerer.reject(pos, feature, reason)
}

func (l *girPackageLowerer) reject(pos token.Pos, feature string, reason string) {
	l.add(rejectedFeature{
		Location: l.location(pos),
		Feature:  feature,
		Reason:   reason,
	})
}

func (l *girPackageLowerer) add(finding rejectedFeature) {
	if finding.Feature == "" || finding.Reason == "" {
		return
	}
	if _, ok := l.seen[finding]; ok {
		return
	}
	l.seen[finding] = struct{}{}
	l.findings = append(l.findings, finding)
}

func (l *girPackageLowerer) location(pos token.Pos) string {
	if l.fset == nil || !pos.IsValid() {
		return ""
	}
	position := l.fset.Position(pos)
	if !position.IsValid() {
		return ""
	}
	location := normalizePath(l.baseDir, position.Filename)
	if position.Line > 0 {
		location += fmt.Sprintf(":%d", position.Line)
	}
	if position.Column > 0 {
		location += fmt.Sprintf(":%d", position.Column)
	}
	return location
}

func emptyGIRContracts() girContracts {
	return girContracts{
		Requires: []girContractExpr{},
		Ensures:  []girContractExpr{},
		Modifies: []string{},
		Loops:    []girLoopContract{},
	}
}

func functionID(packagePath string, name string) string {
	return packagePath + "." + name
}

func girTypeFromGoType(typ types.Type) (girType, bool) {
	if typ == nil {
		return girType{}, false
	}
	if named, ok := typ.(*types.Named); ok {
		return girNamedType(named)
	}
	if basic, ok := typ.Underlying().(*types.Basic); ok {
		switch basic.Kind() {
		case types.Bool, types.UntypedBool:
			return girType{Kind: "bool"}, true
		}
		if width, signed, ok := fixedWidthInteger(typ); ok {
			return girType{Kind: "bv", Width: width, Signed: boolPtr(signed)}, true
		}
	}
	switch underlying := typ.Underlying().(type) {
	case *types.Array:
		return girArrayType(underlying)
	case *types.Struct:
		return girStructType("", underlying)
	}
	return girType{}, false
}

func fixedWidthInteger(typ types.Type) (int, bool, bool) {
	if typ == nil {
		return 0, false, false
	}
	basic, ok := typ.Underlying().(*types.Basic)
	if !ok {
		return 0, false, false
	}
	switch basic.Kind() {
	case types.Int8:
		return 8, true, true
	case types.Int16:
		return 16, true, true
	case types.Int32:
		return 32, true, true
	case types.Int64:
		return 64, true, true
	case types.Uint8:
		return 8, false, true
	case types.Uint16:
		return 16, false, true
	case types.Uint32:
		return 32, false, true
	case types.Uint64:
		return 64, false, true
	}
	return 0, false, false
}

func boolPtr(value bool) *bool {
	return &value
}

func sortRejectedFeatures(findings []rejectedFeature) {
	sort.Slice(findings, func(i, j int) bool {
		if findings[i].Location != findings[j].Location {
			return findings[i].Location < findings[j].Location
		}
		if findings[i].Feature != findings[j].Feature {
			return findings[i].Feature < findings[j].Feature
		}
		return findings[i].Reason < findings[j].Reason
	})
}
