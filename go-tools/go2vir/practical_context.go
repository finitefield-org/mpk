package main

import (
	"bytes"
	"fmt"
	"go/token"
	"runtime"
	"strings"
	"unicode/utf8"
)

// These values identify the inactive, Go/Rust-only candidate configuration.
// The foundation descriptor is a frozen recipe inventory, not a checked
// foundation, installed profile, source capture or application theorem.
const (
	practicalCandidateRegistrySchema  = "mpk.semantic_profile.registry.v3"
	practicalCandidateEntrySchema     = "mpk.semantic_profile.entry.v3"
	practicalCandidateContextSchema   = "mpk.semantic_context.v3"
	practicalCandidateRequestSchema   = "mpk.validated_semantic_request.v3"
	practicalCandidateProfile         = "mpk.go.practical.v1"
	practicalCandidateParamsSchema    = "mpk.semantic_parameters.go_practical.v1"
	practicalCandidateSelectionSchema = "mpk.selection.go_functions.v1"
	practicalCandidateRegistryHash    = "6c300215b21893014f393940416cebc7aa83ac74443918db5fa33a7c7615a7bc"
	practicalCandidateFoundationHash  = "7b038ab4a8fcac4079bdb2228dd6345c8489224233fec81544e2aa98283a225d"
	practicalCandidateLimitsHash      = "49ab79bb15643fa34b5b0d9b15ce4c833d098e38b70e89f0af3206bdac570551"
)

type practicalCandidateReceiver struct {
	Package  string `json:"package"`
	TypeName string `json:"type_name"`
}

type practicalCandidateSelector struct {
	Package  string                      `json:"package"`
	Function string                      `json:"function"`
	Receiver *practicalCandidateReceiver `json:"receiver"`
}

// Own only immutable canonical bytes. No caller registry, context, policy
// adapter, expected hash or foundation bundle is accepted by the producer.
type practicalCandidateRequest struct {
	registry, context, selection, request []byte
}

func (request *practicalCandidateRequest) registryBytes() []byte {
	return bytes.Clone(request.registry)
}

func (request *practicalCandidateRequest) contextBytes() []byte {
	return bytes.Clone(request.context)
}

func (request *practicalCandidateRequest) selectionBytes() []byte {
	return bytes.Clone(request.selection)
}

func (request *practicalCandidateRequest) requestBytes() []byte {
	return bytes.Clone(request.request)
}

func practicalCandidateContracts(suffix string) map[string]string {
	result := make(map[string]string, 9)
	for _, name := range []string{"ai", "evidence", "frontend", "manifest", "policy", "release", "source_map", "vc", "vir"} {
		result[name] = "mpk.profile." + name + "." + suffix + ".v2"
	}
	return result
}

func practicalCandidateEntry(language, profile, parameters, selection, suffix string, foundation any, expected string) (map[string]any, error) {
	entry := map[string]any{
		"schema": practicalCandidateEntrySchema, "source_language": language,
		"semantic_profile": profile, "semantic_parameters_schema": parameters,
		"selection_schema": selection, "foundation_descriptor": foundation,
		"contracts": practicalCandidateContracts(suffix),
	}
	digest, err := hashTypedCanonicalJSON("MPK-SEMANTIC-PROFILE-ENTRY-3.0", entry)
	if err != nil || digest != expected {
		return nil, fmt.Errorf("inactive candidate entry does not match its compiled identity")
	}
	entry["entry_sha256"] = digest
	return entry, nil
}

func practicalCandidateFoundation() map[string]any {
	return map[string]any{
		"schema": "mpk.go.foundation_descriptor.v1", "id": "mpk.go.practical.foundation.v1",
		"content_sha256": practicalCandidateFoundationHash,
	}
}

func practicalCandidateRegistry() (map[string]any, error) {
	fixed, err := practicalCandidateEntry("go", goSemanticProfile, successorSemanticParametersSchema,
		successorSelectionSchema, "go_fixed", nil, "ccff8b793a2ed6cbdfd1fbb534216d5f75c87a172b931d0397c426f5e3bee70c")
	if err != nil {
		return nil, err
	}
	practical, err := practicalCandidateEntry("go", practicalCandidateProfile, practicalCandidateParamsSchema,
		practicalCandidateSelectionSchema, "go_practical", practicalCandidateFoundation(), "0c15131de1279800ee5ee58c4c9a45f9a423e09b1721d9ba198868aa79fe5f9a")
	if err != nil {
		return nil, err
	}
	rust, err := practicalCandidateEntry("rust", "mpk.rust.checked.v0", "mpk.semantic_parameters.rust_checked.v0",
		"mpk.selection.rust_function.v0", "rust_checked", nil, "05dfce894254c6e0b4b72aceb98ab0da8869eafcf45fa3331e3de5f67f4e4508")
	if err != nil {
		return nil, err
	}
	registry := map[string]any{
		"schema": practicalCandidateRegistrySchema, "id": practicalCandidateRegistrySchema,
		"revision": int64(1), "profiles": []any{fixed, practical, rust},
	}
	digest, err := hashTypedCanonicalJSON("MPK-SEMANTIC-PROFILE-REGISTRY-3.0", registry)
	if err != nil || digest != practicalCandidateRegistryHash {
		return nil, fmt.Errorf("inactive candidate registry does not match its compiled identity")
	}
	registry["registry_sha256"] = digest
	return registry, nil
}

