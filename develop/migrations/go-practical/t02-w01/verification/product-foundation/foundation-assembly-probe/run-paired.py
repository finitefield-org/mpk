"""Both current cores inspect full newly assembled inputs; no import/cap trust shortcut."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
execution=root/"initial/tests/result.json";ex=json.loads(execution.read_text())
assert ex["status"]=="passed" and not ex["changed_inputs_during_execution"]
for source in ex["sources"]:assert seal(source["path"])==source
prior_path=root.parent.parent/"core-interfaces/paired/result.json";prior=json.loads(prior_path.read_text())
binary=prior["go_binary"];assert seal(binary["path"])==binary
for source in prior["go_production_sources"]:assert seal(source["path"])==source
out=root/"paired";out.mkdir(exist_ok=False)
record={"status":"running","started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"actual_terminal_Rust_execution":seal(execution),"go_binary":binary,"go_production_sources":prior["go_production_sources"],"prior_actual_binary_build_evidence":seal(prior_path),"cases":[],"scope":"Complete4096-declaration full64 shift+product composition, genuine alternate same-name theorem proof and one false full-assembly Unit claim. Full original bytes and raw output in both cores; exact empty axiom-report hash binds zero axioms without a default for Go absent axiom_count. All original type/body/proof/level/family metadata retained; no new core/import/axiom/fuel/cap rule or complete56/source/private-context/program/application authority."}
result_path=out/"result.json"
def save():result_path.write_text(json.dumps(record,indent=2)+"\n")
save()
try:
    for name,accepted in [("full-shift-product",True),("valid-named-proof-variant",True),("false-unit-in-full-assembly",False)]:
        path=execution.parent/f"certificates/{name}.mpcert";report_path=path.with_suffix(".rust.json")
        certificate,rust_record=seal(path),seal(report_path);rust=json.loads(report_path.read_text())
        command=[binary["path"],"verify",str(path)];start=time.monotonic();completed=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
        stdout,stderr=out/f"{name}.stdout.log",out/f"{name}.stderr.log";stdout.write_bytes(completed.stdout);stderr.write_bytes(completed.stderr)
        go=json.loads(completed.stdout)
        row={"name":name,"certificate":certificate,"rust_report":rust_record,"command":command,"exit_code":completed.returncode,"seconds":time.monotonic()-start,"stdout":seal(stdout),"stderr":seal(stderr),"rust_observation":rust,"go_observation":go,"entire_original_bytes_sealed_before_and_after_actual_Go_execution":True}
        record["cases"].append(row);save()
        assert seal(path)==certificate and seal(report_path)==rust_record and not completed.stderr
        if accepted:
            assert completed.returncode==0 and rust["verdict"]==go["verdict"]=="accepted"
            assert rust["axiom_count"]==0 and not rust["axiom_report"]["entries"] and rust["hashes"]==go["hashes"]
            assert go["declaration_count"]==rust["declaration_count"] and go["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
        else:
            assert completed.returncode!=0 and rust["verdict"]==go["verdict"]=="rejected" and rust["error_code"]=="KERNEL_CORE_CHECK" and go["error_kind"]=="core_check"
            d=json.loads(rust["error_detail"]);assert d["code"]=="CORE_TYPE_MISMATCH" and d["details"]["kind"]=="check_type_mismatch"
            inferred,wanted=[d["details"][k]for k in ["inferred_kind","expected_kind"]]
            assert re.fullmatch(r"term [0-9]+ inferred "+re.escape(inferred)+" but expected "+re.escape(wanted),go["error_detail"])
        print(json.dumps({"name":name,"verdict":go["verdict"],"seconds":row["seconds"]}),flush=True)
    assert seal(binary["path"])==binary
    for source in prior["go_production_sources"]:assert seal(source["path"])==source
    for source in ex["sources"]:assert seal(source["path"])==source
    record.update(status="passed",same_byte_streams=3,zero_axiom_positives=2,genuine_core_type_mismatches=1,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
    print(json.dumps({"status":"passed","streams":3,"positives":2,"type_rejections":1}),flush=True)
except BaseException:
    record["status"]="failed";save();raise
