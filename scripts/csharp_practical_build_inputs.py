#!/usr/bin/env python3
"""Private CSHARP-03-T01-W03 toolchain-closure and reproducibility harness."""

from __future__ import annotations

import copy
import json
import os
import stat
import sys
import tempfile
from pathlib import Path

import csharp_build_inputs as active


REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
DESCRIPTOR_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/build-inputs/build-inputs.json"
)
INVENTORY_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/build-inputs/candidate-inventory.json"
)
CAPTURE_INPUTS_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/capture/capture-inputs.json"
)
SYNTAX_INPUTS_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/syntax/syntax-inputs.json"
)
TYPES_INPUTS_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/types/types-inputs.json"
)
CONSTRUCTION_INPUTS_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/construction/construction-inputs.json"
)
INITIALIZATION_INPUTS_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/initialization/initialization-inputs.json"
)
INITIALIZATION_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w05.initialization_inputs.v1"
STRUCTURAL_INPUTS_PATH = (
    REPOSITORY_ROOT / "develop/migrations/csharp-03/structural/structural-inputs.json"
)
ARRAYS_INPUTS_PATH = REPOSITORY_ROOT / "develop/migrations/csharp-03/arrays/arrays-inputs.json"
ARRAYS_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w07.arrays_inputs.v1"
SEQUENCES_INPUTS_PATH = REPOSITORY_ROOT / "develop/migrations/csharp-03/sequences/sequences-inputs.json"
SEQUENCES_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w08.sequences_inputs.v1"
ORDERED_INPUTS_PATH = REPOSITORY_ROOT / "develop/migrations/csharp-03/ordered/ordered-inputs.json"
ORDERED_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w09.ordered_inputs.v1"
CODECS_INPUTS_PATH = REPOSITORY_ROOT / "develop/migrations/csharp-03/codecs/codecs-inputs.json"
CODECS_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w10.codecs_inputs.v1"
NUMERIC_INPUTS_PATH = REPOSITORY_ROOT / "develop/migrations/csharp-03/numeric/numeric-inputs.json"
NUMERIC_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w11.numeric_inputs.v1"
STRUCTURAL_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w06.structural_inputs.v1"

CONSTRUCTION_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w04.construction_inputs.v1"
TYPES_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w03.types_inputs.v1"
DESCRIPTOR_SCHEMA = "mpk.csharp_practical.t01_w03.private_build_inputs.v0"
INVENTORY_SCHEMA = "mpk.csharp_practical.t01_w03.private_candidate_inventory.v0"
CAPTURE_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w01.capture_inputs.v1"
SYNTAX_INPUTS_SCHEMA = "mpk.csharp_practical.t03_w02.syntax_inputs.v1"
WORK_ITEM = "CSHARP-03-T01-W03"
W02_COMMIT = "f84a5c6ff5122a3a5e64d9305fe999ed1f501f85"
W02_TREE = "c14885505d0eeb6901aa077dd6f497b2fc0a4d5d"
W02_INVENTORY_SHA256 = (
    "6b5b7f601f6174d61496424084d264604a5a3325a460a5c0640bfcd71a564c49"
)
W02_PROJECT_FILES_SHA256 = (
    "4193dc64e338730e67128010e0f17160305a51ed4ba2d4b0df13aad65d7fc443"
)
EXPECTED_ARCHIVE_SHA256 = (
    "a26bc0ad42ed424812caf25b5b8d73df95e2ccefaa0442282ecb8399c440a302"
)
EXPECTED_ARCHIVE_SIZE_BYTES = 10_516_480
EXPECTED_BUILD_RECIPE_SHA256 = (
    "7e1c2cf3b4794a85b22c1af1ca7943bb525123fb4abc9827db8dc45955a63d76"
)
EXPECTED_CANDIDATE_FILES_SHA256 = (
    "e02a1d95f8c7f9fe576de16575b6c1247bebca0f678f8ddfc26ead3ad64a395f"
)
EXPECTED_DESCRIPTOR_RAW_SHA256 = (
    "83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015"
)

EXPECTED_FORBIDDEN_DISCOVERY = {
    "ambient_references": "forbidden",
    "analyzers_at_compile": "forbidden",
    "nuget_package_cache_discovery": "forbidden",
    "project_evaluation": "forbidden",
    "restore": "forbidden",
    "source_generators": "forbidden",
    "reference_selection": "exact_embedded_reference_projection",
    "source_selection": "exact_project_files_manifest",
}

EXPECTED_ENVIRONMENT_VARIABLES = {
    "COMPlus_ReadyToRun": "0",
    "DOTNET_CLI_HOME": "$EMPTY_HOME",
    "DOTNET_CLI_TELEMETRY_OPTOUT": "1",
    "DOTNET_MULTILEVEL_LOOKUP": "0",
    "DOTNET_NOLOGO": "1",
    "DOTNET_ROOT": "$PINNED_DOTNET_ROOT",
    "DOTNET_SKIP_FIRST_TIME_EXPERIENCE": "1",
    "DOTNET_SYSTEM_GLOBALIZATION_INVARIANT": "1",
    "DOTNET_TieredCompilation": "0",
    "DOTNET_TieredPGO": "0",
    "HOME": "$EMPTY_HOME",
    "LANG": "C.UTF-8",
    "LC_ALL": "C.UTF-8",
    "NUGET_HTTP_CACHE_PATH": "$EMPTY_NUGET",
    "NUGET_PACKAGES": "$EMPTY_NUGET",
    "NUGET_PLUGINS_CACHE_PATH": "$EMPTY_NUGET",
    "PATH": "/nonexistent",
    "SOURCE_DATE_EPOCH": "0",
    "TMPDIR": "$FRESH_TMP",
    "TZ": "UTC",
}

