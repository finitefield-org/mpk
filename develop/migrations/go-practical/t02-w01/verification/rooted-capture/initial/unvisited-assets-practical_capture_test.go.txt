//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"syscall"
	"testing"
)

func practicalTestCaptureRoots(t *testing.T) ([]practicalSourceRoot, string) {
	t.Helper()
	base, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	roots := []practicalSourceRoot{{Capability: "workspace/main", Directory: filepath.Join(base, "main")}, {Capability: "workspace/dep", Directory: filepath.Join(base, "dep")}}
	for _, root := range roots {
		if err := os.Mkdir(root.Directory, 0o700); err != nil {
			t.Fatal(err)
		}
	}
	return roots, base
}

func writePracticalTestFile(t *testing.T, name string, data []byte, mode os.FileMode) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(name), 0o700); err != nil {
		t.Fatal(err)
	}
	writeTestFile(t, name, data, mode)
}

func practicalTestSession(t *testing.T, roots []practicalSourceRoot) *practicalCaptureSession {
	t.Helper()
	session, err := newPracticalCaptureSession(roots)
	if err != nil {
		t.Fatal(err)
	}
	return session
}

func TestPracticalCaptureOriginalModulesAndOwnedSnapshot(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	files := []struct {
		root int
		name string
		kind string
		data []byte
	}{
		{0, "go.mod", "go_mod", []byte("module example.com/policy\n\ngo 1.23\nrequire example.com/dep v0.0.0\nreplace example.com/dep => ../dep\n")},
		{0, "policy.go", "go_file", []byte("package policy\nimport \"example.com/dep\"\nfunc Root(x int64) int64 { return dep.Scale(x) }\nfunc Unused(m map[string]int) int { return len(m) }\n")},
		{0, "go.sum", "go_sum", []byte{}},
		{1, "go.mod", "go_mod", []byte("module example.com/dep\n\ngo 1.23\n")},
		{1, "policy.go", "go_file", []byte("package dep\nfunc Scale(x int64) int64 { return x*2 }\n")},
	}
	for _, file := range files {
		writePracticalTestFile(t, filepath.Join(roots[file.root].Directory, file.name), file.data, 0o600)
	}
	// Unvisited assets are not source-capture candidates.
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "assets", "large.bin"), nil, 0o600)
	if err := os.Truncate(filepath.Join(roots[0].Directory, "assets", "large.bin"), maximumCandidateBytes+1); err != nil {
		t.Fatal(err)
	}
	for _, name := range []string{"ignored\nasset.bin", "native\\asset.bin"} {
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, name), []byte("not a loader candidate"), 0o600)
	}
	session := practicalTestSession(t, roots)
	entries, err := session.readDirectory(roots[0].Capability, "")
	if err != nil || len(entries) != 6 {
		t.Fatalf("original module namespace: %v, %v", entries, err)
	}
	entries[0].Name = "caller-mutated"
	for _, file := range files {
		buffer, err := session.readSource(roots[file.root].Capability, file.name, file.kind)
		if err != nil || !bytes.Equal(buffer.sourceBytes(), file.data) {
			t.Fatalf("original %s: %v", file.name, err)
		}
	}
	before := session.counters
	if _, err := session.readSource(roots[0].Capability, "policy.go", "go_file"); err != nil || session.counters != before {
		t.Fatal("repeated source capture charged the same input twice")
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	objects := snapshot.sourceObjects()
	if len(objects) != len(files) || !sort.SliceIsSorted(objects, func(i, j int) bool { return objects[i].ID < objects[j].ID }) {
		t.Fatal("snapshot does not contain every original file in source-ID order")
	}
	for _, object := range objects {
		var expected []byte
		var expectedKind string
		found := false
		for _, file := range files {
			if object.Capability == roots[file.root].Capability && object.Path == file.name {
				expected = file.data
				expectedKind = file.kind
				found = true
			}
		}
		data, exists := snapshot.sourceBytes(object.ID)
		if !found || !exists || object.Kind != expectedKind || !bytes.Equal(data, expected) || object.RawSHA256 != sha256Hex(expected) {
			t.Fatalf("original bytes missing for %s/%s", object.Capability, object.Path)
		}
		if len(data) != 0 {
			data[0] ^= 1
		}
		again, _ := snapshot.sourceBytes(object.ID)
		if !bytes.Equal(again, expected) {
			t.Fatal("snapshot bytes alias a caller buffer")
		}
	}
	second := practicalTestSession(t, []practicalSourceRoot{roots[1], roots[0]})
	for i := len(files) - 1; i >= 0; i-- {
		file := files[i]
		if _, err := second.readSource(roots[file.root].Capability, file.name, file.kind); err != nil {
			t.Fatal(err)
		}
	}
	reordered, err := second.seal()
	if err != nil || !reflect.DeepEqual(reordered.sourceObjects(), objects) {
		t.Fatalf("source identities depend on read/root order: %v", err)
	}
	objects[0].ID = "caller-mutated"
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), []byte("package changed\n"), 0o600)
	for _, object := range snapshot.sourceObjects() {
		if object.Capability == roots[0].Capability && object.Path == "policy.go" {
			data, _ := snapshot.sourceBytes(object.ID)
			if !bytes.Equal(data, files[1].data) {
				t.Fatal("sealed snapshot reread changed source files")
			}
		}
	}
	if _, err := session.readSource(roots[0].Capability, "policy.go", "go_file"); err == nil {
		t.Fatal("sealed capture accepted more reads")
	}
}

