#!/usr/bin/env python3
"""Build, run, normalize, and validate the private T01-W06 Roslyn probe."""

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


WORK_ITEM = "CSHARP-03-T01-W06"
FINAL_SCHEMA = "mpk.csharp_practical.t01_w06.roslyn_exclusion_probe.v0"
RAW_SCHEMA = "mpk.csharp_practical.t01_w06.roslyn_exclusion_probe.raw.v0"
PROBE_PATH = (
    REPOSITORY_ROOT
    / "develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs"
)
RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json"
)
W05_RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json"
)
W05_SOURCE_PATH = (
    REPOSITORY_ROOT / "develop/probes/csharp-03/ControlExceptionPatternProbe.cs"
)
W03_DESCRIPTOR_PATH = practical.DESCRIPTOR_PATH
W03_INVENTORY_PATH = practical.INVENTORY_PATH
W05_COMMIT = "13415911853c0368c103bd9d5feeb8374596d724"
W05_TREE = "5d9000f11b2c3cab35ad08dc61a66fb14894d249"
W05_RESULT_SHA256 = "b1215ad7f4a0e08dc269834229d7158158d31c0e9475218fa0791feea5a1629a"
W05_RESULT_SIZE = 2_331_920
W05_SOURCE_SHA256 = "f62ff3deb7c0fff2799f99426ab9dbd7e6fd373a5fd9d8ed91bbb118a9808f1f"
W05_SOURCE_SIZE = 70_299
W03_DESCRIPTOR_SHA256 = "83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015"
W03_INVENTORY_SHA256 = "ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce"
PROBE_SOURCE_SHA256 = "7e2114bdb75ef5b78e330c24e04c551c7766740ba037a12419547212026c6db6"
PROBE_SOURCE_SIZE = 89_065
EXPECTED_CASE_COUNT = 16
EXPECTED_SHAPE_COUNT = 144
EXPECTED_ADMITTED_EXCEPTION_COUNT = 12
EXPECTED_REJECTED_COUNT = 132
SHAPE_IDS_SHA256 = "6f7cb87aa1efae91b220244b5b85cac5d13e9995b8b93539bc04cc1925060446"
ADMITTED_SHAPE_IDS_SHA256 = (
    "3529ba40edc421a2a19fe74eceaf825426063c4336da2b72c75cc4c06633d35c"
)
REJECTED_SHAPE_IDS_SHA256 = (
    "4a72c24a0b06bb25e4e8b69dcd17695a253d754ee85b6599c636d9d944415ef4"
)
FAMILY_IDS_SHA256 = "407f67fc75f02b61d555834ade2f192e0db3e249f74f16b505291235bb7e93be"
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
    "base_reference_count": 167,
    "language": "C#",
    "language_version": "14.0",
    "nullable_context": "Enable",
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