EXPECTED_ENVIRONMENT_CLOSURE = {
    "ambient_probe": {
        "name": "MPK_CSHARP_PRACTICAL_UNLISTED_AMBIENT",
        "values": ["clean", "hostile"],
    },
    "build_process": "empty_then_exact_declared_variables",
    "unlisted_variables": "ignored",
    "variables": EXPECTED_ENVIRONMENT_VARIABLES,
    "wrapper_environment": {
        "PATH": "/usr/bin:/bin",
        "PYTHONDONTWRITEBYTECODE": "1",
        "TMPDIR": "/tmp",
    },
}


def expected_tar_stats() -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    for archive_id in sorted(active.EXPECTED_TAR_STATS):
        stats = active.EXPECTED_TAR_STATS[archive_id]
        records.append(
            {
                "archive_id": archive_id,
                "directory_0755": stats.get(("directory", 0o755), 0),
                "regular_0644": stats.get(("regular", 0o644), 0),
                "regular_0744": stats.get(("regular", 0o744), 0),
                "regular_0755": stats.get(("regular", 0o755), 0),
            }
        )
    return records


EXPECTED_OFFLINE_EXTRACTION = {
    "archive_cache_file_count": 6,
    "archive_cache_file_mode": "0444",
    "case_collisions": "forbidden",
    "copied_archive_file_mode": "0444",
    "directory_mode": "0755",
    "limits": {
        "max_archive_bytes": active.MAX_ARCHIVE_BYTES,
        "max_archive_entries": active.MAX_ARCHIVE_ENTRIES,
        "max_extracted_bytes": active.MAX_EXTRACTED_BYTES,
        "max_json_bytes": active.MAX_JSON_BYTES,
    },
    "network_after_cache_validation": "forbidden",
    "nupkg_regular_file_mode": "0644",
    "path_traversal": "forbidden",
    "symlinks": "forbidden",
    "tar_member_stats": expected_tar_stats(),
}

EXPECTED_ARCHIVE_LAYOUT = {
    "directory_mode": "0755",
    "file_mode": "0644",
    "format": "ustar",
    "gid": 0,
    "gname": "",
    "mtime": 0,
    "uid": 0,
    "uname": "",
}


def deep_copy(value: object) -> object:
    return copy.deepcopy(value)


def descriptor_sha256(descriptor: dict[str, object]) -> str:
    return active.raw_sha256(active.canonical(descriptor) + b"\n")


def validate_toolchain_snapshot(value: object) -> dict[str, object]:
    toolchain = active.exact_keys(
        value,
        {
            "archives",
            "host",
            "id",
            "managed_projection",
            "package_graph",
            "reference_projection",
            "roslyn_source",
            "schema",
            "toolchain_inputs_sha256",
        },
    )
    if (
        active.text(toolchain["schema"]) != "mpk.csharp.toolchain_inputs.v0"
        or active.text(toolchain["toolchain_inputs_sha256"]) != active.TOOLCHAIN_HASH
    ):
        raise active.CSharpBuildFailure()
    payload = dict(toolchain)
    del payload["toolchain_inputs_sha256"]
    if (
        len(active.canonical(payload)) != 29_335
        or active.typed_sha256(active.TOOLCHAIN_DOMAIN, payload)
        != active.TOOLCHAIN_HASH
    ):
        raise active.CSharpBuildFailure()
    active.validate_toolchain(toolchain)
    return toolchain


def validate_build_recipe(value: object) -> dict[str, object]:
    recipe = active.exact_keys(
        value,
        {
            "compiler",
            "compiler_arguments",
            "id",
            "language_version",
            "network_namespace",
            "package_restore",
            "runtime_framework",
            "runtime_version",
            "source_date_epoch",
            "target_framework",
        },
    )
    expected = {
        "compiler": "sdk/10.0.400/Roslyn/bincore/csc.dll",
        "compiler_arguments": list(active.COMPILER_ARGUMENTS),
        "id": "mpk.csharp.build_recipe.csc_direct.v0",
        "language_version": "14.0",
        "network_namespace": "required",
        "package_restore": "forbidden",
        "runtime_framework": "Microsoft.NETCore.App",
        "runtime_version": "10.0.11",
        "source_date_epoch": 0,
        "target_framework": "net10.0",
    }
    if recipe != expected:
        raise active.CSharpBuildFailure()
    return recipe


def observed_environment_variables() -> dict[str, str]:
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-practical-env-") as temporary:
        work = Path(temporary) / "work"
        result = active.closed_dotnet_environment(
            Path("/mpk/toolchain/dotnet"), work
        )
        replacements = {
            str(work / "empty-home"): "$EMPTY_HOME",
            str(work / "empty-nuget"): "$EMPTY_NUGET",
            str(work / "tmp"): "$FRESH_TMP",
            "/mpk/toolchain/dotnet": "$PINNED_DOTNET_ROOT",
        }
        return {key: replacements.get(value, value) for key, value in result.items()}


