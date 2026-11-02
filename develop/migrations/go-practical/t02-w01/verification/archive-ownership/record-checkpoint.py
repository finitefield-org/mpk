"""Bind actual ownership regressions, affected consumers and complete new build evidence."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
checks={}
for attempt,modes in [("original",["regression"]),("reviewed",["regression"]),("final",["selected","format","vet","legacy"])]:
    for mode in modes:
        path=root/attempt/mode/"result.json";d=json.loads(path.read_text())
        assert d["status"]==("failed" if attempt!="final" else "passed")
        assert d["sources_unchanged_during_execution"]
        for key in ["stdout","stderr"]:assert seal(d[key]["path"])==d[key]
        for source in d["sources"]:
            saved=seal(Path(d["source_snapshot"])/source["path"])
            assert (saved["sha256"],saved["bytes"])==(source["sha256"],source["bytes"])
            if attempt=="final":assert seal(source["path"])==source
        assert seal(root/attempt/"runner.py")["sha256"]==d["runner"]["sha256"]
        if attempt!="final":assert len(d["tests_failed"])==5 and not d["root_tests_passed"]
        checks[attempt+"/"+mode]={"record":seal(path),"seconds":d["wall_seconds"]}
selected=json.loads((root/"final/selected/result.json").read_text())
assert len(selected["root_tests_passed"])==14 and len(selected["subtests_passed"])==45 and not selected["tests_failed"]
legacy=json.loads((root/"final/legacy/result.json").read_text())
assert legacy["command"][legacy["command"].index("-run")+1]=="^$" and not legacy["root_tests_passed"] and not legacy["subtests_passed"]
original_inputs=root/"observed-inputs/result.json";observed=json.loads(original_inputs.read_text())
assert observed["status"]=="passed" and observed["full_original_first_and_cached_inputs_identical_before_and_after_fix"]
for source in observed["original_bytes"]:assert seal(source["path"])==source
build_path=root/"build-input-observation.json";build=json.loads(build_path.read_text())
assert build["status"]=="passed" and build["independent_full_source_object_ids"]==170
for source in build["full_original_inputs"]:
    for key in ["original","owned_copy"]:assert seal(source[key]["path"])==source[key]
    assert (source["original"]["sha256"],source["original"]["bytes"])==(source["owned_copy"]["sha256"],source["owned_copy"]["bytes"])
assert build["two_actual_independent_empty_cache_builds_byte_equal"]
frozen=json.loads((root.parent/"word-signed-equality/frozen-design-preservation.json").read_text())["inputs"]
assert len(frozen)==255
for source in frozen:assert seal(source["path"])==source
(root/"frozen-design-preservation.json").write_text(json.dumps({"status":"passed","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"frozen_inputs":255,"inputs":frozen,"mismatches":[],"scope":"All frozen original specification/schema/vector/recipe inputs unchanged; archive ownership changes no semantic/source-object kind/limit/hash/scanner policy."},indent=2)+"\n")
requirements_path=Path("develop/migrations/go-practical/t02-w01/requirements.json")
assert str(requirements_path) not in {r["path"] for r in frozen}
requirements=json.loads(requirements_path.read_text());by={r["id"]:r for r in requirements["requirements"]};assert len(by)==7
compiler=by["T02-W01-R03"]
compiler.setdefault("prior_frontend_build_checkpoints",[]).append(compiler["latest_frontend_build_checkpoint"])
compiler["latest_frontend_build_checkpoint"]={"evidence":"verification/archive-ownership/final/selected/reproduced-builds.json","sealed_original_inputs":170,"frontend_files":34,"vendor_files":133,"independent_empty_cache_builds":2,"binary_bytes":build["binary_bytes"],"binary_sha256":build["binary_sha256"],"registered_practical_context":False,"native_Linux_acceptance_claimed":False}
by["T02-W01-R05"]["latest_archive_ownership_checkpoint"]={"evidence":"verification/archive-ownership/verification.json","scope":"Initial and cache-hit returned archive values separately own exact original raw bytes and complete member maps. Mutating a returned map/byte slice no longer changes the session's verified checksum inventory, permits changed dependency content or changes sealed original bytes. Shared namespace/source budget charges remain identical. All4 actual original regressions and complete logged ZIP/member/main inputs are retained. Affected14 roots45 subtests, vet/format and Go1.23 compile-only compatibility pass; current170input frontend rebuilds agree byte-for-byte. Complete original package/export/context loading and immutable application replay remain due.","immutable_application_replay_claimed":False,"public_profile_activated":False}
by["T02-W01-R07"]["latest_capture_scoped_checks"]={"evidence":"verification/archive-ownership/verification.json","affected_Go_roots":14,"subtests":45,"new_ownership_regression_cases":4,"original_failing_epochs_preserved":2,"complete_original_inputs_observed_independently":True,"two_fresh_current_frontend_builds_identical":True,"legacy":"Go1.23 compile/link only; no tests executed","vet_format":"passed","unaffected_prior_suites_repeated":False,"CSharp_tests_executed":False,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
assert by["T02-W01-R06"]["status"]=="pending" and not requirements["T02_W01_complete"] and not requirements["public_profile_activated"]
requirements_path.write_text(json.dumps(requirements,ensure_ascii=False,indent=2)+"\n")
record={"status":"scoped_checks_passed_strict_review_and_commit_pending","entry_commit":"7154ec0b2ac2ab56ac21e87e018d2f2c02545366","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"production_source":seal("go-tools/go2vir/practical_archive.go"),"regression_source":seal("go-tools/go2vir/practical_archive_ownership_test.go"),"actual_bug":"The first parsed result and every cached result shared member-map/source-byte allocations with the private transaction cache. A returned map could authorize changed member bytes through a later fresh cache lookup, and returned bytes could change captured original buffers. Each side now owns independent exact raw bytes/member maps.","checks":checks,"affected_Go_roots":14,"subtests":45,"actual_original_ownership_regressions":4,"independent_logged_original_inputs":seal(original_inputs),"independent_full_current_build_inputs":seal(build_path),"current_frontend_reproduction":seal(root/"final/selected/reproduced-builds.json"),"current_frontend_sha256":build["binary_sha256"],"current_frontend_bytes":build["binary_bytes"],"frozen_inputs":seal(root/"frozen-design-preservation.json"),"observer_failure":seal(root/"observer-failures/result.json"),"observer_repair":"Losslessly concatenate all original Go JSON Output fragments in recorded order, then inspect complete logged payload lines. Initial line-fragment observer source/stdout/stderr remains. No source/input/JSON normalization or record exclusion.","all_archive_validation_handles_terminal":True,"unrelated_ordinary_product_handles_still_running":True,"staged_or_committed":False,"strict_changed_input_review":"pending after all relevant actual processes terminal","CSharp_tests_executed":False,"T02_W01_complete":False,"public_profile_activated":False,"native_Linux_acceptance_claimed":False,"immutable_original_application_replay_claimed":False,"remaining_application_demands":169,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp","remaining_scope":"Full56family foundation/descriptor/private context/registry and source/carrier/type/operation/state/program bindings, original package/export/selected-closure immutable capture/loading/replay, all169 application demands and original T02-T11 work/gates remain required. Archive/private buffer ownership is an input-integrity prerequisite, not original application verification authority."}
p=root/"verification.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"Go_roots":14,"subtests":45,"current_frontend_sha256":build["binary_sha256"],"frozen_inputs_unchanged":255}))
