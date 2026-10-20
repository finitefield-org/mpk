//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"path/filepath"
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