func TestPracticalCaptureRejectsObservedSourceAndNamespaceChanges(t *testing.T) {
	for _, change := range []string{"content-with-restored-mtime", "removed-file", "new-directory-entry", "replaced-root"} {
		t.Run(change, func(t *testing.T) {
			roots, base := practicalTestCaptureRoots(t)
			file := filepath.Join(roots[0].Directory, "nested", "policy.go")
			writePracticalTestFile(t, file, []byte("package policy\n"), 0o600)
			session := practicalTestSession(t, roots)
			if _, err := session.readSource(roots[0].Capability, "nested/policy.go", "go_file"); err != nil {
				t.Fatal(err)
			}
			info, err := os.Stat(file)
			if err != nil {
				t.Fatal(err)
			}
			switch change {
			case "content-with-restored-mtime":
				writePracticalTestFile(t, file, []byte("package forged\n"), 0o600)
				if err := os.Chtimes(file, info.ModTime(), info.ModTime()); err != nil {
					t.Fatal(err)
				}
			case "removed-file":
				if err := os.Remove(file); err != nil {
					t.Fatal(err)
				}
			case "new-directory-entry":
				writePracticalTestFile(t, filepath.Join(filepath.Dir(file), "new.go"), []byte("package policy\n"), 0o600)
			case "replaced-root":
				if err := os.Rename(roots[0].Directory, filepath.Join(base, "old-root")); err != nil {
					t.Fatal(err)
				}
				if err := os.Mkdir(roots[0].Directory, 0o700); err != nil {
					t.Fatal(err)
				}
			}
			if _, err := session.seal(); err == nil {
				t.Fatal("changed original source/namespace was sealed")
			}
		})
	}
}

