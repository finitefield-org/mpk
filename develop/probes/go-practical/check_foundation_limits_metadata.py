#!/usr/bin/env python3
"""Check private packaging, exact joint core closure and preserved earlier receipts."""
import re
from urllib.parse import unquote
from check_scope_report import require,sha,strict_json
from run_foundation_limits import ROOT,RECORD,check_retained
from run_context_protocol import check_retained as check_context
from run_identity_dag import check_retained as check_identity
from run_structural_limits import check_retained as check_structural
from run_source_admission import check_retained as check_admission
from run_lifetime_probes import check_retained as check_lifetime
from run_storage_view_probes import check_retained as check_views
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04=RECORD.parent;files=sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for path in files:strict_json(path.read_bytes())
    report=check_retained();require(len(check_context()["cases"])==152 and check_identity()["outcomes"]["hash_vectors"]==44,"retained context/identity evidence")
    require(len(check_structural()["counter_boundaries"])==192 and len(check_admission()["cases"])==81,"retained numeric/source evidence")
    require(len(check_lifetime()["ordinary_cases"])==5 and len(check_views()["ordinary_cases"])==10 and len(check_overlay()["outcomes"])==24,"retained lifetime/view/overlay evidence")
    require(check_variant(strict_json((w04/"variant-termination/observations.json").read_bytes()))==13 and check_foundation(strict_json((w04/"nat-termination/observations.json").read_bytes()))==39,"retained ordinary core evidence")
    work=strict_json((w04/"freeze-worklist.json").read_bytes());draft=work["foundation_limits_protocol_draft"]
    require((draft["closed_root_schemas"],draft["counters"],draft["operations"],draft["families"],draft["core_interfaces"],draft["configuration_vectors"],draft["hash_vectors"],draft["targeted_tests"])==(3,64,56,8,2,102,21,20),"packaging coverage")
    require(draft["complete_ordinary_foundation"] is False and draft["production_core_changed"] is False and draft["remaining"] and work["status"]=="in progress; not frozen or published" and work["open_decisions"],"W04 remains open with full source/ordinary scope")
    require(report["outcomes"]=={"configuration_candidate":16,"configuration_rejected":86},"configuration-only outcomes")
    defs=strict_json((RECORD/"definitions.json").read_bytes());require(len(defs["mirrored_checker_inputs"])==77 and defs["representation"]=="operation_recipe_inventory_draft","no recipe index promoted to complete kernel definitions")
    for m in defs["evidence_receipts"]+[defs["operation_table"]]+sum([i["sources"] for i in defs["core_interfaces"]],[]):require(sha((ROOT/m["path"]).read_bytes())==m["sha256"] and len((ROOT/m["path"]).read_bytes())==m["size_bytes"],"exact retained recipe/core/evidence member bytes")
    inventory=strict_json((w04/"surface-inventory.json").read_bytes());pins=inventory["discovered_files"]
    require(len(pins)==len({p["path"] for p in pins})==143,"predecessor coverage")
    for p in pins+inventory["baseline"]+[p for o in inventory["component_owners"] for p in o["existing_anchors"]]:require(sha((ROOT/p["path"]).read_bytes())==p["sha256"],"unchanged predecessor/baseline")
    initial=w04/"nat-type-selection-initial";d=strict_json((initial/"observations.json").read_bytes())
    for p in d["inputs"]:require(sha((initial/p["snapshot"]).read_bytes())==p["sha256"],"retained diagnostic source")
    require(sha((initial/"intended-strong-positive.mpcert").read_bytes())==d["candidate_sha256"],"retained diagnostic candidate")
    d=strict_json((initial/"history-alias-observations.json").read_bytes());require(sha((initial/"history-alias-producer.rs").read_bytes())==d["producer_sha256"] and sha((initial/"intended-strong-history-alias.mpcert").read_bytes())==d["candidate_sha256"],"retained history diagnostic")
    docs=[w04/"README.md",RECORD/"README.md",RECORD/"semantic-rules.md",w04/"context-protocol/README.md",w04/"identity-dag/README.md",w04/"structural-limits/README.md",w04/"source-admission/README.md",w04/"storage-lifetimes/README.md",w04/"storage-views/README.md",w04/"overlay-contracts/README.md",w04/"variant-termination/README.md",w04/"nat-termination/README.md",w04/"type-error-zero-decisions.md",w04/"operation-loop-decisions.md",ROOT/"develop/docs/09_go_practical_subset_design-todo.md",ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",doc.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((doc.parent/unquote(target.split("#",1)[0])).is_file(),"local document link: "+target);links+=1
    require(list(map(int,re.findall(r"^## (\d+)\.",docs[-1].read_text(),re.M)))==list(range(1,42)),"ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs,three packaging roots/102 configuration cases/21 hash vectors; retained152/44/192/81/5/10/24/13/39 evidence,143 predecessor pins,two diagnostics,sixteen documents/{links} links; W04 open")


if __name__=="__main__":main()
