//go:build go1.25 && (linux || darwin)

package main

import (
	"go/ast"
	"go/types"
	"testing"
)

// Keep original Go observations for heterogeneous same-U error payloads.
// No native type fact or target table enters the independent Rust constructor.
func TestPracticalErrorRepresentationOriginalPayloads(t *testing.T) {
	files := map[string][]byte{
		"helper/helper.go": []byte("package helper\n"),
        "go.mod": []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go": []byte(`package policy
import _ "example.com/policy/helper"
type Scalar bool
type Bits uint64
type Empty struct{}
type Product struct{ Flag bool; N int16; Bytes [3]uint8; Empty [0]bool }
type Text struct{ Message string }
type Backed struct{ Tail []uint8 }
func(Scalar)Error()string{return ""}
func(Bits)Error()string{return ""}
func(Empty)Error()string{return ""}
func(Product)Error()string{return ""}
func(Text)Error()string{return ""}
func(Backed)Error()string{return ""}
func Entry(e error)string{return e.Error()}
`),
	}
	owner, _ := practicalStorageTestOwner(t, files)
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "dispatch-inputs.json", practicalBoxingInput(t, owner))
	declarations := map[string]any{}
	for path, loaded := range owner.loader.packages {
		if path != "example.com/policy" {
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
	names := []string{"Scalar", "Bits", "Empty", "Product", "Text", "Backed"}
	entry := []types.Type{}
	for _, name := range names {
		entry = append(entry, owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())
	}
	retainPracticalSyntaxEvidence(t, "dispatch-00-input.json", map[string]any{"name": "heterogeneous-payloads", "roots": []any{map[string]any{"name": "Entry", "ref": practicalDispatchRef(root)}}, "entry_cases": names, "expected_rejection": ""})
	retainPracticalSyntaxEvidence(t, "dispatch-00-actual.json", practicalNativeErrorClosure(t, owner, []practicalClosureRef{root}, entry))
	retainPracticalSyntaxEvidence(t, "dispatch-controls.json", map[string]any{"cases": 1, "complete_error_payload_types": 6, "source_runtime_validity_proof": false})
}
