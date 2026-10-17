"""Publish/check the inactive Go design package, with no runtime activation."""
import argparse
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
W04 = ROOT / "develop/migrations/go-practical/t01-w04"
RECORD = W04.parent / "t01-w05"
VECTORS = ROOT / "develop/specs/vectors"
PREDECESSOR = "68a44416e1b1a6dc3a84587cca784fc4b3a2b862"
SHELL_OWNER = "scripts/check-go-practical-spec.sh"
PUBLICATIONS = {
    "go-practical-profile-v1": ("mpk.go.practical.profile.conformance.v1", "GO_PRACTICAL_PROFILE_V1.md", ["crates/mpk-vc/tests/go_practical_spec.rs"]),
    "go-practical-foundation-v1": ("mpk.go.practical.foundation.conformance.v1", "GO_PRACTICAL_FOUNDATION_V1.md", ["crates/mpk-vc/tests/go_practical_foundation.rs"]),
    "go-practical-shared-artifacts-v1": ("mpk.go.practical.shared_artifacts.conformance.v1", "GO_PRACTICAL_SHARED_ARTIFACTS_V1.md", ["crates/mpk-vc/tests/go_practical_artifacts.rs"]),
    "core-nat-induction-v1": ("mpk.core.nat_induction.conformance.v1", "CORE_NAT_INDUCTION_V1.md", ["crates/mpk-core/src/nat_induct.rs", "go-tools/mpk-checker-ref/nat_induct_test.go"]),
    "core-bool-type-cases-v1": ("mpk.core.bool_type_cases.conformance.v1", "CORE_BOOL_TYPE_CASES_V1.md", ["crates/mpk-core/src/bool_rec_type.rs", "go-tools/mpk-checker-ref/bool_rec_type_test.go"]),
}

COMMON = ["source-admission/catalog.json", "operation-signatures/catalog.json", "contract-wire/catalog.json",
          "structural-limits/limits.json", "foundation-limits/semantic-rules.md", "program-binding/README.md",
          "program-binding/interfaces.json", "program-binding/host-resources.json", "consumer-review/dispositions.json",
          "consumer-review/requirements.json", "type-error-zero-decisions.md", "operation-loop-decisions.md"]
FOUNDATION = ["closed-carriers/schemas.json", "foundation-limits/schemas.json", "foundation-limits/definitions.json",
              "foundation-limits/limits.json", "storage-views/README.md", "storage-lifetimes/README.md",
              "word-foundation/README.md", "word-arithmetic/README.md", "nat-termination/README.md", "variant-termination/README.md"]
SHARED = ["identity-dag/catalog.json", "context-protocol/schemas.json", "contract-wire/catalog.json",
          "shared-vir/schema.json", "vir-body/schema.json", "capture-artifacts/schema.json",
          "proof-artifacts/schema.json", "frontend-envelope/schema.json", "policy-api/schema.json",
          "api-contracts/schema.json", "ai-explanation/schema.json", "release-contracts/schema.json"]


def sha(raw): return hashlib.sha256(raw).hexdigest()
def encoded(value): return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode()
def read(path): return json.loads(path.read_text())
def ref(path):
    raw = path.read_bytes()
    return {"path": str(path.relative_to(ROOT)), "sha256": sha(raw), "bytes": len(raw)}


def audit_predecessor():
    d = read(W04 / "completion-audit.json")
    assert d["status"] == "design_complete_inactive" and not d["unresolved_design_questions"]
    for r in d["design_inputs"]:
        assert sha((ROOT / r["path"]).read_bytes()) == r["sha256"], "frozen W04 member changed: " + r["path"]
    return d


