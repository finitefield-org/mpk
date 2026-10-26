package main

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"sort"
	"strings"

	"golang.org/x/mod/module"
)

// Only private original capture and immutable snapshot readers implement this
// boundary. It does not authorize roots or replace compiler/context validation.
type practicalEmbedReader interface {
	readDirectory(string, string) ([]practicalDirectoryEntry, error)
	embeddedSource(string, string) (practicalSourceBuffer, error)
}

func (session *practicalCaptureSession) embeddedSource(capability, relative string) (practicalSourceBuffer, error) {
	kind := "auxiliary"
	if old, exists := session.files[practicalCaptureKey(capability, relative)]; exists {
		kind = old.kind
	}
	return session.readSource(capability, relative, kind)
}
func (snapshot practicalSourceSnapshot) embeddedSource(capability, relative string) (practicalSourceBuffer, error) {
	id, exists := snapshot.paths[practicalCaptureKey(capability, relative)]
	if !exists {
		return practicalSourceBuffer{}, fmt.Errorf("embedded source %s/%s: %w", capability, relative, errPracticalNotCaptured)
	}
	buffer := snapshot.buffers[id]
	return practicalSnapshotBuffer(snapshot, buffer.object)
}
func practicalBadEmbedName(name string) bool {
	return module.CheckFilePath(name) != nil || oneOf(name, ".bzr", ".hg", ".git", ".svn")
}
func practicalEmbedLocation(location practicalModuleLocation, relative string) string {
	if location.directory == "" {
		return relative
	}
	if relative == "" {
		return location.directory
	}
	return location.directory + "/" + relative
}

type practicalEmbeddedSources struct {
	files    []string
	objects  []practicalSourceObject
	patterns map[string][]string
}

func (session *practicalCaptureSession) captureEmbeddedSources(location practicalModuleLocation, patterns []string) (practicalEmbeddedSources, error) {
	result, err := resolvePracticalEmbeddedSources(session, location, patterns)
	if err != nil {
		return result, session.reject(err)
	}
	return result, nil
}
func (snapshot practicalSourceSnapshot) replayEmbeddedSources(location practicalModuleLocation, patterns []string) (practicalEmbeddedSources, error) {
	if snapshot.contentSHA256 == "" || snapshot.contentSHA256 != practicalSnapshotDigest(snapshot) || len(snapshot.ids) != len(snapshot.buffers) || len(snapshot.ids) != len(snapshot.paths) {
		return practicalEmbeddedSources{}, fmt.Errorf("embedded replay lost its complete original snapshot")
	}
	return resolvePracticalEmbeddedSources(snapshot, location, patterns)
}

// Reproduce the pinned Go command's default embedfollowsymlinks=0 selection
// over the complete original namespace. The collector observes every required
// directory; immutable replay fails rather than consulting a host fallback.
func resolvePracticalEmbeddedSources(inputs practicalEmbedReader, location practicalModuleLocation, patterns []string) (practicalEmbeddedSources, error) {
	result := practicalEmbeddedSources{files: []string{}, objects: []practicalSourceObject{}, patterns: map[string][]string{}}
	objects := map[string]practicalSourceObject{}
	directories := map[string][]practicalDirectoryEntry{}
	readDir := func(relative string) ([]practicalDirectoryEntry, error) {
		if entries, ok := directories[relative]; ok {
			return entries, nil
		}
		entries, err := inputs.readDirectory(location.capability, practicalEmbedLocation(location, relative))
		if err != nil {
			return nil, err
		}
		directories[relative] = entries
		return entries, nil
	}
	moduleBoundary := func(entries []practicalDirectoryEntry) bool {
		for _, e := range entries {
			if e.Name == "go.mod" {
				return true
			}
		}
		return false
	}
	for _, pattern := range patterns {
		glob, all := strings.CutPrefix(pattern, "all:")
		if _, err := path.Match(glob, ""); err != nil || glob == "." || !fs.ValidPath(glob) {
			return result, fmt.Errorf("invalid original embed pattern %q", pattern)
		}
		type matched struct {
			relative string
			kind     os.FileMode
		}
		candidates := []matched{{"", os.ModeDir}}
		for _, component := range strings.Split(glob, "/") {
			next := []matched{}
			for _, parent := range candidates {
				if parent.kind != os.ModeDir {
					continue
				}
				entries, err := readDir(parent.relative)
				if err != nil {
					return result, err
				}
				for _, entry := range entries {
					ok, err := path.Match(component, entry.Name)
					if err != nil {
						return result, err
					}
					if !ok {
						continue
					}
					relative := entry.Name
					if parent.relative != "" {
						relative = parent.relative + "/" + entry.Name
					}
					next = append(next, matched{relative, entry.Type})
				}
			}
			candidates = next
		}
		selected := map[string]bool{}
		add := func(relative string) error {
			buffer, err := inputs.embeddedSource(location.capability, practicalEmbedLocation(location, relative))
			if err != nil {
				return err
			}
			if old, exists := objects[relative]; exists && old != buffer.object {
				return fmt.Errorf("embedded source changed its original identity")
			}
			objects[relative] = buffer.object
			selected[relative] = true
			return nil
		}
		for _, candidate := range candidates {
			parts := strings.Split(candidate.relative, "/")
			for i, name := range parts {
				if practicalBadEmbedName(name) {
					return result, fmt.Errorf("invalid original embed path component %q", name)
				}
				if i+1 < len(parts) || candidate.kind == os.ModeDir {
					entries, err := readDir(strings.Join(parts[:i+1], "/"))
					if err != nil {
						return result, err
					}
					if moduleBoundary(entries) {
						return result, fmt.Errorf("original embedded path crosses a module boundary")
					}
				}
			}
			switch candidate.kind {
			case 0:
				if err := add(candidate.relative); err != nil {
					return result, err
				}
			case os.ModeDir:
				pending := []string{candidate.relative}
				count := 0
				for len(pending) > 0 {
					directory := pending[len(pending)-1]
					pending = pending[:len(pending)-1]
					entries, err := readDir(directory)
					if err != nil {
						return result, err
					}
					if directory != candidate.relative && moduleBoundary(entries) {
						continue
					}
					for _, entry := range entries {
						hidden := strings.HasPrefix(entry.Name, ".") || strings.HasPrefix(entry.Name, "_")
						bad := practicalBadEmbedName(entry.Name)
						if bad || hidden && !all {
							if entry.Type == os.ModeDir || hidden {
								continue
							}
							return result, fmt.Errorf("invalid original embedded filename %q", entry.Name)
						}
						file := directory + "/" + entry.Name
						if entry.Type == os.ModeDir {
							pending = append(pending, file)
						} else if entry.Type == 0 {
							count++
							if err := add(file); err != nil {
								return result, err
							}
						}
					}
				}
				if count == 0 {
					return result, fmt.Errorf("original embedded directory contains no embeddable files")
				}
			default:
				return result, fmt.Errorf("original embed pattern selected an irregular file")
			}
		}
		if len(selected) == 0 {
			return result, fmt.Errorf("original embed pattern has no matching files")
		}
		files := []string{}
		for file := range selected {
			files = append(files, file)
		}
		sort.Strings(files)
		result.patterns[pattern] = files
	}
	for file := range objects {
		result.files = append(result.files, file)
	}
	sort.Strings(result.files)
	for _, file := range result.files {
		result.objects = append(result.objects, objects[file])
	}
	return result, nil
}
