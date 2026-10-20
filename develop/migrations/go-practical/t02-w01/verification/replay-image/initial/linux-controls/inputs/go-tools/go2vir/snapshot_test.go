package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestSnapshotUsesOnlyImmutableCapturedBuffers(t *testing.T) {
	originalRoot := copyPreflightFixture(t)
	capture, err := captureSourceTree(originalRoot, preflightRequest())
	if err != nil {
		t.Fatalf("capture fixture: %v", err)
	}
	originalIdentity := filepath.Join(originalRoot, "identity.go")
	writeTestFile(t, originalIdentity, []byte("package changed\n"), 0o600)
	if err := os.Remove(filepath.Join(originalRoot, "helper", "helper.go")); err != nil {
		t.Fatalf("remove original helper after capture: %v", err)
	}

	snapshot, err := buildSourceSnapshot(t.TempDir(), capture)
	if err != nil {
		t.Fatalf("build private snapshot: %v", err)
	}
	defer snapshot.Close()
	for _, input := range capture.Inputs {
		path := filepath.Join(snapshot.Root, filepath.FromSlash(input.NormalizedPath))
		content, err := os.ReadFile(path)
		if err != nil {
			t.Fatalf("read snapshot input %s: %v", input.NormalizedPath, err)
		}
		if string(content) != string(input.Bytes) {
			t.Fatalf("snapshot input %s differs from its captured buffer", input.NormalizedPath)
		}
		info, err := os.Stat(path)
		if err != nil || info.Mode().Perm()&0o222 != 0 {
			t.Fatalf("snapshot input %s is not read-only", input.NormalizedPath)
		}
	}
	if content, err := os.ReadFile(filepath.Join(snapshot.Root, "identity.go")); err != nil || string(content) == "package changed\n" {
		t.Fatal("private snapshot reread the mutated original source")
	}
}

func TestSnapshotRejectsEmptyCapture(t *testing.T) {
	if _, err := buildSourceSnapshot(t.TempDir(), sourceCapture{}); err == nil {
		t.Fatal("empty source capture was accepted")
	}
}

func TestSnapshotRejectsCapturedBufferMutation(t *testing.T) {
	capture, err := captureSourceTree(copyPreflightFixture(t), preflightRequest())
	if err != nil {
		t.Fatalf("capture fixture: %v", err)
	}
	capture.Inputs[0].Bytes[0] ^= 1
	if _, err := buildSourceSnapshot(t.TempDir(), capture); err == nil {
		t.Fatal("captured buffer mutation was accepted")
	}
}

func TestSnapshotIdentityDoesNotAliasCapturedBuffers(t *testing.T) {
	capture, err := captureSourceTree(copyPreflightFixture(t), preflightRequest())
	if err != nil {
		t.Fatalf("capture fixture: %v", err)
	}
	snapshot, err := buildSourceSnapshot(t.TempDir(), capture)
	if err != nil {
		t.Fatalf("build snapshot: %v", err)
	}
	defer snapshot.Close()
	if !snapshot.matchesCapture(capture) {
		t.Fatal("new snapshot did not match its capture")
	}
	capture.Inputs[0].Bytes[0] ^= 1
	if snapshot.matchesCapture(capture) {
		t.Fatal("snapshot identity aliased a mutated capture buffer")
	}
}
