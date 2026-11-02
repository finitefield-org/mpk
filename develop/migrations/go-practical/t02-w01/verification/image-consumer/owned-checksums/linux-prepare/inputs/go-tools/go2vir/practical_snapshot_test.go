//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"errors"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"syscall"
	"testing"
)

func TestPracticalSnapshotOwnsBytesAfterCollectorRelease(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	original := []byte("package policy\nfunc Identity(x int64) int64 { return x }\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), original, 0o600)
	session := practicalTestSession(t, roots)
	collected, err := session.readSource(roots[0].Capability, "policy.go", "go_file")
	if err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	// The collector and its consumers may release or reuse their own buffers.
	// A sealed snapshot must own a distinct allocation after that boundary.
	collected.data[0] ^= 1
	actual, exists := snapshot.sourceBytes(collected.object.ID)
	if !exists || !bytes.Equal(actual, original) {
		t.Fatalf("sealed bytes alias a released collector buffer: got %q, want %q", actual, original)
	}
}

type practicalSnapshotTestFixture struct {
	roots     []practicalSourceRoot
	session   *practicalCaptureSession
	snapshot  practicalSourceSnapshot
	collected []practicalSourceBuffer
}

func preparePracticalSnapshotFixture(t *testing.T, reverse bool) practicalSnapshotTestFixture {
	t.Helper()
	roots, base := practicalTestCaptureRoots(t)
	unused := practicalSourceRoot{Capability: "workspace/unused", Directory: filepath.Join(base, "unused")}
	writePracticalTestFile(t, filepath.Join(unused.Directory, "uncaptured.go"), []byte("package unused\n"), 0o600)
	roots = append(roots, unused)
	inputs := []struct {
		root, name, kind string
		data             []byte
	}{
		{roots[0].Capability, "policy.go", "go_file", []byte("package policy\nfunc Selected(x int64) int64 { return x }\nfunc Unused(m map[string]int) int { return len(m) }\n")},
		{roots[0].Capability, "go.mod", "go_mod", []byte("module example.com/policy\ngo 1.25.0\nreplace example.com/dep => ../dep\n")},
		{roots[0].Capability, "go.sum", "go_sum", []byte{}},
		{roots[0].Capability, "policy.contract.json", "overlay", []byte("{}\n")},
		{roots[0].Capability, "metadata.json", "metadata", []byte("{\"original\":true}\r\n")},
		{roots[0].Capability, "exports/pkg.a", "export_data", []byte{0, 0xff, 0x80, '\n'}},
		{roots[0].Capability, "other_windows.go", "excluded_source", []byte("//go:build windows\n\npackage policy\n")},
		{roots[0].Capability, "opaque.asset", "auxiliary", []byte{0xff, 0, 1}},
		{roots[1].Capability, "policy.go", "go_file", []byte("package dep\nfunc Identity(x int64) int64 { return x }\n")},
	}
	byCapability := map[string]string{}
	for _, root := range roots {
		byCapability[root.Capability] = root.Directory
	}
	for _, input := range inputs {
		writePracticalTestFile(t, filepath.Join(byCapability[input.root], input.name), input.data, 0o600)
	}
	for _, name := range []string{"unread.go", "ignored\nasset.bin", "native\\asset.bin", "pending/uncaptured.go"} {
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, name), []byte("unread original asset\n"), 0o600)
	}
	if err := os.Mkdir(filepath.Join(roots[0].Directory, "empty"), 0o700); err != nil {
		t.Fatal(err)
	}
	for name, target := range map[string]string{"linked.go": "policy.go", "linked-dir": "pending"} {
		if err := os.Symlink(target, filepath.Join(roots[0].Directory, name)); err != nil {
			t.Fatal(err)
		}
	}
	if err := syscall.Mkfifo(filepath.Join(roots[0].Directory, "special.go"), 0o600); err != nil {
		t.Fatal(err)
	}
	orderedRoots := append([]practicalSourceRoot(nil), roots...)
	if reverse {
		for i, j := 0, len(orderedRoots)-1; i < j; i, j = i+1, j-1 {
			orderedRoots[i], orderedRoots[j] = orderedRoots[j], orderedRoots[i]
		}
		for i, j := 0, len(inputs)-1; i < j; i, j = i+1, j-1 {
			inputs[i], inputs[j] = inputs[j], inputs[i]
		}
	}
	session := practicalTestSession(t, orderedRoots)
	collected := []practicalSourceBuffer{}
	for _, input := range inputs {
		buffer, err := session.readSource(input.root, input.name, input.kind)
		if err != nil {
			t.Fatal(err)
		}
		collected = append(collected, buffer)
	}
	if _, err := session.readDirectory(roots[0].Capability, "empty"); err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	return practicalSnapshotTestFixture{roots, session, snapshot, collected}
}

