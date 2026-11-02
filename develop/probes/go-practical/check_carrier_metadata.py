#!/usr/bin/env python3
"""Check actual carrier certificates and preserve the complete prior W04 evidence."""
import re
from urllib.parse import unquote
from run_closed_carriers import ROOT,RECORD,check_retained
from run_foundation_limits import check_retained as check_package
from run_context_protocol import check_retained as check_context
from run_identity_dag import check_retained as check_identity
from run_structural_limits import check_retained as check_structural
from run_source_admission import check_retained as check_admission
from run_lifetime_probes import check_retained as check_lifetime
from run_storage_view_probes import check_retained as check_views
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation
from check_scope_report import strict_json,require,sha


def main():
    w04=RECORD.parent;files=sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for path in files:strict_json(path.read_bytes())
    r=check_retained();require(len(check_package()["cases"])==102 and len(check_context()["cases"])==152 and check_identity()["outcomes"]["hash_vectors"]==44,"retained packaging/context/identity evidence")
    require(len(check_structural()["counter_boundaries"])==192 and len(check_admission()["cases"])==81,"retained counter/source evidence")
    require(len(check_lifetime()["ordinary_cases"])==5 and len(check_views()["ordinary_cases"])==10 and len(check_overlay()["outcomes"])==24,"retained lifetime/view/overlay evidence")
    require(check_variant(strict_json((w04/"variant-termination/observations.json").read_bytes()))==13 and check_foundation(strict_json((w04/"nat-termination/observations.json").read_bytes()))==39,"retained joint core/ordinary proof evidence")
    d=strict_json((w04/"freeze-worklist.json").read_bytes());work=d["closed_carrier_draft"]
    require((work["source_types"],work["specializations"],work["plain_error_specializations"],work["accepted_ordinary_sets"],work["core_rejected_controls"],work["targeted_tests"])==(39,42,43,2,6,20),"actual carrier coverage")
    require(work["source_proofs_checked"] is False and work["finite_ids_proved"] is False and work["remaining"] and d["status"]=="in progress; not frozen or published" and d["open_decisions"],"full W04 scope remains open")
    require(len(r["mirrored_checker_inputs"])==77,"joint private source closure")
    inventory=strict_json((w04/"surface-inventory.json").read_bytes());pins=inventory["discovered_files"]
    require(len(pins)==len({p["path"] for p in pins})==143,"predecessor coverage")
    for p in pins+inventory["baseline"]+[p for o in inventory["component_owners"] for p in o["existing_anchors"]]:require(sha((ROOT/p["path"]).read_bytes())==p["sha256"],"unchanged predecessor/baseline")
    initial=w04/"nat-type-selection-initial";x=strict_json((initial/"observations.json").read_bytes())
    for p in x["inputs"]:require(sha((initial/p["snapshot"]).read_bytes())==p["sha256"],"retained diagnostic source")
    require(sha((initial/"intended-strong-positive.mpcert").read_bytes())==x["candidate_sha256"],"retained diagnostic certificate")
    x=strict_json((initial/"history-alias-observations.json").read_bytes());require(sha((initial/"history-alias-producer.rs").read_bytes())==x["producer_sha256"] and sha((initial/"intended-strong-history-alias.mpcert").read_bytes())==x["candidate_sha256"],"retained history diagnostic")
    docs=[w04/"README.md",RECORD/"README.md",w04/"foundation-limits/README.md",w04/"foundation-limits/semantic-rules.md",w04/"context-protocol/README.md",w04/"identity-dag/README.md",w04/"structural-limits/README.md",w04/"source-admission/README.md",w04/"storage-lifetimes/README.md",w04/"storage-views/README.md",w04/"overlay-contracts/README.md",w04/"variant-termination/README.md",w04/"nat-termination/README.md",w04/"type-error-zero-decisions.md",w04/"operation-loop-decisions.md",ROOT/"develop/docs/09_go_practical_subset_design-todo.md",ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",doc.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((doc.parent/unquote(target.split("#",1)[0])).is_file(),"local document link: "+target);links+=1
    require(list(map(int,re.findall(r"^## (\d+)\.",docs[-1].read_text(),re.M)))==list(range(1,43)),"ordered ledger sections")
    print(f"PASS:{len(files)} strict JSONs,39 source types/42–43 specializations/eight paired core cases/two legacy,retained102/152/44/192/81/5/10/24/13/39 evidence,143 predecessor pins,two diagnostics,seventeen documents/{links} links; W04 open")


if __name__=="__main__":main()
