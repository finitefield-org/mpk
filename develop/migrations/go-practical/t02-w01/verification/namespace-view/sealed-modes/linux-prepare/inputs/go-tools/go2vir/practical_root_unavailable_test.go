//go:build !go1.25 || (!linux && !darwin)

package main

import "testing"

func TestPracticalRootUnavailableIsAnError(t *testing.T) {
	root, err := openPracticalSourceRoot(t.TempDir())
	if root != nil || err == nil {
		t.Fatal("unavailable rooted capture silently accepted a source capability")
	}
}
