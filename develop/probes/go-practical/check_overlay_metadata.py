#!/usr/bin/env python3
"""Validate the current overlay draft and preserve its source/proof boundary."""
import re
from urllib.parse import unquote
from check_scope_report import require, sha, strict_json
from run_overlay_probes import ROOT, RECORD, retained_check
from overlay_contracts import OPERATORS
from check_variant_report import check as check_variant
from check_termination_report import check as check_foundation


def main():
    w04 = RECORD.parent
    files = sorted(p for p in w04.rglob("*.json") if "verification" not in p.relative_to(w04).parts)
    for p in files: strict_json(p.read_bytes())
    report = retained_check()
    require(report["native_executions"] == 2 and len(report["outcomes"]) == 24, "overlay current coverage")
    counts = {k: sum(r["outcome"] == k for r in report["outcomes"]) for k in ("normalized_unproved", "normalized_pending_variant", "configuration_rejection")}
    require(list(counts.values()) == [5, 1, 18], "overlay exact candidate boundaries")
    require(check_variant(strict_json((w04 / "variant-termination/observations.json").read_bytes())) == 13, "retained variant evidence")
    require(check_foundation(strict_json((w04 / "nat-termination/observations.json").read_bytes())) == 39, "retained foundation evidence")
    operators = strict_json((RECORD / "operators.json").read_bytes())
    require(operators["source_operators"] == sorted(OPERATORS) and len(OPERATORS) == 70
            and operators["normalized_operators"] == sorted((OPERATORS - {"local"}) | {"binding"}), "closed source/normalized operator sets")
    described = set(re.findall(r"`([a-z][a-z0-9_.]*)`", (RECORD / "README.md").read_text()))
    require(OPERATORS <= described, "every operator signature documented")
    pending = next(r for r in report["outcomes"] if r["id"] == "missing-for-variants-pending")["normalized"]
    require([p["ordinal"] for p in pending["pending_candidates"]] == [0, 1, 3], "nonrange totality stays pending")
    for row in report["outcomes"]:
        if row["outcome"] != "configuration_rejection":
            n = row["normalized"]
            require(n["proof_status"] == "unproved design candidate; all source/safety/domain/frame/post/termination members remain required", "no type-to-proof promotion")
            require(n["entry_storage"] == {"input_access": "read_only", "concurrent_external_mutation": "forbidden"}, "visible input storage domain")
    work = strict_json((w04 / "freeze-worklist.json").read_bytes())
    draft = work["overlay_contract_draft"]
    require((draft["source_expression_heads"], draft["normalized_expression_heads"], draft["source_overlay_vectors"], draft["normalized_unproved"], draft["normalized_pending_variant"], draft["configuration_rejections"]) == (70, 70, 24, 5, 1, 18), "overlay worklist counts")
    require(draft["production_admission_executed"] is False and draft["independent_artifact_validator_implemented"] is False and draft["proofs_discharged"] is False, "overlay implementation/proof boundary")
    require(work["status"] == "in progress; not frozen or published" and work["open_decisions"], "W04 remains open")
    require(draft["header_phase"] == "before current condition evaluation; stopped tag plus complete test/body/binding/post macrostep", "LC02 header scheduling preserved")
    inventory = strict_json((w04 / "surface-inventory.json").read_bytes())
    pins = inventory["discovered_files"]
    require(len(pins) == len({r["path"] for r in pins}) == 143, "predecessor coverage")
    for pin in pins + inventory["baseline"] + [p for o in inventory["component_owners"] for p in o["existing_anchors"]]:
        require(sha((ROOT / pin["path"]).read_bytes()) == pin["sha256"], "unchanged predecessor/baseline source")
    initial = w04 / "nat-type-selection-initial"
    diagnostic = strict_json((initial / "observations.json").read_bytes())
    for pin in diagnostic["inputs"]:
        require(sha((initial / pin["snapshot"]).read_bytes()) == pin["sha256"], "retained construction snapshot")
    require(sha((initial / "intended-strong-positive.mpcert").read_bytes()) == diagnostic["candidate_sha256"], "retained diagnostic candidate")
    other = strict_json((initial / "history-alias-observations.json").read_bytes())
    require(sha((initial / "history-alias-producer.rs").read_bytes()) == other["producer_sha256"]
            and sha((initial / "intended-strong-history-alias.mpcert").read_bytes()) == other["candidate_sha256"], "retained history diagnostic")
    documents = [w04 / "README.md", RECORD / "README.md", w04 / "variant-termination/README.md", w04 / "nat-termination/README.md", w04 / "operation-loop-decisions.md",
                 ROOT / "develop/docs/09_go_practical_subset_design-todo.md", ROOT / "develop/docs/go-practical-implementation-traceability-ledger.md"]
    links = 0
    for doc in documents:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", doc.read_text()):
            target = target.strip("<>")
            if re.match(r"^[a-z]+://", target) or target.startswith("#"): continue
            require((doc.parent / unquote(target.split("#", 1)[0])).is_file(), "local document link: " + target)
            links += 1
    require(list(map(int, re.findall(r"^## (\d+)\.", documents[-1].read_text(), re.M))) == list(range(1, 35)), "ordered ledger sections")
    print(f"PASS: {len(files)} strict JSONs, 24 overlay vectors/70+70 heads, 13+39 retained proof cases, 143 predecessor pins, two diagnostics, seven documents/{links} links; W04 open")


if __name__ == "__main__": main()
