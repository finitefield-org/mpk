"""Per-file candidate routing, never a completed migration/semantic review."""
import json
from pathlib import Path
from run_proof_pilot import ROOT,sha


def proposal():
    inventory=json.loads((ROOT/"develop/migrations/go-practical/t01-w04/surface-inventory.json").read_text())
    rows=[]
    for pin in inventory["discovered_files"]:
        path=pin["path"];raw=(ROOT/path).read_bytes()
        if sha(raw)!=pin["sha256"]:raise ValueError("unchanged predecessor source")
        if "csharp_practical" in path:
            action="retain_reserved_inactive_csharp_bytes_and_reject_public_dispatch";component="release";work="GO-PRAC-T07-W02"
        elif path.startswith(("crates/mpk-core/","crates/mpk-kernel/","go-tools/mpk-checker-ref/")):
            action="stage_additive_induction_type_cases_with_legacy_rule_certificate_equivalence";component="core_nat";work="GO-PRAC-T02-W03"
        elif path.startswith("crates/mpk-api/") or any(s in path for s in ("policy_","successor_policy","ai_explain","v2_tests")):
            action="adapt_public_evidence_api_policy_to_one_new_family_preserve_four_profiles";component="policy";work="GO-PRAC-T07-W01"
        elif path.startswith(("go-tools/go2vir/","csharp-tools/","java-tools/","rust-tools/")):
            action="adapt_public_producer_wrapper_preserve_profile_source_semantics_and_certificates";component="release";work="GO-PRAC-T07-W02"
        elif path.startswith("scripts/") or any(s in path for s in ("release","frontend_","successor_cli","atomic_cutover")):
            action="atomic_install_protocol_release_fixture_and_whole_image_rollback";component="release";work="GO-PRAC-T07-W02"
        else:
            action="independent_successor_import_hash_source_vc_and_predecessor_equivalence";component="artifacts";work="GO-PRAC-T02-W03"
        owner=next(o for o in inventory["component_owners"] if o["component"]==component)
        rows.append({**pin,"candidate_disposition":action,"work_item":work,"planned_primary_executor":owner["primary_production_executor"],"planned_primary_test":owner["primary_test_owner"],
                     "existing_path_role":"test/support" if "/tests/" in path or path.endswith("_test.go") else "producer/consumer/helper",
                     "review_status":"candidate routing from pinned path/known surface; full symbol/preimage/migration review remains required"})
    return {"schema":"mpk.go_practical.identity_migration_draft.v0","status":"private per-file proposal; no production migration completed","entry_commit":"47b2a5e9846b5200ca1106ab9407de310af6e169","files":rows,
            "atomic_activation_owner":"GO-PRAC-T07-W04","rollback":"restore the complete pre-cutover installed image, binaries, registries, bundles, contracts, fixtures and docs together",
            "pending":["review all exact symbols/shared and profile-specific preimages, including additional discovery beyond the143-file predecessor inventory","freeze full shared roots/unions, every producer/consumer/test migration and four-profile certificate/source equivalence","private implementation and both-checker/native Linux gates before one-family activation; inactive C# remains inactive"]}
