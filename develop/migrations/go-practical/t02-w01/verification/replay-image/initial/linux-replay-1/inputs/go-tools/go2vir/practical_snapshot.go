package main

import (
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path"
	"sort"
	"strings"
)

// A present but unread file, or an unvisited directory, is not evidence of
// absence. Consumers must fail the closed-input operation instead of trying
// another filesystem, module version, source, or toolchain.
var errPracticalNotCaptured = errors.New("practical input was not captured")

// Created only after seal has revalidated the original roots, namespaces and
// bytes. No filesystem handles, mutable file metadata or collector allocations
// survive this boundary. This is not an installed context or application proof.
type practicalSourceSnapshot struct {
	buffers       map[string]practicalSourceBuffer
	ids           []string
	paths         map[string]string
	roots         []string
	directories   map[string][]practicalDirectoryEntry
	contentSHA256 string
}

func copyPracticalSourceSnapshot(session *practicalCaptureSession) (practicalSourceSnapshot, error) {
	snapshot := practicalSourceSnapshot{
		buffers: make(map[string]practicalSourceBuffer), paths: make(map[string]string),
		directories: make(map[string][]practicalDirectoryEntry),
	}
	for capability := range session.roots {
		snapshot.roots = append(snapshot.roots, capability)
	}
	sort.Strings(snapshot.roots)
	for key, directory := range session.directories {
		capability, relative, ok := strings.Cut(key, "\x00")
		if !ok || !snapshot.hasRoot(capability) || relative != "" && !validPracticalLogicalPath(relative) {
			return practicalSourceSnapshot{}, fmt.Errorf("captured directory has an inconsistent capability/path")
		}
		entries := append([]practicalDirectoryEntry{}, directory.entries...)
		for index, entry := range entries {
			if entry.Name == "" || entry.Name == "." || entry.Name == ".." || strings.ContainsAny(entry.Name, "/\x00") || entry.Type&^os.ModeType != 0 || index > 0 && entries[index-1].Name >= entry.Name {
				return practicalSourceSnapshot{}, fmt.Errorf("captured native directory namespace is inconsistent")
			}
		}
		snapshot.directories[key] = entries
	}
	for key := range snapshot.directories {
		capability, relative, _ := strings.Cut(key, "\x00")
		if relative != "" {
			entry, err := snapshot.pathEntry(capability, relative)
			if err != nil || entry.Type != os.ModeDir {
				return practicalSourceSnapshot{}, fmt.Errorf("captured directory is disconnected from its original parent namespace")
			}
		}
	}
	for key, file := range session.files {
		if key != practicalCaptureKey(file.capability, file.path) || file.buffer.object.Capability != file.capability || file.buffer.object.Path != file.path || file.buffer.object.Kind != file.kind {
			return practicalSourceSnapshot{}, fmt.Errorf("captured source location/role is inconsistent")
		}
		entry, err := snapshot.pathEntry(file.capability, file.path)
		if err != nil || entry.Type != 0 {
			return practicalSourceSnapshot{}, fmt.Errorf("captured source is not an original regular namespace entry")
		}
		owned, err := newPracticalSourceBuffer(file.capability, file.path, file.kind, file.buffer.data)
		if err != nil || owned.object != file.buffer.object {
			return practicalSourceSnapshot{}, fmt.Errorf("captured source identity changed while owning the snapshot")
		}
		id := owned.object.ID
		if _, exists := snapshot.buffers[id]; exists {
			return practicalSourceSnapshot{}, fmt.Errorf("duplicate practical source identity")
		}
		snapshot.buffers[id] = owned
		snapshot.paths[key] = id
		snapshot.ids = append(snapshot.ids, id)
	}
	sort.Strings(snapshot.ids)
	snapshot.contentSHA256 = practicalSnapshotDigest(snapshot)
	return snapshot, nil
}

func (snapshot practicalSourceSnapshot) hasRoot(capability string) bool {
	index := sort.SearchStrings(snapshot.roots, capability)
	return index < len(snapshot.roots) && snapshot.roots[index] == capability
}

func (snapshot practicalSourceSnapshot) rootCapabilities() []string {
	return append([]string(nil), snapshot.roots...)
}