func TestPracticalSnapshotRetainsVisitedNamespacesAfterOriginalRemoval(t *testing.T) {
	fixture := preparePracticalSnapshotFixture(t, false)
	expected := map[string][]practicalDirectoryEntry{}
	for key := range fixture.session.directories {
		capability, relative, _ := strings.Cut(key, "\x00")
		var physical string
		for _, root := range fixture.roots {
			if root.Capability == capability {
				physical = filepath.Join(root.Directory, filepath.FromSlash(relative))
			}
		}
		entries, err := os.ReadDir(physical)
		if err != nil {
			t.Fatal(err)
		}
		expected[key] = []practicalDirectoryEntry{}
		for _, entry := range entries {
			expected[key] = append(expected[key], practicalDirectoryEntry{entry.Name(), entry.Type()})
		}
	}
	for _, directory := range fixture.session.directories {
		if len(directory.entries) != 0 {
			directory.entries[0].Name = "released-collector-directory"
		}
	}
	for _, buffer := range fixture.collected {
		if len(buffer.data) != 0 {
			buffer.data[0] ^= 1
		}
	}
	for _, root := range fixture.roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	fixture.session.openRoot = func(string) (practicalRootReader, error) {
		t.Fatal("sealed lookup reopened the original filesystem")
		return nil, nil
	}
	for key, entries := range expected {
		capability, relative, _ := strings.Cut(key, "\x00")
		actual, err := fixture.snapshot.readDirectory(capability, relative)
		if err != nil || !reflect.DeepEqual(actual, entries) {
			t.Fatalf("sealed namespace %q differs from original native enumeration: %v, %v", key, actual, err)
		}
	}
	for _, object := range fixture.snapshot.sourceObjects() {
		buffer, err := fixture.snapshot.readSource(object.Capability, object.Path, object.Kind)
		if err != nil || buffer.object != object || sha256Hex(buffer.data) != object.RawSHA256 {
			t.Fatalf("original source was lost after collector/root release: %s, %v", object.Path, err)
		}
	}
	if got := practicalSnapshotDigest(fixture.snapshot); got != fixture.snapshot.contentSHA256 {
		t.Fatal("sealed identity changed with the original roots or collector")
	}
}

func TestPracticalSnapshotLookupSeparatesAbsenceAndUncapturedInputs(t *testing.T) {
	fixture := preparePracticalSnapshotFixture(t, false)
	for _, item := range []struct {
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
		{"absent-in-known-directory", "workspace/main", "exports/absent.a", "export_data", "absent", false},
		{"absent-directory", "workspace/main", "absent", "", "absent", true},
		{"present-unread-file", "workspace/main", "unread.go", "go_file", "uncaptured", false},
		{"present-unvisited-directory", "workspace/main", "pending", "", "uncaptured", true},
		{"present-in-unvisited-directory", "workspace/main", "pending/uncaptured.go", "go_file", "uncaptured", false},
		{"unknown-below-unvisited-directory", "workspace/main", "pending/absent.go", "go_file", "uncaptured", false},
		{"unvisited-capability-root", "workspace/unused", "", "", "uncaptured", true},
		{"unknown-file-in-unvisited-root", "workspace/unused", "absent.go", "go_file", "uncaptured", false},
		{"unbound-capability", "workspace/foreign", "policy.go", "go_file", "rejected", false},
		{"wrong-kind", "workspace/main", "policy.go", "metadata", "rejected", false},
		{"unknown-kind", "workspace/main", "policy.go", "caller-kind", "rejected", false},
		{"wrong-case", "workspace/main", "POLICY.go", "go_file", "rejected", false},
		{"linked-file", "workspace/main", "linked.go", "go_file", "rejected", false},
		{"linked-directory", "workspace/main", "linked-dir", "", "rejected", true},
		{"linked-ancestor", "workspace/main", "linked-dir/uncaptured.go", "go_file", "rejected", false},
		{"special-file", "workspace/main", "special.go", "go_file", "rejected", false},
		{"file-as-directory", "workspace/main", "policy.go", "", "rejected", true},
		{"directory-as-file", "workspace/main", "exports", "go_file", "rejected", false},
		{"escape", "workspace/main", "../policy.go", "go_file", "rejected", false},
		{"absolute", "workspace/main", "/policy.go", "go_file", "rejected", false},
		{"repeated-separator", "workspace/main", "exports//pkg.a", "export_data", "rejected", false},
		{"native-unread-backslash", "workspace/main", "native\\asset.bin", "auxiliary", "rejected", false},
		{"native-unread-newline", "workspace/main", "ignored\nasset.bin", "auxiliary", "rejected", false},
	} {
		t.Run(item.name, func(t *testing.T) {
			var err error
			if item.directory {
				_, err = fixture.snapshot.readDirectory(item.capability, item.relative)
			} else {
				_, err = fixture.snapshot.readSource(item.capability, item.relative, item.kind)
			}
			absent, uncaptured := errors.Is(err, fs.ErrNotExist), errors.Is(err, errPracticalNotCaptured)
			if item.outcome == "present" && err != nil || item.outcome == "absent" && !absent || item.outcome == "uncaptured" && !uncaptured || item.outcome == "rejected" && (err == nil || absent || uncaptured) {
				t.Fatalf("lookup outcome differs: want %s, got %v", item.outcome, err)
			}
			if absent && uncaptured {
				t.Fatal("uncaptured input was also classified as proven absence")
			}
		})
	}
}