func TestPracticalCaptureRejectsLinksSpecialFilesAndInvalidPaths(t *testing.T) {
	roots, base := practicalTestCaptureRoots(t)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "real", "policy.go"), []byte("package policy\n"), 0o600)
	writePracticalTestFile(t, filepath.Join(base, "outside.go"), []byte("outside capability\n"), 0o600)
	if err := os.Symlink("real", filepath.Join(roots[0].Directory, "alias")); err != nil {
		t.Fatal(err)
	}
	if err := os.Symlink("../outside.go", filepath.Join(roots[0].Directory, "leaf.go")); err != nil {
		t.Fatal(err)
	}
	if err := syscall.Mkfifo(filepath.Join(roots[0].Directory, "pipe.go"), 0o600); err != nil {
		t.Fatal(err)
	}
	for _, relative := range []string{"alias/policy.go", "leaf.go", "pipe.go", "real", "", "../outside.go", "/outside.go", "real/../leaf.go", "real//policy.go", "real\\policy.go"} {
		session := practicalTestSession(t, roots)
		if _, err := session.readSource(roots[0].Capability, relative, "go_file"); err == nil {
			t.Fatalf("linked/special/invalid source %q accepted", relative)
		}
		if _, err := session.seal(); err == nil {
			t.Fatal("ignored source read failure produced a sealed snapshot")
		}
	}
	if _, err := practicalTestSession(t, roots).readSource("unbound", "real/policy.go", "go_file"); err == nil {
		t.Fatal("unbound root capability accepted")
	}
	linked := filepath.Join(base, "root-link")
	if err := os.Symlink(roots[0].Directory, linked); err != nil {
		t.Fatal(err)
	}
	if _, err := newPracticalCaptureSession([]practicalSourceRoot{{Capability: "main", Directory: linked}}); err == nil {
		t.Fatal("symbolic root capability accepted")
	}
}

type practicalCaptureOpenHook struct {
	practicalRootReader
	beforeOpen func(string)
}

func (root practicalCaptureOpenHook) Open(name string) (*os.File, error) {
	root.beforeOpen(name)
	return root.practicalRootReader.Open(name)
}

func TestPracticalCaptureRejectsLeafSwapBetweenStatAndOpen(t *testing.T) {
	for _, replacement := range []string{"outside-symlink", "inside-symlink", "fifo", "new-inode"} {
		t.Run(replacement, func(t *testing.T) {
			roots, base := practicalTestCaptureRoots(t)
			file := filepath.Join(roots[0].Directory, "policy.go")
			writePracticalTestFile(t, file, []byte("package policy\n"), 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "other.go"), []byte("package forged\n"), 0o600)
			writePracticalTestFile(t, filepath.Join(base, "outside.go"), []byte("outside capability\n"), 0o600)
			session := practicalTestSession(t, roots)
			swapped := false
			session.openRoot = func(name string) (practicalRootReader, error) {
				reader, err := openPracticalSourceRoot(name)
				return practicalCaptureOpenHook{practicalRootReader: reader, beforeOpen: func(name string) {
					if name != "policy.go" || swapped {
						return
					}
					swapped = true
					if err := os.Remove(file); err != nil {
						t.Fatal(err)
					}
					var err error
					switch replacement {
					case "outside-symlink":
						err = os.Symlink("../outside.go", file)
					case "inside-symlink":
						err = os.Symlink("other.go", file)
					case "fifo":
						err = syscall.Mkfifo(file, 0o600)
					case "new-inode":
						err = os.WriteFile(file, []byte("package forged\n"), 0o600)
					}
					if err != nil {
						t.Fatal(err)
					}
				}}, err
			}
			buffer, err := session.readSource(roots[0].Capability, "policy.go", "go_file")
			if !swapped || err == nil || len(buffer.sourceBytes()) != 0 || len(session.files) != 0 {
				t.Fatalf("leaf race admitted data: swapped=%v error=%v", swapped, err)
			}
		})
	}
}

func TestPracticalCaptureSharesBudgetsAcrossCapabilities(t *testing.T) {
	for _, budget := range []string{"candidate-count", "manifest-count", "bytes", "contracts", "contract-bytes", "directories", "directory-entries"} {
		t.Run(budget, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			for _, root := range roots {
				writePracticalTestFile(t, filepath.Join(root.Directory, "input"), []byte("abc"), 0o600)
			}
			session := practicalTestSession(t, roots)
			kind := "auxiliary"
			switch budget {
			case "candidate-count":
				session.counters.candidates = maximumCandidateEntries - 1
			case "manifest-count":
				session.counters.manifestInputs = maximumManifestInputs - 1
			case "bytes":
				session.counters.capturedBytes = maximumCapturedBytes - 3
			case "contracts":
				kind = "overlay"
				session.counters.contractCandidates = maximumContracts - 1
			case "contract-bytes":
				kind = "overlay"
				session.counters.contractBytes = maximumTotalContractBytes - 3
			case "directories":
				session.counters.directories = maximumVisitedDirectories - 1
			case "directory-entries":
				session.counters.directoryEntries = maximumDirectoryEntries - 1
			}
			if _, err := session.readSource(roots[0].Capability, "input", kind); err != nil {
				t.Fatalf("inclusive remaining budget rejected: %v", err)
			}
			if _, err := session.readSource(roots[1].Capability, "input", kind); err == nil {
				t.Fatalf("%s budget was reset for another module capability", budget)
			}
			if _, err := session.seal(); err == nil {
				t.Fatal("ignored cross-capability resource failure produced a snapshot")
			}
		})
	}
}