def validate_project_file_manifest(value: object) -> list[object]:
    records = active.array(value)
    if len(records) != 34:
        raise active.CSharpBuildFailure()
    for untyped in records:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        active.validate_relative_path(active.text(record["path"]))
        if active.integer(record["size_bytes"]) == 0:
            raise active.CSharpBuildFailure()
        active.validate_hex(record["sha256"], 64)
    if active.canonical_sha256(records) != W02_PROJECT_FILES_SHA256:
        raise active.CSharpBuildFailure()
    return records


def validate_descriptor_value(value: object) -> dict[str, object]:
    descriptor = active.exact_keys(
        value,
        {
            "baseline",
            "build_recipe",
            "candidate_inventory",
            "environment_closure",
            "forbidden_discovery",
            "notice_sources",
            "offline_extraction",
            "project_files",
            "project_root",
            "schema",
            "toolchain_inputs",
            "toolchain_inputs_sha256",
            "work_item",
        },
    )
    baseline = active.exact_keys(
        descriptor["baseline"],
        {"artifact_consumer_inventory", "raw_sha256", "source_commit", "source_tree"},
    )
    if (
        active.text(descriptor["schema"]) != DESCRIPTOR_SCHEMA
        or active.text(descriptor["work_item"]) != WORK_ITEM
        or active.text(descriptor["candidate_inventory"])
        != "develop/migrations/csharp-03/build-inputs/candidate-inventory.json"
        or active.text(descriptor["project_root"]) != "csharp-tools/csharp2vir"
        or active.text(descriptor["toolchain_inputs_sha256"])
        != active.TOOLCHAIN_HASH
        or baseline
        != {
            "artifact_consumer_inventory": (
                "develop/migrations/csharp-03/artifact-consumer-inventory.json"
            ),
            "raw_sha256": W02_INVENTORY_SHA256,
            "source_commit": W02_COMMIT,
            "source_tree": W02_TREE,
        }
    ):
        raise active.CSharpBuildFailure()

    toolchain = validate_toolchain_snapshot(descriptor["toolchain_inputs"])
    if active.text(toolchain["toolchain_inputs_sha256"]) != active.text(
        descriptor["toolchain_inputs_sha256"]
    ):
        raise active.CSharpBuildFailure()

    validate_project_file_manifest(descriptor["project_files"])
    validate_build_recipe(descriptor["build_recipe"])
    active.validate_notice_sources(active.array(descriptor["notice_sources"]))

    forbidden = active.exact_keys(
        descriptor["forbidden_discovery"], set(EXPECTED_FORBIDDEN_DISCOVERY)
    )
    environment = active.exact_keys(
        descriptor["environment_closure"], set(EXPECTED_ENVIRONMENT_CLOSURE)
    )
    extraction = active.exact_keys(
        descriptor["offline_extraction"], set(EXPECTED_OFFLINE_EXTRACTION)
    )
    if (
        forbidden != EXPECTED_FORBIDDEN_DISCOVERY
        or environment != EXPECTED_ENVIRONMENT_CLOSURE
        or extraction != EXPECTED_OFFLINE_EXTRACTION
        or observed_environment_variables() != EXPECTED_ENVIRONMENT_VARIABLES
    ):
        raise active.CSharpBuildFailure()
    return descriptor


def load_descriptor() -> dict[str, object]:
    return validate_descriptor_value(
        active.strict_json_file(DESCRIPTOR_PATH, canonical_transport=True)
    )


def validate_capture_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != CAPTURE_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W01"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_capture_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_INPUTS")
    return manifest


def load_capture_inputs() -> dict[str, object]:
    return validate_capture_inputs_value(
        active.strict_json_file(CAPTURE_INPUTS_PATH, canonical_transport=True)
    )


def validate_syntax_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != SYNTAX_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W02"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_syntax_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_INPUTS")
    return manifest


def load_syntax_inputs() -> dict[str, object]:
    return validate_syntax_inputs_value(
        active.strict_json_file(SYNTAX_INPUTS_PATH, canonical_transport=True)
    )


def validate_types_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != TYPES_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W03"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_types_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_INPUTS")
    return manifest


def load_types_inputs() -> dict[str, object]:
    return validate_types_inputs_value(
        active.strict_json_file(TYPES_INPUTS_PATH, canonical_transport=True)
    )


def validate_construction_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != CONSTRUCTION_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W04"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_construction_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_INPUTS")
    return manifest


def load_construction_inputs() -> dict[str, object]:
    return validate_construction_inputs_value(
        active.strict_json_file(CONSTRUCTION_INPUTS_PATH, canonical_transport=True)
    )


def validate_initialization_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != INITIALIZATION_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W05"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_initialization_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_INPUTS")
    return manifest


def load_initialization_inputs() -> dict[str, object]:
    return validate_initialization_inputs_value(
        active.strict_json_file(INITIALIZATION_INPUTS_PATH, canonical_transport=True)
    )


def validate_structural_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != STRUCTURAL_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W06"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_structural_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/structural/source-routes.json",
        "develop/migrations/csharp-03/structural/source.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_INPUTS")
    return manifest