// Only a missing exact name in a visited complete parent enumeration returns
// fs.ErrNotExist. Native unrelated names remain verbatim in those enumerations;
// requested paths still use the frozen logical-path rules and exact spelling.
func (snapshot practicalSourceSnapshot) pathEntry(capability, relative string) (practicalDirectoryEntry, error) {
	if !validPracticalLogicalPath(capability) || !validPracticalLogicalPath(relative) || !snapshot.hasRoot(capability) {
		return practicalDirectoryEntry{}, fmt.Errorf("invalid or unbound sealed practical source location")
	}
	parent := ""
	parts := strings.Split(relative, "/")
	for index, name := range parts {
		entries, observed := snapshot.directories[practicalCaptureKey(capability, parent)]
		if !observed {
			return practicalDirectoryEntry{}, fmt.Errorf("directory %s/%s: %w", capability, parent, errPracticalNotCaptured)
		}
		i := sort.Search(len(entries), func(i int) bool { return entries[i].Name >= name })
		if i == len(entries) || entries[i].Name != name {
			for _, entry := range entries {
				if foldPracticalASCII(entry.Name) == foldPracticalASCII(name) {
					return practicalDirectoryEntry{}, fmt.Errorf("sealed source spelling differs from its original namespace")
				}
			}
			return practicalDirectoryEntry{}, fmt.Errorf("sealed source %s/%s: %w", capability, path.Join(parent, name), fs.ErrNotExist)
		}
		entry := entries[i]
		if index == len(parts)-1 {
			return entry, nil
		}
		if entry.Type != os.ModeDir {
			return practicalDirectoryEntry{}, fmt.Errorf("sealed source ancestor is linked or not a directory")
		}
		parent = path.Join(parent, name)
	}
	return practicalDirectoryEntry{}, fmt.Errorf("empty sealed source path")
}

func (snapshot practicalSourceSnapshot) readDirectory(capability, relative string) ([]practicalDirectoryEntry, error) {
	if !validPracticalLogicalPath(capability) || !snapshot.hasRoot(capability) {
		return nil, fmt.Errorf("invalid or unbound sealed practical source capability")
	}
	if relative != "" {
		entry, err := snapshot.pathEntry(capability, relative)
		if err != nil {
			return nil, err
		}
		if entry.Type != os.ModeDir {
			return nil, fmt.Errorf("sealed source is linked or not a directory")
		}
	}
	entries, observed := snapshot.directories[practicalCaptureKey(capability, relative)]
	if !observed {
		return nil, fmt.Errorf("directory %s/%s: %w", capability, relative, errPracticalNotCaptured)
	}
	return append([]practicalDirectoryEntry{}, entries...), nil
}

func (snapshot practicalSourceSnapshot) readSource(capability, relative, kind string) (practicalSourceBuffer, error) {
	if !validPracticalSourceKind(kind) {
		return practicalSourceBuffer{}, fmt.Errorf("unknown sealed practical source role")
	}
	entry, err := snapshot.pathEntry(capability, relative)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	if entry.Type != 0 {
		return practicalSourceBuffer{}, fmt.Errorf("sealed source is linked or not a regular file")
	}
	id, captured := snapshot.paths[practicalCaptureKey(capability, relative)]
	if !captured {
		return practicalSourceBuffer{}, fmt.Errorf("source %s/%s: %w", capability, relative, errPracticalNotCaptured)
	}
	buffer := snapshot.buffers[id]
	if buffer.object.Kind != kind {
		return practicalSourceBuffer{}, fmt.Errorf("sealed source role differs from its captured original role")
	}
	return practicalSourceBuffer{object: buffer.object, data: buffer.sourceBytes()}, nil
}

func (snapshot practicalSourceSnapshot) sourceObjects() []practicalSourceObject {
	objects := make([]practicalSourceObject, len(snapshot.ids))
	for i, id := range snapshot.ids {
		objects[i] = snapshot.buffers[id].sourceObject()
	}
	return objects
}

func (snapshot practicalSourceSnapshot) sourceBytes(id string) ([]byte, bool) {
	buffer, exists := snapshot.buffers[id]
	if !exists {
		return nil, false
	}
	return buffer.sourceBytes(), true
}

// Private derived content identity, not a new public wire schema or authority.
// Frame counts, byte lengths and native type bits explicitly. Native names may
// contain invalid UTF-8, so passing them through JSON strings would lose bytes.
func practicalSnapshotDigest(snapshot practicalSourceSnapshot) string {
	hash := sha256.New()
	_, _ = hash.Write([]byte("MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\x00"))
	var integer [8]byte
	number := func(value uint64) {
		binary.LittleEndian.PutUint64(integer[:], value)
		_, _ = hash.Write(integer[:])
	}
	text := func(value string) {
		number(uint64(len(value)))
		_, _ = hash.Write([]byte(value))
	}
	number(uint64(len(snapshot.roots)))
	for _, capability := range snapshot.roots {
		text(capability)
	}
	keys := make([]string, 0, len(snapshot.directories))
	for key := range snapshot.directories {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	number(uint64(len(keys)))
	for _, key := range keys {
		capability, relative, _ := strings.Cut(key, "\x00")
		text(capability)
		text(relative)
		entries := snapshot.directories[key]
		number(uint64(len(entries)))
		for _, entry := range entries {
			text(entry.Name)
			number(uint64(entry.Type))
		}
	}
	number(uint64(len(snapshot.ids)))
	for _, id := range snapshot.ids {
		text(id)
	}
	return fmt.Sprintf("%x", hash.Sum(nil))
}
