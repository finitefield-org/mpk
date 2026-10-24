//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"maps"
	"os"
	"path/filepath"
	"testing"
)

func practicalContractTestOutput(t *testing.T, name string) {
	t.Helper()
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		directory := filepath.Join(output, name)
		if err := os.Mkdir(directory, 0700); err != nil {
			t.Fatal(err)
		}
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", directory)
	}
}

func TestPracticalContractInputsRequireOriginalOwner(t *testing.T) {
	for _, owner := range []*practicalOriginalPackages{nil, {}, {closed: true}, {inputs: &practicalImageInputs{}, loader: &practicalSourceTypes{}}} {
		input, err := owner.contractInputs()
		if err == nil || input.canonical != nil || input.overlays != nil {
			t.Fatal("unowned contract handoff accepted", err)
		}
	}
}

func practicalContractInputEvidence(t *testing.T, input practicalContractInputs) map[string]any {
	t.Helper()
	rows := []any{}
	for _, buffer := range input.overlays {
		rows = append(rows, map[string]any{"object": buffer.object, "raw_hex": hex.EncodeToString(buffer.data)})
	}
	sources := []any{}
	for _, buffer := range input.source.components.sources {
		sources = append(sources, map[string]any{"object": buffer.object, "raw_hex": hex.EncodeToString(buffer.data)})
	}
	return map[string]any{"envelope_hex": hex.EncodeToString(input.canonical), "components_hex": hex.EncodeToString(input.source.components.canonical), "sources": sources, "overlays": rows}
}

func TestPracticalContractInputsOwnEveryCapturedSidecar(t *testing.T) {
	practicalContractTestOutput(t, "owned")
	overlay := func(name, source, function string) []byte {
		raw, err := json.Marshal(map[string]any{"schema": "mpk.go.practical.source_overlay.v1", "source": source, "function": map[string]any{"package": name, "function": function, "receiver": nil}, "termination": "partial", "requires": []any{}, "ensures": []any{map[string]any{"op": "bool.literal", "value": true}}, "error_inputs": []any{}, "loops": []any{}, "modifies": []any{}})
		if err != nil {
			t.Fatal(err)
		}
		return raw
	}
	files := map[string][]byte{
		"go.mod":                []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":             []byte("package policy\nimport \"example.com/policy/helper\"\nfunc Root(x uint16)uint16{return helper.Work(x)}\nfunc Unused(c chan int, f func(int)int)int{return f(<-c)}\nfunc init(){panic(\"must not execute\")}\n"),
		"helper/helper.go":      []byte("package helper\nfunc Work(x uint16)uint16{return x+1}\nfunc Unused(m map[int]int)int{return m[0]}\n"),
		"contracts/root.json":   overlay("example.com/policy", "policy.go", "Root"),
		"contracts/helper.json": overlay("example.com/policy/helper", "helper/helper.go", "Work"),
		"contracts/unused.json": overlay("example.com/policy", "policy.go", "Unused"),
	}
	owner, _ := practicalStorageTestOwnerWithOverlays(t, files, "example.com/policy", []string{"contracts/root.json", "contracts/helper.json", "contracts/unused.json"})
	input, err := owner.contractInputs()
	if err != nil {
		t.Fatal(err)
	}
	if len(input.overlays) != 3 {
		t.Fatal("captured unselected sidecar omitted")
	}
	source, err := owner.sourceInputs()
	if err != nil || !bytes.Equal(source.components.canonical, input.source.components.canonical) {
		t.Fatal("source-only component changed", err)
	}
	retainPracticalSyntaxEvidence(t, "contract-inputs.json", practicalContractInputEvidence(t, input))
	canonical, components := bytes.Clone(input.canonical), bytes.Clone(input.source.components.canonical)
	clear(input.canonical)
	clear(input.source.components.canonical)
	for i := range input.overlays {
		clear(input.overlays[i].data)
		input.overlays[i].object.ID = "caller mutation"
	}
	for i := range input.source.components.sources {
		clear(input.source.components.sources[i].data)
	}
	again, err := owner.contractInputs()
	if err != nil || !bytes.Equal(canonical, again.canonical) || !bytes.Equal(components, again.source.components.canonical) {
		t.Fatal("caller mutated original handoff", err)
	}
	for _, buffer := range again.overlays {
		if !bytes.Equal(buffer.data, files[buffer.object.Path]) {
			t.Fatal("original sidecar bytes changed")
		}
	}
	owner.Close()
	if !bytes.Equal(canonical, again.canonical) {
		t.Fatal("owned packet retired with owner")
	}
	for _, buffer := range again.overlays {
		if !bytes.Equal(buffer.data, files[buffer.object.Path]) {
			t.Fatal("owned sidecar retired with owner")
		}
	}
	if value, err := owner.contractInputs(); err == nil || value.canonical != nil {
		t.Fatal("closed owner emitted contract inputs")
	}
	retainPracticalSyntaxEvidence(t, "contract-ownership.json", map[string]any{"passed": true, "captured_sidecars": 3, "whole_original_files": files, "source_only_component_byte_identical": true, "original_paths_and_donor_buffers_retired": true, "application_initializers_executed": false})
}

