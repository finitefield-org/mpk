package main

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"sort"
	"strconv"
	"strings"

	"golang.org/x/tools/go/packages"
)

type loweringFinding struct {
	Code       string
	Message    string
	FunctionID string
	Origin     sourceOrigin
}

type featureDetector struct {
	pkg         *packages.Package
	paths       map[string]string
	packages    map[string]struct{}
	function    string
	resultCount int
	findings    []loweringFinding
	seen        map[string]struct{}
}

func detectGoFeatures(loaded packageLoadResult) []loweringFinding {
	findings := make([]loweringFinding, 0)
	allowedPackages := make(map[string]struct{}, len(loaded.Packages))
	for _, value := range loaded.Packages {
		allowedPackages[value.PackagePath] = struct{}{}
	}
	for _, value := range loaded.Packages {
		detector := featureDetector{
			pkg: value.packageValue, paths: packageSourcePaths(value),
			packages: allowedPackages, seen: make(map[string]struct{}),
		}
		for _, file := range detector.pkg.Syntax {
			detector.file(file)
		}
		findings = append(findings, detector.findings...)
	}
	sort.Slice(findings, func(left, right int) bool {
		l, r := findings[left], findings[right]
		if l.Origin.NormalizedPath != r.Origin.NormalizedPath {
			return l.Origin.NormalizedPath < r.Origin.NormalizedPath
		}
		if l.Origin.Start != r.Origin.Start {
			return l.Origin.Start < r.Origin.Start
		}
		if l.Code != r.Code {
			return l.Code < r.Code
		}
		if l.Message != r.Message {
			return l.Message < r.Message
		}
		if l.FunctionID != r.FunctionID {
			return l.FunctionID < r.FunctionID
		}
		return l.Origin.End < r.Origin.End
	})
	return findings
}

func (d *featureDetector) file(file *ast.File) {
	for _, declaration := range file.Decls {
		switch declaration := declaration.(type) {
		case *ast.FuncDecl:
			d.functionDecl(declaration)
		case *ast.GenDecl:
			d.topLevelDecl(declaration)
		default:
			d.reject(declaration, "GO_SUBSET_SYNTAX", "unsupported top-level declaration")
		}
	}
}

func (d *featureDetector) topLevelDecl(declaration *ast.GenDecl) {
	switch declaration.Tok {
	case token.IMPORT:
		for _, spec := range declaration.Specs {
			value := spec.(*ast.ImportSpec)
			path, _ := strconv.Unquote(value.Path.Value)
			switch path {
			case "unsafe":
				d.reject(value, "GO_SUBSET_UNSAFE", "unsafe imports are outside the Go profile")
			case "C":
				d.reject(value, "GO_SUBSET_CGO", "cgo is outside the Go profile")
			case "reflect":
				d.reject(value, "GO_SUBSET_REFLECTION", "reflection is outside the Go profile")
			default:
				if _, capturedPackage := d.packages[path]; !capturedPackage {
					d.reject(value, "GO_SUBSET_IMPORT", "external imports are outside the Go profile")
				}
			}
		}
	case token.VAR:
		d.reject(declaration, "GO_SUBSET_GLOBAL_STATE", "package mutable state is outside the Go profile")
	case token.CONST:
		for _, spec := range declaration.Specs {
			value := spec.(*ast.ValueSpec)
			if len(value.Names) != 1 || value.Type == nil || len(value.Values) != 1 || value.Names[0].Name == "_" {
				d.reject(value, "GO_SUBSET_SYNTAX", "constants require one named explicit typed declaration")
			}
			if len(value.Names) == 1 && !validASCIIIdentifier(value.Names[0].Name) {
				d.reject(value.Names[0], "GO_SUBSET_SYNTAX", "constant name is not an ASCII identifier")
			}
			if expressionContainsIdentifier(value, "iota") {
				d.reject(value, "GO_SUBSET_SYNTAX", "iota constants are outside the Go profile")
			}
			if len(value.Names) == 1 {
				if object, ok := d.pkg.TypesInfo.Defs[value.Names[0]].(*types.Const); ok {
					d.typeAt(value, object.Type())
				}
			}
			for _, expression := range value.Values {
				ast.Inspect(expression, func(node ast.Node) bool {
					if typed, ok := node.(ast.Expr); ok {
						d.expression(typed)
					}
					return true
				})
			}
		}
	case token.TYPE:
		for _, spec := range declaration.Specs {
			value := spec.(*ast.TypeSpec)
			if !validASCIIIdentifier(value.Name.Name) {
				d.reject(value.Name, "GO_SUBSET_SYNTAX", "type name is not an ASCII identifier")
			}
			if value.Assign.IsValid() || value.TypeParams != nil {
				d.reject(value, "GO_SUBSET_GENERICS", "type aliases and generic types are outside the Go profile")
				continue
			}
			object, ok := d.pkg.TypesInfo.Defs[value.Name].(*types.TypeName)
			if !ok {
				d.reject(value, "GO_LOWER_TYPE", "type declaration lacks compiler identity")
				continue
			}
			named, ok := object.Type().(*types.Named)
			if !ok {
				d.reject(value, "GO_LOWER_TYPE", "only named structs are accepted")
				continue
			}
			if _, ok := virStructDecl(named); !ok {
				d.rejectType(value, object.Type())
			}
		}
	default:
		d.reject(declaration, "GO_SUBSET_SYNTAX", "unsupported declaration")
	}
}

