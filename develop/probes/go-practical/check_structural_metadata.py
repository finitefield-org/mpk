#!/usr/bin/env python3
"""Check numeric/state draft and preserved predecessor source/proof evidence."""
import re
from urllib.parse import unquote
from check_scope_report import require,sha,strict_json
from run_structural_limits import ROOT,RECORD,check_retained
from run_source_admission import check_retained as check_admission
from run_lifetime_probes import check_retained as check_lifetime
from run_storage_view_probes import check_retained as check_views
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04=RECORD.parent;files=sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files:strict_json(p.read_bytes())
    r=check_retained();require(len(check_admission()["cases"])==81,"retained admission evidence")
    require(len(check_lifetime()["ordinary_cases"])==5 and len(check_views()["ordinary_cases"])==10,"retained lifetime/view laws")
    require(len(check_overlay()["outcomes"])==24,"retained overlay evidence")
    require(check_variant(strict_json((w04/"variant-termination/observations.json").read_bytes()))==13,"retained variant evidence")
    require(check_foundation(strict_json((w04/"nat-termination/observations.json").read_bytes()))==39,"retained foundation evidence")
    work=strict_json((w04/"freeze-worklist.json").read_bytes());draft=work["structural_limit_draft"]
    require((draft["counter_candidates"],draft["isolated_boundaries"],draft["native_layout_sources"],draft["targeted_tests"])==(64,192,85,20),"numeric accounting coverage")
    require(draft["production_counter_enforcement"] is False and draft["source_allocator_binding_proved"] is False and draft["remaining"],"counter/source scope")
    require(work["status"]=="in progress; not frozen or published" and work["open_decisions"],"W04 remains open")
    limits=strict_json((w04/"candidate-limits.json").read_bytes())["structural_accounting_draft"]
    require((limits["value_units"],limits["current_store_units"],limits["all_store_units"],limits["typed_syntax_nodes_candidate"],limits["predecessor_typed_syntax_nodes"])==(2097152,8388608,33554432,4194304,1000000),"explicit numeric candidates and predecessor preservation")
    require(limits["unit_scope"]=="abstract carrier units; not Go bytes/RSS, proof cost or a support guarantee" and limits["source_finite_id_proved"] is False,"finite-ID/resource scope")
    inventory=strict_json((w04/"surface-inventory.json").read_bytes());pins=inventory["discovered_files"]
    require(len(pins)==len({p["path"] for p in pins})==143,"predecessor coverage")
    owners=inventory["component_owners"];producers={o["primary_production_executor"] for o in owners};tests={o["primary_test_owner"] for o in owners}
    for o in owners:producers.update(o.get("additional_production_executors",[]));tests.update(o.get("additional_test_owners",[]))
    for c in strict_json((RECORD/"limits.json").read_bytes())["counters"]:require(c["planned_producer"] in producers and c["planned_test"] in tests,"planned counter enforcement ownership")
    for p in pins+inventory["baseline"]+[p for o in owners for p in o["existing_anchors"]]:require(sha((ROOT/p["path"]).read_bytes())==p["sha256"],"unchanged predecessor/baseline")
    initial=w04/"nat-type-selection-initial";d=strict_json((initial/"observations.json").read_bytes())
    for p in d["inputs"]:require(sha((initial/p["snapshot"]).read_bytes())==p["sha256"],"retained failed construction sources")
    require(sha((initial/"intended-strong-positive.mpcert").read_bytes())==d["candidate_sha256"],"retained failed construction candidate")
    d=strict_json((initial/"history-alias-observations.json").read_bytes());require(sha((initial/"history-alias-producer.rs").read_bytes())==d["producer_sha256"] and sha((initial/"intended-strong-history-alias.mpcert").read_bytes())==d["candidate_sha256"],"retained history diagnostic")
    docs=[w04/"README.md",RECORD/"README.md",w04/"source-admission/README.md",w04/"storage-lifetimes/README.md",w04/"storage-views/README.md",w04/"overlay-contracts/README.md",w04/"variant-termination/README.md",w04/"nat-termination/README.md",w04/"type-error-zero-decisions.md",w04/"operation-loop-decisions.md",ROOT/"develop/docs/09_go_practical_subset_design-todo.md",ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",doc.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((doc.parent/unquote(target.split("#",1)[0])).is_file(),"local document link: "+target);links+=1
    require(list(map(int,re.findall(r"^## (\d+)\.",docs[-1].read_text(),re.M)))==list(range(1,39)),"ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs, 64 counters/192 boundaries/85 native layouts +HTTP1417549 nodes; retained81/5/10/24/13/39 evidence, 143 predecessor pins, two diagnostics, twelve documents/{links} links; W04 open")


if __name__=="__main__":main()