func TestPracticalContractInputsRejectDamagedSidecar(t *testing.T) {
	practicalContractTestOutput(t, "damaged")
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte("package policy\nfunc Root(){}\n"), "original.json": []byte("{}\n")}, "example.com/policy", []string{"original.json"})
	original := owner.inputs
	defer func() { owner.inputs = original }()
	rows := []any{}
	for _, control := range []string{"raw-bytes", "missing-list-member", "missing-buffer", "missing-path"} {
		candidate := *original
		candidate.snapshot.ids = append([]string(nil), original.snapshot.ids...)
		candidate.snapshot.buffers = maps.Clone(original.snapshot.buffers)
		candidate.snapshot.paths = maps.Clone(original.snapshot.paths)
		for i, id := range candidate.snapshot.ids {
			buffer := candidate.snapshot.buffers[id]
			if buffer.object.Kind != "overlay" {
				continue
			}
			switch control {
			case "raw-bytes":
				buffer.data = bytes.Clone(buffer.data)
				buffer.data[0] = '['
				candidate.snapshot.buffers[id] = buffer
			case "missing-list-member":
				candidate.snapshot.ids = append(candidate.snapshot.ids[:i], candidate.snapshot.ids[i+1:]...)
			case "missing-buffer":
				delete(candidate.snapshot.buffers, id)
			case "missing-path":
				delete(candidate.snapshot.paths, practicalCaptureKey(buffer.object.Capability, buffer.object.Path))
			}
			break
		}
		owner.inputs = &candidate
		value, err := owner.contractInputs()
		if err == nil || value.canonical != nil || candidate.failed == nil {
			t.Fatal("damaged captured annotation accepted", control, err)
		}
		_, sticky := owner.contractInputs()
		if sticky != candidate.failed {
			t.Fatal("damaged sidecar failure did not remain sticky")
		}
		rows = append(rows, map[string]any{"control": control, "error": fmt.Sprint(err), "sticky": true})
	}
	retainPracticalSyntaxEvidence(t, "damaged-contract-input.json", map[string]any{"rejected": true, "controls": rows})
}

func TestPracticalContractInputsNoAnnotationsKeepDefaults(t *testing.T) {
	practicalContractTestOutput(t, "empty")
	owner, _ := practicalStorageTestOwner(t, map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte("package policy\nfunc Root(x uint16)uint16{return x}\nfunc Unused(c chan int)int{return <-c}\nfunc init(){panic(\"must not execute\")}\n")})
	packet, err := owner.contractInputs()
	if err != nil || len(packet.overlays) != 0 {
		t.Fatal("empty captured annotation set changed", err)
	}
	retainPracticalSyntaxEvidence(t, "contract-inputs-empty.json", practicalContractInputEvidence(t, packet))
}
