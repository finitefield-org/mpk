"""Independent complete row inspection, safe while whole-part checking runs."""
from pathlib import Path
import datetime
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = Path.cwd()


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


def fingerprint(value):
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), sort_keys=True).encode()).hexdigest()


def worker():
    spec = importlib.util.spec_from_file_location("source_entry_independent_inspection", ROOT / "inspect-entry.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    units = []
    for unit in ["pure", "wide", "supplement"]:
        expected = {}
        rows = []
        members = comparisons = previous_types = previous_bodies = shared_proofs = 0
        for path in sorted((ROOT / "rust-factored/certificates/rows").glob(unit + "-*.mpcert")):
            metadata = path.with_suffix(".entry.json")
            meta = json.loads(metadata.read_bytes())
            actual, count = module.inspect(path, meta)
            members += count
            for name, value in actual.items():
                digest = fingerprint(value)
                if name in expected:
                    assert expected[name] == digest
                expected[name] = digest
                comparisons += 1
            old = ROOT / "rust-corrected/certificates/rows" / path.name
            if old.exists():
                assert meta == json.loads(old.with_suffix(".entry.json").read_bytes())
                _, _, table, term = module.m.decode(old)
                for name, value in module.named(table, term).items():
                    assert actual[name][:2] == value[:2], (path.name, name, "complete previous type")
                    previous_types += 1
                    if len(meta["inputs"]) > 1 and name.startswith(meta["module"] + ".PackedArgument"):
                        assert value[0] == 2
                        shared_proofs += 1
                    else:
                        assert actual[name] == value, (path.name, name, "complete previous body/proof")
                        previous_bodies += 1
            rows.append({"certificate": seal(path), "metadata": seal(metadata), "module": meta["module"], "function": meta["function"], "members": count})
        assert rows
        previous_source = 0
        if unit != "supplement":
            old = ROOT.parent / "source-invocations/generation-linked/certificates" / (unit + "-000.mpcert")
            _, _, table, term = module.m.decode(old)
            for name, value in module.named(table, term).items():
                if not name.startswith("Mpk.Go.Practical.SourceInvocation."):
                    assert expected[name] == fingerprint(value)
                    previous_source += 1
        units.append({"unit": unit, "rows": rows, "complete_bodies": len(rows), "complete_entry_members": members,
                      "whole_original_declaration_comparisons": comparisons, "previous_entry_type_comparisons": previous_types,
                      "previous_entry_body_proof_comparisons": previous_bodies, "independently_reconstructed_shared_proofs": shared_proofs,
                      "previous_source_recipe_comparisons": previous_source, "expected_named_union": expected})
    record = {"status": "passed", "units": units, "scope": "Independent complete row type/body/proof reconstruction. Every prior raw theorem type is retained. Shared projection proofs are reconstructed explicitly. Whole-part union/checker acceptance is reported separately."}
    with (ROOT / "row-review.json").open("x") as out:
        json.dump(record, out, indent=2)
        out.write("\n")
    print(json.dumps({"status": "passed", "complete_bodies": sum(u["complete_bodies"] for u in units), "entry_members": sum(u["complete_entry_members"] for u in units)}))


def main():
    out = ROOT / "row-inspection"
    out.mkdir(exist_ok=False)
    files = set(ROOT.glob("*.py"))
    files.update((ROOT / "rust-factored/certificates/rows").glob("*"))
    files.update((ROOT / "rust-corrected/certificates/rows").glob("*"))
    for name in ["crates", "proofs/std", "fixtures", "develop/specs/vectors", "release/bundles", "go-tools/mpk-checker-ref"]:
        files.update(p for p in (REPO / name).rglob("*") if p.is_file())
    files.update([REPO / "Cargo.toml", REPO / "Cargo.lock", ROOT.parent / "source-invocations/structural-oracle.py",
                  ROOT.parent / "source-error-layouts/structural-oracle.py", REPO / "develop/probes/go-practical/carrier_terms.py"])
    files.update((ROOT.parent / "source-invocations/generation-linked/certificates").glob("*.mpcert"))
    files.update(p for p in (ROOT / "project").rglob("*") if p.is_file())
    sources = [seal(p) for p in sorted(files)]
    with tarfile.open(out / "inputs.tar.gz", "x:gz", dereference=True) as archive:
        for p in sorted(files):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    inputs = out / "inputs.json"
    inputs.write_text(json.dumps({"sources": sources, "complete_archive": seal(out / "inputs.tar.gz")}, indent=2) + "\n")
    command = [sys.executable, "-B", str(Path(__file__).resolve()), "--worker"]
    result = {"status": "running", "command": command, "inputs": seal(inputs), "driver": seal(Path(__file__).resolve()),
              "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "expected_formatter_mutation": False,
              "selection_reason": "Inspect all complete emitted source rows independently while the unchanged Rust whole-part verifier remains live; no consumed production source or original input is edited.",
              "full_gate": "Deferred to final T02-W04: ./scripts/check-fast.sh"}
    path = out / "result.json"
    path.write_text(json.dumps(result, indent=2) + "\n")
    start = time.monotonic()
    with (out / "stdout.log").open("xb") as stdout, (out / "stderr.log").open("xb") as stderr:
        code = subprocess.run(command, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = sources == [seal(p) for p in sorted(files)]
    result.update(status="passed" if code == 0 and unchanged else "failed", exit_code=code, consumed_inputs_unchanged=unchanged,
                  stdout=seal(out / "stdout.log"), stderr=seal(out / "stderr.log"), seconds=time.monotonic() - start,
                  finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: result[k] for k in ["status", "exit_code", "seconds", "consumed_inputs_unchanged"]}))
    raise SystemExit(result["status"] != "passed")


if __name__ == "__main__":
    if sys.argv[1:] == ["--worker"]:
        worker()
    else:
        assert not sys.argv[1:]
        main()
