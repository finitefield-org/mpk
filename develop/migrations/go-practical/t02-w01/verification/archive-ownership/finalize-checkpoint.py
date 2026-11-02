"""Preserve initial scoped receipt and bind final related-process/source state."""
from pathlib import Path
import datetime,hashlib,json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
product_path=root.parent/"product-foundation/verification.json";product=json.loads(product_path.read_text())
assert product["status"]=="scoped_checks_passed" and product["all_actual_validation_handles_terminal"]
path=root/"verification.json";initial=path.read_bytes();d=json.loads(initial)
assert d["status"]=="scoped_checks_passed_strict_review_and_commit_pending" and d["all_archive_validation_handles_terminal"]
for source in [d["production_source"],d["regression_source"]]:assert seal(source["path"])==source
for check in d["checks"].values():assert seal(check["record"]["path"])==check["record"]
for key in ["independent_logged_original_inputs","independent_full_current_build_inputs","current_frontend_reproduction","frozen_inputs","observer_failure"]:assert seal(d[key]["path"])==d[key]
assert d["affected_Go_roots"]==14 and d["subtests"]==45 and d["current_frontend_sha256"]=="7a8a86f713dbc1e0098db6af12f5b24134f5603466e59dfc9cf25edabea2142a"
saved=root/"verification-before-final-product-consolidation.json";assert not saved.exists();saved.write_bytes(initial)
d.update(status="scoped_checks_passed",initial_scoped_receipt=seal(saved),final_state_observed_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),finalization_runner=seal(__file__),related_final_product_checkpoint=seal(product_path),unrelated_ordinary_product_handles_still_running=False,all_related_product_validation_handles_terminal=True,strict_changed_input_review="../product-foundation/input-review/changed-inputs-review.json",staged_or_committed=False)
assert not d["T02_W01_complete"] and not d["public_profile_activated"] and not d["immutable_original_application_replay_claimed"] and not d["CSharp_tests_executed"]
path.write_text(json.dumps(d,indent=2)+"\n")
print(json.dumps({"status":d["status"],"original_initial_receipt_preserved":True,"all_related_actual_handles_terminal":True,"Go_roots":14,"subtests":45,"T02_W01_complete":False}))
