"""Preserve the full original product inference failure in the unchanged Go core."""
from pathlib import Path
import hashlib
import json
import subprocess
import time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
prior_path=root.parent/"core-interfaces/paired/result.json"
prior=json.loads(prior_path.read_text())
binary=prior["go_binary"]
assert seal(binary["path"])==binary
for source in prior["go_production_sources"]:
    assert seal(source["path"])==source
certificate=root/"ordered-projections/construction/certificates/product-foundation.mpcert"
rust_path=certificate.with_suffix(".rust.json")
rust=json.loads(rust_path.read_text())
assert json.loads(rust["error_detail"])["code"]=="CORE_NOT_A_FUNCTION"
out=root/"original-Go"
out.mkdir(exist_ok=False)
command=[binary["path"],"verify",str(certificate)]
started=time.monotonic()
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
stdout,stderr=out/"stdout.log",out/"stderr.log"
stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
go=json.loads(result.stdout)
record={"status":"full_original_observation_preserved","certificate":seal(certificate),"rust_report":seal(rust_path),"go_binary":binary,"go_production_sources":prior["go_production_sources"],"prior_build_evidence":seal(prior_path),"command":command,"exit_code":result.returncode,"seconds":time.monotonic()-started,"stdout":seal(stdout),"stderr":seal(stderr),"rust_observation":rust,"go_observation":go,"scope":"Actual complete original candidate in both unchanged cores, not a semantic counterexample or prefix acceptance. Original recipe relied on delta unfolding during neutral function-type inference; repair must stay in ordinary emitted types without kernel/fuel changes."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
assert result.returncode!=0 and not result.stderr and go["verdict"]=="rejected" and go["error_kind"]=="core_check"
print(json.dumps({"status":record["status"],"Go_original_error":go["error_detail"]}))
