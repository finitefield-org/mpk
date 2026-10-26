package main

import (
	"encoding/hex"
	"fmt"
	"os"
	"sort"
	"strings"
)

const practicalSnapshotInputsDomain = "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0"
const practicalCaptureInputsDomain = "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0"

type practicalSnapshotInputEntry struct {
	NameHex  string `json:"name_hex"`
	TypeBits uint32 `json:"type_bits"`
}
type practicalSnapshotInputDirectory struct {
	Capability string                        `json:"capability"`
	Path       string                        `json:"path"`
	Entries    []practicalSnapshotInputEntry `json:"entries"`
}

// The complete immutable observed namespace and all captured bytes. Native
// names retain arbitrary bytes; an unread file is never invented as absent.
type practicalSnapshotInputs struct {
	canonical []byte
	sources   []practicalSourceBuffer
}

func (snapshot practicalSourceSnapshot) snapshotInputs() (practicalSnapshotInputs, error) {
	if snapshot.contentSHA256 == "" || snapshot.contentSHA256 != practicalSnapshotDigest(snapshot) ||
		len(snapshot.ids) != len(snapshot.buffers) || len(snapshot.ids) != len(snapshot.paths) || len(snapshot.roots) == 0 {
		return practicalSnapshotInputs{}, fmt.Errorf("snapshot handoff lost its complete saved inventory")
	}
	if len(snapshot.roots) > maximumVisitedDirectories {
		return practicalSnapshotInputs{}, fmt.Errorf("snapshot capability count exceeds the frozen bound")
	}
	folded := map[string]bool{}
	for i, capability := range snapshot.roots {
		key := foldPracticalASCII(capability)
		if !validPracticalLogicalPath(capability) || folded[key] || i > 0 && snapshot.roots[i-1] >= capability {
			return practicalSnapshotInputs{}, fmt.Errorf("invalid original snapshot capability set")
		}
		folded[key] = true
	}
	counters := captureCounters{}
	objects := []practicalSourceObject{}
	sources := []practicalSourceBuffer{}
	locations := map[string]bool{}
	for i, id := range snapshot.ids {
		original, exists := snapshot.buffers[id]
		if !exists || original.object.ID != id || i > 0 && snapshot.ids[i-1] >= id {
			return practicalSnapshotInputs{}, fmt.Errorf("snapshot handoff has an inconsistent original source set")
		}
		buffer, err := snapshot.readSource(original.object.Capability, original.object.Path, original.object.Kind)
		if err != nil {
			return practicalSnapshotInputs{}, err
		}
		key := practicalCaptureKey(original.object.Capability, foldPracticalASCII(original.object.Path))
		if buffer.object != original.object || locations[key] {
			return practicalSnapshotInputs{}, fmt.Errorf("snapshot handoff crossed an original source location")
		}
		locations[key] = true
		if _, err := buffer.canonicalObject(); err != nil {
			return practicalSnapshotInputs{}, err
		}
		if err := counters.admitCandidate(buffer.object.Bytes, buffer.object.Kind == "overlay", true); err != nil {
			return practicalSnapshotInputs{}, err
		}
		objects = append(objects, buffer.object)
		sources = append(sources, buffer)
	}
	keys := make([]string, 0, len(snapshot.directories))
	for key := range snapshot.directories {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	directories := []practicalSnapshotInputDirectory{}
	for _, key := range keys {
		capability, relative, ok := strings.Cut(key, "\x00")
		if !ok {
			return practicalSnapshotInputs{}, fmt.Errorf("snapshot directory lost its original key")
		}
		entries, err := snapshot.readDirectory(capability, relative)
		if err != nil {
			return practicalSnapshotInputs{}, err
		}
		if err := counters.addDirectory(len(entries)); err != nil {
			return practicalSnapshotInputs{}, err
		}
		encoded := make([]practicalSnapshotInputEntry, 0, len(entries))
		for i, entry := range entries {
			if entry.Name == "" || entry.Name == "." || entry.Name == ".." || strings.ContainsAny(entry.Name, "/\x00") || entry.Type&^os.ModeType != 0 || i > 0 && entries[i-1].Name >= entry.Name {
				return practicalSnapshotInputs{}, fmt.Errorf("invalid original directory entry")
			}
			encoded = append(encoded, practicalSnapshotInputEntry{hex.EncodeToString([]byte(entry.Name)), uint32(entry.Type)})
		}
		directories = append(directories, practicalSnapshotInputDirectory{capability, relative, encoded})
	}
	body := map[string]any{
		"schema":       "mpk.go.practical.source_snapshot_inputs.v1",
		"capabilities": snapshot.rootCapabilities(), "directories": directories,
		"objects": objects, "snapshot_sha256": snapshot.contentSHA256,
	}
	digest, err := hashTypedCanonicalJSON(practicalSnapshotInputsDomain, body)
	if err != nil {
		return practicalSnapshotInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalSnapshotInputs{}, err
	}
	return practicalSnapshotInputs{canonical, sources}, nil
}

type practicalCaptureInputs struct {
	selection practicalSelectionInputs
	snapshot  practicalSnapshotInputs
	canonical []byte
}

// No replacement inventory or namespace callback is accepted. The original
// owner supplies both sets under one lock; authentication remains separate.
func (owner *practicalOriginalPackages) captureInputs(roots []practicalClosureRef) (practicalCaptureInputs, error) {
	if owner == nil {
		return practicalCaptureInputs{}, fmt.Errorf("capture inputs require the original package owner")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	return owner.captureInputsLocked(roots)
}

func (owner *practicalOriginalPackages) captureInputsLocked(roots []practicalClosureRef) (practicalCaptureInputs, error) {
	selection, err := owner.selectionInputsLocked(roots)
	if err != nil {
		return practicalCaptureInputs{}, err
	}
	snapshot, err := owner.inputs.snapshot.snapshotInputs()
	if err != nil {
		return practicalCaptureInputs{}, owner.inputs.reject(err)
	}
	selected, err := decodeStrictJSON(selection.canonical)
	if err != nil {
		return practicalCaptureInputs{}, err
	}
	complete, err := decodeStrictJSON(snapshot.canonical)
	if err != nil {
		return practicalCaptureInputs{}, err
	}
	body := map[string]any{
		"schema":                  "mpk.go.practical.source_capture_inputs.v1",
		"selection_inputs_sha256": selected.(map[string]jsonValue)["content_sha256"],
		"snapshot_inputs_sha256":  complete.(map[string]jsonValue)["content_sha256"],
	}
	digest, err := hashTypedCanonicalJSON(practicalCaptureInputsDomain, body)
	if err != nil {
		return practicalCaptureInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalCaptureInputs{}, err
	}
	return practicalCaptureInputs{selection, snapshot, canonical}, nil
}
