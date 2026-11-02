//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// This reference observes the intact compiler-owned graph. None of these facts
// enter the independent source constructor; entry cases are explicit predicates.
func practicalNativeDomain(typ types.Type, cases []types.Type, active map[types.Type]bool) (map[string]any, string) {
	typ = types.Unalias(typ)
	if active[typ] {
		return nil, "Cycle"
	}
	active[typ] = true
	defer delete(active, typ)
	if named, ok := typ.(*types.Named); ok {
		return practicalNativeDomain(named.Underlying(), cases, active)
	}
	plain, depth, weight, contains := true, 0, uint64(1), false
	children := []types.Type{}
	kind := ""
	switch t := typ.(type) {
	case *types.Basic:
		if t.Kind() == types.String {
			weight = 257
		}
	case *types.Array:
		kind = "array"
		children = append(children, t.Elem())
	case *types.Slice:
		kind = "slice"
		children = append(children, t.Elem())
	case *types.Struct:
		kind = "struct"
		for i := 0; i < t.NumFields(); i++ {
			children = append(children, t.Field(i).Type())
		}
	case *types.Interface:
		kind = "error"
		children = append(children, cases...)
		contains = true
	default:
		return nil, "Payload"
	}
	for _, child := range children {
		p, err := practicalNativeDomain(child, cases, active)
		if err != "" {
			return nil, err
		}
		plain = plain && p["plain"].(bool)
		contains = contains || p["contains_error"].(bool)
		if p["depth"].(int)+1 > depth {
			depth = p["depth"].(int) + 1
		}
		switch kind {
		case "array":
			weight = 1 + uint64(typ.(*types.Array).Len())*p["weight"].(uint64)
		case "slice":
			if !plain {
				return nil, "NonPlainBacking"
			}
			weight = 37
		default:
			weight += p["weight"].(uint64)
		}
	}
	if kind == "struct" || kind == "error" {
		if depth == 0 {
			depth = 1
		}
	}
	if kind == "slice" {
		plain = false
	}
	if depth > 16 {
		return nil, "Depth"
	}
	if weight > 2097152 {
		return nil, "Weight"
	}
	return map[string]any{"plain": plain, "depth": depth, "weight": weight, "contains_error": contains}, ""
}
func TestPracticalSourceDomainsOriginalContexts(t *testing.T) {
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		output = filepath.Join(output, "source-domains")
		if err := os.Mkdir(output, 0700); err != nil {
			t.Fatal(err)
		}
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", output)
	}
	policy := `package policy
import h "example.com/policy/helper"
type Code int64
type Alias = Code
type Flag bool
type Unit struct{}
type Ref struct{Values []uint8}
type Recursive struct{E error}
type ZeroRecursive struct{E [0]error}
type Text string
type Arr [1]uint8
type Pointer struct{}
type Wrong struct{}
type Named error
type Record struct{One error;Values []struct{E error};Zero [0]error}
func(Code)Error()string{panic("signature only")}
func(Flag)Error()string{return ""}
func(Unit)Error()string{return ""}
func(Ref)Error()string{panic("signature only")}
func(Recursive)Error()string{return ""}
func(ZeroRecursive)Error()string{return ""}
func(Text)Error()string{return ""}
func(Arr)Error()string{return ""}
func(*Pointer)Error()string{return ""}
func(Wrong)Error()int{return 0}
func Plain(e error)error{return e}
func Nested(x Record){_=x}
func ReceiverDummy(){}
func(x Record)Receiver(e error){_=x;_=e}
func Slice(e error,s []error){_=e;_=s}
func Box(e error)error{_=e;return h.Other(0)}
func NamedInput(e Named){_=e}
func Scalar(x uint8)uint8{return x}
func Depth16(x ` + strings.Repeat("[0]", 16) + `uint8){_=x}
func DepthError(x ` + strings.Repeat("[0]", 16) + `error){_=x}
func WeightGood(x [256][31]string){_=x}
func WeightBad(x [256][32]string){_=x}
func HiddenWeight(x [0][256][32]string){_=x}
func Empty(){var x struct{};_=x}
//go:noinline
func(Directed)Error()string{return ""}
type Directed uint8
`
	fields := []string{}
	for i := 0; i < 64; i++ {
		fields = append(fields, fmt.Sprintf("F%d [256][31]string", i))
	}
	policy += "type Huge struct{" + strings.Join(fields, ";") + "}\nfunc(Huge)Error()string{return \"\"}\n"
	for i := 0; i < 70; i++ {
		policy += fmt.Sprintf("type Case%d uint8\nfunc(Case%d)Error()string{return \"\"}\n", i, i)
	}
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(policy), "helper/helper.go": []byte("package helper\ntype Other uint8\nfunc(Other)Error()string{panic(\"signature only\")}\n")}
	owner, _ := practicalStorageTestOwner(t, files)
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "domains-inputs.json", practicalBoxingInput(t, owner))
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
							typ := loaded.info.Defs[ts.Name].Type()
							id := ""
							if _, ok := types.Unalias(typ).(*types.Named); ok {
								id = practicalNativeBoxType(t, owner, typ)
							}
							declarations[path+"."+ts.Name.Name] = map[string]any{"source": span.sourceID, "start": span.start, "end": span.end, "type_id": id}
						}
					}
				}
			}
		}
	}
	retainPracticalSyntaxEvidence(t, "domains-declarations.json", declarations)
	roots := map[string]any{}
	for _, name := range []string{"Plain", "Nested", "Receiver", "Slice", "Box", "NamedInput", "Scalar", "Depth16", "DepthError", "WeightGood", "WeightBad", "HiddenWeight", "Empty"} {
		ref := practicalClosureTestRef(t, owner, "example.com/policy", name)
		roots[name] = map[string]any{"source": ref.span.sourceID, "start": ref.span.start, "end": ref.span.end}
	}
	retainPracticalSyntaxEvidence(t, "domains-roots.json", roots)
	observations := []any{}
	for _, c := range []struct {
		root  string
		cases []string
	}{
		{"Plain", nil}, {"Plain", []string{"Code", "Flag", "Unit"}}, {"Plain", []string{"Ref"}},
		{"Plain", []string{"Recursive"}}, {"Plain", []string{"ZeroRecursive"}},
		{"Nested", []string{"Code"}}, {"Receiver", []string{"Code"}}, {"Slice", nil},
		{"Slice", []string{"Ref"}}, {"Depth16", nil}, {"DepthError", nil},
		{"WeightGood", nil}, {"WeightBad", nil}, {"HiddenWeight", nil}, {"Empty", nil},
	} {
		loaded := owner.loader.packages["example.com/policy"]
		cases := []types.Type{}
		for _, name := range c.cases {
			cases = append(cases, loaded.pkg.Scope().Lookup(name).Type())
		}
		var sig *types.Signature
		for _, file := range loaded.files {
			for _, decl := range file.Decls {
				if f, ok := decl.(*ast.FuncDecl); ok && f.Name.Name == c.root {
					sig = loaded.info.Defs[f.Name].Type().(*types.Signature)
				}
			}
		}
		inputs := []types.Type{}
		if sig.Recv() != nil {
			inputs = append(inputs, sig.Recv().Type())
		}
		for i := 0; i < sig.Params().Len(); i++ {
			inputs = append(inputs, sig.Params().At(i).Type())
		}
		props := []any{}
		for _, typ := range inputs {
			p, err := practicalNativeDomain(typ, cases, map[types.Type]bool{})
			props = append(props, map[string]any{"properties": p, "error": err})
		}
		observations = append(observations, map[string]any{"root": c.root, "cases": c.cases, "inputs": props})
	}
	retainPracticalSyntaxEvidence(t, "domains-observations.json", observations)
}
