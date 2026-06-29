#!/usr/bin/env python3
"""Generate and check the deterministic MPK release evidence report."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any


REPORT_SCHEMA = "mpk.release.evidence.v0"
MANIFEST_PATH = Path("fixtures/package-manifest/valid/basic-package.json")
LOCK_PATH = Path("fixtures/package-lock/valid/basic-package-lock.json")
DEFAULT_REPORT_PATH = Path("release-report.json")


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
    }


def collect_certificates(repo_root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
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


def run_package_verify_certs(repo_root: Path) -> dict[str, Any]:
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
            path_str(MANIFEST_PATH),
        ],
    )
    return {
        "command": "cargo run --quiet -p mpk-cli -- package verify-certs "
        + path_str(MANIFEST_PATH),
        "verdict": "accepted",
        "stdout": result.stdout.strip(),
    }


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


def checker_summary(report: dict[str, Any] | None) -> dict[str, Any] | None:
    if report is None:
        return None
    return {
        "verdict": report["verdict"],
        "module": report["module"],
        "declaration_count": report.get("declaration_count", 0),
        "axiom_count": report.get("axiom_count", 0),
        "hashes": report["hashes"],
    }


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

    with path.open("r", encoding="utf-8") as handle:
        value = json.load(handle, object_pairs_hook=reject_duplicates)
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
