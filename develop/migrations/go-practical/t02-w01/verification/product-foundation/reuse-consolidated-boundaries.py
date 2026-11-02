"""Bind current main boundary roots to exact prior whole-byte Go executions."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
execution=root/"consolidated/tests/result.json";current=json.loads(execution.read_text())
assert current["status"]=="passed" and not current["changed_inputs_during_execution"]
assert "3 passed; 0 failed" in Path(current["stdout"]["path"]).read_text()
for source in current["sources"]:assert seal(source["path"])==source
prior_path=root/"paired-ready/result.json";prior=json.loads(prior_path.read_text())
assert prior["status"]=="partial_actual_streams_preserved" and len(prior["cases"])==112
binary=prior["go_binary"];assert seal(binary["path"])==binary
for source in prior["go_production_sources"]:assert seal(source["path"])==source
out=root/"consolidated/paired-boundaries";out.mkdir(exist_ok=False)
rows=[]
for certificate in sorted((execution.parent/"certificates").glob("*.mpcert")):
    name=certificate.stem;record=seal(certificate);rust_path=certificate.with_suffix(".rust.json");rust=json.loads(rust_path.read_text())
    matched=[row for row in prior["cases"] if row["name"]==name and (row["certificate"]["sha256"],row["certificate"]["bytes"])==(record["sha256"],record["bytes"])]
    assert len(matched)==1,name
    previous=matched[0]
    for key in ["certificate","rust_report","stdout","stderr"]:assert seal(previous[key]["path"])==previous[key]
    assert Path(previous["certificate"]["path"]).read_bytes()==certificate.read_bytes()
    assert previous["rust_observation"]==rust
    assert not Path(previous["stderr"]["path"]).read_bytes()
    go=json.loads(Path(previous["stdout"]["path"]).read_text());assert go==previous["go_observation"]
    if rust["verdict"]=="accepted":
        assert previous["exit_code"]==0 and go["verdict"]=="accepted" and rust["axiom_count"]==0 and not rust["axiom_report"]["entries"]
        assert rust["hashes"]==go["hashes"] and go["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        assert rust["verdict"]==go["verdict"]=="rejected" and previous["exit_code"]!=0
        assert json.loads(rust["error_detail"])["code"]=="CORE_TYPE_MISMATCH" and go["error_kind"]=="core_check"
    rows.append({"name":name,"current_certificate":record,"current_actual_Rust_report":seal(rust_path),"actual_prior_whole_byte_Go_execution":previous,"entire_original_certificate_and_Rust_observation_identical":True})
assert len(rows)==52
assert sum(r["actual_prior_whole_byte_Go_execution"]["go_observation"]["verdict"]=="accepted" for r in rows)==2
assert seal(binary["path"])==binary
for source in prior["go_production_sources"]:assert seal(source["path"])==source
record={"status":"passed","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"current_actual_terminal_Rust_execution":seal(execution),"prior_actual_terminal_Go_execution":seal(prior_path),"go_binary":binary,"go_production_sources":prior["go_production_sources"],"same_byte_streams":52,"accepted_zero_axiom_positives":2,"genuine_core_type_mismatches":50,"new_Go_kernel_executions":0,"rows":rows,"scope":"Current three main boundary roots actually execute after retiring the redundant expanded consumer prototype. Every complete generated certificate and Rust observation equals its exact completed original input. Reuse requires original full Go stdout/stderr, raw input/report and unchanged actual binary/all13 production source pins. No repeated unchanged Go verification, missing-field axiom default, source/JSON rewrite, prefix/resource substitute, omitted control or source/program/context/profile/application proof authority. Complete all65 shared product consumer matrix remains separately active and checked in both production cores."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":"passed","whole_byte_streams_reused":52,"positives":2,"type_rejections":50}))
