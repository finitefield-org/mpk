// Private original-source facts. No observed application body/initializer runs.
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"reflect"
	"sort"
)

type admissionRequest struct {
	Source   string `json:"source"`
	Language string `json:"language"`
}
type admissionIssue struct {
	Code   string `json:"code"`
	Detail string `json:"detail"`
	Start  int    `json:"start_byte"`
	End    int    `json:"end_byte"`
}
type admissionNode struct {
	Kind  string `json:"kind"`
	Op    string `json:"op,omitempty"`
	Label string `json:"label,omitempty"`
	Start int    `json:"start_byte"`
	End   int    `json:"end_byte"`
}
type admissionFunction struct {
	ID       string           `json:"id"`
	Name     string           `json:"name"`
	Start    int              `json:"start_byte"`
	End      int              `json:"end_byte"`
	Calls    []string         `json:"calls"`
	Nodes    []admissionNode  `json:"nodes"`
	Issues   []admissionIssue `json:"issues"`
	Builtins []string         `json:"builtins"`
}

func admissionTypeIssue(raw types.Type, plain bool, trail map[types.Type]bool) string {
	if raw == nil {
		return ""
	}
	if a, ok := raw.(*types.Alias); ok && a.TypeArgs() != nil && a.TypeArgs().Len() != 0 {
		return "generic alias instantiation"
	}
	t := types.Unalias(raw)
	if t == types.Universe.Lookup("error").Type() {
		// Complete global U/recursive payload/plainness validation is a separate
		// required gate, never established by this syntax/closure observer.
		return ""
	}
	if trail[t] {
		return "recursive expanded type graph"
	}
	trail[t] = true
	defer delete(trail, t)
	switch t := t.(type) {
	case *types.Named:
		if t.TypeArgs() != nil && t.TypeArgs().Len() != 0 {
			return "generic named instantiation"
		}
		return admissionTypeIssue(t.Underlying(), plain, trail)
	case *types.Basic:
		if t.Kind() == types.UntypedNil || t.Info()&(types.IsBoolean|types.IsString) != 0 || t.Info()&types.IsInteger != 0 && t.Kind() != types.Uintptr {
			return ""
		}
	case *types.Array:
		return admissionTypeIssue(t.Elem(), plain, trail)
	case *types.Slice:
		if plain {
			return "slice nested inside a backing cell"
		}
		return admissionTypeIssue(t.Elem(), true, trail)
	case *types.Struct:
		for i := 0; i < t.NumFields(); i++ {
			if t.Field(i).Embedded() {
				return "embedded field"
			}
			if why := admissionTypeIssue(t.Field(i).Type(), plain, trail); why != "" {
				return why
			}
		}
		return ""
	}
	return "type outside closed Root/Plain candidates: " + types.TypeString(t, nil)
}

func admissionConversionAllowed(from, to types.Type) bool {
	from, to = types.Unalias(from), types.Unalias(to)
	if admissionTypeIssue(from, false, map[types.Type]bool{}) != "" || admissionTypeIssue(to, false, map[types.Type]bool{}) != "" {
		return false
	}
	if to == types.Universe.Lookup("error").Type() {
		return true // boxing/closed-U gate remains mandatory
	}
	a, b := from.Underlying(), to.Underlying()
	if aa, ok := a.(*types.Basic); ok {
		if aa.Kind() == types.UntypedNil {
			_, ok := b.(*types.Slice)
			return ok
		}
		if bb, ok := b.(*types.Basic); ok {
			for _, flag := range []types.BasicInfo{types.IsBoolean, types.IsInteger, types.IsString} {
				if aa.Info()&flag != 0 && bb.Info()&flag != 0 {
					return true
				}
			}
			// An exact untyped numeric constant may be converted by the Go
			// compiler before materialization; runtime floats remain excluded.
		}
		return false
	}
	switch a.(type) {
	case *types.Array, *types.Struct:
		return types.IdenticalIgnoreTags(a, b)
	case *types.Slice:
		return types.Identical(a, b)
	}
	return false
}