func (d *featureDetector) functionDecl(declaration *ast.FuncDecl) {
	previous := d.function
	previousResults := d.resultCount
	d.function = canonicalFunctionID(d.pkg, declaration)
	defer func() { d.function, d.resultCount = previous, previousResults }()
	if !validASCIIIdentifier(declaration.Name.Name) {
		d.reject(declaration.Name, "GO_SUBSET_SYNTAX", "function name is not an ASCII identifier")
	}
	if declaration.Name.Name == "init" {
		d.reject(declaration.Name, "GO_SUBSET_GLOBAL_STATE", "init functions are outside the Go profile")
	}
	if declaration.Body == nil {
		d.reject(declaration, "GO_SUBSET_SYNTAX", "functions without bodies are outside the Go profile")
		return
	}
	if declaration.Type.TypeParams != nil {
		d.reject(declaration, "GO_SUBSET_GENERICS", "generic functions are outside the Go profile")
	}
	if object, ok := d.pkg.TypesInfo.Defs[declaration.Name].(*types.Func); ok {
		signature, _ := object.Type().(*types.Signature)
		if signature != nil {
			d.resultCount = signature.Results().Len()
			if signature.Variadic() {
				d.reject(declaration, "GO_SUBSET_SYNTAX", "variadic parameters are outside the Go profile")
			}
			if signature.Recv() != nil {
				if _, pointer := signature.Recv().Type().Underlying().(*types.Pointer); pointer {
					d.reject(declaration.Recv, "GO_SUBSET_POINTER", "pointer receivers are outside the Go profile")
				}
			}
			d.tuple(declaration, signature.Params(), 256, "VIR_LIMIT_PARAMS")
			d.tuple(declaration, signature.Results(), 16, "VIR_LIMIT_RESULTS")
		}
	}
	ast.Inspect(declaration.Body, func(node ast.Node) bool {
		if node == nil {
			return true
		}
		switch value := node.(type) {
		case *ast.FuncLit:
			d.reject(value, "GO_SUBSET_FUNCTION_VALUE", "function literals are outside the Go profile")
			return false
		case ast.Stmt:
			d.statement(value)
		case ast.Expr:
			d.expression(value)
		}
		return true
	})
}

func (d *featureDetector) tuple(node ast.Node, tuple *types.Tuple, maximum int, code string) {
	if tuple == nil {
		return
	}
	if tuple.Len() > maximum {
		d.reject(node, code, "function tuple exceeds the profile limit")
	}
	for index := 0; index < tuple.Len(); index++ {
		d.typeAt(node, tuple.At(index).Type())
	}
}

