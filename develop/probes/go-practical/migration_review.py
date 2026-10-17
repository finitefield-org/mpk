"""Finite reviewed migration inventory and requirement handoff for W04.

The original 143-path inventory is immutable. A second scan records indirect
consumers separately. No production file or historical observation is rewritten.
"""
from collections import Counter
import json
from pathlib import Path
import subprocess

from run_proof_pilot import ROOT, sha
from program_design import RECORD as PROGRAM, catalogue, host_contract

W04 = PROGRAM.parent
RECORD = W04 / "consumer-review"
INDIRECT_PATTERN = r"mpk\.(ai\.api\.v2|policy\.(scan|evidence|receipt|reproduction)\.v[12]|frontend\.(request|success|diagnostic)\.v[12]|release\.(bundle_registry|frontend_bundle|toolchain_bundle)\.v1)|SourceManifest|SuccessorVcCertificateSkeleton|CompiledProfileEnvelope|BundleRegistry"
EXPLICIT = (
    "crates/mpk-cert/src/export.rs", "crates/mpk-cli/tests/ai_explain_v1.rs",
    "crates/mpk-cli/tests/csharp_ai_explain.rs", "crates/mpk-cli/tests/java_release_gate.rs",
    "scripts/java_release_gate.py", "scripts/build-release-bundles.sh",
    "scripts/check-release-bundles.sh", "scripts/generate-release-report.py",
)

GROUPS = {
    "inactive_csharp": {
        "action": "retain exact reserved inactive identity/meaning; isolate dispatch; reconcile only on later CSharp resumption",
        "work_item": "GO-PRAC-T02-W03",
        "test_owner": "crates/mpk-vc/tests/go_practical_artifacts.rs",
        "coverage": "reserved CSharp schemas and vectors unchanged; practical CSharp never activated as a Go prerequisite",
    },
    "certificate_core": {
        "action": "retain Certificate v0 codec/hash/export and existing Nat/Bool rules; add only separately frozen Nat.induct/Bool.recType handling",
        "work_item": "GO-PRAC-T02-W03",
        "test_owner": "crates/mpk-core/src/nat_induct.rs",
        "coverage": "paired-checker constructor/universe/iota rejection controls; unchanged old certificate and declaration interface hashes",
    },
    "api": {
        "action": "implement exact v4 context/source/session/atomic dispatch; keep old codecs isolated for historical tests; install only the new family at T07",
        "work_item": "GO-PRAC-T07-W01",
        "test_owner": "crates/mpk-api/tests/go_practical_policy.rs",
        "coverage": "33 route contracts, wrong context/source/statement, stale incarnation/revision and failed-transaction rollback",
    },
    "policy_ai": {
        "action": "migrate policy/evidence/AI roots and whole-source statement binding; dispatch exact retained axiom policy; preserve failure/status/redaction semantics",
        "work_item": "GO-PRAC-T07-W01",
        "test_owner": "crates/mpk-cli/tests/go_practical_policy.rs",
        "coverage": "all four languages, both Rust targets, full mvp-theory formats/allowlists/five-category reports; Go zero axioms; all property batches",
    },
    "release": {
        "action": "build v3 descriptors/registry/consumer from real bytes; rebind owner scopes; install one complete image atomically after native gates",
        "work_item": "GO-PRAC-T07-W02",
        "test_owner": "crates/mpk-cli/tests/go_practical_release.rs",
        "coverage": "six exact contexts, two-pass build/installed replay, strong practical-Go host, four-profile preservation and atomic rollback",
    },
    "frontend_transport": {
        "action": "port retained producer/consumer envelopes, manifests, maps, selectors and hashes to v3; preserve captured source and retained language semantics",
        "work_item": "GO-PRAC-T02-W03",
        "test_owner": "crates/mpk-vc/tests/go_practical_artifacts.rs",
        "coverage": "GoFixed/Rust32/Rust64/CSharp/Java source, IR, contract, VC and diagnostics equality apart from explicitly changed wire ownership/hash refs",
    },
    "go_capture": {
        "action": "add practical-only immutable offline capture/two-closure path and exact limits; keep fixed-profile admission and type-check behavior unchanged",
        "work_item": "GO-PRAC-T02-W01",
        "test_owner": "go-tools/go2vir/practical_capture_test.go",
        "coverage": "original-package errors, modules/replacements, versions, path/TOCTOU/counter controls and old Go corpus",
    },
    "go_semantics": {
        "action": "add practical-only typed lowering/carriers/effects without widening GoFixed; common wire emission follows frontend-transport migration",
        "work_item": "GO-PRAC-T03-W02",
        "test_owner": "go-tools/go2vir/practical_lower_test.go",
        "coverage": "evaluation order, source identities, exact result/state linkage; requirement owners retain T03–T06 feature proofs",
    },
    "shared_model": {
        "action": "add independent v3/v4 imports and ordinary assembly; preserve old internal semantic models and historical parsers; do not widen v1/v2",
        "work_item": "GO-PRAC-T02-W03",
        "test_owner": "crates/mpk-vc/tests/go_practical_artifacts.rs",
        "coverage": "closed schemas/preimages/context/source/statement linkage; retained admission and operation/axiom behavior unchanged",
    },
    "retained_frontend": {
        "action": "retain language lowering/type/contract semantics; update only explicit successor envelope/hash/emission interfaces and their fixtures",
        "work_item": "GO-PRAC-T02-W03",
        "test_owner": "crates/mpk-vc/tests/go_practical_artifacts.rs",
        "coverage": "captured inputs and all retained theorem demands preserved; wrong target/nominal/contract/source links reject",
    },
    "regression_tests": {
        "action": "retain semantic and negative assertions; add successor-family fixtures separately; old literal-schema fixtures replay at their original source epoch",
        "work_item": "GO-PRAC-T02-W03",
        "test_owner": "crates/mpk-vc/tests/go_practical_artifacts.rs",
        "coverage": "new producer/consumer crossing plus unchanged predecessor semantic results; no relabeling old observation hashes",
    },
}


