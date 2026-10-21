//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/types"
	"reflect"
	"sort"
	"strings"
	"testing"
)

func practicalClosureEvidence(c practicalSourceClosure) map[string]any {
	ref := func(r practicalClosureRef) map[string]any {
		return map[string]any{"package": r.packagePath, "source": r.span.sourceID, "start": r.span.start, "end": r.span.end}
	}
	refs := func(v []practicalClosureRef) []any {
		out := []any{}
		for _, r := range v {
			out = append(out, ref(r))
		}
		return out
	}
	decls := []any{}
	for _, d := range c.declarations {
		decls = append(decls, map[string]any{"ref": ref(d.ref), "module_source_id": d.moduleSourceID, "kind": d.kind, "names": append([]string(nil), d.names...), "roles": append([]string(nil), d.roles...)})
	}
	calls := []any{}
	for _, call := range c.calls {
		calls = append(calls, map[string]any{"caller": ref(call.caller), "callee": ref(call.callee), "source": call.span.sourceID, "start": call.span.start, "end": call.span.end})
	}
	dependencies := []any{}
	for _, d := range c.dependencies {
		dependencies = append(dependencies, map[string]any{"from": ref(d.from), "to": ref(d.to), "role": d.role})
	}
	issues := []any{}
	for _, issue := range c.issues {
		issues = append(issues, map[string]any{"code": issue.code, "declaration": ref(issue.declaration), "source": issue.span.sourceID, "start": issue.span.start, "end": issue.span.end, "path": refs(issue.path)})
	}
	sources := []any{}
	for _, s := range c.sources {
		sources = append(sources, map[string]any{"object": s.object, "raw": s.sourceBytes()})
	}
	return map[string]any{"image_digest": c.imageDigest, "compiler_digest": c.compilerDigest, "main_source_id": c.mainSourceID, "roots": refs(c.roots), "declarations": decls, "calls": calls, "dependencies": dependencies, "issues": issues, "sources": sources, "source_admission_or_execution_or_program_proof": false}
}
func practicalClosureTestRef(t *testing.T, owner *practicalOriginalPackages, pkg, name string) practicalClosureRef {
	t.Helper()
	var match *ast.FuncDecl
	for _, file := range owner.loader.packages[pkg].files {
		for _, d := range file.Decls {
			if f, ok := d.(*ast.FuncDecl); ok && f.Name.Name == name {
				if match != nil {
					t.Fatal("ambiguous fixture function", name)
				}
				match = f
			}
		}
	}
	if match == nil {
		t.Fatal("missing fixture function", name)
	}
	// Independent physical token positions, not the production closure index.
	f := owner.loader.syntax.fileSet.File(match.Pos())
	source := strings.TrimSuffix(f.Name(), ".go")
	return practicalClosureRef{pkg, practicalSourceSpan{source, f.Offset(match.Pos()), f.Offset(match.End())}}
}
func TestPracticalSourceClosureRequiresOriginalOwner(t *testing.T) {
	for _, owner := range []*practicalOriginalPackages{nil, {}} {
		result, err := owner.sourceClosure([]practicalClosureRef{{"example.com/policy", practicalSourceSpan{}}})
		if err == nil || len(result.declarations) != 0 {
			t.Fatal("unowned original closure accepted", err)
		}
	}
}
func TestPracticalSourceClosureOriginalPackagesAndStaticPaths(t *testing.T) {
	policy := `package policy
import (h "example.com/policy/helper";"unsafe")
type W int64
type Alias=W
type Code uint8
func(Code)Error()string{panic("error body must remain outside proof unless called")}
type Embedded struct{h.Record}
type Pointer uint8
func(*Pointer)Use()int64{return 1}
const N=h.N
const Shadow=3
var Counter int64
var GlobalArray [3]byte
type BoundArray [4]W
var Inferred=NewArray()
func NewArray()BoundArray{return BoundArray{}}
func InferredLen()int{return len(Inferred)}
func InferredLocal()int{a:=NewArray();return len(a)}
var Startup=Start()
func Start()int64{panic("application variable initializer must not execute")}
func init(){panic("application initialization must not execute")}
func Handler(c chan int,m map[int]int,f func(int)int)int{return <-c+m[0]+f(int(unsafe.Sizeof(m)))}
func Pure(x int64)int64{return h.Sub(x)+int64(N)}
func ErrorOnly()error{return Code(1)}
func CallError(c Code)string{return c.Error()}
func Dispatch(e error)string{return e.Error()}
func Method(r h.Record)int64{return r.Sum()}
func MethodExpression(r h.Record)int64{return h.Record.Sum(r)}
func Promoted(r Embedded)int64{return r.Sum()}
func PointerCall(p Pointer)int64{return p.Use()}
func MethodValue(r h.Record)int64{f:=r.Sum;return f()}
func Indirect(f func(int64)int64)int64{return f(1)}
func GlobalRead()int64{return Counter}
func GlobalWrite(){Counter=1}
func GlobalConstant()int{return len(GlobalArray)}
func GlobalIgnoredElement()int{return len([1]int64{Counter})}
func GlobalNonconstant()int{return len([1]int64{GlobalRead()})}
func BadHelper(){defer Pure(1)}
func BranchRoot(){if false{BadHelper()}}
func Self(){Self()}
func A(){B()}
func B(){A()}
func CycleRoot(){A()}
//go:linkname Opaque
func Opaque(x int64)int64
func OpaqueRoot(x int64)int64{return Opaque(x)}
func Generic[T ~int64](x T)T{return x}
func GenericRoot(x int64)int64{return Generic(x)}
func Min(a,b int64)int64{return min(a,b)}
func Local(){const(A,B=len([iota+1]byte{}),1;_,_);type L [A]byte;var x L;_=len(x);_=B}
func Context(){const(A=len([Shadow]byte{});Shadow;Result);_=Result}
func ContextBlank(){const(_=len([Shadow]byte{});Shadow;Result);_=Result}
func StringRange(s string){for range s{}}
func Slice(s []int64)int64{var result int64;for _,x:=range s{result+=x};return result}
func Spread(s []int64)[]int64{return append(s,s...)}
func TooMany()(int,int,int,int,int,int,int,int,int,int,int,int,int,int,int,int,int){return 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0}
`
	params, args := []string{}, []string{}
	for n := 0; n < 70; n++ {
		params = append(params, fmt.Sprintf("p%d int64", n))
		args = append(args, "1")
	}
	policy += "func Many(" + strings.Join(params, ",") + ")int64{return p69}\nfunc ManyRoot()int64{return Many(" + strings.Join(args, ",") + ")}\n"
	for n := 0; n < 512; n++ {
		body := "return x"
		if n < 511 {
			body = fmt.Sprintf("return Chain%d(x)", n+1)
		}
		policy += fmt.Sprintf("func Chain%d(x int64)int64{%s}\n", n, body)
	}
	for n := 0; n < 70; n++ {
		policy += fmt.Sprintf("func Root%d(x int64)int64{return h.Sub(x)}\n", n)
	}
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(strings.ReplaceAll(policy, "\n", "\r\n")), "helper/helper.go": []byte("package helper\nconst N=3\ntype Record struct{Value int64}\nfunc(r Record)Sum()int64{return r.Value}\nfunc Sub(x int64)int64{return x-1}\nfunc Unused(f func(int)int)int{return f(1)}\n")}
	owner, originals := practicalStorageTestOwner(t, files)
	tests := []struct {
		name  string
		codes []string
	}{
		{"Pure", nil}, {"ErrorOnly", nil}, {"Method", nil}, {"MethodExpression", nil}, {"GlobalConstant", nil}, {"GlobalIgnoredElement", nil}, {"Local", nil}, {"Slice", nil}, {"Spread", nil}, {"Context", nil}, {"ContextBlank", nil}, {"InferredLen",nil},{"InferredLocal",nil},{"ManyRoot", nil}, {"Chain0", nil},
		{"CallError", []string{"builtin_panic"}}, {"Dispatch", []string{"indirect_call"}}, {"Promoted", []string{"promoted_selector"}}, {"PointerCall", []string{"syntax_*ast.StarExpr"}}, {"MethodValue", []string{"function_value", "indirect_call"}}, {"Indirect", []string{"function_value_type", "indirect_call"}}, {"GlobalRead", []string{"mutable_package_state"}}, {"GlobalWrite", []string{"mutable_package_state"}}, {"GlobalNonconstant", []string{"mutable_package_state"}}, {"BranchRoot", []string{"syntax_*ast.DeferStmt"}}, {"Self", []string{"call_cycle"}}, {"CycleRoot", []string{"call_cycle"}}, {"OpaqueRoot", []string{"missing_body"}}, {"GenericRoot", []string{"generic_declaration"}}, {"Min", []string{"builtin_min"}}, {"TooMany", []string{"result_limit"}}, {"StringRange", []string{"range_kind"}}, {"init", []string{"initialization_root", "builtin_panic"}},
	}
	var saved practicalSourceClosure
	for index, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			root := practicalClosureTestRef(t, owner, "example.com/policy", test.name)
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("closure-%02d-input.json", index), map[string]any{"root": map[string]any{"package": root.packagePath, "source": root.span.sourceID, "start": root.span.start, "end": root.span.end}, "expected_issue_codes": test.codes})
			result, err := owner.sourceClosure([]practicalClosureRef{root})
			if err != nil {
				t.Fatal(err)
			}
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("closure-%02d-actual.json", index), practicalClosureEvidence(result))
			codes := map[string]bool{}
			for _, issue := range result.issues {
				codes[issue.code] = true
				if len(issue.path) == 0 || issue.path[0] != root || issue.path[len(issue.path)-1] != issue.declaration && issue.code != "call_cycle" {
					t.Fatal("issue lost its physical root-to-declaration path", issue)
				}
			}
			if len(test.codes) == 0 && len(codes) != 0 {
				t.Fatal("pure selected source has issues", codes)
			}
			for _, code := range test.codes {
				if !codes[code] {
					t.Fatal("missing required transitive issue", code, codes)
				}
			}
			for _, call := range result.calls {
				if call.span.sourceID != call.caller.span.sourceID || call.span.start < call.caller.span.start || call.span.end > call.caller.span.end {
					t.Fatal("call crossed physical caller bounds")
				}
			}
			if test.name == "BranchRoot" {
				for _, issue := range result.issues {
					if issue.code == "syntax_*ast.DeferStmt" && len(issue.path) != 2 {
						t.Fatal("unreachable helper omitted root path")
					}
				}
			}
			if test.name == "ErrorOnly" {
				found := false
				for _, d := range result.declarations {
					if d.kind == "function" && reflect.DeepEqual(d.names, []string{"Error"}) {
						found = true
						if !reflect.DeepEqual(d.roles, []string{"signature_dependency"}) {
							t.Fatal("error compatibility pulled the Error body into proof", d)
						}
					}
				}
				if !found {
					t.Fatal("missing concrete Error signature dependency")
				}
			}
			if test.name == "Context" || test.name == "ContextBlank" {
				global, local := false, false
				for _, d := range result.declarations {
					if d.kind != "const" || !reflect.DeepEqual(d.names, []string{"Shadow"}) {
						continue
					}
					for _, role := range d.roles {
						if role == "constant_dependency" {
							if d.ref.span.start < root.span.start {
								global = true
							} else {
								local = true
							}
						}
					}
				}
				if !global || !local {
					t.Fatal("inherited expression lost an actual outer/local shadow dependency", global, local)
				}
			}
			if test.name == "GlobalRead" || test.name == "GlobalWrite" {
				found := false
				for _, d := range result.declarations {
					if d.kind == "var" && reflect.DeepEqual(d.names, []string{"Counter"}) {
						for _, role := range d.roles {
							found = found || role == "rejected_dependency"
						}
					}
				}
				if !found {
					t.Fatal("reachable mutable variable remained type-check-only")
				}
			}
			if test.name == "GlobalConstant" {
				found := false
				for _, d := range result.declarations {
					if d.kind == "var" && reflect.DeepEqual(d.names, []string{"GlobalArray"}) {
						found = reflect.DeepEqual(d.roles, []string{"type_dependency"})
					}
				}
				if !found {
					t.Fatal("non-evaluated array operand lost its type-only dependency")
				}
			}
			if test.name == "Chain0" {
				if len(result.calls) != 511 {
					t.Fatal("flat helper closure was truncated", len(result.calls))
				}
			}
			if test.name == "Pure" {
				saved = result
				found := false
				for _, s := range result.sources {
					if s.object == originals["policy.go"].object && bytes.Equal(s.data, files["policy.go"]) {
						found = true
					}
				}
				if !found {
					t.Fatal("whole unused handlers/initializers not retained")
				}
			}
			inventory := map[practicalClosureRef]bool{}
			for _, d := range result.declarations {
				inventory[d.ref] = true
			}
			dependencySet := map[practicalClosureDependency]bool{}
			for _, d := range result.dependencies {
				if !inventory[d.from] || !inventory[d.to] || dependencySet[d] {
					t.Fatal("dependency is duplicated or outside original inventory", d)
				}
				dependencySet[d] = true
			}
			for _, call := range result.calls {
				if !dependencySet[practicalClosureDependency{call.caller, call.callee, "body_dependency"}] {
					t.Fatal("source call is missing its body dependency", call)
				}
			}
			if test.name == "Pure" {
				nested := 0
				for _, d := range result.declarations {
					if d.kind == "function" {
						continue
					}
					for _, fn := range result.declarations {
						if fn.kind == "function" && d.ref.span.sourceID == fn.ref.span.sourceID && fn.ref.span.start < d.ref.span.start && d.ref.span.end < fn.ref.span.end {
							nested++
							if !reflect.DeepEqual(d.roles, []string{"type_check_only"}) {
								t.Fatal("unselected local declaration entered proof", d)
							}
						}
					}
				}
				if nested < 11 {
					t.Fatal("unselected nested declaration inventory omitted", nested)
				}
			}
			// Native object calls independently match the physical declaration graph.
			practicalClosureCheckNativeCalls(t, owner, result)
		})
	}
	roots := []practicalClosureRef{}
	for n := 0; n < 70; n++ {
		roots = append(roots, practicalClosureTestRef(t, owner, "example.com/policy", fmt.Sprintf("Root%d", n)))
	}
	many, err := owner.sourceClosure(roots)
	if err != nil || len(many.roots) != 70 || len(many.calls) != 70 || len(many.issues) != 0 {
		t.Fatal("many selected roots truncated", err)
	}
	reverse := append([]practicalClosureRef(nil), roots...)
	for i, j := 0, len(reverse)-1; i < j; i, j = i+1, j-1 {
		reverse[i], reverse[j] = reverse[j], reverse[i]
	}
	again, err := owner.sourceClosure(reverse)
	if err != nil || !reflect.DeepEqual(practicalClosureEvidence(many), practicalClosureEvidence(again)) {
		t.Fatal("caller root order changed canonical closure", err)
	}
	retainPracticalSyntaxEvidence(t, "closure-many-roots.json", practicalClosureEvidence(many))
	for _, bad := range [][]practicalClosureRef{nil, {roots[0], roots[0]}, {{"wrong/package", roots[0].span}}, {{roots[0].packagePath, practicalSourceSpan{roots[0].span.sourceID, roots[0].span.start + 1, roots[0].span.end}}}} {
		if _, err := owner.sourceClosure(bad); err == nil {
			t.Fatal("non-original root accepted")
		}
	}
	// Returned sources and graph slices own their allocations across fresh reads
	// and closing the image owner; no application initializer was executed.
	keep := practicalClosureEvidence(saved)
	clear(saved.sources[0].data)
	saved.roots[0].packagePath = "mutated-result"
	saved.declarations[0].roles[0] = "mutated-result"
	fresh, err := owner.sourceClosure([]practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Pure")})
	if err != nil || !reflect.DeepEqual(keep, practicalClosureEvidence(fresh)) {
		t.Fatal("returned closure mutation changed owner", err)
	}
	owner.Close()
	if _, err := owner.sourceClosure(roots[:1]); err == nil {
		t.Fatal("closed owner accepted")
	}
	if len(fresh.sources) == 0 || len(fresh.sources[0].data) == 0 {
		t.Fatal("closing owner erased returned source evidence")
	}
	retainPracticalSyntaxEvidence(t, "closure-final-controls.json", map[string]any{"cases": len(tests), "cross_reader_helper_links": 512, "roots": 70, "parameter_slots": 70, "source_ownership": true, "error_method_signature_only": true, "application_initializers_executed": false, "source_or_execution_program_proof": false})
}

