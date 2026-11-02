"""Seal completed canonical product evidence without promoting ongoing matrices."""
from pathlib import Path
import datetime
import hashlib
import json
import time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
live=root/"paired-ready/result.json"
while True:
    try:
        observation=json.loads(live.read_text())
        break
    except json.JSONDecodeError:time.sleep(0.01)
case=next(row for row in observation["cases"] if row["name"]=="product-foundation")
for key in ["certificate","rust_report","stdout","stderr"]:
    assert seal(case[key]["path"])==case[key]
assert case["certificate"]["sha256"]=="06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e"
assert case["certificate"]["bytes"]==11788 and case["exit_code"]==0 and case["stderr"]["bytes"]==0
rust,go=case["rust_observation"],case["go_observation"]
assert rust["verdict"]==go["verdict"]=="accepted" and rust["axiom_count"]==0
assert not rust["axiom_report"]["entries"] and rust["declaration_count"]==go["declaration_count"]==78
assert rust["hashes"]==go["hashes"]
assert rust["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
assert seal(observation["go_binary"]["path"])==observation["go_binary"]
for source in observation["go_production_sources"]:
    assert seal(source["path"])==source
construction=root/"explicit-result-types/construction/result.json"
value=json.loads(construction.read_text())
assert value["status"]=="passed" and not value["changed_inputs_during_execution"]
reader=root.parent/"word-signed-division/prerequisite-analysis/measure.py"
ns={"__file__":str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]",1)[0],str(reader),"exec"),ns)
Tables=ns["Tables"];current=Tables(case["certificate"])
assert len(current.declarations)==78 and len(current.terms)==1164
old_path=Path("develop/migrations/go-practical/t02-w01/verification/core-interfaces/final/rust-tests/certificates/core-foundation.mpcert")
old=Tables(seal(old_path));assert len(old.declarations)==35
assert old.record["sha256"]=="5b3b9c6e3eafa8f3c5652967cbad1f6d7eaa786d1ca23da894884a8a1b7ff373"
fp=current.fingerprints()
for name,original in old.fingerprints().items():assert fp[name]==original
checks={}
for suffix in ["final/clippy/result.json","final/format/result.json","final/diagnostic-list/result.json","explicit-helper-list/result.json","alias-consumers/initial/clippy/result.json","alias-consumers/initial/format/result.json"]:
    path=root/suffix;check=json.loads(path.read_text());assert check["status"]=="passed"
    for key in ["stdout","stderr"]:assert seal(check[key]["path"])==check[key]
    checks[suffix]=seal(path)
record={"status":"checked_generic_component_pending_full_consumers","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"entry_commit":"7154ec0b2ac2ab56ac21e87e018d2f2c02545366","runner":seal(__file__),"canonical_complete_case":case,"go_binary":observation["go_binary"],"go_production_sources":observation["go_production_sources"],"original_live_paired_record_path":str(live),"Rust_construction_execution":seal(construction),"original35_core_certificate":old.record,"all35_original_named_types_bodies_proofs_levels_families_unchanged":True,"component":{"declarations":78,"terms":1164,"bytes":11788,"axioms":0,"new_definitions":14,"new_laws":29,"logical_hashes":rust["hashes"]},"completed_selected_checks":checks,"frozen_preservation":seal(root/"frozen-design-preservation.json"),"retained_scalar_inputs":seal(root/"retained-inputs.json"),"full_expanded_matrix_complete":False,"full_type_sharing_matrix_complete":False,"semantic_control_matrix_complete":False,"T02_W01_complete":False,"public_profile_activated":False,"remaining_application_demands":169,"CSharp_tests_executed":False,"scope":"Only the completed canonical78-declaration product component and explicit finished lint/format/ignored-helper listings. Actual full raw certificates/reports/Go stdout/stderr and exact source/binary identities are retained per case, independently of the live overall result file. All65-arity matrices/controls/final reviews/Git audits remain pending. No source/program/context/application/profile or full56-family completion is implied."}
p=root/"generic-component.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"both_kernels":"accepted","declarations":78,"axioms":0,"old_core_identities":35}))
