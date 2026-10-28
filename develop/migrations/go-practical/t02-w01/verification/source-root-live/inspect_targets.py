"""Record the actual Cargo target configuration for the private observer."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
out=ROOT/'cargo-target-observation';out.mkdir(exist_ok=False)
command=['cargo','metadata','--offline','--locked','--no-deps','--format-version','1']
run=subprocess.run(command,cwd=REPO,capture_output=True,check=False)
(out/'stdout.json').write_bytes(run.stdout);(out/'stderr.log').write_bytes(run.stderr)
assert run.returncode==0
doc=json.loads(run.stdout);package=next(p for p in doc['packages'] if p['name']=='mpk-cli');target=next(t for t in package['targets'] if t['name']=='go_practical_live_roots')
assert target['kind']==['example'] and target['test'] is False
result={'status':'passed','selection_reason':'Inspect the actual example target after two exploratory forced test-cfg invocations incorrectly applied test flags to ordinary dependency libraries without their test harness/dev dependencies. The standard targeted Linux Clippy command is recorded separately.','command':command,'exit_code':run.returncode,'example_kind':target['kind'],'example_test_harness_enabled':target['test'],'stdout_sha256':hashlib.sha256(run.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(run.stderr).hexdigest(),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'example_test_harness_enabled':target['test']}))