def load_structural_inputs() -> dict[str, object]:
    return validate_structural_inputs_value(
        active.strict_json_file(STRUCTURAL_INPUTS_PATH, canonical_transport=True)
    )


def validate_arrays_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != ARRAYS_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W07"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_arrays_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_INPUTS")
    return manifest


def load_arrays_inputs() -> dict[str, object]:
    return validate_arrays_inputs_value(
        active.strict_json_file(ARRAYS_INPUTS_PATH, canonical_transport=True)
    )


def validate_sequences_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != SEQUENCES_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W08"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_sequences_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSequences.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/sequences/source-direct.json",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_INPUTS")
    return manifest


def load_sequences_inputs() -> dict[str, object]:
    return validate_sequences_inputs_value(
        active.strict_json_file(SEQUENCES_INPUTS_PATH, canonical_transport=True)
    )


def validate_ordered_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != ORDERED_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W09"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_ordered_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalOrderedCollections.cs",
        "csharp-tools/csharp2vir/PracticalSequences.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/ordered/source-ordered.json",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_INPUTS")
    return manifest


def load_ordered_inputs() -> dict[str, object]:
    return validate_ordered_inputs_value(
        active.strict_json_file(ORDERED_INPUTS_PATH, canonical_transport=True)
    )


def validate_codecs_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != CODECS_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W10"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_codecs_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalStrings.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/codecs/source-strings.json",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_INPUTS")
    return manifest


def load_codecs_inputs() -> dict[str, object]:
    return validate_codecs_inputs_value(
        active.strict_json_file(CODECS_INPUTS_PATH, canonical_transport=True)
    )


