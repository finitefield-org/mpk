#!/usr/bin/env python3
"""Materialize the source overlay for the staging-only successor go2vir build."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


SOURCE_SHA256 = {
    "bundle_selection.go": "3edf6c0ae55243d330d5c43f1962fd9ab2268fefd93066ad87ee1e5273711730",
    "emit.go": "6da74b868da85caf5cc4ada0f66e9f23b4dcdf186568f8b760a922a89845bc41",
    "main.go": "bdf05d56138e987e72726fc95883d5e4fcc64599c6d3b40feff2d8aadb637cf9",
    "manifest.go": "7346fbddd624041ddbc04b70d495f27f0d8575263df73dc387067ef1f5d8e4d6",
    "protocol.go": "ca48eea4285edbc04a36f257745060a29e5a769455c9cd3720ba98b3c4037a75",
    "registered_selection.go": "a35437f1d168cf3ae7af5e46536c33a07794e571ae0548b229c597bfdf8a9e87",
    "vir_types.go": "2b3e334749ee1c02bb33c2bb194ee1a53ea952971565922b128d0f6d32cdd69a",
}


SUCCESSOR_TYPES = r'''

const (
	successorProfileRegistrySchema = "mpk.semantic_profile.registry.v1"
	successorProfileRegistryID = "mpk.semantic_profile.registry.v1"
	successorProfileRegistryRevision = int64(2)
	successorProfileRegistryRevisionArgument = "2"
	successorProfileRegistrySHA256 = "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75"
	successorGoProfileEntrySHA256 = "b10ec338d1f2b3fefc015e4d46c27def43e92ff3d87341624b48c93db951ca96"
	successorSemanticParametersSchema = "mpk.semantic_parameters.go_fixed.v0"
	successorSelectionSchema = "mpk.selection.go_function.v0"
)

type successorProfileRegistryIdentity struct {
	Schema string `json:"schema"`
	ID string `json:"id"`
	Revision int64 `json:"revision"`
	RegistrySHA256 string `json:"registry_sha256"`
}

type successorSemanticParametersEnvelope struct {
	Schema string `json:"schema"`
	Value semanticParameters `json:"value"`
}

type successorSemanticContext struct {
	ProfileRegistry successorProfileRegistryIdentity `json:"profile_registry"`
	ProfileEntrySHA256 string `json:"profile_entry_sha256"`
	SourceLanguage string `json:"source_language"`
	SemanticProfile string `json:"semantic_profile"`
	SemanticParameters successorSemanticParametersEnvelope `json:"semantic_parameters"`
}

type successorSelectionEnvelope struct {
	Schema string `json:"schema"`
	Value goSelection `json:"value"`
}

func fixedSuccessorSemanticContext() successorSemanticContext {
	return successorSemanticContext{
		ProfileRegistry: successorProfileRegistryIdentity{
			Schema: successorProfileRegistrySchema,
			ID: successorProfileRegistryID,
			Revision: successorProfileRegistryRevision,
			RegistrySHA256: successorProfileRegistrySHA256,
		},
		ProfileEntrySHA256: successorGoProfileEntrySHA256,
		SourceLanguage: "go",
		SemanticProfile: goSemanticProfile,
		SemanticParameters: successorSemanticParametersEnvelope{
			Schema: successorSemanticParametersSchema,
			Value: semanticParameters{TargetID: goTarget, PointerWidth: goPointerWidth},
		},
	}
}

func successorSelection(value goSelection) successorSelectionEnvelope {
	return successorSelectionEnvelope{Schema: successorSelectionSchema, Value: value}
}

func marshalSuccessor(value any) ([]byte, error) {
	return json.Marshal(value)
}

func (value virModule) MarshalJSON() ([]byte, error) {
	type successorVIR struct {
		Schema string `json:"schema"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		Units []virUnit `json:"units"`
		VIRHash string `json:"vir_hash"`
	}
	return marshalSuccessor(successorVIR{
		Schema: value.Schema,
		SemanticContext: fixedSuccessorSemanticContext(),
		Units: value.Units,
		VIRHash: value.VIRHash,
	})
}

func (value virContract) MarshalJSON() ([]byte, error) {
	type successorContract struct {
		SemanticContext successorSemanticContext `json:"semantic_context"`
		UnitID string `json:"unit_id"`
		FunctionID string `json:"function_id"`
		Requires []virContractExpr `json:"requires"`
		Ensures []virContractExpr `json:"ensures"`
		Modifies []string `json:"modifies"`
		Panic string `json:"panic"`
		Termination string `json:"termination"`
		Loops []virLoopContract `json:"loops"`
		ContractHash string `json:"contract_hash"`
	}
	return marshalSuccessor(successorContract{
		SemanticContext: fixedSuccessorSemanticContext(),
		UnitID: value.UnitID,
		FunctionID: value.FunctionID,
		Requires: value.Requires,
		Ensures: value.Ensures,
		Modifies: value.Modifies,
		Panic: value.Panic,
		Termination: value.Termination,
		Loops: value.Loops,
		ContractHash: value.ContractHash,
	})
}

func (value sourceMap) MarshalJSON() ([]byte, error) {
	type successorMap struct {
		Schema string `json:"schema"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		SourceIRSchema string `json:"source_ir_schema"`
		SourceIRHash string `json:"source_ir_hash"`
		Entries []sourceMapEntry `json:"entries"`
		SourceMapHash string `json:"source_map_hash"`
	}
	return marshalSuccessor(successorMap{
		Schema: value.Schema,
		SemanticContext: fixedSuccessorSemanticContext(),
		SourceIRSchema: value.SourceIRSchema,
		SourceIRHash: value.SourceIRHash,
		Entries: value.Entries,
		SourceMapHash: value.SourceMapHash,
	})
}
'''


SUCCESSOR_MANIFEST = r'''

func (value sourceManifest) MarshalJSON() ([]byte, error) {
	type successorTarget struct {
		ID string `json:"id"`
		PointerWidth int64 `json:"pointer_width"`
	}
	type successorManifest struct {
		Schema string `json:"schema"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		Selection successorSelectionEnvelope `json:"selection"`
		LimitProfile string `json:"limit_profile"`
		ReleaseRegistry releaseRegistryIdentity `json:"release_registry"`
		Toolchain toolchainIdentity `json:"toolchain"`
		Frontend frontendIdentity `json:"frontend"`
		Units []manifestUnit `json:"units"`
		Target successorTarget `json:"target"`
		Inputs []manifestInput `json:"inputs"`
		InputSetHash string `json:"input_set_hash"`
		VIRHash string `json:"vir_hash"`
		SourceMapHash string `json:"source_map_hash"`
		SourceManifestHash string `json:"source_manifest_hash"`
	}
	return marshalSuccessor(successorManifest{
		Schema: value.Schema,
		SemanticContext: fixedSuccessorSemanticContext(),
		Selection: successorSelection(value.Selection),
		LimitProfile: value.LimitProfile,
		ReleaseRegistry: value.ReleaseRegistry,
		Toolchain: value.Toolchain,
		Frontend: value.Frontend,
		Units: value.Units,
		Target: successorTarget{ID: value.Target.ID, PointerWidth: value.Target.PointerWidth},
		Inputs: value.Inputs,
		InputSetHash: value.InputSetHash,
		VIRHash: value.VIRHash,
		SourceMapHash: value.SourceMapHash,
		SourceManifestHash: value.SourceManifestHash,
	})
}
'''


SUCCESSOR_PROTOCOL = r'''

func (value nonSuccessEnvelope) MarshalJSON() ([]byte, error) {
	type successorEnvelope struct {
		Schema string `json:"schema"`
		Status string `json:"status"`
		Phase string `json:"phase"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		Selection successorSelectionEnvelope `json:"selection"`
		RejectedFeatures []issue `json:"rejected_features"`
		Diagnostics []issue `json:"diagnostics"`
	}
	return marshalSuccessor(successorEnvelope{
		Schema: value.Schema,
		Status: value.Status,
		Phase: value.Phase,
		SemanticContext: fixedSuccessorSemanticContext(),
		Selection: successorSelection(value.Selection),
		RejectedFeatures: value.RejectedFeatures,
		Diagnostics: value.Diagnostics,
	})
}
'''


SUCCESSOR_SUCCESS = r'''

func (value successEnvelope) MarshalJSON() ([]byte, error) {
	type successorEnvelope struct {
		Schema string `json:"schema"`
		Status string `json:"status"`
		Phase string `json:"phase"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		Selection successorSelectionEnvelope `json:"selection"`
		IR virArtifact `json:"ir"`
		SourceManifest sourceManifest `json:"source_manifest"`
		SourceMap sourceMap `json:"source_map"`
		RejectedFeatures []issue `json:"rejected_features"`
		Diagnostics []issue `json:"diagnostics"`
	}
	return marshalSuccessor(successorEnvelope{
		Schema: value.Schema,
		Status: value.Status,
		Phase: value.Phase,
		SemanticContext: fixedSuccessorSemanticContext(),
		Selection: successorSelection(value.Selection),
		IR: value.IR,
		SourceManifest: value.SourceManifest,
		SourceMap: value.SourceMap,
		RejectedFeatures: value.RejectedFeatures,
		Diagnostics: value.Diagnostics,
	})
}
'''


def replace_once(source: str, old: str, new: str, path: str) -> str:
    count = source.count(old)
    if count != 1:
        raise ValueError(f"{path}: expected one migration anchor, found {count}")
    return source.replace(old, new)


def transform(path: str, source: str) -> str:
    if path == "vir_types.go":
        replacements = {
            'virSchema         = "mpk.vir.v0"': 'virSchema         = "mpk.vir.v1"',
            'virHashDomain     = "MPK-VIR-0.1"': 'virHashDomain     = "MPK-VIR-1.0"',
            'contractDomain    = "MPK-CONTRACT-0.1"': 'contractDomain    = "MPK-CONTRACT-1.0"',
            'sourceMapSchema   = "mpk.source_map.v0"': 'sourceMapSchema   = "mpk.source_map.v1"',
            'sourceMapDomain   = "MPK-SOURCE-MAP-0.1"': 'sourceMapDomain   = "MPK-SOURCE-MAP-1.0"',
        }
        for old, new in replacements.items():
            source = replace_once(source, old, new, path)
        return source + SUCCESSOR_TYPES
    if path == "manifest.go":
        source = replace_once(
            source,
            'sourceManifestSchema = "mpk.source_manifest.v0"',
            'sourceManifestSchema = "mpk.source_manifest.v1"',
            path,
        )
        source = replace_once(
            source,
            'sourceManifestDomain = "MPK-SOURCE-MANIFEST-0.1"',
            'sourceManifestDomain = "MPK-SOURCE-MANIFEST-1.0"',
            path,
        )
        return source + SUCCESSOR_MANIFEST
    if path == "protocol.go":
        source = replace_once(
            source,
            'const frontendCLISchema = "mpk.frontend.cli.v0"',
            'const frontendCLISchema = "mpk.frontend.cli.v1"',
            path,
        )
        return source + SUCCESSOR_PROTOCOL
    if path == "emit.go":
        return source + SUCCESSOR_SUCCESS
    if path == "registered_selection.go":
        source = replace_once(
            source,
            'Schema:         "mpk.release.bundle_registry.v0",',
            'Schema:         "mpk.release.bundle_registry.v1",',
            path,
        )
        return replace_once(
            source,
            'registeredFrontendVersion = "go1.25.0-profile-v0"',
            'registeredFrontendVersion = "go1.25.0-profile-v1-staging"',
            path,
        )
    if path == "bundle_selection.go":
        return replace_once(
            source,
            'candidate.Registry.Schema != "mpk.release.bundle_registry.v0"',
            'candidate.Registry.Schema != "mpk.release.bundle_registry.v1"',
            path,
        )
    if path == "main.go":
        source = replace_once(
            source,
            "--function FUNCTION --frontend-bundle-id ID",
            "--function FUNCTION --profile-registry-id ID --profile-registry-revision REVISION --profile-registry-sha256 SHA256 --profile-entry-sha256 SHA256 --frontend-bundle-id ID",
            path,
        )
        source = replace_once(
            source,
            'registryID        = "mpk.release.registry.v0"',
            'registryID        = "mpk.release.registry.v1"',
            path,
        )
        source = replace_once(
            source,
            "\tSemanticProfile             string\n\tTarget                      string",
            "\tSemanticProfile             string\n\tProfileRegistryID           string\n\tProfileRegistryRevision     string\n\tProfileRegistrySHA256       string\n\tProfileEntrySHA256          string\n\tTarget                      string",
            path,
        )
        source = replace_once(source, "if len(args) < 24", "if len(args) < 32", path)
        source = replace_once(
            source,
            '\t\t{name: "--function", apply: func(r *lowerRequest, value string) { r.Function = value }},\n\t\t{name: "--frontend-bundle-id"',
            '\t\t{name: "--function", apply: func(r *lowerRequest, value string) { r.Function = value }},\n'
            '\t\t{name: "--profile-registry-id", apply: func(r *lowerRequest, value string) { r.ProfileRegistryID = value }},\n'
            '\t\t{name: "--profile-registry-revision", apply: func(r *lowerRequest, value string) { r.ProfileRegistryRevision = value }},\n'
            '\t\t{name: "--profile-registry-sha256", apply: func(r *lowerRequest, value string) { r.ProfileRegistrySHA256 = value }},\n'
            '\t\t{name: "--profile-entry-sha256", apply: func(r *lowerRequest, value string) { r.ProfileEntrySHA256 = value }},\n'
            '\t\t{name: "--frontend-bundle-id"',
            path,
        )
        source = replace_once(
            source,
            "\tif request.Target != goTarget {\n\t\treturn fmt.Errorf(\"--target must be %s\", goTarget)\n\t}\n",
            "\tif request.Target != goTarget {\n\t\treturn fmt.Errorf(\"--target must be %s\", goTarget)\n\t}\n"
            "\tif request.ProfileRegistryID != successorProfileRegistryID ||\n"
            "\t\trequest.ProfileRegistryRevision != successorProfileRegistryRevisionArgument ||\n"
            "\t\trequest.ProfileRegistrySHA256 != successorProfileRegistrySHA256 ||\n"
            "\t\trequest.ProfileEntrySHA256 != successorGoProfileEntrySHA256 {\n"
            "\t\treturn fmt.Errorf(\"semantic profile registry assertions do not match the staged Go profile\")\n"
            "\t}\n",
            path,
        )
        return source
    raise ValueError(f"unsupported overlay source {path}")


def materialize(
    repository: Path,
    output: Path,
    source_prefix: Path | None,
    overlay_prefix: Path | None,
) -> None:
    source_root = repository / "go-tools/go2vir"
    output.mkdir(parents=True, exist_ok=False)
    replacements: dict[str, str] = {}
    for name, expected_hash in sorted(SOURCE_SHA256.items()):
        path = source_root / name
        raw = path.read_bytes()
        actual_hash = hashlib.sha256(raw).hexdigest()
        if actual_hash != expected_hash:
            raise ValueError(f"{name}: active source changed ({actual_hash})")
        transformed = transform(name, raw.decode("utf-8")).encode("utf-8")
        destination = output / name
        destination.write_bytes(transformed)
        destination.chmod(0o444)
        source_name = (source_prefix / "go-tools/go2vir" / name) if source_prefix else path.resolve()
        overlay_name = (overlay_prefix / name) if overlay_prefix else destination.resolve()
        replacements[str(source_name)] = str(overlay_name)
    overlay = json.dumps(
        {"Replace": replacements},
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8") + b"\n"
    (output / "overlay.json").write_bytes(overlay)
    (output / "overlay.json").chmod(0o444)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-prefix", type=Path)
    parser.add_argument("--overlay-prefix", type=Path)
    arguments = parser.parse_args()
    materialize(
        arguments.repository.resolve(),
        arguments.output.resolve(),
        arguments.source_prefix,
        arguments.overlay_prefix,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
