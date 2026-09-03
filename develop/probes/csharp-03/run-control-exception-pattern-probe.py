#!/usr/bin/env python3
"""Build, run, normalize, and validate the private T01-W05 Roslyn probe."""

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


WORK_ITEM = "CSHARP-03-T01-W05"
FINAL_SCHEMA = "mpk.csharp_practical.t01_w05.roslyn_control_probe.v0"
RAW_SCHEMA = "mpk.csharp_practical.t01_w05.roslyn_control_probe.raw.v0"
PROBE_PATH = REPOSITORY_ROOT / "develop/probes/csharp-03/ControlExceptionPatternProbe.cs"
RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json"
)
W04_RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/roslyn-data-construction.json"
)
W04_SOURCE_PATH = REPOSITORY_ROOT / "develop/probes/csharp-03/DataConstructionProbe.cs"
W03_DESCRIPTOR_PATH = practical.DESCRIPTOR_PATH
W03_INVENTORY_PATH = practical.INVENTORY_PATH
W04_COMMIT = "b6680168c2666be503741575c009f0a26dd0da22"
W04_TREE = "0f1e86bbdf986870b60fe335da58290baac26b0f"
W04_RESULT_SHA256 = "c5de8bc209331c2295497210a570ba0be32e0871b3dd2576980d6c109222142e"
W04_RESULT_SIZE = 5_925_271
W04_SOURCE_SHA256 = "e49a96c63ef1dc8548d54b5ad5cb6dd81ebb90b56fa7a27d54adfcb99c1d4657"
W03_DESCRIPTOR_SHA256 = "83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015"
W03_INVENTORY_SHA256 = "ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce"
PROBE_SOURCE_SHA256 = "f62ff3deb7c0fff2799f99426ab9dbd7e6fd373a5fd9d8ed91bbb118a9808f1f"
PROBE_SOURCE_SIZE = 70_299
SHAPE_IDS_SHA256 = "431e5891260b9e3284f6b3646ae25d4643d9d53c8fdede0db69a1d2fd5d2d501"
ADMITTED_SHAPE_IDS_SHA256 = (
    "524e05d67fa72c5520176711f06a42739f44d881ad30e2c0b31cfbc83f76864c"
)
REJECTED_SHAPE_IDS_SHA256 = (
    "b510c715a9d915a1217bccfbcc80611877c06a5bd8cf036bd1959db7012e0870"
)
EXPECTED_CASE_COUNT = 18
EXPECTED_SHAPE_COUNT = 103
EXPECTED_ADMITTED_SHAPE_COUNT = 62
EXPECTED_REJECTED_SHAPE_COUNT = 41
EXPECTED_OBSERVATION_COUNTS = {
    ("decision_graph", "admitted_shape"): 22,
    ("decision_graph", "rejected_near_miss"): 18,
    ("exception_region", "admitted_shape"): 11,
    ("exception_region", "rejected_near_miss"): 14,
}
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
    "abrupt_completion": (
        "abrupt.",
        "near_miss.exception.finally_",
        "near_miss.exception.rethrow_",
    ),
    "catch_and_throw": (
        "exception.catch.",
        "exception.propagation.",
        "exception.source.",
        "exception.throw.",
        "near_miss.exception.builtin_",
        "near_miss.exception.catch_",
        "near_miss.exception.rethrow_",
        "near_miss.exception.source_",
        "near_miss.exception.throw_",
    ),
    "filters_finally_and_regions": (
        "exception.filter.",
        "exception.finally.",
        "exception.search.",
        "exception.unwind.",
        "near_miss.exception.filter_",
        "near_miss.exception.finally_",
    ),
    "guards": (
        "pattern.guard.",
        "exception.filter.",
        "near_miss.exception.filter_",
        "near_miss.pattern.guard_",
    ),
    "loops_and_structured_branches": (
        "loop.",
        "abrupt.break.",
        "abrupt.continue.",
        "near_miss.loop.",
    ),
    "patterns": ("pattern.", "near_miss.pattern."),
    "switch_statements_and_expressions": ("switch.", "near_miss.switch."),
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
        text_value = data.decode("utf-8")
        return json.loads(
            text_value,
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
    checks = (
        (PROBE_PATH, MAX_PROBE_SOURCE_BYTES, PROBE_SOURCE_SIZE, PROBE_SOURCE_SHA256),
        (W04_RESULT_PATH, MAX_PROBE_OUTPUT_BYTES, W04_RESULT_SIZE, W04_RESULT_SHA256),
        (W04_SOURCE_PATH, MAX_PROBE_SOURCE_BYTES, 80_645, W04_SOURCE_SHA256),
    )
    for path, maximum, expected_size, expected_hash in checks:
        data = read_regular(path, maximum)
        if len(data) != expected_size or sha256(data) != expected_hash:
            raise ProbeFailure("FROZEN_INPUT_DRIFT")
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
        target = destination / PurePosixPath(active.text(record["runtime_path"])).name
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
    probe_binary = output / "ControlExceptionPatternProbe.dll"
    reference_root = roots["microsoft-netcore-app-ref"] / "ref/net10.0"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(probe_binary),
            "/main:ControlExceptionPatternProbe",
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
    if unexpected != ["ControlExceptionPatternProbe.dll"]:
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


CASE_KEYS = {
    "abrupt_completions",
    "compiler_outcome",
    "control_flow_graphs",
    "decision_graphs",
    "diagnostics",
    "disposition",
    "exception_regions",
    "id",
    "operation_roots",
    "source",
    "source_order",
    "source_utf8_sha256",
    "syntax",
    "targets",
}
TARGET_KEYS = {
    "candidate_reason",
    "candidate_symbols",
    "marker_span",
    "operation",
    "shape_id",
    "source_ordinal",
    "symbol",
    "syntax",
    "type",
}
DIAGNOSTIC_KEYS = {
    "id",
    "is_suppressed",
    "location_kind",
    "severity",
    "span",
    "warning_level",
}


def compiler_outcome_class(case: dict[str, object], disposition: str) -> str:
    outcome = text(case["compiler_outcome"])
    diagnostics = array(case["diagnostics"])
    severities = [
        text(exact_object(value, DIAGNOSTIC_KEYS)["severity"])
        for value in diagnostics
    ]
    expected_outcome = "error" if "Error" in severities else "success"
    if outcome != expected_outcome:
        raise ProbeFailure("COMPILER_OUTCOME")
    if disposition == "admitted_shape":
        if outcome != "success" or diagnostics:
            raise ProbeFailure("ADMITTED_COMPILER_OUTCOME")
        return "admitted"
    if disposition != "rejected_near_miss":
        raise ProbeFailure("CASE_DISPOSITION")
    if outcome == "error":
        return "error"
    return "warning" if diagnostics else "clean"


def cfg_eligible_root_count(case: dict[str, object]) -> int:
    count = 0
    for value in array(case["operation_roots"]):
        if not isinstance(value, dict):
            raise ProbeFailure("OPERATION_ROOT")
        if text(value.get("kind")) in {"ConstructorBodyOperation", "MethodBodyOperation"}:
            count += 1
    return count


def upgrade_mutation_field(family: str, observation: dict[str, object]) -> str:
    if family == "decision_graph":
        nodes = array(observation["nodes"])
        if not nodes or not isinstance(nodes[0], dict):
            raise ProbeFailure("UPGRADE_MUTATION_FIELD")
        return "nodes[0].operation_kind"
    if family != "exception_region":
        raise ProbeFailure("UPGRADE_MUTATION_FIELD")
    catches = array(observation["catches"])
    if catches:
        if not array(observation["handler_search_order"]):
            raise ProbeFailure("UPGRADE_MUTATION_FIELD")
        return "handler_search_order[0]"
    return "nesting_depth"


def apply_upgrade_mutation(
    observation: dict[str, object], mutation_field: str
) -> None:
    if mutation_field == "nodes[0].operation_kind":
        node = array(observation["nodes"])[0]
        if not isinstance(node, dict):
            raise ProbeFailure("UPGRADE_MUTATION_FIELD")
        node["operation_kind"] = text(node["operation_kind"]) + "#upgrade-mutation"
    elif mutation_field == "handler_search_order[0]":
        search = array(observation["handler_search_order"])
        search[0] = integer(search[0]) + 1
    elif mutation_field == "nesting_depth":
        observation["nesting_depth"] = integer(observation["nesting_depth"]) + 1
    else:
        raise ProbeFailure("UPGRADE_MUTATION_FIELD")


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
    upgrade_mutations: list[dict[str, object]] = []
    observation_ids: set[str] = set()
    case_ids: set[str] = set()
    rejected_outcome_classes: set[str] = set()
    for case_value in raw_cases:
        case = exact_object(case_value, CASE_KEYS)
        case_id = text(case["id"])
        if not case_id or case_id in case_ids:
            raise ProbeFailure("CASE_ID")
        case_ids.add(case_id)
        disposition = text(case["disposition"])
        if disposition not in {"admitted_shape", "rejected_near_miss"}:
            raise ProbeFailure("CASE_DISPOSITION")
        outcome_class = compiler_outcome_class(case, disposition)
        if disposition == "rejected_near_miss":
            rejected_outcome_classes.add(outcome_class)
        source = text(case["source"])
        if "Mpk." in source or "MPK" in source:
            raise ProbeFailure("SOURCE_DEPENDENCY")
        if require_sha256(case["source_utf8_sha256"]) != sha256(source.encode("utf-8")):
            raise ProbeFailure("SOURCE_HASH")
        if not array(case["syntax"]) or not array(case["operation_roots"]):
            raise ProbeFailure("CASE_OBSERVATIONS")
        if case["compiler_outcome"] == "success" and len(
            array(case["control_flow_graphs"])
        ) != cfg_eligible_root_count(case):
            raise ProbeFailure("SUCCESS_CFG_CLOSURE")
        if not (
            array(case["decision_graphs"])
            or array(case["exception_regions"])
            or array(case["abrupt_completions"])
        ):
            raise ProbeFailure("CONTROL_OBSERVATION")
        source_order = array(case["source_order"])
        starts = []
        for ordinal, value in enumerate(source_order):
            record = exact_object(value, {"category", "id", "source_ordinal", "start"})
            if integer(record["source_ordinal"]) != ordinal:
                raise ProbeFailure("SOURCE_ORDER_ORDINAL")
            starts.append(integer(record["start"]))
        if starts != sorted(starts):
            raise ProbeFailure("SOURCE_ORDER")
        for target_value in array(case["targets"]):
            target = exact_object(target_value, TARGET_KEYS)
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
                    "source_ordinal": integer(target["source_ordinal"]),
                }
            )
        for family, key in (
            ("decision_graph", "decision_graphs"),
            ("exception_region", "exception_regions"),
        ):
            for observation_value in array(case[key]):
                observation = exact_object(
                    observation_value,
                    {
                        "edges",
                        "id",
                        "nodes",
                        "root",
                        "source_ordinal",
                    }
                    if family == "decision_graph"
                    else {
                        "body",
                        "catches",
                        "finally",
                        "handler_search_order",
                        "id",
                        "nesting_depth",
                        "source_ordinal",
                        "span",
                        "throws",
                    },
                )
                observation_id = text(observation["id"])
                if observation_id in observation_ids:
                    raise ProbeFailure("OBSERVATION_ID")
                observation_ids.add(observation_id)
                mutation_field = upgrade_mutation_field(family, observation)
                upgrade_mutations.append(
                    {
                        "case_id": case_id,
                        "disposition": disposition,
                        "family": family,
                        "mutation_id": (
                            "CSHARP-03-T01-W05-UPGRADE-"
                            + family.upper()
                            + "-"
                            + observation_id.upper().replace("#", "-").replace(".", "-")
                        ),
                        "mutation_field": mutation_field,
                        "observation_id": observation_id,
                        "observation_sha256": sha256(canonical(observation)),
                    }
                )
    shape_index.sort(key=lambda row: text(row["shape_id"]))
    upgrade_mutations.sort(key=lambda row: text(row["mutation_id"]))
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
    if rejected_outcome_classes != {"clean", "error", "warning"}:
        raise ProbeFailure("REJECTED_COMPILER_OUTCOMES")
    if (
        len(raw_cases) != EXPECTED_CASE_COUNT
        or len(shape_ids) != EXPECTED_SHAPE_COUNT
        or len(admitted) != EXPECTED_ADMITTED_SHAPE_COUNT
        or len(rejected) != EXPECTED_REJECTED_SHAPE_COUNT
        or len(set(shape_ids)) != len(shape_ids)
        or sha256(canonical(shape_ids)) != SHAPE_IDS_SHA256
        or sha256(canonical(admitted)) != ADMITTED_SHAPE_IDS_SHA256
        or sha256(canonical(rejected)) != REJECTED_SHAPE_IDS_SHA256
    ):
        raise ProbeFailure(
            "SHAPE_CATALOG",
            canonical(
                {
                    "actual": {
                        "admitted_count": len(admitted),
                        "admitted_sha256": sha256(canonical(admitted)),
                        "case_count": len(raw_cases),
                        "rejected_count": len(rejected),
                        "rejected_sha256": sha256(canonical(rejected)),
                        "shape_count": len(shape_ids),
                        "shape_sha256": sha256(canonical(shape_ids)),
                    },
                    "expected": {
                        "admitted_count": EXPECTED_ADMITTED_SHAPE_COUNT,
                        "admitted_sha256": ADMITTED_SHAPE_IDS_SHA256,
                        "case_count": EXPECTED_CASE_COUNT,
                        "rejected_count": EXPECTED_REJECTED_SHAPE_COUNT,
                        "rejected_sha256": REJECTED_SHAPE_IDS_SHA256,
                        "shape_count": EXPECTED_SHAPE_COUNT,
                        "shape_sha256": SHAPE_IDS_SHA256,
                    },
                }
            ),
        )
    mutation_ids = [text(row["mutation_id"]) for row in upgrade_mutations]
    families = {text(row["family"]) for row in upgrade_mutations}
    dispositions = {text(row["disposition"]) for row in upgrade_mutations}
    mutation_fields = {text(row["mutation_field"]) for row in upgrade_mutations}
    observation_counts: dict[tuple[str, str], int] = {}
    for row in upgrade_mutations:
        key = (text(row["family"]), text(row["disposition"]))
        observation_counts[key] = observation_counts.get(key, 0) + 1
    if (
        not upgrade_mutations
        or len(mutation_ids) != len(set(mutation_ids))
        or families != {"decision_graph", "exception_region"}
        or dispositions != {"admitted_shape", "rejected_near_miss"}
        or observation_counts != EXPECTED_OBSERVATION_COUNTS
        or mutation_fields
        != {
            "handler_search_order[0]",
            "nesting_depth",
            "nodes[0].operation_kind",
        }
    ):
        raise ProbeFailure("UPGRADE_MUTATIONS")
    canonical_raw = canonical(raw) + b"\n"
    return {
        "baseline": {
            "build_inputs": W03_DESCRIPTOR_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": W03_INVENTORY_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "data_probe": W04_RESULT_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "data_probe_raw_sha256": W04_RESULT_SHA256,
            "source_commit": W04_COMMIT,
            "source_tree": W04_TREE,
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
            "reference_projection_sha256": active.REFERENCE_HASH,
            "size_bytes": PROBE_SOURCE_SIZE,
            "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
        },
        "schema": FINAL_SCHEMA,
        "shape_index": shape_index,
        "upgrade_mutations": upgrade_mutations,
        "work_item": WORK_ITEM,
    }


