"""Inspect each complete Rust-reported original stream while other roots continue."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import runpy
import subprocess
import time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
execution=root/"final/tests/result.json"
ex=json.loads(execution.read_text())
assert ex["status"] in {"running","passed"}
for source in ex["sources"]:
    assert seal(source["path"])==source
prior_path=root.parent/"core-interfaces/paired/result.json"
prior=json.loads(prior_path.read_text());binary=prior["go_binary"]
assert seal(binary["path"])==binary
for source in prior["go_production_sources"]:
    assert seal(source["path"])==source
negatives=runpy.run_path(str(root/"planned-controls.py"))["names"]()
positives={"product-foundation","honest-different-product"}|{f"balanced-shape-{n:02}" for n in range(65)}
expected=positives|negatives
out=root/"paired-ready";out.mkdir(exist_ok=False)
record={"status":"running","started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"Rust_execution_path":str(execution),"Rust_execution_start_snapshot":seal(execution),"go_binary":binary,"go_production_sources":prior["go_production_sources"],"prior_build_evidence":seal(prior_path),"cases":[],"scope":"Only complete closed certificate files with a complete original Rust JSON report are inspected. Rust checks full bytes before writing either file. Both complete raw files are hash/length-checked before and after actual Go verification. No prefix, format/fuel/resource substitute, missing-field default, source/program/profile/context or application proof authority. Full test-process completion is required separately before final success/staging/source edits."}


def save():
    (out/"result.json").write_text(json.dumps(record,indent=2)+"\n")


save();done=set()
try:
    while done!=expected:
        available=[]
        for certificate in sorted((execution.parent/"certificates").glob("*.mpcert")):
            name=certificate.stem
            assert name in expected
            if name in done:continue
            rust_path=certificate.with_suffix(".rust.json")
            try:
                raw=rust_path.read_bytes()
                if not raw.endswith(b"\n"):continue
                rust=json.loads(raw)
            except (FileNotFoundError,json.JSONDecodeError):continue
            available.append((certificate,rust_path,rust))
        if not available:
            terminal=json.loads(execution.read_text())
            if terminal["status"]!="running":
                record.update(status="partial_actual_streams_preserved",missing_names=sorted(expected-done),Rust_terminal_execution=seal(execution))
                save();print(json.dumps({"status":record["status"],"completed":len(done),"missing":len(expected-done)}),flush=True)
                raise SystemExit(1)
            time.sleep(1)
            continue
        for certificate,rust_path,rust in available:
            name=certificate.stem;before,report=seal(certificate),seal(rust_path)
            command=[binary["path"],"verify",str(certificate)];started=time.monotonic()
            result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
            stdout,stderr=out/(name+".stdout.log"),out/(name+".stderr.log")
            stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
            go=json.loads(result.stdout)
            row={"name":name,"certificate":before,"rust_report":report,"command":command,"exit_code":result.returncode,"seconds":time.monotonic()-started,"stdout":seal(stdout),"stderr":seal(stderr),"rust_observation":rust,"go_observation":go,"same_original_bytes":True}
            record["cases"].append(row);save()
            assert seal(certificate)==before and seal(rust_path)==report and not result.stderr
            if name in positives:
                assert result.returncode==0 and rust["verdict"]==go["verdict"]=="accepted"
                assert rust["axiom_count"]==0 and not rust["axiom_report"]["entries"]
                assert go["hashes"]==rust["hashes"] and go["declaration_count"]==rust["declaration_count"]
                assert go["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
            else:
                assert result.returncode!=0 and rust["verdict"]==go["verdict"]=="rejected"
                assert rust["error_code"]=="KERNEL_CORE_CHECK" and go["error_kind"]=="core_check"
                detail=json.loads(rust["error_detail"]);assert detail["code"]=="CORE_TYPE_MISMATCH"
                if detail["details"]["kind"]=="check_type_mismatch":
                    inferred,wanted=[detail["details"][key] for key in ["inferred_kind","expected_kind"]]
                    assert re.fullmatch(r"term [0-9]+ inferred "+re.escape(inferred)+" but expected "+re.escape(wanted),go["error_detail"])
                else:
                    assert detail["details"]["kind"]=="lambda_domain_mismatch" and go["error_detail"]=="lambda domain mismatch"
            done.add(name)
            print(json.dumps({"name":name,"verdict":go["verdict"],"seconds":row["seconds"]}),flush=True)
    terminal=json.loads(execution.read_text())
    while terminal["status"]=="running":
        time.sleep(1);terminal=json.loads(execution.read_text())
    assert terminal["status"]=="passed" and not terminal["changed_inputs_during_execution"]
    assert "4 passed; 0 failed" in (execution.parent/"stdout.log").read_text()
    record.update(status="passed",finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),Rust_terminal_execution=seal(execution),same_byte_streams=117,accepted_zero_axiom_positives=67,genuine_core_type_mismatches=50)
    save();print(json.dumps({"status":"passed","streams":117,"accepted":67,"type_rejections":50}),flush=True)
except SystemExit:
    raise
except BaseException:
    record["status"]="failed";save();raise
