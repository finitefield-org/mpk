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

type rejectedFeature struct {
	Location string `json:"location,omitempty"`
	Feature  string `json:"feature"`
	Reason   string `json:"reason"`
}

type featureDetector struct {
	baseDir  string
	pkg      *packages.Package
	fset     *token.FileSet
	findings []rejectedFeature
	seen     map[rejectedFeature]struct{}
}

func detectUnsupportedFeatures(loaded packageLoadResult) []rejectedFeature {
	var findings []rejectedFeature
	for _, pkg := range loaded.Packages {
		detector := &featureDetector{
			baseDir: loaded.BaseDir,
			pkg:     pkg,
			fset:    pkg.Fset,
			seen:    make(map[rejectedFeature]struct{}),
		}
		detector.detectPackage()
		findings = append(findings, detector.findings...)
	}

	sort.Slice(findings, func(i, j int) bool {
		if findings[i].Location != findings[j].Location {
			return findings[i].Location < findings[j].Location
		}
		if findings[i].Feature != findings[j].Feature {
			return findings[i].Feature < findings[j].Feature
		}
		return findings[i].Reason < findings[j].Reason
	})
	return findings
}

func (d *featureDetector) detectPackage() {
	for _, path := range d.pkg.IgnoredFiles {
		d.rejectFile(path, "build constraints", "build constraints that change selected files are rejected by Go subset v0")
	}
	for _, path := range d.pkg.OtherFiles {
		d.rejectFile(path, "cgo", "non-Go source files and cgo inputs are rejected by Go subset v0")
	}

	for _, file := range d.pkg.Syntax {
		d.detectFile(file)
	}
}

func (d *featureDetector) detectFile(file *ast.File) {
	d.detectBuildConstraints(file)

	for _, decl := range file.Decls {
		switch decl := decl.(type) {
		case *ast.FuncDecl:
			d.detectFunctionDecl(decl)
		case *ast.GenDecl:
			d.detectTopLevelGenDecl(decl)
		default:
			d.reject(decl.Pos(), "declarations", "only imports, constants, types, variables, functions, and methods are understood by Go subset v0")
		}
	}
}

func (d *featureDetector) detectBuildConstraints(file *ast.File) {
	for _, group := range file.Comments {
		for _, comment := range group.List {
			text := strings.TrimSpace(comment.Text)
			if strings.HasPrefix(text, "//go:build") || strings.HasPrefix(text, "// +build") {
				d.reject(comment.Pos(), "build constraints", "build constraints that change selected files are rejected by Go subset v0")
			}
		}
	}
}

func (d *featureDetector) detectTopLevelGenDecl(decl *ast.GenDecl) {
	switch decl.Tok {
	case token.IMPORT:
		for _, spec := range decl.Specs {
			importSpec, ok := spec.(*ast.ImportSpec)
			if !ok {
				continue
			}
			d.detectImport(importSpec)
		}
	case token.VAR:
		for _, spec := range decl.Specs {
			valueSpec, ok := spec.(*ast.ValueSpec)
			if !ok {
				continue
			}
			for _, name := range valueSpec.Names {
				d.reject(name.Pos(), "package-level mutable state", "package-level mutable state is rejected by Go subset v0")
			}
		}
	case token.CONST:
		for _, spec := range decl.Specs {
			d.detectValueSpecTypes(spec)
		}
	case token.TYPE:
		for _, spec := range decl.Specs {
			typeSpec, ok := spec.(*ast.TypeSpec)
			if !ok {
				continue
			}
			if typeSpec.TypeParams != nil && len(typeSpec.TypeParams.List) > 0 {
				d.reject(typeSpec.Name.Pos(), "generics", "generic type declarations are rejected by Go subset v0")
			}
			if obj, ok := d.pkg.TypesInfo.Defs[typeSpec.Name].(*types.TypeName); ok {
				d.detectType(typeSpec.Pos(), obj.Type())
			}
		}
	default:
		d.reject(decl.Pos(), "declarations", fmt.Sprintf("%s declarations are rejected by Go subset v0", decl.Tok))
	}
}

