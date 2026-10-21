"""Observe original rejected proof bytes in the unchanged independent Go core."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import time

OUT=Path(__file__).resolve().parent

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}

prior_path=OUT.parent/"core-interfaces/paired/result.json"
prior=json.loads(prior_path.read_bytes());binary=prior["go_binary"]
assert seal(binary["path"])==binary and all(seal(row["path"])==row for row in prior["go_production_sources"])
run=OUT/"failed-paired";run.mkdir(exist_ok=False)
driver=run/"check-failed-originals.py";driver.write_bytes(Path(__file__).read_bytes())
record={"status":"running","started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"go_binary":binary,"go_production_sources":prior["go_production_sources"],"actual_driver_prelaunch":seal(driver),"cases":[],"source_or_program_authority":False}

def save():
    (run/"result.json").write_text(json.dumps(record,indent=2)+"\n")

save()
try:
    for epoch in ["total-order","factored-boolean-total"]:
        path=OUT/"epochs"/epoch/"foundation/candidates/order-foundation.mpcert"
        rust_path=path.with_suffix(".rust.json");rust=json.loads(rust_path.read_bytes())
        command=[binary["path"],"verify",str(path)];start=time.monotonic()
        result=subprocess.run(command,capture_output=True,check=False)
        stdout=run/(epoch+".stdout.log");stderr=run/(epoch+".stderr.log");stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
        go=json.loads(result.stdout)
        record["cases"].append({"epoch":epoch,"input":seal(path),"Rust_report":seal(rust_path),"command":command,"exit_code":result.returncode,"seconds":time.monotonic()-start,"stdout":seal(stdout),"stderr":seal(stderr),"Rust":rust,"Go":go});save()
        assert not result.stderr and result.returncode!=0
        assert rust["verdict"]==go["verdict"]=="rejected"
        assert json.loads(rust["error_detail"])["code"]=="CORE_TYPE_MISMATCH" and go["error_kind"]=="core_check"
    assert seal(binary["path"])==binary and all(seal(row["path"])==row for row in prior["go_production_sources"])
    record.update(status="passed",finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
except BaseException:
    record["status"]="failed";save();raise
print(json.dumps({"status":record["status"],"original_same_byte_failures":len(record["cases"])}))
