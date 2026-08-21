package main

import (
	"fmt"
	"sort"
)

const (
	sourceManifestSchema = "mpk.source_manifest.v0"
	inputSetDomain       = "MPK-INPUT-SET-0.1"
	sourceManifestDomain = "MPK-SOURCE-MANIFEST-0.1"
	maximumManifestBytes = 4_194_304
)

type manifestUnit struct {
	Identity string `json:"identity"`
	Name     string `json:"name"`
	Kind     string `json:"kind"`
}

type manifestInput struct {
	Kind           string `json:"kind"`
	NormalizedPath string `json:"normalized_path"`
	SizeBytes      int64  `json:"size_bytes"`
	SHA256         string `json:"sha256"`
}

type sourceManifest struct {
	Schema             string                  `json:"schema"`
	SourceLanguage     string                  `json:"source_language"`
	SemanticProfile    string                  `json:"semantic_profile"`
	SemanticParameters semanticParameters      `json:"semantic_parameters"`
	Selection          goSelection             `json:"selection"`
	LimitProfile       string                  `json:"limit_profile"`
	ReleaseRegistry    releaseRegistryIdentity `json:"release_registry"`
	Toolchain          toolchainIdentity       `json:"toolchain"`
	Frontend           frontendIdentity        `json:"frontend"`
	Units              []manifestUnit          `json:"units"`
	Target             targetIdentity          `json:"target"`
	Inputs             []manifestInput         `json:"inputs"`
	InputSetHash       string                  `json:"input_set_hash"`
	VIRHash            string                  `json:"vir_hash"`
	SourceMapHash      string                  `json:"source_map_hash"`
	SourceManifestHash string                  `json:"source_manifest_hash"`
}

func buildSourceManifest(request lowerRequest, capture sourceCapture, loaded packageLoadResult, selection validatedLauncherSelection, virHash, sourceMapHash string) (sourceManifest, []byte, error) {
	if capture.SelectedPackage != request.Package || request.SemanticProfile != goSemanticProfile || selection.Target.ID != request.Target || selection.Target.PointerWidth != goPointerWidth ||
		selection.Registry.ID != request.ReleaseRegistryID || selection.Registry.RegistrySHA256 != request.ReleaseRegistrySHA256 ||
		selection.Frontend.BundleID != request.FrontendBundleID || selection.Frontend.BinarySHA256 != request.FrontendSHA256 ||
		selection.Toolchain.BundleID != request.ToolchainBundleID || selection.Toolchain.DistributionSHA256 != request.ToolchainDistributionSHA256 {
		return sourceManifest{}, nil, fmt.Errorf("manifest identity projection differs from the validated request")
	}
	if !sha256Pattern.MatchString(virHash) || !sha256Pattern.MatchString(sourceMapHash) {
		return sourceManifest{}, nil, fmt.Errorf("manifest artifact hashes are invalid")
	}
	inputs, err := manifestInputs(capture, loaded)
	if err != nil {
		return sourceManifest{}, nil, err
	}
	inputSetHash, err := hashTypedCanonicalJSON(inputSetDomain, inputs)
	if err != nil {
		return sourceManifest{}, nil, err
	}
	units := make([]manifestUnit, 0, len(loaded.Packages))
	for _, packageValue := range loaded.Packages {
		units = append(units, manifestUnit{Identity: packageValue.PackagePath, Name: packageValue.Name, Kind: "package"})
	}
	sort.Slice(units, func(left, right int) bool { return units[left].Identity < units[right].Identity })
	if len(units) == 0 || len(units) > 256 {
		return sourceManifest{}, nil, fmt.Errorf("manifest unit count is invalid")
	}
	for index := 1; index < len(units); index++ {
		if units[index-1].Identity >= units[index].Identity {
			return sourceManifest{}, nil, fmt.Errorf("manifest units are not strictly sorted")
		}
	}
	manifest := sourceManifest{
		Schema:             sourceManifestSchema,
		SourceLanguage:     "go",
		SemanticProfile:    request.SemanticProfile,
		SemanticParameters: semanticParameters{TargetID: request.Target, PointerWidth: goPointerWidth},
		Selection:          goSelection{Package: request.Package, Function: request.Function},
		LimitProfile:       selection.LimitProfileID,
		ReleaseRegistry:    selection.Registry,
		Toolchain:          selection.Toolchain,
		Frontend:           selection.Frontend,
		Units:              units,
		Target:             selection.Target,
		Inputs:             inputs,
		InputSetHash:       inputSetHash,
		VIRHash:            virHash,
		SourceMapHash:      sourceMapHash,
		SourceManifestHash: zeroSHA256(),
	}
	strict, err := strictValueFromTyped(manifest)
	if err != nil {
		return sourceManifest{}, nil, err
	}
	payload, err := withoutRootField(strict, "source_manifest_hash")
	if err != nil {
		return sourceManifest{}, nil, err
	}
	manifestHash, err := hashCanonicalJSON(sourceManifestDomain, payload)
	if err != nil {
		return sourceManifest{}, nil, err
	}
	manifest.SourceManifestHash = manifestHash
	canonical, err := canonicalJSON(manifest)
	if err != nil {
		return sourceManifest{}, nil, err
	}
	if len(canonical) > maximumManifestBytes {
		return sourceManifest{}, nil, fmt.Errorf("source manifest exceeds the canonical byte limit")
	}
	return manifest, canonical, nil
}