func TestPracticalSnapshotReadResultsAreIndependentCopies(t *testing.T) {
	fixture := preparePracticalSnapshotFixture(t, false)
	snapshot := fixture.snapshot
	roots := snapshot.rootCapabilities()
	roots[0] = "caller-root"
	objects := snapshot.sourceObjects()
	objects[0].ID = "caller-id"
	for _, object := range snapshot.sourceObjects() {
		buffer, err := snapshot.readSource(object.Capability, object.Path, object.Kind)
		if err != nil {
			t.Fatal(err)
		}
		buffer.object.ID = "caller-id"
		if len(buffer.data) != 0 {
			buffer.data[0] ^= 1
		}
		byID, exists := snapshot.sourceBytes(object.ID)
		if !exists || sha256Hex(byID) != object.RawSHA256 {
			t.Fatal("location lookup returned snapshot-owned bytes")
		}
		if len(byID) != 0 {
			byID[0] ^= 1
		}
		again, err := snapshot.readSource(object.Capability, object.Path, object.Kind)
		if err != nil || again.object != object || sha256Hex(again.data) != object.RawSHA256 {
			t.Fatal("source-ID lookup returned snapshot-owned metadata/bytes")
		}
	}
	entries, err := snapshot.readDirectory(fixture.roots[0].Capability, "")
	if err != nil {
		t.Fatal(err)
	}
	entries[0].Name, entries[0].Type = "caller-entry", os.ModeSymlink
	again, err := snapshot.readDirectory(fixture.roots[0].Capability, "")
	if err != nil || again[0].Name == "caller-entry" || snapshot.rootCapabilities()[0] == "caller-root" || practicalSnapshotDigest(snapshot) != snapshot.contentSHA256 {
		t.Fatal("directory/root metadata aliases returned arrays")
	}
	if _, exists := snapshot.sourceBytes("unknown-source-id"); exists {
		t.Fatal("unknown source ID invented a buffer")
	}
}

