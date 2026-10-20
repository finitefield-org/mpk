"""List the separately born-ignored second original-candidate diagnostic."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
previous=json.loads((root/"final/clippy/result.json").read_text())
assert previous["status"]=="passed"
for source in previous["sources"]:
    assert seal(source["path"])==source
out=root/"explicit-helper-list";out.mkdir(exist_ok=False)
(out/"runner.py").write_bytes(Path(__file__).read_bytes())
env=os.environ.copy();env.update(previous["environment"])
for name in ["RUST_MIN_STACK","MPK_CORE_MAX_STEPS","MPK_CORE_MAX_SECONDS"]:env.pop(name,None)
command=["cargo","test","--offline","--locked","-p","mpk-vc","--test","go_practical_product_foundation_explicit_diagnostic","--","--ignored","--list"]
started=time.monotonic()
result=subprocess.run(command,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
stdout,stderr=out/"stdout.log",out/"stderr.log"
stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
changed=[r for r in previous["sources"] if seal(r["path"])!=r]
record={"status":"passed" if result.returncode==0 and not changed else "failed","command":command,"exit_code":result.returncode,"seconds":time.monotonic()-started,"observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"environment":previous["environment"],"sources":previous["sources"],"source_snapshot":previous["source_snapshot"],"runner":seal(__file__),"changed_inputs_during_execution":changed,"stdout":seal(stdout),"stderr":seal(stderr),"CSharp_tests_executed":False,"scope":"Listing only the separately ignored complete original computed-constructor diagnostic; no original diagnostic/kernel stream or unaffected suite is rerun. Full gate stays deferred to T02-W04."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
assert record["status"]=="passed"
assert result.stdout.decode().strip()=="identify_first_explicit_product_type_failure: test\n\n1 test, 0 benchmarks"
print(json.dumps({"status":record["status"],"ignored_helpers_listed":1,"seconds":record["seconds"]}))
