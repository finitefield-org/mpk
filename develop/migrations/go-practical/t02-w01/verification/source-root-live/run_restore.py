"""Independently restore every retained native original evidence file."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
epoch=sys.argv[1];assert epoch.replace('-','').isalnum()
run=ROOT/epoch;storage=json.loads((run/'evidence-storage.json').read_bytes());assert storage['status']=='passed'
dest=Path('/private/tmp')/f'mpk-go-live-evidence-restored-{epoch}';assert not dest.exists()
command=['python3','-B',str(ROOT/'restore_evidence.py'),epoch,str(dest)]
start=time.monotonic()
with (run/'restore.stdout').open('xb') as stdout,(run/'restore.stderr').open('xb') as stderr:
 code=subprocess.run(command,cwd=REPO,stdout=stdout,stderr=stderr,check=False).returncode
reported=json.loads((run/'restore.stdout').read_bytes()) if code==0 else {}
okay=code==0 and reported.get('status')=='passed' and reported.get('all_original_files_and_modes_match') is True and reported.get('restored_files')==storage['original_files'] and reported.get('restored_bytes')==storage['original_bytes']
result={'status':'passed' if okay else 'failed','selection_reason':'Restore every original file and directory from the complete encoded archive; compare each byte hash, size and mode and the full reconstructed archive digest.','command':command,'exit_code':code,'seconds':time.monotonic()-start,'restored_files':reported.get('restored_files'),'restored_bytes':reported.get('restored_bytes'),'restored_directories':reported.get('restored_directories'),'all_original_files_and_modes_match':reported.get('all_original_files_and_modes_match',False),'destination':str(dest),'stdout_sha256':hashlib.sha256((run/'restore.stdout').read_bytes()).hexdigest(),'stderr_sha256':hashlib.sha256((run/'restore.stderr').read_bytes()).hexdigest(),'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
(run/'restore-check.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','restored_files','restored_bytes','restored_directories','all_original_files_and_modes_match']}),flush=True);raise SystemExit(not okay)