func practicalCandidateParameters() map[string]any {
	return map[string]any{
		"target_id": "linux/amd64", "pointer_width": int64(64), "goos": "linux", "goarch": "amd64",
		"goamd64": "v1", "compiler_release": practicalCompilerRelease,
		"effective_languages": []string{"go1.22", "go1.23", "go1.24", "go1.25"},
		"limits_id":           "mpk.go.practical.limits.v1", "limits_sha256": practicalCandidateLimitsHash,
	}
}

func practicalCandidateIdentifier(name string) bool {
	return name != "_" && token.IsIdentifier(name)
}

func practicalCandidateRoots(roots []practicalCandidateSelector) ([]practicalCandidateSelector, error) {
	if len(roots) == 0 || len(roots) > 8192 {
		return nil, fmt.Errorf("practical selection needs one to 8192 roots")
	}
	result := make([]practicalCandidateSelector, 0, len(roots))
	var previous []byte
	for _, input := range roots {
		root := input
		if input.Receiver != nil {
			receiver := *input.Receiver
			root.Receiver = &receiver
		}
		if !utf8.ValidString(root.Package) || root.Package == "" || strings.HasPrefix(root.Package, "/") ||
			strings.ContainsAny(root.Package, "\\\x00\n\r") || !practicalCandidateIdentifier(root.Function) {
			return nil, fmt.Errorf("practical selection has an invalid package or function")
		}
		if root.Receiver == nil {
			if root.Function == "init" {
				return nil, fmt.Errorf("package init is outside function-entry selection")
			}
		} else if root.Receiver.Package != root.Package || !practicalCandidateIdentifier(root.Receiver.TypeName) {
			return nil, fmt.Errorf("practical selection has an invalid value receiver")
		}
		canonical, err := canonicalJSON(root)
		if err != nil {
			return nil, err
		}
		if previous != nil && bytes.Compare(previous, canonical) >= 0 {
			return nil, fmt.Errorf("practical selection roots are not unique canonical order")
		}
		previous = canonical
		result = append(result, root)
	}
	return result, nil
}

func newPracticalCandidateRequest(roots []practicalCandidateSelector) (*practicalCandidateRequest, error) {
	if runtime.Version() != practicalCompilerRelease {
		return nil, fmt.Errorf("practical candidate requires the exact Go1.25.0 frontend")
	}
	selected, err := practicalCandidateRoots(roots)
	if err != nil {
		return nil, err
	}
	registry, err := practicalCandidateRegistry()
	if err != nil {
		return nil, err
	}
	context := map[string]any{
		"schema":               practicalCandidateContextSchema,
		"profile_registry":     map[string]any{"schema": practicalCandidateRegistrySchema, "id": practicalCandidateRegistrySchema, "revision": int64(1), "registry_sha256": practicalCandidateRegistryHash},
		"profile_entry_sha256": "0c15131de1279800ee5ee58c4c9a45f9a423e09b1721d9ba198868aa79fe5f9a",
		"source_language":      "go", "semantic_profile": practicalCandidateProfile,
		"semantic_parameters":   map[string]any{"schema": practicalCandidateParamsSchema, "value": practicalCandidateParameters()},
		"foundation_descriptor": practicalCandidateFoundation(),
	}
	selection := map[string]any{"schema": practicalCandidateSelectionSchema, "value": map[string]any{"roots": selected}}
	body := map[string]any{"schema": practicalCandidateRequestSchema, "semantic_context": context, "selection": selection}
	digest, err := hashTypedCanonicalJSON("MPK-VALIDATED-SEMANTIC-REQUEST-3.0", body)
	if err != nil {
		return nil, err
	}
	body["request_sha256"] = digest
	result := &practicalCandidateRequest{}
	for _, item := range []struct {
		value any
		dest  *[]byte
	}{{registry, &result.registry}, {context, &result.context}, {selection, &result.selection}, {body, &result.request}} {
		*item.dest, err = canonicalJSON(item.value)
		if err != nil {
			return nil, err
		}
	}
	return result, nil
}
