"""Negative controls for exact historical seals; no production exclusions."""
import copy
import json
from pathlib import Path
import tempfile
from compare import module, ROOT, HERE


def main():
    model = module(ROOT / "scripts/check-no-active-gir.sh", "seal_checked")
    manifest, *_ = model.validate_manifest()
    read = model.read_strict_json; original_validate = model.validate_regular_file
    cases = []
    for kind in ("new_production_path", "wrong_hash", "wrong_size", "wrong_class", "wrong_epoch", "missing", "duplicate", "escape", "old_manifest_schema"):
        value = copy.deepcopy(manifest); row = value["sealed_records"][0]
        if kind == "new_production_path": row["path"] = "crates/mpk-vc/src/vir.rs"
        elif kind == "wrong_hash": row["sha256"] = "0" * 64
        elif kind == "wrong_size": row["size_bytes"] = True
        elif kind == "wrong_class": row["classification"] = "active_source"
        elif kind == "wrong_epoch": row["source_epoch"] = "0" * 40
        elif kind == "missing": value["sealed_records"].pop()
        elif kind == "duplicate": value["sealed_records"].append(copy.deepcopy(row))
        elif kind == "escape": row["path"] = "../outside"
        else: value["schema"] = value["schema"].rsplit(".", 1)[0] + ".v0"
        model.read_strict_json = lambda path, v=value: v
        try: model.validate_manifest()
        except model.GateError as error: cases.append({"case": kind, "rejected": True, "error": str(error)})
        else: raise AssertionError("unsafe retention accepted: " + kind)
    model.read_strict_json = read
    target = next(r for r in manifest["sealed_records"] if r["classification"] == "frozen_private_model")
    raw = (ROOT / target["path"]).read_bytes()
    with tempfile.TemporaryDirectory() as td:
        fake = Path(td) / "changed"; fake.write_bytes(bytes([raw[0] ^ 1]) + raw[1:])
        def redirected(relative, context):
            if relative == target["path"] and context == "sealed historical record": return fake
            return original_validate(relative, context)
        model.validate_regular_file = redirected
        try: model.validate_manifest()
        except model.GateError as error:
            assert "historical bytes changed" in str(error)
            cases.append({"case": "same_size_changed_actual_bytes", "rejected": True, "error": str(error)})
        else: raise AssertionError("changed historical input accepted")
    model.validate_regular_file = original_validate; model.validate_manifest()
    active = model.active_retained_record_exclusions(manifest)
    assert all(not p.startswith(("crates/", "go-tools/", "rust-tools/", "csharp-tools/", "java-tools/", "scripts/")) for p in (r["path"] for r in manifest["sealed_records"]))
    report = {"schema": "mpk.obsolete_interface.seal_controls.v0", "cases": cases,
              "sealed_records": len(manifest["sealed_records"]), "total_retained_records": len(active),
              "production_seal_paths": [], "original_repository_files_modified_by_test": False}
    (HERE / "seal-controls.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print("PASS: 10 changed/foreign/path/epoch controls rejected; exact historical bytes required; no production seal paths")


if __name__ == "__main__": main()
