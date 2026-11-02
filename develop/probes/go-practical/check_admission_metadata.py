#!/usr/bin/env python3
"""Check source/closure draft scope and preserve prior evidence boundaries."""
import re
from urllib.parse import unquote
from check_scope_report import require, sha, strict_json
from run_source_admission import ROOT, RECORD, check_retained, COUNTS
from run_lifetime_probes import check_retained as check_lifetime
from run_storage_view_probes import check_retained as check_views
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04 = RECORD.parent
    files = sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files: strict_json(p.read_bytes())
    r = check_retained()
    require(len(check_lifetime()["ordinary_cases"]) == 5 and len(check_views()["ordinary_cases"]) == 10, "retained lifetime/view evidence")
    require(len(check_overlay()["outcomes"]) == 24, "retained overlay evidence")
    require(check_variant(strict_json((w04 / "variant-termination/observations.json").read_bytes())) == 13, "retained variant evidence")
    require(check_foundation(strict_json((w04 / "nat-termination/observations.json").read_bytes())) == 39, "retained foundation evidence")
    require(r["direct_child_parallelism"] == 1 and r["child_ceiling_seconds"] == 60, "private sequential measurement")
    work = strict_json((w04 / "freeze-worklist.json").read_bytes()); draft = work["source_admission_draft"]
    require((draft["ast_nodes"], draft["predeclared_builtins"], draft["original_sources"], draft["targeted_tests"]) == (56, 18, 81, 15), "source catalog coverage")
    require(draft["outcomes"] == COUNTS and draft["production_admission_executed"] is False and draft["ordinary_source_proofs_checked"] is False and draft["remaining"], "candidate/source/proof boundary")
    require(draft["path"] == (RECORD / "README.md").relative_to(ROOT).as_posix() and draft["receipt"] == (RECORD / "observations.json").relative_to(ROOT).as_posix(), "source admission artifact owners")
    require(work["status"] == "in progress; not frozen or published" and work["open_decisions"], "W04 remains open")
    inventory = strict_json((w04 / "surface-inventory.json").read_bytes()); pins = inventory["discovered_files"]
    require(len(pins) == len({p["path"] for p in pins}) == 143, "predecessor coverage")
    producers = {o["primary_production_executor"] for o in inventory["component_owners"]}
    tests = {o["primary_test_owner"] for o in inventory["component_owners"]}
    c = strict_json((RECORD / "catalog.json").read_bytes())
    for row in c["ast_nodes"] + c["builtins"]:
        require(row["primary_production_executor"] in producers and row["primary_test_owner"] in tests, "planned catalog execution/test ownership")
    for pin in pins + inventory["baseline"] + [p for o in inventory["component_owners"] for p in o["existing_anchors"]]:
        require(sha((ROOT / pin["path"]).read_bytes()) == pin["sha256"], "unchanged predecessor/baseline source")
    initial = w04 / "nat-type-selection-initial"; d = strict_json((initial / "observations.json").read_bytes())
    for pin in d["inputs"]: require(sha((initial / pin["snapshot"]).read_bytes()) == pin["sha256"], "retained diagnostic source")
    require(sha((initial / "intended-strong-positive.mpcert").read_bytes()) == d["candidate_sha256"], "retained diagnostic candidate")
    d = strict_json((initial / "history-alias-observations.json").read_bytes())
    require(sha((initial / "history-alias-producer.rs").read_bytes()) == d["producer_sha256"] and sha((initial / "intended-strong-history-alias.mpcert").read_bytes()) == d["candidate_sha256"], "retained history diagnostic")
    docs = [w04 / "README.md", RECORD / "README.md", w04 / "storage-lifetimes/README.md", w04 / "storage-views/README.md", w04 / "overlay-contracts/README.md", w04 / "variant-termination/README.md", w04 / "nat-termination/README.md", w04 / "type-error-zero-decisions.md", w04 / "operation-loop-decisions.md",
            ROOT / "develop/docs/09_go_practical_subset_design-todo.md", ROOT / "develop/docs/go-practical-implementation-traceability-ledger.md"]
    links = 0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", doc.read_text()):
            target = target.strip("<>")
            if re.match(r"^[a-z]+://", target) or target.startswith("#"): continue
            require((doc.parent / unquote(target.split("#", 1)[0])).is_file(), "local document link: " + target); links += 1
    require(list(map(int, re.findall(r"^## (\d+)\.", docs[-1].read_text(), re.M))) == list(range(1, 38)), "ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs, 56 AST/18 builtin rules and 81 original sources; retained5/10/24/13/39 evidence, 143 predecessor pins, two diagnostics, eleven documents/{links} links; W04 open")


if __name__ == "__main__": main()