func (d *featureDetector) statement(statement ast.Stmt) {
	switch value := statement.(type) {
	case *ast.AssignStmt:
		if len(value.Lhs) != 1 || len(value.Rhs) != 1 || value.Tok != token.ASSIGN && value.Tok != token.DEFINE {
			d.reject(value, "GO_SUBSET_ASSIGNMENT", "only one-target local assignments are accepted")
			return
		}
		name, ok := value.Lhs[0].(*ast.Ident)
		if !ok {
			d.reject(value, "GO_SUBSET_ASSIGNMENT", "assignment target must be a local")
			return
		}
		if name.Name != "_" {
			object := d.pkg.TypesInfo.Uses[name]
			if value.Tok == token.DEFINE {
				object = d.pkg.TypesInfo.Defs[name]
			}
			variable, ok := object.(*types.Var)
			if !ok || variable.Parent() == d.pkg.Types.Scope() {
				d.reject(value, "GO_SUBSET_ASSIGNMENT", "assignment target must be a source local")
			} else if value.Tok == token.ASSIGN && isParameter(d.pkg, variable) {
				d.reject(value, "GO_SUBSET_ASSIGNMENT", "parameter assignment is outside the Go profile")
			}
		}
	case *ast.DeclStmt:
		declaration, ok := value.Decl.(*ast.GenDecl)
		if !ok || declaration.Tok != token.VAR || len(declaration.Specs) != 1 {
			d.reject(value, "GO_SUBSET_SYNTAX", "only one local var declaration is accepted")
			return
		}
		spec := declaration.Specs[0].(*ast.ValueSpec)
		if len(spec.Names) != 1 || len(spec.Values) > 1 || spec.Names[0].Name == "_" {
			d.reject(value, "GO_SUBSET_ASSIGNMENT", "local declarations require one nonblank name")
		}
	case *ast.ForStmt:
		if value.Cond == nil || shortCircuitRoot(value.Cond) || value.Init != nil && !acceptedLoopEdge(value.Init) || value.Post != nil && !acceptedLoopEdge(value.Post) || containsLoopOrReturn(value.Body) {
			d.reject(value, "GO_SUBSET_LOOP", "loop does not match the contracted for-loop shape")
		}
	case *ast.RangeStmt:
		d.reject(value, "GO_SUBSET_ITERATION", "range is outside the deterministic Go profile")
	case *ast.GoStmt:
		d.reject(value, "GO_SUBSET_GOROUTINE", "goroutines are outside the Go profile")
	case *ast.DeferStmt:
		d.reject(value, "GO_SUBSET_DEFER", "defer is outside the Go profile")
	case *ast.SendStmt, *ast.SelectStmt:
		d.reject(value, "GO_SUBSET_CHANNEL", "channels are outside the Go profile")
	case *ast.BranchStmt, *ast.IncDecStmt, *ast.LabeledStmt, *ast.SwitchStmt, *ast.TypeSwitchStmt:
		d.reject(value, "GO_SUBSET_SYNTAX", "unsupported control-flow statement")
	case *ast.IfStmt:
		if _, nested := value.Else.(*ast.IfStmt); nested {
			d.reject(value.Else, "GO_SUBSET_SYNTAX", "else-if is outside the Go profile")
		}
	case *ast.ReturnStmt:
		if d.resultCount > 0 && len(value.Results) == 0 {
			d.reject(value, "GO_SUBSET_ASSIGNMENT", "naked return is outside the Go profile")
		}
	case *ast.BlockStmt, *ast.ExprStmt, *ast.EmptyStmt:
	default:
		d.reject(value, "GO_SUBSET_SYNTAX", "unsupported statement")
	}
}