def validate_numeric_inputs_value(value: object) -> dict[str, object]:
    manifest = active.exact_keys(value, {"files", "schema", "work_item"})
    if (
        active.text(manifest["schema"]) != NUMERIC_INPUTS_SCHEMA
        or active.text(manifest["work_item"]) != "CSHARP-03-T03-W11"
    ):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_INPUTS")
    records = active.array(manifest["files"])
    expected_paths = [
        "crates/mpk-cli/tests/csharp_practical_numeric_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalNumeric.cs",
        "csharp-tools/csharp2vir/PracticalStrings.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/numeric/numeric-runtime.json",
    ]
    if len(records) != len(expected_paths):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_INPUTS")
    for untyped, expected_path in zip(records, expected_paths):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        path = active.validate_relative_path(active.text(record["path"]))
        if path != expected_path:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_INPUTS")
        size, sha256, _mode = active.hash_regular_file(
            REPOSITORY_ROOT / path, 2 * 1024 * 1024
        )
        if (
            active.integer(record["size_bytes"]) != size
            or active.validate_hex(active.text(record["sha256"]), 64) != sha256
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_INPUTS")
    return manifest


def load_numeric_inputs() -> dict[str, object]:
    return validate_numeric_inputs_value(
        active.strict_json_file(NUMERIC_INPUTS_PATH, canonical_transport=True)
    )


def copy_bound_file(
    source: Path,
    destination: Path,
    record: dict[str, object],
    failure_code: str,
) -> None:
    active.copy_candidate_file(source, destination)
    size, sha256, _sha512 = active.hash_regular_file(destination, 2 * 1024 * 1024)
    if (
        size != active.integer(record["size_bytes"])
        or sha256 != active.validate_hex(active.text(record["sha256"]), 64)
    ):
        raise active.CSharpBuildFailure(failure_code)


def validate_archive_cache_modes(
    toolchain: dict[str, object], archives_root: Path
) -> None:
    expected_names = {
        active.descriptor_cache_name(record)
        for record in active.archive_records(toolchain)
    }
    observed_names: set[str] = set()
    for path in archives_root.iterdir():
        metadata = path.lstat()
        if (
            stat.S_ISLNK(metadata.st_mode)
            or not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
            or stat.S_IMODE(metadata.st_mode) != 0o444
        ):
            raise active.CSharpBuildFailure()
        observed_names.add(path.name)
    if observed_names != expected_names:
        raise active.CSharpBuildFailure()


def private_candidate_inventory(
    generated: dict[str, object],
    descriptor: dict[str, object],
    archive: bytes,
) -> dict[str, object]:
    frontend = active.array(generated["frontend_files"])
    notices = active.array(generated["notice_files"])
    records = [*frontend, *notices]
    return {
        "archive_layout": EXPECTED_ARCHIVE_LAYOUT,
        "archive_sha256": active.raw_sha256(archive),
        "archive_size_bytes": len(archive),
        "build_count": 2,
        "build_recipe_sha256": active.canonical_sha256(
            descriptor["build_recipe"]
        ),
        "candidate_file_count": len(records),
        "candidate_files_sha256": active.canonical_sha256(records),
        "descriptor_raw_sha256": descriptor_sha256(descriptor),
        "frontend_files": frontend,
        "notice_files": notices,
        "project_files_sha256": active.canonical_sha256(
            active.array(descriptor["project_files"])
        ),
        "registration": {
            "active_registry_memberships": 0,
            "release_descriptor": "absent",
            "state": "private_unregistered",
        },
        "schema": INVENTORY_SCHEMA,
        "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
        "work_item": WORK_ITEM,
    }


def validate_file_records(value: object, expected_count: int) -> list[object]:
    records = active.array(value)
    if len(records) != expected_count:
        raise active.CSharpBuildFailure()
    previous = ""
    for untyped in records:
        record = active.exact_keys(
            untyped, {"mode", "path", "sha256", "size_bytes"}
        )
        path = active.validate_relative_path(active.text(record["path"]))
        if (
            path <= previous
            or active.text(record["mode"]) != "0644"
            or active.integer(record["size_bytes"]) == 0
        ):
            raise active.CSharpBuildFailure()
        previous = path
        active.validate_hex(record["sha256"], 64)
    return records


def validate_inventory_value(
    value: object, descriptor: dict[str, object]
) -> dict[str, object]:
    inventory = active.exact_keys(
        value,
        {
            "archive_layout",
            "archive_sha256",
            "archive_size_bytes",
            "build_count",
            "build_recipe_sha256",
            "candidate_file_count",
            "candidate_files_sha256",
            "descriptor_raw_sha256",
            "frontend_files",
            "notice_files",
            "project_files_sha256",
            "registration",
            "schema",
            "toolchain_inputs_sha256",
            "work_item",
        },
    )
    frontend = validate_file_records(inventory["frontend_files"], 5)
    notices = validate_file_records(inventory["notice_files"], 13)
    records = [*frontend, *notices]
    registration = active.exact_keys(
        inventory["registration"],
        {"active_registry_memberships", "release_descriptor", "state"},
    )
    layout = active.exact_keys(
        inventory["archive_layout"], set(EXPECTED_ARCHIVE_LAYOUT)
    )
    if (
        active.text(inventory["schema"]) != INVENTORY_SCHEMA
        or active.text(inventory["work_item"]) != WORK_ITEM
        or active.text(inventory["toolchain_inputs_sha256"])
        != active.TOOLCHAIN_HASH
        or active.integer(inventory["build_count"]) != 2
        or active.integer(inventory["candidate_file_count"]) != 18
        or active.integer(inventory["archive_size_bytes"])
        != EXPECTED_ARCHIVE_SIZE_BYTES
        or active.validate_hex(inventory["archive_sha256"], 64)
        != inventory["archive_sha256"]
        or active.validate_hex(inventory["candidate_files_sha256"], 64)
        != inventory["candidate_files_sha256"]
        or active.validate_hex(inventory["descriptor_raw_sha256"], 64)
        != inventory["descriptor_raw_sha256"]
        or active.validate_hex(inventory["project_files_sha256"], 64)
        != inventory["project_files_sha256"]
        or active.validate_hex(inventory["build_recipe_sha256"], 64)
        != inventory["build_recipe_sha256"]
        or inventory["descriptor_raw_sha256"] != descriptor_sha256(descriptor)
        or inventory["project_files_sha256"]
        != active.canonical_sha256(active.array(descriptor["project_files"]))
        or inventory["build_recipe_sha256"]
        != active.canonical_sha256(descriptor["build_recipe"])
        or inventory["candidate_files_sha256"]
        != active.canonical_sha256(records)
        or inventory["archive_sha256"] != EXPECTED_ARCHIVE_SHA256
        or inventory["build_recipe_sha256"] != EXPECTED_BUILD_RECIPE_SHA256
        or inventory["candidate_files_sha256"]
        != EXPECTED_CANDIDATE_FILES_SHA256
        or inventory["descriptor_raw_sha256"]
        != EXPECTED_DESCRIPTOR_RAW_SHA256
        or inventory["project_files_sha256"] != W02_PROJECT_FILES_SHA256
        or layout != EXPECTED_ARCHIVE_LAYOUT
        or registration
        != {
            "active_registry_memberships": 0,
            "release_descriptor": "absent",
            "state": "private_unregistered",
        }
    ):
        raise active.CSharpBuildFailure()
    return inventory


def load_inventory(descriptor: dict[str, object]) -> dict[str, object]:
    return validate_inventory_value(
        active.strict_json_file(INVENTORY_PATH, canonical_transport=True), descriptor
    )


def write_inventory(value: dict[str, object]) -> None:
    INVENTORY_PATH.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(
        prefix=".candidate-inventory-", dir=INVENTORY_PATH.parent
    )
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(active.canonical(value) + b"\n")
            output.flush()
            os.fsync(output.fileno())
        os.chmod(name, 0o644)
        os.replace(name, INVENTORY_PATH)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def checked_archives(toolchain: dict[str, object]) -> Path:
    archives = active.check_cached_archives(toolchain)
    validate_archive_cache_modes(toolchain, archives)
    return archives


def check_build_inputs() -> None:
    descriptor = load_descriptor()
    load_capture_inputs()
    load_syntax_inputs()
    load_types_inputs()
    load_construction_inputs()
    load_initialization_inputs()
    load_structural_inputs()
    load_arrays_inputs()
    load_sequences_inputs()
    load_ordered_inputs()
    load_codecs_inputs()
    load_numeric_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-input-check-"
    ) as temporary:
        active.materialize_closure(toolchain, archives, Path(temporary))
    active.validate_project_files(active.array(descriptor["project_files"]))
    load_inventory(descriptor)


