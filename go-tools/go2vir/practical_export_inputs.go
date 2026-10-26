package main

import (
	"encoding/hex"
	"fmt"
	"path"
	"sort"
)

const practicalExportInputsDomain = "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0"

type practicalExportInputs struct {
	capture   practicalCaptureInputs
	canonical []byte
}

// The original owner supplies its complete command output and export universe
// under one lock. No caller metadata, package/source adapter or export callback
// enters this boundary. Installed compiler/context authority remains separate.
func (owner *practicalOriginalPackages) exportInputs(roots []practicalClosureRef) (practicalExportInputs, error) {
	if owner == nil {
		return practicalExportInputs{}, fmt.Errorf("export inputs require the original owner")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	capture, err := owner.captureInputsLocked(roots)
	if err != nil {
		return practicalExportInputs{}, err
	}
	if owner.loader.stdlib == nil || len(owner.packageOutput) == 0 {
		return practicalExportInputs{}, fmt.Errorf("export inputs lost original compiler observations")
	}
	rows, err := decodePracticalPackageExports(owner.packageOutput)
	if err != nil {
		return practicalExportInputs{}, err
	}
	if err := validatePracticalExportRoot(rows, owner.requestedPackage); err != nil {
		return practicalExportInputs{}, err
	}
	if len(rows) != len(owner.packages) {
		return practicalExportInputs{}, fmt.Errorf("export inputs lost complete package membership")
	}
	origins := []any{}
	for _, capability := range owner.inputs.snapshot.roots {
		original, ok := owner.inputs.origins[capability]
		if !ok || !practicalReplayAbsolute(original) {
			return practicalExportInputs{}, fmt.Errorf("export inputs lost original capability roots")
		}
		origins = append(origins, map[string]any{"capability": capability, "path": original})
	}
	names := []string{}
	for name := range owner.packages {
		names = append(names, name)
	}
	sort.Strings(names)
	packages := []any{}
	for _, name := range names {
		pkg := owner.packages[name]
		if len(pkg.sources) == 0 {
			return practicalExportInputs{}, fmt.Errorf("export package has no original Go source")
		}
		first := pkg.sources[0]
		directory := path.Dir(first.Path)
		if directory == "." {
			directory = ""
		}
		var export any
		if pkg.data != nil {
			buffer, err := practicalSnapshotBuffer(owner.inputs.snapshot, *pkg.data)
			if err != nil {
				return practicalExportInputs{}, err
			}
			if err := validatePracticalExportArchive(buffer.data, pkg.buildID); err != nil {
				return practicalExportInputs{}, err
			}
			export = pkg.data.ID
		}
		for _, source := range pkg.sources {
			if source.Capability != first.Capability || path.Dir(source.Path) != path.Dir(first.Path) {
				return practicalExportInputs{}, fmt.Errorf("export package crossed original source locations")
			}
			if _, err := practicalSnapshotBuffer(owner.inputs.snapshot, source); err != nil {
				return practicalExportInputs{}, err
			}
		}
		packages = append(packages, map[string]any{"import_path": name, "capability": first.Capability, "directory": directory, "export_source_object_id": export})
	}
	selected, err := decodeStrictJSON(capture.canonical)
	if err != nil {
		return practicalExportInputs{}, err
	}
	body := map[string]any{"schema": "mpk.go.practical.source_export_inputs.v1", "source_capture_inputs_sha256": selected.(map[string]jsonValue)["content_sha256"], "image_sha256": owner.imageDigest, "compiler_sha256": owner.compilerDigest, "go_binary_sha256": owner.loader.stdlib.compilerSHA256, "sdk_capability": owner.loader.stdlib.location.capability, "requested_package": owner.requestedPackage, "origins": origins, "packages": packages, "metadata_hex": hex.EncodeToString(owner.packageOutput), "diagnostics_hex": hex.EncodeToString(owner.packageDiagnostics)}
	digest, err := hashTypedCanonicalJSON(practicalExportInputsDomain, body)
	if err != nil {
		return practicalExportInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalExportInputs{}, err
	}
	return practicalExportInputs{capture, canonical}, nil
}