FAMILY_PREFIXES = (
    ("exception.compiler_metadata.", "exception.compiler_metadata"),
    ("exception.incidental.", "exception.incidental_metadata"),
    ("exception.nullable.", "exception.nullable_shorthand"),
    ("exception.array.", "exception.array_non_generic"),
    ("near_miss.dependency.generated_source.", "dependency.generated_source"),
    ("near_miss.dependency.namespace.", "dependency.namespace"),
    ("near_miss.dependency.package.", "dependency.package"),
    ("near_miss.dependency.assembly.", "dependency.assembly"),
    ("near_miss.dependency.attribute.", "dependency.attribute"),
    ("near_miss.dependency.interface.", "dependency.interface"),
    ("near_miss.dependency.base_type.", "dependency.base_type"),
    ("near_miss.dependency.project.", "dependency.project"),
    ("near_miss.dependency.ambient.", "dependency.ambient"),
    ("near_miss.attribute.compiler_marker.", "attribute.compiler_marker_spelling"),
    ("near_miss.attribute.source.", "attribute.source_written"),
    ("near_miss.generic.declaration.", "generic.declaration"),
    ("near_miss.generic.method.", "generic.method"),
    ("near_miss.generic.type_parameter.", "generic.type_parameter"),
    ("near_miss.generic.constraint.", "generic.constraint"),
    ("near_miss.generic.variance.", "generic.variance"),
    ("near_miss.generic.explicit_call.", "generic.explicit_call"),
    ("near_miss.generic.inferred_call.", "generic.inferred_call"),
    ("near_miss.generic.closed_use.", "generic.closed_use"),
    ("near_miss.generic.framework_type.", "generic.framework_type"),
    ("near_miss.generic.open_type.", "generic.open_type"),
    ("near_miss.generic.explicit_nullable.", "generic.explicit_nullable"),
    ("near_miss.generic.unsupported_nullable.", "generic.unsupported_nullable"),
    ("near_miss.generic.transitive_metadata.", "generic.transitive_metadata"),
    ("near_miss.iterator.async.", "iterator.async"),
    ("near_miss.iterator.declaration.", "iterator.declaration"),
    ("near_miss.iterator.yield.", "iterator.yield"),
    ("near_miss.iterator.protocol.", "iterator.protocol"),
    ("near_miss.iterator.state_machine", "iterator.state_machine"),
    ("near_miss.async.declaration.", "async.declaration"),
    ("near_miss.async.await.", "async.await"),
    ("near_miss.async.task.", "async.task"),
    ("near_miss.async.value_task.", "async.value_task"),
    ("near_miss.async.awaiter.", "async.awaiter"),
    ("near_miss.async.cancellation.", "async.cancellation"),
    ("near_miss.async.parallel.", "async.parallel"),
    ("near_miss.async.state_machine", "async.state_machine"),
)
EXPECTED_FAMILIES = {family for _, family in FAMILY_PREFIXES}
ADMITTED_NULLABLE_VALUE_SHAPES = {
    "exception.nullable.shorthand.default",
    "exception.nullable.shorthand.implicit_conversion",
    "exception.nullable.shorthand.local_type",
    "exception.nullable.shorthand.return_type",
}
ADMITTED_INCIDENTAL_SHAPES = {
    "exception.incidental.array_length": ("PropertyReference", "System.Array.Length"),
    "exception.incidental.date_only_constructor": (
        "ObjectCreation",
        "System.DateOnly.DateOnly(int, int, int)",
    ),
    "exception.incidental.decimal_round": (
        "Invocation",
        "decimal.Round(decimal, int, System.MidpointRounding)",
    ),
    "exception.incidental.string_length": ("PropertyReference", "string.Length"),
}

