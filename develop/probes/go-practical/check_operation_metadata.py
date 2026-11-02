#!/usr/bin/env python3
"""Check operation/evaluation draft consistency; do not declare W04 frozen."""
import re
from urllib.parse import unquote

from check_evaluation_report import check as check_evaluation
from check_scope_report import check as check_scope, exact, require, sha, strict_json
from evaluation_cases import RECORD, inputs
from operation_catalog import W04, ROOT, catalog
from scope_cases import RECORD as SCOPE_RECORD


def main():
    paths = sorted(p for p in W04.rglob("*.json") if "verification" not in p.relative_to(W04).parts)
    for path in paths:
        strict_json(path.read_bytes())
    table = strict_json((W04/"operation-table.json").read_bytes())
    require(exact(table, catalog()), "operation generator/draft consistency")
    require([r["id"] for r in table["operations"]] == [f"GP-OP-{i:02}" for i in range(1,57)],
            "complete unique logical operation family coverage")
    work = strict_json((W04/"freeze-worklist.json").read_bytes())
    inventory = strict_json((W04/"surface-inventory.json").read_bytes())
    owners = {r["component"]:r for r in inventory["component_owners"]}
    reqs = {r["id"]:r for r in work["requirements"]}
    require(len(owners)==len(inventory["component_owners"])==17
            and len(reqs)==len(work["requirements"])==24, "owner/requirement uniqueness")
    native_ids = {r["id"] for r in inputs()}
    for op in table["operations"]:
        require(op["component"] in owners and op["requirement"] in reqs, "operation ownership")
        owner = owners[op["component"]]
        for field in ("primary_production_executor","primary_test_owner"):
            require(op[field]==owner[field], "operation planned executor/test")
        require(op["evaluation_rule"] in table["evaluation_rules"], "operation evaluation rule")
        require(set(op["native_evaluation_vectors"]) <= native_ids, "native operation vector reference")
        require(op["signature"] and op["source_forms"] and op["result_relation"], "operation semantic row")
    pins = inventory["discovered_files"]
    require(len(pins)==len({r["path"] for r in pins})==143, "predecessor source coverage")
    for pin in pins:
        raw = (ROOT/pin["path"]).read_bytes()
        require(len(raw)==pin["bytes"] and sha(raw)==pin["sha256"], "predecessor source identity")
    for pin in inventory["baseline"]+[p for o in owners.values() for p in o["existing_anchors"]]:
        require(sha((ROOT/pin["path"]).read_bytes())==pin["sha256"], "baseline/anchor identity")
    require(check_evaluation(strict_json((RECORD/"observations.json").read_bytes()))==54,
            "native evaluation coverage")
    require(check_scope(strict_json((SCOPE_RECORD/"observations.json").read_bytes()))==49,
            "unchanged type/error/zero native evidence")
    decision = work["operation_loop_decision_draft"]
    require(decision["production_admission_executed"] is False and decision["ordinary_termination_proved"] is False
            and decision["operation_families"]==56 and decision["native_cases"]==54
            and decision["native_source_forms"]==35, "draft scope/count identity")
    document = ROOT/decision["path"]
    rules = re.findall(r"^\| ((?:EV|CT|LC)\d{2}) \|", document.read_text(), re.M)
    require(rules==decision["rule_ids"] and len(rules)==len(set(rules))==25, "evaluation/contract/loop rules")
    require(work["status"]=="in progress; not frozen or published" and work["open_decisions"]
            and table["pending"], "W04 pending freeze")
    documents = [W04/"README.md", W04/"type-error-zero-decisions.md", document,
                 ROOT/"develop/docs/09_go_practical_subset_design-todo.md",
                 ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links = 0
    for p in documents:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", p.read_text()):
            target = target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):
                continue
            path = unquote(target.split("#",1)[0])
            require((p.parent/path).is_file(), "local document link: "+path)
            links += 1
    sections = list(map(int,re.findall(r"^## (\d+)\.",documents[-1].read_text(),re.M)))
    require(sections==list(range(1,32)), "ordered ledger sections")
    print(f"PASS: {len(paths)} strict JSONs, 56 operation families/25 rules, 54 evaluation cases/35 forms, "
          f"49 unchanged scope cases, 143 predecessor pins, five documents/{links} local links; W04 open")


if __name__=="__main__":
    main()
