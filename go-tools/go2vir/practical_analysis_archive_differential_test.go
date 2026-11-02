//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// Capture the entire original frontend and all three complete analysis ZIPs.
// Controls copy the complete original tree and change only the named input.
func TestPracticalAnalysisArchiveDifferential(t *testing.T) {
	roots := practicalAnalysisTestRoots(t, "")
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability})
	if err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	if len(inputs.modules) != 3 || len(inputs.vendor) != 133 || len(inputs.differences) != 1 {
		t.Fatal("complete original analysis owner differs")
	}
	packages, err := parsePracticalAnalysisVendorManifest(inputs.vendorManifest)
	if err != nil || len(packages) != 29 {
		t.Fatalf("complete original analysis package manifest: %v", err)
	}
	type sourceRecord struct {
		Object  practicalSourceObject `json:"object"`
		RawFile string                `json:"raw_file"`
	}
	type archiveRecord struct {
		Module           string                   `json:"module"`
		Version          string                   `json:"version"`
		SourceID         string                   `json:"source_id"`
		Checksum         string                   `json:"checksum"`
		NamespaceEntries uint64                   `json:"namespace_entries"`
		Members          []practicalArchiveMember `json:"members"`
	}
	type control struct {
		Name     string `json:"name"`
		Accepted bool   `json:"accepted"`
	}
	out := os.Getenv("MPK_GO_ANALYSIS_ARCHIVE_OUTPUT")
	if out != "" {
		if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
			t.Fatal(err)
		}
	}
	writeBytes := func(hash string, raw []byte) {
		t.Helper()
		if out == "" {
			return
		}
		file := filepath.Join(out, "original-bytes", hash+".raw")
		previous, readErr := os.ReadFile(file)
		if os.IsNotExist(readErr) {
			if err := os.WriteFile(file, raw, 0o600); err != nil {
				t.Fatal(err)
			}
		} else if readErr != nil || !bytes.Equal(previous, raw) {
			t.Fatal("different original bytes share a digest")
		}
	}
	var sources []sourceRecord
	for _, object := range snapshot.sourceObjects() {
		raw, exists := snapshot.sourceBytes(object.ID)
		if !exists || sha256Hex(raw) != object.RawSHA256 {
			t.Fatal("complete sealed original source bytes changed")
		}
		writeBytes(object.RawSHA256, raw)
		sources = append(sources, sourceRecord{object, object.RawSHA256 + ".raw"})
	}
	var vendorIDs []string
	for _, source := range inputs.vendor {
		vendorIDs = append(vendorIDs, source.object.ID)
	}
	var archives []archiveRecord
	for _, module := range inputs.modules {
		archive := module.archive
		archives = append(archives, archiveRecord{module.identity.Path, module.identity.Version, archive.source.object.ID, archive.sum, archive.namespaceEntries, archive.memberInventory()})
	}
	var controls []control
	for _, test := range []struct {
		name string
		want bool
	}{
		{"changed-vendor-go", false},
		{"missing-only-package-file", false},
		{"unknown-asset", false},
		{"omitted-one-of-several-go-files", true},
		{"changed-todo", true},
	} {
		destination := filepath.Join(t.TempDir(), "complete-frontend")
		if err := copyToolchainFixturePath(".", destination); err != nil {
			t.Fatal(err)
		}
		modify := func(relative string, change func([]byte) []byte) {
			t.Helper()
			file := filepath.Join(destination, filepath.FromSlash(relative))
			data, err := os.ReadFile(file)
			if err != nil {
				t.Fatal(err)
			}
			if err := os.Chmod(file, 0o600); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile(file, change(data), 0o600); err != nil {
				t.Fatal(err)
			}
		}
		switch test.name {
		case "changed-vendor-go":
			modify("vendor/golang.org/x/tools/go/packages/packages.go", func(raw []byte) []byte { return append(raw, '\n') })
		case "missing-only-package-file":
			if err := os.Remove(filepath.Join(destination, "vendor/golang.org/x/sync/errgroup/errgroup.go")); err != nil {
				t.Fatal(err)
			}
		case "unknown-asset":
			writePracticalTestFile(t, filepath.Join(destination, "vendor/golang.org/x/tools/go/packages/caller.data"), []byte("caller"), 0o600)
		case "omitted-one-of-several-go-files":
			if err := os.Remove(filepath.Join(destination, "vendor/golang.org/x/tools/go/packages/doc.go")); err != nil {
				t.Fatal(err)
			}
		case "changed-todo":
			modify("vendor/golang.org/x/tools/go/ssa/TODO", func(raw []byte) []byte { return append(raw, []byte("\nextra\n")...) })
		}
		controlRoots := practicalAnalysisTestRoots(t, destination)
		controlSession := practicalTestSession(t, controlRoots)
		_, captureErr := controlSession.capturePracticalAnalysis(practicalModuleLocation{capability: controlRoots[0].Capability}, practicalModuleLocation{capability: controlRoots[1].Capability})
		accepted := captureErr == nil
		if accepted {
			_, captureErr = controlSession.seal()
			accepted = captureErr == nil
		}
		controls = append(controls, control{test.name, accepted})
		if accepted != test.want {
			t.Fatalf("%s: accepted=%v, want %v: %v", test.name, accepted, test.want, captureErr)
		}
	}
	var changed practicalSourceBuffer
	for _, source := range inputs.vendor {
		if strings.HasSuffix(source.object.Path, "/go/packages/packages.go") {
			raw := append(source.sourceBytes(), '\n')
			changed, err = newPracticalSourceBuffer(source.object.Capability, source.object.Path, source.object.Kind, raw)
			if err != nil {
				t.Fatal(err)
			}
			writeBytes(changed.object.RawSHA256, raw)
		}
	}
	if changed.object.ID == "" {
		t.Fatal("original changed-source control missing")
	}
	archive := inputs.modules[2].archive
	badZIP := archive.source.sourceBytes()
	badZIP[len(badZIP)-1] ^= 1
	changedArchive, err := newPracticalSourceBuffer(archive.source.object.Capability, archive.source.object.Path, archive.source.object.Kind, badZIP)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := parsePracticalModuleArchive(inputs.modules[2].identity, changedArchive, archive.sum, int(maximumDirectoryEntries)); err == nil {
		t.Fatal("altered complete archive was accepted")
	}
	writeBytes(changedArchive.object.RawSHA256, badZIP)
	controls = append(controls, control{"changed-archive", false})
	if out != "" {
		data := map[string]any{"snapshot_sources": sources, "snapshot_sha256": snapshot.contentSHA256, "manifest_source_id": inputs.vendorManifest.object.ID, "vendor_source_ids": vendorIDs, "archives": archives, "original_differences": len(inputs.differences), "controls": controls, "changed_vendor_source": sourceRecord{changed.object, changed.object.RawSHA256 + ".raw"}, "changed_archive_source": sourceRecord{changedArchive.object, changedArchive.object.RawSHA256 + ".raw"}, "context_or_source_binary_proof": false}
		encoded, err := json.MarshalIndent(data, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(encoded, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
	t.Logf("complete original analysis capture: %d sources, %d vendor files, %d packages, %d archives; %d controls", len(sources), len(inputs.vendor), len(packages), len(archives), len(controls))
}
