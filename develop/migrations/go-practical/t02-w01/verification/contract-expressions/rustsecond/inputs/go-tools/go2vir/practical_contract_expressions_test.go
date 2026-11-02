//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

// Native comparisons only. Rust receives original source bytes and a Boolean
// seed, never these Go scope/type observations as constructor authority.
func practicalContractNativeType(t *testing.T, owner *practicalOriginalPackages, typ types.Type) string {
	t.Helper()
	typ = types.Unalias(typ)
	row := map[string]any{"schema": practicalSourceTypeSchema}
	if types.Identical(typ, types.Universe.Lookup("error").Type()) {
		row["kind"], row["name"], row["width"], row["signed"] = "builtin", "error", nil, nil
	} else {
		switch x := typ.(type) {
		case *types.Named:
			return practicalNativeBoxType(t, owner, x)
		case *types.Basic:
			row["kind"], row["name"] = "builtin", x.Name()
			row["width"], row["signed"] = nil, nil
			if x.Info()&types.IsInteger != 0 {
				width := 64
				switch x.Kind() {
				case types.Int8, types.Uint8:
					width = 8
				case types.Int16, types.Uint16:
					width = 16
				case types.Int32, types.Uint32:
					width = 32
				}
				row["width"], row["signed"] = width, x.Info()&types.IsUnsigned == 0
			}
		case *types.Slice:
			row["kind"], row["element"] = "slice", practicalContractNativeType(t, owner, x.Elem())
		case *types.Array:
			row["kind"], row["element"], row["length"] = "array", practicalContractNativeType(t, owner, x.Elem()), x.Len()
		default:
			t.Fatalf("unexpected native header type %v", typ)
		}
	}
	sealed, err := practicalTypeKeySeal(practicalSourceTypeDomain, row)
	if err != nil {
		t.Fatal(err)
	}
	return sealed["id"].(string)
}

func practicalContractNativeHeaders(t *testing.T, owner *practicalOriginalPackages, name string, receiver any) map[string]any {
	t.Helper()
	selector := map[string]any{"package": "example.com/policy", "function": name, "receiver": receiver}
	root, f, loaded := practicalOverlayFunction(t, owner, selector)
	loops := []any{}
	ast.Inspect(f.Body, func(n ast.Node) bool {
		if _, ok := n.(*ast.FuncLit); ok {
			return false
		}
		var body *ast.BlockStmt
		kind := ""
		switch x := n.(type) {
		case *ast.ForStmt:
			body, kind = x.Body, "for"
		case *ast.RangeStmt:
			body, kind = x.Body, "range"
		default:
			return true
		}
		scope := loaded.info.Scopes[n]
		if scope == nil {
			t.Fatal("missing actual native loop scope")
		}
		names := map[string]bool{}
		bindings := []any{}
		for current := scope; current != nil && current != loaded.types.Scope(); current = current.Parent() {
			for _, name := range current.Names() {
				if names[name] {
					continue
				}
				_, object := scope.LookupParent(name, body.Lbrace)
				if object == nil {
					continue
				}
				names[name] = true
				variable, ok := object.(*types.Var)
				if !ok {
					continue
				}
				position := owner.loader.syntax.fileSet.PositionFor(variable.Pos(), false)
				bindings = append(bindings, map[string]any{"name": name, "name_start_byte": position.Offset, "type_id": practicalContractNativeType(t, owner, variable.Type())})
			}
		}
		sort.Slice(bindings, func(i, j int) bool {
			return bindings[i].(map[string]any)["name"].(string) < bindings[j].(map[string]any)["name"].(string)
		})
		span, _, err := owner.loader.syntax.spanBytes(n)
		if err != nil {
			t.Fatal(err)
		}
		loops = append(loops, map[string]any{"selector": map[string]any{"kind": kind, "start_byte": span.start, "end_byte": span.end, "ordinal": len(loops)}, "bindings": bindings})
		return true
	})
	return map[string]any{"root": practicalDispatchRef(root), "function": selector, "loops": loops}
}