func (d *featureDetector) detectImport(importSpec *ast.ImportSpec) {
	importPath, err := strconv.Unquote(importSpec.Path.Value)
	if err != nil {
		d.reject(importSpec.Pos(), "imports", "malformed imports are rejected by Go subset v0")
		return
	}

	switch importPath {
	case "C":
		d.reject(importSpec.Pos(), "cgo", "cgo is rejected by Go subset v0")
	case "unsafe":
		d.reject(importSpec.Pos(), "unsafe", "unsafe is rejected by Go subset v0")
	case "reflect":
		d.reject(importSpec.Pos(), "reflection", "reflection is rejected by Go subset v0")
	default:
		d.reject(importSpec.Pos(), "imports", "imports are rejected until verified pure-function dependencies are modeled")
	}
}

func (d *featureDetector) detectFunctionDecl(decl *ast.FuncDecl) {
	if decl.Name.Name == "init" {
		d.reject(decl.Name.Pos(), "init functions", "init functions are rejected by Go subset v0")
	}
	if decl.Type.TypeParams != nil && len(decl.Type.TypeParams.List) > 0 {
		d.reject(decl.Name.Pos(), "generics", "generic functions are rejected by Go subset v0")
	}
	if decl.Recv != nil {
		d.detectReceiver(decl.Recv)
	}
	if obj, ok := d.pkg.TypesInfo.Defs[decl.Name].(*types.Func); ok {
		d.detectSignature(decl.Pos(), obj.Type().(*types.Signature))
	}
	if decl.Body == nil {
		return
	}

	ast.Inspect(decl.Body, func(node ast.Node) bool {
		if node == nil {
			return true
		}
		switch node := node.(type) {
		case *ast.FuncLit:
			d.reject(node.Pos(), "closures", "closures are rejected by Go subset v0")
			return false
		case ast.Stmt:
			d.detectStatement(node)
		case ast.Expr:
			d.detectExpression(node)
		}
		return true
	})
}

func (d *featureDetector) detectReceiver(receiver *ast.FieldList) {
	for _, field := range receiver.List {
		if _, ok := field.Type.(*ast.StarExpr); ok {
			d.reject(field.Pos(), "pointers", "pointer receivers are rejected by Go subset v0")
		}
		typ := d.pkg.TypesInfo.TypeOf(field.Type)
		d.detectType(field.Pos(), typ)
	}
}

func (d *featureDetector) detectStatement(stmt ast.Stmt) {
	switch stmt := stmt.(type) {
	case *ast.AssignStmt:
		d.detectAssignment(stmt)
	case *ast.BranchStmt:
		d.reject(stmt.Pos(), "control flow", fmt.Sprintf("%s statements are rejected by Go subset v0", stmt.Tok))
	case *ast.DeferStmt:
		d.reject(stmt.Pos(), "defer", "defer is rejected by Go subset v0")
	case *ast.DeclStmt:
		d.detectLocalDecl(stmt.Decl)
	case *ast.ExprStmt, *ast.BlockStmt, *ast.IfStmt, *ast.ReturnStmt, *ast.EmptyStmt:
	case *ast.ForStmt:
		d.reject(stmt.Pos(), "loops", "for loops require explicit invariant metadata before they are accepted")
	case *ast.GoStmt:
		d.reject(stmt.Pos(), "goroutines", "goroutines are rejected by Go subset v0")
	case *ast.IncDecStmt:
		d.reject(stmt.Pos(), "assignments", "increment and decrement statements are rejected; use explicit local assignment")
	case *ast.LabeledStmt:
		d.reject(stmt.Pos(), "labels", "labels are rejected by Go subset v0")
	case *ast.RangeStmt:
		d.reject(stmt.Pos(), "non-deterministic iteration", "range statements are rejected by Go subset v0")
	case *ast.SelectStmt:
		d.reject(stmt.Pos(), "channels", "select statements are rejected by Go subset v0")
	case *ast.SendStmt:
		d.reject(stmt.Pos(), "channels", "channel sends are rejected by Go subset v0")
	case *ast.SwitchStmt, *ast.TypeSwitchStmt:
		d.reject(stmt.Pos(), "switch statements", "switch statements are rejected by Go subset v0")
	default:
		d.reject(stmt.Pos(), "statements", fmt.Sprintf("%T statements are rejected by Go subset v0", stmt))
	}
}