func TestPracticalCaptureRejectsRoleChangesAndASCIICollisions(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "input"), []byte("abc"), 0o600)
	session := practicalTestSession(t, roots)
	if _, err := session.readSource(roots[0].Capability, "input", "go_file"); err != nil {
		t.Fatal(err)
	}
	if _, err := session.readSource(roots[0].Capability, "input", "metadata"); err == nil {
		t.Fatal("one captured path acquired a different source role")
	}
	if _, err := session.seal(); err == nil {
		t.Fatal("ignored role change produced a snapshot")
	}
	if _, err := newPracticalCaptureSession([]practicalSourceRoot{{Capability: "main", Directory: roots[0].Directory}, {Capability: "MAIN", Directory: roots[1].Directory}}); err == nil {
		t.Fatal("ASCII-folded capability collision accepted")
	}
	unicode := practicalTestSession(t, []practicalSourceRoot{{Capability: "Å", Directory: roots[0].Directory}, {Capability: "å", Directory: roots[0].Directory}})
	for _, capability := range []string{"Å", "å"} {
		if _, err := unicode.readSource(capability, "input", "auxiliary"); err != nil {
			t.Fatalf("ASCII-only folding changed Unicode source identity: %v", err)
		}
	}
	if snapshot, err := unicode.seal(); err != nil || len(snapshot.sourceObjects()) != 2 {
		t.Fatalf("distinct Unicode capabilities changed: %v", err)
	}
}

// Model the case-insensitive lookup behavior of a development filesystem while
// retaining its exact original directory names. Capture identities must not
// inherit filesystem name aliases, even on a host that resolves them.
type practicalCaseInsensitiveRoot struct{ practicalRootReader }

func (root practicalCaseInsensitiveRoot) Open(name string) (*os.File, error) {
	return root.practicalRootReader.Open(strings.ToLower(name))
}

func (root practicalCaseInsensitiveRoot) Lstat(name string) (os.FileInfo, error) {
	return root.practicalRootReader.Lstat(strings.ToLower(name))
}

func TestPracticalCaptureRejectsNonExactNamespaceSpelling(t *testing.T) {
	for _, relative := range []string{"nested/INPUT.go", "NESTED/input.go"} {
		t.Run(relative, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "nested", "input.go"), []byte("package policy\n"), 0o600)
			session := practicalTestSession(t, roots)
			session.openRoot = func(name string) (practicalRootReader, error) {
				root, err := openPracticalSourceRoot(name)
				return practicalCaseInsensitiveRoot{root}, err
			}
			if _, err := session.readSource(roots[0].Capability, relative, "go_file"); err == nil {
				t.Fatal("filesystem case alias became a different captured logical path")
			}
		})
	}
}

func TestPracticalCaptureRejectsInternalBufferCorruption(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), []byte("package policy\n"), 0o600)
	session := practicalTestSession(t, roots)
	buffer, err := session.readSource(roots[0].Capability, "policy.go", "go_file")
	if err != nil {
		t.Fatal(err)
	}
	// Deliberately bypass the copy-returning API to model an internal producer
	// bug. A matching live file must not hide corrupted owned snapshot bytes.
	buffer.data[0] ^= 1
	if _, err := session.seal(); err == nil {
		t.Fatal("corrupted owned bytes were sealed under the original file identity")
	}
}
