#!/usr/bin/env python3
"""Generate and check the deterministic MPK release evidence report."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import stat
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any


REPORT_SCHEMA = "mpk.release.evidence.v0"
RUST_BUILD_INPUT_DOMAIN = b"MPK-RUST-BUILD-INPUTS-0.1\0"
RELEASE_REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-0.1\0"
MANIFEST_PATH = Path("fixtures/package-manifest/valid/basic-package.json")
LOCK_PATH = Path("fixtures/package-lock/valid/basic-package-lock.json")
RUST_PAYMENT_POLICY_MANIFEST_PATH = Path(
    "examples/rust-payment-policy/artifacts/package-manifest.json"
)
RUST_PAYMENT_POLICY_EVIDENCE_PATH = Path(
    "examples/rust-payment-policy/artifacts/evidence.json"
)
RUST_PAYMENT_POLICY_CERTIFICATE_PATH = Path(
    "examples/rust-payment-policy/artifacts/program.mpcert"
)
RUST_RELEASE_POLICY_PATH = Path("fixtures/policy-profiles/rust-release-policy.json")
RUST_BUNDLE_REGISTRY_PATH = Path("release/bundles/bundle-registry.json")
RUST_BUILD_INPUTS_PATH = Path("release/build-inputs/rust/build-inputs.json")
RUST_POSITIVE_CORPUS_MANIFEST_PATH = Path("fixtures/rust-basic/manifest.json")
RUST_FINAL_REVIEW_PATH = Path("develop/migrations/rust-frontend-final-review.json")
RUST_PAYMENT_POLICY_ARTIFACT_MANIFEST_PATH = Path(
    "examples/rust-payment-policy/artifacts/manifest.json"
)
RUST_PAYMENT_POLICY_VIR_PATH = Path("examples/rust-payment-policy/artifacts/vir.json")
RUST_PAYMENT_POLICY_SOURCE_MAP_PATH = Path(
    "examples/rust-payment-policy/artifacts/source-map.json"
)
RUST_PAYMENT_POLICY_FRONTEND_MANIFEST_PATH = Path(
    "examples/rust-payment-policy/artifacts/source-manifest.frontend.json"
)
RUST_PAYMENT_POLICY_CERTIFICATE_MANIFEST_PATH = Path(
    "examples/rust-payment-policy/artifacts/source-manifest.certificate.json"
)
RUST_PAYMENT_POLICY_VC_PATH = Path("examples/rust-payment-policy/artifacts/vc.json")
RUST_PAYMENT_POLICY_AXIOM_REPORT_PATH = Path(
    "examples/rust-payment-policy/artifacts/axiom-report.json"
)
RUST_PAYMENT_POLICY_CHECKER_REPORTS_PATH = Path(
    "examples/rust-payment-policy/artifacts/checker-reports.json"
)
RUST_PAYMENT_POLICY_FINDINGS_PATH = Path(
    "examples/rust-payment-policy/artifacts/findings.json"
)
DEFAULT_REPORT_PATH = Path("release-report.json")
RUST_PAYMENT_POLICY_SELECTION = {
    "package": "payment-policy",
    "crate": "payment_policy",
    "kind": "lib",
    "function": "payment_policy::approved_reserve_cents",
}
RUST_PAYMENT_POLICY_RELEASE = {
    "source_language": "rust",
    "semantic_profile": "mpk.rust.checked.v0",
    "strategy_profile": "payment-policy-rust-alpha",
    "checker_profile": "mvp-strict",
    "axiom_profile": "mvp-theory",
}
RUST_PAYMENT_POLICY_PARAMETERS = {
    "overflow_mode": "checked",
    "panic_mode": "abort",
    "pointer_width": 64,
    "target_id": "x86_64-unknown-linux-gnu",
}
RUST_FRONTEND_BUNDLE_ID = "frontend.rust.rust2vir.candidate.v0"
RUST_TOOLCHAIN_BUNDLE_ID = "toolchain.rust.nightly-2025-06-01.candidate.v0"
RUST_PAYMENT_POLICY_TEST = (
    "rust_payment_policy_freezes_dual_checked_and_pending_structural_results"
)
RUST_PAYMENT_POLICY_EVIDENCE_FIELDS = {
    "schema",
    "source_language",
    "semantic_profile",
    "semantic_parameters",
    "strategy_profile",
    "checker_profile",
    "axiom_profile",
    "verification_options",
    "selection",
    "release_registry",
    "frontend",
    "toolchain",
    "limit_profile",
    "verification_limit_profile",
    "input_set_hash",
    "source_ir_schema",
    "source_ir_hash",
    "source_map_hash",
    "frontend_source_manifest_hash",
    "certificate_source_manifest_hash",
    "source_vc_schema",
    "vc_hash",
    "helper_artifacts",
    "properties",
    "trusted_evidence",
    "reproduction_recipes",
}


class ReleaseReportError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="compare against release-report.json")
    mode.add_argument("--write", action="store_true", help="write release-report.json")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    report = build_report(repo_root)
    encoded = encode_report(report)
    report_path = repo_root / DEFAULT_REPORT_PATH

    if args.check:
        try:
            existing = report_path.read_text(encoding="utf-8")
        except FileNotFoundError as error:
            raise ReleaseReportError(f"missing {DEFAULT_REPORT_PATH}") from error
        if existing != encoded:
            raise ReleaseReportError(
                f"{DEFAULT_REPORT_PATH} is stale; rerun scripts/generate-release-report.py --write"
            )
        print(f"release report is current: {DEFAULT_REPORT_PATH}")
        return 0

    if args.write:
        report_path.write_text(encoded, encoding="utf-8")
        print(f"wrote {DEFAULT_REPORT_PATH}")
        return 0

    print(encoded, end="")
    return 0


def build_report(repo_root: Path) -> dict[str, Any]:
    manifest_path = repo_root / MANIFEST_PATH
    lock_path = repo_root / LOCK_PATH
    manifest = read_json_object(manifest_path)
    lock = read_json_object(lock_path)
    validate_lock(manifest, lock, manifest_path, lock_path)

    workspace = tomllib.loads((repo_root / "Cargo.toml").read_text(encoding="utf-8"))
    go_mod = read_go_mod(repo_root / "go-tools/mpk-checker-ref/go.mod")
    certificates = collect_certificates(repo_root, manifest)

    package_verification = run_package_verify_certs(repo_root)
    gates = release_gates(certificates)
    rust_payment_policy = build_rust_payment_policy_report(repo_root)

    return {
        "schema": REPORT_SCHEMA,
        "versions": {
            "mpk_workspace": workspace["workspace"]["package"]["version"],
            "package_manifest": manifest["schema"],
            "package_lock": lock["schema"],
            "go_reference_checker_module": go_mod["module"],
            "go_reference_checker_language": go_mod["go"],
        },
        "artifacts": {
            "manifest": {
                "path": path_str(MANIFEST_PATH),
                "sha256": sha256_file(manifest_path),
            },
            "lock": {
                "path": path_str(LOCK_PATH),
                "sha256": sha256_file(lock_path),
            },
        },
        "package": {
            "module": manifest["module"],
            "imports": manifest["imports"],
            "policy": manifest["policy"],
            "certificate_count": len(manifest["certificates"]),
        },
        "package_verification": package_verification,
        "certificates": certificates,
        "release_gates": gates,
        "rust_payment_policy": rust_payment_policy,
    }


def collect_certificates(
    repo_root: Path, manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    certificates = []
    require_reference = manifest["policy"]["require_reference_checker"]
    for entry in manifest["certificates"]:
        certificate_path = Path(entry["path"])
        source_free = run_json(
            repo_root,
            ["cargo", "run", "--quiet", "-p", "mpk-cli", "--", "check", path_str(certificate_path)],
        )
        axiom_report = run_json(
            repo_root,
            [
                "cargo",
                "run",
                "--quiet",
                "-p",
                "mpk-cli",
                "--",
                "axiom-report",
                path_str(certificate_path),
            ],
        )

        reference = None
        if require_reference:
            reference = run_json(
                repo_root / "go-tools/mpk-checker-ref",
                [
                    "go",
                    "run",
                    "./cmd/mpk-checker-ref",
                    "verify",
                    path_str((repo_root / certificate_path).resolve()),
                ],
            )

        expected_hashes = {
            "export": entry["expected_export_hash"],
            "axiom_report": entry["expected_axiom_report_hash"],
            "certificate": entry["expected_certificate_hash"],
        }
        validate_certificate_evidence(entry, expected_hashes, source_free, axiom_report, reference)

        certificates.append(
            {
                "path": path_str(certificate_path),
                "module": entry["module"],
                "expected_hashes": expected_hashes,
                "source_free_checker": checker_summary(source_free),
                "reference_checker": checker_summary(reference) if reference is not None else None,
                "axiom_report": {
                    "certificate_hash": axiom_report["certificate_hash"],
                    "axiom_report_hash": axiom_report["axiom_report_hash"],
                    "report": axiom_report["axiom_report"],
                },
                "hash_agreement": {
                    "manifest_matches_source_free": expected_hashes == source_free["hashes"],
                    "source_free_matches_axiom_report": source_free["hashes"]["certificate"]
                    == axiom_report["certificate_hash"]
                    and source_free["hashes"]["axiom_report"]
                    == axiom_report["axiom_report_hash"],
                    "source_free_matches_reference": reference is None
                    or source_free["hashes"] == reference["hashes"],
                },
            }
        )

    return certificates


def run_package_verify_certs(
    repo_root: Path, manifest_path: Path = MANIFEST_PATH
) -> dict[str, Any]:
    result = run(
        repo_root,
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "mpk-cli",
            "--",
            "package",
            "verify-certs",
            path_str(manifest_path),
        ],
    )
    return {
        "command": "cargo run --quiet -p mpk-cli -- package verify-certs "
        + path_str(manifest_path),
        "verdict": "accepted",
        "stdout": result.stdout.strip(),
    }


def run_rust_payment_policy_fixture_check(repo_root: Path) -> dict[str, Any]:
    environment = dict(os.environ)
    environment.pop("MPK_UPDATE_RUST_PAYMENT_POLICY", None)
    command = [
        "cargo",
        "test",
        "--quiet",
        "-p",
        "mpk-cli",
        "--test",
        "rust_payment_policy",
        RUST_PAYMENT_POLICY_TEST,
        "--",
        "--exact",
    ]
    result = run(repo_root, command, env=environment)
    if "running 1 test" not in result.stdout or "1 passed; 0 failed" not in result.stdout:
        raise ReleaseReportError(
            "Rust payment-policy fixture check did not run exactly one passing test"
        )
    return {
        "command": " ".join(command),
        "validated": [
            "canonical_policy_evidence",
            "frontend_vc_manifest_linkage",
            "checked_declaration_member_coverage",
            "certificate_empty_import_table",
            "certificate_empty_proof_node_table",
            "certificate_empty_theory_table",
            "dual_checker_agreement",
            "zero_axioms",
        ],
        "verdict": "accepted",
    }


def build_rust_payment_policy_report(repo_root: Path) -> dict[str, Any]:
    manifest_path = repo_root / RUST_PAYMENT_POLICY_MANIFEST_PATH
    evidence_path = repo_root / RUST_PAYMENT_POLICY_EVIDENCE_PATH
    certificate_path = repo_root / RUST_PAYMENT_POLICY_CERTIFICATE_PATH
    release_policy_path = repo_root / RUST_RELEASE_POLICY_PATH
    registry_path = repo_root / RUST_BUNDLE_REGISTRY_PATH

    manifest = read_json_object(manifest_path)
    evidence = read_json_object(evidence_path)
    active_release = read_json_object(release_policy_path)
    registry = read_json_object(registry_path)
    fixture_verification = run_rust_payment_policy_fixture_check(repo_root)
    validate_rust_payment_policy_inputs(
        manifest, evidence, active_release, registry
    )
    run_rust_payment_policy_validation_regressions(
        manifest, evidence, active_release, registry
    )

    certificates = collect_certificates(repo_root, manifest)
    if len(certificates) != 1:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: expected exactly one certificate"
        )
    if Path(certificates[0]["path"]) != RUST_PAYMENT_POLICY_CERTIFICATE_PATH:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: certificate path must be "
            f"{RUST_PAYMENT_POLICY_CERTIFICATE_PATH}"
        )

    package_verification = run_package_verify_certs(
        repo_root, RUST_PAYMENT_POLICY_MANIFEST_PATH
    )
    certificate_gates = release_gates(certificates)
    profile_agreement = rust_profile_agreement(manifest, evidence, active_release)
    evidence_agreement = rust_evidence_agreement(manifest, evidence)
    properties_verified = rust_properties_verified(evidence)
    evidence_theory_table_empty = (
        evidence["trusted_evidence"]["theory_certificates"] == []
    )
    certificate_alpha_shape_validated = fixture_verification["verdict"] == "accepted"
    no_axioms = all(
        certificate["axiom_report"]["report"]["summary"]["total_axiom_count"] == 0
        for certificate in certificates
    )
    passed = (
        certificate_gates["passed"]
        and profile_agreement
        and evidence_agreement
        and properties_verified
        and evidence_theory_table_empty
        and certificate_alpha_shape_validated
        and no_axioms
    )
    if not passed:
        raise ReleaseReportError("Rust payment-policy release gate rejected")

    release_provenance = build_rust_release_provenance(
        repo_root,
        evidence,
        active_release,
        registry,
        certificates,
        certificate_gates,
        fixture_verification,
    )

    return {
        "artifacts": {
            "package_manifest": {
                "path": path_str(RUST_PAYMENT_POLICY_MANIFEST_PATH),
                "sha256": sha256_file(manifest_path),
            },
            "policy_evidence": {
                "path": path_str(RUST_PAYMENT_POLICY_EVIDENCE_PATH),
                "sha256": sha256_file(evidence_path),
            },
            "certificate": {
                "path": path_str(RUST_PAYMENT_POLICY_CERTIFICATE_PATH),
                "sha256": sha256_file(certificate_path),
            },
            "active_release_policy": {
                "path": path_str(RUST_RELEASE_POLICY_PATH),
                "sha256": sha256_file(release_policy_path),
            },
            "bundle_registry": {
                "path": path_str(RUST_BUNDLE_REGISTRY_PATH),
                "sha256": sha256_file(registry_path),
            },
        },
        "package": {
            "module": manifest["module"],
            "imports": manifest["imports"],
            "policy": manifest["policy"],
            "certificate_count": len(manifest["certificates"]),
        },
        "active_release": active_release,
        "evidence": {
            "schema": evidence["schema"],
            "selection": evidence["selection"],
            "profiles": rust_evidence_profiles(evidence),
            "release_registry": evidence["release_registry"],
            "frontend": evidence["frontend"],
            "toolchain": evidence["toolchain"],
            "trusted_evidence": evidence["trusted_evidence"],
        },
        "package_verification": package_verification,
        "fixture_verification": fixture_verification,
        "certificates": certificates,
        "release_provenance": release_provenance,
        "release_gates": {
            **certificate_gates,
            "active_release_profiles_match_evidence": profile_agreement,
            "package_and_evidence_certificate_match": evidence_agreement,
            "all_policy_properties_verified": properties_verified,
            "evidence_theory_certificate_table_empty": evidence_theory_table_empty,
            "certificate_alpha_shape_validated": certificate_alpha_shape_validated,
            "no_axioms": no_axioms,
            "passed": passed,
        },
    }


def build_rust_release_provenance(
    repo_root: Path,
    evidence: dict[str, Any],
    active_release: dict[str, Any],
    registry: dict[str, Any],
    certificates: list[dict[str, Any]],
    certificate_gates: dict[str, Any],
    fixture_verification: dict[str, Any],
) -> dict[str, Any]:
    build_inputs_path = repo_root / RUST_BUILD_INPUTS_PATH
    corpus_manifest_path = repo_root / RUST_POSITIVE_CORPUS_MANIFEST_PATH
    artifact_manifest_path = repo_root / RUST_PAYMENT_POLICY_ARTIFACT_MANIFEST_PATH
    review_path = repo_root / RUST_FINAL_REVIEW_PATH
    build_inputs = read_json_object(build_inputs_path)
    corpus_manifest = read_json_object(corpus_manifest_path)
    artifact_manifest = read_json_object(artifact_manifest_path)
    review = read_json_object(review_path)

    if build_inputs.get("schema") != "mpk.rust.build_inputs.v0":
        raise ReleaseReportError(f"{RUST_BUILD_INPUTS_PATH}: unexpected schema")
    build_inputs_sha256 = require_sha256(
        build_inputs.get("build_inputs_sha256"),
        f"{RUST_BUILD_INPUTS_PATH}: build_inputs_sha256",
    )
    if build_inputs_sha256 != domain_hash_without_field(
        RUST_BUILD_INPUT_DOMAIN, build_inputs, "build_inputs_sha256"
    ):
        raise ReleaseReportError("Rust build-input descriptor self-hash mismatch")
    registry_sha256 = require_sha256(
        registry.get("registry_sha256"),
        f"{RUST_BUNDLE_REGISTRY_PATH}: registry_sha256",
    )
    if registry_sha256 != domain_hash_without_field(
        RELEASE_REGISTRY_DOMAIN, registry, "registry_sha256"
    ):
        raise ReleaseReportError("release bundle registry self-hash mismatch")
    if build_inputs.get("profile_id") != active_release["semantic_profile"]:
        raise ReleaseReportError("Rust build-input semantic profile mismatch")

    rust_frontends = [
        item
        for item in registry.get("frontend_bundles", [])
        if item.get("source_language") == "rust"
    ]
    rust_toolchains = [
        item
        for item in registry.get("toolchain_bundles", [])
        if item.get("source_language") == "rust"
    ]
    rust_tuples = [
        item
        for item in registry.get("tuples", [])
        if item.get("source_language") == "rust"
    ]
    if len(rust_frontends) != 1 or len(rust_toolchains) != 1 or len(rust_tuples) != 2:
        raise ReleaseReportError("Rust release registration must contain one bundle pair and two targets")
    frontend = rust_frontends[0]
    toolchain = rust_toolchains[0]
    if (
        frontend.get("bundle_id") != evidence["frontend"]["bundle_id"]
        or toolchain.get("bundle_id") != evidence["toolchain"]["bundle_id"]
    ):
        raise ReleaseReportError("Rust evidence does not select the registered bundle pair")
    if os.path.lexists(repo_root / "release/bundles/candidates/rust"):
        raise ReleaseReportError("registered Rust release still contains a candidate publication")

    target_libraries = toolchain.get("target_libraries")
    distribution = build_inputs.get("rust_distribution")
    if not isinstance(target_libraries, list) or not isinstance(distribution, dict):
        raise ReleaseReportError("Rust target-library or distribution metadata is malformed")
    target_rows = sorted(
        (
            item.get("target_id"),
            item.get("pointer_width"),
            item.get("component_name"),
            item.get("content_sha256"),
        )
        for item in target_libraries
    )
    tuple_rows = sorted(
        (item.get("target_id"), item.get("pointer_width")) for item in rust_tuples
    )
    expected_targets = [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ]
    if (
        tuple_rows != expected_targets
        or [(row[0], row[1]) for row in target_rows] != expected_targets
        or sorted(distribution.get("targets", []))
        != [row[0] for row in expected_targets]
    ):
        raise ReleaseReportError("Rust registered target closure mismatch")
    for row in rust_tuples:
        if (
            row.get("frontend_bundle_id") != frontend["bundle_id"]
            or row.get("toolchain_bundle_id") != toolchain["bundle_id"]
            or row.get("semantic_profile") != active_release["semantic_profile"]
            or row.get("limit_profile_id") != evidence["limit_profile"]
        ):
            raise ReleaseReportError("Rust registered tuple identity mismatch")

    graphs = build_inputs.get("graphs")
    if not isinstance(graphs, list) or [item.get("id") for item in graphs] != [
        "cargo-fuzz-tool",
        "frontend",
        "fuzz",
    ]:
        raise ReleaseReportError("Rust dependency graph identities are not frozen")
    dependency_graphs = []
    for graph in graphs:
        manifest_identity = graph.get("manifest")
        lock_identity = graph.get("lock")
        packages = graph.get("packages")
        if (
            not isinstance(manifest_identity, dict)
            or not isinstance(lock_identity, dict)
            or not isinstance(packages, list)
            or not isinstance(graph.get("roots"), list)
        ):
            raise ReleaseReportError("Rust dependency graph identity is malformed")
        dependency_graphs.append(
            {
                "id": graph["id"],
                "roots": graph["roots"],
                "manifest_sha256": require_sha256(
                    manifest_identity.get("sha256"), f"Rust graph {graph['id']} manifest"
                ),
                "lock_sha256": require_sha256(
                    lock_identity.get("sha256"), f"Rust graph {graph['id']} lock"
                ),
                "package_count": len(packages),
            }
        )

    cargo_fuzz = build_inputs.get("cargo_fuzz")
    tool_sources = distribution.get("tool_source_digests")
    if (
        not isinstance(cargo_fuzz, dict)
        or cargo_fuzz.get("clean_builds_equal") is not True
        or not isinstance(tool_sources, list)
        or len(tool_sources) != 1
        or tool_sources[0].get("name") != "cargo-fuzz"
        or tool_sources[0].get("version") != cargo_fuzz.get("version")
        or tool_sources[0].get("sha256") != cargo_fuzz.get("archive_sha256")
        or dependency_graphs[0]["roots"] != [f"cargo-fuzz@{cargo_fuzz.get('version')}"]
    ):
        raise ReleaseReportError("cargo-fuzz identity does not match the frozen dependency closure")

    components = build_inputs.get("components")
    licenses = build_inputs.get("licenses")
    if (
        not isinstance(components, list)
        or any(not isinstance(item, dict) for item in components)
        or len(components) != 7
        or not isinstance(licenses, list)
    ):
        raise ReleaseReportError("Rust build-input components or licenses are malformed")
    component_by_name = {
        item.get("name"): item for item in components if isinstance(item, dict)
    }
    if set(component_by_name) != {
        "cargo-home-seed",
        "native-runtime",
        "native-sysroot",
        "notices",
        "tool-sources",
        "toolchain",
        "vendor",
    }:
        raise ReleaseReportError("Rust build-input component closure is incomplete")
    inventoried_paths = {
        row.get("path")
        for component in components
        for row in component.get("files", [])
        if isinstance(component, dict) and isinstance(row, dict)
    }
    license_ids = {
        item.get("id") for item in licenses if isinstance(item, dict)
    }
    if None in license_ids or len(license_ids) != len(licenses):
        raise ReleaseReportError("Rust build-input license identities are malformed")
    for license_row in licenses:
        files = license_row.get("files")
        if not isinstance(files, list) or any(path not in inventoried_paths for path in files):
            raise ReleaseReportError("Rust license/notice file is outside the frozen inventory")
    for component in components:
        if any(ref not in license_ids for ref in component.get("license_refs", [])):
            raise ReleaseReportError("Rust component references an unknown license identity")
        if any(path not in inventoried_paths for path in component.get("notice_refs", [])):
            raise ReleaseReportError("Rust component references an unknown notice file")

    native_component = component_by_name["native-runtime"]
    descriptor_native_files = native_component.get("files")
    registry_native_files = [
        row
        for row in toolchain.get("inventory", {}).get("files", [])
        if row.get("path", "").startswith("native-runtime/")
    ]
    if not isinstance(descriptor_native_files, list):
        raise ReleaseReportError("frozen native-runtime inventory is malformed")
    descriptor_native_by_path = {row.get("path"): row for row in descriptor_native_files}
    registry_native_by_path = {row.get("path"): row for row in registry_native_files}
    descriptor_cpp = descriptor_native_by_path.pop(
        "native-runtime/lib/x86_64-linux-gnu/libstdc++.so.6", None
    )
    registry_cpp = registry_native_by_path.pop(
        "native-runtime/lib/x86_64-linux-gnu/libstdcxx.so.6", None
    )
    if (
        descriptor_native_by_path != registry_native_by_path
        or not isinstance(descriptor_cpp, dict)
        or not isinstance(registry_cpp, dict)
        or descriptor_cpp.get("size_bytes") != registry_cpp.get("size_bytes")
        or descriptor_cpp.get("sha256") == registry_cpp.get("sha256")
        or native_component.get("notice_refs") != ["notices/Ubuntu-Bionic-NOTICE.txt"]
        or toolchain.get("native_runtime", {}).get("layout_profile_id")
        != build_inputs.get("runtime_layout_profile_id")
    ):
        raise ReleaseReportError("registered native runtime differs from frozen build inputs")
    native_runtime_bundle_component = next(
        (item for item in toolchain.get("components", []) if item.get("name") == "native-runtime"),
        None,
    )
    if not isinstance(native_runtime_bundle_component, dict):
        raise ReleaseReportError("registered Rust toolchain omits native-runtime content identity")

    artifact_objects = {
        "vir": read_json_object(repo_root / RUST_PAYMENT_POLICY_VIR_PATH),
        "source_map": read_json_object(repo_root / RUST_PAYMENT_POLICY_SOURCE_MAP_PATH),
        "frontend_manifest": read_json_object(
            repo_root / RUST_PAYMENT_POLICY_FRONTEND_MANIFEST_PATH
        ),
        "certificate_manifest": read_json_object(
            repo_root / RUST_PAYMENT_POLICY_CERTIFICATE_MANIFEST_PATH
        ),
        "vc": read_json_object(repo_root / RUST_PAYMENT_POLICY_VC_PATH),
        "axiom_report": read_json_object(repo_root / RUST_PAYMENT_POLICY_AXIOM_REPORT_PATH),
        "checker_reports": read_json_object(
            repo_root / RUST_PAYMENT_POLICY_CHECKER_REPORTS_PATH
        ),
        "findings": read_json_object(repo_root / RUST_PAYMENT_POLICY_FINDINGS_PATH),
    }
    validate_rust_artifact_inventory(repo_root, artifact_manifest)
    run_rust_artifact_inventory_regression(repo_root, artifact_manifest)
    certificate = certificates[0]
    expected_hashes = certificate["expected_hashes"]
    if (
        artifact_objects["vir"].get("vir_hash") != evidence["source_ir_hash"]
        or artifact_objects["source_map"].get("source_ir_hash") != evidence["source_ir_hash"]
        or artifact_objects["source_map"].get("source_map_hash") != evidence["source_map_hash"]
        or artifact_objects["frontend_manifest"].get("source_manifest_hash")
        != evidence["frontend_source_manifest_hash"]
        or artifact_objects["certificate_manifest"].get("source_manifest_hash")
        != evidence["certificate_source_manifest_hash"]
        or artifact_objects["certificate_manifest"].get("vc_hash") != evidence["vc_hash"]
        or artifact_objects["vc"].get("source_ir_hash") != evidence["source_ir_hash"]
        or artifact_objects["vc"].get("vc_hash") != evidence["vc_hash"]
        or artifact_objects["axiom_report"].get("certificate_hash")
        != expected_hashes["certificate"]
        or artifact_objects["axiom_report"].get("axiom_report_hash")
        != expected_hashes["axiom_report"]
        or artifact_objects["findings"].get("findings") != []
    ):
        raise ReleaseReportError("Rust payment-policy pipeline artifact hash mismatch")
    checker_reports = artifact_objects["checker_reports"]
    fast_report = checker_reports.get("rust_fast_kernel")
    reference_report = checker_reports.get("reference_checker")
    if (
        not isinstance(fast_report, dict)
        or not isinstance(reference_report, dict)
        or fast_report.get("verdict") != "accepted"
        or reference_report.get("verdict") != "accepted"
        or fast_report.get("hashes", {}).get("certificate")
        != expected_hashes["certificate"]
        or reference_report.get("certificate_hash") != expected_hashes["certificate"]
        or fast_report.get("hashes", {}).get("axiom_report")
        != expected_hashes["axiom_report"]
        or reference_report.get("axiom_report_hash") != expected_hashes["axiom_report"]
    ):
        raise ReleaseReportError("Rust payment-policy checker reports disagree")

    generation = corpus_manifest.get("generation")
    corpus_targets = sorted(
        {item.get("target_id") for item in corpus_manifest.get("cases", [])}
    )
    if (
        corpus_manifest.get("schema") != "mpk.rust.positive_corpus.v0"
        or corpus_manifest.get("status") != "reviewed_zero_findings"
        or corpus_manifest.get("findings") != []
        or corpus_manifest.get("deterministic_runs") != 2
        or not isinstance(generation, dict)
        or generation.get("snapshot_compiler_clean_runs") != 2
        or generation.get("downstream_clean_runs") != 2
        or generation.get("byte_identical") is not True
        or corpus_targets != [row[0] for row in expected_targets]
    ):
        raise ReleaseReportError("Rust two-clean-build corpus record is incomplete")

    if (
        set(review) != {"findings", "reviewed_surfaces", "schema", "status", "task", "trust"}
        or review.get("schema") != "mpk.rust_frontend.final_review.v0"
        or review.get("task") != "RUST-07-T05"
        or review.get("status") != "reviewed_zero_findings"
        or review.get("trust") != "untrusted_review_record"
        or review.get("findings") != []
        or not isinstance(review.get("reviewed_surfaces"), list)
        or not review["reviewed_surfaces"]
    ):
        raise ReleaseReportError("Rust final review ledger is not closed")

    path_result = run(
        repo_root,
        ["python3", path_str(Path("scripts/check-artifact-paths.py"))],
    )
    if "artifact path check passed:" not in path_result.stdout:
        raise ReleaseReportError("Rust artifact path gate did not report success")

    pipeline_artifacts = {
        "vir": release_artifact_record(
            repo_root, RUST_PAYMENT_POLICY_VIR_PATH, "vir_hash", evidence["source_ir_hash"]
        ),
        "source_map": release_artifact_record(
            repo_root,
            RUST_PAYMENT_POLICY_SOURCE_MAP_PATH,
            "source_map_hash",
            evidence["source_map_hash"],
        ),
        "frontend_source_manifest": release_artifact_record(
            repo_root,
            RUST_PAYMENT_POLICY_FRONTEND_MANIFEST_PATH,
            "source_manifest_hash",
            evidence["frontend_source_manifest_hash"],
        ),
        "certificate_source_manifest": release_artifact_record(
            repo_root,
            RUST_PAYMENT_POLICY_CERTIFICATE_MANIFEST_PATH,
            "source_manifest_hash",
            evidence["certificate_source_manifest_hash"],
        ),
        "vc": release_artifact_record(
            repo_root, RUST_PAYMENT_POLICY_VC_PATH, "vc_hash", evidence["vc_hash"]
        ),
        "certificate": {
            "path": path_str(RUST_PAYMENT_POLICY_CERTIFICATE_PATH),
            "sha256": sha256_file(repo_root / RUST_PAYMENT_POLICY_CERTIFICATE_PATH),
            "certificate_hash": expected_hashes["certificate"],
        },
        "axiom_report": release_artifact_record(
            repo_root,
            RUST_PAYMENT_POLICY_AXIOM_REPORT_PATH,
            "axiom_report_hash",
            expected_hashes["axiom_report"],
        ),
    }

    return {
        "trust": "untrusted_helper_provenance",
        "build_inputs": {
            "path": path_str(RUST_BUILD_INPUTS_PATH),
            "sha256": sha256_file(build_inputs_path),
            "build_inputs_sha256": build_inputs_sha256,
            "profile_id": build_inputs["profile_id"],
            "recipe_id": build_inputs["recipe_id"],
            "execution_host_profile_id": build_inputs["execution_host_profile_id"],
            "runtime_layout_profile_id": build_inputs["runtime_layout_profile_id"],
            "dependency_graphs": dependency_graphs,
            "cargo_fuzz": {
                key: cargo_fuzz[key]
                for key in (
                    "version",
                    "tag",
                    "commit",
                    "tree",
                    "archive_sha256",
                    "executable_sha256",
                    "clean_builds_equal",
                )
            },
        },
        "registered_release": {
            "registry": evidence["release_registry"],
            "registry_transport_sha256": sha256_file(repo_root / RUST_BUNDLE_REGISTRY_PATH),
            "frontend_bundle_id": frontend["bundle_id"],
            "frontend_bundle_sha256": frontend["bundle_sha256"],
            "toolchain_bundle_id": toolchain["bundle_id"],
            "toolchain_distribution_sha256": toolchain["distribution_sha256"],
            "targets": [
                {
                    "target_id": row[0],
                    "pointer_width": row[1],
                    "component_name": row[2],
                    "content_sha256": row[3],
                }
                for row in target_rows
            ],
            "candidate_publication_path_absent": True,
        },
        "native_runtime_and_notices": {
            "layout_profile_id": toolchain["native_runtime"]["layout_profile_id"],
            "content_sha256": native_runtime_bundle_component["content_sha256"],
            "inventory_file_count": len(descriptor_native_files),
            "license_identity_count": len(licenses),
            "notice_inventory_file_count": len(component_by_name["notices"]["files"]),
            "descriptor_registry_projection_valid": True,
            "portable_cpp_runtime_soname": "libstdcxx.so.6",
            "license_and_notice_references_valid": True,
        },
        "pipeline_artifacts": pipeline_artifacts,
        "gate_results": {
            "registered_bundle_and_target_validation": "accepted",
            "build_input_dependency_and_notice_validation": "accepted",
            "two_clean_build_comparison": {
                "snapshot_compiler_runs": generation["snapshot_compiler_clean_runs"],
                "downstream_runs": generation["downstream_clean_runs"],
                "cargo_fuzz_clean_builds_equal": cargo_fuzz["clean_builds_equal"],
                "byte_identical": generation["byte_identical"],
            },
            "manifest_vir_vc_certificate_hashes_equal": True,
            "checker_agreement": certificate_gates["hash_agreement"]
            and certificate_gates["source_free_checker_accepted"]
            and certificate_gates["reference_checker_accepted"],
            "axiom_report_and_profile_equal": rust_evidence_profiles(evidence)
            == active_release
            and artifact_objects["axiom_report"]["axiom_report_hash"]
            == expected_hashes["axiom_report"],
            "determinism_gate": fixture_verification["verdict"],
            "path_sanitation_gate": "accepted",
            "rust_example_result": "accepted",
            "review_findings": review["findings"],
        },
    }


def validate_rust_artifact_inventory(
    repo_root: Path, artifact_manifest: dict[str, Any]
) -> None:
    if (
        artifact_manifest.get("schema") != "mpk.rust_payment_policy.artifacts.v0"
        or artifact_manifest.get("findings") != []
        or not isinstance(artifact_manifest.get("artifacts"), list)
    ):
        raise ReleaseReportError("Rust payment-policy artifact manifest is not clean")
    artifact_root = RUST_PAYMENT_POLICY_ARTIFACT_MANIFEST_PATH.parent
    artifact_directory = repo_root / artifact_root
    paths: set[str] = set()
    for row in artifact_manifest["artifacts"]:
        if (
            not isinstance(row, dict)
            or set(row) != {"bytes", "path", "sha256"}
            or not isinstance(row.get("path"), str)
            or row["path"] in paths
        ):
            raise ReleaseReportError("Rust payment-policy artifact inventory is malformed")
        paths.add(row["path"])
        relative = Path(row["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ReleaseReportError("Rust payment-policy artifact path is not relative")
        path = repo_root / artifact_root / relative
        if (
            not path.is_file()
            or path.is_symlink()
            or path.stat().st_size != row["bytes"]
            or sha256_file(path) != row["sha256"]
        ):
            raise ReleaseReportError(f"Rust payment-policy artifact inventory drift: {row['path']}")

    actual_paths: set[str] = set()
    for path in artifact_directory.rglob("*"):
        relative = path.relative_to(artifact_directory).as_posix()
        metadata = path.lstat()
        if relative == RUST_PAYMENT_POLICY_ARTIFACT_MANIFEST_PATH.name:
            if path.is_symlink() or not stat.S_ISREG(metadata.st_mode):
                raise ReleaseReportError("Rust payment-policy artifact manifest is not regular")
            continue
        if path.is_symlink() or not stat.S_ISREG(metadata.st_mode):
            raise ReleaseReportError(
                f"Rust payment-policy artifact tree has a non-regular entry: {relative}"
            )
        actual_paths.add(relative)
    if actual_paths != paths:
        missing = sorted(paths - actual_paths)
        unlisted = sorted(actual_paths - paths)
        raise ReleaseReportError(
            "Rust payment-policy artifact inventory is not exhaustive: "
            f"missing={missing}, unlisted={unlisted}"
        )


def run_rust_artifact_inventory_regression(
    repo_root: Path, artifact_manifest: dict[str, Any]
) -> None:
    mutated = copy.deepcopy(artifact_manifest)
    artifacts = mutated.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        raise ReleaseReportError("Rust payment-policy artifact regression has no fixture")
    artifacts.pop()
    try:
        validate_rust_artifact_inventory(repo_root, mutated)
    except ReleaseReportError:
        return
    raise ReleaseReportError(
        "Rust payment-policy artifact regression accepted an unlisted tree entry"
    )


def release_artifact_record(
    repo_root: Path, path: Path, hash_name: str, hash_value: str
) -> dict[str, Any]:
    return {
        "path": path_str(path),
        "sha256": sha256_file(repo_root / path),
        hash_name: hash_value,
    }


def require_sha256(value: Any, context: str) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise ReleaseReportError(f"{context}: expected lowercase SHA-256")
    try:
        decoded = bytes.fromhex(value)
    except ValueError as error:
        raise ReleaseReportError(f"{context}: expected lowercase SHA-256") from error
    if len(decoded) != 32 or value != value.lower():
        raise ReleaseReportError(f"{context}: expected lowercase SHA-256")
    return value


def domain_hash_without_field(
    domain: bytes, value: dict[str, Any], field: str
) -> str:
    payload = dict(value)
    payload.pop(field, None)
    canonical = json.dumps(
        payload,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(domain + canonical).hexdigest()


def validate_rust_payment_policy_inputs(
    manifest: dict[str, Any],
    evidence: dict[str, Any],
    active_release: dict[str, Any],
    registry: dict[str, Any],
) -> None:
    if manifest.get("schema") != "mpk.package.v0":
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: expected schema mpk.package.v0"
        )
    for field in ("module", "imports", "certificates", "policy"):
        if field not in manifest:
            raise ReleaseReportError(
                f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: missing {field}"
            )
    if not isinstance(manifest["imports"], list):
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: imports must be a list"
        )
    if manifest["module"] != "Example.Rust.PaymentPolicy" or manifest["imports"] != []:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: unexpected product module or imports"
        )
    if not isinstance(manifest["certificates"], list) or len(manifest["certificates"]) != 1:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: certificates must contain exactly one entry"
        )
    certificate_entry = manifest["certificates"][0]
    if not isinstance(certificate_entry, dict):
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: certificate entry must be an object"
        )
    if certificate_entry.get("path") != path_str(RUST_PAYMENT_POLICY_CERTIFICATE_PATH):
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: certificate path must be "
            f"{RUST_PAYMENT_POLICY_CERTIFICATE_PATH}"
        )
    policy = manifest["policy"]
    if not isinstance(policy, dict):
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: policy must be an object"
        )
    for field in (
        "checker_profile",
        "allowed_axiom_profiles",
        "require_reference_checker",
        "require_source_free_check",
    ):
        if field not in policy:
            raise ReleaseReportError(
                f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: policy missing {field}"
            )
    if policy["require_reference_checker"] is not True:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: reference checker must be required"
        )
    if policy["require_source_free_check"] is not True:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: source-free check must be required"
        )
    if policy != {
        "checker_profile": "mvp-strict",
        "allowed_axiom_profiles": ["mvp-theory"],
        "require_reference_checker": True,
        "require_source_free_check": True,
    }:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_MANIFEST_PATH}: unexpected product policy"
        )

    if evidence.get("schema") != "mpk.policy.evidence.v1":
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_EVIDENCE_PATH}: expected schema mpk.policy.evidence.v1"
        )
    if set(evidence) != RUST_PAYMENT_POLICY_EVIDENCE_FIELDS:
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_EVIDENCE_PATH}: unexpected evidence fields"
        )
    for field in (
        "source_language",
        "semantic_profile",
        "strategy_profile",
        "checker_profile",
        "axiom_profile",
        "selection",
        "release_registry",
        "frontend",
        "toolchain",
        "properties",
        "trusted_evidence",
    ):
        if field not in evidence:
            raise ReleaseReportError(
                f"{RUST_PAYMENT_POLICY_EVIDENCE_PATH}: missing {field}"
            )
    trusted = evidence["trusted_evidence"]
    if not isinstance(trusted, dict):
        raise ReleaseReportError(
            f"{RUST_PAYMENT_POLICY_EVIDENCE_PATH}: trusted_evidence must be an object"
        )
    for field in (
        "certificates",
        "theory_certificates",
        "axiom_report",
        "checker_verdicts",
    ):
        if field not in trusted:
            raise ReleaseReportError(
                f"{RUST_PAYMENT_POLICY_EVIDENCE_PATH}: trusted_evidence missing {field}"
            )
    if trusted["theory_certificates"] != []:
        raise ReleaseReportError(
            "Rust payment-policy evidence activates the forbidden theory path"
        )

    release_fields = (
        "source_language",
        "semantic_profile",
        "strategy_profile",
        "checker_profile",
        "axiom_profile",
    )
    if set(active_release) != set(release_fields):
        raise ReleaseReportError(
            f"{RUST_RELEASE_POLICY_PATH}: expected exactly {', '.join(release_fields)}"
        )
    if active_release != RUST_PAYMENT_POLICY_RELEASE:
        raise ReleaseReportError(
            f"{RUST_RELEASE_POLICY_PATH}: unexpected Rust payment-policy release tuple"
        )
    validate_rust_payment_policy_release_identity(evidence, registry)
    if not rust_profile_agreement(manifest, evidence, active_release):
        raise ReleaseReportError(
            "Rust payment-policy package, evidence, and active release profiles disagree"
        )
    if not rust_evidence_agreement(manifest, evidence):
        raise ReleaseReportError(
            "Rust payment-policy evidence certificate does not match its package manifest"
        )
    if not rust_properties_verified(evidence):
        raise ReleaseReportError(
            "Rust payment-policy evidence contains an unverified or empty property"
        )


def run_rust_payment_policy_validation_regressions(
    manifest: dict[str, Any],
    evidence: dict[str, Any],
    active_release: dict[str, Any],
    registry: dict[str, Any],
) -> None:
    def expect_evidence_rejected(label: str, mutation: Any) -> None:
        mutated = copy.deepcopy(evidence)
        mutation(mutated)
        try:
            validate_rust_payment_policy_inputs(
                manifest, mutated, active_release, registry
            )
        except ReleaseReportError:
            return
        raise ReleaseReportError(
            f"Rust payment-policy validation regression was accepted: {label}"
        )

    expect_evidence_rejected(
        "unknown evidence field", lambda value: value.__setitem__("unknown", True)
    )
    expect_evidence_rejected(
        "wrong selection",
        lambda value: value["selection"].__setitem__("function", "payment_policy::wrong"),
    )
    expect_evidence_rejected(
        "unregistered release",
        lambda value: value["release_registry"].__setitem__("id", "bogus.registry"),
    )
    expect_evidence_rejected(
        "non-strict evidence",
        lambda value: value["verification_options"].__setitem__("strict", False),
    )
    expect_evidence_rejected(
        "non-program certificate",
        lambda value: value["trusted_evidence"]["certificates"][0].__setitem__(
            "id", "other"
        ),
    )
    expect_evidence_rejected(
        "reversed checker order",
        lambda value: value["trusted_evidence"]["checker_verdicts"].reverse(),
    )
    expect_evidence_rejected(
        "untrusted verified member",
        lambda value: value["properties"][0]["members"][0].__setitem__(
            "evidence", [{"kind": "helper_artifact", "artifact_id": "vc"}]
        ),
    )
    expect_evidence_rejected(
        "activated theory evidence",
        lambda value: value["trusted_evidence"]["theory_certificates"].append({}),
    )


def validate_rust_payment_policy_release_identity(
    evidence: dict[str, Any], registry: dict[str, Any]
) -> None:
    if evidence.get("selection") != RUST_PAYMENT_POLICY_SELECTION:
        raise ReleaseReportError("Rust payment-policy evidence selection mismatch")
    if evidence.get("semantic_parameters") != RUST_PAYMENT_POLICY_PARAMETERS:
        raise ReleaseReportError("Rust payment-policy semantic parameters mismatch")
    if evidence.get("verification_options") != {
        "strict": True,
        "update_fixtures": False,
    }:
        raise ReleaseReportError("Rust payment-policy evidence must be strict and frozen")
    if registry.get("schema") != "mpk.release.bundle_registry.v0":
        raise ReleaseReportError(f"{RUST_BUNDLE_REGISTRY_PATH}: unexpected schema")
    expected_registry = {
        "schema": registry.get("schema"),
        "id": registry.get("id"),
        "registry_sha256": registry.get("registry_sha256"),
    }
    if evidence.get("release_registry") != expected_registry:
        raise ReleaseReportError("Rust payment-policy release registry mismatch")

    tuples = [
        row
        for row in registry.get("tuples", [])
        if row.get("source_language") == "rust"
        and row.get("semantic_profile") == "mpk.rust.checked.v0"
        and row.get("target_id") == "x86_64-unknown-linux-gnu"
        and row.get("pointer_width") == 64
        and row.get("frontend_bundle_id") == RUST_FRONTEND_BUNDLE_ID
        and row.get("toolchain_bundle_id") == RUST_TOOLCHAIN_BUNDLE_ID
    ]
    if len(tuples) != 1 or evidence.get("limit_profile") != tuples[0].get(
        "limit_profile_id"
    ):
        raise ReleaseReportError("Rust payment-policy registered tuple mismatch")

    frontends = [
        row
        for row in registry.get("frontend_bundles", [])
        if row.get("bundle_id") == RUST_FRONTEND_BUNDLE_ID
    ]
    toolchains = [
        row
        for row in registry.get("toolchain_bundles", [])
        if row.get("bundle_id") == RUST_TOOLCHAIN_BUNDLE_ID
    ]
    if len(frontends) != 1 or len(toolchains) != 1:
        raise ReleaseReportError("Rust payment-policy registered bundles are not singleton")
    frontend = frontends[0]
    expected_frontend = {
        "binary_sha256": frontend["main"]["binary_sha256"],
        "bundle_id": frontend["bundle_id"],
        "name": frontend["name"],
        "subordinate_binaries": [
            {
                "binary_sha256": row["binary_sha256"],
                "name": row["name"],
                "version": row["version"],
            }
            for row in frontend["subordinate_binaries"]
        ],
        "version": frontend["version"],
    }
    if evidence.get("frontend") != expected_frontend:
        raise ReleaseReportError("Rust payment-policy frontend identity mismatch")

    toolchain = toolchains[0]
    expected_components = []
    for row in toolchain["components"]:
        component = {
            "kind": row["kind"],
            "name": row["name"],
            "release": row["release"],
        }
        if row["kind"] == "executable":
            component["binary_sha256"] = row["binary_sha256"]
            if row["name"] == "rustc":
                component["commit_hash"] = toolchain["compiler"]["rustc_commit"]
        else:
            component["content_sha256"] = row["content_sha256"]
        expected_components.append(component)
    expected_toolchain = {
        "bundle_id": toolchain["bundle_id"],
        "components": expected_components,
        "distribution_sha256": toolchain["distribution_sha256"],
    }
    if evidence.get("toolchain") != expected_toolchain:
        raise ReleaseReportError("Rust payment-policy toolchain identity mismatch")


def rust_evidence_profiles(evidence: dict[str, Any]) -> dict[str, Any]:
    return {
        "source_language": evidence["source_language"],
        "semantic_profile": evidence["semantic_profile"],
        "strategy_profile": evidence["strategy_profile"],
        "checker_profile": evidence["checker_profile"],
        "axiom_profile": evidence["axiom_profile"],
    }


def rust_profile_agreement(
    manifest: dict[str, Any],
    evidence: dict[str, Any],
    active_release: dict[str, Any],
) -> bool:
    evidence_profiles = rust_evidence_profiles(evidence)
    package_policy = manifest["policy"]
    return (
        evidence_profiles == active_release
        and package_policy["checker_profile"] == active_release["checker_profile"]
        and active_release["axiom_profile"]
        in package_policy["allowed_axiom_profiles"]
    )


def rust_evidence_agreement(
    manifest: dict[str, Any], evidence: dict[str, Any]
) -> bool:
    manifest_entry = manifest["certificates"][0]
    certificates = evidence["trusted_evidence"]["certificates"]
    if not isinstance(certificates, list) or len(certificates) != 1:
        return False
    certificate = certificates[0]
    if not isinstance(certificate, dict):
        return False
    axiom_report = evidence["trusted_evidence"]["axiom_report"]
    if not isinstance(axiom_report, dict):
        return False
    counts = axiom_report.get("category_counts")
    if not isinstance(counts, dict):
        return False
    zero_axioms = (
        axiom_report.get("status") == "checked"
        and axiom_report.get("axiom_report_hash")
        == manifest_entry["expected_axiom_report_hash"]
        and counts
        == {
            "total_axiom_count": 0,
            "core_axiom_count": 0,
            "builtin_theory_axiom_count": 0,
            "go_semantics_axiom_count": 0,
            "external_axiom_count": 0,
        }
    )
    certificate_matches = (
        certificate.get("module") == manifest_entry["module"]
        and certificate.get("certificate_hash")
        == manifest_entry["expected_certificate_hash"]
        and certificate.get("export_hash") == manifest_entry["expected_export_hash"]
        and certificate.get("axiom_report_hash")
        == manifest_entry["expected_axiom_report_hash"]
    )
    certificate_id = certificate.get("id")
    verdicts = evidence["trusted_evidence"]["checker_verdicts"]
    if certificate_id != "program":
        return False
    if not isinstance(verdicts, list) or len(verdicts) != 2:
        return False
    if [verdict.get("checker") for verdict in verdicts if isinstance(verdict, dict)] != [
        "rust_fast_kernel",
        "reference_checker",
    ]:
        return False
    verdict_map = {
        verdict.get("checker"): verdict
        for verdict in verdicts
        if isinstance(verdict, dict)
    }
    checkers_agree = set(verdict_map) == {"rust_fast_kernel", "reference_checker"} and all(
        verdict.get("checker_profile") == evidence["checker_profile"]
        and verdict.get("verdict") == "accepted"
        and verdict.get("certificate_ids") == [certificate_id]
        for verdict in verdict_map.values()
    )
    return zero_axioms and certificate_matches and checkers_agree


def rust_properties_verified(evidence: dict[str, Any]) -> bool:
    properties = evidence["properties"]
    certificates = evidence["trusted_evidence"]["certificates"]
    if (
        not isinstance(properties, list)
        or not properties
        or not isinstance(certificates, list)
        or len(certificates) != 1
        or certificates[0].get("id") != "program"
    ):
        return False
    declarations = certificates[0].get("checked_declarations")
    if not isinstance(declarations, list) or not declarations:
        return False
    declaration_by_member: dict[str, dict[str, Any]] = {}
    declaration_names: dict[str, str] = {}
    selected_members: set[str] = set()
    for declaration in declarations:
        if not isinstance(declaration, dict):
            return False
        name = declaration.get("name")
        declaration_hash = declaration.get("declaration_hash")
        member_ids = declaration.get("member_ids")
        if (
            not isinstance(name, str)
            or not isinstance(declaration_hash, str)
            or not isinstance(member_ids, list)
            or name in declaration_names
        ):
            return False
        declaration_names[name] = declaration_hash
        for member_id in member_ids:
            if not isinstance(member_id, str) or member_id in declaration_by_member:
                return False
            declaration_by_member[member_id] = declaration
            if declaration.get("function_id") == RUST_PAYMENT_POLICY_SELECTION["function"]:
                selected_members.add(member_id)
    for declaration in declarations:
        dependencies = declaration.get("dependencies")
        if not isinstance(dependencies, list):
            return False
        if any(
            not isinstance(dependency, dict)
            or declaration_names.get(dependency.get("name"))
            != dependency.get("declaration_hash")
            for dependency in dependencies
        ):
            return False

    observed_members: set[str] = set()
    property_ids: set[str] = set()
    for prop in properties:
        if (
            not isinstance(prop, dict)
            or prop.get("status") != "mpk_verified"
            or not isinstance(prop.get("id"), str)
            or prop["id"] in property_ids
            or not isinstance(prop.get("members"), list)
            or not prop["members"]
        ):
            return False
        property_ids.add(prop["id"])
        for member in prop["members"]:
            if not isinstance(member, dict):
                return False
            member_id = member.get("member_id")
            declaration = declaration_by_member.get(member_id)
            if (
                not isinstance(member_id, str)
                or declaration is None
                or member_id in observed_members
                or member.get("function_id")
                != RUST_PAYMENT_POLICY_SELECTION["function"]
                or member.get("status") != "mpk_verified"
                or member.get("evidence")
                != [{"kind": "checked_declaration", "certificate_id": "program"}]
                or member.get("group_id") != declaration.get("group_id")
                or member.get("declaration_name") != declaration.get("name")
                or member.get("declaration_hash")
                != declaration.get("declaration_hash")
            ):
                return False
            observed_members.add(member_id)
    return bool(selected_members) and observed_members == selected_members


def validate_certificate_evidence(
    entry: dict[str, Any],
    expected_hashes: dict[str, str],
    source_free: dict[str, Any],
    axiom_report: dict[str, Any],
    reference: dict[str, Any] | None,
) -> None:
    if source_free.get("verdict") != "accepted":
        raise ReleaseReportError(f"{entry['path']}: source-free checker rejected")
    if source_free.get("module") != entry["module"]:
        raise ReleaseReportError(f"{entry['path']}: source-free module mismatch")
    if source_free.get("hashes") != expected_hashes:
        raise ReleaseReportError(f"{entry['path']}: source-free hashes mismatch manifest")
    if axiom_report.get("certificate_hash") != expected_hashes["certificate"]:
        raise ReleaseReportError(f"{entry['path']}: axiom-report certificate hash mismatch")
    if axiom_report.get("axiom_report_hash") != expected_hashes["axiom_report"]:
        raise ReleaseReportError(f"{entry['path']}: axiom-report hash mismatch")
    if axiom_report.get("axiom_report") != source_free.get("axiom_report"):
        raise ReleaseReportError(f"{entry['path']}: axiom-report payload mismatch")

    if reference is not None:
        if reference.get("verdict") != "accepted":
            raise ReleaseReportError(f"{entry['path']}: reference checker rejected")
        if reference.get("module") != entry["module"]:
            raise ReleaseReportError(f"{entry['path']}: reference module mismatch")
        if reference.get("hashes") != source_free.get("hashes"):
            raise ReleaseReportError(f"{entry['path']}: reference hash mismatch")
        if reference.get("declaration_count") != source_free.get("declaration_count"):
            raise ReleaseReportError(f"{entry['path']}: checker declaration count mismatch")


def checker_summary(report: dict[str, Any] | None) -> dict[str, Any] | None:
    if report is None:
        return None
    summary = {
        "verdict": report["verdict"],
        "module": report["module"],
        "hashes": report["hashes"],
    }
    for field in ("declaration_count", "axiom_count"):
        if field in report:
            summary[field] = report[field]
    return summary


def release_gates(certificates: list[dict[str, Any]]) -> dict[str, Any]:
    source_free_accepted = all(
        cert["source_free_checker"]["verdict"] == "accepted" for cert in certificates
    )
    reference_accepted = all(
        cert["reference_checker"] is None or cert["reference_checker"]["verdict"] == "accepted"
        for cert in certificates
    )
    hash_agreement = all(all(cert["hash_agreement"].values()) for cert in certificates)
    no_external_axioms = all(
        cert["axiom_report"]["report"]["summary"]["external_axiom_count"] == 0
        for cert in certificates
    )
    axiom_category_counts_match = all(
        axiom_category_count_matches(cert["axiom_report"]["report"]) for cert in certificates
    )
    passed = (
        source_free_accepted
        and reference_accepted
        and hash_agreement
        and no_external_axioms
        and axiom_category_counts_match
    )
    return {
        "source_free_checker_accepted": source_free_accepted,
        "reference_checker_accepted": reference_accepted,
        "hash_agreement": hash_agreement,
        "no_external_axioms": no_external_axioms,
        "axiom_category_counts_match": axiom_category_counts_match,
        "passed": passed,
    }


def axiom_category_count_matches(report: dict[str, Any]) -> bool:
    summary = report["summary"]
    counts = {
        "CoreAxiom": 0,
        "BuiltinTheoryAxiom": 0,
        "GoSemanticsAxiom": 0,
        "ExternalAxiom": 0,
    }
    for entry in report["entries"]:
        category = entry["category"]
        if category not in counts:
            return False
        counts[category] += 1
    return (
        counts["CoreAxiom"] == summary["core_axiom_count"]
        and counts["BuiltinTheoryAxiom"] == summary["builtin_theory_axiom_count"]
        and counts["GoSemanticsAxiom"] == summary["go_semantics_axiom_count"]
        and counts["ExternalAxiom"] == summary["external_axiom_count"]
        and sum(counts.values()) == summary["total_axiom_count"]
    )


def validate_lock(
    manifest: dict[str, Any],
    lock: dict[str, Any],
    manifest_path: Path,
    lock_path: Path,
) -> None:
    if lock["manifest"]["path"] != path_str(MANIFEST_PATH):
        raise ReleaseReportError(f"{LOCK_PATH}: manifest.path mismatch")
    if lock["manifest"]["sha256"] != sha256_file(manifest_path):
        raise ReleaseReportError(f"{LOCK_PATH}: manifest.sha256 mismatch")
    if lock["module"] != manifest["module"]:
        raise ReleaseReportError(f"{LOCK_PATH}: module mismatch")
    if lock["checker_policy"] != manifest["policy"]:
        raise ReleaseReportError(f"{LOCK_PATH}: checker policy mismatch")
    locked_imports = [
        {
            "module": entry["module"],
            "export_hash": entry["export_hash"],
            "certificate_hash": entry["certificate_hash"],
        }
        for entry in manifest["imports"]
    ]
    if lock["locked_imports"] != locked_imports:
        raise ReleaseReportError(f"{lock_path}: locked imports mismatch")


def read_json_object(path: Path) -> dict[str, Any]:
    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise ReleaseReportError(f"{path}: duplicate JSON key: {key}")
            result[key] = value
        return result

    try:
        with path.open("r", encoding="utf-8") as handle:
            value = json.load(handle, object_pairs_hook=reject_duplicates)
    except FileNotFoundError as error:
        try:
            display_path = path.relative_to(Path(__file__).resolve().parent.parent)
        except ValueError:
            display_path = path
        raise ReleaseReportError(f"missing {display_path}") from error
    if not isinstance(value, dict):
        raise ReleaseReportError(f"{path}: expected JSON object")
    return value


def read_go_mod(path: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[0] in {"module", "go"}:
            result[fields[0]] = fields[1]
    if "module" not in result or "go" not in result:
        raise ReleaseReportError(f"{path}: missing module or go version")
    return result


def run_json(cwd: Path, command: list[str]) -> dict[str, Any]:
    result = run(cwd, command)
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ReleaseReportError(
            f"{' '.join(command)} did not emit JSON: {error}: {result.stdout}"
        ) from error
    if not isinstance(value, dict):
        raise ReleaseReportError(f"{' '.join(command)} emitted non-object JSON")
    return value


def run(
    cwd: Path,
    command: list[str],
    *,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise ReleaseReportError(
            f"{' '.join(command)} failed with exit {result.returncode}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def encode_report(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, ensure_ascii=True) + "\n"


def path_str(path: Path) -> str:
    return path.as_posix()


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReleaseReportError as error:
        print(f"release report generation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
