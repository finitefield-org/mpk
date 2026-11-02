#!/usr/bin/env python3
"""Check this partial W04 draft and retained identities, without freezing it."""
import re
from urllib.parse import unquote

from check_scope_report import check, require, sha, strict_json
from scope_cases import RECORD, ROOT, VECTORS

W04 = VECTORS.parent


def main():
    # All draft JSON, including raw native input/output, is duplicate-key checked.
    json_paths = sorted(path for path in W04.rglob("*.json")
                        if "verification" not in path.relative_to(W04).parts)
    for path in json_paths:
        strict_json(path.read_bytes())
    work = strict_json((W04 / "freeze-worklist.json").read_bytes())
    inventory = strict_json((W04 / "surface-inventory.json").read_bytes())
    vectors = strict_json(VECTORS.read_bytes())
    decision = work["semantic_decision_draft"]
    require(work["status"] == "in progress; not frozen or published" and work["open_decisions"],
            "W04 must remain open")
    require(decision["production_admission_executed"] is False
            and decision["native_cases"] == 49
            and check(strict_json((RECORD / "observations.json").read_bytes())) == 49,
            "native evidence is not production admission")
    owners = {row["component"]: row for row in inventory["component_owners"]}
    requirements = {row["id"]: row for row in work["requirements"]}
    require(len(owners) == len(inventory["component_owners"]) == 17
            and len(requirements) == len(work["requirements"]) == 24,
            "unique owner/requirement coverage")
    identities = [row["id"] for row in vectors["cases"]]
    require(len(identities) == len(set(identities)) == 49, "unique source vector IDs")
    for row in vectors["cases"]:
        require(row["requirement"] in requirements and row["primary_component"] in owners,
                "vector requirement/component ownership")
        owner = owners[row["primary_component"]]
        for field in ("primary_production_executor", "primary_test_owner"):
            require(row[field] == owner[field], "planned vector executor/test identity")
        require(owner["executor_status"] == "planned; not implemented or validated",
                "draft executor status")
    pin_count = 0
    pin_paths = set()
    for row in inventory["discovered_files"]:
        raw = (ROOT / row["path"]).read_bytes()
        require(sha(raw) == row["sha256"] and len(raw) == row["bytes"], "predecessor file identity")
        pin_paths.add(row["path"])
        pin_count += 1
    require(pin_count == len(pin_paths) == 143, "discovered predecessor file coverage")
    for row in inventory["baseline"] + [
        pin for owner in owners.values() for pin in owner["existing_anchors"]
    ]:
        require(sha((ROOT / row["path"]).read_bytes()) == row["sha256"], "baseline/anchor identity")
    for key in ("path", "vectors", "native_receipt"):
        require((ROOT / decision[key]).is_file(), "semantic decision artifact")
    rules = re.findall(r"^\| ((?:TG|ER|ZS)\d{2}) \|", (ROOT / decision["path"]).read_text(), re.M)
    require(rules == decision["rule_ids"] and len(rules) == len(set(rules)) == 26, "semantic rule coverage/order")
    documents = [
        W04 / "README.md", ROOT / decision["path"],
        ROOT / "develop/docs/09_go_practical_subset_design-todo.md",
        ROOT / "develop/docs/go-practical-implementation-traceability-ledger.md",
    ]
    links = 0
    for document in documents:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", document.read_text()):
            target = target.strip("<>")
            if re.match(r"^[a-z]+://", target) or target.startswith("#"):
                continue
            path = unquote(target.split("#", 1)[0])
            require((document.parent / path).is_file(), "local document link: " + path)
            links += 1
    ledger = documents[-1].read_text()
    sections = list(map(int, re.findall(r"^## (\d+)\.", ledger, re.M)))
    require(sections == list(range(1, 31)), "ordered ledger sections")
    print(f"PASS: {len(json_paths)} strict JSON files, 49 vectors/26 rules, "
          f"24 requirements/17 owners, {pin_count} predecessor pins, "
          f"four documents/{links} local links; W04/T01 remain open")


if __name__ == "__main__":
    main()
