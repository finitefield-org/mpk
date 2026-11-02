#!/usr/bin/env python3
"""Check current W04 foundation proposal metadata and retained diagnostics."""
import re
from pathlib import Path
from urllib.parse import unquote
from check_scope_report import require,sha,strict_json
from check_termination_report import check
from run_termination_probes import ROOT,RECORD


def main():
    w04=RECORD.parent
    files=sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files:strict_json(p.read_bytes())
    require(check(strict_json((RECORD/"observations.json").read_bytes()))==39,"current private foundation evidence")
    initial=w04/"nat-type-selection-initial"
    first=strict_json((initial/"observations.json").read_bytes())
    require(first["scope"]=="retained intended-positive construction failure, not semantic negative or support","initial failure scope")
    for pin in first["inputs"]:require(sha((initial/pin["snapshot"]).read_bytes())==pin["sha256"],"initial source snapshot")
    require(sha((initial/"intended-strong-positive.mpcert").read_bytes())==first["candidate_sha256"],"initial candidate")
    second=strict_json((initial/"history-alias-observations.json").read_bytes())
    require(sha((initial/"history-alias-producer.rs").read_bytes())==second["producer_sha256"]
            and sha((initial/"intended-strong-history-alias.mpcert").read_bytes())==second["candidate_sha256"],"history diagnostic identity")
    for diagnostic in (first,second):
        require(set(diagnostic["observations"])=={"rust","go"} and all(o["exit_code"]==1 and o["report"]["verdict"]=="rejected" for o in diagnostic["observations"].values()),"intended failures never acceptance")
    work=strict_json((w04/"freeze-worklist.json").read_bytes())
    decision=work["foundation_feasibility_draft"]
    require(decision["general_sort0_strong_induction_checked"] is True
            and decision["ordinary_complete_source_variant_termination_proved"] is False
            and decision["production_checker_changed"] is False,"foundation/variant/public scope")
    require(work["status"]=="in progress; not frozen or published" and work["open_decisions"],"W04 open")
    require("CORE_BOOL_TYPE_CASES_V1.md" in work["specification_owners"],"additional specification ownership")
    row=next(r for r in work["requirements"] if r["id"]=="GO-V1-R019")
    require(row["additional_production_executors"]==["crates/mpk-core/src/bool_rec_type.rs","go-tools/mpk-checker-ref/bool_rec_type.go"],"planned core owners")
    inventory=strict_json((w04/"surface-inventory.json").read_bytes())
    pins=inventory["discovered_files"]
    require(len(pins)==len({r["path"] for r in pins})==143,"predecessor coverage")
    for pin in pins:require(sha((ROOT/pin["path"]).read_bytes())==pin["sha256"],"unchanged production source")
    for pin in inventory["baseline"]+[p for o in inventory["component_owners"] for p in o["existing_anchors"]]:
        require(sha((ROOT/pin["path"]).read_bytes())==pin["sha256"],"baseline/anchor identity")
    documents=[w04/"README.md",RECORD/"README.md",w04/"operation-loop-decisions.md",
               ROOT/"develop/docs/09_go_practical_subset_design-todo.md",
               ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for doc in documents:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",doc.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((doc.parent/unquote(target.split("#",1)[0])).is_file(),"local document link: "+target)
            links+=1
    sections=list(map(int,re.findall(r"^## (\d+)\.",documents[-1].read_text(),re.M)))
    require(sections==list(range(1,33)),"ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs, 39 private proof/interface/regression cases, two retained diagnostics, 143 predecessor pins, five documents/{links} links; W04 open")


if __name__=="__main__":main()