func main() {
	var in admissionRequest
	if err := json.NewDecoder(os.Stdin).Decode(&in); err != nil {
		panic(err)
	}
	digest := sha256.Sum256([]byte(in.Source))
	builtins := []string{}
	for _, name := range types.Universe.Names() {
		if _, ok := types.Universe.Lookup(name).(*types.Builtin); ok {
			builtins = append(builtins, name)
		}
	}
	out := map[string]any{"schema": "mpk.go_practical.admission_native.v0", "source_sha256": hex.EncodeToString(digest[:]), "language": in.Language, "target_sizes": "gc/amd64", "int_bits": 64, "predeclared_builtins": builtins, "functions": []admissionFunction{}, "errors": []string{}}
	finish := func(kind string) {
		out["kind"] = kind
		if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
			panic(err)
		}
	}
	fset := token.NewFileSet()
	file, err := parser.ParseFile(fset, "fixture.go", in.Source, parser.AllErrors)
	if err != nil {
		out["errors"] = []string{err.Error()}
		finish("source_error")
		return
	}
	info := &types.Info{Defs: map[*ast.Ident]types.Object{}, Uses: map[*ast.Ident]types.Object{}, Types: map[ast.Expr]types.TypeAndValue{}, Selections: map[*ast.SelectorExpr]*types.Selection{}}
	errors := []string{}
	conf := types.Config{GoVersion: in.Language, Sizes: types.SizesFor("gc", "amd64"), Error: func(e error) { errors = append(errors, e.Error()) }}
	if _, err = conf.Check("fixture/policy", fset, []*ast.File{file}, info); err != nil {
		out["errors"] = errors
		finish("type_error")
		return
	}
	position := func(p token.Pos) int { return fset.PositionFor(p, false).Offset }
	ids := map[*types.Func]string{}
	names := map[*types.Func]string{}
	for _, decl := range file.Decls {
		if fn, ok := decl.(*ast.FuncDecl); ok {
			obj := info.Defs[fn.Name].(*types.Func)
			name := fn.Name.Name
			if recv := obj.Type().(*types.Signature).Recv(); recv != nil {
				name = types.TypeString(recv.Type(), nil) + "." + name
			}
			ids[obj] = fmt.Sprintf("function:%d", position(fn.Pos()))
			names[obj] = name
		}
	}
	functions := []admissionFunction{}
	for _, decl := range file.Decls {
		fn, ok := decl.(*ast.FuncDecl)
		if !ok {
			continue
		}
		obj := info.Defs[fn.Name].(*types.Func)
		fact := admissionFunction{ID: ids[obj], Name: names[obj], Start: position(fn.Pos()), End: position(fn.End()), Calls: []string{}, Nodes: []admissionNode{}, Issues: []admissionIssue{}, Builtins: []string{}}
		issue := func(code, detail string, n ast.Node) {
			fact.Issues = append(fact.Issues, admissionIssue{code, detail, position(n.Pos()), position(n.End())})
		}
		checkType := func(t types.Type, n ast.Node) {
			if why := admissionTypeIssue(t, false, map[types.Type]bool{}); why != "" {
				issue("TYPE_NOT_ROOT", why, n)
			}
		}
		sig := obj.Type().(*types.Signature)
		if sig.TypeParams().Len() != 0 || sig.RecvTypeParams().Len() != 0 || sig.Variadic() {
			issue("FUNCTION_SIGNATURE", "generic/variadic function", fn.Type)
		}
		if sig.Results().Len() > 16 {
			issue("RESULT_ARITY", "more than sixteen ordered results", fn.Type)
		}
		if sig.Recv() != nil {
			checkType(sig.Recv().Type(), fn.Type)
		}
		for _, tuple := range []*types.Tuple{sig.Params(), sig.Results()} {
			for i := 0; i < tuple.Len(); i++ {
				checkType(tuple.At(i).Type(), fn.Type)
			}
		}
		if fn.Body == nil {
			issue("MISSING_BODY", "no captured source body or registered binding", fn)
		}
		callOperands := map[ast.Expr]bool{}
		ast.Inspect(fn, func(n ast.Node) bool {
			if call, ok := n.(*ast.CallExpr); ok {
				for e := call.Fun; ; {
					callOperands[e] = true
					if p, ok := e.(*ast.ParenExpr); ok {
						e = p.X
						continue
					}
					if p, ok := e.(*ast.IndexExpr); ok {
						e = p.X
						continue
					}
					if p, ok := e.(*ast.IndexListExpr); ok {
						e = p.X
						continue
					}
					if s, ok := e.(*ast.SelectorExpr); ok {
						callOperands[s.Sel] = true
					}
					break
				}
			}
			return true
		})
		ast.Inspect(fn, func(n ast.Node) bool {
			if n == nil {
				return true
			}
			node := admissionNode{Kind: reflect.TypeOf(n).Elem().Name(), Start: position(n.Pos()), End: position(n.End())}
			if e, ok := n.(ast.Expr); ok {
				if tv, exists := info.Types[e]; exists && !tv.IsType() {
					_, tuple := tv.Type.(*types.Tuple)
					_, signature := tv.Type.(*types.Signature)
					untypedConstant := false
					if b, ok := tv.Type.(*types.Basic); ok {
						untypedConstant = tv.Value != nil && b.Info()&types.IsUntyped != 0
					}
					if !tuple && !signature && !untypedConstant {
						checkType(tv.Type, n)
					}
				}
			}
			switch n := n.(type) {
			case *ast.UnaryExpr:
				node.Op = n.Op.String()
			case *ast.BranchStmt:
				node.Op = n.Tok.String()
				if n.Label != nil {
					node.Label = n.Label.Name
				}
			case *ast.RangeStmt:
				switch info.TypeOf(n.X).Underlying().(type) {
				case *types.Array, *types.Slice:
				default:
					issue("RANGE_KIND", "only array/slice range", n)
				}
			case *ast.SelectorExpr:
				if selection := info.Selections[n]; selection != nil && (selection.Indirect() || len(selection.Index()) != 1) {
					issue("SELECTOR_PATH", "pointer or promoted selection", n)
				}
			case *ast.Ident:
				if _, ok := info.Uses[n].(*types.Func); ok && !callOperands[n] {
					issue("FUNCTION_VALUE", "function/method value outside direct call", n)
				}
				if v, ok := info.Uses[n].(*types.Var); ok && v.Pkg() != nil && v.Parent() == v.Pkg().Scope() {
					issue("AMBIENT_STATE", v.Name(), n)
				}
				if v, ok := info.Defs[n].(*types.Var); ok {
					checkType(v.Type(), n)
				}
				if c, ok := info.Uses[n].(*types.Const); ok && c.Type().Underlying().(*types.Basic).Info()&types.IsUntyped == 0 {
					checkType(c.Type(), n)
				}
			case *ast.CallExpr:
				if tv := info.Types[n.Fun]; tv.IsType() {
					from, to := info.TypeOf(n.Args[0]), tv.Type
					constantWord := false
					if a, ok := from.Underlying().(*types.Basic); ok {
						if b, ok := to.Underlying().(*types.Basic); ok {
							constantWord = a.Info()&types.IsUntyped != 0 && a.Info()&types.IsNumeric != 0 && b.Info()&types.IsInteger != 0
						}
					}
					if constantWord && info.Types[n.Args[0]].Value != nil && info.Types[n].Value != nil {
						checkType(to, n)
					} else if !admissionConversionAllowed(from, to) {
						issue("CONVERSION_KIND", "outside GP-OP-03 whitelist", n)
					}
					break
				}
				var called types.Object
				callee := ast.Unparen(n.Fun)
			unwrap:
				for {
					switch f := callee.(type) {
					case *ast.IndexExpr:
						callee = ast.Unparen(f.X)
					case *ast.IndexListExpr:
						callee = ast.Unparen(f.X)
					default:
						break unwrap
					}
				}
				switch f := callee.(type) {
				case *ast.Ident:
					called = info.Uses[f]
				case *ast.SelectorExpr:
					called = info.Uses[f.Sel]
					if selection := info.Selections[f]; selection != nil {
						if _, ok := selection.Recv().Underlying().(*types.Interface); ok {
							issue("INTERFACE_DISPATCH", "open error/interface method dispatch", n)
							called = nil
						}
					}
				}
				switch called := called.(type) {
				case *types.Builtin:
					fact.Builtins = append(fact.Builtins, called.Name())
					if called.Name() == "make" {
						if _, ok := info.TypeOf(n).Underlying().(*types.Slice); !ok {
							issue("MAKE_KIND", "only slice make", n)
						}
					}
				case *types.Func:
					if called.Type().(*types.Signature).TypeParams().Len() != 0 {
						issue("GENERIC_CALL", "generic instantiation outside v1", n)
					}
					if id, ok := ids[called.Origin()]; ok {
						fact.Calls = append(fact.Calls, id)
					} else {
						issue("UNBOUND_CALLEE", called.FullName(), n)
					}
				default:
					issue("INDIRECT_CALL", "not a direct captured function/method", n)
				}
			case *ast.BinaryExpr:
				isError := func(e ast.Expr) bool { return types.Unalias(info.TypeOf(e)) == types.Universe.Lookup("error").Type() }
				isNil := func(e ast.Expr) bool { return info.TypeOf(e) == types.Typ[types.UntypedNil] }
				if (n.Op == token.EQL || n.Op == token.NEQ) && (isError(n.X) || isError(n.Y)) && !isNil(n.X) && !isNil(n.Y) {
					issue("ERROR_COMPARISON", "error may be compared only with nil", n)
				}
			}
			fact.Nodes = append(fact.Nodes, node)
			return true
		})
		sort.Strings(fact.Calls)
		sort.Strings(fact.Builtins)
		functions = append(functions, fact)
	}
	out["functions"] = functions
	finish("typed_original_package")
}
