//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func practicalViewTestImage(t *testing.T) (*practicalReplayImage, string, []byte) {
	t.Helper()
	roots, _ := practicalTestCaptureRoots(t)
	source := []byte("package policy\nfunc Root(x int64) int64{return x}\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), source, 0o600)
	input := capturePracticalNamespaceInput(t, "original", roots[0].Directory, map[string]string{"policy.go": "go_file"})
	retainPracticalNamespaceInput(t, "view-original-input.json", input)
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{input}, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	return image, filepath.Join(roots[0].Directory, "policy.go"), source
}

func TestPracticalReplayViewRejectsChangedBackingAndSafeCleanup(t *testing.T) {
	for _, control := range []string{"bytes", "missing", "mode", "extra-file", "extra-directory", "symlink"} {
		t.Run(control, func(t *testing.T) {
			image, original, source := practicalViewTestImage(t)
			name := image.privatePath(original)
			parent := filepath.Dir(name)
			if err := os.Chmod(parent, 0o755); err != nil {
				t.Fatal(err)
			}
			outside := filepath.Join(t.TempDir(), "outside")
			writePracticalTestFile(t, outside, []byte("outside-unmodified"), 0o444)
			switch control {
			case "bytes":
				if err := os.Chmod(name, 0o644); err != nil {
					t.Fatal(err)
				}
				if err := os.WriteFile(name, bytes.Replace(source, []byte("return x"), []byte("return 0"), 1), 0o444); err != nil {
					t.Fatal(err)
				}
				if err := os.Chmod(name, 0o444); err != nil {
					t.Fatal(err)
				}
			case "missing":
				if err := os.Remove(name); err != nil {
					t.Fatal(err)
				}
			case "mode":
				if err := os.Chmod(name, 0o644); err != nil {
					t.Fatal(err)
				}
			case "extra-file":
				if err := os.WriteFile(filepath.Join(parent, "extra_windows.go"), []byte("package policy\n"), 0o444); err != nil {
					t.Fatal(err)
				}
			case "extra-directory":
				if err := os.Mkdir(filepath.Join(parent, "extra"), 0o555); err != nil {
					t.Fatal(err)
				}
			case "symlink":
				if err := os.Symlink(outside, filepath.Join(parent, "outside-link")); err != nil {
					t.Fatal(err)
				}
			}
			if err := os.Chmod(parent, 0o555); err != nil {
				t.Fatal(err)
			}
			view, err := image.openVerifiedView()
			if view != nil {
				_ = view.Close()
			}
			retainPracticalSyntaxEvidence(t, "view-"+control+"-result.json", map[string]any{"error": fmt.Sprint(err), "original_bytes": source, "semantic_source_counterexample_claimed": false})
			if err == nil || errors.Is(err, fs.ErrNotExist) {
				t.Fatal("changed backing acquired authority or was relabeled as original absence", err)
			}
			if err := image.Close(); err != nil {
				t.Fatal(err)
			}
			info, err := os.Stat(outside)
			if err != nil || info.Mode().Perm() != 0o444 {
				t.Fatal("cleanup followed a link and changed outside permissions", err)
			}
			if raw, err := os.ReadFile(outside); err != nil || string(raw) != "outside-unmodified" {
				t.Fatal("cleanup changed outside bytes", err)
			}
		})
	}
}

func TestPracticalReplayViewOwnsMetadataAndInvalidatesIgnoredFailures(t *testing.T) {
	image, _, source := practicalViewTestImage(t)
	defer image.Close()
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	image.files[0].objects[0].RawSHA256 = "changed-after-opening"
	image.mounts[0].original = "/changed-after-opening"
	for _, directory := range image.namespace {
		if len(directory.entries) != 0 {
			directory.entries[0].Name = "changed-after-opening"
		}
	}
	buffer, err := view.readCaptured("original", "policy.go", "go_file")
	if err != nil || !bytes.Equal(buffer.sourceBytes(), source) {
		t.Fatal("consumer metadata aliased mutable producer projections", err)
	}
	if err := view.check(); err != nil {
		t.Fatal(err)
	}
	_, invalid := view.readCaptured("original", "absent.go", "unknown-role")
	if invalid == nil || errors.Is(invalid, fs.ErrNotExist) {
		t.Fatal("invalid source kind was relabeled as valid observed absence", invalid)
	}
	if err := view.check(); err != invalid {
		t.Fatal("ignored invalid input still allowed consumer validation", err, invalid)
	}
	if _, err := view.readCaptured("original", "policy.go", "go_file"); err != invalid {
		t.Fatal("ignored failure did not invalidate subsequent source access", err)
	}
	if err := view.Close(); err != nil {
		t.Fatal(err)
	}
	if err := view.check(); err == nil {
		t.Fatal("closed consumer view regained validation authority")
	}
}

