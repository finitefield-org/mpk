//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"testing"
)

func capturePracticalNamespaceInput(t *testing.T, capability, directory string, roles map[string]string) practicalReplaySources {
	t.Helper()
	session := practicalTestSession(t, []practicalSourceRoot{{Capability: capability, Directory: directory}})
	names := make([]string, 0, len(roles))
	for name := range roles {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		if _, err := session.readSource(capability, name, roles[name]); err != nil {
			t.Fatal(err)
		}
	}
	input, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	return input
}

func retainPracticalNamespaceInput(t *testing.T, name string, input practicalReplaySources) {
	t.Helper()
	raw := map[string][]byte{}
	for _, object := range input.snapshot.sourceObjects() {
		raw[object.ID], _ = input.snapshot.sourceBytes(object.ID)
	}
	retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+"."+name, map[string]any{"origins": input.origins, "objects": input.snapshot.sourceObjects(), "raw": raw, "namespace": input.snapshot.directories})
}

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

func TestPracticalReplayNamespaceAllowsCongruentAliasesAndNestedRefinement(t *testing.T) {
	for _, nested := range []bool{false, true} {
		t.Run(fmt.Sprint(nested), func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			outer := roots[0].Directory
			inner := outer
			if nested {
				inner = filepath.Join(outer, "nested")
			}
			source := []byte("package policy\nfunc Root(x int64) int64{return x}\n")
			writePracticalTestFile(t, filepath.Join(outer, "policy.go"), source, 0o600)
			writePracticalTestFile(t, filepath.Join(inner, "policy.go"), source, 0o600)
			first := capturePracticalNamespaceInput(t, "outer", outer, map[string]string{"policy.go": "go_file"})
			second := capturePracticalNamespaceInput(t, "inner", inner, map[string]string{"policy.go": "go_file"})
			retainPracticalNamespaceInput(t, fmt.Sprintf("namespace-congruent-%t-first.json", nested), first)
			retainPracticalNamespaceInput(t, fmt.Sprintf("namespace-congruent-%t-second.json", nested), second)
			image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{first, second}, nil, "")
			if err != nil {
				t.Fatal("consistent original alias/refinement rejected", err)
			}
			defer image.Close()
			view, err := image.openVerifiedView()
			if err != nil {
				t.Fatal(err)
			}
			for _, capability := range []string{"inner", "outer"} {
				buffer, err := view.readCaptured(capability, "policy.go", "go_file")
				if err != nil || !bytes.Equal(buffer.sourceBytes(), source) {
					t.Fatal("original alias lost its own source identity", capability, err)
				}
			}
			if _, err := view.readCaptured("outer", "absent.go", "go_file"); !errors.Is(err, fs.ErrNotExist) {
				t.Fatal("original observed absence changed", err)
			}
			if err := view.check(); err != nil {
				t.Fatal("valid absence invalidated the consumer transaction", err)
			}
			if err := image.Close(); err == nil {
				t.Fatal("owner removed backing while an owned consumer was live")
			}
			if nested {
				if _, err := view.readCaptured("outer", "nested/policy.go", "go_file"); !errors.Is(err, errPracticalNotCaptured) {
					t.Fatal("outer root borrowed an unvisited nested capability's bytes", err)
				}
				if err := view.check(); !errors.Is(err, errPracticalNotCaptured) {
					t.Fatal("ignored missing-capture lookup did not invalidate the transaction", err)
				}
			}
			if err := view.Close(); err != nil {
				t.Fatal(err)
			}
			if err := view.Close(); err != nil {
				t.Fatal("consumer close is not idempotent", err)
			}
			if err := image.Close(); err != nil {
				t.Fatal(err)
			}
			if _, err := image.openVerifiedView(); err == nil {
				t.Fatal("closed original image regained consumer authority")
			}
		})
	}
}

func TestPracticalReplayNamespaceRejectsTypeRoleByteAndParentContradictions(t *testing.T) {
	for _, control := range []string{"unread-added-name", "file-to-directory", "role", "bytes", "absent-parent"} {
		t.Run(control, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			original := roots[0].Directory
			source := []byte("package policy\nfunc Root(x int64) int64{return x}\n")
			writePracticalTestFile(t, filepath.Join(original, "policy.go"), source, 0o600)
			if control == "file-to-directory" {
				writePracticalTestFile(t, filepath.Join(original, "nested"), []byte("unread-original-bytes"), 0o600)
			}
			first := capturePracticalNamespaceInput(t, "first", original, map[string]string{"policy.go": "go_file"})
			retainPracticalNamespaceInput(t, "namespace-"+control+"-first.json", first)
			secondRoot, kind := original, "go_file"
			switch control {
			case "unread-added-name":
				writePracticalTestFile(t, filepath.Join(original, "unread.bin"), []byte("later-unread"), 0o600)
			case "file-to-directory":
				if err := os.Remove(filepath.Join(original, "nested")); err != nil {
					t.Fatal(err)
				}
				secondRoot = filepath.Join(original, "nested")
				writePracticalTestFile(t, filepath.Join(secondRoot, "policy.go"), source, 0o600)
			case "absent-parent":
				secondRoot = filepath.Join(original, "new-root")
				writePracticalTestFile(t, filepath.Join(secondRoot, "policy.go"), source, 0o600)
			case "role":
				kind = "excluded_source"
			case "bytes":
				writePracticalTestFile(t, filepath.Join(original, "policy.go"), bytes.Replace(source, []byte("return x"), []byte("return 0"), 1), 0o600)
			}
			second := capturePracticalNamespaceInput(t, "second", secondRoot, map[string]string{"policy.go": kind})
			retainPracticalNamespaceInput(t, "namespace-"+control+"-second.json", second)
			image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{first, second}, nil, "")
			if image != nil {
				defer image.Close()
			}
			retainPracticalSyntaxEvidence(t, "namespace-"+control+"-result.json", map[string]any{"error": fmt.Sprint(err), "semantic_source_counterexample_claimed": false})
			if err == nil {
				t.Fatal("inconsistent original physical view acquired one replay image")
			}
		})
	}
}
