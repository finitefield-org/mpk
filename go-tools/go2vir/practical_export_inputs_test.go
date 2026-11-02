//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"testing"
)

func TestPracticalExportInputsKeepWholeOriginalCompilerUniverse(t *testing.T) {
	practicalContractTestOutput(t, "owned")
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{
		"go.mod":          []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":       []byte("package policy\nimport(\"embed\";\"net/http\")\n//go:embed all:assets\nvar Unused embed.FS\nfunc Root(x uint16)uint16{return x+1}\nfunc UnusedHandler(r *http.Request){ go func(){select{}}() }\nfunc init(){panic(\"never execute original initialization\")}\n"),
		"assets/data.bin": {0, 1, 255, 128}, "assets/.hidden": []byte("original hidden bytes\n"),
	}, "example.com/policy", nil)
	defer owner.Close()
	roots := []practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")}
	input, err := owner.exportInputs(roots)
	if err != nil {
		t.Fatal(err)
	}
	packet := practicalCaptureInputEvidence(t, input.capture)
	packet["export_inputs_hex"] = hex.EncodeToString(input.canonical)
	retainPracticalSyntaxEvidence(t, "complete-original-export-inputs.json", packet)
	original := bytes.Clone(input.canonical)
	clear(input.canonical)
	clear(input.capture.canonical)
	for i := range input.capture.snapshot.sources {
		clear(input.capture.snapshot.sources[i].data)
	}
	again, err := owner.exportInputs(roots)
	if err != nil || !bytes.Equal(original, again.canonical) {
		t.Fatal("export handoff aliases original owner", err)
	}
	packageCount, objects := len(owner.packages), len(owner.inputs.snapshot.ids)
	owner.Close()
	if !bytes.Equal(original, again.canonical) {
		t.Fatal("export input retired with owner")
	}
	if got, err := owner.exportInputs(roots); err == nil || got.canonical != nil {
		t.Fatal("closed owner accepted")
	}
	var absent *practicalOriginalPackages
	if got, err := absent.exportInputs(roots); err == nil || got.canonical != nil {
		t.Fatal("nil owner accepted")
	}
	retainPracticalSyntaxEvidence(t, "original-export-ownership.json", map[string]any{"packages": packageCount, "complete_original_objects": objects, "complete_raw_metadata_retained": true, "returned_buffers_independent": true, "owner_retirement_keeps_handoff": true, "closed_and_nil_owners_rejected": true, "original_initialization_or_unused_body_executed": false})
}
