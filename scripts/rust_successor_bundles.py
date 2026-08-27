#!/usr/bin/env python3
"""Build and verify the staging-only successor Rust bundle and corpus."""

from __future__ import annotations

import base64
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


FRONTEND_ID = "frontend.rust.rust2vir.candidate.v1"
FRONTEND_VERSION = "0.1.0-profile-v1-staging"
TOOLCHAIN_ID = "toolchain.rust.nightly-2025-06-01.candidate.v1"
HOST_ID = "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
RUNTIME_ID = "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
PROFILE_ENTRY_SHA256 = "1cee9716bb21d07e07b8bc1de59ecaf83437549a4d595039486312260816f057"
PROFILE_REGISTRY = {
    "schema": "mpk.semantic_profile.registry.v1",
    "id": "mpk.semantic_profile.registry.v1",
    "revision": 2,
    "registry_sha256": "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75",
}
ACTIVE_REGISTRY_TRANSPORT_SHA256 = (
    "56fa7b398bd63fe2cac488063da00de151279cf65ba0d9cbd3c4fc74716c578a"
)
ACTIVE_REGISTRY_SHA256 = (
    "bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98"
)
ACTIVE_POSITIVE_INDEX_SHA256 = (
    "ff67cb90312b2f613e7ef018d60c42ae7e788b85bbda30239085965a6aa8d308"
)
ACTIVE_DRIVER_VECTOR_SHA256 = (
    "c126a970fdd72eaee41d19fd521a7387c075f3e3779a6d3136cbfaf7856ce640"
)
ACTIVE_SUBSET_VECTOR_SHA256 = (
    "bcd95ff8767b7f19b994adc131f8efc2ea683ef95c77ce74e8862fc4613f9181"
)
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-1.0\0"
REQUEST_DOMAIN = b"MPK-RUST-DRIVER-REQUEST-1.0\0"
PAYLOAD_DOMAIN = b"MPK-RUST-DRIVER-PAYLOAD-1.0\0"
VIR_DOMAIN = b"MPK-VIR-1.0\0"
CONTRACT_DOMAIN = b"MPK-CONTRACT-1.0\0"
TOOLCHAIN_CONTENT_HASHES = {
    "native-runtime": "6d8ebe276575c5019abdc97051baf78e166354249eca4d6b65f638c5fb171005",
    "rust-compiler-runtime": "7698b22d00656113340f692fd9212a1494077fd470f924948945e690da401292",
    "rust-target-i686": "8f606996b669eb0f4314309d145d93c6eeaad8b261791584387bcff46ccafb0a",
    "rust-target-x86_64": "d8c45533753e17186cefde3e0830f7b358a8b4c818eb732d8814a31861335a15",
}
TOOLCHAIN_DISTRIBUTION_SHA256 = (
    "86dab73dadd3a3184064e7d7da7e878562eba4cfc4c8a969bc8f44a5e865c90a"
)


class StagingFailure(Exception):
    pass


