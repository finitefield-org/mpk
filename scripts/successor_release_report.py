#!/usr/bin/env python3
"""Generate the deterministic active successor release evidence report."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

import successor_release_bundles


REPORT_SCHEMA = "mpk.release.evidence.v1"
MANIFEST_PATH = Path("fixtures/package-manifest/valid/basic-package.json")
LOCK_PATH = Path("fixtures/package-lock/valid/basic-package-lock.json")
SEMANTIC_REGISTRY_PATH = Path("release/bundles/semantic-profile-registry.json")
BUNDLE_REGISTRY_PATH = Path("release/bundles/bundle-registry.json")
DEFAULT_REPORT_PATH = Path("release-report.json")
ARCHIVED_REPORTS = (
    Path("develop/migrations/archive/csharp-02-final-review.json"),
    Path("develop/migrations/archive/go-successor-semantic-difference-report.json"),
    Path("develop/migrations/archive/rust-successor-semantic-difference-report.json"),
)
ACTIVE_FIXTURES = (
    Path("fixtures/csharp/policy/scan.json"),
    Path("fixtures/csharp/policy/evidence.json"),
    Path("fixtures/csharp/policy/program-certificate.hex"),
    Path("fixtures/csharp/ai/request.json"),
    Path("fixtures/csharp/ai/explanation.json"),
)


class ReleaseReportError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="compare with release-report.json")
    mode.add_argument("--write", action="store_true", help="write release-report.json")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    encoded = encode_report(build_report(repo_root))
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
    manifest = read_json_object(repo_root / MANIFEST_PATH)
    lock = read_json_object(repo_root / LOCK_PATH)
    validate_lock(repo_root, manifest, lock)
    workspace = tomllib.loads((repo_root / "Cargo.toml").read_text(encoding="utf-8"))
    go_mod = read_go_mod(repo_root / "go-tools/mpk-checker-ref/go.mod")
    certificates = collect_certificates(repo_root, manifest)
    semantic, registry, candidates = active_release(repo_root)
    review_reports = archived_reports(repo_root)
    validate_review_reports(review_reports)

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
            "manifest": artifact(repo_root, MANIFEST_PATH),
            "lock": artifact(repo_root, LOCK_PATH),
            "semantic_registry": artifact(repo_root, SEMANTIC_REGISTRY_PATH),
            "bundle_registry": artifact(repo_root, BUNDLE_REGISTRY_PATH),
            "active_fixtures": [artifact(repo_root, path) for path in ACTIVE_FIXTURES],
        },
        "package": {
            "module": manifest["module"],
            "imports": manifest["imports"],
            "policy": manifest["policy"],
            "certificate_count": len(manifest["certificates"]),
        },
        "certificates": certificates,
        "release_gates": release_gates(certificates),
        "successor_release": {
            "status": "active_successor",
            "semantic_registry": {
                "schema": semantic["schema"],
                "id": semantic["id"],
                "revision": semantic["revision"],
                "registry_sha256": semantic["registry_sha256"],
                "profiles": [
                    {
                        "source_language": entry["source_language"],
                        "semantic_profile": entry["semantic_profile"],
                        "entry_sha256": entry["entry_sha256"],
                        "contracts": entry["contracts"],
                    }
                    for entry in semantic["profiles"]
                ],
            },
            "bundle_registry": {
                "schema": registry["schema"],
                "id": registry["id"],
                "registry_sha256": registry["registry_sha256"],
                "frontend_bundle_ids": [
                    entry["bundle_id"] for entry in registry["frontend_bundles"]
                ],
                "toolchain_bundle_ids": [
                    entry["bundle_id"] for entry in registry["toolchain_bundles"]
                ],
                "tuples": registry["tuples"],
            },
            "candidate_projections": [
                {
                    "language": language,
                    "path": f"release/bundles/candidates/{language}.json",
                    "sha256": sha256_file(
                        repo_root / f"release/bundles/candidates/{language}.json"
                    ),
                    "schema": candidate["schema"],
                }
                for language, candidate in sorted(candidates.items())
            ],
            "migration_reports": review_reports,
            "reproduction": [
                "./scripts/build-release-bundles.sh --check successor",
                "./scripts/check-release-bundles.sh --fixture successor",
                "sudo ./scripts/check-csharp-frontend.sh",
                "python3 scripts/generate-release-report.py --check",
            ],
            "certificate_v0_unchanged": True,
            "proof_authority": "certificate_only",
        },
    }


def active_release(
    repo_root: Path,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, dict[str, Any]]]:
    try:
        registry, candidates = successor_release_bundles.validate_release_models()
    except (successor_release_bundles.SuccessorReleaseFailure, OSError, ValueError) as error:
        raise ReleaseReportError("active successor release metadata is invalid") from error
    semantic = read_canonical_object(repo_root / SEMANTIC_REGISTRY_PATH)
    if (
        semantic.get("schema") != "mpk.semantic_profile.registry.v1"
        or semantic.get("id") != "mpk.semantic_profile.registry.v1"
        or semantic.get("revision") != 2
        or {entry.get("source_language") for entry in semantic.get("profiles", [])}
        != {"go", "rust", "csharp"}
    ):
        raise ReleaseReportError("active semantic registry is not the frozen revision-2 set")
    csharp = next(
        entry for entry in semantic["profiles"] if entry["source_language"] == "csharp"
    )
    if set(csharp.get("contracts", {})) != {
        "ai",
        "evidence",
        "frontend",
        "manifest",
        "policy",
        "release",
        "source_map",
        "vc",
        "vir",
    }:
        raise ReleaseReportError("active C# entry does not bind all nine contracts")
    return semantic, registry, candidates


def archived_reports(repo_root: Path) -> list[dict[str, Any]]:
    result = []
    for path in ARCHIVED_REPORTS:
        value = read_json_object(repo_root / path)
        result.append(
            {
                "path": path.as_posix(),
                "sha256": sha256_file(repo_root / path),
                "schema": value.get("schema"),
                "status": value.get("status"),
                "summary": value.get("summary"),
            }
        )
    return result


def validate_review_reports(reports: list[dict[str, Any]]) -> None:
    csharp = reports[0]
    if csharp["status"] != "reviewed_zero_findings":
        raise ReleaseReportError("C# final review is not the zero-finding archive")
    for report in reports[1:]:
        summary = report.get("summary")
        if not isinstance(summary, dict):
            raise ReleaseReportError(f"{report['path']}: missing semantic-difference summary")
        counters = [
            value
            for name, value in summary.items()
            if name.endswith("_changes") and isinstance(value, int)
        ]
        if not counters or any(counters):
            raise ReleaseReportError(f"{report['path']}: nonzero semantic difference")


def collect_certificates(
    repo_root: Path, manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    certificates = []
    for entry in manifest["certificates"]:
        path = Path(entry["path"])
        source_free = run_json(
            repo_root,
            ["cargo", "run", "--quiet", "-p", "mpk-cli", "--", "check", path.as_posix()],
        )
        axiom = run_json(
            repo_root,
            [
                "cargo",
                "run",
                "--quiet",
                "-p",
                "mpk-cli",
                "--",
                "axiom-report",
                path.as_posix(),
            ],
        )
        reference = None
        if manifest["policy"]["require_reference_checker"]:
            reference = run_json(
                repo_root / "go-tools/mpk-checker-ref",
                [
                    "go",
                    "run",
                    "./cmd/mpk-checker-ref",
                    "verify",
                    str((repo_root / path).resolve()),
                ],
            )
        expected = {
            "export": entry["expected_export_hash"],
            "axiom_report": entry["expected_axiom_report_hash"],
            "certificate": entry["expected_certificate_hash"],
        }
        validate_certificate(entry, expected, source_free, axiom, reference)
        certificates.append(
            {
                "path": path.as_posix(),
                "module": entry["module"],
                "expected_hashes": expected,
                "source_free_checker": checker_summary(source_free),
                "reference_checker": checker_summary(reference),
                "axiom_report": {
                    "certificate_hash": axiom["certificate_hash"],
                    "axiom_report_hash": axiom["axiom_report_hash"],
                    "report": axiom["axiom_report"],
                },
                "hash_agreement": {
                    "manifest_matches_source_free": expected == source_free["hashes"],
                    "source_free_matches_axiom_report": source_free["hashes"]["certificate"]
                    == axiom["certificate_hash"]
                    and source_free["hashes"]["axiom_report"] == axiom["axiom_report_hash"],
                    "source_free_matches_reference": reference is None
                    or source_free["hashes"] == reference["hashes"],
                },
            }
        )
    return certificates


def validate_certificate(
    entry: dict[str, Any],
    expected: dict[str, str],
    source_free: dict[str, Any],
    axiom: dict[str, Any],
    reference: dict[str, Any] | None,
) -> None:
    if (
        source_free.get("verdict") != "accepted"
        or source_free.get("module") != entry["module"]
        or source_free.get("hashes") != expected
        or axiom.get("certificate_hash") != expected["certificate"]
        or axiom.get("axiom_report_hash") != expected["axiom_report"]
        or axiom.get("axiom_report") != source_free.get("axiom_report")
    ):
        raise ReleaseReportError(f"{entry['path']}: source-free evidence mismatch")
    if reference is not None and (
        reference.get("verdict") != "accepted"
        or reference.get("module") != entry["module"]
        or reference.get("hashes") != source_free.get("hashes")
        or reference.get("declaration_count") != source_free.get("declaration_count")
    ):
        raise ReleaseReportError(f"{entry['path']}: reference-checker evidence mismatch")


def checker_summary(report: dict[str, Any] | None) -> dict[str, Any] | None:
    if report is None:
        return None
    return {
        key: report[key]
        for key in ("verdict", "module", "hashes", "declaration_count", "axiom_count")
        if key in report
    }


def release_gates(certificates: list[dict[str, Any]]) -> dict[str, Any]:
    source_free = all(item["source_free_checker"]["verdict"] == "accepted" for item in certificates)
    reference = all(
        item["reference_checker"] is None
        or item["reference_checker"]["verdict"] == "accepted"
        for item in certificates
    )
    hashes = all(all(item["hash_agreement"].values()) for item in certificates)
    categories = all(
        axiom_category_count_matches(item["axiom_report"]["report"])
        for item in certificates
    )
    no_external = all(
        item["axiom_report"]["report"]["summary"]["external_axiom_count"] == 0
        for item in certificates
    )
    return {
        "source_free_checker_accepted": source_free,
        "reference_checker_accepted": reference,
        "hash_agreement": hashes,
        "no_external_axioms": no_external,
        "axiom_category_counts_match": categories,
        "passed": source_free and reference and hashes and no_external and categories,
    }


def axiom_category_count_matches(report: dict[str, Any]) -> bool:
    summary = report["summary"]
    names = {
        "CoreAxiom": "core_axiom_count",
        "BuiltinTheoryAxiom": "builtin_theory_axiom_count",
        "GoSemanticsAxiom": "go_semantics_axiom_count",
        "ExternalAxiom": "external_axiom_count",
    }
    counts = {name: 0 for name in names}
    for entry in report["entries"]:
        if entry["category"] not in counts:
            return False
        counts[entry["category"]] += 1
    return all(counts[name] == summary[field] for name, field in names.items()) and sum(
        counts.values()
    ) == summary["total_axiom_count"]


def validate_lock(repo_root: Path, manifest: dict[str, Any], lock: dict[str, Any]) -> None:
    if (
        lock["manifest"]["path"] != MANIFEST_PATH.as_posix()
        or lock["manifest"]["sha256"] != sha256_file(repo_root / MANIFEST_PATH)
        or lock["module"] != manifest["module"]
        or lock["checker_policy"] != manifest["policy"]
    ):
        raise ReleaseReportError(f"{LOCK_PATH}: package lock mismatch")
    expected_imports = [
        {
            "module": entry["module"],
            "export_hash": entry["export_hash"],
            "certificate_hash": entry["certificate_hash"],
        }
        for entry in manifest["imports"]
    ]
    if lock["locked_imports"] != expected_imports:
        raise ReleaseReportError(f"{LOCK_PATH}: locked imports mismatch")


def read_canonical_object(path: Path) -> dict[str, Any]:
    value = read_json_object(path)
    expected = json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8") + b"\n"
    if path.read_bytes() != expected:
        raise ReleaseReportError(f"{path}: noncanonical JSON transport")
    return value


def read_json_object(path: Path) -> dict[str, Any]:
    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise ReleaseReportError(f"{path}: duplicate JSON key: {key}")
            result[key] = value
        return result

    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicates)
    except (FileNotFoundError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ReleaseReportError(f"cannot read {path}") from error
    if not isinstance(value, dict):
        raise ReleaseReportError(f"{path}: expected a JSON object")
    return value


def read_go_mod(path: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[0] in {"module", "go"}:
            result[fields[0]] = fields[1]
    if set(result) != {"module", "go"}:
        raise ReleaseReportError(f"{path}: missing module or Go version")
    return result


def run_json(cwd: Path, command: list[str]) -> dict[str, Any]:
    result = run(cwd, command)
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ReleaseReportError(f"{' '.join(command)} did not emit JSON") from error
    if not isinstance(value, dict):
        raise ReleaseReportError(f"{' '.join(command)} emitted non-object JSON")
    return value


def run(cwd: Path, command: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        cwd=cwd,
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


def artifact(repo_root: Path, path: Path) -> dict[str, str]:
    return {"path": path.as_posix(), "sha256": sha256_file(repo_root / path)}


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def encode_report(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, ensure_ascii=True) + "\n"


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReleaseReportError as error:
        print(f"release report generation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
