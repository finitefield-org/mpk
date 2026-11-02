package main

import (
	"fmt"
	"path/filepath"
	"strings"
)

const practicalCompilerInputsDomain = "MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0"

// Original prepared-bundle bytes have the existing bundle limits, separately
// from application capture. A compiler executable is not an application source
// file and must not be truncated or relabeled to fit its16MiB file budget.
type practicalCompilerFile struct {
	entry inventoryFile
	data  []byte
}

type practicalCompilerInputs struct {
	exports   practicalExportInputs
	canonical []byte
	files     []practicalCompilerFile
}

func (inputs *practicalCompilerInputs) Close() {
	if inputs == nil {
		return
	}
	for i := range inputs.files {
		clear(inputs.files[i].data)
	}
	inputs.files = nil
	clear(inputs.canonical)
	inputs.canonical = nil
}

// Consume the already materialized complete original compiler through a live
// verified image view. No caller file list, root, content or metadata adapter is
// accepted. The returned bundle owns its bytes after source/image retirement.
// This is byte/inventory correspondence, not installed execution/context or a
// proof that the compiler implements Go correctly.
func (image *practicalReplayImage) compilerInputs(owner *practicalOriginalPackages, roots []practicalClosureRef) (result *practicalCompilerInputs, failure error) {
	if image == nil || owner == nil {
		return nil, fmt.Errorf("compiler inputs require their original image and packages")
	}
	view, err := image.openVerifiedView()
	if err != nil {
		return nil, err
	}
	defer func() {
		if err := view.Close(); err != nil && failure == nil {
			failure = err
		}
		if failure != nil && result != nil {
			result.Close()
			result = nil
		}
	}()
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	if owner.closed || owner.inputs == nil || owner.loader == nil || owner.loader.stdlib == nil || image.compiler == nil ||
		owner.imageDigest != image.contentSHA256 || owner.compilerDigest != image.compiler.selection.Toolchain.DistributionSHA256 ||
		owner.inputs.origins[owner.loader.stdlib.location.capability] != image.compilerOriginal {
		return nil, fmt.Errorf("compiler inputs crossed original image/package/compiler ownership")
	}
	compiler := *image.compiler
	if err := compiler.revalidate(); err != nil {
		return nil, err
	}
	exports, err := owner.exportInputsLocked(roots)
	if err != nil {
		return nil, err
	}
	projection, err := decodePracticalObservationJSON(exports.canonical)
	if err != nil {
		return nil, err
	}
	result = &practicalCompilerInputs{exports: exports}
	for _, expected := range compiler.candidate.ToolchainInventory.Files {
		original := filepath.Join(image.compilerOriginal, filepath.FromSlash(strings.TrimPrefix(expected.Path, "go/")))
		name := filepath.ToSlash(filepath.Join("namespace", original[1:]))
		bound, ok := view.plan.files[original]
		if !ok || bound != (practicalReplayExpectedFile{original, expected.SizeBytes, expected.SHA256, expected.Executable}) {
			return result, fmt.Errorf("original compiler inventory is not the materialized image inventory")
		}
		info, err := view.root.Lstat(name)
		if err != nil || !releaseFileModeValid(info.Mode(), expected.Executable) || info.Size() != expected.SizeBytes {
			return result, fmt.Errorf("original compiler file has missing or changed bytes/mode")
		}
		raw, err := readPracticalRegularFile(view.root, name, info, int(maximumBundleFileBytes))
		if err != nil || int64(len(raw)) != expected.SizeBytes || sha256Hex(raw) != expected.SHA256 {
			clear(raw)
			return result, fmt.Errorf("original compiler file changed during its complete bounded read")
		}
		result.files = append(result.files, practicalCompilerFile{entry: expected, data: raw})
	}
	if err := view.check(); err != nil {
		return result, err
	}
	if err := compiler.revalidate(); err != nil {
		return result, err
	}
	components := []any{}
	for _, component := range compiler.candidate.ToolchainComponents {
		components = append(components, map[string]any{"identity": component.Identity, "inventory": component.Inventory})
	}
	body := map[string]any{
		"schema": "mpk.go.practical.compiler_image_inputs.v1", "source_export_inputs_sha256": projection["content_sha256"],
		"sdk_capability": owner.loader.stdlib.location.capability, "original_root": image.compilerOriginal,
		"host": compiler.host, "toolchain": compiler.candidate.Toolchain, "inventory": compiler.candidate.ToolchainInventory, "components": components,
	}
	digest, err := hashTypedCanonicalJSON(practicalCompilerInputsDomain, body)
	if err != nil {
		return result, err
	}
	body["content_sha256"] = digest
	result.canonical, err = canonicalJSON(body)
	return result, err
}
