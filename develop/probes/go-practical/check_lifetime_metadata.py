#!/usr/bin/env python3
"""Check current lifetime evidence without rewriting historical checkpoints."""
import re
from urllib.parse import unquote
from check_scope_report import require, sha, strict_json
from run_lifetime_probes import ROOT, RECORD, check_retained
from run_storage_view_probes import check_retained as check_views
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04 = RECORD.parent
    files = sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files: strict_json(p.read_bytes())
    r = check_retained()
    require(len(r["ordinary_cases"]) == 5 and len(r["legacy"]) == 2, "lifetime coverage")
    require(len(check_views()["ordinary_cases"]) == 10, "retained owner/view evidence")
    require(len(check_overlay()["outcomes"]) == 24, "retained overlay evidence")
    require(check_variant(strict_json((w04 / "variant-termination/observations.json").read_bytes())) == 13, "retained variant evidence")
    require(check_foundation(strict_json((w04 / "nat-termination/observations.json").read_bytes())) == 39, "retained foundation evidence")
    require(r["direct_child_parallelism"] == 1 and r["child_ceiling_seconds"] == 60 and r["ceiling_scope"] == "private measurement setting, not normative profile limits", "private resource scope")
    work = strict_json((w04 / "freeze-worklist.json").read_bytes()); draft = work["lifetime_accounting_draft"]
    require((draft["native_observations"], draft["ordinary_claims"], draft["accepted_zero_axiom"], draft["altered_type_mismatch"], draft["legacy_certificates"], draft["targeted_tests"]) == (12, 5, 2, 3, 2, 24), "lifetime draft counts")
    require(draft["production_checker_changed"] is False and draft["complete_source_normalization_proved"] is False and draft["remaining"], "source/production boundary")
    require(draft["caller_supplied_live_set"] is False and draft["caller_supplied_escape_list"] is False, "derived root closure")
    require(draft["path"] == (RECORD / "README.md").relative_to(ROOT).as_posix() and draft["receipt"] == (RECORD / "observations.json").relative_to(ROOT).as_posix(), "lifetime artifact owners")
    require(work["status"] == "in progress; not frozen or published" and work["open_decisions"], "W04 remains open")
    executors = ["go-tools/go2vir/practical_storage.go", "crates/mpk-vc/src/go_practical_storage.rs"]
    tests = ["go-tools/go2vir/practical_storage_test.go", "crates/mpk-vc/tests/go_practical_storage.rs"]
    for row in work["requirements"]:
        if row["id"] in {"GO-V1-R013", "GO-V1-R014", "GO-V1-R015"}:
            require(row["additional_production_executors"] == executors and row["additional_test_owners"] == tests, "planned storage executors")
    limits = strict_json((w04 / "candidate-limits.json").read_bytes())["lifetime_accounting_interpretation_draft"]
    require((limits["slot_candidate"], limits["cell_candidate"], limits["model_iteration_allocations"], limits["model_iteration_peak_positive_slots"]) == (2, 128, 1000, 1), "candidate live-owner accounting")
    require(limits["native_gc_observed"] is False and limits["source_binding_proved"] is False and "freeze remain open" in limits["status"], "candidate proof/resource boundary")
    inventory = strict_json((w04 / "surface-inventory.json").read_bytes()); pins = inventory["discovered_files"]
    require(len(pins) == len({p["path"] for p in pins}) == 143, "predecessor coverage")
    for row in inventory["component_owners"]:
        if row["component"] in {"slices", "construction"}:
            require(row["additional_production_executors"] == executors and row["additional_test_owners"] == tests, "storage surface owners")
    for pin in pins + inventory["baseline"] + [p for o in inventory["component_owners"] for p in o["existing_anchors"]]:
        require(sha((ROOT / pin["path"]).read_bytes()) == pin["sha256"], "unchanged predecessor/baseline source")
    initial = w04 / "nat-type-selection-initial"; d = strict_json((initial / "observations.json").read_bytes())
    for pin in d["inputs"]: require(sha((initial / pin["snapshot"]).read_bytes()) == pin["sha256"], "retained diagnostic source")
    require(sha((initial / "intended-strong-positive.mpcert").read_bytes()) == d["candidate_sha256"], "retained diagnostic candidate")
    d = strict_json((initial / "history-alias-observations.json").read_bytes())
    require(sha((initial / "history-alias-producer.rs").read_bytes()) == d["producer_sha256"] and sha((initial / "intended-strong-history-alias.mpcert").read_bytes()) == d["candidate_sha256"], "retained history diagnostic")
    docs = [w04 / "README.md", RECORD / "README.md", w04 / "storage-views/README.md", w04 / "overlay-contracts/README.md", w04 / "variant-termination/README.md", w04 / "nat-termination/README.md", w04 / "type-error-zero-decisions.md", w04 / "operation-loop-decisions.md",
            ROOT / "develop/docs/09_go_practical_subset_design-todo.md", ROOT / "develop/docs/go-practical-implementation-traceability-ledger.md"]
    links = 0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", doc.read_text()):
            target = target.strip("<>")
            if re.match(r"^[a-z]+://", target) or target.startswith("#"): continue
            require((doc.parent / unquote(target.split("#", 1)[0])).is_file(), "local document link: " + target); links += 1
    require(list(map(int, re.findall(r"^## (\d+)\.", docs[-1].read_text(), re.M))) == list(range(1, 37)), "ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs, 12 native/model +5 ordinary +2 legacy, retained10/24/13/39 evidence, 143 predecessor pins, two diagnostics, ten documents/{links} links; W04 open")


if __name__ == "__main__": main()
