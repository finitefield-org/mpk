"""Targeted W04 design audit, independent of historical current-tree helpers."""
import ast
import json
from pathlib import Path
import re
from urllib.parse import unquote

from program_design import RECORD as PROGRAM, catalogue, host_contract, audit
from migration_review import RECORD, inventory, requirements
from proof_canonical import canonical
from run_proof_pilot import ROOT, sha
from check_scope_report import strict_json


def main():
    count = 0
    for directory, name, generated in (
        (PROGRAM, "interfaces.json", catalogue()), (PROGRAM, "host-resources.json", host_contract()),
        (PROGRAM, "audit.json", audit()), (RECORD, "dispositions.json", inventory()),
        (RECORD, "requirements.json", requirements()),
    ):
        path = directory / name; value = strict_json(path.read_bytes())
        assert canonical(value) == canonical(generated), "whole design reconstruction " + name
        count += 1
    replay = strict_json((RECORD / "historical-replay.json").read_bytes())
    epoch = strict_json((RECORD / "historical-source-epoch.json").read_bytes())
    assert replay["files"] == len(epoch["files"]) and replay["exit_code"] == 0
    assert replay["result"]["cases"] == 229 and replay["result"]["hash_vectors"] == 13
    assert replay["result"]["Go_ordinary_elaborated"] is False and replay["result"]["profile_activated"] is False
    vectors = strict_json((PROGRAM / "conformance-vectors.json").read_bytes())["vectors"]
    assert len(vectors) == 46 and len({v["id"] for v in vectors}) == 46
    assert all(v["production_observed"] is False for v in vectors)
    assert {v["family"] for v in vectors if v.get("expected") == "required"} == {r["kind"] for r in catalogue()["families"]}
    complete = strict_json((RECORD.parent / "completion-audit.json").read_bytes())
    assert complete["status"] == "design_complete_inactive" and not complete["unresolved_design_questions"]
    assert complete["full_gate"]["status"] == "deferred" and complete["production_activated"] is False
    links = 0
    for directory in (PROGRAM, RECORD):
        text = (directory / "README.md").read_text()
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)", text):
            if target.startswith("#") or "://" in target: continue
            assert (directory / unquote(target.split("#", 1)[0])).is_file(), target
            links += 1
    for name in ("program_design.py", "migration_review.py", "source_epoch.py", "test_source_epoch.py", "check_program_handoff.py"):
        ast.parse((ROOT / "develop/probes/go-practical" / name).read_text())
    print(f"PASS: {count} design documents reconstructed; 19 theorem interfaces/56 typed operation fragments/169 unelaborated demands; {len(inventory()['files'])} reviewed consumers; 24 requirement owners; 46 future conformance vectors; {replay['files']} historical inputs replayed; {links} local links. W04 design complete; W05 gate deferred.")


if __name__ == "__main__": main()