def test_capture() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_capture_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-capture-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_CAPTURE_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-capture-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalCaptureHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-capture",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_capture_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_CAPTURE_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CAPTURE_TEST_FAILURE")


def test_syntax() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_syntax_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-syntax-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_SYNTAX_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-syntax-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalSyntaxHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-syntax",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_syntax_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_SYNTAX_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SYNTAX_TEST_FAILURE")


def test_types() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_types_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-types-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_TYPES_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-types-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalTypesHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-types",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_types_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_TYPES_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_TYPES_TEST_FAILURE")


def test_construction() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_construction_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-construction-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_CONSTRUCTION_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-construction-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalConstructionHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-construction",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_construction_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_CONSTRUCTION_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CONSTRUCTION_TEST_FAILURE")


def test_initialization() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_initialization_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-initialization-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_INITIALIZATION_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-initialization-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalInitializationHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-initialization",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_initialization_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_INITIALIZATION_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_INITIALIZATION_TEST_FAILURE")


def test_structural() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_structural_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-structural-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_STRUCTURAL_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-structural-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalStructuralHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-structural",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStructural.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_structural_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_STRUCTURAL_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
                str(copied["develop/migrations/csharp-03/structural/source.cs"]),
                str(copied["develop/migrations/csharp-03/structural/source-routes.json"]),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_STRUCTURAL_TEST_FAILURE")


def test_arrays() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_arrays_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-arrays-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_ARRAYS_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-arrays-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalArraysHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-arrays",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalArrays.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStructural.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_arrays_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_ARRAYS_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ARRAYS_TEST_FAILURE")


def test_sequences() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_sequences_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-sequences-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_SEQUENCES_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-sequences-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalSequencesHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-sequences",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalArrays.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSequences.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStructural.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_sequences_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_SEQUENCES_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SEQUENCES_TEST_FAILURE")


def test_ordered() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_ordered_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-ordered-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_ORDERED_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-ordered-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalOrderedHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-ordered",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalArrays.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSequences.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalOrderedCollections.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStructural.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_ordered_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_ORDERED_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_ORDERED_TEST_FAILURE")


def test_codecs() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_codecs_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-codecs-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_CODECS_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-codecs-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalCodecsHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-codecs",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalArrays.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStrings.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStructural.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_codecs_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_CODECS_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_CODECS_TEST_FAILURE")


