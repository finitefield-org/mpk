//go:build go1.25

package main

import (
	"io/fs"
	"os"
)

// Keep permission changes and recursive removal beneath the owned image root.
// In particular, cleanup never chmods a symlink or an outside file reached by it.
func removePracticalReplayImage(name string) error {
	root, err := os.OpenRoot(name)
	if os.IsNotExist(err) {
		return nil
	}
	if err != nil {
		return err
	}
	defer root.Close()
	if err := fs.WalkDir(root.FS(), ".", func(path string, entry fs.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		if entry.IsDir() {
			return root.Chmod(path, 0o700)
		}
		return nil
	}); err != nil {
		return err
	}
	directory, err := root.Open(".")
	if err != nil {
		return err
	}
	entries, readErr := directory.ReadDir(-1)
	closeErr := directory.Close()
	if readErr != nil {
		return readErr
	}
	if closeErr != nil {
		return closeErr
	}
	for _, entry := range entries {
		if err := root.RemoveAll(entry.Name()); err != nil {
			return err
		}
	}
	if err := root.Close(); err != nil {
		return err
	}
	return os.Remove(name)
}