func (d *featureDetector) expression(expression ast.Expr) {
	if identifier, ok := expression.(*ast.Ident); ok {
		if _, packageName := d.pkg.TypesInfo.Uses[identifier].(*types.PkgName); packageName {
			return
		}
		if _, function := d.pkg.TypesInfo.Uses[identifier].(*types.Func); function {
			return
		}
	}
	if selector, ok := expression.(*ast.SelectorExpr); ok {
		if _, function := d.pkg.TypesInfo.Uses[selector.Sel].(*types.Func); function {
			return
		}
	}
	if typ := d.pkg.TypesInfo.TypeOf(expression); typ != nil && !isIntegerLiteralSyntax(expression) {
		d.typeAt(expression, typ)
	}
	switch value := expression.(type) {
	case *ast.BasicLit:
		switch value.Kind {
		case token.STRING, token.CHAR:
			d.reject(value, "GO_SUBSET_STRING", "string and character literals are outside the Go profile")
		case token.FLOAT:
			d.reject(value, "GO_SUBSET_FLOAT", "floating-point literals are outside the Go profile")
		case token.IMAG:
			d.reject(value, "GO_SUBSET_COMPLEX", "complex literals are outside the Go profile")
		}
	case *ast.CallExpr:
		d.call(value)
	case *ast.UnaryExpr:
		switch value.Op {
		case token.AND, token.MUL:
			d.reject(value, "GO_SUBSET_POINTER", "pointer operations are outside the Go profile")
		case token.ARROW:
			d.reject(value, "GO_SUBSET_CHANNEL", "channel receives are outside the Go profile")
		}
	case *ast.BinaryExpr:
		if value.Op == token.AND_NOT {
			d.reject(value, "GO_SUBSET_SYNTAX", "bit-clear is outside the Go profile")
		}
		if isComparison(value.Op) && (untypedIntegerExpression(d.pkg.TypesInfo.TypeOf(value.X)) || untypedIntegerExpression(d.pkg.TypesInfo.TypeOf(value.Y))) {
			d.reject(value, "GO_LOWER_UNTYPED_INTEGER", "untyped integer comparison has no accepted fixed-width context")
		}
		if (value.Op == token.EQL || value.Op == token.NEQ) && aggregateType(d.pkg.TypesInfo.TypeOf(value.X)) {
			d.reject(value, "GO_SUBSET_SYNTAX", "aggregate equality is outside the Go profile")
		}
	case *ast.SliceExpr:
		d.reject(value, "GO_SUBSET_SLICES", "slicing is outside the Go profile")
	case *ast.TypeAssertExpr:
		d.reject(value, "GO_SUBSET_INTERFACE", "type assertions are outside the Go profile")
	case *ast.FuncType:
		d.reject(value, "GO_SUBSET_FUNCTION_VALUE", "function values are outside the Go profile")
	case *ast.SelectorExpr:
		if object, ok := d.pkg.TypesInfo.Uses[value.Sel].(*types.Const); ok && object.Pkg() != nil && object.Pkg().Path() != d.pkg.PkgPath {
			d.reject(value, "GO_SUBSET_IMPORT", "cross-unit constants are outside the Go profile")
		}
	}
}

func (d *featureDetector) call(call *ast.CallExpr) {
	if identifier, ok := call.Fun.(*ast.Ident); ok {
		if builtin, ok := d.pkg.TypesInfo.Uses[identifier].(*types.Builtin); ok {
			code := "GO_SUBSET_SYNTAX"
			switch builtin.Name() {
			case "new", "make":
				code = "GO_SUBSET_HEAP"
			case "append", "copy":
				code = "GO_SUBSET_SLICES"
			case "delete":
				code = "GO_SUBSET_MAPS"
			case "close":
				code = "GO_SUBSET_CHANNEL"
			case "panic", "recover":
				code = "GO_SUBSET_PANIC"
			case "print", "println":
				code = "GO_SUBSET_IO"
			}
			d.reject(call, code, "builtin call is outside the Go profile")
			return
		}
		if _, ok := d.pkg.TypesInfo.Uses[identifier].(*types.TypeName); ok {
			return
		}
	}
	if _, ok := resolvedStaticFunction(d.pkg, call.Fun); !ok {
		d.reject(call, "GO_SUBSET_FUNCTION_VALUE", "call is not a direct static function or conversion")
	}
}

func (d *featureDetector) typeAt(node ast.Node, typ types.Type) {
	if _, ok := virTypeFromGo(typ); ok {
		return
	}
	d.rejectType(node, typ)
}

func (d *featureDetector) rejectType(node ast.Node, typ types.Type) {
	code := "GO_LOWER_TYPE"
	if typ != nil {
		switch value := typ.Underlying().(type) {
		case *types.Basic:
			switch value.Kind() {
			case types.Int, types.Uint, types.Uintptr, types.UntypedInt, types.UntypedRune:
				code = "GO_SUBSET_MACHINE_INT"
			case types.String, types.UntypedString:
				code = "GO_SUBSET_STRING"
			case types.Float32, types.Float64, types.UntypedFloat:
				code = "GO_SUBSET_FLOAT"
			case types.Complex64, types.Complex128, types.UntypedComplex:
				code = "GO_SUBSET_COMPLEX"
			case types.UnsafePointer:
				code = "GO_SUBSET_UNSAFE"
			}
		case *types.Map:
			code = "GO_SUBSET_MAPS"
		case *types.Slice:
			code = "GO_SUBSET_SLICES"
		case *types.Pointer:
			code = "GO_SUBSET_POINTER"
		case *types.Interface:
			code = "GO_SUBSET_INTERFACE"
		case *types.Chan:
			code = "GO_SUBSET_CHANNEL"
		case *types.Signature:
			code = "GO_SUBSET_FUNCTION_VALUE"
		}
		if named, isNamed := typ.(*types.Named); isNamed {
			if _, isStruct := named.Underlying().(*types.Struct); !isStruct {
				code = "GO_SUBSET_SYNTAX"
			}
		}
	}
	d.reject(node, code, "type is outside the fixed-width Go profile")
}

