#!/usr/bin/env python3
"""Check the private context draft and preserve prior identity/proof evidence."""
import re
from urllib.parse import unquote
from check_scope_report import require,sha,strict_json
from run_context_protocol import ROOT,RECORD,check_retained
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
    report=check_retained();require(check_identity()["outcomes"]["hash_vectors"]==44,"retained three-language identity/hash evidence")
    require(len(check_structural()["counter_boundaries"])==192 and len(check_admission()["cases"])==81,"retained numeric/source evidence")
    require(len(check_lifetime()["ordinary_cases"])==5 and len(check_views()["ordinary_cases"])==10,"retained lifetime/view evidence")
    require(len(check_overlay()["outcomes"])==24,"retained overlay evidence")
    require(check_variant(strict_json((w04/"variant-termination/observations.json").read_bytes()))==13 and check_foundation(strict_json((w04/"nat-termination/observations.json").read_bytes()))==39,"retained ordinary foundation evidence")
    work=strict_json((w04/"freeze-worklist.json").read_bytes());draft=work["context_protocol_draft"]
    require((draft["closed_root_schemas"],draft["profiles"],draft["valid_target_contexts"],draft["vectors"],draft["legacy_queries"],draft["targeted_tests"])==(6,5,6,152,31,20),"context coverage")
    require(draft["complete_foundation"] is False and draft["production_source_import"] is False and draft["profile_activated"] is False and draft["remaining"],"private context/foundation/activation boundary")
    require(work["status"]=="in progress; not frozen or published" and work["open_decisions"],"W04 remains open")
    fixture=strict_json((RECORD/"registration-fixture.json").read_bytes());entries=fixture["registry"]["profiles"]
    require(len(entries)==5 and sum(e["foundation_descriptor"] is None for e in entries)==4 and fixture["foundation_fixture"]["complete_definition_bundle"] is False,"nullable legacy/exact Go foundation fixture")
    require(report["outcomes"]=={"context":{"configuration_rejected":61,"context_candidate":6},"linkage":{"configuration_rejected":9,"linkage_candidate":6},"registry":{"configuration_rejected":12,"registry_candidate":1},"request":{"configuration_rejected":40,"request_candidate":11},"wire":{"configuration_rejected":6}},"configuration evidence only")
    inventory=strict_json((w04/"surface-inventory.json").read_bytes());pins=inventory["discovered_files"]
    require(len(pins)==len({p["path"] for p in pins})==143,"predecessor coverage")
    for p in pins+inventory["baseline"]+[p for o in inventory["component_owners"] for p in o["existing_anchors"]]:require(sha((ROOT/p["path"]).read_bytes())==p["sha256"],"unchanged predecessor/baseline")
    initial=w04/"nat-type-selection-initial";d=strict_json((initial/"observations.json").read_bytes())
    for p in d["inputs"]:require(sha((initial/p["snapshot"]).read_bytes())==p["sha256"],"retained diagnostic source")
    require(sha((initial/"intended-strong-positive.mpcert").read_bytes())==d["candidate_sha256"],"retained diagnostic candidate")
    d=strict_json((initial/"history-alias-observations.json").read_bytes());require(sha((initial/"history-alias-producer.rs").read_bytes())==d["producer_sha256"] and sha((initial/"intended-strong-history-alias.mpcert").read_bytes())==d["candidate_sha256"],"retained history diagnostic")
    docs=[w04/"README.md",RECORD/"README.md",w04/"identity-dag/README.md",w04/"structural-limits/README.md",w04/"source-admission/README.md",w04/"storage-lifetimes/README.md",w04/"storage-views/README.md",w04/"overlay-contracts/README.md",w04/"variant-termination/README.md",w04/"nat-termination/README.md",w04/"type-error-zero-decisions.md",w04/"operation-loop-decisions.md",ROOT/"develop/docs/09_go_practical_subset_design-todo.md",ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",doc.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((doc.parent/unquote(target.split("#",1)[0])).is_file(),"local document link: "+target);links+=1
    require(list(map(int,re.findall(r"^## (\d+)\.",docs[-1].read_text(),re.M)))==list(range(1,41)),"ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs,six closed root schemas/152 vectors/31 actual legacy queries; retained44/192/81/5/10/24/13/39 evidence,143 predecessor pins,two diagnostics,fourteen documents/{links} links; W04 open")


if __name__=="__main__":main()
