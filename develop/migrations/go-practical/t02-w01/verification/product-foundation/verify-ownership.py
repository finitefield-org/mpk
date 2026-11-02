"""A real valid extra definition is accepted independently of exact owning import."""
from pathlib import Path
import hashlib
import json
import subprocess
import time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
execution=root/"eager-ownership/tests/result.json"
ex=json.loads(execution.read_text())
assert ex["status"]=="passed" and not ex["changed_inputs_during_execution"]
assert "1 passed; 0 failed" in (execution.parent/"stdout.log").read_text()
for source in ex["sources"]:assert seal(source["path"])==source
certificate=execution.parent/"certificates/honest-different-product.mpcert"
rust_path=certificate.with_suffix(".rust.json");rust=json.loads(rust_path.read_text())
assert rust["verdict"]=="accepted" and rust["declaration_count"]==79 and rust["axiom_count"]==0 and not rust["axiom_report"]["entries"]
prior=json.loads((root.parent/"core-interfaces/paired/result.json").read_text())
binary=prior["go_binary"];assert seal(binary["path"])==binary
for source in prior["go_production_sources"]:assert seal(source["path"])==source
out=root/"owning-extra-Go";out.mkdir(exist_ok=False)
command=[binary["path"],"verify",str(certificate)];start=time.monotonic()
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
stdout,stderr=out/"stdout.log",out/"stderr.log"
stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr);go=json.loads(result.stdout)
record={"status":"actual_extra_definition_observed","Rust_execution":seal(execution),"certificate":seal(certificate),"Rust_report":seal(rust_path),"go_binary":binary,"go_production_sources":prior["go_production_sources"],"command":command,"exit_code":result.returncode,"seconds":time.monotonic()-start,"stdout":seal(stdout),"stderr":seal(stderr),"Rust_observation":rust,"Go_observation":go,"scope":"Complete actual extra-definition input in both unchanged cores, independent of exact importer linkage. Full65arity matrices remain pending; no source/program/context/profile or application proof authority."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
assert result.returncode==0 and not result.stderr and go["verdict"]=="accepted"
assert go["hashes"]==rust["hashes"] and go["declaration_count"]==79
assert go["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
print(json.dumps({"status":"both_cores_accept_exact_import_rejects","declarations":79,"axioms":0}))
