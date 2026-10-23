//go:build go1.25 && (linux || darwin)

package main

import (
	"go/ast"
	"go/types"
	"testing"
)

// Native observations remain comparison data. The Rust constructor consumes
// original source buffers and derives its own complete value/error domain.
func TestPracticalSourceStringIndexOriginalTypes(t *testing.T) {
	files := map[string][]byte{
		"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"),
		"helper/helper.go": []byte(`package helper
type Remote string
type RemoteIndex uint32
`),
		"policy.go": []byte(`package policy
import "example.com/policy/helper"
type Named string
type Chained Named
type Alias = Named
type Name struct { Value Named }
type Chain struct { Value Chained }
type Text struct { Plain string; Named Named; Chain Chained; Imported helper.Remote; Empty [0]uint8; Flag bool }
type Code uint8
type Offset int16
type ChainIndex Offset
type AliasIndex = Offset
func(Name)Error()string{return ""}
func(Chain)Error()string{return ""}
func(Text)Error()string{return ""}
func(Code)Error()string{return ""}
func Entry(e error)string{return e.Error()}
func Byte(s string, n Named, c Chained, h helper.Remote, i int, u uint, i8 int8, u8 uint8, i16 int16, u16 uint16, i32 int32, u32 uint32, i64 int64, u64 uint64, offset Offset, chain ChainIndex, remote helper.RemoteIndex, alias AliasIndex) uint8 {
 return s[i]+n[i8]+c[u]+h[remote]
}
`),
	}
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
	root := practicalClosureTestRef(t, owner, "example.com/policy", "Entry")
	byteRoot := practicalClosureTestRef(t, owner, "example.com/policy", "Byte")
	names := []string{"Name", "Chain", "Text", "Code"}
	entry := []types.Type{}
	for _, name := range names {
		entry = append(entry, owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())
	}
	retainPracticalSyntaxEvidence(t, "dispatch-00-input.json", map[string]any{"name": "source-string-indices", "roots": []any{map[string]any{"name": "Entry", "ref": practicalDispatchRef(root)}, map[string]any{"name": "Byte", "ref": practicalDispatchRef(byteRoot)}}, "entry_cases": names, "expected_rejection": ""})
	retainPracticalSyntaxEvidence(t, "dispatch-00-actual.json", practicalNativeErrorClosure(t, owner, []practicalClosureRef{root, byteRoot}, entry))
	retainPracticalSyntaxEvidence(t, "dispatch-controls.json", map[string]any{"cases": 1, "complete_error_payload_types": 4, "native_string_types": 4, "source_runtime_validity_proof": false})
	nativeType := func(typ types.Type) string {
		if _, ok := types.Unalias(typ).(*types.Named); ok {
			return practicalNativeBoxType(t, owner, typ)
		}
		b, ok := typ.Underlying().(*types.Basic)
		if !ok {
			t.Fatalf("non scalar index fixture type: %v", typ)
		}
		var width any
		var signed any
		if b.Info()&types.IsInteger != 0 {
			w := 64
			switch b.Kind() {
			case types.Int8, types.Uint8:
				w = 8
			case types.Int16, types.Uint16:
				w = 16
			case types.Int32, types.Uint32:
				w = 32
			}
			width = w
			signed = b.Info()&types.IsUnsigned == 0
		}
		key, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": types.Typ[b.Kind()].Name(), "width": width, "signed": signed})
		if err != nil {
			t.Fatal(err)
		}
		return key["id"].(string)
	}
	signature := owner.loader.packages["example.com/policy"].types.Scope().Lookup("Byte").Type().(*types.Signature)
	indices := map[string]any{}
	strings := map[string]bool{}
	for i := 0; i < signature.Params().Len(); i++ {
		typ := signature.Params().At(i).Type()
		basic := typ.Underlying().(*types.Basic)
		id := nativeType(typ)
		if basic.Kind() == types.String {
			strings[id] = true
			continue
		}
		w := 64
		switch basic.Kind() {
		case types.Int8, types.Uint8:
			w = 8
		case types.Int16, types.Uint16:
			w = 16
		case types.Int32, types.Uint32:
			w = 32
		}
		indices[id] = map[string]any{"type_id": id, "width": w, "signed": basic.Info()&types.IsUnsigned == 0, "go_type": types.TypeString(typ, nil)}
	}
	if len(indices) != 13 || len(strings) != 4 {
		t.Fatalf("wrong complete native matrix: %d / %d", len(indices), len(strings))
	}
	retainPracticalSyntaxEvidence(t, "index-types.json", indices)
	retainPracticalSyntaxEvidence(t, "string-types.json", strings)

}