def generate(name):
    schema, spec, owners = PUBLICATIONS[name]; members = list(COMMON)
    if name == "go-practical-foundation-v1": members += FOUNDATION
    elif name == "go-practical-shared-artifacts-v1": members += SHARED
    elif name.startswith("core-"): members = ["foundation-limits/definitions.json", "nat-termination/README.md"]
    else: members += ["source-vectors.json", "evaluation-vectors.json", "operation-table.json"]
    defs = read(W04 / "foundation-limits/definitions.json")
    core = [r for r in defs["core_interfaces"] if not name.startswith("core-") or (r["name"] == "Std.Nat.induct") == (name == "core-nat-induction-v1")]
    v = {
        "schema": schema, "status": "frozen_inactive_design", "work_item": "GO-PRAC-T01-W05",
        "semantic_profile": "mpk.go.practical.v1", "owner_tests": owners,
        "predecessor_commit": PREDECESSOR, "owning_spec": ref(ROOT / "develop/specs" / spec),
        "publication_generator": ref(Path(__file__)),
        "contract_members": [ref(W04 / p) for p in sorted(set(members))],
        "core_interfaces": core,
        "design_completion": ref(W04 / "completion-audit.json"),
        "implementation_status": {"registered_producer": False, "Go_program_proof": False,
                                  "native_host_gate": False, "installed_activation": False},
        "historical_evidence_rule": "exact original Git epoch/receipts; current implementation requires new inputs and execution; metadata cannot promote proof or release readiness",
        "required_later_gates": ["T02–T06 complete source/ordinary/application proofs and stage gates", "T07 first native release", "T08–T11 maps/pointers and second native release"],
    }
    if name.startswith("core-"):
        v["measured_evidence"] = [ref(W04 / "nat-termination/observations.json")]
        v["vectors"] = [{"id": name + "-" + str(i), "condition": g, "expected": "enforce_exact_guard", "production_observed": False}
                        for i, g in enumerate(core[0]["guards"], 1)]
        if name == "core-bool-type-cases-v1": v["measured_interface_cases"] = ref(W04 / "nat-termination/interfaces.json")
    else:
        v["vectors"] = read(W04 / "program-binding/conformance-vectors.json")["vectors"]
        v["requirement_owners"] = ref(W04 / "consumer-review/requirements.json")
    return v


def check_package(write=False):
    audit = audit_predecessor(); RECORD.mkdir(parents=True, exist_ok=True)
    manifest_path = VECTORS / "manifest.json"; old_path = RECORD / "predecessor-spec-manifest.json"
    if not old_path.exists():
        assert write, "missing retained manifest snapshot"
        old_path.write_bytes(manifest_path.read_bytes())
    old = read(old_path); assert len(old["vectors"]) == 26
    expected = copy.deepcopy(old)
    for name, (schema, spec, owners) in PUBLICATIONS.items():
        doc = generate(name); raw = encoded(doc); path = VECTORS / (name + ".json")
        if write: path.write_bytes(raw)
        assert path.read_bytes() == raw, "exact normative vector reconstruction: " + name
        expected["vectors"].append({"schema_id": schema, "path": str(path.relative_to(ROOT)), "sha256": sha(raw),
                                    "owning_spec": "develop/specs/" + spec,
                                    "implementation_test_owners": owners + [SHELL_OWNER]})
    expected["vectors"].sort(key=lambda r: r["path"])
    if write: manifest_path.write_bytes(encoded(expected))
    assert read(manifest_path) == expected, "exact26 retained entries plus5 inactive Go/core vector sets"
    for r in old["vectors"]:
        assert sha((ROOT / r["path"]).read_bytes()) == r["sha256"], "retained specification vector bytes unchanged"
    return {"schema": "mpk.go_practical.w05_publication_audit.v0", "status": "inactive_specification_published",
            "predecessor_commit": PREDECESSOR, "frozen_W04_members": len(audit["design_inputs"]),
            "retained_vector_sets": 26, "new_vector_sets": 5, "total_vector_sets": 31,
            "specifications": [ref(ROOT / "develop/specs" / v[1]) for v in PUBLICATIONS.values()],
            "vectors": [ref(VECTORS / (n + ".json")) for n in PUBLICATIONS], "manifest": ref(manifest_path),
            "program_proofs_generated": False, "public_profile_activated": False,
            "full_T01_gate": "separate execution required before T01 completion"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    g = parser.add_mutually_exclusive_group(required=True); g.add_argument("--write", action="store_true"); g.add_argument("--check", action="store_true")
    args = parser.parse_args(); result = check_package(args.write)
    path = RECORD / "publication-audit.json"
    if args.write: path.write_bytes(encoded(result))
    assert path.read_bytes() == encoded(result), "publication audit reconstruction"
    print("PASS: five inactive specifications/vector sets; all255 W04 pins and26 retained vector sets unchanged;31 total manifest entries; no proof or activation inferred")


if __name__ == "__main__": main()
