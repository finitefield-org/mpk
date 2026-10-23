//go:build go1.25 && (linux || darwin)

package main

import (
	"go/ast"
	"go/types"
	"testing"
)

// Native observations remain comparison data. The Rust constructor consumes
// original source buffers and derives its own complete value/error domain.
func TestPracticalSourceStringOriginalTypes(t *testing.T) {
	files := map[string][]byte{
		"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"),
		"helper/helper.go": []byte(`package helper
type Remote string
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
func(Name)Error()string{return ""}
func(Chain)Error()string{return ""}
func(Text)Error()string{return ""}
func(Code)Error()string{return ""}
func Entry(e error)string{return e.Error()}
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
	names := []string{"Name", "Chain", "Text", "Code"}
	entry := []types.Type{}
	for _, name := range names {
		entry = append(entry, owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())
	}
	retainPracticalSyntaxEvidence(t, "dispatch-00-input.json", map[string]any{"name": "source-strings", "roots": []any{map[string]any{"name": "Entry", "ref": practicalDispatchRef(root)}}, "entry_cases": names, "expected_rejection": ""})
	retainPracticalSyntaxEvidence(t, "dispatch-00-actual.json", practicalNativeErrorClosure(t, owner, []practicalClosureRef{root}, entry))
	retainPracticalSyntaxEvidence(t, "dispatch-controls.json", map[string]any{"cases": 1, "complete_error_payload_types": 4, "native_string_types": 4, "source_runtime_validity_proof": false})
}
