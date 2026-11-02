#!/usr/bin/env python3
"""Check current storage-view evidence and the unchanged predecessor boundary."""
import re
from urllib.parse import unquote
from check_scope_report import require, sha, strict_json
from run_storage_view_probes import ROOT, RECORD, check_retained
from run_overlay_probes import retained_check as check_overlay
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04 = RECORD.parent
    files = sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files: strict_json(p.read_bytes())
    r = check_retained()
    require(len(r["ordinary_cases"]) == 10 and len(r["legacy"]) == 2, "storage coupling coverage")
    require(len(check_overlay()["outcomes"]) == 24, "retained overlay evidence")
    require(check_variant(strict_json((w04 / "variant-termination/observations.json").read_bytes())) == 13, "retained variant evidence")
    require(check_foundation(strict_json((w04 / "nat-termination/observations.json").read_bytes())) == 39, "retained foundation evidence")
    require(r["direct_child_parallelism"] == 1 and r["child_ceiling_seconds"] == 60 and r["ceiling_scope"] == "private measurement setting, not normative profile limits", "private resource scope")
    work = strict_json((w04 / "freeze-worklist.json").read_bytes()); draft = work["storage_view_draft"]
    require((draft["native_observations"], draft["ordinary_claims"], draft["accepted_zero_axiom"], draft["altered_type_mismatch"], draft["legacy_certificates"]) == (15, 10, 5, 5, 2), "storage draft counts")
    require(draft["production_checker_changed"] is False and draft["complete_source_normalization_proved"] is False and draft["remaining"], "storage source/production boundary")
    require(work["status"] == "in progress; not frozen or published" and work["open_decisions"], "W04 remains open")
    expected_executors = ["go-tools/go2vir/practical_storage.go", "crates/mpk-vc/src/go_practical_storage.rs"]
    expected_tests = ["go-tools/go2vir/practical_storage_test.go", "crates/mpk-vc/tests/go_practical_storage.rs"]
    for row in work["requirements"]:
        if row["id"] in {"GO-V1-R013", "GO-V1-R014", "GO-V1-R015"}:
            require(row["additional_production_executors"] == expected_executors and row["additional_test_owners"] == expected_tests, "planned storage executors")
    limits = strict_json((w04 / "candidate-limits.json").read_bytes())["storage_view_interpretation_draft"]
    require((limits["slot_candidate"], limits["cell_candidate"]) == (2, 128) and "freeze remain open" in limits["status"], "candidate counts not normative freeze")
    inventory = strict_json((w04 / "surface-inventory.json").read_bytes()); pins = inventory["discovered_files"]
    require(len(pins) == len({p["path"] for p in pins}) == 143, "predecessor coverage")
    for row in inventory["component_owners"]:
        if row["component"] in {"slices", "construction"}:
            require(row["additional_production_executors"] == expected_executors and row["additional_test_owners"] == expected_tests, "storage surface owners")
    for pin in pins + inventory["baseline"] + [p for o in inventory["component_owners"] for p in o["existing_anchors"]]:
        require(sha((ROOT / pin["path"]).read_bytes()) == pin["sha256"], "unchanged predecessor/baseline source")
    initial = w04 / "nat-type-selection-initial"; d = strict_json((initial / "observations.json").read_bytes())
    for pin in d["inputs"]: require(sha((initial / pin["snapshot"]).read_bytes()) == pin["sha256"], "retained diagnostic source")
    require(sha((initial / "intended-strong-positive.mpcert").read_bytes()) == d["candidate_sha256"], "retained diagnostic candidate")
    d = strict_json((initial / "history-alias-observations.json").read_bytes())
    require(sha((initial / "history-alias-producer.rs").read_bytes()) == d["producer_sha256"] and sha((initial / "intended-strong-history-alias.mpcert").read_bytes()) == d["candidate_sha256"], "retained history diagnostic")
    docs = [w04 / "README.md", RECORD / "README.md", w04 / "overlay-contracts/README.md", w04 / "variant-termination/README.md", w04 / "nat-termination/README.md", w04 / "type-error-zero-decisions.md", w04 / "operation-loop-decisions.md",
            ROOT / "develop/docs/09_go_practical_subset_design-todo.md", ROOT / "develop/docs/go-practical-implementation-traceability-ledger.md"]
    links = 0
    for doc in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", doc.read_text()):
            target = target.strip("<>")
            if re.match(r"^[a-z]+://", target) or target.startswith("#"): continue
            require((doc.parent / unquote(target.split("#", 1)[0])).is_file(), "local document link: " + target); links += 1
    require(list(map(int, re.findall(r"^## (\d+)\.", docs[-1].read_text(), re.M))) == list(range(1, 36)), "ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs, 15 native/model +10 ordinary +2 legacy cases, retained24/13/39 evidence, 143 predecessor pins, two diagnostics, nine documents/{links} links; W04 open")


if __name__ == "__main__": main()
