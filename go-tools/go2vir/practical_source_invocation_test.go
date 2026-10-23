//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"fmt"
	"go/ast"
	"go/types"
	"os"
	"path/filepath"
	"testing"
)

func TestPracticalSourceInvocationOriginalSignatures(t *testing.T) {
	root := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification")
	read := func(path string) []byte {
		b, err := os.ReadFile(filepath.Join(root, path))
		if err != nil {
			t.Fatal(err)
		}
		return b
	}
	for _, name := range []string{"pure", "wide"} {
		t.Run(name, func(t *testing.T) {
			if out := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); out != "" {
				out = filepath.Join(out, name)
				if err := os.Mkdir(out, 0700); err != nil {
					t.Fatal(err)
				}
				t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", out)
			}
			policy := append(read("source-execution/program.go"), read("source-invocations/additions.go.txt")...)
			owner, _ := practicalStorageTestOwner(t, map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": policy, "helper/helper.go": []byte("package helper\ntype Remote string\nfunc Text()Remote{return \"abc\"}\n"), "shadow/shadow.go": []byte("package shadow\nfunc len(s string)int{return 1}\nfunc Length(s string)int{return len(s)}\n")})
			defer owner.Close()
			retainPracticalSyntaxEvidence(t, "unit-inputs.json", practicalStringOperationInput(t, owner))
			checkPracticalSourceInputOwnership(t, owner)
			roots := []practicalClosureRef{}
			rootRows := []any{}
			cases := []any{}
			entry := []types.Type{}
			var entryRow any
			declarations := map[string]any{}
			for _, file := range owner.loader.packages["example.com/policy"].files {
				for _, decl := range file.Decls {
					if f, ok := decl.(*ast.FuncDecl); ok && f.Recv == nil {
						r := practicalClosureTestRef(t, owner, "example.com/policy", f.Name.Name)
						roots = append(roots, r)
						rootRows = append(rootRows, practicalDispatchRef(r))
						if f.Name.Name == "Entry" {
							entryRow = practicalDispatchRef(r)
						}
					}
					if g, ok := decl.(*ast.GenDecl); ok {
						for _, s := range g.Specs {
							if ts, ok := s.(*ast.TypeSpec); ok {
								span, _, err := owner.loader.syntax.spanBytes(ts)
								if err != nil {
									t.Fatal(err)
								}
								declarations["example.com/policy."+ts.Name.Name] = map[string]any{"source": span.sourceID, "start": span.start, "end": span.end}
							}
						}
					}
				}
			}
			if name == "wide" {
				for i := 0; i < 70; i++ {
					n := fmt.Sprintf("Case%02d", i)
					cases = append(cases, declarations["example.com/policy."+n])
					entry = append(entry, owner.loader.packages["example.com/policy"].types.Scope().Lookup(n).Type())
				}
			}
			retainPracticalSyntaxEvidence(t, "unit-selection.json", map[string]any{"name": name, "roots": rootRows, "cases": cases, "entry": entryRow, "declarations": declarations})
			closure := practicalNativeErrorClosure(t, owner, roots, entry)
			retainPracticalSyntaxEvidence(t, "native-closure.json", closure)
			bodies := map[string]bool{}
			for _, r := range closure["bodies"].([]map[string]any) {
				bodies[fmt.Sprint(r["source"], ":", r["start"], ":", r["end"])] = true
			}
			// Only calls in the independently selected complete native body set. The
			// Go type facts below are comparison output, never Rust constructor input.
			observations := []any{}
			for _, loaded := range owner.loader.packages {
				for _, file := range loaded.files {
					for _, decl := range file.Decls {
						f, ok := decl.(*ast.FuncDecl)
						if !ok {
							continue
						}
						caller, _, err := owner.loader.syntax.spanBytes(f)
						if err != nil {
							t.Fatal(err)
						}
						if !bodies[fmt.Sprint(caller.sourceID, ":", caller.start, ":", caller.end)] {
							continue
						}
						ast.Inspect(f.Body, func(n ast.Node) bool {
							call, ok := n.(*ast.CallExpr)
							if !ok {
								return true
							}
							object, selected := practicalClosureFunction(loaded.info, call.Fun)
							if _, ok := object.(*types.Func); !ok {
								return true
							}
							signature, ok := loaded.info.TypeOf(call.Fun).(*types.Signature)
							if !ok {
								t.Fatal("native call signature missing")
							}
							inputTypes := []types.Type{}
							groups := []struct {
								node     ast.Expr
								receiver bool
							}{}
							if selected != nil && selected.Kind() == types.MethodVal {
								receiver := practicalClosureUnparen(call.Fun).(*ast.SelectorExpr).X
								inputTypes = append(inputTypes, loaded.info.TypeOf(receiver))
								groups = append(groups, struct {
									node     ast.Expr
									receiver bool
								}{receiver, true})
							}
							for i := 0; i < signature.Params().Len(); i++ {
								inputTypes = append(inputTypes, signature.Params().At(i).Type())
							}
							for i, a := range call.Args {
								groups = append(groups, struct {
									node     ast.Expr
									receiver bool
								}{a, selected != nil && selected.Kind() == types.MethodExpr && i == 0})
							}
							inputs := []string{}
							results := []string{}
							for _, ty := range inputTypes {
								inputs = append(inputs, practicalInvocationNativeType(t, owner, ty))
							}
							for i := 0; i < signature.Results().Len(); i++ {
								results = append(results, practicalInvocationNativeType(t, owner, signature.Results().At(i).Type()))
							}
							arguments := []any{}
							position := 0
							for _, g := range groups {
								span, _, err := owner.loader.syntax.spanBytes(g.node)
								if err != nil {
									t.Fatal(err)
								}
								ty := loaded.info.TypeOf(g.node)
								typesList := []types.Type{ty}
								if tuple, ok := ty.(*types.Tuple); ok {
									typesList = nil
									for i := 0; i < tuple.Len(); i++ {
										typesList = append(typesList, tuple.At(i).Type())
									}
								}
								values := []any{}
								for i, ty := range typesList {
									nilValue := loaded.info.Types[g.node].IsNil()
									if basic, ok := ty.(*types.Basic); ok && (basic.Info()&types.IsUntyped) != 0 {
										ty = inputTypes[position]
									}
									values = append(values, map[string]any{"projection": i, "parameter": position, "source_type": practicalInvocationNativeType(t, owner, ty), "parameter_type": inputs[position], "nil": nilValue})
									position++
								}
								arguments = append(arguments, map[string]any{"start": span.start, "end": span.end, "receiver": g.receiver, "values": values})
							}
							span, _, err := owner.loader.syntax.spanBytes(call)
							if err != nil {
								t.Fatal(err)
							}
							observations = append(observations, map[string]any{"source": span.sourceID, "start": span.start, "end": span.end, "caller_start": caller.start, "caller_end": caller.end, "inputs": inputs, "results": results, "arguments": arguments})
							return true
						})
					}
				}
			}
			retainPracticalSyntaxEvidence(t, "native-calls.json", observations)
		})
	}
}