func (d *featureDetector) detectLocalDecl(decl ast.Decl) {
	genDecl, ok := decl.(*ast.GenDecl)
	if !ok {
		d.reject(decl.Pos(), "declarations", "local declarations other than var declarations are rejected by Go subset v0")
		return
	}
	if genDecl.Tok != token.VAR {
		d.reject(genDecl.Pos(), "declarations", "local declarations other than var declarations are rejected by Go subset v0")
		return
	}
	for _, spec := range genDecl.Specs {
		d.detectValueSpecTypes(spec)
	}
}

func (d *featureDetector) detectValueSpecTypes(spec ast.Spec) {
	valueSpec, ok := spec.(*ast.ValueSpec)
	if !ok {
		return
	}
	for _, name := range valueSpec.Names {
		if obj, ok := d.pkg.TypesInfo.Defs[name].(*types.Var); ok {
			d.detectType(name.Pos(), obj.Type())
		}
		if obj, ok := d.pkg.TypesInfo.Defs[name].(*types.Const); ok {
			d.detectType(name.Pos(), obj.Type())
		}
	}
}

func (d *featureDetector) detectAssignment(assign *ast.AssignStmt) {
	for _, lhs := range assign.Lhs {
		ident, ok := lhs.(*ast.Ident)
		if !ok {
			d.reject(lhs.Pos(), "assignments", "assignments are limited to local variables in Go subset v0")
			continue
		}
		if ident.Name == "_" {
			continue
		}
		obj := d.pkg.TypesInfo.Uses[ident]
		if obj == nil {
			obj = d.pkg.TypesInfo.Defs[ident]
		}
		if obj == nil {
			continue
		}
		if _, ok := obj.(*types.Var); !ok || obj.Parent() == d.pkg.Types.Scope() {
			d.reject(lhs.Pos(), "assignments", "assignments are limited to local variables in Go subset v0")
		}
	}
}

func (d *featureDetector) detectExpression(expr ast.Expr) {
	if typ := d.pkg.TypesInfo.TypeOf(expr); typ != nil {
		d.detectType(expr.Pos(), typ)
	}

	switch expr := expr.(type) {
	case *ast.BasicLit:
		d.detectBasicLit(expr)
	case *ast.CallExpr:
		d.detectCall(expr)
	case *ast.ChanType:
		d.reject(expr.Pos(), "channels", "channels are rejected by Go subset v0")
	case *ast.FuncType:
		d.reject(expr.Pos(), "function values", "function values are rejected by Go subset v0")
	case *ast.IndexExpr:
		d.detectIndex(expr)
	case *ast.SliceExpr:
		d.reject(expr.Pos(), "mutable slices", "slice expressions are rejected by Go subset v0")
	case *ast.StarExpr:
		d.reject(expr.Pos(), "pointers", "pointer dereference is rejected by Go subset v0")
	case *ast.TypeAssertExpr:
		d.reject(expr.Pos(), "interfaces", "type assertions are rejected by Go subset v0")
	case *ast.UnaryExpr:
		if expr.Op == token.AND {
			d.reject(expr.Pos(), "pointers", "address-taking is rejected by Go subset v0")
		}
		if expr.Op == token.ARROW {
			d.reject(expr.Pos(), "channels", "channel receives are rejected by Go subset v0")
		}
	}
}

func (d *featureDetector) detectBasicLit(lit *ast.BasicLit) {
	switch lit.Kind {
	case token.STRING:
		d.reject(lit.Pos(), "strings", "strings are rejected by Go subset v0")
	case token.FLOAT:
		d.reject(lit.Pos(), "floating point", "floating-point numbers are rejected by Go subset v0")
	case token.IMAG:
		d.reject(lit.Pos(), "complex numbers", "complex numbers are rejected by Go subset v0")
	}
}

