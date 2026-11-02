"""Retain the complete corrected review transcript after it finishes."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
temp=Path('/private/tmp/mpk-go-live-root-review-final');assert not temp.exists();temp.mkdir(mode=0o700)
command=['python3','-B',str(ROOT/'review.py')];start=time.monotonic()
with (temp/'stdout.log').open('xb') as stdout,(temp/'stderr.log').open('xb') as stderr:
 code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
out=ROOT/'review-final';out.mkdir(exist_ok=False)
for name in ['stdout.log','stderr.log']:shutil.copyfile(temp/name,out/name)
review=json.loads((ROOT/'review.json').read_bytes()) if (ROOT/'review.json').exists() else {}
okay=code==0 and review.get('status')=='passed'
result={'status':'passed' if okay else 'failed','selection_reason':'Rerun the full original-data review after correcting the manifest aggregation; preserve the first failure separately.','command':command,'exit_code':code,'seconds':time.monotonic()-start,'review_sha256':hashlib.sha256((ROOT/'review.json').read_bytes()).hexdigest() if (ROOT/'review.json').exists() else None,'stdout_sha256':hashlib.sha256((out/'stdout.log').read_bytes()).hexdigest(),'stderr_sha256':hashlib.sha256((out/'stderr.log').read_bytes()).hexdigest(),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'seconds':round(result['seconds'],2)}),flush=True);raise SystemExit(not okay)
