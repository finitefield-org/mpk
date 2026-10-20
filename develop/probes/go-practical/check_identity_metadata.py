#!/usr/bin/env python3
"""Check identity-only evidence and preserve all prior W04 boundaries."""
import re
from urllib.parse import unquote
from check_scope_report import require,sha,strict_json
from run_identity_dag import ROOT,RECORD,check_retained
from run_structural_limits import check_retained as check_structural
from run_source_admission import check_retained as check_admission
from run_lifetime_probes import check_retained as check_lifetime
from run_storage_view_probes import check_retained as check_views
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04=RECORD.parent;files=sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files:strict_json(p.read_bytes())
    r=check_retained();require(len(check_structural()["counter_boundaries"])==192 and len(check_admission()["cases"])==81,"retained numeric/source evidence")
    require(len(check_lifetime()["ordinary_cases"])==5 and len(check_views()["ordinary_cases"])==10,"retained lifetime/view evidence")
    require(len(check_overlay()["outcomes"])==24,"retained overlay evidence")
    require(check_variant(strict_json((w04/"variant-termination/observations.json").read_bytes()))==13 and check_foundation(strict_json((w04/"nat-termination/observations.json").read_bytes()))==39,"retained ordinary foundation evidence")
    work=strict_json((w04/"freeze-worklist.json").read_bytes());draft=work["identity_dag_draft"]
    require((draft["candidate_identities"],draft["candidate_domains"],draft["closed_identity_schemas"],draft["identity_nodes"],draft["hash_vectors"],draft["targeted_tests"],draft["candidate_migration_paths"])==(101,41,10,26,44,19,143),"identity/DAG/vector coverage")
    require(draft["full_shared_schema_freeze"] is False and draft["production_source_import"] is False and draft["migration_completed"] is False and draft["remaining"],"private identity/source/migration boundary")
    require(work["status"]=="in progress; not frozen or published" and work["open_decisions"],"W04 remains open")
    c=strict_json((RECORD/"catalog.json").read_bytes());ids={x["identity"] for x in c["identities"] if x["identity"]};domains={x["domain"] for x in c["identities"] if x["domain"]}
    require(not ids.intersection(c["reserved_retired"]["identities"]) and not domains.intersection(c["reserved_retired"]["domains"]),"retired identity tombstones")
    require(set(c["profiles_at_v1_activation"])=={"mpk.go.practical.v1","mpk.go.fixed.v0","mpk.rust.checked.v0"},"one future family with retained Go and Rust profiles")
    inventory=strict_json((w04/"surface-inventory.json").read_bytes());pins=inventory["discovered_files"]
    require(len(pins)==len({p["path"] for p in pins})==143,"predecessor coverage")
    for p in pins+inventory["baseline"]+[p for o in inventory["component_owners"] for p in o["existing_anchors"]]:require(sha((ROOT/p["path"]).read_bytes())==p["sha256"],"unchanged predecessor/baseline")
    initial=w04/"nat-type-selection-initial";d=strict_json((initial/"observations.json").read_bytes())
    for p in d["inputs"]:require(sha((initial/p["snapshot"]).read_bytes())==p["sha256"],"retained diagnostic source")
    require(sha((initial/"intended-strong-positive.mpcert").read_bytes())==d["candidate_sha256"],"retained diagnostic candidate")
    d=strict_json((initial/"history-alias-observations.json").read_bytes());require(sha((initial/"history-alias-producer.rs").read_bytes())==d["producer_sha256"] and sha((initial/"intended-strong-history-alias.mpcert").read_bytes())==d["candidate_sha256"],"retained history diagnostic")
    docs=[w04/"README.md",RECORD/"README.md",w04/"structural-limits/README.md",w04/"source-admission/README.md",w04/"storage-lifetimes/README.md",w04/"storage-views/README.md",w04/"overlay-contracts/README.md",w04/"variant-termination/README.md",w04/"nat-termination/README.md",w04/"type-error-zero-decisions.md",w04/"operation-loop-decisions.md",ROOT/"develop/docs/09_go_practical_subset_design-todo.md",ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",doc.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((doc.parent/unquote(target.split("#",1)[0])).is_file(),"local document link: "+target);links+=1
    require(list(map(int,re.findall(r"^## (\d+)\.",docs[-1].read_text(),re.M)))==list(range(1,40)),"ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs,101 identities/41 domains/10 schemas/26-node DAG/44 hash cases; retained192/81/5/10/24/13/39 evidence,143 predecessor pins,two diagnostics,thirteen documents/{links} links; W04 open")


if __name__=="__main__":main()
