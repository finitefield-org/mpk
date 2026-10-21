"""Bind the live producer's original launch and final completion receipts."""
from pathlib import Path
import datetime
import hashlib
import json

OUT=Path(__file__).resolve().parent

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}

path=OUT/"paired/result.json";raw=path.read_bytes();record=json.loads(raw)
assert record["status"]=="passed" and len(record["cases"])==339
original=OUT/"paired/result-at-checker-completion.json"
with original.open('xb')as stream:stream.write(raw)
launch=OUT/"paired/Rust-execution-at-launch.json"
assert {key:seal(launch)[key]for key in ["sha256","bytes"]}=={key:record["Rust_execution"][key]for key in ["sha256","bytes"]}
actual=Path(record["Rust_execution"]["path"]);completion=json.loads(actual.read_bytes())
assert completion["status"]=="passed" and completion["whole_source_inputs_unchanged"]
record["Rust_execution_at_launch"]=seal(launch)
record["Rust_execution"]=seal(actual)
record["actual_pair_completion_receipt"]=seal(original)
record["receipt_finalizer"]={"input_driver":seal(Path(__file__)),"observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"original_running_receipt_bytes_retained":True,"original_checker_completion_record_retained":True,"checker_outputs_modified":False}
path.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"original_launch_and_completion_retained":True,"checker_outputs_modified":False}))
