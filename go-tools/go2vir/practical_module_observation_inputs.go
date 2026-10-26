package main

import (
	"encoding/hex"
	"fmt"
	"sort"
)

const practicalModuleObservationInputsDomain = "MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0"

type practicalModuleObservationInputs struct {
	exports   practicalExportInputs
	canonical []byte
}

// Retain the complete original resolver observation and every selected manifest
// under the same owner lock as the source/export inputs. The receiver cannot
// substitute a graph, manifest, package, command or source adapter.
func (owner *practicalOriginalPackages) moduleObservationInputs(roots []practicalClosureRef) (practicalModuleObservationInputs, error) {
	if owner == nil {
		return practicalModuleObservationInputs{}, fmt.Errorf("module observations require the original owner")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	exports, err := owner.exportInputsLocked(roots)
	if err != nil {
		return practicalModuleObservationInputs{}, err
	}
	if len(owner.moduleOutput) == 0 || len(owner.modules) == 0 {
		return practicalModuleObservationInputs{}, fmt.Errorf("module inputs lost original resolver observations")
	}
	modules := append([]practicalResolvedModule(nil), owner.modules...)
	sort.Slice(modules, func(i, j int) bool { return modules[i].identity.Path < modules[j].identity.Path })
	rows := []any{}
	for i, selected := range modules {
		if i > 0 && modules[i-1].identity.Path == selected.identity.Path {
			return practicalModuleObservationInputs{}, fmt.Errorf("module inputs repeat a selected path")
		}
		original, err := practicalSnapshotBuffer(owner.inputs.snapshot, selected.manifest.object)
		if err != nil || original.object.Kind != "go_mod" {
			return practicalModuleObservationInputs{}, fmt.Errorf("module inputs lost an original selected manifest")
		}
		rows = append(rows, map[string]any{"path": selected.identity.Path, "version": selected.identity.Version, "replacement_path": selected.replacement.Path, "replacement_version": selected.replacement.Version, "manifest_source_object_id": original.object.ID})
	}
	value, err := decodeStrictJSON(exports.canonical)
	if err != nil {
		return practicalModuleObservationInputs{}, err
	}
	body := map[string]any{"schema": "mpk.go.practical.source_module_observations.v1", "source_export_inputs_sha256": value.(map[string]jsonValue)["content_sha256"], "modules": rows, "metadata_hex": hex.EncodeToString(owner.moduleOutput), "diagnostics_hex": hex.EncodeToString(owner.moduleDiagnostics)}
	digest, err := hashTypedCanonicalJSON(practicalModuleObservationInputsDomain, body)
	if err != nil {
		return practicalModuleObservationInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalModuleObservationInputs{}, err
	}
	return practicalModuleObservationInputs{exports, canonical}, nil
}
