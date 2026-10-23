"""Check every actual emitted stream, retaining failures and source epochs."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import tarfile
import time

ROOT = Path(__file__).resolve().parent
TOOL = ROOT.parent / "execution-histories/reference-build/mpk-checker-ref"


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


def retained_driver(expected):
    for path in sorted(ROOT.glob("*/inputs.json")):
        receipt = json.loads(path.read_bytes())
        if expected not in receipt["sources"]:
            continue
        archive = Path(receipt["complete_archive"]["path"])
        assert receipt["complete_archive"] == seal(archive)
        name = str(Path(expected["path"]).relative_to(Path.cwd()))
        with tarfile.open(archive, "r:gz") as packed:
            member = packed.getmember(name)
            raw = packed.extractfile(member).read()
        assert len(raw) == expected["bytes"] and hashlib.sha256(raw).hexdigest() == expected["sha256"]
        assert member.mode == expected["mode"]
        return
    raise AssertionError("The original executed comparison driver must remain complete")


def main():
    prior = ROOT.parent / "execution-histories"
    build = json.loads((prior / "reference-build/inputs.json").read_bytes())
    originals = [r for r in build["sources"] if "/go-tools/mpk-checker-ref/" in r["path"]]
    current = {str(p.resolve()) for p in (Path.cwd()/"go-tools/mpk-checker-ref").rglob("*") if p.is_file()}
    assert current == {r["path"] for r in originals}
    for original in originals:
        assert seal(Path(original["path"])) == original
    receipt = json.loads((prior/"paired/rust-borrow-fixed/foundation/result.json").read_bytes())
    assert seal(TOOL) == receipt["inputs"]["checker"]
    out = ROOT / "paired"
    out.mkdir(exist_ok=True)
    streams = sorted(ROOT.glob("rust*/certificates/*.mpcert"))
    assert streams, "No actual Rust certificate streams"
    rows = []
    for cert in streams:
        epoch = cert.parent.parent.name
        target = out / epoch / cert.stem
        report = cert.with_suffix(".rust.json")
        before = {"certificate": seal(cert), "rust_report": seal(report), "checker": seal(TOOL), "driver": seal(Path(__file__))}
        rust = json.loads(report.read_bytes())
        command = [str(TOOL), "verify", str(cert)]
        attempt = 0
        while target.exists() and not (target / "result.json").exists():
            interrupted = json.loads((target / "interrupted-attempt.json").read_bytes())
            assert interrupted["interruption"]["status"] == "verified-interrupted"
            assert interrupted["prelaunch"] == seal(target / "prelaunch.json")
            for retained in interrupted["retained_outputs"]:
                assert retained == seal(Path(retained["path"]))
            attempt += 1
            target = out / epoch / (cert.stem + f"-attempt-{attempt:03}")
        if target.exists():
            row = json.loads((target / "result.json").read_bytes())
            assert row["command"] == command
            for field in ["certificate", "rust_report", "checker"]:
                assert row["inputs"][field] == before[field]
            if row["inputs"]["driver"] != before["driver"]:
                retained_driver(row["inputs"]["driver"])
            for field in ["stdout", "stderr"]:
                assert row[field] == seal(Path(row[field]["path"]))
        else:
            target.mkdir(parents=True, exist_ok=False)
            (target / "prelaunch.json").write_text(json.dumps({"inputs": before, "command": command}, indent=2) + "\n")
            start = time.monotonic()
            with (target / "stdout.log").open("xb") as stdout, (target / "stderr.log").open("xb") as stderr:
                result = subprocess.run(command, stdout=stdout, stderr=stderr, check=False)
            assert before == {"certificate": seal(cert), "rust_report": seal(report), "checker": seal(TOOL), "driver": seal(Path(__file__))}
            row = {"inputs": before, "command": command, "exit_code": result.returncode,
                   "seconds": time.monotonic() - start, "stdout": seal(target / "stdout.log"), "stderr": seal(target / "stderr.log")}
            (target / "result.json").write_text(json.dumps(row, indent=2) + "\n")
        go = json.loads((target / "stdout.log").read_bytes())
        assert go["verdict"] == rust["verdict"], (epoch, cert.name, rust, go)
        if rust["verdict"] == "accepted":
            assert row["exit_code"] == 0
            for field in ["module", "declaration_count", "hashes"]:
                assert go[field] == rust[field], (epoch, cert.name, field)
            # The existing Go verifyOutput uint64 has json omitempty: absence
            # is its zero value. Retain the original report bytes unchanged.
            assert go.get("axiom_count", 0) == rust["axiom_count"] == 0
            assert go["hashes"]["certificate"] == hashlib.sha256(b"MPK-MODULE-CERT-0.1\0" + cert.read_bytes()).hexdigest()
        else:
            assert row["exit_code"] == 1
            assert json.loads(rust["error_detail"])["code"] == "CORE_TYPE_MISMATCH"
            # This Go CLI exposes the core category and the existing human
            # detail, not Rust's nested JSON. These exact forms come from the
            # CoreCheckTypeMismatch sites in the retained compiled Go source.
            assert go["error_kind"] == "core_check"
            assert re.fullmatch(r"term [0-9]+ inferred [a-z_]+ but expected [a-z_]+|lambda domain mismatch", go["error_detail"])
        rows.append({"epoch": epoch, "certificate": seal(cert), "verdict": go["verdict"], "result": seal(target / "result.json")})
    summary = {"status": "passed", "same_bytes_both_full_checkers": True, "streams": rows,
               "accepted": sum(row["verdict"] == "accepted" for row in rows),
               "rejected": sum(row["verdict"] == "rejected" for row in rows),
               "scope": "Checker agreement on all actual successful/failed emitted source epochs. A failed Rust epoch is retained as failed, never promoted by this comparison. No Go source correspondence or complete foundation claim."}
    (out / "comparison.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({key: summary[key] for key in ["status", "accepted", "rejected"]}))


if __name__ == "__main__":
    main()
