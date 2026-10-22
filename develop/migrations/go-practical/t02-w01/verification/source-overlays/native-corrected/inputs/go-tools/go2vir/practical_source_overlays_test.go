//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/json"
	"fmt"
	"go/ast"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func practicalOverlayFunction(t *testing.T, owner *practicalOriginalPackages, selector map[string]any) (practicalClosureRef, *ast.FuncDecl, *practicalTypedPackage) {
	t.Helper()
	path := selector["package"].(string)
	loaded := owner.loader.packages[path]
	var function *types.Func
	if receiver, ok := selector["receiver"].(map[string]any); ok {
		typ := loaded.types.Scope().Lookup(receiver["type_name"].(string)).Type()
		selection := types.NewMethodSet(typ).Lookup(loaded.types, selector["function"].(string))
		if selection == nil {
			t.Fatal("missing native selected method")
		}
		function = selection.Obj().(*types.Func)
	} else {
		function = loaded.types.Scope().Lookup(selector["function"].(string)).(*types.Func)
	}
	for _, file := range loaded.files {
		for _, decl := range file.Decls {
			if f, ok := decl.(*ast.FuncDecl); ok && loaded.info.Defs[f.Name] == function {
				span, _, err := owner.loader.syntax.spanBytes(f)
				if err != nil {
					t.Fatal(err)
				}
				return practicalClosureRef{path, span}, f, loaded
			}
		}
	}
	t.Fatal("missing native function declaration")
	return practicalClosureRef{}, nil, nil
}
func practicalOverlayBinding(t *testing.T, owner *practicalOriginalPackages, raw map[string]any) map[string]any {
	t.Helper()
	root, f, loaded := practicalOverlayFunction(t, owner, raw["function"].(map[string]any))
	sig := loaded.info.Defs[f.Name].Type().(*types.Signature)
	inputs := []types.Type{}
	if sig.Recv() != nil {
		inputs = append(inputs, sig.Recv().Type())
	}
	for i := 0; i < sig.Params().Len(); i++ {
		inputs = append(inputs, sig.Params().At(i).Type())
	}
	entries := []any{}
	for _, entry := range raw["error_inputs"].([]any) {
		e := entry.(map[string]any)
		position := int(e["input"].(float64))
		typ := inputs[position]
		path := []any{}
		for _, step := range e["path"].([]any) {
			s := step.(map[string]any)
			switch s["kind"] {
			case "elements":
				switch x := typ.Underlying().(type) {
				case *types.Array:
					typ = x.Elem()
				case *types.Slice:
					typ = x.Elem()
				default:
					t.Fatal("native element owner")
				}
				path = append(path, "elements")
			case "field":
				structure := typ.Underlying().(*types.Struct)
				found := -1
				for i := 0; i < structure.NumFields(); i++ {
					if structure.Field(i).Name() == s["name"] {
						found = i
						typ = structure.Field(i).Type()
						break
					}
				}
				if found < 0 {
					t.Fatal("native field selector")
				}
				path = append(path, found)
			}
		}
		if !types.Identical(typ, types.Universe.Lookup("error").Type()) {
			t.Fatal("native path is not actual error")
		}
		cases := []string{}
		for _, name := range e["cases"].([]any) {
			text := name.(string)
			dot := strings.LastIndex(text, ".")
			pkg := owner.loader.packages[text[:dot]]
			typ := pkg.types.Scope().Lookup(text[dot+1:]).Type()
			cases = append(cases, practicalNativeBoxType(t, owner, typ))
		}
		sort.Strings(cases)
		entries = append(entries, map[string]any{"input": position, "path": path, "cases": cases})
	}
	loops := []any{}
	ordinal := 0
	ast.Inspect(f.Body, func(n ast.Node) bool {
		if _, ok := n.(*ast.FuncLit); ok {
			return false
		}
		kind := ""
		switch n.(type) {
		case *ast.ForStmt:
			kind = "for"
		case *ast.RangeStmt:
			kind = "range"
		}
		if kind != "" {
			span, _, err := owner.loader.syntax.spanBytes(n)
			if err != nil {
				t.Fatal(err)
			}
			loops = append(loops, map[string]any{"kind": kind, "start_byte": span.start, "end_byte": span.end, "ordinal": ordinal})
			ordinal++
		}
		return true
	})
	return map[string]any{"root": practicalDispatchRef(root), "inputs": len(inputs), "results": sig.Results().Len(), "entries": entries, "loops": loops}
}
func TestPracticalSourceOverlayOriginalContexts(t *testing.T) {
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		output = filepath.Join(output, "source-overlays")
		if err := os.Mkdir(output, 0700); err != nil {
			t.Fatal(err)
		}
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", output)
	}
	policy := `package policy
import h "example.com/policy/helper"
type Code uint8
type CodeAlias=Code
type ImportedAlias=h.Other
type Err=error
func(Code)Error()string{return "code"}
type Inner struct{E error}
type Record struct{First error;Cells []struct{E error};Zero [0]error}
type Embedded struct{Inner;Imported h.Node}
type Blank struct{_ uint8;E error}
type Recv struct{E error}
type RecvAlias=Recv
type Other uint8
func(Other)Work(m map[string]int){_=m}
func Plain(e error)error{return e}
func Calls(e error)string{return e.Error()}
func Nested(x Record){_=x}
func EmbeddedInput(x Embedded){_=x}
func BlankInput(x Blank){_=x}
func(r Recv)Work(e error)(out error){
 for i:=0;i<2;i++{for _,v:=range [1]uint8{1}{_=v;_=i}}
 for j:=0;j<1;j++{_=j}
 _=r;return e
}
func(r Recv)init()int{_=r;return 1}
func RootInit(r Recv)int{return r.init()}
func(r *Recv)Pointer()int{return 1}
func Unnamed(uint16)(uint16,error){return 0,nil}
func NoError(x uint16)uint16{return x}
//line forged-absent.go:999
func 名前(e Err)error{return e}
func init(){}
`
	params := []string{}
	for i := 0; i < 70; i++ {
		params = append(params, fmt.Sprintf("e%d error", i))
	}
	policy += "func Many(" + strings.Join(params, ",") + "){"
	for i := 0; i < 70; i++ {
		policy += fmt.Sprintf("_=e%d;", i)
	}
	policy += "}\n"
	owner, _ := practicalStorageTestOwner(t, map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(strings.ReplaceAll(policy, "\n", "\r\n")), "helper/helper.go": []byte("package helper\ntype Other uint8\nfunc(Other)Error()string{return \"helper\"}\ntype Node struct{E error}\n")})
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "overlays-inputs.json", practicalBoxingInput(t, owner))
	function := func(name string, receiver any) map[string]any {
		return map[string]any{"package": "example.com/policy", "function": name, "receiver": receiver}
	}
	field := func(name string) any { return map[string]any{"kind": "field", "name": name} }
	elements := map[string]any{"kind": "elements"}
	entry := func(input int, path []any, cases []string) any {
		if cases == nil {
			cases = []string{}
		}
		return map[string]any{"input": input, "path": path, "cases": cases}
	}
	recv := map[string]any{"package": "example.com/policy", "type_name": "Recv"}
	alias := map[string]any{"package": "example.com/policy", "type_name": "RecvAlias"}
	rows := []struct {
		name     string
		selector map[string]any
		entries  []any
		annotate bool
	}{
		{"plain", function("Plain", nil), []any{entry(0, []any{}, []string{"example.com/policy.Code"})}, false},
		{"nil-only", function("Calls", nil), []any{entry(0, []any{}, nil)}, false},
		{"imported", function("Calls", nil), []any{entry(0, []any{}, []string{"example.com/policy.ImportedAlias"})}, false},
		{"nested", function("Nested", nil), []any{entry(0, []any{field("First")}, nil), entry(0, []any{field("Cells"), elements, field("E")}, []string{"example.com/policy.CodeAlias"}), entry(0, []any{field("Zero"), elements}, nil)}, false},
		{"embedded", function("EmbeddedInput", nil), []any{entry(0, []any{field("Inner"), field("E")}, nil), entry(0, []any{field("Imported"), field("E")}, nil)}, false},
		{"blank-ordinal", function("BlankInput", nil), []any{entry(0, []any{field("E")}, nil)}, false},
		{"receiver", function("Work", recv), []any{entry(0, []any{field("E")}, nil), entry(1, []any{}, []string{"example.com/policy.Code"})}, true},
		{"receiver-alias", function("Work", alias), []any{entry(0, []any{field("E")}, nil), entry(1, []any{}, []string{"example.com/policy.CodeAlias"})}, true},
		{"method-init", function("init", recv), []any{entry(0, []any{field("E")}, nil)}, false},
		{"unnamed", function("Unnamed", nil), []any{}, false}, {"no-error", function("NoError", nil), []any{}, false},
		{"physical-unicode", function("名前", nil), []any{entry(0, []any{}, nil)}, false},
	}
	many := []any{}
	for i := 0; i < 70; i++ {
		many = append(many, entry(i, []any{}, nil))
	}
	rows = append(rows, struct {
		name     string
		selector map[string]any
		entries  []any
		annotate bool
	}{"seventy-inputs", function("Many", nil), many, false})
	directory, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	overlaySession := practicalTestSession(t, []practicalSourceRoot{{Capability: "overlays", Directory: directory}})
	observations := []any{}
	for i, row := range rows {
		raw := map[string]any{"schema": "mpk.go.practical.source_overlay.v1", "source": "policy.go", "function": row.selector, "termination": "partial", "requires": []any{}, "ensures": []any{map[string]any{"op": "bool.literal", "value": true}}, "error_inputs": row.entries, "loops": []any{}, "modifies": []any{}}
		// Roundtrip only to match actual raw JSON numeric values for the independent
		// native observer. Original sidecar bytes are read through a rooted capture.
		bytes, err := canonicalJSON(raw)
		if err != nil {
			t.Fatal(err)
		}
		var value map[string]any
		if err := json.Unmarshal(bytes, &value); err != nil {
			t.Fatal(err)
		}
		native := practicalOverlayBinding(t, owner, value)
		if row.annotate {
			loops := native["loops"].([]any)
			raw["loops"] = []any{map[string]any{"selector": loops[1], "invariants": []any{map[string]any{"op": "bool.literal", "value": true}}, "variant": map[string]any{"kind": "candidate", "expression": map[string]any{"op": "nat.literal", "value": "1"}}}}
			raw["termination"] = "total"
		}
		bytes, err = canonicalJSON(raw)
		if err != nil {
			t.Fatal(err)
		}
		bytes = append(bytes, '\n')
		path := fmt.Sprintf("contracts/%02d.json", i)
		writePracticalTestFile(t, filepath.Join(directory, path), bytes, 0600)
		buffer, err := overlaySession.readSource("overlays", path, "overlay")
		if err != nil {
			t.Fatal(err)
		}
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("overlay-%02d-input.json", i), map[string]any{"name": row.name, "object": buffer.object, "raw": buffer.sourceBytes()})
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("overlay-%02d-actual.json", i), native)
		observations = append(observations, map[string]any{"name": row.name, "object": buffer.object})
	}
	sealed, err := overlaySession.seal()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, sealed)
	if err := removePracticalReplayImage(directory); err != nil {
		t.Fatal(err)
	}
	ref, _, _ := practicalOverlayFunction(t, owner, function("init", recv))
	closure, err := owner.sourceClosure([]practicalClosureRef{ref})
	if err != nil || len(closure.issues) != 0 {
		t.Fatal("ordinary value method init is not package initialization", err, practicalClosureEvidence(closure)["issues"])
	}
	retainPracticalSyntaxEvidence(t, "method-init-production.json", practicalClosureEvidence(closure))
	retainPracticalSyntaxEvidence(t, "overlay-controls.json", map[string]any{"cases": len(rows), "input_slots": 70, "original_sidecar_namespace_removed": true, "observations": observations, "typed_contract_or_context_authority": false})
}
