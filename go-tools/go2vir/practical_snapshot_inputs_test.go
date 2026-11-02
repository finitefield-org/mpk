//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"io/fs"
	"maps"
	"os"
	"testing"
)

func practicalSnapshotInputEvidence(input practicalSnapshotInputs, inline bool) map[string]any {
	sources := []any{}
	for _, source := range input.sources {
		row := map[string]any{"object": source.object}
		if inline {
			row["raw_hex"] = hex.EncodeToString(source.data)
		} else {
			row["raw_file"] = source.object.ID + ".raw"
		}
		sources = append(sources, row)
	}
	return map[string]any{"snapshot_hex": hex.EncodeToString(input.canonical), "snapshot_sources": sources}
}
func practicalCaptureInputEvidence(t *testing.T, input practicalCaptureInputs) map[string]any {
	t.Helper()
	packet := practicalSelectionInputEvidence(t, input.selection)
	for key, value := range practicalSnapshotInputEvidence(input.snapshot, false) {
		packet[key] = value
	}
	packet["capture_hex"] = hex.EncodeToString(input.canonical)
	return packet
}

func TestPracticalSnapshotInputsRetainWholeNativeNamespace(t *testing.T) {
	practicalContractTestOutput(t, "namespace")
	fixture := preparePracticalSnapshotFixture(t, false)
	input, err := fixture.snapshot.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "snapshot-inputs.json", practicalSnapshotInputEvidence(input, true))
	for _, root := range fixture.roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	queries := []struct {
		name, capability, relative, kind, outcome string
		directory                                 bool
	}{
		{"captured", "workspace/main", "policy.go", "go_file", "present", false},
		{"empty-original", "workspace/main", "go.sum", "go_sum", "present", false},
		{"other-capability", "workspace/dep", "policy.go", "go_file", "present", false},
		{"known-directory", "workspace/main", "exports", "", "present", true},
		{"empty-directory", "workspace/main", "empty", "", "present", true},
		{"absent-leaf", "workspace/main", "absent.go", "go_file", "absent", false},
		{"absent-ancestor", "workspace/main", "absent/child.go", "go_file", "absent", false},
		{"absent-known-directory", "workspace/main", "exports/absent.a", "export_data", "absent", false},
		{"absent-directory", "workspace/main", "absent", "", "absent", true},
		{"unread-file", "workspace/main", "unread.go", "go_file", "uncaptured", false},
		{"unvisited-directory", "workspace/main", "pending", "", "uncaptured", true},
		{"unvisited-present", "workspace/main", "pending/uncaptured.go", "go_file", "uncaptured", false},
		{"unvisited-unknown", "workspace/main", "pending/absent.go", "go_file", "uncaptured", false},
		{"unvisited-root", "workspace/unused", "", "", "uncaptured", true},
		{"unvisited-root-unknown", "workspace/unused", "absent.go", "go_file", "uncaptured", false},
		{"unbound", "workspace/foreign", "policy.go", "go_file", "rejected", false},
		{"wrong-role", "workspace/main", "policy.go", "metadata", "rejected", false},
		{"unknown-role", "workspace/main", "policy.go", "caller-kind", "rejected", false},
		{"wrong-case", "workspace/main", "POLICY.go", "go_file", "rejected", false},
		{"linked-file", "workspace/main", "linked.go", "go_file", "rejected", false},
		{"linked-directory", "workspace/main", "linked-dir", "", "rejected", true},
		{"linked-ancestor", "workspace/main", "linked-dir/uncaptured.go", "go_file", "rejected", false},
		{"special", "workspace/main", "special.go", "go_file", "rejected", false},
		{"file-as-directory", "workspace/main", "policy.go", "", "rejected", true},
		{"directory-as-file", "workspace/main", "exports", "go_file", "rejected", false},
		{"escape", "workspace/main", "../policy.go", "go_file", "rejected", false},
		{"absolute", "workspace/main", "/policy.go", "go_file", "rejected", false},
		{"separator", "workspace/main", "exports//pkg.a", "export_data", "rejected", false},
		{"native-backslash", "workspace/main", "native\\asset.bin", "auxiliary", "rejected", false},
		{"native-newline", "workspace/main", "ignored\nasset.bin", "auxiliary", "rejected", false},
	}
	observed := []any{}
	for _, query := range queries {
		var err error
		if query.directory {
			_, err = fixture.snapshot.readDirectory(query.capability, query.relative)
		} else {
			_, err = fixture.snapshot.readSource(query.capability, query.relative, query.kind)
		}
		outcome := "rejected"
		if err == nil {
			outcome = "present"
		} else if errors.Is(err, fs.ErrNotExist) {
			outcome = "absent"
		} else if errors.Is(err, errPracticalNotCaptured) {
			outcome = "uncaptured"
		}
		if outcome != query.outcome {
			t.Fatal(query.name, outcome, err)
		}
		observed = append(observed, map[string]any{"name": query.name, "capability": query.capability, "path": query.relative, "kind": query.kind, "directory": query.directory, "outcome": outcome})
	}
	canonical := bytes.Clone(input.canonical)
	clear(input.canonical)
	for i := range input.sources {
		clear(input.sources[i].data)
	}
	again, err := fixture.snapshot.snapshotInputs()
	if err != nil || !bytes.Equal(canonical, again.canonical) {
		t.Fatal("returned snapshot aliases original", err)
	}
	retainPracticalSyntaxEvidence(t, "namespace-lookups.json", map[string]any{"queries": observed, "physical_original_roots_removed": true, "returned_data_independent": true})
	// Framing units preserve every native byte even on a host which cannot
	// create a particular filename. These units claim no native capture authority.
	raw := practicalSourceSnapshot{roots: []string{"root"}, directories: map[string][]practicalDirectoryEntry{practicalCaptureKey("root", ""): {{Name: "\ufffd"}, {Name: string([]byte{0xfe})}, {Name: string([]byte{0xff})}}}}
	raw.contentSHA256 = practicalSnapshotDigest(raw)
	unit, err := raw.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "raw-native-name-unit.json", practicalSnapshotInputEvidence(unit, true))
}

