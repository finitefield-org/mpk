"""Restore the exact original full application and registered SDK fixture."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
epoch=sys.argv[1];assert epoch.replace('-','').isalnum();out=ROOT/epoch;out.mkdir(exist_ok=False)
destination=Path('/private/tmp')/f'mpk-go-capture-binding-{epoch}';assert not destination.exists()
command=['python3','-B',str(ROOT/'restore_fixture.py'),str(destination)];start=time.monotonic()
with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
 code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
fixture=json.loads((destination/'fixture.json').read_bytes()) if (destination/'fixture.json').exists() else {}
okay=code==0 and fixture.get('status')=='passed' and fixture.get('original_application_entries')==74 and fixture.get('registered_sdk_files')==5577 and fixture.get('registered_sdk_bytes')==166354960 and fixture.get('all_original_evidence_files')==16880
result={'status':'passed' if okay else 'failed','selection_reason':'Restore the complete retained original application/dependency entries, every registered SDK byte and the entire original evidence set for the changed binding tests.','command':command,'exit_code':code,'seconds':time.monotonic()-start,'destination':str(destination),'fixture':fixture,'stdout_sha256':hashlib.sha256((out/'stdout.log').read_bytes()).hexdigest(),'stderr_sha256':hashlib.sha256((out/'stderr.log').read_bytes()).hexdigest(),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh','native_Linux_release_acceptance':False}
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','exit_code','seconds','destination']}),flush=True);raise SystemExit(not okay)
