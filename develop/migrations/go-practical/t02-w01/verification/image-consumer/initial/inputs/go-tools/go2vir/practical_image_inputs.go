package main

import (
	"bytes"
	"fmt"
	"maps"
	"sort"
	"strings"

	"golang.org/x/mod/module"
)

// Independently owned bytes restored through a verified rooted image view.
// The original roots are names for module correspondence, never paths to open.
// This consumer has no filesystem/process callback or registered-context claim.
type practicalImageInputs struct {
	snapshot    practicalSourceSnapshot
	origins     map[string]string
	archives    map[string]practicalModuleArchive
	archiveSums map[string]practicalModuleSums
	counters    captureCounters
	failed      error
}

func (view *practicalReplayView) restoreOriginalInputs() (*practicalImageInputs, error) {
	view.mutex.Lock()
	defer view.mutex.Unlock()
	if err := view.ready(); err != nil {
		return nil, err
	}
	if err := verifyPracticalReplayMaterialization(view.root, view.plan); err != nil {
		return nil, view.reject(err)
	}
	result := &practicalImageInputs{
		origins: maps.Clone(view.plan.origins), archives: map[string]practicalModuleArchive{},
		archiveSums: map[string]practicalModuleSums{},
		snapshot: practicalSourceSnapshot{roots: append([]string(nil), view.plan.namespace.roots...),
			buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{},
			directories: map[string][]practicalDirectoryEntry{}},
	}
	for key, entries := range view.plan.namespace.directories {
		if err := result.counters.addDirectory(len(entries)); err != nil {
			return nil, view.reject(err)
		}
		result.snapshot.directories[key] = append([]practicalDirectoryEntry(nil), entries...)
	}
	for key, object := range view.plan.captured {
		buffer, err := view.readCapturedLocked(object.Capability, object.Path, object.Kind)
		if err != nil {
			return nil, err
		}
		if key != practicalCaptureKey(object.Capability, object.Path) || buffer.object != object {
			return nil, view.reject(fmt.Errorf("restored original input crossed its captured namespace identity"))
		}
		if err := result.counters.admitCandidate(object.Bytes, object.Kind == "overlay", true); err != nil {
			return nil, view.reject(err)
		}
		result.snapshot.ids = append(result.snapshot.ids, object.ID)
		result.snapshot.paths[key] = object.ID
		result.snapshot.buffers[object.ID] = buffer
	}
	sort.Strings(result.snapshot.ids)
	result.snapshot.contentSHA256 = practicalSnapshotDigest(result.snapshot)
	if err := verifyPracticalReplayMaterialization(view.root, view.plan); err != nil {
		return nil, view.reject(err)
	}
	return result, nil
}

func (inputs *practicalImageInputs) reject(err error) error {
	if inputs.failed == nil {
		inputs.failed = err
	}
	return inputs.failed
}

func (inputs *practicalImageInputs) readSource(capability, relative, kind string) (practicalSourceBuffer, error) {
	if inputs.failed != nil {
		return practicalSourceBuffer{}, inputs.failed
	}
	buffer, err := inputs.snapshot.readSource(capability, relative, kind)
	if err != nil {
		return practicalSourceBuffer{}, inputs.reject(err)
	}
	if _, err := buffer.canonicalObject(); err != nil {
		return practicalSourceBuffer{}, inputs.reject(err)
	}
	return buffer, nil
}

func (inputs *practicalImageInputs) readDirectory(capability, relative string) ([]practicalDirectoryEntry, error) {
	if inputs.failed != nil {
		return nil, inputs.failed
	}
	entries, err := inputs.snapshot.readDirectory(capability, relative)
	if err != nil {
		return nil, inputs.reject(err)
	}
	return entries, nil
}

func (inputs *practicalImageInputs) originalRoots() map[string]string {
	return maps.Clone(inputs.origins)
}

func (inputs *practicalImageInputs) practicalPhysicalDirectory(location practicalModuleLocation) (string, error) {
	return practicalModulePhysicalDirectory(inputs, location)
}

func (inputs *practicalImageInputs) captureModule(location practicalModuleLocation, main bool) (practicalCapturedModule, error) {
	return capturePracticalModule(inputs, location, main)
}

func (inputs *practicalImageInputs) captureLocalReplacement(owner practicalCapturedModule, directive int) (practicalCapturedModule, error) {
	return capturePracticalLocalReplacement(inputs, owner, directive)
}

func (inputs *practicalImageInputs) captureVersionedModule(location practicalModuleLocation, identity module.Version, sums practicalModuleSums) (practicalVersionedModule, error) {
	return capturePracticalVersionedModule(inputs, location, identity, sums)
}

