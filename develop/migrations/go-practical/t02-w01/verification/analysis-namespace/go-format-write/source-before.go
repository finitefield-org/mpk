//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

// Observe only namespace-relative reads. A rehashed snapshot is still not
// authenticated as the original filesystem.
func TestPracticalAnalysisNamespaceDifferential(t *testing.T) {
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
	if err != nil {
		t.Fatal(err)
	}
	if len(inputs.vendor) != 133 || len(originalInputs.sources) != 213 {
		t.Fatal("complete original analysis owner differs")
	}
	clone := func() practicalSourceSnapshot {
		copy := practicalSourceSnapshot{buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{}, directories: map[string][]practicalDirectoryEntry{}, ids: append([]string(nil), original.ids...), roots: append([]string(nil), original.roots...)}
		for key, value := range original.buffers {
			copy.buffers[key] = value
		}
		for key, value := range original.paths {
			copy.paths[key] = value
		}
		for key, value := range original.directories {
			copy.directories[key] = append([]practicalDirectoryEntry(nil), value...)
		}
		return copy
	}
	forget := func(snapshot *practicalSourceSnapshot, prefix string, recursive bool) {
		kept := make([]string, 0, len(snapshot.ids))
		for _, id := range snapshot.ids {
			object := snapshot.buffers[id].object
			match := object.Capability == roots[0].Capability && (object.Path == prefix || recursive && strings.HasPrefix(object.Path, prefix+"/"))
			if match {
				delete(snapshot.buffers, id)
				delete(snapshot.paths, practicalCaptureKey(object.Capability, object.Path))
			} else {
				kept = append(kept, id)
			}
		}
		snapshot.ids = kept
	}
	const directory = "vendor/golang.org/x/tools/go/packages"
	const file = directory + "/doc.go"
	addEntry := func(snapshot *practicalSourceSnapshot, name string, mode os.FileMode) {
		key := practicalCaptureKey(roots[0].Capability, directory)
		snapshot.directories[key] = append(snapshot.directories[key], practicalDirectoryEntry{Name: name, Type: mode})
		sort.Slice(snapshot.directories[key], func(i, j int) bool { return snapshot.directories[key][i].Name < snapshot.directories[key][j].Name })
	}
	walk := func(snapshot practicalSourceSnapshot) (int, error) {
		var visit func(string) (int, error)
		visit = func(directory string) (int, error) {
			entries, err := snapshot.readDirectory(roots[0].Capability, directory)
			if err != nil {
				return 0, err
			}
			count := 0
			for _, entry := range entries {
				file := path.Join(directory, entry.Name)
				if entry.Type.IsDir() {
					n, err := visit(file)
					if err != nil {
						return 0, err
					}
					count += n
					continue
				}
				if file == inputs.vendorManifest.object.Path {
					continue
				}
				kind := "auxiliary"
				if strings.HasSuffix(file, ".go") {
					kind = "go_file"
				} else if entry.Name != "LICENSE" && entry.Name != "PATENTS" && entry.Name != "TODO" {
					return 0, fmt.Errorf("unknown vendor input")
				}
				if _, err := snapshot.readSource(roots[0].Capability, file, kind); err != nil {
					return 0, err
				}
				count++
			}
			return count, nil
		}
		return visit("vendor")
	}
	type observation struct {
		Name       string   `json:"name"`
		Snapshot   string   `json:"snapshot_hex"`
		SourceIDs  []string `json:"source_ids"`
		Accepted   bool     `json:"accepted"`
		Files      int      `json:"files"`
	}
	var rows []observation
	for _, test := range []struct {
		name string
		want bool
		files int
	}{
		{"original-complete-snapshot", true, 133},
		{"unread-file", false, 0},
		{"unread-directory", false, 0},
		{"unknown-unread-file", false, 0},
		{"symbolic-link", false, 0},
		{"coherently-pruned-snapshot", true, 132},
	} {
		candidate := clone()
		switch test.name {
		case "unread-file":
			forget(&candidate, file, false)
		case "unread-directory":
			forget(&candidate, directory, true)
			delete(candidate.directories, practicalCaptureKey(roots[0].Capability, directory))
		case "unknown-unread-file":
			addEntry(&candidate, "ghost.go", 0)
		case "symbolic-link":
			addEntry(&candidate, "symlink.go", os.ModeSymlink)
		case "coherently-pruned-snapshot":
			forget(&candidate, file, false)
			key := practicalCaptureKey(roots[0].Capability, directory)
			entries := candidate.directories[key][:0]
			for _, entry := range candidate.directories[key] {
				if entry.Name != "doc.go" {
					entries = append(entries, entry)
				}
			}
			candidate.directories[key] = entries
		}
		candidate.contentSHA256 = practicalSnapshotDigest(candidate)
		input, err := candidate.snapshotInputs()
		if err != nil {
			t.Fatalf("%s: native snapshot import: %v", test.name, err)
		}
		count, err := walk(candidate)
		if (err == nil) != test.want || count != test.files {
			t.Fatalf("%s: files=%d error=%v, want files=%d accepted=%v", test.name, count, err, test.files, test.want)
		}
		rows = append(rows, observation{test.name, hex.EncodeToString(input.canonical), append([]string(nil), candidate.ids...), err == nil, count})
	}
	if out := os.Getenv("MPK_GO_ANALYSIS_NAMESPACE_OUTPUT"); out != "" {
		if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
			t.Fatal(err)
		}
		type sourceRecord struct {
			Object practicalSourceObject `json:"object"`
			RawFile string `json:"raw_file"`
		}
		var sources []sourceRecord
		for _, source := range originalInputs.sources {
			raw := source.sourceBytes()
			file := filepath.Join(out, "original-bytes", source.object.RawSHA256+".raw")
			old, readErr := os.ReadFile(file)
			if os.IsNotExist(readErr) {
				if err := os.WriteFile(file, raw, 0o600); err != nil {
					t.Fatal(err)
				}
			} else if readErr != nil || !bytes.Equal(old, raw) {
				t.Fatal("different original bytes share a digest")
			}
			sources = append(sources, sourceRecord{source.object, source.object.RawSHA256 + ".raw"})
		}
		var archiveIDs []string
		for _, module := range inputs.modules {
			archiveIDs = append(archiveIDs, module.archive.source.object.ID)
		}
		data := map[string]any{"original_sources": sources, "manifest_source_id": inputs.vendorManifest.object.ID, "archive_source_ids": archiveIDs, "cases": rows, "authenticated_original_root_claimed": false}
		encoded, err := json.MarshalIndent(data, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(encoded, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}