func (d *featureDetector) detectCall(call *ast.CallExpr) {
	if ident, ok := call.Fun.(*ast.Ident); ok {
		if builtin, ok := d.pkg.TypesInfo.Uses[ident].(*types.Builtin); ok {
			d.detectBuiltinCall(call.Pos(), builtin.Name())
			return
		}
	}

	if selector, ok := call.Fun.(*ast.SelectorExpr); ok {
		if selection := d.pkg.TypesInfo.Selections[selector]; selection != nil {
			if selection.Kind() == types.MethodVal {
				if _, ok := selection.Recv().Underlying().(*types.Interface); ok {
					d.reject(call.Pos(), "interface dynamic dispatch", "interface dynamic dispatch is rejected by Go subset v0")
				}
			}
		}
	}

	if typ := d.pkg.TypesInfo.TypeOf(call.Fun); typ != nil {
		if _, ok := typ.Underlying().(*types.Signature); ok {
			switch call.Fun.(type) {
			case *ast.Ident, *ast.SelectorExpr:
			default:
				d.reject(call.Pos(), "function values", "calls through function values are rejected by Go subset v0")
			}
		}
	}
}

func (d *featureDetector) detectBuiltinCall(pos token.Pos, name string) {
	switch name {
	case "new":
		d.reject(pos, "heap allocation", "heap allocation with new is rejected by Go subset v0")
	case "make":
		d.reject(pos, "heap allocation", "heap allocation with make is rejected by Go subset v0")
	case "append", "copy":
		d.reject(pos, "mutable slices", fmt.Sprintf("%s is rejected because mutable slices are rejected by Go subset v0", name))
	case "delete":
		d.reject(pos, "maps", "delete is rejected because maps are rejected by Go subset v0")
	case "close":
		d.reject(pos, "channels", "close is rejected because channels are rejected by Go subset v0")
	case "panic":
		d.reject(pos, "panic/recover", "panic is rejected by Go subset v0")
	case "recover":
		d.reject(pos, "panic/recover", "recover is rejected by Go subset v0")
	case "complex", "real", "imag":
		d.reject(pos, "complex numbers", fmt.Sprintf("%s is rejected because complex numbers are rejected by Go subset v0", name))
	case "print", "println":
		d.reject(pos, "runtime I/O", fmt.Sprintf("%s is rejected because runtime I/O is rejected by Go subset v0", name))
	}
}

func (d *featureDetector) detectIndex(index *ast.IndexExpr) {
	typ := d.pkg.TypesInfo.TypeOf(index.X)
	if typ == nil {
		return
	}
	switch typ.Underlying().(type) {
	case *types.Array:
	case *types.Slice:
		d.reject(index.Pos(), "mutable slices", "slice indexing is rejected because mutable slices are rejected by Go subset v0")
	case *types.Map:
		d.reject(index.Pos(), "maps", "map indexing is rejected because maps are rejected by Go subset v0")
	case *types.Basic:
		if isStringType(typ) {
			d.reject(index.Pos(), "strings", "string indexing is rejected because strings are rejected by Go subset v0")
		}
	default:
		d.reject(index.Pos(), "indexing", "only fixed-array indexing is accepted by Go subset v0")
	}
}

func (d *featureDetector) detectSignature(pos token.Pos, signature *types.Signature) {
	if signature == nil {
		return
	}
	if typeParams := signature.TypeParams(); typeParams != nil && typeParams.Len() > 0 {
		d.reject(pos, "generics", "generic functions are rejected by Go subset v0")
	}
	if receiver := signature.Recv(); receiver != nil {
		d.detectType(pos, receiver.Type())
	}
	d.detectTuple(pos, signature.Params())
	d.detectTuple(pos, signature.Results())
}

func (d *featureDetector) detectTuple(pos token.Pos, tuple *types.Tuple) {
	if tuple == nil {
		return
	}
	for i := 0; i < tuple.Len(); i++ {
		d.detectType(pos, tuple.At(i).Type())
	}
}

func (d *featureDetector) detectType(pos token.Pos, typ types.Type) {
	if feature, reason, rejected := unsupportedType(typ); rejected {
		d.reject(pos, feature, reason)
	}
}