func practicalClosureCheckNativeCalls(t *testing.T, owner *practicalOriginalPackages, c practicalSourceClosure) {
	t.Helper()
	targets := map[*types.Func]practicalClosureRef{}
	for path, loaded := range owner.loader.packages {
		for _, file := range loaded.files {
			for _, d := range file.Decls {
				if fn, ok := d.(*ast.FuncDecl); ok {
					if obj, ok := loaded.info.Defs[fn.Name].(*types.Func); ok {
						tf := owner.loader.syntax.fileSet.File(fn.Pos())
						targets[obj.Origin()] = practicalClosureRef{path, practicalSourceSpan{strings.TrimSuffix(tf.Name(), ".go"), tf.Offset(fn.Pos()), tf.Offset(fn.End())}}
					}
				}
			}
		}
	}
	expected := map[practicalClosureRef][]practicalClosureCall{}
	for _, decl := range c.declarations {
		body := false
		for _, role := range decl.roles {
			body = body || role == "selected_root" || role == "body_dependency"
		}
		if !body {
			continue
		}
		loaded := owner.loader.packages[decl.ref.packagePath]
		var fn *ast.FuncDecl
		for _, file := range loaded.files {
			for _, d := range file.Decls {
				if f, ok := d.(*ast.FuncDecl); ok {
					tf := owner.loader.syntax.fileSet.File(f.Pos())
					if strings.TrimSuffix(tf.Name(), ".go") == decl.ref.span.sourceID && tf.Offset(f.Pos()) == decl.ref.span.start {
						fn = f
					}
				}
			}
		}
		if fn == nil || fn.Body == nil {
			continue
		}
		ast.Inspect(fn.Body, func(node ast.Node) bool {
			call, ok := node.(*ast.CallExpr)
			if !ok {
				return true
			}
			fun := call.Fun
			for {
				p, ok := fun.(*ast.ParenExpr)
				if !ok {
					break
				}
				fun = p.X
			}
			var obj types.Object
			var selection *types.Selection
			switch f := fun.(type) {
			case *ast.Ident:
				obj = loaded.info.Uses[f]
			case *ast.SelectorExpr:
				selection = loaded.info.Selections[f]
				if selection != nil {
					obj = selection.Obj()
				} else {
					obj = loaded.info.Uses[f.Sel]
				}
			}
			if builtin, ok := obj.(*types.Builtin); ok && (builtin.Name() == "len" || builtin.Name() == "cap") && loaded.info.Types[call].Value != nil {
				return false
			}
			target, ok := obj.(*types.Func)
			if !ok {
				return true
			}
			if selection != nil {
				if _, ok := selection.Recv().Underlying().(*types.Interface); ok {
					return true
				}
			}
			if destination, exists := targets[target.Origin()]; exists {
				cf := owner.loader.syntax.fileSet.File(call.Pos())
				expected[decl.ref] = append(expected[decl.ref], practicalClosureCall{decl.ref, destination, practicalSourceSpan{strings.TrimSuffix(cf.Name(), ".go"), cf.Offset(call.Pos()), cf.Offset(call.End())}})
			}
			return true
		})
	}
	actual := map[practicalClosureRef][]practicalClosureCall{}
	for _, call := range c.calls {
		actual[call.caller] = append(actual[call.caller], call)
	}
	order := func(v []practicalClosureCall) {
		sort.Slice(v, func(i, j int) bool { return v[i].span.start < v[j].span.start })
	}
	for _, v := range expected {
		order(v)
	}
	for _, v := range actual {
		order(v)
	}
	// Generic syntax is separately diagnosed. The independent direct-call walk
	// intentionally leaves generic indexing to the native generic negative case.
	for ref, want := range expected {
		if !reflect.DeepEqual(want, actual[ref]) {
			t.Fatal("native original call/declaration/physical positions differ", ref, want, actual[ref])
		}
	}
	for ref, calls := range actual {
		if _, ok := expected[ref]; !ok && len(calls) > 0 {
			loaded := owner.loader.packages[ref.packagePath]
			generic := false
			for _, call := range calls {
				for expr := range loaded.info.Instances {
					tf := owner.loader.syntax.fileSet.File(expr.Pos())
					if strings.TrimSuffix(tf.Name(), ".go") == call.span.sourceID && tf.Offset(expr.Pos()) >= call.span.start && tf.Offset(expr.Pos()) < call.span.end {
						generic = true
					}
				}
			}
			if !generic {
				t.Fatal("extra non-native source call", ref, calls)
			}
		}
	}
}