func manifestInputs(capture sourceCapture, loaded packageLoadResult) ([]manifestInput, error) {
	loadedSources := make(map[string]struct{})
	for _, packageValue := range loaded.Packages {
		for _, path := range packageValue.CompiledGoFiles {
			loadedSources[path] = struct{}{}
		}
	}
	inputs := make([]manifestInput, 0, len(capture.Inputs))
	buildManifests := 0
	for _, captured := range capture.Inputs {
		if !sha256Pattern.MatchString(captured.SHA256) || captured.SHA256 != sha256Hex(captured.Bytes) {
			return nil, fmt.Errorf("captured manifest input differs from its immutable digest")
		}
		switch captured.Kind {
		case buildManifestInputKind:
			if captured.NormalizedPath != "go.mod" || len(captured.Bytes) == 0 {
				return nil, fmt.Errorf("manifest build input set is invalid")
			}
			buildManifests++
		case lockfileInputKind:
			if captured.NormalizedPath != "go.sum" || len(captured.Bytes) != 0 {
				return nil, fmt.Errorf("manifest lockfile input set is invalid")
			}
		case sourceInputKind:
			if len(captured.Bytes) == 0 {
				return nil, fmt.Errorf("manifest source input is empty")
			}
			if _, used := loadedSources[captured.NormalizedPath]; !used {
				return nil, fmt.Errorf("captured source is absent from the compiled package inventory")
			}
			delete(loadedSources, captured.NormalizedPath)
		case contractInputKind:
			if len(captured.Bytes) == 0 {
				return nil, fmt.Errorf("manifest contract input is empty")
			}
		default:
			return nil, fmt.Errorf("unknown captured manifest input kind %q", captured.Kind)
		}
		inputs = append(inputs, manifestInput{
			Kind:           captured.Kind,
			NormalizedPath: captured.NormalizedPath,
			SizeBytes:      int64(len(captured.Bytes)),
			SHA256:         captured.SHA256,
		})
	}
	if buildManifests != 1 || len(loadedSources) != 0 || len(inputs) == 0 || len(inputs) > maximumManifestInputs {
		return nil, fmt.Errorf("manifest input inventory is incomplete")
	}
	sort.Slice(inputs, func(left, right int) bool {
		if inputs[left].NormalizedPath != inputs[right].NormalizedPath {
			return inputs[left].NormalizedPath < inputs[right].NormalizedPath
		}
		return inputs[left].Kind < inputs[right].Kind
	})
	for index := 1; index < len(inputs); index++ {
		if inputs[index-1].NormalizedPath >= inputs[index].NormalizedPath {
			return nil, fmt.Errorf("manifest input paths are not unique and sorted")
		}
	}
	return inputs, nil
}

func strictValueFromTyped(value any) (jsonValue, error) {
	canonical, err := canonicalJSON(value)
	if err != nil {
		return nil, err
	}
	return decodeStrictJSON(canonical)
}

func zeroSHA256() string {
	return "0000000000000000000000000000000000000000000000000000000000000000"
}
