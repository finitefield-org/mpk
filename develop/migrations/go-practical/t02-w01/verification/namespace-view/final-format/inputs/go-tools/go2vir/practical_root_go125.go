//go:build go1.25 && (linux || darwin)

package main

import (
	"os"
	"syscall"
)

type practicalOSRoot struct {
	*os.Root
}

func (root practicalOSRoot) Open(name string) (*os.File, error) {
	// A regular-file-to-FIFO race must not block before its type is checked.
	return root.OpenFile(name, os.O_RDONLY|syscall.O_NONBLOCK, 0)
}

func openPracticalSourceRoot(name string) (practicalRootReader, error) {
	root, err := os.OpenRoot(name)
	if err != nil {
		return nil, err
	}
	return practicalOSRoot{Root: root}, nil
}
