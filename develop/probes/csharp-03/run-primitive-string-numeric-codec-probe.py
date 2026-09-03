#!/usr/bin/env python3
"""Build, run, normalize, and validate the private T01-W07 runtime probe."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import platform
import stat
import sys
import tempfile
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_ROOT = REPOSITORY_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_ROOT))

import csharp_build_inputs as active  # noqa: E402
import csharp_practical_build_inputs as practical  # noqa: E402


WORK_ITEM = "CSHARP-03-T01-W07"
FINAL_SCHEMA = "mpk.csharp_practical.t01_w07.runtime_semantics_probe.v0"
OBSERVATION_SCHEMA = "mpk.csharp_practical.t01_w07.runtime_semantics_observations.v0"
RAW_SCHEMA = "mpk.csharp_practical.t01_w07.runtime_semantics_probe.raw.v0"
PROBE_PATH = REPOSITORY_ROOT / "develop/probes/csharp-03/PrimitiveStringNumericCodecProbe.cs"
RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json"
)
W06_RESULT_PATH = (
    REPOSITORY_ROOT
    / "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json"
)
W06_SOURCE_PATH = (
    REPOSITORY_ROOT / "develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs"
)
W03_DESCRIPTOR_PATH = practical.DESCRIPTOR_PATH
W03_INVENTORY_PATH = practical.INVENTORY_PATH

W06_COMMIT = "22673dbc96d8ba4f0d9a4cb97c3f2490c00d1804"
W06_TREE = "687631b3799ba385ccde29de9d72286c48d3f8fc"
W06_RESULT_SHA256 = "5dadf10613f95be9b35c108008a33474c55d222bef1be987c2614c6dcc48fe96"
W06_RESULT_SIZE = 4_511_101
W06_SOURCE_SHA256 = "7e2114bdb75ef5b78e330c24e04c551c7766740ba037a12419547212026c6db6"
W06_SOURCE_SIZE = 89_065
W03_DESCRIPTOR_SHA256 = "83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015"
W03_INVENTORY_SHA256 = "ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce"

PROBE_SOURCE_SHA256 = "d587acd6b1baab5602c8da8c54a803a9baa797400b70a6328bfd059e6a9f5f42"
PROBE_SOURCE_SIZE = 126_717
EXPECTED_VECTOR_COUNT = 3_468
EXPECTED_OPERATION_COUNT = 154
EXPECTED_VECTOR_IDS_SHA256 = "e4e2f9c55154bec304a66e80c5d574c071307ff91e4bd93b3a0073153905073c"
EXPECTED_OPERATION_IDS_SHA256 = "96db56971b3cc908ac618880bf4d1993567d0217ea3a325c14deb4691277b3a5"
EXPECTED_FAMILY_IDS_SHA256 = "802e897a25d358fce385ea9390da70a5c2cd5bb9a3d6f4dc5a419e5ee6e9da37"
EXPECTED_CULTURE_VARIANT_IDS_SHA256 = "d17191a68f4d0e2e0596e309e4e945765f294f7e0a2a2a397e558fc66ae0c965"
EXPECTED_CULTURE_VARIANT_COUNT = 83

MAX_PROBE_SOURCE_BYTES = 2 * 1024 * 1024
MAX_RAW_OUTPUT_BYTES = 32 * 1024 * 1024
MAX_RESULT_BYTES = 128 * 1024 * 1024
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

CULTURE_PROFILES = ("hostile-arabic", "hostile-comma", "hostile-swap")
EXPECTED_FAMILIES = {
    "codec.date_time",
    "codec.decimal",
    "codec.duration_instant",
    "codec.floating_bits",
    "codec.guid",
    "codec.integer",
    "decimal.arithmetic",
    "decimal.comparison",
    "decimal.conversion",
    "decimal.culture_rejection",
    "decimal.edge",
    "decimal.representation",
    "decimal.rounding",
    "error_precedence",
    "floating.conversion",
    "floating.culture_rejection",
    "floating.double",
    "floating.single",
    "string.concat",
    "string.culture_rejection",
    "string.interpolation",
    "string.null",
    "string.ordinal",
    "string.range",
    "string.substring",
    "string.utf16",
}
EXPECTED_ENCODINGS = {
    "ascii",
    "bcl_general_parse",
    "bool",
    "comparison_sign",
    "date_day_number",
    "decimal_bits",
    "guid_n_ascii",
    "ieee_binary32_bits",
    "ieee_binary64_bits",
    "none",
    "signed_decimal",
    "time_ticks",
    "u16_hex",
    "utf16_hex",
}
EXPECTED_RUNTIME_EXCEPTIONS = {
    "System.ArgumentNullException",
    "System.ArgumentOutOfRangeException",
    "System.DivideByZeroException",
    "System.IndexOutOfRangeException",
    "System.NullReferenceException",
    "System.OverflowException",
}
RAW_KEYS = {"culture", "runtime", "schema", "vectors", "work_item"}
VECTOR_KEYS = {
    "accepted_domain",
    "differential",
    "error_precedence",
    "family",
    "id",
    "inputs",
    "operation",
    "profile",
    "profile_outcome",
    "runtime_culture_sensitive",
}
RESULT_KEYS = {"error_id", "kind", "result_encoding", "value"}
DIFFERENTIAL_KEYS = {"exception", "kind", "result_encoding", "value"}

COVERAGE = {
    "culture_independence_and_error_precedence": ["error_precedence"],
    "decimal_arithmetic_rounding_overflow": [
        "decimal.arithmetic",
        "decimal.comparison",
        "decimal.conversion",
        "decimal.culture_rejection",
        "decimal.edge",
        "decimal.representation",
        "decimal.rounding",
    ],
    "exact_boundary_codecs": [
        "codec.date_time",
        "codec.decimal",
        "codec.duration_instant",
        "codec.floating_bits",
        "codec.guid",
        "codec.integer",
    ],
    "floating_bits_arithmetic_and_conversions": [
        "floating.conversion",
        "floating.culture_rejection",
        "floating.double",
        "floating.single",
    ],
    "utf16_ordinal_concat_interpolation_and_null": [
        "string.concat",
        "string.culture_rejection",
        "string.interpolation",
        "string.null",
        "string.ordinal",
        "string.range",
        "string.substring",
        "string.utf16",
    ],
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


def strict_json(data: bytes, maximum: int) -> object:
    if len(data) > maximum or data.startswith(b"\xef\xbb\xbf"):
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
    if len(digest) != 64 or any(character not in "0123456789abcdef" for character in digest):
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


def utf16_hex(value: str) -> str:
    data = value.encode("utf-16-le")
    return "".join(f"{int.from_bytes(data[index:index + 2], 'little'):04x}" for index in range(0, len(data), 2))


def expected_culture(profile: str) -> dict[str, object]:
    values = {
        "hostile-arabic": ("*", "\u066b", "\u066c", "\u2212", "dd*MM*yyyy", "!"),
        "hostile-comma": (".", ",", ".", "~", "yyyy.MM.dd", "-"),
        "hostile-swap": ("_", ";", ",", "NEG", "MM_dd_yyyy", "."),
    }
    date, decimal, group, negative, pattern, time = values[profile]
    return {
        "date_separator_utf16": utf16_hex(date),
        "decimal_separator_utf16": utf16_hex(decimal),
        "group_separator_utf16": utf16_hex(group),
        "negative_sign_utf16": utf16_hex(negative),
        "profile": profile,
        "short_date_pattern_utf16": utf16_hex(pattern),
        "time_separator_utf16": utf16_hex(time),
    }


def validate_frozen_inputs() -> dict[str, object]:
    checks = (
        (PROBE_PATH, MAX_PROBE_SOURCE_BYTES, PROBE_SOURCE_SIZE, PROBE_SOURCE_SHA256),
        (W06_RESULT_PATH, MAX_RESULT_BYTES, W06_RESULT_SIZE, W06_RESULT_SHA256),
        (W06_SOURCE_PATH, MAX_PROBE_SOURCE_BYTES, W06_SOURCE_SIZE, W06_SOURCE_SHA256),
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


def run_probe(
    runtime: Path,
    binary: Path,
    profile: str,
    root: Path,
    mutation: str,
) -> bytes:
    environment = active.closed_dotnet_environment(runtime, root)
    environment["MPK_CSHARP_PRACTICAL_UNLISTED_RUNTIME"] = mutation
    executed = active.execute_isolated(
        [
            str(runtime / "dotnet"),
            "exec",
            "--runtimeconfig",
            str(REPOSITORY_ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json"),
            "--fx-version",
            "10.0.11",
            str(binary),
            profile,
        ],
        cwd=binary.parent,
        environment=environment,
    )
    if executed.returncode != 0 or executed.stderr or not executed.stdout:
        raise ProbeFailure("EXECUTION", executed.stderr or executed.stdout)
    if len(executed.stdout) > MAX_RAW_OUTPUT_BYTES:
        raise ProbeFailure("OUTPUT_SIZE")
    return executed.stdout


def run_once(
    descriptor: dict[str, object], roots: dict[str, Path], root: Path
) -> tuple[list[bytes], str]:
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    output = root / "output"
    output.mkdir(mode=0o700)
    toolchain = active.exact_keys(
        descriptor["toolchain_inputs"], set(descriptor["toolchain_inputs"])
    )
    sdk = roots["dotnet-sdk-linux-x64"]
    runtime = roots["dotnet-runtime-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    probe_binary = output / "PrimitiveStringNumericCodecProbe.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(probe_binary),
            "/main:PrimitiveStringNumericCodecProbe",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    for untyped in active.array(toolchain["reference_projection"]["inventory"]):
        record = active.exact_keys(untyped, {"path", "sha256", "size_bytes"})
        arguments.append(
            "/reference:"
            + str(roots["microsoft-netcore-app-ref"] / active.text(record["path"]))
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
    if sorted(path.name for path in output.iterdir()) != ["PrimitiveStringNumericCodecProbe.dll"]:
        raise ProbeFailure("COMPILER_OUTPUT_SET")
    binary = read_regular(probe_binary, active.MAX_EXTRACTED_BYTES)
    binary_sha256 = sha256(binary)
    runs: list[bytes] = []
    for profile in CULTURE_PROFILES:
        clean = run_probe(
            runtime,
            probe_binary,
            profile,
            root / ("runtime-" + profile + "-clean"),
            "clean",
        )
        hostile = run_probe(
            runtime,
            probe_binary,
            profile,
            root / ("runtime-" + profile + "-hostile"),
            "hostile-unlisted-value",
        )
        if clean != hostile:
            raise ProbeFailure("UNLISTED_RUNTIME_INPUT_DEPENDENCY")
        runs.append(clean)
    return runs, binary_sha256


def validate_result_observation(value: object, profile: bool) -> dict[str, object]:
    keys = RESULT_KEYS if profile else DIFFERENTIAL_KEYS
    result = exact_object(value, keys)
    kind = text(result["kind"])
    encoding = text(result["result_encoding"])
    result_value = text(result["value"])
    if encoding not in EXPECTED_ENCODINGS or not result_value.isascii():
        raise ProbeFailure("RESULT_ENCODING")
    if len(result_value) > 1024:
        raise ProbeFailure("RESULT_BOUND")
    if profile:
        error = result["error_id"]
        if error is not None and (not isinstance(error, str) or not error.isascii()):
            raise ProbeFailure("PROFILE_ERROR")
        if kind == "value":
            if error is not None or encoding == "none":
                raise ProbeFailure("PROFILE_VALUE")
        elif kind in {"error", "rejected"}:
            if not isinstance(error, str) or encoding != "none" or result_value:
                raise ProbeFailure("PROFILE_FAILURE")
        else:
            raise ProbeFailure("PROFILE_KIND")
    else:
        exception = result["exception"]
        if kind == "value":
            if exception is not None or encoding == "none":
                raise ProbeFailure("DIFFERENTIAL_VALUE")
        elif kind == "exception":
            if exception not in EXPECTED_RUNTIME_EXCEPTIONS or encoding != "none" or result_value:
                raise ProbeFailure("DIFFERENTIAL_EXCEPTION")
        elif kind == "not_applicable":
            if exception is not None or encoding != "none" or result_value:
                raise ProbeFailure("DIFFERENTIAL_NOT_APPLICABLE")
        else:
            raise ProbeFailure("DIFFERENTIAL_KIND")
    return result


def semantic_projection(vector: dict[str, object]) -> dict[str, object]:
    result = dict(vector)
    del result["differential"]
    return result


def validate_raw(value: object, expected_profile: str) -> dict[str, object]:
    raw = exact_object(value, RAW_KEYS)
    if raw["schema"] != RAW_SCHEMA or raw["work_item"] != WORK_ITEM:
        raise ProbeFailure("RAW_IDENTITY")
    if raw["culture"] != expected_culture(expected_profile):
        raise ProbeFailure("CULTURE_PROFILE")
    if raw["runtime"] != {
        "architecture": "X64",
        "framework_description": ".NET 10.0.11",
        "runtime_version": "10.0.11",
    }:
        raise ProbeFailure("RUNTIME_IDENTITY")

    vectors = array(raw["vectors"])
    ids: list[str] = []
    for value in vectors:
        vector = exact_object(value, VECTOR_KEYS)
        vector_id = text(vector["id"])
        family = text(vector["family"])
        operation = text(vector["operation"])
        domain = text(vector["accepted_domain"])
        if (
            not vector_id
            or not vector_id.isascii()
            or family not in EXPECTED_FAMILIES
            or not operation
            or not operation.isascii()
            or not domain
            or not domain.isascii()
            or len(domain) > 512
        ):
            raise ProbeFailure("VECTOR_IDENTITY")
        inputs = array(vector["inputs"])
        if not inputs or any(not isinstance(item, str) or not item.isascii() or len(item) > 512 for item in inputs):
            raise ProbeFailure("VECTOR_INPUTS")
        precedence = tuple(text(item) for item in array(vector["error_precedence"]))
        if (
            len(precedence) != len(set(precedence))
            or any(
                not item.isascii()
                or not item.startswith(
                    ("exception.", "obligation.", "parse_error.", "sidecar.", "source_rejection.")
                )
                for item in precedence
            )
        ):
            raise ProbeFailure("ERROR_PRECEDENCE")
        profile = validate_result_observation(vector["profile"], True)
        differential = validate_result_observation(vector["differential"], False)
        profile_outcome = text(vector["profile_outcome"])
        if profile_outcome == "candidate_admitted":
            if profile["kind"] not in {"value", "error"}:
                raise ProbeFailure("ADMITTED_RESULT")
        elif profile_outcome == "candidate_rejected":
            if profile["kind"] != "rejected":
                raise ProbeFailure("REJECTED_RESULT")
        else:
            raise ProbeFailure("PROFILE_OUTCOME")
        if profile["kind"] in {"error", "rejected"} and profile["error_id"] not in precedence:
            raise ProbeFailure("ERROR_PRECEDENCE_COVERAGE")
        culture_sensitive = boolean(vector["runtime_culture_sensitive"])
        if (
            not culture_sensitive
            and profile["kind"] == "value"
            and differential["kind"] == "value"
            and profile["result_encoding"] == differential["result_encoding"]
            and profile["value"] != differential["value"]
        ):
            raise ProbeFailure("CANDIDATE_RUNTIME_MISMATCH")
        ids.append(vector_id)
    if ids != sorted(ids) or len(ids) != len(set(ids)):
        raise ProbeFailure("VECTOR_ORDER")
    return raw


def build_indexes(
    runs: list[dict[str, object]],
) -> tuple[list[object], list[object], list[object], list[object], list[object]]:
    if [text(run["culture"]["profile"]) for run in runs] != list(CULTURE_PROFILES):
        raise ProbeFailure("CULTURE_ORDER")
    vectors_by_run = [array(run["vectors"]) for run in runs]
    baseline = [exact_object(value, VECTOR_KEYS) for value in vectors_by_run[0]]
    baseline_ids = [text(value["id"]) for value in baseline]
    for values in vectors_by_run[1:]:
        vectors = [exact_object(value, VECTOR_KEYS) for value in values]
        if [text(value["id"]) for value in vectors] != baseline_ids:
            raise ProbeFailure("CULTURE_VECTOR_CATALOG")
        for first, other in zip(baseline, vectors):
            if semantic_projection(first) != semantic_projection(other):
                raise ProbeFailure("CULTURE_PROFILE_RESULT_DRIFT")
            if not boolean(first["runtime_culture_sensitive"]) and first != other:
                raise ProbeFailure("CULTURE_INVARIANT_RUNTIME_DRIFT")

    vector_index: list[object] = []
    culture_variants: list[object] = []
    operations: dict[str, list[dict[str, object]]] = {}
    families: dict[str, list[dict[str, object]]] = {}
    for ordinal, vector in enumerate(baseline):
        vector_id = text(vector["id"])
        operation = text(vector["operation"])
        family = text(vector["family"])
        semantic_hash = sha256(canonical(semantic_projection(vector)))
        runtime_rows = []
        runtime_payloads = []
        for profile, values in zip(CULTURE_PROFILES, vectors_by_run):
            candidate = exact_object(values[ordinal], VECTOR_KEYS)
            differential = exact_object(candidate["differential"], DIFFERENTIAL_KEYS)
            runtime_payloads.append(differential)
            runtime_rows.append(
                {
                    "culture": profile,
                    "observation_sha256": sha256(canonical(differential)),
                }
            )
        invariant = all(runtime_payloads[0] == value for value in runtime_payloads[1:])
        if not invariant:
            if not boolean(vector["runtime_culture_sensitive"]):
                raise ProbeFailure("UNDECLARED_CULTURE_VARIANT")
            culture_variants.append(
                {
                    "family": family,
                    "operation": operation,
                    "runtime_observations": runtime_rows,
                    "vector_id": vector_id,
                }
            )
        vector_index.append(
            {
                "candidate_observation_sha256": semantic_hash,
                "culture_invariant_runtime": invariant,
                "family": family,
                "operation": operation,
                "profile_outcome": vector["profile_outcome"],
                "runtime_observations": runtime_rows,
                "vector_id": vector_id,
            }
        )
        operations.setdefault(operation, []).append(vector)
        families.setdefault(family, []).append(vector)

    operation_index: list[object] = []
    mutations: list[object] = []
    for operation in sorted(operations):
        rows = operations[operation]
        domains = {text(row["accepted_domain"]) for row in rows}
        precedence_values = {
            tuple(text(item) for item in array(row["error_precedence"])) for row in rows
        }
        row_families = {text(row["family"]) for row in rows}
        if len(domains) != 1 or len(precedence_values) != 1:
            raise ProbeFailure("OPERATION_CONTRACT_DRIFT", operation)
        vector_ids = [text(row["id"]) for row in rows]
        profile_outcomes = sorted({text(row["profile_outcome"]) for row in rows})
        result_encodings = sorted(
            {text(object_value(row["profile"])["result_encoding"]) for row in rows}
        )
        error_ids = sorted(
            {
                text(object_value(row["profile"])["error_id"])
                for row in rows
                if object_value(row["profile"])["error_id"] is not None
            }
        )
        required_observations = {
            failure
            for failure in next(iter(precedence_values))
            if failure.startswith(("exception.", "parse_error.", "source_rejection."))
        }
        if (
            not operation.startswith("precedence.")
            and set(error_ids) != required_observations
        ):
            raise ProbeFailure(
                "OPERATION_ERROR_COVERAGE",
                canonical(
                    {
                        "observed": error_ids,
                        "operation": operation,
                        "required": sorted(required_observations),
                    }
                ),
            )
        operation_families = sorted(row_families)
        operation_index.append(
            {
                "accepted_domain": next(iter(domains)),
                "observed_error_ids": error_ids,
                "error_precedence": list(next(iter(precedence_values))),
                "families": operation_families,
                "operation": operation,
                "possible_failures": list(next(iter(precedence_values))),
                "profile_outcomes": profile_outcomes,
                "result_encodings": result_encodings,
                "vector_ids": vector_ids,
                "vector_ids_sha256": sha256(canonical(vector_ids)),
            }
        )
        first = rows[0]
        mutations.append(
            {
                "candidate_observation_sha256": sha256(canonical(semantic_projection(first))),
                "families": operation_families,
                "mutation_field": "inputs[0]",
                "mutation_id": "CSHARP-03-T01-W07-RUNTIME-INPUT-"
                + operation.upper().replace(".", "-").replace("_", "-"),
                "operation": operation,
                "vector_id": text(first["id"]),
            }
        )

    family_index: list[object] = []
    for family in sorted(families):
        rows = families[family]
        family_vector_ids = [text(row["id"]) for row in rows]
        operation_ids = sorted({text(row["operation"]) for row in rows})
        family_index.append(
            {
                "family": family,
                "operation_ids": operation_ids,
                "vector_ids": family_vector_ids,
                "vector_ids_sha256": sha256(canonical(family_vector_ids)),
            }
        )

    vector_ids_hash = sha256(canonical(baseline_ids))
    operation_ids = [text(object_value(row)["operation"]) for row in operation_index]
    operation_ids_hash = sha256(canonical(operation_ids))
    family_ids = [text(object_value(row)["family"]) for row in family_index]
    family_ids_hash = sha256(canonical(family_ids))
    culture_variant_ids = [text(object_value(row)["vector_id"]) for row in culture_variants]
    culture_variant_hash = sha256(canonical(culture_variant_ids))
    actual = {
        "culture_variant_count": len(culture_variant_ids),
        "culture_variant_ids_sha256": culture_variant_hash,
        "family_ids_sha256": family_ids_hash,
        "operation_count": len(operation_ids),
        "operation_ids_sha256": operation_ids_hash,
        "vector_count": len(baseline_ids),
        "vector_ids_sha256": vector_ids_hash,
    }
    expected = {
        "culture_variant_count": EXPECTED_CULTURE_VARIANT_COUNT,
        "culture_variant_ids_sha256": EXPECTED_CULTURE_VARIANT_IDS_SHA256,
        "family_ids_sha256": EXPECTED_FAMILY_IDS_SHA256,
        "operation_count": EXPECTED_OPERATION_COUNT,
        "operation_ids_sha256": EXPECTED_OPERATION_IDS_SHA256,
        "vector_count": EXPECTED_VECTOR_COUNT,
        "vector_ids_sha256": EXPECTED_VECTOR_IDS_SHA256,
    }
    if actual != expected:
        raise ProbeFailure("CATALOG", canonical({"actual": actual, "expected": expected}))
    if set(family_ids) != EXPECTED_FAMILIES:
        raise ProbeFailure("FAMILY_SET")
    return vector_index, operation_index, family_index, mutations, culture_variants


def normalize(raw_runs: list[bytes], binary_sha256: str) -> dict[str, object]:
    if len(raw_runs) != len(CULTURE_PROFILES):
        raise ProbeFailure("CULTURE_COUNT")
    runs = [
        validate_raw(strict_json(raw, MAX_RAW_OUTPUT_BYTES), profile)
        for raw, profile in zip(raw_runs, CULTURE_PROFILES)
    ]
    vector_index, operation_index, family_index, mutations, variants = build_indexes(runs)
    observations = {
        "culture_runs": runs,
        "schema": OBSERVATION_SCHEMA,
        "work_item": WORK_ITEM,
    }
    observation_bytes = canonical(observations) + b"\n"
    coverage = [
        {"families": families, "requirement": requirement}
        for requirement, families in sorted(COVERAGE.items())
    ]
    return {
        "baseline": {
            "build_inputs": W03_DESCRIPTOR_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "build_inputs_raw_sha256": W03_DESCRIPTOR_SHA256,
            "candidate_inventory": W03_INVENTORY_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "candidate_inventory_raw_sha256": W03_INVENTORY_SHA256,
            "dependency_probe": W06_RESULT_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "dependency_probe_raw_sha256": W06_RESULT_SHA256,
            "source_commit": W06_COMMIT,
            "source_tree": W06_TREE,
        },
        "coverage": coverage,
        "culture_variants": variants,
        "family_index": family_index,
        "measurement": {
            "culture_run_count_per_build": len(CULTURE_PROFILES) * 2,
            "probe_binary_sha256": binary_sha256,
            "raw_observation_sha256": sha256(observation_bytes),
            "raw_observation_size_bytes": len(observation_bytes),
            "runtime_input_mutations": [
                "unlisted_environment.clean",
                "unlisted_environment.hostile",
            ],
        },
        "observations": observations,
        "operation_index": operation_index,
        "probe_input": {
            "compiler_arguments": list(COMPILER_ARGUMENTS),
            "culture_profiles": list(CULTURE_PROFILES),
            "path": PROBE_PATH.relative_to(REPOSITORY_ROOT).as_posix(),
            "raw_sha256": PROBE_SOURCE_SHA256,
            "reference_projection_sha256": active.REFERENCE_HASH,
            "size_bytes": PROBE_SOURCE_SIZE,
            "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
        },
        "schema": FINAL_SCHEMA,
        "upgrade_mutations": mutations,
        "vector_index": vector_index,
        "work_item": WORK_ITEM,
    }


def validate_document(value: object, *, check_live_inputs: bool) -> dict[str, object]:
    document = exact_object(
        value,
        {
            "baseline",
            "coverage",
            "culture_variants",
            "family_index",
            "measurement",
            "observations",
            "operation_index",
            "probe_input",
            "schema",
            "upgrade_mutations",
            "vector_index",
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
        "dependency_probe": "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
        "dependency_probe_raw_sha256": W06_RESULT_SHA256,
        "source_commit": W06_COMMIT,
        "source_tree": W06_TREE,
    }:
        raise ProbeFailure("BASELINE")
    if document["probe_input"] != {
        "compiler_arguments": list(COMPILER_ARGUMENTS),
        "culture_profiles": list(CULTURE_PROFILES),
        "path": "develop/probes/csharp-03/PrimitiveStringNumericCodecProbe.cs",
        "raw_sha256": PROBE_SOURCE_SHA256,
        "reference_projection_sha256": active.REFERENCE_HASH,
        "size_bytes": PROBE_SOURCE_SIZE,
        "toolchain_inputs_sha256": active.TOOLCHAIN_HASH,
    }:
        raise ProbeFailure("PROBE_INPUT")
    observations = exact_object(
        document["observations"], {"culture_runs", "schema", "work_item"}
    )
    if observations["schema"] != OBSERVATION_SCHEMA or observations["work_item"] != WORK_ITEM:
        raise ProbeFailure("OBSERVATION_IDENTITY")
    culture_runs = array(observations["culture_runs"])
    if len(culture_runs) != len(CULTURE_PROFILES):
        raise ProbeFailure("CULTURE_COUNT")
    runs = [
        validate_raw(value, profile)
        for value, profile in zip(culture_runs, CULTURE_PROFILES)
    ]
    vector_index, operation_index, family_index, mutations, variants = build_indexes(runs)
    if document["vector_index"] != vector_index:
        raise ProbeFailure("VECTOR_INDEX")
    if document["operation_index"] != operation_index:
        raise ProbeFailure("OPERATION_INDEX")
    if document["family_index"] != family_index:
        raise ProbeFailure("FAMILY_INDEX")
    if document["upgrade_mutations"] != mutations:
        raise ProbeFailure("UPGRADE_MUTATIONS")
    if document["culture_variants"] != variants:
        raise ProbeFailure("CULTURE_VARIANTS")
    expected_coverage = [
        {"families": families, "requirement": requirement}
        for requirement, families in sorted(COVERAGE.items())
    ]
    if document["coverage"] != expected_coverage:
        raise ProbeFailure("COVERAGE")
    covered = {
        text(family)
        for row in array(document["coverage"])
        for family in array(object_value(row)["families"])
    }
    if covered != EXPECTED_FAMILIES:
        raise ProbeFailure("COVERAGE_CLOSURE")
    measurement = exact_object(
        document["measurement"],
        {
            "culture_run_count_per_build",
            "probe_binary_sha256",
            "raw_observation_sha256",
            "raw_observation_size_bytes",
            "runtime_input_mutations",
        },
    )
    require_sha256(measurement["probe_binary_sha256"])
    observation_bytes = canonical(observations) + b"\n"
    if (
        integer(measurement["culture_run_count_per_build"]) != 6
        or require_sha256(measurement["raw_observation_sha256"]) != sha256(observation_bytes)
        or integer(measurement["raw_observation_size_bytes"]) != len(observation_bytes)
        or measurement["runtime_input_mutations"]
        != ["unlisted_environment.clean", "unlisted_environment.hostile"]
    ):
        raise ProbeFailure("MEASUREMENT")
    if check_live_inputs:
        validate_frozen_inputs()
    return document


def load_result(*, check_live_inputs: bool) -> dict[str, object]:
    data = read_regular(RESULT_PATH, MAX_RESULT_BYTES)
    value = strict_json(data, MAX_RESULT_BYTES)
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
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-t01-w07-") as temporary:
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
        prefix=".runtime-primitive-string-numeric-codec-", dir=RESULT_PATH.parent
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


def find_vector(document: dict[str, object], vector_id: str) -> dict[str, object]:
    observations = object_value(document["observations"])
    first = object_value(array(observations["culture_runs"])[0])
    for value in array(first["vectors"]):
        vector = exact_object(value, VECTOR_KEYS)
        if vector["id"] == vector_id:
            return vector
    raise ProbeFailure("VECTOR_MISSING")


def self_test() -> None:
    document = load_result(check_live_inputs=True)
    operations: set[str] = set()
    for value in array(document["upgrade_mutations"]):
        row = exact_object(
            value,
            {
                "candidate_observation_sha256",
                "families",
                "mutation_field",
                "mutation_id",
                "operation",
                "vector_id",
            },
        )
        vector = find_vector(document, text(row["vector_id"]))
        if sha256(canonical(semantic_projection(vector))) != require_sha256(
            row["candidate_observation_sha256"]
        ):
            raise ProbeFailure("SELF_TEST_BASE_HASH")
        changed_vector = copy.deepcopy(vector)
        inputs = array(changed_vector["inputs"])
        inputs[0] = text(inputs[0]) + "#runtime-input-mutation"
        if sha256(canonical(semantic_projection(changed_vector))) == require_sha256(
            row["candidate_observation_sha256"]
        ):
            raise ProbeFailure("SELF_TEST_INPUT_MUTATION")
        operations.add(text(row["operation"]))
    expected_operations = {
        text(object_value(value)["operation"])
        for value in array(document["operation_index"])
    }
    if operations != expected_operations:
        raise ProbeFailure("SELF_TEST_OPERATIONS")

    for extra in (False, True):
        changed = copy.deepcopy(document)
        culture_runs = array(object_value(changed["observations"])["culture_runs"])
        if extra:
            culture_runs.append(copy.deepcopy(culture_runs[0]))
        else:
            culture_runs.pop()
        try:
            validate_document(changed, check_live_inputs=False)
        except ProbeFailure as failure:
            if failure.code != "CULTURE_COUNT":
                raise ProbeFailure("SELF_TEST_CULTURE_COUNT_CODE") from failure
        else:
            raise ProbeFailure("SELF_TEST_CULTURE_COUNT")

    mutations = (
        ("floating.single.add.v00.v00", "profile.value"),
        ("codec.integer.i32.parse.plus", "profile.error_id"),
        ("string.concat.char_char", "profile.error_id"),
        ("precedence.sidecar.codec_before_rounding", "error_precedence"),
    )
    for vector_id, field in mutations:
        changed = copy.deepcopy(document)
        vector = find_vector(changed, vector_id)
        if field == "profile.value":
            profile = object_value(vector["profile"])
            profile["value"] = text(profile["value"]) + "0"
        elif field == "profile.error_id":
            profile = object_value(vector["profile"])
            profile["error_id"] = text(profile["error_id"]) + ".mutation"
        elif field == "error_precedence":
            array(vector["error_precedence"]).reverse()
        else:
            raise ProbeFailure("SELF_TEST_FIELD")
        try:
            validate_document(changed, check_live_inputs=False)
        except ProbeFailure:
            pass
        else:
            raise ProbeFailure("SELF_TEST_SCHEMA_ACCEPTED", vector_id)


def main(argv: list[str]) -> int:
    try:
        if argv == ["check-record"]:
            load_result(check_live_inputs=True)
        elif argv == ["self-test"]:
            self_test()
        elif argv == ["check"]:
            expected = read_regular(RESULT_PATH, MAX_RESULT_BYTES)
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
