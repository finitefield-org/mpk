"""Check exact predicate pins and hash mutations with unchanged binaries.

These certificates define predicates; they do not discharge application proofs.
Observed process exits and matching Go/Rust reports are both required.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import time

CASES = (
    "count_fill", "while", "for", "short_circuit", "switch", "is_binding",
    "guard_order", "guard_throw", "total_variable", "index_update",
    "foreach_string", "foreach_string_var", "foreach_array", "foreach_array_var",
    "lookup", "governing_throw", "type", "string_property", "measures",
)


def sha(data):
    return sha256(data).hexdigest()


def cert_hash(data):
    return sha(b"MPK-MODULE-CERT-0.1\0" + data)


def run(args, case, certificate, mutation, checker):
    data = certificate if mutation == "positive" else certificate[:-1] + bytes([certificate[-1] ^ 1])
    stem = f"{case}-{mutation}-{checker}"
    binary = args.go if checker == "go" else args.rust
    candidate = args.reports / f"{stem}.mpcert"
    candidate.write_bytes(data)
    command = [str(binary), str(candidate)] if checker == "go" else [str(binary), "check", str(candidate)]
    started_at = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    report_path = args.reports / f"{stem}.json"
    stderr_path = args.reports / f"{stem}.stderr"
    with report_path.open("wb") as out, stderr_path.open("wb") as err:
        completed = subprocess.run(command, stdin=subprocess.DEVNULL, stdout=out, stderr=err, check=False)
    report = json.loads(report_path.read_bytes())
    expected = cert_hash(data)
    assert completed.returncode == (0 if mutation == "positive" else 1), (stem, completed.returncode)
    assert report["verdict"] == ("accepted" if mutation == "positive" else "rejected"), (stem, report)
    if mutation == "positive":
        if checker == "go":
            assert bytes(report["report"]["CertificateHash"]).hex() == expected
            assert report["report"]["AxiomCount"] == 0
        else:
            assert report["hashes"]["certificate"] == expected
            assert report["axiom_count"] == 0 and report["error_code"] is None
    elif checker == "go":
        assert report["error_kind"] == "hash_mismatch" and report["certificate"] == expected
    else:
        assert report["error_code"] == "KERNEL_HASH_MISMATCH" and report["hashes"]["certificate"] == expected
    return stem, {
        "exit_code": completed.returncode, "certificate_sha256": expected,
        "report_sha256": sha(report_path.read_bytes()), "stderr_sha256": sha(stderr_path.read_bytes()),
        "started_at": started_at, "elapsed_seconds": round(time.monotonic() - start, 3),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pins", type=Path, required=True)
    parser.add_argument("--reports", type=Path, required=True)
    parser.add_argument("--rust", type=Path, required=True)
    parser.add_argument("--go", type=Path, required=True)
    parser.add_argument("--audit-only", action="store_true", help="Reaudit retained reports and observed exits without rerunning checkers")
    args = parser.parse_args()
    args.reports.mkdir(parents=True, exist_ok=True)
    certificates, fixtures = {}, []
    for case in CASES:
        metadata_path = args.pins / f"{case}.json"
        metadata = json.loads(metadata_path.read_bytes())
        data = bytes.fromhex((args.pins / f"{case}.hex").read_text())
        if case != "measures":
            assert cert_hash(data) == metadata["certificate_sha256"]
            assert metadata["application_scope_pending"] is True
        else:
            assert metadata["observations"] == 1800 and len(metadata["definitions"]) == 24
        certificates[case] = data
        fixtures.append({"id": case, "certificate_sha256": cert_hash(data),
                         "bytes_sha256": sha(data), "metadata_sha256": sha(metadata_path.read_bytes())})
    if args.audit_only:
        stages = json.loads((args.reports / "stages.json").read_bytes())
    else:
        stages = {}
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(run, args, case, certificates[case], mutation, checker)
                       for case in CASES for mutation in ("positive", "hash") for checker in ("go", "rust")]
            for future in as_completed(futures):
                stem, row = future.result()
                stages[stem] = row
                (args.reports / "stages.json").write_text(json.dumps(stages, indent=2, sort_keys=True) + "\n")
                print("PASS", stem, row["elapsed_seconds"], flush=True)
    expected_stages = {f"{case}-{mutation}-{checker}" for case in CASES
                       for mutation in ("positive", "hash") for checker in ("go", "rust")}
    assert set(stages) == expected_stages
    for case in CASES:
        for mutation in ("positive", "hash"):
            data = certificates[case]
            if mutation == "hash":
                data = data[:-1] + bytes([data[-1] ^ 1])
            for checker in ("go", "rust"):
                stem = f"{case}-{mutation}-{checker}"
                row = stages[stem]
                assert row["exit_code"] == (0 if mutation == "positive" else 1)
                assert row["certificate_sha256"] == cert_hash(data)
                assert (args.reports / f"{stem}.mpcert").read_bytes() == data
                assert row["report_sha256"] == sha((args.reports / f"{stem}.json").read_bytes())
                assert row["stderr_sha256"] == sha((args.reports / f"{stem}.stderr").read_bytes())
                report = json.loads((args.reports / f"{stem}.json").read_bytes())
                assert report["verdict"] == ("accepted" if mutation == "positive" else "rejected")
                if mutation == "hash" and checker == "go":
                    assert report["error_kind"] == "hash_mismatch" and report["certificate"] == cert_hash(data)
                elif mutation == "hash":
                    assert report["error_code"] == "KERNEL_HASH_MISMATCH" and report["hashes"]["certificate"] == cert_hash(data)
    for case in CASES:
        go = json.loads((args.reports / f"{case}-positive-go.json").read_bytes())["report"]
        rust = json.loads((args.reports / f"{case}-positive-rust.json").read_bytes())
        assert go["Module"] == rust["module"] and go["DeclarationCount"] == rust["declaration_count"]
        assert go["AxiomCount"] == rust["axiom_count"] == 0
        assert rust["error_code"] is None
        assert all(n == 0 for n in rust["axiom_report"]["summary"].values())
        assert all(n == 0 for n in go["AxiomReport"]["Summary"].values())
        for g, r in (("CertificateHash", "certificate"), ("ExportHash", "export"), ("AxiomReportHash", "axiom_report")):
            assert bytes(go[g]).hex() == rust["hashes"][r], (case, g)
        assert rust["hashes"]["certificate"] == cert_hash(certificates[case])
    receipt = {
        "status": "passed", "proof_scope": "predicate definitions only; application proofs pending",
        "case_count": len(CASES), "distinct_certificate_count": len(set(certificates.values())),
        "stage_count": len(stages), "fixtures": fixtures, "stages": stages,
        "binaries": {"rust": {"path": str(args.rust), "sha256": sha(args.rust.read_bytes())},
                     "go": {"path": str(args.go), "sha256": sha(args.go.read_bytes())}},
    }
    assert len(stages) == 4 * len(CASES)
    (args.reports / "verification.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print("COMPLETE", len(stages), flush=True)


if __name__ == "__main__":
    main()