RAW_KEYS = {"cases", "compiler", "schema", "synthetic_references", "work_item"}
CASE_KEYS = {
    "compiler_outcome",
    "diagnostics",
    "disposition",
    "emitted_metadata",
    "extra_references",
    "generated_sources",
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
    "converted_type",
    "declared_symbol",
    "emitted_evidence",
    "enclosing_symbol",
    "family",
    "generic_facts",
    "marker_span",
    "operation",
    "profile_outcome",
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
GENERIC_FACT_KEYS = {
    "constructed_nullable_value_type",
    "immediate_specialization",
    "source_contains_generic_name",
    "source_contains_nullable_shorthand",
    "source_contains_type_parameter",
    "symbol_arity",
    "symbol_is_generic",
    "type_arguments",
    "type_parameters",
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
        return json.loads(
            data.decode("utf-8"),
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


def object_value(value: object) -> dict[str, object]:
    if not isinstance(value, dict):
        raise ProbeFailure("SCHEMA_OBJECT")
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


def boolean(value: object) -> bool:
    if not isinstance(value, bool):
        raise ProbeFailure("SCHEMA_BOOLEAN")
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
        (W05_RESULT_PATH, MAX_PROBE_OUTPUT_BYTES, W05_RESULT_SIZE, W05_RESULT_SHA256),
        (W05_SOURCE_PATH, MAX_PROBE_SOURCE_BYTES, W05_SOURCE_SIZE, W05_SOURCE_SHA256),
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
    probe_binary = output / "DependencyGenericSuspensionProbe.dll"
    reference_root = roots["microsoft-netcore-app-ref"] / "ref/net10.0"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(probe_binary),
            "/main:DependencyGenericSuspensionProbe",
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
    if unexpected != ["DependencyGenericSuspensionProbe.dll"]:
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


def family_for_shape(shape_id: str) -> str:
    for prefix, family in FAMILY_PREFIXES:
        if shape_id.startswith(prefix):
            return family
    raise ProbeFailure("SHAPE_FAMILY")


def validate_span(value: object, source_length: int) -> None:
    span = exact_object(value, {"end", "length", "start"})
    start = integer(span["start"])
    length = integer(span["length"])
    end = integer(span["end"])
    if start + length != end or end > source_length:
        raise ProbeFailure("SPAN")


def compiler_outcome(case: dict[str, object], disposition: str) -> str:
    outcome = text(case["compiler_outcome"])
    diagnostics = array(case["diagnostics"])
    severities = [
        text(exact_object(value, DIAGNOSTIC_KEYS)["severity"])
        for value in diagnostics
    ]
    expected = "error" if "Error" in severities else "success"
    if outcome != expected:
        raise ProbeFailure("COMPILER_OUTCOME")
    if disposition == "admitted_exception_observation":
        if outcome != "success" or diagnostics:
            raise ProbeFailure("ADMITTED_EXCEPTION_OUTCOME")
        return "admitted"
    if disposition != "rejected_profile_form":
        raise ProbeFailure("CASE_DISPOSITION")
    if outcome == "error":
        return "error"
    return "warning" if diagnostics else "clean"


def mutation_field(target: dict[str, object]) -> str:
    evidence = array(target["emitted_evidence"])
    if evidence:
        return "emitted_evidence[0]"
    operation = target["operation"]
    if isinstance(operation, dict) and isinstance(operation.get("kind"), str):
        return "operation.kind"
    symbol = target["symbol"]
    if isinstance(symbol, dict) and isinstance(symbol.get("display"), str):
        return "symbol.display"
    declared = target["declared_symbol"]
    if isinstance(declared, dict) and isinstance(declared.get("display"), str):
        return "declared_symbol.display"
    return "syntax.kind"


def apply_mutation(target: dict[str, object], field: str) -> None:
    if field == "emitted_evidence[0]":
        array(target["emitted_evidence"])[0] = "#upgrade-mutation"
    elif field == "operation.kind":
        operation = object_value(target["operation"])
        operation["kind"] = text(operation["kind"]) + "#upgrade-mutation"
    elif field == "symbol.display":
        symbol = object_value(target["symbol"])
        symbol["display"] = text(symbol["display"]) + "#upgrade-mutation"
    elif field == "declared_symbol.display":
        declared = object_value(target["declared_symbol"])
        declared["display"] = text(declared["display"]) + "#upgrade-mutation"
    elif field == "syntax.kind":
        syntax = object_value(target["syntax"])
        syntax["kind"] = text(syntax["kind"]) + "#upgrade-mutation"
    else:
        raise ProbeFailure("MUTATION_FIELD")


def catalog(raw: dict[str, object]) -> tuple[list[object], list[object], list[object]]:
    if raw["schema"] != RAW_SCHEMA or raw["work_item"] != WORK_ITEM:
        raise ProbeFailure("RAW_IDENTITY")
    if raw["compiler"] != EXPECTED_COMPILER:
        raise ProbeFailure("COMPILER_IDENTITY")
    synthetic = array(raw["synthetic_references"])
    synthetic_ids: list[str] = []
    synthetic_origins: list[str] = []
    for value in synthetic:
        reference = exact_object(
            value,
            {
                "assembly_name",
                "id",
                "origin",
                "pe_sha256",
                "pe_size_bytes",
                "source_sha256",
                "virtual_path",
            },
        )
        synthetic_ids.append(text(reference["id"]))
        synthetic_origins.append(text(reference["origin"]))
        require_sha256(reference["pe_sha256"])
        require_sha256(reference["source_sha256"])
        integer(reference["pe_size_bytes"])
        if not text(reference["virtual_path"]).startswith("/virtual/"):
            raise ProbeFailure("SYNTHETIC_PATH")
    if synthetic_ids != ["ambient-project", "mpk-package", "mpk-project"]:
        raise ProbeFailure("SYNTHETIC_IDS")
    if set(synthetic_origins) != {"ambient", "package", "project"}:
        raise ProbeFailure("SYNTHETIC_ORIGINS")

    cases = array(raw["cases"])
    case_ids: set[str] = set()
    shape_records: list[dict[str, object]] = []
    rejected_outcomes: set[str] = set()
    positive_case_count = 0
    for case_value in cases:
        case = exact_object(case_value, CASE_KEYS)
        case_id = text(case["id"])
        disposition = text(case["disposition"])
        if not case_id or case_id in case_ids:
            raise ProbeFailure("CASE_ID")
        case_ids.add(case_id)
        outcome_class = compiler_outcome(case, disposition)
        if disposition == "rejected_profile_form":
            rejected_outcomes.add(outcome_class)
        else:
            positive_case_count += 1
        source = text(case["source"])
        source_length = len(source.encode("utf-16-le")) // 2
        if require_sha256(case["source_utf8_sha256"]) != sha256(source.encode("utf-8")):
            raise ProbeFailure("SOURCE_HASH")
        if not array(case["syntax"]) or not array(case["operation_roots"]):
            raise ProbeFailure("CASE_OBSERVATIONS")
        for generated_value in array(case["generated_sources"]):
            generated = exact_object(
                generated_value, {"hint_name", "path", "source", "source_utf8_sha256"}
            )
            if require_sha256(generated["source_utf8_sha256"]) != sha256(
                text(generated["source"]).encode("utf-8")
            ):
                raise ProbeFailure("GENERATED_SOURCE_HASH")
        targets = array(case["targets"])
        order = array(case["source_order"])
        if len(targets) != len(order):
            raise ProbeFailure("SOURCE_ORDER_COUNT")
        previous_start = -1
        for ordinal, (target_value, order_value) in enumerate(zip(targets, order)):
            target = exact_object(target_value, TARGET_KEYS)
            ordering = exact_object(order_value, {"shape_id", "source_ordinal", "start"})
            shape_id = text(target["shape_id"])
            family = family_for_shape(shape_id)
            expected_outcome = (
                "admitted_exception" if shape_id.startswith("exception.") else "rejected"
            )
            if (
                integer(target["source_ordinal"]) != ordinal
                or integer(ordering["source_ordinal"]) != ordinal
                or ordering["shape_id"] != shape_id
                or target["family"] != family
                or target["profile_outcome"] != expected_outcome
            ):
                raise ProbeFailure("TARGET_IDENTITY")
            validate_span(target["marker_span"], source_length)
            validate_span(object_value(target["syntax"])["span"], source_length)
            start = integer(exact_object(target["marker_span"], {"end", "length", "start"})["start"])
            if integer(ordering["start"]) != start or start < previous_start:
                raise ProbeFailure("SOURCE_ORDER")
            previous_start = start
            facts = exact_object(target["generic_facts"], GENERIC_FACT_KEYS)
            boolean(facts["constructed_nullable_value_type"])
            boolean(facts["source_contains_generic_name"])
            boolean(facts["source_contains_nullable_shorthand"])
            boolean(facts["source_contains_type_parameter"])
            integer(facts["symbol_arity"])
            boolean(facts["symbol_is_generic"])
            array(facts["type_arguments"])
            array(facts["type_parameters"])
            constructed_nullable = boolean(facts["constructed_nullable_value_type"])
            specialization = facts["immediate_specialization"]
            if shape_id in ADMITTED_NULLABLE_VALUE_SHAPES:
                if (
                    not constructed_nullable
                    or boolean(facts["source_contains_generic_name"])
                ):
                    raise ProbeFailure("NULLABLE_EXCEPTION")
                specialized = exact_object(
                    specialization,
                    {"payload", "residual_type_parameter", "shape"},
                )
                payload = exact_object(
                    specialized["payload"],
                    {
                        "containing_assembly",
                        "display",
                        "metadata_name",
                        "nullable_annotation",
                        "original_definition",
                        "special_type",
                        "type_kind",
                    },
                )
                if (
                    text(specialized["shape"]) != "option"
                    or boolean(specialized["residual_type_parameter"])
                    or text(payload["display"]) != "int"
                    or text(payload["type_kind"]) != "Struct"
                ):
                    raise ProbeFailure("NULLABLE_SPECIALIZATION")
            elif expected_outcome == "admitted_exception" and constructed_nullable:
                raise ProbeFailure("UNEXPECTED_CONSTRUCTED_EXCEPTION")
            elif not constructed_nullable and specialization is not None:
                raise ProbeFailure("UNEXPECTED_SPECIALIZATION")
            if shape_id == "exception.nullable.reference_annotation" and (
                constructed_nullable or specialization is not None
            ):
                raise ProbeFailure("REFERENCE_NULLABLE_CLASSIFICATION")
            if shape_id == "exception.array.not_constructed_generic" and (
                constructed_nullable
                or specialization is not None
                or boolean(facts["symbol_is_generic"])
            ):
                raise ProbeFailure("ARRAY_GENERIC_CLASSIFICATION")
            if shape_id in ADMITTED_INCIDENTAL_SHAPES:
                operation = object_value(target["operation"])
                symbol = object_value(target["symbol"])
                expected_operation, expected_symbol = ADMITTED_INCIDENTAL_SHAPES[
                    shape_id
                ]
                if (
                    text(operation["kind"]) != expected_operation
                    or text(symbol["display"]) != expected_symbol
                    or boolean(facts["source_contains_generic_name"])
                ):
                    raise ProbeFailure("INCIDENTAL_METADATA_BOUNDARY")
            if target["operation"] is None and target["symbol"] is None and target["declared_symbol"] is None:
                if target["syntax"] is None:
                    raise ProbeFailure("TARGET_OBSERVATION")
            shape_records.append(
                {
                    "case_id": case_id,
                    "disposition": disposition,
                    "family": family,
                    "observation_sha256": sha256(canonical(target)),
                    "profile_outcome": expected_outcome,
                    "shape_id": shape_id,
                    "source_ordinal": ordinal,
                }
            )
    if len(cases) != EXPECTED_CASE_COUNT or positive_case_count != 1:
        raise ProbeFailure("CASE_COUNT")
    if rejected_outcomes != {"clean", "error", "warning"}:
        raise ProbeFailure("REJECTED_OUTCOMES")

    shape_records.sort(key=lambda value: text(value["shape_id"]))
    shape_ids = [text(value["shape_id"]) for value in shape_records]
    if len(shape_ids) != len(set(shape_ids)):
        raise ProbeFailure("DUPLICATE_SHAPE")
    admitted = [
        text(value["shape_id"])
        for value in shape_records
        if value["profile_outcome"] == "admitted_exception"
    ]
    rejected = [
        text(value["shape_id"])
        for value in shape_records
        if value["profile_outcome"] == "rejected"
    ]
    actual_hashes = {
        "all": sha256(canonical(shape_ids)),
        "admitted": sha256(canonical(admitted)),
        "rejected": sha256(canonical(rejected)),
    }
    if (
        len(shape_ids) != EXPECTED_SHAPE_COUNT
        or len(admitted) != EXPECTED_ADMITTED_EXCEPTION_COUNT
        or len(rejected) != EXPECTED_REJECTED_COUNT
        or actual_hashes["all"] != SHAPE_IDS_SHA256
        or actual_hashes["admitted"] != ADMITTED_SHAPE_IDS_SHA256
        or actual_hashes["rejected"] != REJECTED_SHAPE_IDS_SHA256
    ):
        raise ProbeFailure(
            "SHAPE_CATALOG",
            canonical(
                {
                    "actual_counts": [len(shape_ids), len(admitted), len(rejected)],
                    "actual_hashes": actual_hashes,
                }
            ),
        )

    family_rows: list[object] = []
    for family in sorted({text(value["family"]) for value in shape_records}):
        rows = [value for value in shape_records if value["family"] == family]
        family_shape_ids = [text(value["shape_id"]) for value in rows]
        family_rows.append(
            {
                "family": family,
                "profile_outcomes": sorted(
                    {text(value["profile_outcome"]) for value in rows}
                ),
                "shape_ids": family_shape_ids,
                "shape_ids_sha256": sha256(canonical(family_shape_ids)),
            }
        )
    family_ids = [text(object_value(value)["family"]) for value in family_rows]
    if set(family_ids) != EXPECTED_FAMILIES or sha256(canonical(family_ids)) != FAMILY_IDS_SHA256:
        raise ProbeFailure(
            "FAMILY_CATALOG",
            canonical(
                {
                    "actual_families": family_ids,
                    "actual_sha256": sha256(canonical(family_ids)),
                }
            ),
        )

    targets_by_shape: dict[str, dict[str, object]] = {}
    for case_value in cases:
        case = exact_object(case_value, CASE_KEYS)
        for target_value in array(case["targets"]):
            target = exact_object(target_value, TARGET_KEYS)
            targets_by_shape[text(target["shape_id"])] = target
    mutations: list[object] = []
    for row in shape_records:
        shape_id = text(row["shape_id"])
        target = targets_by_shape[shape_id]
        field = mutation_field(target)
        mutations.append(
            {
                "case_id": row["case_id"],
                "family": row["family"],
                "mutation_field": field,
                "mutation_id": "CSHARP-03-T01-W06-UPGRADE-"
                + shape_id.upper().replace(".", "-"),
                "observation_sha256": row["observation_sha256"],
                "profile_outcome": row["profile_outcome"],
                "shape_id": shape_id,
            }
        )
    mutations.sort(key=lambda value: text(object_value(value)["mutation_id"]))
    if len(mutations) != len(shape_records):
        raise ProbeFailure("MUTATION_COUNT")
    return (
        [dict(value) for value in shape_records],
        family_rows,
        mutations,
    )


def normalize(raw_bytes: bytes, binary_sha256: str) -> dict[str, object]:
    raw = exact_object(strict_json(raw_bytes), RAW_KEYS)
    shape_index, family_index, mutations = catalog(raw)
    canonical_raw = canonical(raw) + b"\n"
    return {
        "baseline": {
            "build_inputs": W03_DESCRIPTOR_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": W03_INVENTORY_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "control_probe": W05_RESULT_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "control_probe_raw_sha256": W05_RESULT_SHA256,
            "source_commit": W05_COMMIT,
            "source_tree": W05_TREE,
        },
        "family_index": family_index,
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
        "upgrade_mutations": mutations,
        "work_item": WORK_ITEM,
    }


def target_by_shape(document: dict[str, object], shape_id: str) -> dict[str, object]:
    observations = exact_object(document["observations"], RAW_KEYS)
    for case_value in array(observations["cases"]):
        case = exact_object(case_value, CASE_KEYS)
        for target_value in array(case["targets"]):
            target = exact_object(target_value, TARGET_KEYS)
            if target["shape_id"] == shape_id:
                return target
    raise ProbeFailure("SHAPE_TARGET_MISSING")


def validate_document(value: object, *, check_live_inputs: bool) -> dict[str, object]:
    document = exact_object(
        value,
        {
            "baseline",
            "family_index",
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
        "control_probe": "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json",
        "control_probe_raw_sha256": W05_RESULT_SHA256,
        "source_commit": W05_COMMIT,
        "source_tree": W05_TREE,
    }:
        raise ProbeFailure("BASELINE")
    if document["probe_input"] != {
        "compiler_arguments": list(COMPILER_ARGUMENTS),
        "path": "develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs",
        "raw_sha256": PROBE_SOURCE_SHA256,
        "reference_projection_sha256": active.REFERENCE_HASH,
        "size_bytes": PROBE_SOURCE_SIZE,
        "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
    }:
        raise ProbeFailure("PROBE_INPUT")
    observations = exact_object(document["observations"], RAW_KEYS)
    shape_index, family_index, mutations = catalog(observations)
    if document["shape_index"] != shape_index:
        raise ProbeFailure("SHAPE_INDEX")
    if document["family_index"] != family_index:
        raise ProbeFailure("FAMILY_INDEX")
    if document["upgrade_mutations"] != mutations:
        raise ProbeFailure("UPGRADE_MUTATIONS")
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
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-t01-w06-") as temporary:
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
        prefix=".roslyn-dependency-generic-suspension-", dir=RESULT_PATH.parent
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
    families: set[str] = set()
    for row_value in array(document["upgrade_mutations"]):
        row = exact_object(
            row_value,
            {
                "case_id",
                "family",
                "mutation_field",
                "mutation_id",
                "observation_sha256",
                "profile_outcome",
                "shape_id",
            },
        )
        shape_id = text(row["shape_id"])
        families.add(text(row["family"]))
        changed = copy.deepcopy(document)
        target = target_by_shape(changed, shape_id)
        apply_mutation(target, text(row["mutation_field"]))
        if sha256(canonical(target)) == require_sha256(row["observation_sha256"]):
            raise ProbeFailure("SELF_TEST_MUTATION_ACCEPTED")
        try:
            validate_document(changed, check_live_inputs=False)
        except ProbeFailure:
            pass
        else:
            raise ProbeFailure("SELF_TEST_SCHEMA_ACCEPTED")
    if families != EXPECTED_FAMILIES:
        raise ProbeFailure("SELF_TEST_FAMILIES")


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
