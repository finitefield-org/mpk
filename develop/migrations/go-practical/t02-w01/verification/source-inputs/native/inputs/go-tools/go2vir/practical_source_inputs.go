package main

import (
	"fmt"
	"sort"
)

// Complete source inputs for independent source analysis. Native type graphs,
// call/dispatch facts and supplied semantic adapters never enter this packet.
// The image/compiler identities retain its private provenance, not an installed
// context, source admission or a source/program proof.
type practicalSourceInputs struct {
	components                                practicalTypeKeyComponents
	imageDigest, compilerDigest, mainSourceID string
	requestedPackage, contentSHA256           string
}

// Retain every non-intrinsic package in the intact original export universe,
// including unselected files and bodies. The only type-component seed is the
// fixed logical Bool; the independent reader derives actual source types.
// Other captured metadata remains owned by the complete image identity.
func (owner *practicalOriginalPackages) sourceInputs() (practicalSourceInputs, error) {
	if owner == nil {
		return practicalSourceInputs{}, fmt.Errorf("source inputs require an owned original package universe")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	if owner.closed || owner.inputs == nil || owner.loader == nil || owner.loader.syntax == nil {
		return practicalSourceInputs{}, fmt.Errorf("source input owner is closed or incomplete")
	}
	for _, err := range []error{owner.inputs.failed, owner.loader.failed, owner.loader.syntax.failed} {
		if err != nil {
			return practicalSourceInputs{}, err
		}
	}
	for name, digest := range map[string]string{"source input image": owner.imageDigest, "source input compiler": owner.compilerDigest, "source input main module": owner.mainSourceID} {
		if err := validateDigest(name, digest); err != nil {
			return practicalSourceInputs{}, err
		}
	}
	selected := owner.loader.packages[owner.requestedPackage]
	if selected == nil || selected.moduleSourceID != owner.mainSourceID {
		return practicalSourceInputs{}, fmt.Errorf("source inputs crossed the original selected main module")
	}
	b := practicalTypeKeyBuilder{
		owner: owner, sources: map[string]practicalSourceBuffer{},
		packages: map[string]map[string]any{}, packageIDs: map[string]string{},
	}
	imports := make([]string, 0, len(owner.packages))
	for name, pkg := range owner.packages {
		if !pkg.intrinsic {
			imports = append(imports, name)
		}
	}
	sort.Strings(imports)
	roots := make([]string, 0, len(imports))
	for _, name := range imports {
		id, err := b.packageKey(name)
		if err != nil {
			return practicalSourceInputs{}, err
		}
		roots = append(roots, id)
	}
	if _, exists := b.packageIDs[owner.requestedPackage]; !exists {
		return practicalSourceInputs{}, fmt.Errorf("source inputs lost their original requested package")
	}
	sort.Strings(roots)
	objects := make([]practicalSourceObject, 0, len(b.sources))
	buffers := make([]practicalSourceBuffer, 0, len(b.sources))
	for _, buffer := range b.sources {
		objects = append(objects, buffer.object)
		buffers = append(buffers, buffer)
	}
	sort.Slice(objects, func(i, j int) bool { return objects[i].ID < objects[j].ID })
	sort.Slice(buffers, func(i, j int) bool { return buffers[i].object.ID < buffers[j].object.ID })
	seed, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": "bool", "width": nil, "signed": nil})
	if err != nil {
		return practicalSourceInputs{}, err
	}
	component := map[string]any{
		"schema": practicalTypeComponentsSchema, "source_objects": objects,
		"packages": practicalTypeKeyRows(b.packages), "package_roots": roots,
		"declarations": []any{}, "types": []any{seed}, "type_roots": []any{seed["id"]},
		"underlying": []any{}, "aliases": []any{},
	}
	componentHash, err := hashTypedCanonicalJSON(practicalTypeComponentsDomain, component)
	if err != nil {
		return practicalSourceInputs{}, err
	}
	component["content_sha256"] = componentHash
	canonical, err := canonicalJSON(component)
	if err != nil {
		return practicalSourceInputs{}, err
	}
	// This private identity ties the source-only component to its intact owner.
	// It does not replace the frozen captured-context/admission algorithm.
	digest, err := hashTypedCanonicalJSON("MPK-PRIVATE-GO-SOURCE-INPUTS-1.0", map[string]any{
		"image_sha256": owner.imageDigest, "compiler_sha256": owner.compilerDigest,
		"main_source_object_id": owner.mainSourceID, "requested_package": owner.requestedPackage,
		"source_components_sha256": componentHash,
	})
	if err != nil {
		return practicalSourceInputs{}, err
	}
	return practicalSourceInputs{
		components:  practicalTypeKeyComponents{canonical: canonical, sources: buffers},
		imageDigest: owner.imageDigest, compilerDigest: owner.compilerDigest,
		mainSourceID: owner.mainSourceID, requestedPackage: owner.requestedPackage,
		contentSHA256: digest,
	}, nil
}
