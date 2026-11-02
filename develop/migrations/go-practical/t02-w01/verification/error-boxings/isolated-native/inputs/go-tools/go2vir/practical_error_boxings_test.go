//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

// The reference observes intact owned native packages. No native type, expected
// target list or AST is passed to the independent Rust source constructor.
func practicalNativeBoxType(t *testing.T, owner *practicalOriginalPackages, typ types.Type) string {
	t.Helper()
	named, ok := types.Unalias(typ).(*types.Named)
	if !ok {
		t.Fatalf("fixture boxing has no original named payload: %v", typ)
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	storage := newPracticalStorageTypes(owner)
	index, err := storage.declaration(named.Obj())
	if err != nil {
		t.Fatal(err)
	}
	d := storage.declarations[index]
	b := practicalTypeKeyBuilder{owner: owner, sources: map[string]practicalSourceBuffer{}, packages: map[string]map[string]any{}, packageIDs: map[string]string{}}
	pkg, err := b.packageKey(d.packagePath)
	if err != nil {
		t.Fatal(err)
	}
	declaration, err := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": pkg, "source_object_id": d.span.sourceID, "kind": "type", "start_byte": d.span.start, "end_byte": d.span.end, "name": d.name})
	if err != nil {
		t.Fatal(err)
	}
	key, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "nominal", "declaration_id": declaration["id"]})
	if err != nil {
		t.Fatal(err)
	}
	return key["id"].(string)
}
func practicalBoxingInput(t *testing.T, owner *practicalOriginalPackages) map[string]any {
	t.Helper()
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	b := practicalTypeKeyBuilder{owner: owner, sources: map[string]practicalSourceBuffer{}, packages: map[string]map[string]any{}, packageIDs: map[string]string{}}
	roots := []string{}
	for _, path := range []string{"example.com/policy", "example.com/policy/helper"} {
		id, err := b.packageKey(path)
		if err != nil {
			t.Fatal(err)
		}
		roots = append(roots, id)
	}
	sort.Strings(roots)
	objects := []practicalSourceObject{}
	sources := []map[string]any{}
	for _, source := range b.sources {
		objects = append(objects, source.object)
		sources = append(sources, map[string]any{"object": source.object, "raw": source.sourceBytes()})
	}
	sort.Slice(objects, func(i, j int) bool { return objects[i].ID < objects[j].ID })
	sort.Slice(sources, func(i, j int) bool {
		return sources[i]["object"].(practicalSourceObject).ID < sources[j]["object"].(practicalSourceObject).ID
	})
	boolKey, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": "bool", "width": nil, "signed": nil})
	if err != nil {
		t.Fatal(err)
	}
	c := map[string]any{"schema": practicalTypeComponentsSchema, "source_objects": objects, "packages": practicalTypeKeyRows(b.packages), "declarations": []any{}, "types": []any{boolKey}, "underlying": []any{}, "aliases": []any{}, "package_roots": roots, "type_roots": []any{boolKey["id"]}}
	hash, err := hashTypedCanonicalJSON(practicalTypeComponentsDomain, c)
	if err != nil {
		t.Fatal(err)
	}
	c["content_sha256"] = hash
	raw, err := canonicalJSON(c)
	if err != nil {
		t.Fatal(err)
	}
	return map[string]any{"canonical_components": raw, "sources": sources}
}
func practicalNativeErrorBoxings(t *testing.T, owner *practicalOriginalPackages, ref practicalClosureRef, runtimeOnly bool) []map[string]any {
	t.Helper()
	loaded := owner.loader.packages[ref.packagePath]
	info := loaded.info
	var function *ast.FuncDecl
	for _, file := range loaded.files {
		for _, decl := range file.Decls {
			if f, ok := decl.(*ast.FuncDecl); ok {
				span, _, err := owner.loader.syntax.spanBytes(f)
				if err != nil {
					t.Fatal(err)
				}
				if span == ref.span {
					function = f
				}
			}
		}
	}
	if function == nil || function.Body == nil {
		t.Fatal("missing owned body")
	}
	errorType := types.Universe.Lookup("error").Type()
	signature := info.Defs[function.Name].(*types.Func).Type().(*types.Signature)
	sites := map[string]map[string]any{}
	record := func(expr ast.Expr, projection any, from, to types.Type) {
		if from == nil || to == nil || !types.Identical(to, errorType) || types.Identical(from, errorType) || types.Identical(from, types.Typ[types.UntypedNil]) {
			return
		}
		if !types.AssignableTo(from, to) {
			t.Fatal("native invalid boxing", from, to)
		}
		span, _, err := owner.loader.syntax.spanBytes(expr)
		if err != nil {
			t.Fatal(err)
		}
		site := map[string]any{"source": span.sourceID, "start": span.start, "end": span.end, "projection": projection, "type_id": practicalNativeBoxType(t, owner, from)}
		raw, err := canonicalJSON(site)
		if err != nil {
			t.Fatal(err)
		}
		sites[string(raw)] = site
	}
	arguments := func(args []ast.Expr, targets []types.Type) {
		actual := []types.Type{}
		origins := []ast.Expr{}
		slots := []int{}
		for _, arg := range args {
			typ := info.TypeOf(arg)
			if tuple, ok := typ.(*types.Tuple); ok {
				for i := 0; i < tuple.Len(); i++ {
					actual = append(actual, tuple.At(i).Type())
					origins = append(origins, arg)
					slots = append(slots, i)
				}
			} else {
				actual = append(actual, typ)
				origins = append(origins, arg)
				slots = append(slots, 0)
			}
		}
		if len(actual) != len(targets) {
			t.Fatal("native boxing arity", len(actual), len(targets))
		}
		for i, from := range actual {
			record(origins[i], slots[i], from, targets[i])
		}
	}
	tupleTypes := func(tuple *types.Tuple) []types.Type {
		out := []types.Type{}
		for i := 0; i < tuple.Len(); i++ {
			out = append(out, tuple.At(i).Type())
		}
		return out
	}
	type task struct {
		node    ast.Node
		runtime bool
	}
	work := []task{{function.Body, true}}
	for len(work) > 0 {
		item := work[len(work)-1]
		work = work[:len(work)-1]
		node := item.node
		if expr, ok := node.(ast.Expr); ok && info.Types[expr].IsType() {
			item.runtime = false
		}
		if !runtimeOnly || item.runtime {
			switch n := node.(type) {
			case *ast.ValueSpec:
				if len(n.Values) > 0 {
					targets := []types.Type{}
					for _, id := range n.Names {
						targets = append(targets, info.Defs[id].Type())
					}
					arguments(n.Values, targets)
				}
			case *ast.AssignStmt:
				if n.Tok == token.ASSIGN || n.Tok == token.DEFINE {
					targets := []types.Type{}
					for _, lhs := range n.Lhs {
						if id, ok := lhs.(*ast.Ident); ok && id.Name == "_" {
							targets = append(targets, nil)
						} else {
							targets = append(targets, info.TypeOf(lhs))
						}
					}
					arguments(n.Rhs, targets)
				}
			case *ast.ReturnStmt:
				if len(n.Results) > 0 {
					arguments(n.Results, tupleTypes(signature.Results()))
				}
			case *ast.RangeStmt:
				if n.Tok == token.ASSIGN && n.Value != nil {
					var element types.Type
					switch typ := info.TypeOf(n.X).Underlying().(type) {
					case *types.Array:
						element = typ.Elem()
					case *types.Slice:
						element = typ.Elem()
					}
					record(n.X, "range_element", element, info.TypeOf(n.Value))
				}
			case *ast.CompositeLit:
				switch typ := info.TypeOf(n).Underlying().(type) {
				case *types.Array:
					for _, e := range n.Elts {
						if kv, ok := e.(*ast.KeyValueExpr); ok {
							e = kv.Value
						}
						record(e, 0, info.TypeOf(e), typ.Elem())
					}
				case *types.Slice:
					for _, e := range n.Elts {
						if kv, ok := e.(*ast.KeyValueExpr); ok {
							e = kv.Value
						}
						record(e, 0, info.TypeOf(e), typ.Elem())
					}
				case *types.Struct:
					for i, e := range n.Elts {
						field := i
						if kv, ok := e.(*ast.KeyValueExpr); ok {
							e = kv.Value
							name := kv.Key.(*ast.Ident).Name
							for j := 0; j < typ.NumFields(); j++ {
								if typ.Field(j).Name() == name {
									field = j
									break
								}
							}
						}
						record(e, 0, info.TypeOf(e), typ.Field(field).Type())
					}
				}
			case *ast.CallExpr:
				if info.Types[n.Fun].IsType() {
					arguments(n.Args, []types.Type{info.TypeOf(n.Fun)})
				} else if id, ok := n.Fun.(*ast.Ident); ok && info.Uses[id] == types.Universe.Lookup("append") {
					if !n.Ellipsis.IsValid() {
						actual := []types.Type{}
						for _, arg := range n.Args {
							if tuple, ok := info.TypeOf(arg).(*types.Tuple); ok {
								actual = append(actual, tupleTypes(tuple)...)
							} else {
								actual = append(actual, info.TypeOf(arg))
							}
						}
						slice := actual[0].Underlying().(*types.Slice)
						targets := []types.Type{actual[0]}
						for range actual[1:] {
							targets = append(targets, slice.Elem())
						}
						arguments(n.Args, targets)
					}
				} else if sig, ok := info.TypeOf(n.Fun).(*types.Signature); ok && !sig.Variadic() {
					arguments(n.Args, tupleTypes(sig.Params()))
				}
			}
		}
		if n, ok := node.(*ast.RangeStmt); ok {
			work = append(work, task{n.X, item.runtime && practicalClosureRangeEvaluated(info, n)}, task{n.Body, item.runtime})
			if n.Tok != token.DEFINE {
				if n.Key != nil {
					work = append(work, task{n.Key, item.runtime})
				}
				if n.Value != nil {
					work = append(work, task{n.Value, item.runtime})
				}
			}
			continue
		}
		if n, ok := node.(*ast.CallExpr); ok {
			if id, ok := n.Fun.(*ast.Ident); ok {
				if builtin, ok := info.Uses[id].(*types.Builtin); ok && (builtin.Name() == "len" || builtin.Name() == "cap") && info.Types[n].Value != nil {
					for _, arg := range n.Args {
						work = append(work, task{arg, false})
					}
					continue
				}
			}
		}
		for _, child := range practicalClosureChildren(node) {
			work = append(work, task{child, item.runtime})
		}
	}
	keys := []string{}
	for key := range sites {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	out := []map[string]any{}
	for _, key := range keys {
		out = append(out, sites[key])
	}
	return out
}
func TestPracticalErrorBoxingsOriginalContexts(t *testing.T) {
	if path := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); path != "" {
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", filepath.Join(path, "error-boxings"))
	}
	policy := `package policy
import h "example.com/policy/helper"
type Code int64
type Alias=Code
type AliasError=error
const Value Code=1
func(Code)Error()string{panic("boxing must not call Error")}
type R struct{}
func(R)Take(e error){_=e}
type S struct{Err error}
type Outer struct{Inner S;Values [1]error}
var GlobalErrors=[1]error{h.Other(0)}
func Pair()(Code,Code){return 1,2}
func Produce()error{return h.Other(0)}
func GetCode()Code{return 0}
func Take(e error){_=e}
func TakeTwo(a,b error){_=a;_=b}
func ReceiverArgs()(R,Code){return R{},0}
func AppendArgs()([]error,Code){return nil,0}
func Basic(x Code)error{return x}
func Aliased(x Alias)AliasError{return x}
func Constant()error{return Value}
func Nil()error{return nil}
func Convert()error{return error((Code(0)))}
func Rebox(e error)error{return error(e)}
func Assign(){var e error;e=Code(1);_=e}
func Declared(){var e error=Code(1);_=e}
func Named()(out error){out=Code(1);return}
func TupleReturn()(error,error){return Pair()}
func TupleDeclared(){var x,y error=Pair();_,_=x,y}
func TupleAssign(){var x,y error;x,y=Pair();_,_=x,y}
func TupleCall(){TakeTwo(Pair())}
func SingleCall(){Take(Code(1))}
func Method(r R){r.Take(Code(1))}
func MethodExpression(r R){R.Take(r,Code(1))}
func MethodTuple(){R.Take(ReceiverArgs())}
func Append(s []error)[]error{return append(s,Code(1),Code(2))}
func AppendTuple()[]error{return append(AppendArgs())}
func Spread(s,t []error)[]error{return append(s,t...)}
func StructLit()S{return S{Err:Code(1)}}
func ArrayLit()[2]error{return [2]error{Code(1),nil}}
func SliceLit()[]error{return []error{Code(1)}}
func Nested()Outer{return Outer{Inner:S{Err:Code(1)},Values:[1]error{Code(2)}}}
func FieldAssign(s S)S{s.Err=Code(1);return s}
func IndexAssign(s []error){s[0]=Code(1)}
func ReusedShort(){var e error;e,n:=Code(1),0;_=e;_=n}
func BlankTyped(){var _ error=Code(1)}
func BlankDefault(){_=Code(1)}
func BlankTuple(){var e error;e,_=Pair();_=e}
func ConstLen(){const N=len([1]error{Code(1)});_=N}
func DirectLen()int{return len([1]error{Code(1)})}
func NonconstantLen()int{return len([1]error{GetCode()})}
func TypeLength(x [len([1]error{Code(1)})]Code)Code{return x[0]}
func GlobalLen()int{return len(GlobalErrors)}
func Branches(flag bool)error{if flag{return Code(1)};return h.Other(2)}
func HelperOnly()error{return Produce()}
func Unreachable(){if false{Take(h.Other(3))}}
func RangeZero(){for range [1]error{Code(1)}{}}
func RangeOne(){for i:=range [1]error{Code(1)}{_=i}}
func RangeBlank(){for i,_:=range [1]error{Code(1)}{_=i}}
func RangeTwo(){for i,e:=range [1]error{Code(1)}{_=i;_=e}}
func RangeAssign(){var e error;for _,e=range [1]Code{1}{_=e}}
func GlobalRange(){for range GlobalErrors{}}
func UnusedOther()error{return h.Other(0)}
`
	names := []string{"Basic", "Aliased", "Constant", "Nil", "Convert", "Rebox", "Assign", "Declared", "Named", "TupleReturn", "TupleDeclared", "TupleAssign", "TupleCall", "SingleCall", "Method", "MethodExpression", "MethodTuple", "Append", "AppendTuple", "Spread", "StructLit", "ArrayLit", "SliceLit", "Nested", "FieldAssign", "IndexAssign", "ReusedShort", "BlankTyped", "BlankDefault", "BlankTuple", "ConstLen", "DirectLen", "NonconstantLen", "TypeLength", "GlobalLen", "Branches", "HelperOnly", "Unreachable", "RangeZero", "RangeOne", "RangeBlank", "RangeTwo", "RangeAssign", "GlobalRange"}
	params, args := []string{}, []string{}
	for n := 0; n < 70; n++ {
		params = append(params, fmt.Sprintf("e%d error", n))
		args = append(args, "Code(0)")
	}
	policy += "func Many(" + strings.Join(params, ",") + "){"
	for n := 0; n < 70; n++ {
		policy += fmt.Sprintf("_=e%d;", n)
	}
	policy += "}\nfunc ManyCaller(){Many(" + strings.Join(args, ",") + ")}\n"
	names = append(names, "ManyCaller")
	results, values := []string{}, []string{}
	for range 16 {
		results = append(results, "Code")
		values = append(values, "0")
	}
	policy += "func WideCode()(" + strings.Join(results, ",") + "){return " + strings.Join(values, ",") + "}\n"
	for n := range results {
		results[n] = "error"
	}
	policy += "func Wide()(" + strings.Join(results, ",") + "){return WideCode()}\n"
	names = append(names, "Wide")
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(strings.ReplaceAll(policy, "\n", "\r\n")), "helper/helper.go": []byte("package helper\ntype Other uint8\nfunc(Other)Error()string{panic(\"imported Error must stay type-only\")}\n")}
	owner, _ := practicalStorageTestOwner(t, files)
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "boxings-inputs.json", practicalBoxingInput(t, owner))
	for index, name := range names {
		t.Run(name, func(t *testing.T) {
			root := practicalClosureTestRef(t, owner, "example.com/policy", name)
			closure, err := owner.sourceClosure([]practicalClosureRef{root})
			if err != nil {
				t.Fatal(err)
			}
			if len(closure.issues) != 0 {
				t.Fatal("unexpected source closure issues", practicalClosureEvidence(closure)["issues"])
			}
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("boxing-%02d-input.json", index), map[string]any{"name": name, "root": practicalClosureEvidence(closure)["roots"]})
			typed := practicalNativeErrorBoxings(t, owner, root, false)
			runtime := []map[string]any{}
			seen := map[string]bool{}
			for _, decl := range closure.declarations {
				body := false
				for _, role := range decl.roles {
					body = body || role == "selected_root" || role == "body_dependency"
				}
				if decl.kind == "function" && body {
					for _, site := range practicalNativeErrorBoxings(t, owner, decl.ref, true) {
						raw, _ := canonicalJSON(site)
						if !seen[string(raw)] {
							seen[string(raw)] = true
							runtime = append(runtime, site)
						}
					}
				}
			}
			sort.Slice(runtime, func(i, j int) bool {
				a, _ := canonicalJSON(runtime[i])
				b, _ := canonicalJSON(runtime[j])
				return string(a) < string(b)
			})
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("boxing-%02d-actual.json", index), map[string]any{"typed": typed, "body": runtime, "closure": practicalClosureEvidence(closure), "closed_global_error_universe_or_source_program_proof": false})
		})
	}
	retainPracticalSyntaxEvidence(t, "boxings-controls.json", map[string]any{"cases": len(names), "parameters": 70, "result_slots": 16, "application_initializers_or_Error_bodies_executed": false})
}
