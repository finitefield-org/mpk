package main

import (
	"encoding/hex"
	"fmt"

	"golang.org/x/mod/module"
)

const practicalModuleGraphInputsDomain = "MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-1.0"

type practicalModuleGraphInputs struct {
	modules   practicalModuleObservationInputs
	canonical []byte
}

// Carry all original bytes and both original-root/final summary inventories
// under the original source/export/module owner. Roots, edges and projections
// remain native preparation observations pending independent reconstruction.
func (owner *practicalOriginalPackages) moduleGraphInputs(roots []practicalClosureRef) (practicalModuleGraphInputs, error) {
	if owner == nil {
		return practicalModuleGraphInputs{}, fmt.Errorf("module graph inputs require the original owner")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	modules, err := owner.moduleObservationInputsLocked(roots)
	if err != nil {
		return practicalModuleGraphInputs{}, err
	}
	graph := owner.moduleGraph
	if graph == nil || len(graph.output) == 0 {
		return practicalModuleGraphInputs{}, fmt.Errorf("module graph inputs have no original native graph observation")
	}
	identity := func(m module.Version) map[string]any { return map[string]any{"path": m.Path, "version": m.Version} }
	rows := func(summaries []practicalGraphSummary) ([]any, error) {
		result := []any{}
		for _, summary := range summaries {
			var source any
			if summary.source != nil {
				buffer, err := practicalSnapshotBuffer(owner.inputs.snapshot, summary.source.object)
				if err != nil || buffer.object.Kind != "go_mod" {
					return nil, fmt.Errorf("module graph inputs lost their original manifest")
				}
				source = buffer.object.ID
			} else if summary.identity.Path != "go" && summary.identity.Path != "toolchain" {
				return nil, fmt.Errorf("real graph module has no original source")
			}
			requirements := []any{}
			for _, r := range summary.require {
				requirements = append(requirements, identity(r))
			}
			result = append(result, map[string]any{"module": identity(summary.identity), "replacement_path_hex": hex.EncodeToString([]byte(summary.replacement.Path)), "replacement_version": summary.replacement.Version, "manifest_source_object_id": source, "go_version": summary.goVersion, "pruned": summary.pruned, "requirements": requirements})
		}
		return result, nil
	}
	initial, err := rows(graph.initial)
	if err != nil {
		return practicalModuleGraphInputs{}, err
	}
	final, err := rows(graph.final)
	if err != nil {
		return practicalModuleGraphInputs{}, err
	}
	value, err := decodeStrictJSON(modules.canonical)
	if err != nil {
		return practicalModuleGraphInputs{}, err
	}
	body := map[string]any{"schema": "mpk.go.practical.source_module_graph_inputs.v1", "source_module_observations_sha256": value.(map[string]jsonValue)["content_sha256"], "metadata_hex": hex.EncodeToString(graph.output), "diagnostics_hex": hex.EncodeToString(graph.diagnostics), "initial_summaries": initial, "final_summaries": final}
	digest, err := hashTypedCanonicalJSON(practicalModuleGraphInputsDomain, body)
	if err != nil {
		return practicalModuleGraphInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalModuleGraphInputs{}, err
	}
	return practicalModuleGraphInputs{modules, canonical}, nil
}
