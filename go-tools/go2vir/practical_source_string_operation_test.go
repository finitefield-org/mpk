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

func practicalStringOperationInput(t *testing.T, owner *practicalOriginalPackages) map[string]any {
	t.Helper()
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	b := practicalTypeKeyBuilder{owner: owner, sources: map[string]practicalSourceBuffer{}, packages: map[string]map[string]any{}, packageIDs: map[string]string{}}
	roots := []string{}
	for path, pkg := range owner.packages {
		if pkg.intrinsic {
			continue
		}
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

func TestPracticalSourceStringOperationOriginalTypes(t *testing.T) {
	for _, name := range []string{"pure", "wide"} {
		t.Run(name, func(t *testing.T) {
			if out := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); out != "" {
				out = filepath.Join(out, name)
				if err := os.Mkdir(out, 0700); err != nil {
					t.Fatal(err)
				}
				t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", out)
			}
			files := map[string][]byte{
				"go.mod":           []byte("module example.com/policy\ngo 1.25.0\n"),
				"helper/helper.go": []byte("package helper\ntype Remote string\nfunc Text() Remote{return \"abc\"}\n"),
				"shadow/shadow.go": []byte("package shadow\nfunc len(s string)int{return 1}\nfunc Length(s string)int{return len(s)}\n"),
				"policy.go": []byte(`package policy
import "example.com/policy/helper"
import "example.com/policy/shadow"
type Text string
type Alias = Text
type Index uint64
func Length(s string)int{return len(s)}
func NamedLength(s Alias)int{return len((s))}
func Twice(s string)int{return len(s)+len(s)}
func At(s Text,i Index)byte{return s[i]}
func Signed(s string,i int8)byte{return s[i]}
func Literal(i int)byte{return "\x00\xff"[i]}
func Constant()int{return len("a")}
func Nested(s string,t Text,i int8)byte{return s[t[i]]}
func FromCall(i int)byte{return helper.Text()[i]}
func CallLength()int{return len(helper.Text())}
func Arrays(a [2]string)int{return len(a)}
func Slices(s []byte)int{return len(s)}
func Shadow()int{return shadow.Length("x")}
`),
			}
			rootsNames := []string{"Length", "NamedLength", "Twice", "At", "Signed", "Literal", "Constant", "Nested", "FromCall", "CallLength", "Arrays", "Slices", "Shadow"}
			caseNames := []string{}
			if name == "wide" {
				files["policy.go"] = append(files["policy.go"], []byte("func Entry(e error)string{return e.Error()}\n")...)
				rootsNames = append(rootsNames, "Entry")
				for i := 0; i < 70; i++ {
					n := fmt.Sprintf("Case%02d", i)
					caseNames = append(caseNames, n)
					files["policy.go"] = append(files["policy.go"], []byte(fmt.Sprintf("type %s int16\nfunc(%s)Error()string{return \"\"}\n", n, n))...)
				}
			}
			owner, _ := practicalStorageTestOwner(t, files)
			defer owner.Close()
			retainPracticalSyntaxEvidence(t, "unit-inputs.json", practicalStringOperationInput(t, owner))
			retainPracticalSyntaxEvidence(t, "incomplete-unit-inputs.json", practicalBoxingInput(t, owner))
			roots := []practicalClosureRef{}
			rootRows := []any{}
			caseRows := []any{}
			entry := []types.Type{}
			var entryRow any
			for _, n := range rootsNames {
				r := practicalClosureTestRef(t, owner, "example.com/policy", n)
				roots = append(roots, r)
				rootRows = append(rootRows, practicalDispatchRef(r))
				if n == "Entry" {
					entryRow = practicalDispatchRef(r)
				}
			}
			declarations := map[string]any{}
			for path, loaded := range owner.loader.packages {
				if path != "example.com/policy" && path != "example.com/policy/helper" && path != "example.com/policy/shadow" {
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
			for _, n := range caseNames {
				caseRows = append(caseRows, declarations["example.com/policy."+n])
				entry = append(entry, owner.loader.packages["example.com/policy"].types.Scope().Lookup(n).Type())
			}
			retainPracticalSyntaxEvidence(t, "unit-selection.json", map[string]any{"name": name, "roots": rootRows, "cases": caseRows, "entry": entryRow, "declarations": declarations})
			closure := practicalNativeErrorClosure(t, owner, roots, entry)
			retainPracticalSyntaxEvidence(t, "native-closure.json", closure)
			bodies := map[string]bool{}
			for _, r := range closure["bodies"].([]map[string]any) {
				bodies[fmt.Sprint(r["source"], ":", r["start"], ":", r["end"])] = true
			}
			observations := []any{}
			for _, loaded := range owner.loader.packages {
				for _, file := range loaded.files {
					for _, decl := range file.Decls {
						f, ok := decl.(*ast.FuncDecl)
						if !ok {
							continue
						}
						span, _, err := owner.loader.syntax.spanBytes(f)
						if err != nil {
							t.Fatal(err)
						}
						if !bodies[fmt.Sprint(span.sourceID, ":", span.start, ":", span.end)] {
							continue
						}
						ast.Inspect(f.Body, func(n ast.Node) bool {
							var operands []ast.Expr
							kind := ""
							var constant any
							if call, ok := n.(*ast.CallExpr); ok {
								fun := call.Fun
								for {
									p, ok := fun.(*ast.ParenExpr)
									if !ok {
										break
									}
									fun = p.X
								}
								id, ok := fun.(*ast.Ident)
								if !ok {
									return true
								}
								b, ok := loaded.info.Uses[id].(*types.Builtin)
								if !ok || b.Name() != "len" || len(call.Args) != 1 {
									return true
								}
								ty, ok := loaded.info.TypeOf(call.Args[0]).Underlying().(*types.Basic)
								if !ok || (ty.Kind() != types.String && ty.Kind() != types.UntypedString) {
									return true
								}
								kind = "Length"
								operands = call.Args
								if value := loaded.info.Types[call].Value; value != nil {
									kind = "ConstantLength"
									constant = value.ExactString()
								}
							}
							if index, ok := n.(*ast.IndexExpr); ok {
								ty, ok := loaded.info.TypeOf(index.X).Underlying().(*types.Basic)
								if !ok || (ty.Kind() != types.String && ty.Kind() != types.UntypedString) {
									return true
								}
								kind = "Index"
								operands = []ast.Expr{index.X, index.Index}
							}
							if kind == "" {
								return true
							}
							site, _, err := owner.loader.syntax.spanBytes(n)
							if err != nil {
								t.Fatal(err)
							}
							args := []any{}
							for _, a := range operands {
								as, _, err := owner.loader.syntax.spanBytes(a)
								if err != nil {
									t.Fatal(err)
								}
								args = append(args, map[string]any{"start": as.start, "end": as.end, "type": types.TypeString(loaded.info.TypeOf(a), nil)})
							}
							observations = append(observations, map[string]any{"source": site.sourceID, "start": site.start, "end": site.end, "caller_start": span.start, "caller_end": span.end, "kind": kind, "constant": constant, "operands": args, "result": types.TypeString(loaded.info.TypeOf(n.(ast.Expr)), nil)})
							return true
						})
					}
				}
			}
			retainPracticalSyntaxEvidence(t, "native-operations.json", observations)
			if len(observations) != 12 {
				t.Fatalf("expected complete 12-site inventory, got%d", len(observations))
			}
		})
	}
}
