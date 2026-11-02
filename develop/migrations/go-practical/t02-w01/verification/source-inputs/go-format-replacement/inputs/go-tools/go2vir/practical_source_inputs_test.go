//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"testing"
)

func TestPracticalSourceInputsRequireOriginalOwner(t *testing.T) {
	for _, owner := range []*practicalOriginalPackages{nil, {}, {closed: true}, {inputs: &practicalImageInputs{}, loader: &practicalSourceTypes{}}} {
		input, err := owner.sourceInputs()
		if err == nil || len(input.components.canonical) != 0 || len(input.components.sources) != 0 {
			t.Fatal("unowned source input packet accepted", err)
		}
	}
}

func TestPracticalSourceInputsKeepOriginalReplacementPackage(t *testing.T) {
	owner, _ := practicalStorageTestOwnerForPackage(t, map[string][]byte{
		"go.mod": []byte("module example.com/policy\ngo 1.25.0\nrequire example.com/policy/api v0.0.0\nreplace example.com/policy/api => ./api\n"),
		"policy.go": []byte("package policy\nfunc init(){panic(\"must not execute\")}\n"),
		"api/go.mod": []byte("module example.com/policy/api\ngo 1.25.0\n"),
		"api/root.go": []byte("package api\nfunc Root(x uint16)uint16{return x+1}\n"),
		"api/unused.go": []byte("package api\nfunc Unused(c chan int,f func(int)int)int{return f(<-c)}\nfunc init(){panic(\"must not execute\")}\n"),
	}, "example.com/policy/api")
	defer owner.Close()
	if owner.loader.packages[owner.requestedPackage].moduleSourceID == owner.mainSourceID {
		t.Fatal("replacement case did not retain distinct original modules")
	}
	input, err := owner.sourceInputs()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "unit-inputs.json", practicalStringOperationInput(t, owner))
	root := practicalClosureTestRef(t, owner, "example.com/policy/api", "Root")
	retainPracticalSyntaxEvidence(t, "replacement-root.json", map[string]any{"root": practicalDispatchRef(root), "main_source_object_id":input.mainSourceID,"requested_module_source_object_id":owner.loader.packages[owner.requestedPackage].moduleSourceID})
	checkPracticalSourceInputOwnership(t, owner)
}

// Run within the complete original zero/70-error-case fixture, after its
// application/module/SDK paths and replay image have already been retired.
func checkPracticalSourceInputOwnership(t *testing.T, owner *practicalOriginalPackages) {
	t.Helper()
	first, err := owner.sourceInputs()
	if err != nil {
		t.Fatal(err)
	}
	canonical := bytes.Clone(first.components.canonical)
	objects := make([]practicalSourceObject, len(first.components.sources))
	raw := make([][]byte, len(objects))
	for i, source := range first.components.sources {
		objects[i], raw[i] = source.object, source.sourceBytes()
	}
	if first.imageDigest != owner.imageDigest || first.compilerDigest != owner.compilerDigest || first.mainSourceID != owner.mainSourceID || first.requestedPackage != owner.requestedPackage || len(first.components.nodeKeys) != 0 {
		t.Fatal("source-only packet lost its original owner or acquired native expression facts")
	}
	retainPracticalSyntaxEvidence(t, "source-input-provenance.json", map[string]any{
		"image_sha256": first.imageDigest, "compiler_sha256": first.compilerDigest,
		"main_source_object_id": first.mainSourceID, "requested_package": first.requestedPackage,
		"source_inputs_sha256":                first.contentSHA256,
		"registered_context_or_program_proof": false,
	})
	clear(first.components.canonical)
	for i := range first.components.sources {
		clear(first.components.sources[i].data)
		first.components.sources[i].object.ID = "caller mutation"
	}
	again, err := owner.sourceInputs()
	if err != nil || !bytes.Equal(again.components.canonical, canonical) || again.contentSHA256 != first.contentSHA256 || len(again.components.sources) != len(objects) {
		t.Fatal("caller-owned source packet mutated the original universe", err)
	}
	check := func() {
		if !bytes.Equal(again.components.canonical, canonical) {
			t.Fatal("source packet lost its original canonical bytes")
		}
		for i, source := range again.components.sources {
			if source.object != objects[i] || !bytes.Equal(source.sourceBytes(), raw[i]) {
				t.Fatal("source packet lost an original buffer", i)
			}
		}
	}
	check()
	t.Cleanup(func() {
		owner.Close()
		check()
		if closed, err := owner.sourceInputs(); err == nil || len(closed.components.canonical) != 0 {
			t.Fatal("closed original owner emitted new source inputs", err)
		}
	})
}