func TestPracticalReplayViewRejectsMutationAfterOpeningAndRestoration(t *testing.T) {
	image, original, source := practicalViewTestImage(t)
	defer image.Close()
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	name := image.privatePath(original)
	if err := os.Chmod(name, 0o644); err != nil {
		t.Fatal(err)
	}
	changed := bytes.Replace(source, []byte("return x"), []byte("return 0"), 1)
	if err := os.WriteFile(name, changed, 0o444); err != nil {
		t.Fatal(err)
	}
	if err := os.Chmod(name, 0o444); err != nil {
		t.Fatal(err)
	}
	_, failure := view.readCaptured("original", "policy.go", "go_file")
	retainPracticalSyntaxEvidence(t, "view-opened-mutation.json", map[string]any{"original": source, "changed": changed, "error": fmt.Sprint(failure), "semantic_source_counterexample_claimed": false})
	if failure == nil || errors.Is(failure, fs.ErrNotExist) {
		t.Fatal("opened consumer borrowed changed bytes or claimed original absence", failure)
	}
	if err := os.Chmod(name, 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(name, source, 0o444); err != nil {
		t.Fatal(err)
	}
	if err := os.Chmod(name, 0o444); err != nil {
		t.Fatal(err)
	}
	if err := view.check(); err != failure {
		t.Fatal("restoring producer bytes erased an ignored consumer failure", err, failure)
	}
}

func TestPracticalReplayViewRejectsSpecialPermissionBits(t *testing.T) {
	for _, directory := range []bool{false, true} {
		for _, special := range []os.FileMode{os.ModeSetuid, os.ModeSetgid, os.ModeSticky} {
			t.Run(fmt.Sprintf("directory-%t-mode-%d", directory, special), func(t *testing.T) {
				image, original, _ := practicalViewTestImage(t)
				defer image.Close()
				name := image.privatePath(original)
				mode := os.FileMode(0o444)
				if directory {
					name, mode = filepath.Dir(name), 0o555
				}
				if err := os.Chmod(name, mode|special); err != nil {
					t.Fatal(err)
				}
				info, err := os.Lstat(name)
				if err == nil && info.Mode()&special == 0 && runtime.GOOS == "darwin" {
					retainPracticalSyntaxEvidence(t, fmt.Sprintf("view-special-%t-%d.json", directory, special), map[string]any{
						"directory": directory, "special_mode": special, "actual_mode": info.Mode(), "materialized": false,
						"limitation": "Darwin backing cleared the requested special bit; complete controls execute on Linux tmpfs",
					})
					t.Skip("Darwin backing cleared the requested special bit; validated separately on Linux tmpfs")
				}
				if err != nil || info.Mode()&special == 0 {
					t.Fatal("special permission control was not materialized", err)
				}
				view, failure := image.openVerifiedView()
				if view != nil {
					_ = view.Close()
				}
				retainPracticalSyntaxEvidence(t, fmt.Sprintf("view-special-%t-%d.json", directory, special), map[string]any{
					"directory": directory, "special_mode": special, "actual_mode": info.Mode(), "error": fmt.Sprint(failure),
					"materialized": true,
					"semantic_source_counterexample_claimed": false,
				})
				if failure == nil || errors.Is(failure, fs.ErrNotExist) {
					t.Fatal("special permission bits acquired sealed image authority", failure)
				}
			})
		}
	}
}

func TestPracticalReplayViewRejectsModeChangeAfterOpening(t *testing.T) {
	image, original, _ := practicalViewTestImage(t)
	defer image.Close()
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	name := image.privatePath(original)
	if err := os.Chmod(name, 0o444|os.ModeSticky); err != nil {
		t.Fatal(err)
	}
	_, failure := view.readCaptured("original", "policy.go", "go_file")
	if failure == nil || errors.Is(failure, fs.ErrNotExist) {
		t.Fatal("opened view accepted changed source mode", failure)
	}
	if err := os.Chmod(name, 0o444); err != nil {
		t.Fatal(err)
	}
	if err := view.check(); err != failure {
		t.Fatal("restoring permissions erased an ignored consumer failure", err, failure)
	}
}
