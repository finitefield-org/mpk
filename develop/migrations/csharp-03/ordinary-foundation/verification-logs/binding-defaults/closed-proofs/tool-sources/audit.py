"""Audit pinned closed-default proof candidates against independent checker reports.

The report directory contains `<source>-{go,rust}.json` and
`<source>-hash-{go,rust}.json` from the exact binaries supplied below. The
status file records their observed process exit codes; this script never
infers an exit code from JSON output alone.
"""

import argparse
import hashlib
import json
from pathlib import Path


CASES = ("binding-vc-lookup", "binding-vc-option", "extra-lookup-nullable")
DOMAIN = b"MPK-MODULE-CERT-0.1\0"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def certificate_hash(data):
    return sha(DOMAIN + data)


def read_json(path):
    return json.loads(path.read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports", required=True, type=Path)
    parser.add_argument("--statuses", required=True, type=Path)
    parser.add_argument("--rust-binary", required=True, type=Path)
    parser.add_argument("--go-binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    pins = Path(__file__).resolve().parents[1]
    foundation = pins.parents[2]
    original_dir = foundation / "binding-defaults"
    original = read_json(original_dir / "certificates.json")
    assert original["conditions"] == len(CASES)
    by_id = {row["id"]: row["metadata"] for row in original["sources"]}
    assert {name for name, meta in by_id.items() if meta["conditions"]} == set(CASES)

    statuses = read_json(args.statuses)
    expected_statuses = {
        f"{name}-{mutation}-{checker}": 1 if mutation == "hash" else 0
        for name in CASES
        for mutation in ("positive", "hash")
        for checker in ("go", "rust")
    }
    assert statuses == expected_statuses, (statuses, expected_statuses)

    checked = []
    for name in CASES:
        metadata_path = pins / f"{name}.json"
        hex_path = pins / f"{name}.hex"
        metadata = read_json(metadata_path)
        certificate = bytes.fromhex(hex_path.read_text())
        old = by_id[name]
        assert metadata.keys() == old.keys() | {"closed_condition_theorems"}
        assert {
            key: value for key, value in metadata.items()
            if key not in ("closed_condition_theorems", "certificate_sha256")
        } == {
            key: value for key, value in old.items() if key != "certificate_sha256"
        }
        assert len(metadata["conditions"]) == len(metadata["closed_condition_theorems"]) == 1
        condition = metadata["conditions"][0]
        assert condition["sequent"]["kind"] == "actual_default"
        assert condition["sequent"]["id"] in metadata["pending_proof_ids"]
        assert metadata["closed_condition_theorems"] == [
            condition["condition_definition"] + ".Proof"
        ]
        assert certificate_hash(certificate) == metadata["certificate_sha256"]
        assert certificate != bytes.fromhex((original_dir / f"{name}.hex").read_text())

        changed = certificate[:-1] + bytes([certificate[-1] ^ 1])
        assert (args.reports / f"{name}.mpcert").read_bytes() == certificate
        assert (args.reports / f"{name}-hash.mpcert").read_bytes() == changed
        go = read_json(args.reports / f"{name}-go.json")
        rust = read_json(args.reports / f"{name}-rust.json")
        go_hash = read_json(args.reports / f"{name}-hash-go.json")
        rust_hash = read_json(args.reports / f"{name}-hash-rust.json")
        assert go["verdict"] == rust["verdict"] == "accepted"
        report = go["report"]
        assert rust["error_code"] is None
        assert rust["hashes"]["certificate"] == metadata["certificate_sha256"]
        assert bytes(report["CertificateHash"]).hex() == metadata["certificate_sha256"]
        assert rust["module"] == report["Module"]
        assert rust["declaration_count"] == report["DeclarationCount"]
        assert rust["axiom_count"] == report["AxiomCount"] == 0
        assert rust["axiom_report"]["summary"]["total_axiom_count"] == 0
        assert rust["hashes"]["export"] == bytes(report["ExportHash"]).hex()
        assert rust["hashes"]["axiom_report"] == bytes(report["AxiomReportHash"]).hex()
        assert go_hash["verdict"] == rust_hash["verdict"] == "rejected"
        assert go_hash["error_kind"] == "hash_mismatch"
        assert rust_hash["error_code"] == "KERNEL_HASH_MISMATCH"
        assert go_hash["certificate"] == certificate_hash(changed)
        assert rust_hash["hashes"]["certificate"] == certificate_hash(changed)
        checked.append({
            "source": name,
            "certificate_bytes": len(certificate),
            "certificate_sha256": metadata["certificate_sha256"],
            "hex_file_sha256": sha(hex_path.read_bytes()),
            "metadata_file_sha256": sha(metadata_path.read_bytes()),
            "closed_condition_theorem": metadata["closed_condition_theorems"][0],
            "original_application_proofs_pending": len(metadata["pending_proof_ids"]),
            "declarations": rust["declaration_count"],
            "axioms": rust["axiom_count"],
            "export_hash": rust["hashes"]["export"],
            "axiom_report_hash": rust["hashes"]["axiom_report"],
            "reports_sha256": {
                f"{mutation}-{checker}": sha((args.reports / filename).read_bytes())
                for mutation, checker, filename in (
                    ("positive", "go", f"{name}-go.json"),
                    ("positive", "rust", f"{name}-rust.json"),
                    ("hash", "go", f"{name}-hash-go.json"),
                    ("hash", "rust", f"{name}-hash-rust.json"),
                )
            },
        })

    result = {
        "status": "passed_scoped_closed_default_proof_candidates",
        "work_item": "CSHARP-03-T06-W09",
        "internal_unit": 4,
        "selection_reason": "The three eligible actual-default Boolean conditions are the entire closed-condition subset of the original 45 binding sources. The checker tests use their exact pinned certificates and one-bit hash mutations; the original 45 definition pins and W06 pending proof IDs remain unchanged.",
        "proof_candidates": checked,
        "unchanged_checker_binaries": {
            "go_sha256": sha(args.go_binary.read_bytes()),
            "rust_sha256": sha(args.rust_binary.read_bytes()),
        },
        "stage_exit_codes": statuses,
        "stage_exit_codes_file_sha256": sha(args.statuses.read_bytes()),
        "audit_source_sha256": sha(Path(__file__).read_bytes()),
        "original_application_proofs_discharged": 0,
        "scope_limit": "These are kernel-checked equalities for already-closed Boolean definitions, not the exact original W06 application theorem types. Ineligible source-use absence, all original W06 proofs, and W09 assembly remain pending.",
        "full_gate": "deferred_to_T06_W12",
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: value for key, value in result.items() if key != "proof_candidates"}))


if __name__ == "__main__":
    main()