def test_numeric() -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    project_records: dict[str, dict[str, object]] = {}
    project_files = active.array(descriptor["project_files"])
    active.validate_project_files(project_files)
    for untyped in project_files:
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        project_records[active.validate_relative_path(active.text(record["path"]))] = record
    manifest = load_numeric_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = checked_archives(toolchain)
    with tempfile.TemporaryDirectory(
        prefix="mpk-csharp-practical-numeric-test-"
    ) as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        work = temporary_root / "work"
        work.mkdir(mode=0o700, parents=True, exist_ok=False)
        copied: dict[str, Path] = {}
        for untyped in active.array(manifest["files"]):
            record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
            relative = active.text(record["path"])
            target = work / Path(relative).name
            copy_bound_file(
                REPOSITORY_ROOT / relative,
                target,
                record,
                "CSHARP_PRACTICAL_NUMERIC_INPUTS",
            )
            copied[relative] = target

        sdk = roots["dotnet-sdk-linux-x64"]
        compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
        output = work / "csharp2vir-practical-numeric-tests.dll"
        arguments = list(active.COMPILER_ARGUMENTS)
        arguments.extend(
            [
                "/out:" + str(output),
                "/main:Mpk.CSharp2Vir.PracticalNumericHarness",
                "/pathmap:" + str(work) + "=/_/csharp-practical-numeric",
            ]
        )
        reference_root = roots["microsoft-netcore-app-ref"]
        for untyped in active.array(toolchain["reference_projection"]["inventory"]):
            record = active.exact_keys(
                untyped, {"path", "size_bytes", "sha256"}
            )
            arguments.append(
                "/reference:" + str(reference_root / active.text(record["path"]))
            )
        managed_roots = {
            "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
            "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
        }
        managed_sources: list[tuple[Path, str]] = []
        for untyped in active.array(toolchain["managed_projection"]):
            record = active.exact_keys(
                untyped,
                {
                    "package_id",
                    "archive_path",
                    "runtime_path",
                    "size_bytes",
                    "sha256",
                },
            )
            source = managed_roots[active.text(record["package_id"])] / active.text(
                record["archive_path"]
            )
            arguments.append("/reference:" + str(source))
            managed_sources.append((source, Path(active.text(record["runtime_path"])).name))
        arguments.extend(
            [
                str(copied["csharp-tools/csharp2vir/PracticalArrays.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStrings.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalNumeric.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalCapture.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalDataTypes.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalConstruction.cs"]),
                str(copied["csharp-tools/csharp2vir/PracticalStructural.cs"]),
                str(copied["crates/mpk-cli/tests/csharp_practical_numeric_harness.cs"]),
            ]
        )
        build_environment = active.closed_dotnet_environment(
            sdk, temporary_root / "build-environment"
        )
        result = active.execute_isolated(
            [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
            cwd=work,
            environment=build_environment,
        )
        if (
            result.returncode != 0
            or result.stdout
            or result.stderr
            or not output.is_file()
        ):
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_TEST_BUILD")

        for source, name in managed_sources:
            active.copy_candidate_file(source, work / name)
        runtime_config = work / "csharp2vir.runtimeconfig.json"
        runtime_record = project_records.get("csharp2vir.runtimeconfig.json")
        if runtime_record is None:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_INPUTS")
        copy_bound_file(
            REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json",
            runtime_config,
            runtime_record,
            "CSHARP_PRACTICAL_NUMERIC_INPUTS",
        )
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = active.closed_dotnet_environment(
            runtime, temporary_root / "runtime-environment"
        )
        result = active.execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(runtime_config),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=work,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_NUMERIC_TEST_FAILURE")


def check_full(*, update: bool) -> None:
    active.validate_build_host()
    descriptor = load_descriptor()
    active.validate_project_files(active.array(descriptor["project_files"]))
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    checked_archives(toolchain)
    generated, archive = active.build_twice(toolchain, descriptor)
    private = private_candidate_inventory(generated, descriptor, archive)
    validate_inventory_value(private, descriptor)
    if update:
        write_inventory(private)
    elif private != load_inventory(descriptor):
        raise active.CSharpBuildFailure("CSHARP_PRACTICAL_BUILD_INVENTORY_MISMATCH")


def expect_rejected(callback: object) -> None:
    try:
        callback()
    except active.CSharpBuildFailure:
        return
    raise active.CSharpBuildFailure("CSHARP_PRACTICAL_SELF_TEST")


def mutate_hex(value: str) -> str:
    return ("0" if value[0] != "0" else "1") + value[1:]


def self_test() -> None:
    descriptor = load_descriptor()
    inventory = load_inventory(descriptor)
    capture_inputs = load_capture_inputs()
    syntax_inputs = load_syntax_inputs()

    changed_capture_inputs = deep_copy(capture_inputs)
    changed_capture_inputs["files"][0]["sha256"] = mutate_hex(
        changed_capture_inputs["files"][0]["sha256"]
    )
    expect_rejected(lambda: validate_capture_inputs_value(changed_capture_inputs))

    changed_syntax_inputs = deep_copy(syntax_inputs)
    changed_syntax_inputs["files"][2]["sha256"] = mutate_hex(
        changed_syntax_inputs["files"][2]["sha256"]
    )
    expect_rejected(lambda: validate_syntax_inputs_value(changed_syntax_inputs))

    types_inputs = load_types_inputs()
    changed_types_inputs = deep_copy(types_inputs)
    changed_types_inputs["files"][2]["sha256"] = mutate_hex(changed_types_inputs["files"][2]["sha256"])
    expect_rejected(lambda: validate_types_inputs_value(changed_types_inputs))

    construction_inputs = load_construction_inputs()
    for key, replacement in [("sha256", "0" * 64), ("size_bytes", 0), ("path", "unexpected.cs")]:
        changed_construction = deep_copy(construction_inputs)
        changed_construction["files"][2][key] = replacement
        expect_rejected(lambda: validate_construction_inputs_value(changed_construction))
    changed_construction = deep_copy(construction_inputs)
    changed_construction["files"].pop()
    expect_rejected(lambda: validate_construction_inputs_value(changed_construction))

    arrays_inputs = load_arrays_inputs()
    for key, replacement in (("sha256", "0" * 64), ("size_bytes", 0), ("path", "README.md")):
        changed = deep_copy(arrays_inputs)
        changed["files"][0][key] = replacement
        expect_rejected(lambda: validate_arrays_inputs_value(changed))
    changed = deep_copy(arrays_inputs)
    changed["files"].pop()
    expect_rejected(lambda: validate_arrays_inputs_value(changed))
    sequences_inputs = load_sequences_inputs()
    for key, replacement in (("sha256", "0" * 64), ("size_bytes", 0), ("path", "README.md")):
        changed = deep_copy(sequences_inputs)
        changed["files"][0][key] = replacement
        expect_rejected(lambda: validate_sequences_inputs_value(changed))
    changed = deep_copy(sequences_inputs)
    changed["files"].pop()
    expect_rejected(lambda: validate_sequences_inputs_value(changed))
    ordered_inputs = load_ordered_inputs()
    for key, replacement in (("sha256", "0" * 64), ("size_bytes", 0), ("path", "README.md")):
        changed = deep_copy(ordered_inputs)
        changed["files"][0][key] = replacement
        expect_rejected(lambda: validate_ordered_inputs_value(changed))
    changed = deep_copy(ordered_inputs)
    changed["files"].pop()
    expect_rejected(lambda: validate_ordered_inputs_value(changed))
    codecs_inputs = load_codecs_inputs()
    for key, replacement in (("sha256", "0" * 64), ("size_bytes", 0), ("path", "README.md")):
        changed = deep_copy(codecs_inputs)
        changed["files"][0][key] = replacement
        expect_rejected(lambda: validate_codecs_inputs_value(changed))
    changed = deep_copy(codecs_inputs)
    changed["files"].pop()
    expect_rejected(lambda: validate_codecs_inputs_value(changed))
    numeric_inputs = load_numeric_inputs()
    for key, replacement in (("sha256", "0" * 64), ("size_bytes", 0), ("path", "README.md")):
        changed = deep_copy(numeric_inputs)
        changed["files"][0][key] = replacement
        expect_rejected(lambda: validate_numeric_inputs_value(changed))
    changed = deep_copy(numeric_inputs)
    changed["files"].pop()
    expect_rejected(lambda: validate_numeric_inputs_value(changed))
    structural_inputs = load_structural_inputs()
    for key, replacement in (("sha256", "0" * 64), ("size_bytes", 0), ("path", "README.md")):
        changed = deep_copy(structural_inputs)
        changed["files"][0][key] = replacement
        expect_rejected(lambda: validate_structural_inputs_value(changed))
    changed = deep_copy(structural_inputs)
    changed["files"].pop()
    expect_rejected(lambda: validate_structural_inputs_value(changed))
    initialization_inputs = load_initialization_inputs()
    for key, replacement in [("sha256", "0" * 64), ("size_bytes", 0), ("path", "unexpected.cs")]:
        changed_initialization = deep_copy(initialization_inputs)
        changed_initialization["files"][2][key] = replacement
        expect_rejected(lambda: validate_initialization_inputs_value(changed_initialization))
    changed_initialization = deep_copy(initialization_inputs)
    changed_initialization["files"].pop()
    expect_rejected(lambda: validate_initialization_inputs_value(changed_initialization))

    descriptor_mutations: list[tuple[str, object]] = []

    changed = deep_copy(descriptor)
    changed["toolchain_inputs"]["archives"][0]["sha256"] = mutate_hex(
        changed["toolchain_inputs"]["archives"][0]["sha256"]
    )
    descriptor_mutations.append(("one-byte", changed))

    changed = deep_copy(descriptor)
    changed["project_files"].pop()
    descriptor_mutations.append(("file-count", changed))

    changed = deep_copy(descriptor)
    changed["offline_extraction"]["archive_cache_file_mode"] = "0644"
    descriptor_mutations.append(("mode", changed))

    changed = deep_copy(descriptor)
    changed["build_recipe"]["compiler_arguments"][0] = "/logo"
    descriptor_mutations.append(("flag", changed))

    changed = deep_copy(descriptor)
    changed["toolchain_inputs"]["reference_projection"]["inventory"][0][
        "sha256"
    ] = mutate_hex(
        changed["toolchain_inputs"]["reference_projection"]["inventory"][0][
            "sha256"
        ]
    )
    descriptor_mutations.append(("reference", changed))

    changed = deep_copy(descriptor)
    changed["environment_closure"]["variables"]["TZ"] = "Asia/Tokyo"
    descriptor_mutations.append(("declared-environment", changed))

    changed = deep_copy(descriptor)
    changed["notice_sources"].pop()
    descriptor_mutations.append(("notice-count", changed))

    changed = deep_copy(descriptor)
    changed["forbidden_discovery"]["restore"] = "allowed"
    descriptor_mutations.append(("restore-policy", changed))

    for _name, mutation in descriptor_mutations:
        expect_rejected(lambda mutation=mutation: validate_descriptor_value(mutation))

    inventory_mutations: list[tuple[str, object]] = []

    changed = deep_copy(inventory)
    changed["frontend_files"][0]["sha256"] = mutate_hex(
        changed["frontend_files"][0]["sha256"]
    )
    inventory_mutations.append(("candidate-byte", changed))

    changed = deep_copy(inventory)
    changed["notice_files"].pop()
    inventory_mutations.append(("candidate-file-count", changed))

    changed = deep_copy(inventory)
    changed["frontend_files"][0]["mode"] = "0755"
    inventory_mutations.append(("candidate-mode", changed))

    changed = deep_copy(inventory)
    changed["archive_sha256"] = mutate_hex(changed["archive_sha256"])
    inventory_mutations.append(("archive-byte", changed))

    for _name, mutation in inventory_mutations:
        expect_rejected(
            lambda mutation=mutation: validate_inventory_value(mutation, descriptor)
        )


def main(argv: list[str]) -> int:
    try:
        if argv == ["check-build-inputs"]:
            check_build_inputs()
        elif argv == ["test-capture"]:
            test_capture()
        elif argv == ["test-syntax"]:
            test_syntax()
        elif argv == ["test-numeric"]:
            test_numeric()
        elif argv == ["test-codecs"]:
            test_codecs()
        elif argv == ["test-ordered"]:
            test_ordered()
        elif argv == ["test-sequences"]:
            test_sequences()
        elif argv == ["test-arrays"]:
            test_arrays()
        elif argv == ["test-structural"]:
            test_structural()
        elif argv == ["test-initialization"]:
            test_initialization()
        elif argv == ["test-construction"]:
            test_construction()
        elif argv == ["test-types"]:
            test_types()
        elif argv == ["check"]:
            check_full(update=False)
        elif argv == ["update-inventory"]:
            check_full(update=True)
        elif argv == ["self-test"]:
            self_test()
        else:
            raise active.CSharpBuildFailure("CSHARP_PRACTICAL_BUILD_USAGE", 64)
        return 0
    except active.CSharpBuildFailure as error:
        sys.stderr.write(error.code + "\n")
        return error.exit_code
    except (OSError, KeyError, TypeError, ValueError):
        sys.stderr.write("CSHARP_PRACTICAL_BUILD_IO\n")
        return 74


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