func TestPracticalContractExpressionOriginalScopes(t *testing.T) {
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		output = filepath.Join(output, "contract-expressions")
		if err := os.Mkdir(output, 0700); err != nil {
			t.Fatal(err)
		}
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", output)
	}
	policy := `package policy
import h "example.com/policy/helper"
type RejectCode uint8
type RejectAlias=RejectCode
func(RejectCode)Error()string{return "reject"}
type OtherCode uint16
func(OtherCode)Error()string{return "other"}
type Imported=h.Code
type Line struct{Amount int64;_ uint8}
type Flag bool
type Text string
type Bytes []uint8
type Word int8
type WordAlias=Word
type OtherWord int8
type Pair [2]uint8
type Struct struct{A uint8;B [2]bool}
type NotComparable struct{A []uint8}
type Huge [256][256][256]bool
type BadBacking [][]uint8
type ErrorCells []error
type ContractOnly uint8
func(ContractOnly)Error()string{m:=map[int]int{};_=m;return "not selected"}
type Recv struct{A uint8}
func(r Recv)Method(xs []uint8)(out uint8){for i:=0;i<1;i++{_=i;_=xs};return r.A}
func Slots(i uint16)(out uint16){for i:=uint8(0);i<1;i++{_=i};return i}
func Check(xs []uint8,w int64,b bool,err error)(out int64){
 i:=int8(9)
 for i:=uint16(0);i<2;i++{
  _=i
  hidden:=uint8(3)
  for j:=0;j<1;j++{_=j;_=hidden}
 }
 _=i
 for k,v:=range xs{
  _=k;_=v
  for j:=0;j<1;j++{_=j;_=v}
 }
 {const i=1;for j:=0;j<1;j++{_=j;_=i}}
 {type i uint8;var z i;for z<1{z++}}
 for j:=0;j<1;j++{_=j}
 later:=uint8(1);_=later
 _=b;_=err
 return w
}
//line nonexistent.go:999
func 名前(xs []uint8){for 番:=0;番<1;番++{_=番;_=xs}}
`
	owner, _ := practicalStorageTestOwner(t, map[string][]byte{
		"go.mod":           []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":        []byte(strings.ReplaceAll(policy, "\n", "\r\n")),
		"helper/helper.go": []byte("package helper\ntype Code uint8\nfunc(Code)Error()string{return \"helper\"}\n"),
	})
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "inputs.json", practicalBoxingInput(t, owner))
	rows := []map[string]any{}
	for _, name := range []string{"Check", "Slots", "Method", "名前"} {
		var receiver any
		if name == "Method" {
			receiver = map[string]any{"package": "example.com/policy", "type_name": "Recv"}
		}
		rows = append(rows, practicalContractNativeHeaders(t, owner, name, receiver))
	}
	directory, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	for i, row := range rows {
		entries := []any{}
		if i == 0 {
			entries = append(entries, map[string]any{"input": 3, "path": []any{}, "cases": []string{"example.com/policy.RejectCode"}})
		}
		raw := map[string]any{"schema": "mpk.go.practical.source_overlay.v1", "source": "policy.go", "function": row["function"], "termination": "partial", "requires": []any{}, "ensures": []any{map[string]any{"op": "bool.literal", "value": true}}, "error_inputs": entries, "loops": []any{}, "modifies": []any{}}
		data, err := canonicalJSON(raw)
		if err != nil {
			t.Fatal(err)
		}
		writePracticalTestFile(t, filepath.Join(directory, fmt.Sprintf("%02d.json", i)), append(data, '\n'), 0600)
	}
	session := practicalTestSession(t, []practicalSourceRoot{{Capability: "overlays", Directory: directory}})
	for i, row := range rows {
		buffer, err := session.readSource("overlays", fmt.Sprintf("%02d.json", i), "overlay")
		if err != nil {
			t.Fatal(err)
		}
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("overlay-%02d.json", i), map[string]any{"object": buffer.object, "raw": buffer.sourceBytes(), "native": row})
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, snapshot)
	if err := removePracticalReplayImage(directory); err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "controls.json", map[string]any{"original_sidecar_namespace_removed": true, "roots": len(rows), "constructor_receives_native_facts": false})
}