func (d *featureDetector) reject(node ast.Node, code, message string) {
	origin := originForNode(d.pkg.Fset, d.paths, node)
	key := fmt.Sprintf("%s:%d:%s:%s", origin.NormalizedPath, origin.Start, code, d.function)
	if _, exists := d.seen[key]; exists {
		return
	}
	d.seen[key] = struct{}{}
	d.findings = append(d.findings, loweringFinding{Code: code, Message: message, FunctionID: d.function, Origin: origin})
}

func packageSourcePaths(value loadedPackage) map[string]string {
	paths := make(map[string]string, len(value.packageValue.CompiledGoFiles))
	for index, absolute := range value.packageValue.CompiledGoFiles {
		if index < len(value.CompiledGoFiles) {
			paths[absolute] = value.CompiledGoFiles[index]
		}
	}
	return paths
}

func originForNode(files *token.FileSet, paths map[string]string, node ast.Node) sourceOrigin {
	if files == nil || node == nil {
		return sourceOrigin{}
	}
	start := files.PositionFor(node.Pos(), false)
	end := files.PositionFor(node.End(), false)
	path := paths[start.Filename]
	if path == "" {
		path = strings.TrimPrefix(strings.ReplaceAll(start.Filename, "\\", "/"), "/")
	}
	if !start.IsValid() || !end.IsValid() || start.Offset < 0 || end.Offset <= start.Offset {
		return sourceOrigin{}
	}
	return sourceOrigin{Kind: "source", InputKind: sourceInputKind, NormalizedPath: path, Start: int64(start.Offset), End: int64(end.Offset)}
}

func isParameter(pkg *packages.Package, variable *types.Var) bool {
	for _, file := range pkg.Syntax {
		for _, declaration := range file.Decls {
			function, ok := declaration.(*ast.FuncDecl)
			if !ok {
				continue
			}
			object, _ := pkg.TypesInfo.Defs[function.Name].(*types.Func)
			if object == nil {
				continue
			}
			signature, _ := object.Type().(*types.Signature)
			if signature == nil {
				continue
			}
			if signature.Recv() == variable {
				return true
			}
			for index := 0; index < signature.Params().Len(); index++ {
				if signature.Params().At(index) == variable {
					return true
				}
			}
		}
	}
	return false
}

func expressionContainsIdentifier(node ast.Node, name string) bool {
	found := false
	ast.Inspect(node, func(current ast.Node) bool {
		if identifier, ok := current.(*ast.Ident); ok && identifier.Name == name {
			found = true
			return false
		}
		return !found
	})
	return found
}

func acceptedLoopEdge(statement ast.Stmt) bool {
	assignment, ok := statement.(*ast.AssignStmt)
	return ok && len(assignment.Lhs) == 1 && len(assignment.Rhs) == 1 && (assignment.Tok == token.ASSIGN || assignment.Tok == token.DEFINE)
}

func containsLoopOrReturn(block *ast.BlockStmt) bool {
	found := false
	ast.Inspect(block, func(node ast.Node) bool {
		switch node.(type) {
		case *ast.ForStmt, *ast.RangeStmt, *ast.ReturnStmt:
			found = true
			return false
		}
		return !found
	})
	return found
}

func aggregateType(typ types.Type) bool {
	if typ == nil {
		return false
	}
	switch typ.Underlying().(type) {
	case *types.Array, *types.Struct:
		return true
	default:
		return false
	}
}

func isIntegerLiteralSyntax(expression ast.Expr) bool {
	literal, ok := expression.(*ast.BasicLit)
	return ok && literal.Kind == token.INT
}

func untypedIntegerExpression(typ types.Type) bool {
	if typ == nil {
		return false
	}
	basic, ok := typ.Underlying().(*types.Basic)
	return ok && (basic.Kind() == types.Int || basic.Kind() == types.UntypedInt || basic.Kind() == types.UntypedRune)
}

func isComparison(operator token.Token) bool {
	return operator == token.EQL || operator == token.NEQ || operator == token.LSS || operator == token.LEQ || operator == token.GTR || operator == token.GEQ
}
