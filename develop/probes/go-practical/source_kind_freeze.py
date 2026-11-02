"""Final inactive source-kind vocabulary; preceding candidate graphs stay sealed."""
import ast
import hashlib
import json
import re
from pathlib import Path
from proof_canonical import canonical, digest, parse

ROOT = Path(__file__).resolve().parents[3]
RECORD = ROOT / "develop/migrations/go-practical/t01-w05/source-kind"
KINDS = ("go_file", "go_mod", "go_sum", "overlay", "metadata", "export_data", "excluded_source", "auxiliary")
SCHEMA = "mpk.go.source_object.v1"
DOMAIN = "MPK-GO-SOURCE-OBJECT-1.0"
FIELDS = {"schema", "capability", "path", "kind", "raw_sha256", "bytes", "id"}
SAFE = (1 << 53) - 1


def need(ok, message):
    if not ok: raise ValueError(message)


def logical_path(value):
    need(type(value) is str and value and len(value.encode()) <= 1024 and not value.startswith("/")
         and "\\" not in value and all(p not in ("", ".", "..") for p in value.split("/"))
         and all(ord(c) >= 32 for c in value), "logical capture path")


def validate(value, raw=None):
    need(type(value) is dict and set(value) == FIELDS and value["schema"] == SCHEMA, "exact source-object schema/fields")
    need(type(value["kind"]) is str and value["kind"] in KINDS, "final closed source-kind enum; no preliminary tag alias")
    for field in ("capability", "path"): logical_path(value[field])
    for field in ("raw_sha256", "id"):
        need(type(value[field]) is str and re.fullmatch("[0-9a-f]{64}", value[field]), "exact source hash")
    need(type(value["bytes"]) is int and 0 <= value["bytes"] <= SAFE, "unsigned safe source size")
    need(value["id"] == digest(DOMAIN, {k: v for k, v in value.items() if k != "id"}), "full source identity preimage")
    if raw is not None:
        need(type(raw) is bytes and len(raw) == value["bytes"] and hashlib.sha256(raw).hexdigest() == value["raw_sha256"], "independent captured raw bytes")
    return {"status": "metadata_linked", "source_admission_proved": False}


def build():
    fixture_path = ROOT / "develop/migrations/go-practical/t01-w04/identity-dag/fixture.json"
    fixture = json.loads(fixture_path.read_text())
    preceding = next(n for n in fixture["nodes"] if n.get("schema") == SCHEMA and n["path"] == "policy.go")
    tree = ast.parse((ROOT / "develop/probes/go-practical/identity_vectors.py").read_text())
    assignment = next(n for n in tree.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "SOURCE" for t in n.targets))
    raw = ast.literal_eval(assignment.value).encode()
    node = {k: v for k, v in preceding.items() if k != "id"}; node["kind"] = "go_file"; node["id"] = digest(DOMAIN, node)
    validate(node, raw)
    need(node["id"] != preceding["id"], "a new tag requires a new source identity, not relabeling")
    try: validate(preceding, raw)
    except ValueError: pass
    else: raise AssertionError("preliminary fixture must reject at the new source-kind boundary")
    vectors = [{"id": "source-kind-go-file", "node": node, "raw_bytes_hex": raw.hex(), "expected": "metadata_linked", "source_admission_proved": False},
               {"id": "source-kind-preliminary-crossing", "fixture_ref": {"path": str(fixture_path.relative_to(ROOT)), "sha256": hashlib.sha256(fixture_path.read_bytes()).hexdigest(), "node_id": preceding["id"]},
                "expected": "source_kind_rejected", "source_admission_proved": False}]
    return {"schema": "mpk.go_practical.final_source_kind_design.v0", "status": "final_inactive_v1_freeze_amendment", "source_object_schema": SCHEMA,
            "source_kind_enum": list(KINDS), "fields": sorted(FIELDS), "domain": DOMAIN, "hash_preimage_fields": sorted(FIELDS - {"id"}),
            "vectors": vectors, "aliases": [], "production_executor": "go-tools/go2vir/practical_capture.go",
            "independent_importer": "crates/mpk-vc/src/go_practical_artifacts.rs", "work_item": "GO-PRAC-T02-W01/GO-PRAC-T02-W03",
            "historical_rule": "Preceding private models and generated fixtures remain sealed historical candidate data, not production importers or current source evidence. T02 regenerates the complete source/context/VIR/VC/policy graph with the final enum and fresh IDs; no graph relabeling or old proof promotion.",
            "scope": "source-kind/identity metadata only; no Go compilation, source correspondence, ordinary proposition or registered-run authority"}


def write():
    RECORD.mkdir(parents=True, exist_ok=True)
    value = build(); (RECORD / "catalog.json").write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    print("PASS: final eight source-kind tags; new identity over unchanged raw bytes; preliminary fixture crossing rejected; no source proof")


if __name__ == "__main__": write()