def target_by_shape(document: dict[str, object], shape_id: str) -> dict[str, object]:
    for case_value in array(document["observations"]["cases"]):
        case = exact_object(case_value, CASE_KEYS)
        for target_value in array(case["targets"]):
            target = exact_object(target_value, TARGET_KEYS)
            if target["shape_id"] == shape_id:
                return target
    raise ProbeFailure("SHAPE_TARGET_MISSING")


def observation_by_id(
    document: dict[str, object], family: str, observation_id: str
) -> dict[str, object]:
    key = "decision_graphs" if family == "decision_graph" else "exception_regions"
    for case_value in array(document["observations"]["cases"]):
        case = exact_object(case_value, CASE_KEYS)
        for value in array(case[key]):
            if isinstance(value, dict) and value.get("id") == observation_id:
                return value
    raise ProbeFailure("UPGRADE_OBSERVATION_MISSING")


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
            "upgrade_mutations",
            "work_item",
        },
    )
    if document["schema"] != FINAL_SCHEMA or document["work_item"] != WORK_ITEM:
        raise ProbeFailure("DOCUMENT_IDENTITY")
    if document["baseline"] != {
        "build_inputs": "develop/migrations/csharp-03/build-inputs/build-inputs.json",
        "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
        "candidate_inventory": "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
        "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
        "data_probe": "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
        "data_probe_raw_sha256": W04_RESULT_SHA256,
        "source_commit": W04_COMMIT,
        "source_tree": W04_TREE,
    }:
        raise ProbeFailure("BASELINE")
    if document["probe_input"] != {
        "compiler_arguments": list(COMPILER_ARGUMENTS),
        "path": "develop/probes/csharp-03/ControlExceptionPatternProbe.cs",
        "raw_sha256": PROBE_SOURCE_SHA256,
        "reference_projection_sha256": active.REFERENCE_HASH,
        "size_bytes": PROBE_SOURCE_SIZE,
        "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
    }:
        raise ProbeFailure("PROBE_INPUT")
    observations = exact_object(
        document["observations"], {"cases", "compiler", "schema", "work_item"}
    )
    if observations["schema"] != RAW_SCHEMA or observations["work_item"] != WORK_ITEM:
        raise ProbeFailure("RAW_IDENTITY")
    if observations["compiler"] != EXPECTED_COMPILER:
        raise ProbeFailure("COMPILER_IDENTITY")
    measurement = exact_object(
        document["measurement"],
        {
            "probe_binary_sha256",
            "raw_observation_sha256",
            "raw_observation_size_bytes",
        },
    )
    require_sha256(measurement["probe_binary_sha256"])
    raw_bytes = canonical(observations) + b"\n"
    if (
        require_sha256(measurement["raw_observation_sha256"]) != sha256(raw_bytes)
        or integer(measurement["raw_observation_size_bytes"]) != len(raw_bytes)
    ):
        raise ProbeFailure("RAW_MEASUREMENT")

    cases = array(observations["cases"])
    case_links: dict[str, str] = {}
    actual_targets: dict[str, tuple[str, str, int, str]] = {}
    actual_observations: dict[str, tuple[str, str, str, str]] = {}
    rejected_outcome_classes: set[str] = set()
    for case_value in cases:
        case = exact_object(case_value, CASE_KEYS)
        case_id = text(case["id"])
        disposition = text(case["disposition"])
        if case_id in case_links:
            raise ProbeFailure("CASE_ID")
        if disposition not in {"admitted_shape", "rejected_near_miss"}:
            raise ProbeFailure("CASE_DISPOSITION")
        case_links[case_id] = disposition
        source = text(case["source"])
        if require_sha256(case["source_utf8_sha256"]) != sha256(source.encode("utf-8")):
            raise ProbeFailure("SOURCE_HASH")
        if "Mpk." in source or "MPK" in source:
            raise ProbeFailure("SOURCE_DEPENDENCY")
        outcome_class = compiler_outcome_class(case, disposition)
        if disposition == "rejected_near_miss":
            rejected_outcome_classes.add(outcome_class)
        if not array(case["syntax"]) or not array(case["operation_roots"]):
            raise ProbeFailure("CASE_OBSERVATIONS")
        if case["compiler_outcome"] == "success" and len(
            array(case["control_flow_graphs"])
        ) != cfg_eligible_root_count(case):
            raise ProbeFailure("SUCCESS_CFG_CLOSURE")
        source_order = array(case["source_order"])
        previous_start = -1
        for expected_ordinal, order_value in enumerate(source_order):
            order = exact_object(
                order_value, {"category", "id", "source_ordinal", "start"}
            )
            start = integer(order["start"])
            if integer(order["source_ordinal"]) != expected_ordinal or start < previous_start:
                raise ProbeFailure("SOURCE_ORDER")
            previous_start = start
        for target_value in array(case["targets"]):
            target = exact_object(target_value, TARGET_KEYS)
            shape_id = text(target["shape_id"])
            if shape_id in actual_targets:
                raise ProbeFailure("DUPLICATE_SHAPE")
            actual_targets[shape_id] = (
                case_id,
                disposition,
                integer(target["source_ordinal"]),
                sha256(canonical(target)),
            )
        for family, key in (
            ("decision_graph", "decision_graphs"),
            ("exception_region", "exception_regions"),
        ):
            for observation_value in array(case[key]):
                if not isinstance(observation_value, dict):
                    raise ProbeFailure("UPGRADE_OBSERVATION")
                observation_id = text(observation_value.get("id"))
                if observation_id in actual_observations:
                    raise ProbeFailure("OBSERVATION_ID")
                actual_observations[observation_id] = (
                    case_id,
                    family,
                    sha256(canonical(observation_value)),
                    upgrade_mutation_field(family, observation_value),
                )
    if len(cases) != EXPECTED_CASE_COUNT:
        raise ProbeFailure("CASE_COUNT")
    if rejected_outcome_classes != {"clean", "error", "warning"}:
        raise ProbeFailure("REJECTED_COMPILER_OUTCOMES")

    shape_index = array(document["shape_index"])
    shape_ids: list[str] = []
    admitted: list[str] = []
    rejected: list[str] = []
    for row_value in shape_index:
        row = exact_object(
            row_value,
            {
                "case_id",
                "disposition",
                "observation_sha256",
                "shape_id",
                "source_ordinal",
            },
        )
        shape_id = text(row["shape_id"])
        expected = actual_targets.get(shape_id)
        if expected is None or (
            text(row["case_id"]),
            text(row["disposition"]),
            integer(row["source_ordinal"]),
            require_sha256(row["observation_sha256"]),
        ) != expected:
            raise ProbeFailure("SHAPE_LINK")
        shape_ids.append(shape_id)
        if row["disposition"] == "admitted_shape":
            admitted.append(shape_id)
        elif row["disposition"] == "rejected_near_miss":
            rejected.append(shape_id)
        else:
            raise ProbeFailure("SHAPE_DISPOSITION")
    if (
        shape_ids != sorted(shape_ids)
        or len(shape_ids) != len(actual_targets)
        or len(shape_ids) != EXPECTED_SHAPE_COUNT
        or len(admitted) != EXPECTED_ADMITTED_SHAPE_COUNT
        or len(rejected) != EXPECTED_REJECTED_SHAPE_COUNT
        or sha256(canonical(shape_ids)) != SHAPE_IDS_SHA256
        or sha256(canonical(admitted)) != ADMITTED_SHAPE_IDS_SHA256
        or sha256(canonical(rejected)) != REJECTED_SHAPE_IDS_SHA256
    ):
        raise ProbeFailure("SHAPE_CATALOG")
    if document["coverage"] != expected_coverage(shape_ids):
        raise ProbeFailure("COVERAGE")

    mutations = array(document["upgrade_mutations"])
    mutation_ids: list[str] = []
    linked_observations: set[str] = set()
    observation_counts: dict[tuple[str, str], int] = {}
    for row_value in mutations:
        row = exact_object(
            row_value,
            {
                "case_id",
                "disposition",
                "family",
                "mutation_id",
                "mutation_field",
                "observation_id",
                "observation_sha256",
            },
        )
        observation_id = text(row["observation_id"])
        expected = actual_observations.get(observation_id)
        if expected is None or (
            text(row["case_id"]),
            text(row["family"]),
            require_sha256(row["observation_sha256"]),
            text(row["mutation_field"]),
        ) != expected:
            raise ProbeFailure("UPGRADE_LINK")
        if row["disposition"] != case_links[text(row["case_id"])]:
            raise ProbeFailure("UPGRADE_DISPOSITION")
        mutation_ids.append(text(row["mutation_id"]))
        linked_observations.add(observation_id)
        count_key = (text(row["family"]), text(row["disposition"]))
        observation_counts[count_key] = observation_counts.get(count_key, 0) + 1
    if (
        mutation_ids != sorted(mutation_ids)
        or len(mutation_ids) != len(set(mutation_ids))
        or linked_observations != set(actual_observations)
        or observation_counts != EXPECTED_OBSERVATION_COUNTS
        or {text(row["family"]) for row in mutations}
        != {"decision_graph", "exception_region"}
        or {text(row["disposition"]) for row in mutations}
        != {"admitted_shape", "rejected_near_miss"}
        or {text(row["mutation_field"]) for row in mutations}
        != {
            "handler_search_order[0]",
            "nesting_depth",
            "nodes[0].operation_kind",
        }
    ):
        raise ProbeFailure("UPGRADE_MUTATIONS")
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
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-t01-w05-") as temporary:
        temporary_root = Path(temporary)
        roots = active.materialize_closure(
            toolchain, archives, temporary_root / "closure"
        )
        first_raw, first_binary = run_once(descriptor, roots, temporary_root / "first")
        second_raw, second_binary = run_once(descriptor, roots, temporary_root / "second")
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
        prefix=".roslyn-control-exception-pattern-", dir=RESULT_PATH.parent
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
    mutations = array(document["upgrade_mutations"])
    families: set[str] = set()
    for row_value in mutations:
        row = exact_object(
            row_value,
            {
                "case_id",
                "disposition",
                "family",
                "mutation_id",
                "mutation_field",
                "observation_id",
                "observation_sha256",
            },
        )
        family = text(row["family"])
        families.add(family)
        observation = observation_by_id(
            document, family, text(row["observation_id"])
        )
        mutated = copy.deepcopy(observation)
        apply_upgrade_mutation(mutated, text(row["mutation_field"]))
        if sha256(canonical(mutated)) == require_sha256(row["observation_sha256"]):
            raise ProbeFailure("SELF_TEST_MUTATION_ACCEPTED")
    if families != {"decision_graph", "exception_region"}:
        raise ProbeFailure("SELF_TEST_FAMILIES")
    for family in sorted(families):
        changed = copy.deepcopy(document)
        row = next(
            value
            for value in array(changed["upgrade_mutations"])
            if isinstance(value, dict) and value.get("family") == family
        )
        observation = observation_by_id(
            changed, family, text(row["observation_id"])
        )
        apply_upgrade_mutation(observation, text(row["mutation_field"]))
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
