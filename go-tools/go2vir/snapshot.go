package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
)

type sourceSnapshot struct {
	Root   string
	inputs map[string]snapshotInputIdentity
}

type snapshotInputIdentity struct {
	Kind      string
	SizeBytes int64
	SHA256    string
}

func buildSourceSnapshot(parent string, capture sourceCapture) (sourceSnapshot, error) {
	if err := validateSnapshotCapture(capture); err != nil {
		return sourceSnapshot{}, err
	}
	root, err := os.MkdirTemp(parent, "go2vir-source-")
	if err != nil {
		return sourceSnapshot{}, fmt.Errorf("create private source snapshot: %w", err)
	}
	cleanup := true
	defer func() {
		if cleanup {
			_ = removeSnapshot(root)
		}
	}()

	directories := map[string]struct{}{root: {}}
	for _, input := range capture.Inputs {
		destination := filepath.Join(root, filepath.FromSlash(input.NormalizedPath))
		directory := filepath.Dir(destination)
		if err := os.MkdirAll(directory, 0o700); err != nil {
			return sourceSnapshot{}, fmt.Errorf("create private snapshot directory: %w", err)
		}
		for current := directory; current != root && current != filepath.Dir(current); current = filepath.Dir(current) {
			directories[current] = struct{}{}
		}
		file, err := os.OpenFile(destination, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o400)
		if err != nil {
			return sourceSnapshot{}, fmt.Errorf("create private snapshot input: %w", err)
		}
		writeErr := writeAll(file, input.Bytes)
		closeErr := file.Close()
		if writeErr != nil {
			return sourceSnapshot{}, fmt.Errorf("write private snapshot input: %w", writeErr)
		}
		if closeErr != nil {
			return sourceSnapshot{}, fmt.Errorf("close private snapshot input: %w", closeErr)
		}
		if err := os.Chmod(destination, 0o400); err != nil {
			return sourceSnapshot{}, fmt.Errorf("seal private snapshot input: %w", err)
		}
	}
	directoryList := make([]string, 0, len(directories))
	for directory := range directories {
		directoryList = append(directoryList, directory)
	}
	sort.Slice(directoryList, func(left, right int) bool {
		return len(directoryList[left]) > len(directoryList[right])
	})
	for _, directory := range directoryList {
		if err := os.Chmod(directory, 0o500); err != nil {
			return sourceSnapshot{}, fmt.Errorf("seal private snapshot directory: %w", err)
		}
	}
	inputs := make(map[string]snapshotInputIdentity, len(capture.Inputs))
	for _, input := range capture.Inputs {
		inputs[input.NormalizedPath] = snapshotInputIdentity{Kind: input.Kind, SizeBytes: int64(len(input.Bytes)), SHA256: input.SHA256}
	}
	cleanup = false
	return sourceSnapshot{Root: root, inputs: inputs}, nil
}

func (snapshot sourceSnapshot) matchesCapture(capture sourceCapture) bool {
	if len(snapshot.inputs) != len(capture.Inputs) {
		return false
	}
	for _, input := range capture.Inputs {
		identity, exists := snapshot.inputs[input.NormalizedPath]
		if !exists || identity.Kind != input.Kind || identity.SizeBytes != int64(len(input.Bytes)) || identity.SHA256 != input.SHA256 || input.SHA256 != sha256Hex(input.Bytes) {
			return false
		}
	}
	return true
}

func validateSnapshotCapture(capture sourceCapture) error {
	if len(capture.Inputs) == 0 || len(capture.Inputs) > maximumManifestInputs {
		return fmt.Errorf("source capture input count is invalid")
	}
	paths := make([]string, 0, len(capture.Inputs))
	totalBytes := uint64(0)
	for index, input := range capture.Inputs {
		if !validPortablePath(input.NormalizedPath) || !sha256Pattern.MatchString(input.SHA256) || input.SHA256 != sha256Hex(input.Bytes) || len(input.Bytes) > maximumCandidateBytes {
			return fmt.Errorf("source capture contains an invalid immutable input")
		}
		switch input.Kind {
		case buildManifestInputKind, lockfileInputKind, sourceInputKind, contractInputKind:
		default:
			return fmt.Errorf("source capture contains an unknown input kind")
		}
		if index > 0 {
			previous := capture.Inputs[index-1]
			if previous.NormalizedPath >= input.NormalizedPath {
				return fmt.Errorf("source capture inputs are not strictly sorted")
			}
		}
		totalBytes += uint64(len(input.Bytes))
		if totalBytes > maximumCapturedBytes {
			return fmt.Errorf("source capture byte count is invalid")
		}
		paths = append(paths, input.NormalizedPath)
	}
	if err := validateCapturedPathUniqueness(paths); err != nil {
		return fmt.Errorf("source capture paths are not unique")
	}
	return nil
}

func (snapshot sourceSnapshot) Close() error {
	if snapshot.Root == "" {
		return nil
	}
	return removeSnapshot(snapshot.Root)
}

func removeSnapshot(root string) error {
	_ = filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if info.IsDir() {
			return os.Chmod(path, 0o700)
		}
		return os.Chmod(path, 0o600)
	})
	return os.RemoveAll(root)
}

func (snapshot sourceSnapshot) normalizedPath(path string) (string, error) {
	relative, err := filepath.Rel(snapshot.Root, path)
	if err != nil || relative == "." || filepath.IsAbs(relative) || isParentRelativePath(relative) {
		return "", fmt.Errorf("loader path is outside the private source snapshot")
	}
	normalized := filepath.ToSlash(relative)
	if !validPortablePath(normalized) {
		return "", fmt.Errorf("loader returned a nonportable source path")
	}
	return normalized, nil
}

func isParentRelativePath(path string) bool {
	return path == ".." || len(path) > 3 && path[:3] == ".."+string(filepath.Separator)
}