func TestPracticalCaptureInputsKeepOriginalOwnedUniverse(t *testing.T) {
	practicalContractTestOutput(t, "owned")
	annotation, _ := json.Marshal(map[string]any{"schema": "mpk.go.practical.source_overlay.v1", "source": "policy.go", "function": map[string]any{"package": "example.com/policy", "function": "Root", "receiver": nil}, "termination": "partial", "requires": []any{}, "ensures": []any{map[string]any{"op": "bool.literal", "value": true}}, "error_inputs": []any{}, "loops": []any{}, "modifies": []any{}})
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{
		"go.mod":              []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":           []byte("package policy\nimport \"example.com/policy/helper\"\nfunc Root(x uint16)uint16{return helper.Work(x)}\nfunc Unused(c chan int,f func(int)int)int{return f(<-c)}\nfunc init(){panic(\"must not execute\")}\n"),
		"helper/helper.go":    []byte("package helper\nfunc Work(x uint16)uint16{return x+1}\n"),
		"excluded_windows.go": []byte("//go:build windows\n\npackage policy\n"),
		"policy_test.go":      []byte("package policy\nfunc TestOnly(){}\n"),
		"external_test.go":    []byte("package policy_test\nfunc ExternalTestOnly(){}\n"),
		"platform_windows.s":  []byte("original excluded assembly bytes\n"),
		"contracts/root.json": annotation,
	}, "example.com/policy", []string{"contracts/root.json"})
	defer owner.Close()
	roots := []practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")}
	input, err := owner.captureInputs(roots)
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "capture-inputs.json", practicalCaptureInputEvidence(t, input))
	if len(input.snapshot.sources) != len(owner.inputs.snapshot.ids) {
		t.Fatal("capture omitted original objects")
	}
	original := owner.inputs
	controls := []any{}
	for _, mutation := range []string{"missing-source", "changed-unselected-bytes", "missing-directory"} {
		candidate := *original
		candidate.snapshot.buffers = maps.Clone(original.snapshot.buffers)
		candidate.snapshot.directories = maps.Clone(original.snapshot.directories)
		if mutation == "missing-directory" {
			for key := range candidate.snapshot.directories {
				delete(candidate.snapshot.directories, key)
				break
			}
		} else {
			for id, source := range candidate.snapshot.buffers {
				if source.object.Kind == "excluded_source" {
					if mutation == "missing-source" {
						delete(candidate.snapshot.buffers, id)
					} else {
						source.data = bytes.Clone(source.data)
						source.data[0] ^= 1
						candidate.snapshot.buffers[id] = source
					}
					break
				}
			}
		}
		owner.inputs = &candidate
		if value, err := owner.captureInputs(roots); err == nil || value.canonical != nil {
			t.Fatal(mutation, "accepted")
		}
		if _, err := owner.captureInputs(roots); err == nil || candidate.failed == nil {
			t.Fatal("input failure was not sticky", mutation)
		}
		controls = append(controls, map[string]any{"control": mutation, "rejected_with_sticky_error": true})
	}
	owner.inputs = original
	canonical := bytes.Clone(input.canonical)
	snapshot := bytes.Clone(input.snapshot.canonical)
	clear(input.canonical)
	clear(input.snapshot.canonical)
	for i := range input.snapshot.sources {
		clear(input.snapshot.sources[i].data)
	}
	again, err := owner.captureInputs(roots)
	if err != nil || !bytes.Equal(canonical, again.canonical) || !bytes.Equal(snapshot, again.snapshot.canonical) {
		t.Fatal("capture input mutation crossed owner", err)
	}
	owner.Close()
	if !bytes.Equal(canonical, again.canonical) {
		t.Fatal("capture inputs retired with owner")
	}
	if value, err := owner.captureInputs(roots); err == nil || value.canonical != nil {
		t.Fatal("closed owner accepted")
	}
	retainPracticalSyntaxEvidence(t, "capture-ownership.json", map[string]any{"status": "passed", "complete_objects": len(again.snapshot.sources), "controls": controls, "caller_mutation_isolated": true, "owner_retirement_keeps_inputs": true})
}
