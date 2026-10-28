//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"testing"
)

// Retain the complete original owner while probing the shared replay budget.
func TestPracticalAnalysisBudgetDifferential(t *testing.T) {
	roots := practicalAnalysisTestRoots(t, "")
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability})
	if err != nil {
		t.Fatal(err)
	}
	original, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	originalInputs, err := original.snapshotInputs()
	if err != nil || len(originalInputs.sources) != 213 || len(inputs.vendor) != 133 || len(inputs.modules) != 3 {
		t.Fatalf("complete original analysis owner differs: %v", err)
	}
	entries, archiveEntries := 0, 0
	for _, directory := range original.directories {
		entries += len(directory)
	}
	for _, module := range inputs.modules {
		archiveEntries += int(module.archive.namespaceEntries)
	}
	if archiveEntries != 2403 || entries+archiveEntries >= maximumDirectoryEntries {
		t.Fatal("original namespace inventory differs")
	}
	type observation struct {
		Name             string `json:"name"`
		SnapshotFile     string `json:"snapshot_file"`
		DirectoryEntries int    `json:"directory_entries"`
		ArchiveEntries   int    `json:"archive_entries"`
		Accepted         bool   `json:"accepted"`
	}
	var rows []observation
	for _, probe := range []struct {
		name string
		padding int
		accepted bool
	}{
		{"original", 0, true},
		{"exact-shared-limit", maximumDirectoryEntries - entries - archiveEntries, true},
		{"one-over-shared-limit", maximumDirectoryEntries - entries - archiveEntries + 1, false},
	} {
		candidate := practicalSourceSnapshot{buffers: original.buffers, paths: original.paths, ids: original.ids, roots: original.roots, directories: map[string][]practicalDirectoryEntry{}}
		for key, directory := range original.directories {
			candidate.directories[key] = append([]practicalDirectoryEntry(nil), directory...)
		}
		key := practicalCaptureKey(roots[0].Capability, "")
		for i := 0; i < probe.padding; i++ {
			candidate.directories[key] = append(candidate.directories[key], practicalDirectoryEntry{Name: fmt.Sprintf("zz_mpk_budget_%06d.data", i)})
		}
		sort.Slice(candidate.directories[key], func(i, j int) bool { return candidate.directories[key][i].Name < candidate.directories[key][j].Name })
		candidate.contentSHA256 = practicalSnapshotDigest(candidate)
		snapshot, err := candidate.snapshotInputs()
		if err != nil {
			t.Fatalf("%s: snapshot: %v", probe.name, err)
		}
		var counters captureCounters
		for _, directory := range candidate.directories {
			if err := counters.addDirectory(len(directory)); err != nil {
				t.Fatalf("%s: directory budget: %v", probe.name, err)
			}
		}
		directoryEntries := int(counters.directoryEntries)
		var archiveErr error
		for _, module := range inputs.modules {
			var archive practicalModuleArchive
			archive, archiveErr = parsePracticalModuleArchive(module.identity, module.archive.source, module.archive.sum, maximumDirectoryEntries-int(counters.directoryEntries))
			if archiveErr != nil {
				break
			}
			counters.directoryEntries += archive.namespaceEntries
		}
		if (archiveErr == nil) != probe.accepted {
			t.Fatalf("%s: accepted=%v error=%v", probe.name, probe.accepted, archiveErr)
		}
		filename := probe.name + ".snapshot.json"
		if out := os.Getenv("MPK_GO_ANALYSIS_BUDGET_OUTPUT"); out != "" {
			if err := os.MkdirAll(out, 0o700); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile(filepath.Join(out, filename), snapshot.canonical, 0o600); err != nil {
				t.Fatal(err)
			}
		}
		rows = append(rows, observation{probe.name, filename, directoryEntries, archiveEntries, archiveErr == nil})
	}
	if out := os.Getenv("MPK_GO_ANALYSIS_BUDGET_OUTPUT"); out != "" {
		encoded, err := json.MarshalIndent(map[string]any{"source_ids": original.ids, "cases": rows, "original_sources": len(originalInputs.sources), "vendor_files": len(inputs.vendor), "original_root_authenticated": false}, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(encoded, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}
