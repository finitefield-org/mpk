//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"golang.org/x/mod/module"
)

// Reuse the exact original versioned application capture and its info bytes.
func TestPracticalModuleInfoDifferential(t *testing.T) {
	root := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "module-observations", "native", "native-originals", "versioned-owned")
	data, err := os.ReadFile(filepath.Join(root, "complete-original-module-observations.json"))
	if err != nil {
		t.Fatal(err)
	}
	type sourceRecord struct {
		Object  practicalSourceObject `json:"object"`
		RawFile string                `json:"raw_file"`
	}
	var packet struct {
		SnapshotSources []sourceRecord `json:"snapshot_sources"`
	}
	if err := json.Unmarshal(data, &packet); err != nil {
		t.Fatal(err)
	}
	if len(packet.SnapshotSources) < 5000 {
		t.Fatal("original application and SDK capture narrowed")
	}
	const infoPath = "cache/download/example.com/!dep/@v/v1.2.0.info"
	var record *sourceRecord
	for i := range packet.SnapshotSources {
		if packet.SnapshotSources[i].Object.Capability == "workspace/dep" && packet.SnapshotSources[i].Object.Path == infoPath {
			if record != nil {
				t.Fatal("duplicate original info")
			}
			record = &packet.SnapshotSources[i]
		}
	}
	if record == nil {
		t.Fatal("original cached info absent")
	}
	raw, err := os.ReadFile(filepath.Join(root, "original-source-objects", record.RawFile))
	if err != nil {
		t.Fatal(err)
	}
	original, err := newPracticalSourceBuffer(record.Object.Capability, record.Object.Path, record.Object.Kind, raw)
	if err != nil || original.object != record.Object {
		t.Fatal("original metadata bytes differ", err)
	}
	identity := module.Version{Path: "example.com/Dep", Version: "v1.2.0"}
	replace := func(old, new string) []byte {
		t.Helper()
		if !bytes.Contains(raw, []byte(old)) {
			t.Fatal("original metadata anchor missing")
		}
		return bytes.Replace(raw, []byte(old), []byte(new), 1)
	}
	add := func(field string) []byte {
		t.Helper()
		at := bytes.LastIndexByte(raw, '}')
		if at < 0 {
			t.Fatal("original object end missing")
		}
		return bytes.Join([][]byte{raw[:at], []byte("," + field), raw[at:]}, nil)
	}
	type probe struct {
		name     string
		data     []byte
		accepted bool
	}
	probes := []probe{
		{"original", raw, true},
		{"lowercase-version", replace(`"Version"`, `"version"`), true},
		{"unicode-version", replace(`"Version"`, `"Verſion"`), true},
		{"wrong-version", replace(`v1.2.0`, `v1.2.1`), false},
		{"duplicate-version", replace(`"Version":"v1.2.0"`, `"Version":"v1.2.0","Version":"v1.2.0"`), false},
		{"earlier-alias", replace(`"Version":"v1.2.0"`, `"version":"v0.0.0","Version":"v1.2.0"`), true},
		{"later-alias", replace(`"Version":"v1.2.0"`, `"Version":"v1.2.0","version":"v0.0.0"`), false},
		{"time-number", replace(`"Time":"2020-01-02T03:04:05Z"`, `"Time":17`), false},
		{"time-invalid", replace(`2020-01-02T03:04:05Z`, `2020-02-30T03:04:05Z`), false},
		{"time-escaped", replace(`2020-01-02T03:04:05Z`, `2020-01-02T03:04:05\u005a`), false},
		{"time-null", replace(`"Time":"2020-01-02T03:04:05Z"`, `"Time":null`), true},
		{"time-offset-boundary", replace(`2020-01-02T03:04:05Z`, `2020-01-02T03:04:05+24:60`), true},
		{"origin-invalid", add(`"Origin":{"Hash":true}`), false},
		{"unknown-data", add(`"Extra":{"opaque":[true,null,1]}`), true},
	}
	type observation struct {
		Name     string       `json:"name"`
		Source   sourceRecord `json:"source"`
		Accepted bool         `json:"accepted"`
	}
	out := os.Getenv("MPK_GO_MODULE_INFO_OUTPUT")
	if out != "" {
		if err := os.MkdirAll(filepath.Join(out, "observed-bytes"), 0o700); err != nil {
			t.Fatal(err)
		}
	}
	var observations []observation
	for _, probe := range probes {
		source, err := newPracticalSourceBuffer(record.Object.Capability, infoPath, "metadata", probe.data)
		if err != nil {
			t.Fatal(err)
		}
		err = validatePracticalModuleInfo(source, identity)
		if (err == nil) != probe.accepted {
			t.Fatalf("%s: %v", probe.name, err)
		}
		filename := source.object.RawSHA256 + ".raw"
		if out != "" {
			file := filepath.Join(out, "observed-bytes", filename)
			old, readErr := os.ReadFile(file)
			if os.IsNotExist(readErr) {
				if err := os.WriteFile(file, source.sourceBytes(), 0o600); err != nil {
					t.Fatal(err)
				}
			} else if readErr != nil || !bytes.Equal(old, source.data) {
				t.Fatal("inconsistent source bytes")
			}
		}
		observations = append(observations, observation{probe.name, sourceRecord{source.object, filename}, err == nil})
	}
	if out != "" {
		data, err := json.MarshalIndent(map[string]any{"observations": observations, "original_source_id": record.Object.ID, "original_source_objects": len(packet.SnapshotSources), "original_info_path": infoPath, "original_root_or_vcs_authority": false}, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(data, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}