func (inputs *practicalImageInputs) originalModuleSums(sums practicalModuleSums) (practicalModuleSums, error) {
	source := sums.source.object
	buffer, err := inputs.readSource(source.Capability, source.Path, "go_sum")
	if err != nil || buffer.object != source {
		return practicalModuleSums{}, inputs.reject(fmt.Errorf("image checksums are not their original captured input"))
	}
	original, exists := inputs.archiveSums[source.ID]
	if !exists {
		original, err = parsePracticalModuleSums(buffer)
		if err != nil {
			return practicalModuleSums{}, inputs.reject(err)
		}
		inputs.archiveSums[source.ID] = original
	}
	original.entries = maps.Clone(original.entries)
	return original, nil
}

func (inputs *practicalImageInputs) captureModuleArchive(capability, relative string, identity module.Version, sums practicalModuleSums) (practicalModuleArchive, error) {
	original, err := inputs.originalModuleSums(sums)
	if err != nil {
		return practicalModuleArchive{}, err
	}
	buffer, err := inputs.readSource(capability, relative, "auxiliary")
	if err != nil {
		return practicalModuleArchive{}, err
	}
	expected := original.entries[identity]
	if old, exists := inputs.archives[buffer.object.ID]; exists {
		if old.identity != identity || old.sum != expected {
			return practicalModuleArchive{}, inputs.reject(fmt.Errorf("image module ZIP crossed original module/checksum ownership"))
		}
		return copyPracticalModuleArchive(old), nil
	}
	archive, err := parsePracticalModuleArchive(identity, buffer, expected, maximumDirectoryEntries-int(inputs.counters.directoryEntries))
	if err != nil {
		return practicalModuleArchive{}, inputs.reject(err)
	}
	inputs.counters.directoryEntries += archive.namespaceEntries
	inputs.archives[buffer.object.ID] = copyPracticalModuleArchive(archive)
	return copyPracticalModuleArchive(archive), nil
}

func (inputs *practicalImageInputs) bindOriginalModuleOutput(mainLocation, cache practicalModuleLocation, output []byte) ([]practicalResolvedModule, error) {
	if inputs.failed != nil {
		return nil, inputs.failed
	}
	if inputs.snapshot.contentSHA256 == "" || inputs.snapshot.contentSHA256 != practicalSnapshotDigest(inputs.snapshot) {
		return nil, inputs.reject(fmt.Errorf("image module consumer lost its complete original snapshot"))
	}
	main, err := inputs.captureModule(mainLocation, true)
	if err != nil {
		return nil, err
	}
	rows, err := decodePracticalGoModules(bytes.Clone(output))
	if err != nil {
		return nil, inputs.reject(err)
	}
	return bindPracticalGoModuleRows(inputs, main, cache, rows)
}

// Preserve the exact original capability subset, including unread names. This
// is used to restore the original source and export snapshots independently.
func (inputs *practicalImageInputs) snapshotForRoots(roots []string) (practicalSourceSnapshot, error) {
	if inputs.failed != nil {
		return practicalSourceSnapshot{}, inputs.failed
	}
	if len(roots) == 0 {
		return practicalSourceSnapshot{}, inputs.reject(fmt.Errorf("image snapshot subset has no original root"))
	}
	selected := map[string]bool{}
	result := practicalSourceSnapshot{buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{},
		directories: map[string][]practicalDirectoryEntry{}}
	for _, capability := range roots {
		if !inputs.snapshot.hasRoot(capability) || selected[capability] {
			return result, inputs.reject(fmt.Errorf("image snapshot subset has an unknown or duplicate original root"))
		}
		selected[capability] = true
		result.roots = append(result.roots, capability)
	}
	sort.Strings(result.roots)
	for key, entries := range inputs.snapshot.directories {
		capability, _, _ := strings.Cut(key, "\x00")
		if selected[capability] {
			result.directories[key] = append([]practicalDirectoryEntry(nil), entries...)
		}
	}
	for _, object := range inputs.snapshot.sourceObjects() {
		if selected[object.Capability] {
			buffer, err := inputs.readSource(object.Capability, object.Path, object.Kind)
			if err != nil {
				return practicalSourceSnapshot{}, err
			}
			result.ids = append(result.ids, object.ID)
			result.paths[practicalCaptureKey(object.Capability, object.Path)] = object.ID
			result.buffers[object.ID] = buffer
		}
	}
	result.contentSHA256 = practicalSnapshotDigest(result)
	return result, nil
}