def repository_root() -> Path:
    return Path(__file__).resolve().parent.parent


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def raw_hash(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def typed_hash(domain: bytes, value: object) -> str:
    return hashlib.sha256(domain + canonical(value)).hexdigest()


def load_module(relative: str, name: str):
    path = repository_root() / relative
    specification = importlib.util.spec_from_file_location(name, path)
    if specification is None or specification.loader is None:
        raise StagingFailure("RUST_SUCCESSOR_MODULE_UNAVAILABLE")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def overlay_module():
    return load_module("scripts/rust_successor_overlay.py", "mpk_rust_successor_overlay")


def build_module():
    return load_module("scripts/rust_build_inputs.py", "mpk_rust_successor_build_inputs")


def checked_json(path: Path, expected_hash: str | None = None) -> dict[str, object]:
    raw = path.read_bytes()
    if expected_hash is not None and raw_hash(raw) != expected_hash:
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE") from error
    if not isinstance(value, dict):
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    return value


def checked_active_registry() -> dict[str, object]:
    value = checked_json(
        repository_root() / "release/bundles/bundle-registry.json",
        ACTIVE_REGISTRY_TRANSPORT_SHA256,
    )
    if (
        value.get("schema") != "mpk.release.bundle_registry.v0"
        or value.get("id") != "mpk.release.registry.v0"
        or value.get("registry_sha256") != ACTIVE_REGISTRY_SHA256
        or canonical(value) + b"\n"
        != (repository_root() / "release/bundles/bundle-registry.json").read_bytes()
    ):
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    return value


def semantic_parameters(target_id: str, pointer_width: int) -> dict[str, object]:
    return {
        "overflow_mode": "checked",
        "panic_mode": "abort",
        "pointer_width": pointer_width,
        "target_id": target_id,
    }


def semantic_context(target_id: str, pointer_width: int) -> dict[str, object]:
    return {
        "profile_registry": PROFILE_REGISTRY,
        "profile_entry_sha256": PROFILE_ENTRY_SHA256,
        "source_language": "rust",
        "semantic_profile": "mpk.rust.checked.v0",
        "semantic_parameters": {
            "schema": "mpk.semantic_parameters.rust_checked.v0",
            "value": semantic_parameters(target_id, pointer_width),
        },
    }


def selection_envelope(value: dict[str, object]) -> dict[str, object]:
    return {"schema": "mpk.selection.rust_function.v0", "value": value}


def profile_envelope(contract_id: str, value: dict[str, object]) -> dict[str, object]:
    return {
        "profile_entry_sha256": PROFILE_ENTRY_SHA256,
        "contract_id": contract_id,
        "value": value,
    }


def rust_release_value(active_toolchain: dict[str, object]) -> dict[str, object]:
    targets = copy.deepcopy(active_toolchain["target_libraries"])
    for target in targets:
        target["content_sha256"] = TOOLCHAIN_CONTENT_HASHES[target["component_name"]]
    return {
        "compiler": active_toolchain["compiler"],
        "execution_host_profile_id": active_toolchain["execution_host_profile_id"],
        "native_runtime": active_toolchain["native_runtime"],
        "target_libraries": targets,
    }


def assemble_candidate(main: bytes, driver: bytes) -> tuple[bytes, bytes]:
    active = checked_active_registry()
    frontends = [
        item
        for item in active["frontend_bundles"]
        if item.get("source_language") == "rust" and item.get("name") == "rust2vir"
    ]
    toolchains = [
        item
        for item in active["toolchain_bundles"]
        if item.get("source_language") == "rust"
    ]
    hosts = [item for item in active["execution_host_profiles"] if item.get("id") == HOST_ID]
    layouts = [
        item for item in active["native_runtime_layout_profiles"] if item.get("id") == RUNTIME_ID
    ]
    if any(len(values) != 1 for values in (frontends, toolchains, hosts, layouts)):
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    active_frontend = frontends[0]
    active_toolchain = toolchains[0]

    main_hash = raw_hash(main)
    driver_hash = raw_hash(driver)
    inventory = {
        "schema": "mpk.release.bundle_inventory.v0",
        "scope": {"kind": "frontend_bundle", "bundle_id": FRONTEND_ID},
        "files": [
            {
                "path": "bin/rust2vir",
                "executable": True,
                "size_bytes": len(main),
                "sha256": main_hash,
            },
            {
                "path": "bin/rust2vir-driver",
                "executable": True,
                "size_bytes": len(driver),
                "sha256": driver_hash,
            },
        ],
    }
    successor_frontend = {
        "schema": "mpk.release.frontend_bundle.v1",
        "bundle_id": FRONTEND_ID,
        "name": "rust2vir",
        "version": FRONTEND_VERSION,
        "profile_contracts": [
            profile_envelope(
                "mpk.profile.frontend.rust_checked.v0",
                {
                    "limit_profile_id": "mpk.vir.limits.v0",
                    "environment_profile_id": "mpk.rust.frontend_environment.v0",
                    "argument_profile_id": "mpk.rust.frontend_arguments.v0",
                },
            )
        ],
        "main": {
            "name": "rust2vir",
            "version": FRONTEND_VERSION,
            "path": "bin/rust2vir",
            "binary_sha256": main_hash,
            "runtime": active_frontend["main"]["runtime"],
        },
        "subordinate_binaries": [
            {
                "name": "rust2vir-driver",
                "version": FRONTEND_VERSION,
                "path": "bin/rust2vir-driver",
                "binary_sha256": driver_hash,
                "runtime": active_frontend["subordinate_binaries"][0]["runtime"],
            }
        ],
        "inventory": inventory,
        "bundle_sha256": typed_hash(CONTENT_DOMAIN, inventory),
    }

    toolchain_inventory = copy.deepcopy(active_toolchain["inventory"])
    toolchain_inventory["scope"]["bundle_id"] = TOOLCHAIN_ID
    components = copy.deepcopy(active_toolchain["components"])
    for component in components:
        if component["kind"] != "content":
            continue
        component["inventory"]["scope"]["bundle_id"] = TOOLCHAIN_ID
        expected = TOOLCHAIN_CONTENT_HASHES[component["name"]]
        if typed_hash(CONTENT_DOMAIN, component["inventory"]) != expected:
            raise StagingFailure("RUST_SUCCESSOR_TOOLCHAIN_IDENTITY")
        component["content_sha256"] = expected
    if typed_hash(CONTENT_DOMAIN, toolchain_inventory) != TOOLCHAIN_DISTRIBUTION_SHA256:
        raise StagingFailure("RUST_SUCCESSOR_TOOLCHAIN_IDENTITY")
    successor_toolchain = {
        "schema": "mpk.release.toolchain_bundle.v1",
        "bundle_id": TOOLCHAIN_ID,
        "execution_host_profile_id": active_toolchain["execution_host_profile_id"],
        "profile_contracts": [
            profile_envelope(
                "mpk.profile.release.rust_checked.v0",
                rust_release_value(active_toolchain),
            )
        ],
        "components": components,
        "inventory": toolchain_inventory,
        "distribution_sha256": TOOLCHAIN_DISTRIBUTION_SHA256,
    }
    tuples = [
        {
            "semantic_context": semantic_context("i686-unknown-linux-gnu", 32),
            "limit_profile_id": "mpk.vir.limits.v0",
            "frontend_bundle_id": FRONTEND_ID,
            "toolchain_bundle_id": TOOLCHAIN_ID,
        },
        {
            "semantic_context": semantic_context("x86_64-unknown-linux-gnu", 64),
            "limit_profile_id": "mpk.vir.limits.v0",
            "frontend_bundle_id": FRONTEND_ID,
            "toolchain_bundle_id": TOOLCHAIN_ID,
        },
    ]
    candidate: dict[str, object] = {
        "schema": "mpk.release.bundle_candidate.v1",
        "profile_registry": PROFILE_REGISTRY,
        "execution_host_profiles": hosts,
        "native_runtime_layout_profiles": layouts,
        "frontend_bundles": [successor_frontend],
        "toolchain_bundles": [successor_toolchain],
        "tuples": tuples,
    }
    registry = copy.deepcopy(candidate)
    registry["schema"] = "mpk.release.bundle_registry.v1"
    registry["id"] = "mpk.release.registry.v1"
    registry["registry_sha256"] = typed_hash(REGISTRY_DOMAIN, registry)
    return canonical(candidate) + b"\n", canonical(registry) + b"\n"


def staged_descriptor_paths() -> tuple[Path, Path]:
    root = repository_root() / "develop/migrations/csharp-02-staging"
    return root / "rust-bundle-candidate.json", root / "rust-bundle-registry.json"


def write_or_check(path: Path, content: bytes, *, update: bool, missing_code: str) -> None:
    if update:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return
    try:
        current = path.read_bytes()
    except OSError as error:
        raise StagingFailure(missing_code) from error
    if current != content:
        raise StagingFailure("RUST_SUCCESSOR_STAGING_STALE")


def run_snapshot(
    materialize,
    arguments: list[str],
    *,
    retained_target: Path | None = None,
    generated_tree: Path | None = None,
) -> subprocess.CompletedProcess[bytes]:
    rust = build_module()
    vector = rust.load_vector()
    descriptor, cache = rust.check_build_inputs()
    rust.require_image(rust.RUNTIME_IMAGE)
    with tempfile.TemporaryDirectory(
        prefix=".rust2vir-successor-", dir=rust.cache_parent()
    ) as temporary:
        snapshot = Path(temporary) / "snapshot"
        snapshot.mkdir()
        materialize(snapshot / "frontend")
        rust.copy_cache_snapshot(cache, snapshot / "cache")
        rust.validate_cache(descriptor, root=snapshot / "cache")
        writable = snapshot / "writable"
        writable.mkdir()
        limits = rust.frozen_launcher_resources(vector)
        with rust.mounted_writable_workspace(
            writable, limits
        ) as paths, rust.delegated_cgroup_parent(limits) as cgroup_boundary:
            container_name = rust.fresh_container_name("rust2vir-successor")
            argv = rust.hermetic_docker_argv(
                vector,
                snapshot,
                paths,
                arguments,
                container_name,
            )
            result = rust.run_created_docker(
                argv,
                container_name=container_name,
                limits=limits,
                writable_paths=paths,
                cgroup_boundary=cgroup_boundary,
            )
            rust.validate_post_run_cargo_home(paths["cargo-home"], vector)
            if result.returncode == 0 and retained_target is not None:
                rust.capture_candidate_outputs(paths["target"], retained_target)
            if result.returncode == 0 and generated_tree is not None:
                source = paths["work"] / "generated"
                if not source.is_dir() or generated_tree.exists():
                    raise StagingFailure("RUST_SUCCESSOR_FIXTURE_GENERATION")
                shutil.copytree(source, generated_tree)
        rust.validate_cache(descriptor, root=cache)
    return result


def require_success(result: subprocess.CompletedProcess[bytes], code: str) -> None:
    if result.returncode != 0:
        sys.stderr.buffer.write(result.stdout[-65_536:])
        sys.stderr.buffer.write(result.stderr[-65_536:])
        raise StagingFailure(code)


def build_binary_pair(destination: Path) -> tuple[bytes, bytes]:
    result = run_snapshot(
        lambda frontend: overlay_module().materialize(repository_root(), frontend),
        [
            "cargo",
            "build",
            "--locked",
            "--offline",
            "--release",
            "--bins",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--jobs",
            "1",
        ],
        retained_target=destination,
    )
    require_success(result, "RUST_SUCCESSOR_BUILD_FAILED")
    rust = build_module()
    rust.normalize_portable_cpp_runtime(None, destination)
    release = destination / "x86_64-unknown-linux-gnu/release"
    return (release / "rust2vir").read_bytes(), (release / "rust2vir-driver").read_bytes()


def build_and_compare(*, update: bool) -> None:
    checked_active_registry()
    rust = build_module()
    with tempfile.TemporaryDirectory(
        prefix=".rust2vir-successor-binaries-", dir=rust.cache_parent()
    ) as temporary:
        root = Path(temporary)
        first_main, first_driver = build_binary_pair(root / "first")
        second_main, second_driver = build_binary_pair(root / "second")
    if first_main != second_main or first_driver != second_driver:
        raise StagingFailure("RUST_SUCCESSOR_BUILD_NONDETERMINISTIC")
    candidate, registry = assemble_candidate(first_main, first_driver)
    candidate_path, registry_path = staged_descriptor_paths()
    write_or_check(
        candidate_path,
        candidate,
        update=update,
        missing_code="RUST_SUCCESSOR_DESCRIPTOR_MISSING",
    )
    write_or_check(
        registry_path,
        registry,
        update=update,
        missing_code="RUST_SUCCESSOR_DESCRIPTOR_MISSING",
    )


def candidate_identities() -> tuple[dict[str, str], dict[str, object], bytes]:
    candidate_path, registry_path = staged_descriptor_paths()
    candidate = checked_json(candidate_path)
    registry_raw = registry_path.read_bytes()
    registry = checked_json(registry_path)
    try:
        frontend = candidate["frontend_bundles"][0]
        toolchain = candidate["toolchain_bundles"][0]
        identities = {
            "frontend_sha256": frontend["main"]["binary_sha256"],
            "driver_sha256": frontend["subordinate_binaries"][0]["binary_sha256"],
            "registry_sha256": registry["registry_sha256"],
        }
    except (IndexError, KeyError, TypeError) as error:
        raise StagingFailure("RUST_SUCCESSOR_DESCRIPTOR_INVALID") from error
    if (
        candidate.get("schema") != "mpk.release.bundle_candidate.v1"
        or frontend.get("bundle_id") != FRONTEND_ID
        or toolchain.get("bundle_id") != TOOLCHAIN_ID
        or toolchain.get("distribution_sha256") != TOOLCHAIN_DISTRIBUTION_SHA256
        or registry.get("schema") != "mpk.release.bundle_registry.v1"
        or registry.get("id") != "mpk.release.registry.v1"
        or canonical(candidate) + b"\n" != candidate_path.read_bytes()
        or canonical(registry) + b"\n" != registry_raw
        or registry["registry_sha256"] != typed_hash(
            REGISTRY_DOMAIN,
            {key: value for key, value in registry.items() if key != "registry_sha256"},
        )
    ):
        raise StagingFailure("RUST_SUCCESSOR_DESCRIPTOR_INVALID")
    return identities, candidate, registry_raw


def transport_record(template: dict[str, object], value: object) -> dict[str, object]:
    result = {
        key: item
        for key, item in template.items()
        if key not in {"utf8_length", "sha256", "base64"}
    }
    data = canonical(value) + b"\n"
    result.update(
        {
            "utf8_length": len(data),
            "sha256": raw_hash(data),
            "base64": base64.b64encode(data).decode("ascii"),
        }
    )
    return result


def migrate_contract(contract: dict[str, object]) -> None:
    parameters = contract.pop("semantic_parameters")
    if contract.pop("semantic_profile") != "mpk.rust.checked.v0":
        raise StagingFailure("RUST_SUCCESSOR_DRIVER_VECTOR")
    contract["semantic_context"] = semantic_context(
        parameters["target_id"], parameters["pointer_width"]
    )
    contract.pop("contract_hash", None)
    contract["contract_hash"] = typed_hash(CONTRACT_DOMAIN, contract)


def migrate_vir(vir: dict[str, object]) -> None:
    if vir.pop("schema") != "mpk.vir.v0" or vir.pop("source_language") != "rust":
        raise StagingFailure("RUST_SUCCESSOR_DRIVER_VECTOR")
    if vir.pop("semantic_profile") != "mpk.rust.checked.v0":
        raise StagingFailure("RUST_SUCCESSOR_DRIVER_VECTOR")
    parameters = vir.pop("semantic_parameters")
    vir["schema"] = "mpk.vir.v1"
    vir["semantic_context"] = semantic_context(
        parameters["target_id"], parameters["pointer_width"]
    )
    for unit in vir["units"]:
        for function in unit["functions"]:
            migrate_contract(function["contracts"])
    vir.pop("vir_hash", None)
    vir["vir_hash"] = typed_hash(VIR_DOMAIN, vir)


def migrate_common_identity(
    root: dict[str, object],
    identities: dict[str, str],
    candidate: dict[str, object],
) -> None:
    if root.pop("source_language") != "rust" or root.pop("semantic_profile") != "mpk.rust.checked.v0":
        raise StagingFailure("RUST_SUCCESSOR_DRIVER_VECTOR")
    parameters = root.pop("semantic_parameters")
    root["semantic_context"] = semantic_context(
        parameters["target_id"], parameters["pointer_width"]
    )
    root["selection"] = selection_envelope(root["selection"])
    root["release_registry"] = {
        "schema": "mpk.release.bundle_registry.v1",
        "id": "mpk.release.registry.v1",
        "registry_sha256": identities["registry_sha256"],
    }
    frontend_descriptor = candidate["frontend_bundles"][0]
    root["frontend"] = {
        "bundle_id": FRONTEND_ID,
        "name": "rust2vir",
        "version": FRONTEND_VERSION,
        "binary_sha256": identities["frontend_sha256"],
        "subordinate_binaries": [
            {
                "name": "rust2vir-driver",
                "version": FRONTEND_VERSION,
                "binary_sha256": identities["driver_sha256"],
            }
        ],
    }
    toolchain = candidate["toolchain_bundles"][0]
    private_components = []
    for component in toolchain["components"]:
        item = {
            "kind": component["kind"],
            "name": component["name"],
            "release": component["release"],
        }
        if component["kind"] == "executable":
            item["binary_sha256"] = component["binary_sha256"]
            if component["name"] == "rustc":
                item["commit_hash"] = "4d08223c054cf5a56d9761ca925fd46ffebe7115"
        else:
            item["content_sha256"] = component["content_sha256"]
        private_components.append(item)
    root["toolchain"] = {
        "bundle_id": TOOLCHAIN_ID,
        "distribution_sha256": TOOLCHAIN_DISTRIBUTION_SHA256,
        "components": private_components,
    }
    rustc = next(item for item in private_components if item["name"] == "rustc")
    root["compiler"] = {
        "name": "rustc",
        "release": rustc["release"],
        "commit_hash": rustc["commit_hash"],
        "binary_sha256": rustc["binary_sha256"],
        "target": parameters["target_id"],
    }
    if frontend_descriptor["main"]["binary_sha256"] != identities["frontend_sha256"]:
        raise StagingFailure("RUST_SUCCESSOR_DRIVER_VECTOR")


def migrate_driver_vector(
    identities: dict[str, str], candidate: dict[str, object]
) -> bytes:
    path = repository_root() / "develop/specs/vectors/rust-driver-v0.json"
    active_raw = path.read_bytes()
    if raw_hash(active_raw) != ACTIVE_DRIVER_VECTOR_SHA256:
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    vector = json.loads(active_raw)
    vector["schema"] = "mpk.rust.driver.conformance.v1"
    vector["owner_test"] = "rust-tools/rust2vir/tests/successor_protocol_identity.rs"
    vector["request_schema"] = "mpk.rust.driver.request.v1"
    vector["output_schema"] = "mpk.rust.driver.v1"
    vector["hash_domains"]["request"] = "MPK-RUST-DRIVER-REQUEST-1.0"
    vector["hash_domains"]["success_payload"] = "MPK-RUST-DRIVER-PAYLOAD-1.0"

    request_fixture = vector["valid_request"]
    request = request_fixture["value"]
    request["schema"] = "mpk.rust.driver.request.v1"
    migrate_common_identity(request, identities, candidate)
    request.pop("request_fingerprint", None)
    request_fixture["fingerprint_preimage_utf8_length"] = len(canonical(request))
    request["request_fingerprint"] = typed_hash(REQUEST_DOMAIN, request)
    request_fixture["transport"] = transport_record(request_fixture["transport"], request)

    common_fields = [
        "argument_profile_id",
        "compiler",
        "environment_profile_id",
        "frontend",
        "input_set_hash",
        "limit_profile",
        "mir_profile_id",
        "release_registry",
        "request_fingerprint",
        "selection",
        "semantic_context",
        "source_inventory_hash",
        "target_allowlist_id",
        "toolchain",
    ]

    def migrate_output(fixture: dict[str, object]) -> None:
        root = fixture["value"]
        root["schema"] = "mpk.rust.driver.v1"
        migrate_common_identity(root, identities, candidate)
        for field in common_fields:
            root[field] = copy.deepcopy(request[field])
        if root.get("status") == "lowered":
            lowering = root["raw_lowering"]
            lowering["schema"] = "mpk.rust.driver.lowering.v1"
            migrate_vir(lowering["vir"])
            raw_map = root["raw_source_map"]
            raw_map["schema"] = "mpk.rust.driver.raw_source_map.v1"
            raw_map["source_ir_schema"] = "mpk.vir.v1"
            raw_map["source_ir_hash"] = lowering["vir"]["vir_hash"]
            raw_map["semantic_context"] = copy.deepcopy(root["semantic_context"])
            root.pop("payload_hash", None)
            fixture["payload_preimage_utf8_length"] = len(canonical(root))
            root["payload_hash"] = typed_hash(PAYLOAD_DOMAIN, root)
        fixture["transport"] = transport_record(fixture["transport"], root)

    migrate_output(vector["valid_lowered"])
    for fixture in vector["non_success"]:
        migrate_output(fixture)
    for case in vector["request_negative_cases"]:
        if case["id"] == "request.wrong_schema":
            case["mutation"] = "replace schema with mpk.rust.driver.request.v0"
    for case in vector["output_negative_cases"]:
        if case["id"] == "output.wrong_schema":
            case["mutation"] = "replace schema with mpk.rust.driver.v0 and recompute payload hash"
    return canonical(vector)


def artifact(case: dict[str, object], kind: str) -> dict[str, object]:
    matches = [item for item in case["artifacts"] if item["kind"] == kind]
    if len(matches) != 1:
        raise StagingFailure("RUST_SUCCESSOR_FIXTURE_SHAPE")
    return matches[0]


def load_artifact(root: Path, case: dict[str, object], kind: str) -> dict[str, object]:
    descriptor = artifact(case, kind)
    data = (root / descriptor["path"]).read_bytes()
    if len(data) != descriptor["bytes"] or raw_hash(data) != descriptor["sha256"]:
        raise StagingFailure("RUST_SUCCESSOR_FIXTURE_HASH")
    try:
        return json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StagingFailure("RUST_SUCCESSOR_FIXTURE_SHAPE") from error


def vir_projection(value: dict[str, object]) -> list[object]:
    value = copy.deepcopy(value)
    for unit in value["units"]:
        for function in unit["functions"]:
            contract = function["contracts"]
            contract.pop("contract_hash", None)
            contract.pop("semantic_context", None)
            contract.pop("semantic_profile", None)
            contract.pop("semantic_parameters", None)
            for block in function["blocks"]:
                for instruction in block["instructions"]:
                    instruction.pop("contract_hash", None)
    return value["units"]


def required_checks(value: dict[str, object]) -> list[object]:
    checks = []
    for unit in value["units"]:
        for function in unit["functions"]:
            for block in function["blocks"]:
                for instruction in block["instructions"]:
                    checks.extend(instruction["safety_checks"])
    return checks


def finalize_generated_tree(
    generated: Path,
    identities: dict[str, str],
    candidate: dict[str, object],
    registry_raw: bytes,
    driver_vector: bytes,
) -> None:
    root = repository_root()
    active_root = root / "rust-tools/rust2vir/testdata/positive"
    active_index_raw = (active_root / "frontend-index.json").read_bytes()
    if raw_hash(active_index_raw) != ACTIVE_POSITIVE_INDEX_SHA256:
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    subset_path = root / "develop/specs/vectors/rust-subset-v0.json"
    subset_raw = subset_path.read_bytes()
    if raw_hash(subset_raw) != ACTIVE_SUBSET_VECTOR_SHA256:
        raise StagingFailure("RUST_SUCCESSOR_ACTIVE_STATE")
    active_index = json.loads(active_index_raw)
    staged_index_path = generated / "frontend-index.json"
    staged_index = json.loads(staged_index_path.read_bytes())
    active_cases = {case["id"]: case for case in active_index["cases"]}
    if len(active_cases) != 13 or len(staged_index["cases"]) != 13:
        raise StagingFailure("RUST_SUCCESSOR_FIXTURE_SHAPE")
    registry = json.loads(registry_raw)
    release_identity = {
        "schema": registry["schema"],
        "id": registry["id"],
        "registry_sha256": registry["registry_sha256"],
    }
    positive_report = []
    for case in staged_index["cases"]:
        active = active_cases.get(case["id"])
        if active is None:
            raise StagingFailure("RUST_SUCCESSOR_FIXTURE_SHAPE")
        context = semantic_context(case["target_id"], case["pointer_width"])
        raw_selection = {
            "package": "vector",
            "crate": "vector",
            "kind": "lib",
            "function": case["selection"],
        }
        case["semantic_context"] = context
        case["selection"] = selection_envelope(raw_selection)
        case.pop("semantic_profile", None)
        case["source_root"] = (
            f"rust-tools/rust2vir/testdata/positive/{case['fixture']}/source"
        )
        staged_vir = load_artifact(generated, case, "vir")
        active_vir = load_artifact(active_root, active, "vir")
        staged_map = load_artifact(generated, case, "source_map")
        active_map = load_artifact(active_root, active, "source_map")
        staged_manifest = load_artifact(generated, case, "source_manifest_frontend")
        active_manifest = load_artifact(active_root, active, "source_manifest_frontend")
        staged_envelope = load_artifact(generated, case, "frontend_envelope")
        private_request = load_artifact(generated, case, "private_request")
        private_result = load_artifact(generated, case, "private_result")
        raw_lowering = load_artifact(generated, case, "raw_lowering")
        raw_map = load_artifact(generated, case, "raw_source_map")
        if (
            staged_envelope.get("schema") != "mpk.frontend.cli.v1"
            or staged_envelope.get("semantic_context") != context
            or staged_envelope.get("selection") != case["selection"]
            or staged_vir.get("schema") != "mpk.vir.v1"
            or staged_vir.get("semantic_context") != context
            or staged_map.get("schema") != "mpk.source_map.v1"
            or staged_map.get("semantic_context") != context
            or staged_manifest.get("schema") != "mpk.source_manifest.v1"
            or staged_manifest.get("semantic_context") != context
            or staged_manifest.get("selection") != case["selection"]
            or private_request.get("schema") != "mpk.rust.driver.request.v1"
            or private_request.get("semantic_context") != context
            or private_request.get("selection") != case["selection"]
            or private_result.get("schema") != "mpk.rust.driver.v1"
            or raw_lowering.get("schema") != "mpk.rust.driver.lowering.v1"
            or raw_map.get("schema") != "mpk.rust.driver.raw_source_map.v1"
            or raw_map.get("semantic_context") != context
        ):
            raise StagingFailure("RUST_SUCCESSOR_FIXTURE_IDENTITY")
        source_equal = vir_projection(staged_vir) == vir_projection(active_vir)
        checks_equal = required_checks(staged_vir) == required_checks(active_vir)
        target_equal = (
            active["target_id"] == case["target_id"]
            and active["pointer_width"] == case["pointer_width"]
            and staged_manifest["target"]["id"] == active_manifest["target"]["id"]
            and staged_manifest["target"]["pointer_width"]
            == active_manifest["target"]["pointer_width"]
        )
        map_equal = staged_map["entries"] == active_map["entries"]
        manifest_equal = all(
            staged_manifest[field] == active_manifest[field]
            for field in ("inputs", "input_set_hash", "units")
        )
        diagnostics_equal = (
            staged_envelope["diagnostics"]
            == json.loads(
                (active_root / artifact(active, "frontend_envelope")["path"]).read_bytes()
            )["diagnostics"]
        )
        if not all(
            (source_equal, checks_equal, target_equal, map_equal, manifest_equal, diagnostics_equal)
        ):
            raise StagingFailure("RUST_SUCCESSOR_SEMANTIC_DIFFERENCE")
        positive_report.append(
            {
                "id": case["id"],
                "source_behavior_equal": source_equal,
                "required_checks_equal": checks_equal,
                "target_behavior_equal": target_equal,
                "source_map_equal": map_equal,
                "manifest_input_intent_equal": manifest_equal,
                "diagnostics_equal": diagnostics_equal,
            }
        )
    subset = json.loads(subset_raw)
    negative_report = [
        {
            "id": case["id"],
            "status": case["expect"]["status"],
            "phase": case["expect"]["phase"],
            "code": case["expect"]["code"],
            "diagnostics_equal": True,
        }
        for case in subset["rejected_cases"]
    ]
    staged_index.update(
        {
            "schema": "mpk.rust.positive_frontend_corpus.v1",
            "semantic_contexts": [
                semantic_context("i686-unknown-linux-gnu", 32),
                semantic_context("x86_64-unknown-linux-gnu", 64),
            ],
            "release_registry": release_identity,
            "private_protocol": {
                "request_schema": "mpk.rust.driver.request.v1",
                "result_schema": "mpk.rust.driver.v1",
                "raw_lowering_schema": "mpk.rust.driver.lowering.v1",
                "raw_source_map_schema": "mpk.rust.driver.raw_source_map.v1",
                "request_hash_domain": "MPK-RUST-DRIVER-REQUEST-1.0",
                "payload_hash_domain": "MPK-RUST-DRIVER-PAYLOAD-1.0",
            },
            "update_command": "./scripts/check-release-bundles.sh --fixture-update rust-successor",
        }
    )
    staged_index_path.write_bytes(canonical(staged_index))
    (generated / "private-driver-v1.json").write_bytes(driver_vector)
    (generated / "negative-diagnostics.json").write_bytes(
        canonical(
            {
                "schema": "mpk.rust.successor_negative_diagnostics.v1",
                "source_schema": subset["schema"],
                "case_count": len(negative_report),
                "cases": negative_report,
            }
        )
    )
    report = {
        "schema": "mpk.rust_successor_semantic_difference.v1",
        "active_artifact_family": "mpk.vir.v0",
        "successor_artifact_family": "mpk.vir.v1",
        "active_private_protocol": "mpk.rust.driver.v0",
        "successor_private_protocol": "mpk.rust.driver.v1",
        "summary": {
            "positive_cases": len(positive_report),
            "negative_cases": len(negative_report),
            "source_behavior_changes": 0,
            "required_check_changes": 0,
            "target_behavior_changes": 0,
            "source_map_changes": 0,
            "manifest_input_intent_changes": 0,
            "diagnostic_changes": 0,
        },
        "positive_cases": positive_report,
        "negative_cases": negative_report,
    }
    (generated / "semantic-difference-report.json").write_bytes(canonical(report))
    _ = identities


def tree_state(root: Path) -> dict[str, bytes]:
    if not root.is_dir():
        raise StagingFailure("RUST_SUCCESSOR_STAGING_MISSING")
    result = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink() or (not path.is_file() and not path.is_dir()):
            raise StagingFailure("RUST_SUCCESSOR_STAGING_INVALID")
        if path.is_file():
            result[path.relative_to(root).as_posix()] = path.read_bytes()
    return result


def publish_tree(generated: Path, *, update: bool) -> None:
    target = repository_root() / "develop/migrations/csharp-02-staging/rust"
    if not update:
        if tree_state(target) != tree_state(generated):
            raise StagingFailure("RUST_SUCCESSOR_STAGING_STALE")
        return
    if target.exists():
        if not target.is_dir() or target.is_symlink():
            raise StagingFailure("RUST_SUCCESSOR_STAGING_INVALID")
        shutil.rmtree(target)
    shutil.copytree(generated, target)


def check_fixtures(*, update: bool) -> None:
    identities, candidate, registry_raw = candidate_identities()
    driver_vector = migrate_driver_vector(identities, candidate)
    rust = build_module()
    with tempfile.TemporaryDirectory(
        prefix=".rust2vir-successor-fixtures-", dir=rust.cache_parent()
    ) as temporary:
        generated = Path(temporary) / "generated"
        result = run_snapshot(
            lambda frontend: overlay_module().materialize(
                repository_root(),
                frontend,
                identities=identities,
                driver_vector=driver_vector,
            ),
            [
                "cargo",
                "test",
                "--locked",
                "--offline",
                "--target",
                "x86_64-unknown-linux-gnu",
                "--jobs",
                "1",
                "--test",
                "successor_positive_corpus",
                "--test",
                "successor_negative_corpus",
                "--test",
                "successor_protocol_identity",
            ],
            generated_tree=generated,
        )
        require_success(result, "RUST_SUCCESSOR_FIXTURE_TEST_FAILED")
        predecessor_result = run_snapshot(
            lambda frontend: overlay_module().materialize(
                repository_root(),
                frontend,
                driver_vector=driver_vector,
                predecessor_only=True,
            ),
            [
                "cargo",
                "test",
                "--locked",
                "--offline",
                "--target",
                "x86_64-unknown-linux-gnu",
                "--jobs",
                "1",
                "--test",
                "predecessor_identity_rejection",
            ],
        )
        require_success(
            predecessor_result,
            "RUST_SUCCESSOR_PREDECESSOR_IDENTITY_TEST_FAILED",
        )
        finalize_generated_tree(
            generated,
            identities,
            candidate,
            registry_raw,
            driver_vector,
        )
        publish_tree(generated, update=update)


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in {
        "check",
        "update",
        "check-fixtures",
        "update-fixtures",
    }:
        print("RUST_SUCCESSOR_USAGE", file=sys.stderr)
        return 64
    try:
        action = sys.argv[1]
        if action in {"check", "update"}:
            build_and_compare(update=action == "update")
        else:
            check_fixtures(update=action == "update-fixtures")
    except (
        KeyError,
        OSError,
        StopIteration,
        TypeError,
        ValueError,
        StagingFailure,
    ):
        print("RUST_SUCCESSOR_FAILED", file=sys.stderr)
        return 65
    print("RUST_SUCCESSOR_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
