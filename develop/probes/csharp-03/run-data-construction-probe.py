#!/usr/bin/env python3
"""Build, run, normalize, and validate the private T01-W04 Roslyn probe."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import platform
import shutil
import stat
import sys
import tempfile
from pathlib import Path, PurePosixPath


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_ROOT = REPOSITORY_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_ROOT))

import csharp_build_inputs as active  # noqa: E402
import csharp_practical_build_inputs as practical  # noqa: E402


WORK_ITEM = "CSHARP-03-T01-W04"
FINAL_SCHEMA = "mpk.csharp_practical.t01_w04.roslyn_data_probe.v0"
RAW_SCHEMA = "mpk.csharp_practical.t01_w04.roslyn_data_probe.raw.v0"
PROBE_PATH = REPOSITORY_ROOT / "develop/probes/csharp-03/DataConstructionProbe.cs"
RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/roslyn-data-construction.json"
)
W03_DESCRIPTOR_PATH = practical.DESCRIPTOR_PATH
W03_INVENTORY_PATH = practical.INVENTORY_PATH
W03_COMMIT = "4ad2cd480792d8e7cac71eb798e6b55b66bd97fb"
W03_TREE = "3ab99588482bfb3666088fa88dede679c748c17c"
W03_DESCRIPTOR_SHA256 = practical.EXPECTED_DESCRIPTOR_RAW_SHA256
W03_INVENTORY_SHA256 = (
    "ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce"
)
PROBE_SOURCE_SHA256 = (
    "e49a96c63ef1dc8548d54b5ad5cb6dd81ebb90b56fa7a27d54adfcb99c1d4657"
)
PROBE_SOURCE_SIZE = 80_645
SHAPE_IDS_SHA256 = "727b7203815631d83cdb8475a2ce8360061205318763ed36a09fce76628a57b2"
ADMITTED_SHAPE_IDS_SHA256 = (
    "fe3a7b166ac51e184249debc491532b71fa30a9d1a5723cc830da67a8792ff6e"
)
REJECTED_SHAPE_IDS_SHA256 = (
    "506ba206622d81aa61b5ee8973958fc2c68a4155cf64d047e0daec4bcc9fd346"
)
EXPECTED_CASE_COUNT = 14
EXPECTED_SHAPE_COUNT = 181
EXPECTED_ADMITTED_SHAPE_COUNT = 129
EXPECTED_REJECTED_SHAPE_COUNT = 52
MAX_PROBE_SOURCE_BYTES = 2 * 1024 * 1024
MAX_PROBE_OUTPUT_BYTES = 64 * 1024 * 1024
COMPILER_ARGUMENTS = (
    "/nologo",
    "/noconfig",
    "/nostdlib+",
    "/deterministic+",
    "/optimize+",
    "/debug-",
    "/target:exe",
    "/platform:x64",
    "/langversion:14.0",
    "/nullable:enable",
    "/checked+",
    "/unsafe-",
    "/warnaserror+",
    "/utf8output",
    "/filealign:512",
    "/highentropyva+",
)

EXPECTED_COMPILER = {
    "architecture": "X64",
    "language": "C#",
    "language_version": "14.0",
    "nullable_context": "Enable",
    "reference_count": 167,
    "roslyn_common": {
        "culture": "neutral",
        "name": "Microsoft.CodeAnalysis",
        "public_key_token": "31bf3856ad364e35",
        "version": "5.6.0.0",
    },
    "roslyn_csharp": {
        "culture": "neutral",
        "name": "Microsoft.CodeAnalysis.CSharp",
        "public_key_token": "31bf3856ad364e35",
        "version": "5.6.0.0",
    },
    "runtime_version": "10.0.11",
}

REQUIRED_COVERAGE = {
    "arrays": ("array.", "intrinsic.array.", "near_miss.array."),
    "collection_calls": ("near_miss.collection.",),
    "compiler_owned_markers": ("compiler_marker.", "synthesized."),
    "constructors_and_initializers": (
        "constructor.",
        "default.",
        "init.",
        "object_initializer.",
        "required.",
        "near_miss.constructor.",
        "near_miss.object_initializer.",
        "near_miss.required.",
    ),
    "conversions": ("conversion.", "near_miss.conversion."),
    "data_intrinsics": ("intrinsic.",),
    "generic_metadata_boundary": ("near_miss.generic.", "near_miss.intrinsic."),
    "declarations": (
        "enum.",
        "field.",
        "property.",
        "readonly_struct.",
        "sealed_class.",
        "near_miss.struct.",
        "near_miss.class.",
        "near_miss.property.",
    ),
    "expression_bodies": ("expression_body.", "near_miss.expression_body."),
    "instance_calls_and_overloads": ("instance_method.", "overload_resolution."),
    "nullable": (
        "nullable.",
        "conditional_access.",
        "coalesce.",
        "near_miss.nullable.",
        "near_miss.conditional_access.",
        "near_miss.coalesce.",
    ),
    "ordinary_using_and_directives": ("using.", "near_miss.using."),
    "strings": ("string.", "near_miss.string."),
    "var": ("var.", "near_miss.var."),
}


class ProbeFailure(Exception):
    def __init__(self, code: str, detail: bytes | str | None = None):
        super().__init__(code)
        self.code = code
        self.detail = detail


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def strict_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ProbeFailure("DUPLICATE_JSON_KEY")
        result[key] = value
    return result


def strict_json(data: bytes) -> object:
    if len(data) > MAX_PROBE_OUTPUT_BYTES or data.startswith(b"\xef\xbb\xbf"):
        raise ProbeFailure("OUTPUT_TRANSPORT")
    try:
        text = data.decode("utf-8")
        return json.loads(
            text,
            object_pairs_hook=strict_pairs,
            parse_constant=lambda _: (_ for _ in ()).throw(
                ProbeFailure("NONFINITE_JSON_NUMBER")
            ),
        )
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as error:
        raise ProbeFailure("OUTPUT_JSON") from error


def exact_object(value: object, keys: set[str]) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != keys:
        raise ProbeFailure("SCHEMA_KEYS")
    return value


def array(value: object) -> list[object]:
    if not isinstance(value, list):
        raise ProbeFailure("SCHEMA_ARRAY")
    return value


def text(value: object) -> str:
    if not isinstance(value, str):
        raise ProbeFailure("SCHEMA_TEXT")
    return value


def integer(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ProbeFailure("SCHEMA_INTEGER")
    return value


def require_sha256(value: object) -> str:
    digest = text(value)
    if len(digest) != 64 or any(
        character not in "0123456789abcdef" for character in digest
    ):
        raise ProbeFailure("SCHEMA_SHA256")
    return digest


def read_regular(path: Path, maximum: int) -> bytes:
    try:
        info = path.lstat()
        if (
            not stat.S_ISREG(info.st_mode)
            or info.st_nlink != 1
            or info.st_size < 0
            or info.st_size > maximum
        ):
            raise ProbeFailure("INPUT_FILE")
        with path.open("rb") as stream:
            data = stream.read(maximum + 1)
    except OSError as error:
        raise ProbeFailure("INPUT_FILE") from error
    if len(data) != info.st_size or len(data) > maximum:
        raise ProbeFailure("INPUT_FILE")
    return data


def validate_frozen_inputs() -> dict[str, object]:
    probe_source = read_regular(PROBE_PATH, MAX_PROBE_SOURCE_BYTES)
    if len(probe_source) != PROBE_SOURCE_SIZE or sha256(probe_source) != PROBE_SOURCE_SHA256:
        raise ProbeFailure("PROBE_SOURCE_DRIFT")
    descriptor_bytes = read_regular(W03_DESCRIPTOR_PATH, active.MAX_JSON_BYTES)
    inventory_bytes = read_regular(W03_INVENTORY_PATH, active.MAX_JSON_BYTES)
    if sha256(descriptor_bytes) != W03_DESCRIPTOR_SHA256:
        raise ProbeFailure("W03_DESCRIPTOR_DRIFT")
    if sha256(inventory_bytes) != W03_INVENTORY_SHA256:
        raise ProbeFailure("W03_INVENTORY_DRIFT")
    descriptor = practical.load_descriptor()
    practical.load_inventory(descriptor)
    return descriptor


def copy_managed_assemblies(
    toolchain: dict[str, object], roots: dict[str, Path], destination: Path
) -> None:
    package_roots = {
        "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
        "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
    }
    for untyped in active.array(toolchain["managed_projection"]):
        record = active.exact_keys(
            untyped,
            {"archive_path", "package_id", "runtime_path", "sha256", "size_bytes"},
        )
        source = package_roots[active.text(record["package_id"])] / active.text(
            record["archive_path"]
        )
        name = PurePosixPath(active.text(record["runtime_path"])).name
        target = destination / name
        shutil.copyfile(source, target)
        os.chmod(target, 0o444)


def run_once(
    descriptor: dict[str, object], roots: dict[str, Path], root: Path
) -> tuple[bytes, str]:
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    output = root / "output"
    output.mkdir(mode=0o700)
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    sdk = roots["dotnet-sdk-linux-x64"]
    runtime = roots["dotnet-runtime-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    probe_binary = output / "DataConstructionProbe.dll"
    reference_root = roots["microsoft-netcore-app-ref"] / "ref/net10.0"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(probe_binary),
            "/main:DataConstructionProbe",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    for untyped in active.array(toolchain["reference_projection"]["inventory"]):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        arguments.append(
            "/reference:"
            + str(roots["microsoft-netcore-app-ref"] / active.text(record["path"]))
        )
    package_roots = {
        "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
        "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
    }
    for untyped in active.array(toolchain["managed_projection"]):
        record = active.exact_keys(
            untyped,
            {"archive_path", "package_id", "runtime_path", "sha256", "size_bytes"},
        )
        arguments.append(
            "/reference:"
            + str(
                package_roots[active.text(record["package_id"])]
                / active.text(record["archive_path"])
            )
        )
    arguments.append(str(PROBE_PATH))
    build_environment = active.closed_dotnet_environment(sdk, root / "build-environment")
    compiled = active.execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler), *arguments],
        cwd=root,
        environment=build_environment,
    )
    if compiled.returncode != 0 or compiled.stdout or compiled.stderr or not probe_binary.is_file():
        raise ProbeFailure("COMPILER", compiled.stderr or compiled.stdout)
    unexpected = sorted(path.name for path in output.iterdir())
    if unexpected != ["DataConstructionProbe.dll"]:
        raise ProbeFailure("COMPILER_OUTPUT_SET")
    binary = read_regular(probe_binary, active.MAX_EXTRACTED_BYTES)
    binary_sha256 = sha256(binary)
    copy_managed_assemblies(toolchain, roots, output)

    runtime_environment = active.closed_dotnet_environment(
        runtime, root / "runtime-environment"
    )
    executed = active.execute_isolated(
        [
            str(runtime / "dotnet"),
            "exec",
            "--runtimeconfig",
            str(REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json"),
            "--fx-version",
            "10.0.11",
            str(probe_binary),
            str(reference_root),
        ],
        cwd=output,
        environment=runtime_environment,
    )
    if executed.returncode != 0 or executed.stderr or not executed.stdout:
        raise ProbeFailure("EXECUTION", executed.stderr or executed.stdout)
    if len(executed.stdout) > MAX_PROBE_OUTPUT_BYTES:
        raise ProbeFailure("OUTPUT_SIZE")
    return executed.stdout, binary_sha256


def expected_coverage(shape_ids: list[str]) -> list[object]:
    rows: list[object] = []
    covered: set[str] = set()
    for requirement, prefixes in sorted(REQUIRED_COVERAGE.items()):
        matching = sorted(
            shape_id
            for shape_id in shape_ids
            if any(shape_id.startswith(prefix) for prefix in prefixes)
        )
        if not matching:
            raise ProbeFailure("COVERAGE_EMPTY")
        covered.update(matching)
        rows.append({"requirement": requirement, "shape_ids": matching})
    if covered != set(shape_ids):
        raise ProbeFailure("COVERAGE_UNOWNED")
    return rows


def normalize(raw_bytes: bytes, binary_sha256: str) -> dict[str, object]:
    raw = exact_object(
        strict_json(raw_bytes), {"cases", "compiler", "schema", "work_item"}
    )
    if raw["schema"] != RAW_SCHEMA or raw["work_item"] != WORK_ITEM:
        raise ProbeFailure("RAW_IDENTITY")
    if raw["compiler"] != EXPECTED_COMPILER:
        raise ProbeFailure("COMPILER_IDENTITY")
    raw_cases = array(raw["cases"])
    shape_index: list[dict[str, object]] = []
    for case_value in raw_cases:
        case = exact_object(
            case_value,
            {
                "compiler_outcome",
                "control_flow_graphs",
                "diagnostics",
                "disposition",
                "emitted_metadata",
                "id",
                "operation_roots",
                "semantic_nodes",
                "source",
                "source_types",
                "source_utf8_sha256",
                "syntax",
                "targets",
            },
        )
        case_id = text(case["id"])
        disposition = text(case["disposition"])
        if disposition not in {"admitted_shape", "rejected_near_miss"}:
            raise ProbeFailure("CASE_DISPOSITION")
        if disposition == "admitted_shape" and case["compiler_outcome"] != "success":
            raise ProbeFailure("ADMITTED_COMPILER_OUTCOME")
        source = text(case["source"])
        if "Mpk." in source or "MPK" in source:
            raise ProbeFailure("SOURCE_DEPENDENCY")
        if require_sha256(case["source_utf8_sha256"]) != sha256(source.encode("utf-8")):
            raise ProbeFailure("SOURCE_HASH")
        if not array(case["syntax"]) or not array(case["semantic_nodes"]):
            raise ProbeFailure("CASE_OBSERVATIONS")
        for target_value in array(case["targets"]):
            target = exact_object(
                target_value,
                {
                    "candidate_reason",
                    "candidate_symbols",
                    "conversion",
                    "converted_type",
                    "declared_symbol",
                    "emitted_type",
                    "enclosing_flow_root",
                    "marker_span",
                    "operation",
                    "related_type_members",
                    "shape_id",
                    "symbol",
                    "syntax",
                    "type",
                },
            )
            shape_id = text(target["shape_id"])
            if disposition == "admitted_shape" and shape_id.startswith("near_miss."):
                raise ProbeFailure("SHAPE_DISPOSITION")
            if disposition == "rejected_near_miss" and not shape_id.startswith(
                "near_miss."
            ):
                raise ProbeFailure("SHAPE_DISPOSITION")
            shape_index.append(
                {
                    "case_id": case_id,
                    "disposition": disposition,
                    "observation_sha256": sha256(canonical(target)),
                    "shape_id": shape_id,
                    "upgrade_mutation_id": (
                        "CSHARP-03-T01-W04-UPGRADE-"
                        + shape_id.upper().replace(".", "_").replace("-", "_")
                        if disposition == "admitted_shape"
                        else None
                    ),
                }
            )
    shape_index.sort(key=lambda row: text(row["shape_id"]))
    shape_ids = [text(row["shape_id"]) for row in shape_index]
    admitted = [
        text(row["shape_id"])
        for row in shape_index
        if row["disposition"] == "admitted_shape"
    ]
    rejected = [
        text(row["shape_id"])
        for row in shape_index
        if row["disposition"] == "rejected_near_miss"
    ]
    if (
        len(raw_cases) != EXPECTED_CASE_COUNT
        or len(shape_ids) != EXPECTED_SHAPE_COUNT
        or len(admitted) != EXPECTED_ADMITTED_SHAPE_COUNT
        or len(rejected) != EXPECTED_REJECTED_SHAPE_COUNT
        or len(set(shape_ids)) != len(shape_ids)
        or sha256(canonical(sorted(shape_ids))) != SHAPE_IDS_SHA256
        or sha256(canonical(sorted(admitted))) != ADMITTED_SHAPE_IDS_SHA256
        or sha256(canonical(sorted(rejected))) != REJECTED_SHAPE_IDS_SHA256
    ):
        raise ProbeFailure("SHAPE_CATALOG")
    mutation_ids = [
        text(row["upgrade_mutation_id"])
        for row in shape_index
        if row["upgrade_mutation_id"] is not None
    ]
    if len(mutation_ids) != len(set(mutation_ids)) or len(mutation_ids) != len(admitted):
        raise ProbeFailure("UPGRADE_MUTATIONS")

    canonical_raw = canonical(raw) + b"\n"
    return {
        "baseline": {
            "build_inputs": W03_DESCRIPTOR_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": W03_INVENTORY_PATH.relative_to(
                REPOSITORY_ROOT
            ).as_posix(),
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "source_commit": W03_COMMIT,
            "source_tree": W03_TREE,
        },
        "coverage": expected_coverage(shape_ids),
        "measurement": {
            "probe_binary_sha256": binary_sha256,
            "raw_observation_sha256": sha256(canonical_raw),
            "raw_observation_size_bytes": len(canonical_raw),
        },
        "observations": raw,
        "probe_input": {
            "compiler_arguments": list(COMPILER_ARGUMENTS),
            "path": PROBE_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "raw_sha256": PROBE_SOURCE_SHA256,
            "size_bytes": PROBE_SOURCE_SIZE,
            "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
            "reference_projection_sha256": active.REFERENCE_HASH,
        },
        "schema": FINAL_SCHEMA,
        "shape_index": shape_index,
        "work_item": WORK_ITEM,
    }


def target_by_shape(document: dict[str, object], shape_id: str) -> dict[str, object]:
    observations = exact_object(
        document["observations"], {"cases", "compiler", "schema", "work_item"}
    )
    for case_value in array(observations["cases"]):
        case = exact_object(
            case_value,
            {
                "compiler_outcome",
                "control_flow_graphs",
                "diagnostics",
                "disposition",
                "emitted_metadata",
                "id",
                "operation_roots",
                "semantic_nodes",
                "source",
                "source_types",
                "source_utf8_sha256",
                "syntax",
                "targets",
            },
        )
        for target_value in array(case["targets"]):
            target = exact_object(
                target_value,
                {
                    "candidate_reason",
                    "candidate_symbols",
                    "conversion",
                    "converted_type",
                    "declared_symbol",
                    "emitted_type",
                    "enclosing_flow_root",
                    "marker_span",
                    "operation",
                    "related_type_members",
                    "shape_id",
                    "symbol",
                    "syntax",
                    "type",
                },
            )
            if target["shape_id"] == shape_id:
                return target
    raise ProbeFailure("SHAPE_TARGET_MISSING")


def validate_document(value: object, *, check_live_inputs: bool) -> dict[str, object]:
    document = exact_object(
        value,
        {
            "baseline",
            "coverage",
            "measurement",
            "observations",
            "probe_input",
            "schema",
            "shape_index",
            "work_item",
        },
    )
    if document["schema"] != FINAL_SCHEMA or document["work_item"] != WORK_ITEM:
        raise ProbeFailure("DOCUMENT_IDENTITY")
    expected_baseline = {
        "build_inputs": "develop/migrations/csharp-03/build-inputs/build-inputs.json",
        "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
        "candidate_inventory": "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
        "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
        "source_commit": W03_COMMIT,
        "source_tree": W03_TREE,
    }
    if document["baseline"] != expected_baseline:
        raise ProbeFailure("BASELINE")
    probe_input = exact_object(
        document["probe_input"],
        {
            "compiler_arguments",
            "path",
            "raw_sha256",
            "reference_projection_sha256",
            "size_bytes",
            "toolchain_inputs_sha256",
        },
    )
    if probe_input != {
        "compiler_arguments": list(COMPILER_ARGUMENTS),
        "path": "develop/probes/csharp-03/DataConstructionProbe.cs",
        "raw_sha256": PROBE_SOURCE_SHA256,
        "reference_projection_sha256": active.REFERENCE_HASH,
        "size_bytes": PROBE_SOURCE_SIZE,
        "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
    }:
        raise ProbeFailure("PROBE_INPUT")
    measurement = exact_object(
        document["measurement"],
        {
            "probe_binary_sha256",
            "raw_observation_sha256",
            "raw_observation_size_bytes",
        },
    )
    require_sha256(measurement["probe_binary_sha256"])
    require_sha256(measurement["raw_observation_sha256"])
    integer(measurement["raw_observation_size_bytes"])
    observations = exact_object(
        document["observations"], {"cases", "compiler", "schema", "work_item"}
    )
    if observations["schema"] != RAW_SCHEMA or observations["work_item"] != WORK_ITEM:
        raise ProbeFailure("RAW_IDENTITY")
    if observations["compiler"] != EXPECTED_COMPILER:
        raise ProbeFailure("COMPILER_IDENTITY")
    raw_bytes = canonical(observations) + b"\n"
    if (
        sha256(raw_bytes) != measurement["raw_observation_sha256"]
        or len(raw_bytes) != measurement["raw_observation_size_bytes"]
    ):
        raise ProbeFailure("RAW_MEASUREMENT")

    shape_index = array(document["shape_index"])
    shape_ids: list[str] = []
    admitted: list[str] = []
    rejected: list[str] = []
    mutation_ids: list[str] = []
    for value_index in shape_index:
        index = exact_object(
            value_index,
            {
                "case_id",
                "disposition",
                "observation_sha256",
                "shape_id",
                "upgrade_mutation_id",
            },
        )
        case_id = text(index["case_id"])
        shape_id = text(index["shape_id"])
        disposition = text(index["disposition"])
        target = target_by_shape(document, shape_id)
        if require_sha256(index["observation_sha256"]) != sha256(canonical(target)):
            raise ProbeFailure("OBSERVATION_HASH")
        matching_case = next(
            (
                case
                for case in array(observations["cases"])
                if isinstance(case, dict) and case.get("id") == case_id
            ),
            None,
        )
        if not isinstance(matching_case, dict) or matching_case.get("disposition") != disposition:
            raise ProbeFailure("SHAPE_CASE_LINK")
        shape_ids.append(shape_id)
        if disposition == "admitted_shape":
            admitted.append(shape_id)
            mutation_ids.append(text(index["upgrade_mutation_id"]))
        elif disposition == "rejected_near_miss":
            rejected.append(shape_id)
            if index["upgrade_mutation_id"] is not None:
                raise ProbeFailure("REJECTED_MUTATION")
        else:
            raise ProbeFailure("SHAPE_DISPOSITION")
    if (
        shape_ids != sorted(shape_ids)
        or len(shape_ids) != EXPECTED_SHAPE_COUNT
        or len(admitted) != EXPECTED_ADMITTED_SHAPE_COUNT
        or len(rejected) != EXPECTED_REJECTED_SHAPE_COUNT
        or len(set(shape_ids)) != len(shape_ids)
        or len(set(mutation_ids)) != len(mutation_ids)
        or sha256(canonical(shape_ids)) != SHAPE_IDS_SHA256
        or sha256(canonical(sorted(admitted))) != ADMITTED_SHAPE_IDS_SHA256
        or sha256(canonical(sorted(rejected))) != REJECTED_SHAPE_IDS_SHA256
    ):
        raise ProbeFailure("SHAPE_CATALOG")
    if document["coverage"] != expected_coverage(shape_ids):
        raise ProbeFailure("COVERAGE")
    if len(array(observations["cases"])) != EXPECTED_CASE_COUNT:
        raise ProbeFailure("CASE_COUNT")
    if check_live_inputs:
        validate_frozen_inputs()
    return document


def load_result(*, check_live_inputs: bool) -> dict[str, object]:
    data = read_regular(RESULT_PATH, MAX_PROBE_OUTPUT_BYTES)
    value = strict_json(data)
    if data != canonical(value) + b"\n":
        raise ProbeFailure("CANONICAL_TRANSPORT")
    return validate_document(value, check_live_inputs=check_live_inputs)


def run_twice() -> bytes:
    if platform.system() != "Linux" or platform.machine() not in {"x86_64", "amd64"}:
        raise ProbeFailure("LINUX_X64_REQUIRED")
    descriptor = validate_frozen_inputs()
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    archives = practical.checked_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-t01-w04-") as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        first_raw, first_binary = run_once(
            descriptor, roots, temporary_root / "first"
        )
        second_raw, second_binary = run_once(
            descriptor, roots, temporary_root / "second"
        )
        if first_raw != second_raw or first_binary != second_binary:
            raise ProbeFailure("NONDETERMINISTIC_RERUN")
        first = normalize(first_raw, first_binary)
        second = normalize(second_raw, second_binary)
        first_bytes = canonical(first) + b"\n"
        if first_bytes != canonical(second) + b"\n":
            raise ProbeFailure("NONDETERMINISTIC_NORMALIZATION")
        validate_document(first, check_live_inputs=False)
        return first_bytes


def write_result(data: bytes) -> None:
    RESULT_PATH.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(
        prefix=".roslyn-data-construction-", dir=RESULT_PATH.parent
    )
    temporary_path = Path(temporary)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.chmod(temporary_path, 0o644)
        os.replace(temporary_path, RESULT_PATH)
    finally:
        temporary_path.unlink(missing_ok=True)


def self_test() -> None:
    document = load_result(check_live_inputs=True)
    admitted_rows = [
        row
        for row in array(document["shape_index"])
        if isinstance(row, dict) and row.get("disposition") == "admitted_shape"
    ]
    if len(admitted_rows) != EXPECTED_ADMITTED_SHAPE_COUNT:
        raise ProbeFailure("SELF_TEST_CATALOG")
    for row in admitted_rows:
        shape_id = text(row["shape_id"])
        target = target_by_shape(document, shape_id)
        syntax = exact_object(
            target["syntax"],
            {
                "contains_diagnostics",
                "full_span",
                "is_missing",
                "kind",
                "raw_kind",
                "span",
            },
        )
        original = integer(syntax["raw_kind"])
        syntax["raw_kind"] = original + 1
        try:
            expected = require_sha256(row["observation_sha256"])
            if sha256(canonical(target)) == expected:
                raise ProbeFailure("SELF_TEST_MUTATION_ACCEPTED")
        finally:
            syntax["raw_kind"] = original
    changed = copy.deepcopy(document)
    first_shape = text(array(changed["shape_index"])[0]["shape_id"])
    changed_target = target_by_shape(changed, first_shape)
    changed_target["candidate_reason"] = "ChangedObservation"
    try:
        validate_document(changed, check_live_inputs=False)
    except ProbeFailure:
        pass
    else:
        raise ProbeFailure("SELF_TEST_SCHEMA_ACCEPTED")


def main(argv: list[str]) -> int:
    try:
        if argv == ["check-record"]:
            load_result(check_live_inputs=True)
        elif argv == ["self-test"]:
            self_test()
        elif argv == ["check"]:
            expected = read_regular(RESULT_PATH, MAX_PROBE_OUTPUT_BYTES)
            observed = run_twice()
            if observed != expected:
                raise ProbeFailure("RECORDED_RESULT_DRIFT")
        elif argv == ["update"]:
            write_result(run_twice())
        else:
            raise ProbeFailure("USAGE")
        return 0
    except ProbeFailure as failure:
        if failure.detail:
            detail = (
                failure.detail
                if isinstance(failure.detail, bytes)
                else failure.detail.encode("utf-8", "replace")
            )
            sys.stderr.buffer.write(detail[: 256 * 1024])
            if not detail.endswith(b"\n"):
                sys.stderr.buffer.write(b"\n")
        print(failure.code, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
