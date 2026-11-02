//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestPracticalAnalysisBuildListDifferential(t *testing.T) {
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
	wire, err := snapshot.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	build := inputs.buildSources()
	if len(wire.sources) != 213 || len(inputs.sources) != 65 || len(inputs.vendor) != 133 || len(build) != 201 {
		t.Fatal("original full analysis build input set differs")
	}
	var objects []practicalSourceObject
	seen := map[string]bool{}
	for i, source := range build {
		if seen[source.object.Path] || i > 0 && build[i-1].object.Path >= source.object.Path {
			t.Fatal("build input path order or uniqueness differs")
		}
		seen[source.object.Path] = true
		raw, exists := snapshot.sourceBytes(source.object.ID)
		if !exists || !bytes.Equal(raw, source.sourceBytes()) {
			t.Fatal("build input is not an original captured source")
		}
		objects = append(objects, source.object)
	}
	if out := os.Getenv("MPK_GO_ANALYSIS_BUILD_LIST_OUTPUT"); out != "" {
		data, err := json.MarshalIndent(map[string]any{"snapshot_hex": hex.EncodeToString(wire.canonical), "source_ids": snapshot.ids, "build_objects": objects, "original_sources": len(wire.sources), "frontend_go_files": len(inputs.sources), "vendor_files": len(inputs.vendor), "root_or_binary_authority_claimed": false}, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(data, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}