// Comparison-only key derivation for every frozen value-type arm. This accepts
// only actual Go types from the owned source packages, never candidate facts.
func practicalInvocationNativeType(t *testing.T, owner *practicalOriginalPackages, typ types.Type) string {
	t.Helper()
	typ = types.Unalias(typ)
	row := map[string]any{"schema": practicalSourceTypeSchema}
	if types.Identical(typ, types.Universe.Lookup("error").Type()) {
		row["kind"] = "builtin"
		row["name"] = "error"
		row["width"] = nil
		row["signed"] = nil
	} else {
		switch ty := typ.(type) {
		case *types.Named:
			return practicalNativeBoxType(t, owner, ty)
		case *types.Basic:
			row["kind"] = "builtin"
			row["name"] = types.Typ[ty.Kind()].Name()
			row["width"] = nil
			row["signed"] = nil
			if ty.Info()&types.IsInteger != 0 {
				width := 64
				switch ty.Kind() {
				case types.Int8, types.Uint8:
					width = 8
				case types.Int16, types.Uint16:
					width = 16
				case types.Int32, types.Uint32:
					width = 32
				}
				row["width"] = width
				row["signed"] = ty.Info()&types.IsUnsigned == 0
			} else if ty.Kind() != types.Bool && ty.Kind() != types.String {
				t.Fatalf("unsupported native builtin %v", ty)
			}
		case *types.Array:
			row["kind"] = "array"
			row["length"] = ty.Len()
			row["element"] = practicalInvocationNativeType(t, owner, ty.Elem())
		case *types.Slice:
			row["kind"] = "slice"
			row["element"] = practicalInvocationNativeType(t, owner, ty.Elem())
		case *types.Struct:
			fields := []any{}
			for i := 0; i < ty.NumFields(); i++ {
				f := ty.Field(i)
				var pkg any
				if !f.Exported() {
					owner.mutex.Lock()
					b := practicalTypeKeyBuilder{owner: owner, sources: map[string]practicalSourceBuffer{}, packages: map[string]map[string]any{}, packageIDs: map[string]string{}}
					id, err := b.packageKey(f.Pkg().Path())
					owner.mutex.Unlock()
					if err != nil {
						t.Fatal(err)
					}
					pkg = id
				}
				fields = append(fields, map[string]any{"name": f.Name(), "unexported_package_id": pkg, "type_id": practicalInvocationNativeType(t, owner, f.Type()), "tag_bytes_hex": hex.EncodeToString([]byte(ty.Tag(i))), "embedded": f.Embedded()})
			}
			row["kind"] = "struct"
			row["fields"] = fields
		default:
			t.Fatalf("unsupported original native value type %v", typ)
		}
	}
	key, err := practicalTypeKeySeal(practicalSourceTypeDomain, row)
	if err != nil {
		t.Fatal(err)
	}
	return key["id"].(string)
}