def group(path):
    p = Path(path); n = p.name
    if "csharp_practical" in n: return "inactive_csharp"
    if path.startswith(("crates/mpk-core/", "crates/mpk-kernel/", "crates/mpk-cert/", "go-tools/mpk-checker-ref/")):
        return "certificate_core"
    if path.startswith("crates/mpk-api/"): return "api"
    if "release" in n or "atomic_cutover" in n: return "release"
    if any(x in n for x in ("policy", "ai_explain")): return "policy_ai"
    if path.startswith("crates/mpk-cli/src/"):
        return "shared_model" if n == "program_certificate.rs" else "frontend_transport"
    if path.startswith("go-tools/go2vir/"):
        if n.endswith("_test.go"): return "regression_tests"
        if n in {"preflight.go", "snapshot.go", "loader.go", "features.go"}: return "go_capture"
        if n in {"lower.go", "lower_types.go", "ssa.go"}: return "go_semantics"
        return "frontend_transport"
    if path.startswith(("csharp-tools/", "java-tools/", "rust-tools/")): return "retained_frontend"
    if path == "scripts/csharp_build_inputs.py": return "frontend_transport"
    if "/tests/" in path or path.startswith("scripts/java_"): return "regression_tests"
    if path.startswith("crates/mpk-vc/src/"): return "shared_model"
    raise ValueError("unreviewed consumer: " + path)