func unsupportedType(typ types.Type) (string, string, bool) {
	if typ == nil {
		return "", "", false
	}

	switch typ := typ.(type) {
	case *types.TypeParam:
		return "generics", "type parameters are rejected by Go subset v0", true
	case *types.Named:
		if typeParams := typ.TypeParams(); typeParams != nil && typeParams.Len() > 0 {
			return "generics", "generic types are rejected by Go subset v0", true
		}
		if typeArgs := typ.TypeArgs(); typeArgs != nil && typeArgs.Len() > 0 {
			return "generics", "instantiated generic types are rejected by Go subset v0", true
		}
		return unsupportedType(typ.Underlying())
	}

	switch typ := typ.Underlying().(type) {
	case *types.Basic:
		return unsupportedBasicType(typ)
	case *types.Array:
		return unsupportedType(typ.Elem())
	case *types.Chan:
		return "channels", "channels are rejected by Go subset v0", true
	case *types.Interface:
		return "interfaces", "interfaces are rejected by Go subset v0", true
	case *types.Map:
		return "maps", "maps are rejected by Go subset v0", true
	case *types.Pointer:
		return "pointers", "pointers are rejected by Go subset v0", true
	case *types.Signature:
		return "function values", "function values are rejected by Go subset v0", true
	case *types.Slice:
		return "mutable slices", "mutable slices are rejected by Go subset v0", true
	case *types.Struct:
		for i := 0; i < typ.NumFields(); i++ {
			field := typ.Field(i)
			if field.Embedded() {
				return "embedded fields", "embedded struct fields are rejected by Go subset v0", true
			}
			if feature, reason, rejected := unsupportedType(field.Type()); rejected {
				return feature, reason, true
			}
		}
	case *types.Tuple:
		for i := 0; i < typ.Len(); i++ {
			if feature, reason, rejected := unsupportedType(typ.At(i).Type()); rejected {
				return feature, reason, true
			}
		}
	}

	return "", "", false
}

func unsupportedBasicType(typ *types.Basic) (string, string, bool) {
	switch typ.Kind() {
	case types.Bool,
		types.Int8, types.Int16, types.Int32, types.Int64,
		types.Uint8, types.Uint16, types.Uint32, types.Uint64,
		types.UntypedBool, types.UntypedInt, types.UntypedRune:
		return "", "", false
	case types.String, types.UntypedString:
		return "strings", "strings are rejected by Go subset v0", true
	case types.Float32, types.Float64, types.UntypedFloat:
		return "floating point", "floating-point numbers are rejected by Go subset v0", true
	case types.Complex64, types.Complex128, types.UntypedComplex:
		return "complex numbers", "complex numbers are rejected by Go subset v0", true
	case types.Int, types.Uint, types.Uintptr:
		return "machine-width integers", "machine-width integer types are rejected; use explicit int8/int16/int32/int64 or uint8/uint16/uint32/uint64", true
	case types.UnsafePointer:
		return "unsafe", "unsafe pointers are rejected by Go subset v0", true
	case types.UntypedNil:
		return "nil", "nil is rejected because pointer, slice, map, and channel values are not accepted", true
	default:
		if typ.Info()&types.IsString != 0 {
			return "strings", "strings are rejected by Go subset v0", true
		}
		return "", "", false
	}
}

func isStringType(typ types.Type) bool {
	basic, ok := typ.Underlying().(*types.Basic)
	return ok && basic.Info()&types.IsString != 0
}

func (d *featureDetector) rejectFile(path string, feature string, reason string) {
	d.add(rejectedFeature{
		Location: normalizePath(d.baseDir, path),
		Feature:  feature,
		Reason:   reason,
	})
}

func (d *featureDetector) reject(pos token.Pos, feature string, reason string) {
	d.add(rejectedFeature{
		Location: d.location(pos),
		Feature:  feature,
		Reason:   reason,
	})
}

func (d *featureDetector) add(finding rejectedFeature) {
	if finding.Feature == "" || finding.Reason == "" {
		return
	}
	if _, ok := d.seen[finding]; ok {
		return
	}
	d.seen[finding] = struct{}{}
	d.findings = append(d.findings, finding)
}

func (d *featureDetector) location(pos token.Pos) string {
	if d.fset == nil || !pos.IsValid() {
		return ""
	}
	position := d.fset.Position(pos)
	if !position.IsValid() {
		return ""
	}
	location := normalizePath(d.baseDir, position.Filename)
	if position.Line > 0 {
		location += fmt.Sprintf(":%d", position.Line)
	}
	if position.Column > 0 {
		location += fmt.Sprintf(":%d", position.Column)
	}
	return location
}
