package main

import (
	"fmt"
	"sort"
)

const practicalContractInputsDomain = "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0"

// The complete owned source packet and every captured raw sidecar from that
// same original universe. This is an input handoff, not a registered context
// or a source/program proof. It accepts no replacement annotation list.
type practicalContractInputs struct {
	source    practicalSourceInputs
	overlays  []practicalSourceBuffer
	canonical []byte
}

func (owner *practicalOriginalPackages) contractInputs() (practicalContractInputs, error) {
	if owner == nil {
		return practicalContractInputs{}, fmt.Errorf("contract inputs require an owned original package universe")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	return owner.contractInputsLocked()
}

// Caller holds the original owner lock for the complete handoff.
func (owner *practicalOriginalPackages) contractInputsLocked() (practicalContractInputs, error) {
	source, err := owner.sourceInputsLocked()
	if err != nil {
		return practicalContractInputs{}, err
	}
	snapshot := owner.inputs.snapshot
	if snapshot.contentSHA256 == "" || snapshot.contentSHA256 != practicalSnapshotDigest(snapshot) ||
		len(snapshot.ids) != len(snapshot.buffers) || len(snapshot.ids) != len(snapshot.paths) {
		return practicalContractInputs{}, owner.inputs.reject(fmt.Errorf("contract inputs lost their complete original captured inventory"))
	}
	component, err := decodeStrictJSON(source.components.canonical)
	if err != nil {
		return practicalContractInputs{}, err
	}
	root, ok := component.(map[string]jsonValue)
	if !ok {
		return practicalContractInputs{}, fmt.Errorf("contract inputs lost their complete source component")
	}
	overlays := []practicalSourceBuffer{}
	objects := []practicalSourceObject{}
	// The snapshot owns the full captured inventory, including annotations on
	// unused functions. Never infer absence from a source-only package list.
	for _, id := range owner.inputs.snapshot.ids {
		original := owner.inputs.snapshot.buffers[id]
		if original.object.Kind != "overlay" {
			continue
		}
		buffer, err := owner.inputs.readSource(original.object.Capability, original.object.Path, "overlay")
		if err != nil {
			return practicalContractInputs{}, err
		}
		if buffer.object != original.object || buffer.object.ID != id {
			return practicalContractInputs{}, owner.inputs.reject(fmt.Errorf("contract sidecar crossed its complete original identity"))
		}
		overlays = append(overlays, buffer)
		objects = append(objects, buffer.object)
	}
	sort.Slice(overlays, func(i, j int) bool { return overlays[i].object.ID < overlays[j].object.ID })
	sort.Slice(objects, func(i, j int) bool { return objects[i].ID < objects[j].ID })
	body := map[string]any{
		"schema":       "mpk.go.practical.source_contract_inputs.v1",
		"image_sha256": source.imageDigest, "compiler_sha256": source.compilerDigest,
		"main_source_object_id": source.mainSourceID, "requested_package": source.requestedPackage,
		"source_components_sha256": root["content_sha256"], "overlays": objects,
	}
	digest, err := hashTypedCanonicalJSON(practicalContractInputsDomain, body)
	if err != nil {
		return practicalContractInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalContractInputs{}, err
	}
	return practicalContractInputs{source: source, overlays: overlays, canonical: canonical}, nil
}