def inventory():
    source = json.loads((W04 / "surface-inventory.json").read_text())
    old = {r["path"]: r for r in source["discovered_files"]}
    r = subprocess.run(["rg", "-l", INDIRECT_PATTERN, *source["search_roots"],
                        "-g", "*.rs", "-g", "*.go", "-g", "*.cs", "-g", "*.java", "-g", "*.py", "-g", "*.sh"],
                       cwd=ROOT, check=True, capture_output=True, text=True)
    paths = sorted(set(old) | set(r.stdout.splitlines()) | set(EXPLICIT))
    rows = []
    for p in paths:
        raw = (ROOT / p).read_bytes(); digest = sha(raw)
        if p in old: assert old[p]["sha256"] == digest, "unchanged original consumer " + p
        g = group(p); rules = GROUPS[g]
        # Record real references and whole-file pins, including owner-only anchors.
        matches = subprocess.run(["rg", "-n", "--color", "never", source["search_pattern"] + "|" + INDIRECT_PATTERN, p],
                                 cwd=ROOT, capture_output=True, text=True)
        assert matches.returncode in (0, 1)
        refs = []
        for line in matches.stdout.splitlines():
            number, text = line.split(":", 1)
            refs.append({"line": int(number), "text_sha256": sha(text.encode())})
        rows.append({"path": p, "sha256": digest, "bytes": len(raw), "group": g,
                     "discovery": "original_143" if p in old else "explicit_indirect_owner" if p in EXPLICIT else "indirect_reference_scan",
                     "reference_lines": refs, "production_owner": p, **rules})
    assert sum(r["discovery"] == "original_143" for r in rows) == 143
    return {"schema": "mpk.go_practical.consumer_dispositions.v0",
            "status": "reviewed implementation handoff; no active cutover or production conformance inferred",
            "predecessor_inventory_sha256": sha((W04 / "surface-inventory.json").read_bytes()),
            "indirect_search_pattern": INDIRECT_PATTERN, "explicit_indirect_owners": list(EXPLICIT),
            "groups": GROUPS, "counts": dict(sorted(Counter(r["group"] for r in rows).items())), "files": rows,
            "retained_axiom_policy": "exact compiled context dispatch; preserve zero-axiom and mvp-theory formats,allowlists,five-category reports; no blanket zero restriction from observed fixture helper",
            "cutover_rule": "T02 private additive implementations; T07 one installed shared family after all native gates; T08–T11 add practical v2 separately"}


# Semantic evidence, not a new set of source-admission rules.
REQUIREMENT_DOCS = {
    1: ("capture-artifacts", "source-admission", "structural-limits"),
    2: ("capture-artifacts", "identity-dag"),
    3: ("context-protocol", "source-admission"),
    4: ("source-admission", "capture-artifacts", "shared-vir"),
    5: ("source-admission", "vir-body", "program-binding"),
    6: ("closed-carriers", "operation-signatures", "word-foundation"),
    7: ("source-admission", "operation-signatures", "vir-body"),
    8: ("operation-signatures", "vir-body", "program-binding"),
    9: ("closed-carriers", "source-admission", "program-binding"),
    10: ("word-foundation", "word-arithmetic", "operation-signatures"),
    11: ("source-admission", "program-binding"),
    12: ("structural-limits", "operation-signatures", "program-binding"),
    13: ("storage-views", "storage-lifetimes", "closed-carriers"),
    14: ("word-arithmetic", "storage-views", "operation-signatures"),
    15: ("storage-lifetimes", "operation-signatures", "vir-body"),
    16: ("vir-body", "source-admission", "program-binding"),
    17: ("nat-termination", "variant-termination", "program-binding"),
    18: ("overlay-contracts", "contract-wire", "program-binding"),
    19: ("nat-termination", "variant-termination", "foundation-limits"),
    20: ("word-arithmetic", "closed-carriers", "foundation-limits", "program-binding"),
    21: ("context-protocol", "shared-vir", "proof-artifacts", "program-binding"),
    22: ("operation-signatures", "proof-artifacts", "program-binding"),
    23: ("policy-api", "api-contracts", "ai-explanation", "program-binding"),
    24: ("release-contracts", "program-binding", "consumer-review"),
}


def requirements():
    work = json.loads((W04 / "freeze-worklist.json").read_text()); rows = []
    for r in work["requirements"]:
        index = int(r["id"].rsplit("R", 1)[1]); refs = []
        for directory in REQUIREMENT_DOCS[index]:
            p = W04 / directory / "README.md"; assert p.is_file(), str(p)
            refs.append({"path": str(p.relative_to(ROOT)), "sha256": sha(p.read_bytes())})
        rows.append({k: r[k] for k in ("id", "name", "required_meaning", "negative_boundary", "primary_production_executor", "primary_test_owner", "work_item")} |
                    {"design_evidence": refs, "design_status": "specified; production implementation/proofs and release checks remain at assigned work items",
                     "implementation_verified": False})
    assert len(rows) == 24 and len({r["id"] for r in rows}) == 24
    return {"schema": "mpk.go_practical.requirement_handoff.v0", "requirements": rows,
            "scope": "W04 design coverage and explicit implementation/test ownership, not source/application proof acceptance"}


def write():
    RECORD.mkdir(parents=True, exist_ok=True)
    for name, value in (("dispositions.json", inventory()), ("requirements.json", requirements())):
        (RECORD / name).write_text(json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n")


if __name__ == "__main__": write()