func recapturePracticalSnapshot(t *testing.T, fixture practicalSnapshotTestFixture, kindChange, extraDirectory bool, extraRoot *practicalSourceRoot) practicalSourceSnapshot {
	t.Helper()
	roots := append([]practicalSourceRoot(nil), fixture.roots...)
	if extraRoot != nil {
		roots = append(roots, *extraRoot)
	}
	session := practicalTestSession(t, roots)
	for _, object := range fixture.snapshot.sourceObjects() {
		kind := object.Kind
		if kindChange && kind == "metadata" {
			kind = "auxiliary"
		}
		if _, err := session.readSource(object.Capability, object.Path, kind); err != nil {
			t.Fatal(err)
		}
	}
	if _, err := session.readDirectory(fixture.roots[0].Capability, "empty"); err != nil {
		t.Fatal(err)
	}
	if extraDirectory {
		if _, err := session.readDirectory(fixture.roots[0].Capability, "pending"); err != nil {
			t.Fatal(err)
		}
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	return snapshot
}

func practicalSnapshotHashVector(snapshot practicalSourceSnapshot, label, scope string) map[string]any {
	directories := []map[string]any{}
	keys := make([]string, 0, len(snapshot.directories))
	for key := range snapshot.directories {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, key := range keys {
		capability, relative, _ := strings.Cut(key, "\x00")
		entries := []map[string]any{}
		for _, entry := range snapshot.directories[key] {
			entries = append(entries, map[string]any{"name_hex": hex.EncodeToString([]byte(entry.Name)), "type_bits": uint32(entry.Type)})
		}
		directories = append(directories, map[string]any{"capability": capability, "path": relative, "entries": entries})
	}
	sources := []map[string]any{}
	for _, object := range snapshot.sourceObjects() {
		raw, _ := snapshot.sourceBytes(object.ID)
		sources = append(sources, map[string]any{"object": object, "bytes_hex": hex.EncodeToString(raw)})
	}
	return map[string]any{"label": label, "scope": scope, "capabilities": snapshot.rootCapabilities(), "directories": directories, "sources": sources, "source_ids": append([]string(nil), snapshot.ids...), "sha256": practicalSnapshotDigest(snapshot)}
}

func TestPracticalSnapshotIdentityBindsCompleteObservedNamespace(t *testing.T) {
	first := preparePracticalSnapshotFixture(t, false)
	reordered := preparePracticalSnapshotFixture(t, true)
	if !sha256Pattern.MatchString(first.snapshot.contentSHA256) || first.snapshot.contentSHA256 != reordered.snapshot.contentSHA256 {
		t.Fatal("sealed namespace identity depends on physical location, read order or root order")
	}
	vectors := []map[string]any{practicalSnapshotHashVector(first.snapshot, "original", "actual native captured namespace"), practicalSnapshotHashVector(reordered.snapshot, "reordered", "actual native captured namespace")}
	for _, mutation := range []string{"added-entry", "renamed-unread", "changed-native-type", "visited-empty-now-nonempty", "source-bytes", "source-role", "additional-visited-directory", "additional-authorized-root", "unread-content-only"} {
		t.Run(mutation, func(t *testing.T) {
			fixture := preparePracticalSnapshotFixture(t, false)
			var extraRoot *practicalSourceRoot
			file := func(name string) string { return filepath.Join(fixture.roots[0].Directory, filepath.FromSlash(name)) }
			switch mutation {
			case "added-entry":
				writePracticalTestFile(t, file("added.go"), []byte("unread"), 0o600)
			case "renamed-unread":
				if err := os.Rename(file("unread.go"), file("renamed.go")); err != nil {
					t.Fatal(err)
				}
			case "changed-native-type":
				if err := os.Remove(file("unread.go")); err != nil {
					t.Fatal(err)
				}
				if err := os.Mkdir(file("unread.go"), 0o700); err != nil {
					t.Fatal(err)
				}
			case "visited-empty-now-nonempty":
				writePracticalTestFile(t, file("empty/added"), []byte("unread"), 0o600)
			case "source-bytes":
				writePracticalTestFile(t, file("policy.go"), []byte("package different\n"), 0o600)
			case "additional-authorized-root":
				root := practicalSourceRoot{Capability: "workspace/additional", Directory: filepath.Join(filepath.Dir(fixture.roots[0].Directory), "additional")}
				if err := os.Mkdir(root.Directory, 0o700); err != nil {
					t.Fatal(err)
				}
				extraRoot = &root
			case "unread-content-only":
				writePracticalTestFile(t, file("unread.go"), []byte("different unread bytes are not a captured source"), 0o600)
			}
			observed := recapturePracticalSnapshot(t, fixture, mutation == "source-role", mutation == "additional-visited-directory", extraRoot)
			equal := observed.contentSHA256 == fixture.snapshot.contentSHA256
			if equal != (mutation == "unread-content-only") {
				t.Fatalf("complete captured-input/namespace identity did not distinguish %s", mutation)
			}
			vectors = append(vectors, practicalSnapshotHashVector(observed, mutation, "actual independent native recapture"))
		})
	}
	// Raw-name framing is also tested for byte strings which a particular host
	// filesystem may not create. These are hash units, not capture authority.
	for _, name := range []string{string([]byte{0xff}), string([]byte{0xfe}), "\ufffd", "a\x00b", "a", "ab"} {
		snapshot := practicalSourceSnapshot{roots: []string{"root"}, directories: map[string][]practicalDirectoryEntry{practicalCaptureKey("root", ""): {{Name: name}}}}
		vectors = append(vectors, practicalSnapshotHashVector(snapshot, "raw-name-"+hex.EncodeToString([]byte(name)), "private hash framing unit only; not a captured namespace"))
	}
	seen := map[string]bool{}
	for _, vector := range vectors[len(vectors)-6:] {
		hash := vector["sha256"].(string)
		if seen[hash] {
			t.Fatal("raw native bytes were normalized, replaced or ambiguously framed")
		}
		seen[hash] = true
	}
	retainPracticalAnalysisEvidence(t, "namespace-vectors.json", map[string]any{"vectors": vectors, "registered_context": false, "immutable_application_replay_claimed": false})
}

func TestPracticalSnapshotZeroValueDoesNotInventAbsence(t *testing.T) {
	var snapshot practicalSourceSnapshot
	if _, err := snapshot.readDirectory("root", ""); err == nil || errors.Is(err, fs.ErrNotExist) {
		t.Fatal("zero snapshot invented a complete empty namespace")
	}
	if _, err := snapshot.readSource("root", path.Join("nested", "policy.go"), "go_file"); err == nil || errors.Is(err, fs.ErrNotExist) {
		t.Fatal("zero snapshot invented a proven absent source")
	}
}
