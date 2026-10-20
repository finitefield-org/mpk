"""Retain all complete current source inputs and independently reconstruct their IDs."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
source=root/"final/selected/captured-inputs.json";captured=json.loads(source.read_text())
assert not captured["application_proof_claimed"]
objects=captured["build_inputs"];assert len(objects)==170 and len({o["id"] for o in objects})==170
assert {o["capability"] for o in objects}=={"analysis-frontend"}
assert set(o["id"] for o in objects)<=set(captured["snapshot_source_ids"])
out=root/"final/owned-build-inputs";out.mkdir(exist_ok=False)
rows=[]
for obj in objects:
    relative=Path(obj["path"])
    assert not relative.is_absolute() and ".." not in relative.parts
    path=Path("go-tools/go2vir")/relative;record=seal(path)
    assert (record["sha256"],record["bytes"])==(obj["raw_sha256"],obj["bytes"])
    fields={k:v for k,v in obj.items() if k!="id"}
    assert set(fields)=={"schema","capability","path","kind","raw_sha256","bytes"}
    raw=json.dumps(fields,sort_keys=True,separators=(",",":"),ensure_ascii=True).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0"+raw).hexdigest()==obj["id"]
    destination=out/relative;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(path.read_bytes())
    rows.append({"original":record,"owned_copy":seal(destination),"source_object":obj})
reproduced_path=root/"final/selected/reproduced-builds.json";reproduced=json.loads(reproduced_path.read_text())
assert reproduced["byte_exact_equal"] and reproduced["independent_empty_caches"] and reproduced["source_inputs"]==170
assert reproduced["frontend_files"]==34 and reproduced["vendor_files"]==133
assert not reproduced["registered_practical_context"] and not reproduced["native_Linux_acceptance_claimed"]
one,two=reproduced["builds"];assert one==two
assert one["binary_sha256"]=="7a8a86f713dbc1e0098db6af12f5b24134f5603466e59dfc9cf25edabea2142a" and one["bytes"]==8592898
assert one["compiler_distribution_sha256"]=="a80de99f741cb69a8ed887c137b0be3d6368ddd051ac3145eb6feec7f85b6fb7"
outputs=[]
for iteration in [0,1]:
    path=root/f"final/selected/build-{iteration}-output.json";result=json.loads(path.read_text())
    assert result["error"]=="<nil>" and not result["stdout"] and not result["stderr"]
    assert all(flag in result["command"] for flag in ["-mod=vendor","-trimpath","-buildvcs=false","-pgo=off"])
    outputs.append(seal(path))
assert len(captured["auxiliary_differences"])==1
difference=captured["auxiliary_differences"][0]
assert difference["original_bytes"]==310 and difference["actual_bytes"]==309
assert any(o["id"]==difference["source_id"] and o["path"]=="vendor/golang.org/x/tools/go/ssa/TODO" and o["raw_sha256"]==difference["actual_sha256"] and o["bytes"]==309 for o in objects)
record={"status":"passed","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"captured_owned_source_records":seal(source),"reproduced_actual_builds":seal(reproduced_path),"independent_full_source_object_ids":170,"full_original_inputs":rows,"actual_build_outputs":outputs,"unchanged_original_uncompiled_auxiliary_difference":difference,"source_inputs":170,"frontend_files":34,"vendor_files":133,"two_actual_independent_empty_cache_builds_byte_equal":True,"binary_sha256":one["binary_sha256"],"binary_bytes":one["bytes"],"scope":"All170 complete original current Go/vendor/manifest bytes are hash/length-checked and independently source-ID reconstructed, then owned verbatim. The actual two fresh distinct-root empty-cache builds use those same sealed bytes and agree byte-for-byte. The original uncompiled TODO310/309 byte difference is retained explicitly. This supplies no registered context/source-program/immutable original application replay or native Linux receipt. No unrelated third build is repeated."}
p=root/"build-input-observation.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":"passed","whole_source_inputs":170,"two_fresh_builds":"identical","bytes":one["bytes"],"sha256":one["binary_sha256"]}))
