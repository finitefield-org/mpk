"""Compare every complete packed part with the independently checked row union."""
from pathlib import Path
import hashlib
import importlib.util
import json

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("source_entry_row_inspector", ROOT / "inspect-entry.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


def fingerprint(value):
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), sort_keys=True).encode()).hexdigest()


def main():
    inspection = json.loads((ROOT / "row-inspection/result.json").read_bytes())
    assert inspection["status"] == "passed" and inspection["exit_code"] == 0 and inspection["consumed_inputs_unchanged"]
    review = json.loads((ROOT / "row-review.json").read_bytes())
    assert review["status"] == "passed"
    certs = ROOT / "rust-factored/certificates"
    units = []
    for unit in review["units"]:
        inventory = json.loads((certs / (unit["unit"] + "-inventory.json")).read_bytes())
        assert len(inventory["rows"]) == inventory["all_selected_bodies"] == unit["complete_bodies"]
        assert not inventory["single_program_certificate_claim"]
        for row, meta in zip(unit["rows"], inventory["rows"]):
            assert row["certificate"] == seal(Path(row["certificate"]["path"]))
            assert row["metadata"] == seal(Path(row["metadata"]["path"]))
            assert meta == json.loads(Path(row["metadata"]["path"]).read_bytes())
            assert row["function"] == meta["function"] and row["module"] == meta["module"]
        combined, functions, parts = {}, [], []
        expected = unit["expected_named_union"]
        common = {k: v for k, v in expected.items() if not k.startswith("Mpk.Go.Practical.SourceEntry.")}
        for part in inventory["parts"]:
            path = certs / part["file"]
            raw, _, table, term = module.m.decode(path)
            actual = {k: fingerprint(v) for k, v in module.named(table, term).items()}
            for name, digest in actual.items():
                if name in combined:
                    assert combined[name] == digest
                combined[name] = digest
            assert common == {k: v for k, v in actual.items() if not k.startswith("Mpk.Go.Practical.SourceEntry.")}, "every part retains all source/type/U foundation declarations"
            assert {k.rsplit(".", 1)[0] for k in actual if k.startswith("Mpk.Go.Practical.SourceEntry.")} == {f[2] for f in part["functions"]}
            functions.extend(part["functions"])
            rust = json.loads(path.with_suffix(".rust.json").read_bytes())
            assert rust["verdict"] == "accepted" and rust["axiom_count"] == 0
            assert rust["hashes"]["certificate"] == hashlib.sha256(b"MPK-MODULE-CERT-0.1\0" + raw).hexdigest()
            assert part["raw_sha256"] == hashlib.sha256(raw).hexdigest()
            assert part["declarations"] == len(actual) <= 4096
            assert part["terms"] <= 131072 and part["bytes"] == len(raw) <= 2097152
            parts.append({"certificate": seal(path), "rust_report": seal(path.with_suffix(".rust.json")), **part})
        assert combined == expected and len(combined) == inventory["complete_named_declarations"]
        assert functions == [r["function"] + [r["module"]] for r in inventory["rows"]], "every original function exactly once in order"
        snapshots = sorted((certs / "packing").glob(unit["unit"] + "-*.incomplete.mpcert"))
        assert len(snapshots) == inventory["incomplete_packing_states"]
        units.append({"unit": unit["unit"], "complete_bodies": unit["complete_bodies"], "complete_entry_members": unit["complete_entry_members"],
                      "complete_named_union": len(combined), "cases": len(inventory["cases"]), "source_types": len(inventory["source_types"]),
                      "max_inputs": inventory["max_inputs"], "max_results": inventory["max_results"], "parts": parts,
                      "untrusted_incomplete_packing_snapshots": [seal(p) for p in snapshots]})
    record = {"status": "passed", "driver": seal(Path(__file__).resolve()), "independent_rows": seal(ROOT / "row-review.json"),
              "row_inspection": seal(ROOT / "row-inspection/result.json"), "units": units,
              "scope": "Every complete packed part equals the independently reconstructed full row union; each part retains the entire common source/type/U foundation. Entry equations only; no single complete program certificate or source correspondence claim."}
    with (ROOT / "entry-review.json").open("x") as out:
        json.dump(record, out, indent=2)
        out.write("\n")
    print(json.dumps({"status": "passed", "complete_bodies": sum(u["complete_bodies"] for u in units), "complete_parts": sum(len(u["parts"]) for u in units)}))


if __name__ == "__main__":
    main()
