//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"testing"
)

type practicalDispatchOriginal struct {
	ref    practicalClosureRef
	node   *ast.FuncDecl
	object *types.Func
	loaded *practicalTypedPackage
}

func practicalDispatchRef(r practicalClosureRef) map[string]any {
	return map[string]any{"source": r.span.sourceID, "start": r.span.start, "end": r.span.end}
}
func practicalDispatchSorted(rows map[string]map[string]any) []map[string]any {
	keys := []string{}
	for key := range rows {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	out := []map[string]any{}
	for _, key := range keys {
		out = append(out, rows[key])
	}
	return out
}

// Native comparison only: every function, operand, type and target comes from
// the original go/types owner. No observations enter the Rust constructor.
func practicalNativeErrorClosure(t *testing.T, owner *practicalOriginalPackages, roots []practicalClosureRef, entry []types.Type) map[string]any {
	t.Helper()
	functions := map[*types.Func]*practicalDispatchOriginal{}
	byRef := map[practicalClosureRef]*practicalDispatchOriginal{}
	byType := map[string]types.Type{}
	for path, loaded := range owner.loader.packages {
		for _, file := range loaded.files {
			for _, decl := range file.Decls {
				if f, ok := decl.(*ast.FuncDecl); ok {
					object := loaded.info.Defs[f.Name].(*types.Func)
					span, _, err := owner.loader.syntax.spanBytes(f)
					if err != nil {
						t.Fatal(err)
					}
					original := &practicalDispatchOriginal{practicalClosureRef{path, span}, f, object, loaded}
					functions[object.Origin()] = original
					byRef[original.ref] = original
				}
			}
		}
		for _, object := range loaded.info.Defs {
			if tn, ok := object.(*types.TypeName); ok {
				if named, ok := types.Unalias(tn.Type()).(*types.Named); ok && named.Obj().Pkg() != nil {
					byType[practicalNativeBoxType(t, owner, tn.Type())] = types.Unalias(tn.Type())
				}
			}
		}
	}
	union := map[string]bool{}
	for _, typ := range entry {
		union[practicalNativeBoxType(t, owner, typ)] = true
	}
	bodies := map[practicalClosureRef]bool{}
	queue := append([]practicalClosureRef{}, roots...)
	calls := map[string]map[string]any{}
	dispatch := map[string]map[string]any{}
	sites := map[string]struct {
		caller     practicalClosureRef
		node       *ast.CallExpr
		receiver   ast.Expr
		expression bool
	}{}
	expanded := map[string]bool{}
	globalReads := map[string]map[string]any{}
	put := func(rows map[string]map[string]any, row map[string]any) {
		raw, err := canonicalJSON(row)
		if err != nil {
			t.Fatal(err)
		}
		rows[string(raw)] = row
	}
	edge := func(caller practicalClosureRef, node *ast.CallExpr, target practicalClosureRef) {
		span, _, err := owner.loader.syntax.spanBytes(node)
		if err != nil {
			t.Fatal(err)
		}
		put(calls, map[string]any{"caller": practicalDispatchRef(caller), "callee": practicalDispatchRef(target), "source": span.sourceID, "start": span.start, "end": span.end})
		if !bodies[target] {
			queue = append(queue, target)
		}
	}
	for {
		for len(queue) > 0 {
			ref := queue[0]
			queue = queue[1:]
			if bodies[ref] {
				continue
			}
			bodies[ref] = true
			f := byRef[ref]
			if f == nil || f.node.Body == nil {
				t.Fatal("missing original native body")
			}
			for _, site := range practicalNativeErrorBoxings(t, owner, ref, true) {
				union[site["type_id"].(string)] = true
			}
			type task struct {
				node    ast.Node
				runtime bool
			}
			work := []task{{f.node.Body, true}}
			for len(work) > 0 {
				item := work[len(work)-1]
				work = work[:len(work)-1]
				n := item.node
				if !practicalClosurePresent(n) {
					continue
				}
				info := f.loaded.info
				if e, ok := n.(ast.Expr); ok && info.Types[e].IsType() {
					item.runtime = false
				}
				if call, ok := n.(*ast.CallExpr); ok {
					object, selection := practicalClosureFunction(info, call.Fun)
					runArgs := item.runtime
					if builtin, ok := object.(*types.Builtin); ok && (builtin.Name() == "len" || builtin.Name() == "cap") && info.Types[call].Value != nil {
						runArgs = false
					}
					if item.runtime {
						if function, ok := object.(*types.Func); ok {
							if selection != nil && types.Identical(selection.Recv(), types.Universe.Lookup("error").Type()) {
								expression := selection.Kind() == types.MethodExpr
								var receiver ast.Expr
								if expression {
									receiver = call.Args[0]
								} else {
									receiver = practicalClosureUnparen(call.Fun).(*ast.SelectorExpr).X
								}
								span, _, err := owner.loader.syntax.spanBytes(call)
								if err != nil {
									t.Fatal(err)
								}
								key := fmt.Sprint(span)
								sites[key] = struct {
									caller     practicalClosureRef
									node       *ast.CallExpr
									receiver   ast.Expr
									expression bool
								}{ref, call, receiver, expression}
							} else if target := functions[function.Origin()]; target != nil {
								edge(ref, call, target.ref)
							}
						}
					}
					if selection != nil && selection.Kind() == types.MethodVal {
						work = append(work, task{practicalClosureUnparen(call.Fun).(*ast.SelectorExpr).X, item.runtime})
					}
					for _, arg := range call.Args {
						work = append(work, task{arg, runArgs})
					}
					continue
				}
				if r, ok := n.(*ast.RangeStmt); ok {
					work = append(work, task{r.X, item.runtime && practicalClosureRangeEvaluated(info, r)}, task{r.Body, item.runtime})
					continue
				}
				if id, ok := n.(*ast.Ident); ok && item.runtime {
					if v, ok := info.Uses[id].(*types.Var); ok && v.Pkg() != nil && v.Parent() == v.Pkg().Scope() {
						span, _, err := owner.loader.syntax.spanBytes(id)
						if err != nil {
							t.Fatal(err)
						}
						put(globalReads, map[string]any{"caller": practicalDispatchRef(ref), "source": span.sourceID, "start": span.start, "end": span.end})
					}
				}
				for _, child := range practicalClosureChildren(n) {
					work = append(work, task{child, item.runtime})
				}
			}
		}
		for key, site := range sites {
			for id := range union {
				pair := key + ":" + id
				if expanded[pair] {
					continue
				}
				expanded[pair] = true
				typ := byType[id]
				if typ == nil {
					t.Fatal("uncaptured original payload")
				}
				method := types.NewMethodSet(typ).Lookup(nil, "Error")
				if method == nil {
					t.Fatal("missing native case method")
				}
				target := functions[method.Obj().(*types.Func).Origin()]
				if target == nil {
					t.Fatal("uncaptured native target")
				}
				edge(site.caller, site.node, target.ref)
			}
		}
		if len(queue) == 0 {
			break
		}
	}
	ids := []string{}
	for id := range union {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	for _, site := range sites {
		call, _, err := owner.loader.syntax.spanBytes(site.node)
		if err != nil {
			t.Fatal(err)
		}
		receiver, _, err := owner.loader.syntax.spanBytes(site.receiver)
		if err != nil {
			t.Fatal(err)
		}
		targets := []any{}
		for _, id := range ids {
			method := types.NewMethodSet(byType[id]).Lookup(nil, "Error")
			target := functions[method.Obj().(*types.Func).Origin()]
			targets = append(targets, map[string]any{"type_id": id, "callee": practicalDispatchRef(target.ref)})
		}
		put(dispatch, map[string]any{"caller": practicalDispatchRef(site.caller), "source": call.sourceID, "start": call.start, "end": call.end, "receiver_start": receiver.start, "receiver_end": receiver.end, "method_expression": site.expression, "targets": targets})
	}
	bodyRows := map[string]map[string]any{}
	for ref := range bodies {
		put(bodyRows, practicalDispatchRef(ref))
	}
	return map[string]any{"bodies": practicalDispatchSorted(bodyRows), "calls": practicalDispatchSorted(calls), "dispatches": practicalDispatchSorted(dispatch), "global_cases": ids, "global_reads": practicalDispatchSorted(globalReads), "nil_receiver_or_runtime_proof": false}
}
func TestPracticalErrorDispatchOriginalContexts(t *testing.T) {
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		output = filepath.Join(output, "error-dispatch")
		if err := os.Mkdir(output, 0700); err != nil {
			t.Fatal(err)
		}
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", output)
	}
	policy := `package policy
import h "example.com/policy/helper"
type Code uint8
type More uint8
type Last uint8
type Global uint8
type Panic uint8
type Recursive uint8
type Alias=error
var State uint8
func(Code)Error()string{var _ error=More(0);return "code"}
func(More)Error()string{var _ error=Last(0);return h.End()}
func(Last)Error()string{return "last"}
func(Global)Error()string{_=State;return "global"}
func(Panic)Error()string{panic("called body")}
func(Recursive)Error()string{var e error=Recursive(0);return e.Error()}
func Entry(e error)string{return e.Error()}
func Expr(e error)string{return error.Error(e)}
func Paren(e Alias)string{return ((Alias.Error))(e)}
func Produce()error{return Code(0)}
func ReceiverCall()string{return Produce().Error()}
func ExprReceiver()string{return (error.Error)(Produce())}
func Local()string{var e error=Code(0);return e.Error()}
func Concrete()string{return (Code.Error)(Code(0))}
func NoCall()error{return Code(0)}
func BoxGlobal()error{return Global(0)}
func Nested(x struct{E error;S []error})string{return x.E.Error()}
func Guarded(e error)string{if e==nil{return ""};return e.Error()}
func Branch(e error)string{if false{return e.Error()};return ""}
func Imported(e error)string{var _ error=h.Other(0);return e.Error()}
func OtherRoot(e error)string{return e.Error()}
func Inline()string{return error.Error(error(Code(0)))}
type R struct{}
func(R)Text()string{return "receiver"}
func MakeR()R{return R{}}
func ReceiverStatic()string{return ((MakeR().Text))()}
func MakeBadR()R{_=State;return R{}}
func ReceiverBad()string{return MakeBadR().Text()}
`
	for i := 0; i < 70; i++ {
		policy += fmt.Sprintf("type Case%d uint8\nfunc(Case%d)Error()string{return \"\"}\n", i, i)
		policy += fmt.Sprintf("type Chain%d uint8\nfunc(Chain%d)Error()string{", i, i)
		if i < 69 {
			policy += fmt.Sprintf("var _ error=Chain%d(0);", i+1)
		}
		policy += "return \"\"}\n"
	}
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(policy), "helper/helper.go": []byte("package helper\ntype Other uint8\nfunc(Other)Error()string{return End()}\nfunc End()string{return \"end\"}\n")}
	owner, _ := practicalStorageTestOwner(t, files)
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "dispatch-inputs.json", practicalBoxingInput(t, owner))
	declarations := map[string]any{}
	for path, loaded := range owner.loader.packages {
		if path != "example.com/policy" && path != "example.com/policy/helper" {
			continue
		}
		for _, file := range loaded.files {
			for _, decl := range file.Decls {
				if g, ok := decl.(*ast.GenDecl); ok {
					for _, spec := range g.Specs {
						if ts, ok := spec.(*ast.TypeSpec); ok {
							span, _, err := owner.loader.syntax.spanBytes(ts)
							if err != nil {
								t.Fatal(err)
							}
							declarations[path+"."+ts.Name.Name] = map[string]any{"source": span.sourceID, "start": span.start, "end": span.end}
						}
					}
				}
			}
		}
	}
	retainPracticalSyntaxEvidence(t, "dispatch-declarations.json", declarations)
	cases := []struct {
		name   string
		roots  []string
		entry  []string
		reject string
	}{
		{"entry", []string{"Entry"}, []string{"Code"}, ""}, {"nil-only", []string{"Entry"}, nil, ""}, {"expression", []string{"Expr"}, []string{"Code"}, ""}, {"parentheses", []string{"Paren"}, []string{"Code"}, ""},
		{"receiver-call", []string{"ReceiverCall"}, nil, ""}, {"expression-receiver", []string{"ExprReceiver"}, nil, ""}, {"local", []string{"Local"}, nil, ""}, {"concrete-only", []string{"Concrete"}, nil, ""}, {"boxing-only", []string{"NoCall"}, nil, ""}, {"global-boxing-only", []string{"BoxGlobal"}, nil, ""},
		{"nested", []string{"Nested"}, []string{"Code"}, ""}, {"guarded", []string{"Guarded"}, []string{"Code"}, ""}, {"false-branch", []string{"Branch"}, []string{"Code"}, ""}, {"imported", []string{"Imported"}, []string{"Code"}, ""}, {"multiple-roots", []string{"Entry", "OtherRoot"}, []string{"Code"}, ""}, {"inline", []string{"Inline"}, nil, ""}, {"receiver-static", []string{"ReceiverStatic"}, nil, ""},
		{"global-target", []string{"Entry"}, []string{"Global"}, "MutablePackageState"}, {"panic-target", []string{"Entry"}, []string{"Panic"}, "Body"}, {"recursive-target", []string{"Entry"}, []string{"Recursive"}, "CallCycle"}, {"global-receiver", []string{"ReceiverBad"}, nil, "MutablePackageState"},
	}
	wide := []string{}
	for i := 0; i < 70; i++ {
		wide = append(wide, fmt.Sprintf("Case%d", i))
	}
	cases = append(cases, struct {
		name   string
		roots  []string
		entry  []string
		reject string
	}{"seventy-cases", []string{"Entry"}, wide, ""}, struct {
		name   string
		roots  []string
		entry  []string
		reject string
	}{"seventy-growth-rounds", []string{"Entry"}, []string{"Chain0"}, ""})
	for i, c := range cases {
		roots := []practicalClosureRef{}
		rootRows := []any{}
		for _, name := range c.roots {
			ref := practicalClosureTestRef(t, owner, "example.com/policy", name)
			roots = append(roots, ref)
			rootRows = append(rootRows, map[string]any{"name": name, "ref": practicalDispatchRef(ref)})
		}
		entry := []types.Type{}
		for _, name := range c.entry {
			entry = append(entry, owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())
		}
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("dispatch-%02d-input.json", i), map[string]any{"name": c.name, "roots": rootRows, "entry_cases": c.entry, "expected_rejection": c.reject})
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("dispatch-%02d-actual.json", i), practicalNativeErrorClosure(t, owner, roots, entry))
	}
	// Exercise the production receiver walk too, independently of finite tables.
	for _, name := range []string{"ReceiverStatic", "ReceiverBad", "ReceiverCall"} {
		ref := practicalClosureTestRef(t, owner, "example.com/policy", name)
		closure, err := owner.sourceClosure([]practicalClosureRef{ref})
		if err != nil {
			t.Fatal(err)
		}
		retainPracticalSyntaxEvidence(t, "production-"+name+".json", practicalClosureEvidence(closure))
		hasProducer := false
		for _, call := range closure.calls {
			hasProducer = hasProducer || call.caller == ref
		}
		if !hasProducer {
			t.Fatal("original receiver call missing", name)
		}
		if name == "ReceiverBad" && len(closure.issues) == 0 {
			t.Fatal("receiver global state was missed")
		}
	}
	retainPracticalSyntaxEvidence(t, "dispatch-controls.json", map[string]any{"cases": len(cases), "explicit_cases": 70, "growth_rounds": 70, "runtime_or_program_proof": false})
}
