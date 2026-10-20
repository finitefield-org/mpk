//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

func TestPracticalReplayImageRejectsContradictoryOriginalViews(t *testing.T) {
	base, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	original := filepath.Join(base, "original")
	initial := []byte("package policy\nfunc Root(x int64) int64{return x}\n")
	later := []byte("package policy\nfunc Later(x int64) int64{return x+1}\n")
	writePracticalTestFile(t, filepath.Join(original, "policy.go"), initial, 0o600)
	first := practicalTestSession(t, []practicalSourceRoot{{Capability: "first", Directory: original}})
	if _, err := first.readSource("first", "policy.go", "go_file"); err != nil {
		t.Fatal(err)
	}
	firstInput, err := first.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "namespace-first-original.json", map[string]any{"roots": firstInput.origins, "objects": firstInput.snapshot.sourceObjects(), "policy_bytes": initial, "directory": firstInput.snapshot.directories})
	writePracticalTestFile(t, filepath.Join(original, "later.go"), later, 0o600)
	second := practicalTestSession(t, []practicalSourceRoot{{Capability: "second", Directory: original}})
	for _, name := range []string{"policy.go", "later.go"} {
		if _, err := second.readSource("second", name, "go_file"); err != nil {
			t.Fatal(err)
		}
	}
	secondInput, err := second.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "namespace-second-original.json", map[string]any{"roots": secondInput.origins, "objects": secondInput.snapshot.sourceObjects(), "policy_bytes": initial, "later_bytes": later, "directory": secondInput.snapshot.directories})
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{firstInput, secondInput}, nil, "")
	observation := map[string]any{"error": fmt.Sprint(err), "two_original_epochs": true, "semantic_source_counterexample_claimed": false}
	if image != nil {
		defer image.Close()
		var files []map[string]any
		for _, file := range image.files {
			files = append(files, map[string]any{"original": file.original, "objects": file.objects})
		}
		observation["materialized_files"] = files
	}
	retainPracticalSyntaxEvidence(t, "namespace-contradiction-result.json", observation)
	if err == nil {
		t.Fatal("different complete original directory views were merged into one source image")
	}
	if _, err := os.Stat(filepath.Join(original, "later.go")); err != nil {
		t.Fatal(err)
	}
}
