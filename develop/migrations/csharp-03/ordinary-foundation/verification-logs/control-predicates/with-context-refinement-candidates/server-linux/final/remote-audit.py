import io,json,subprocess,tarfile
from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
repo=Path('/root/mpk-w09-context-refinement-candidates-eeffcb02')
reports=Path('/root/mpk-w09-context-refinement-candidates-eeffcb02-linux')
commit='eeffcb023a0c1051d5b0b26948ce7fc10616c0fc'
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True).strip()==''
state=json.loads((reports/'status.json').read_bytes())
assert state['status']=='passed_targeted_tests_lint_format' and state['unique_tests_passed']==8
assert [s['stage'] for s in state['stages']]==['context-units','candidate-units','ownership-unit','ownership-consumer','closed-default-consumer','clippy','format']
for stage in state['stages']:
    assert stage['exit_code']==0
    assert sha256((reports/(stage['stage']+'.log')).read_bytes()).hexdigest()==stage['log_sha256']
for binary,digest in state['test_binaries'].items():assert sha256(Path(binary).read_bytes()).hexdigest()==digest
manifest=json.loads((reports/'source-manifest.json').read_bytes())
all_files={}
for group in manifest:
    for name,digest in manifest[group].items():
        data=(repo/name).read_bytes();assert sha256(data).hexdigest()==digest,name
        assert subprocess.check_output(['/usr/bin/git','show',commit+':'+name],cwd=repo)==data,name
        assert name not in all_files or all_files[name]==digest
        all_files[name]=digest
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),source_commit=commit,clean_checkout=True,git_blob_files_verified=len(all_files),test_log_hashes_verified=True,test_binary_hashes_verified=True,manifest_sha256=sha256((reports/'source-manifest.json').read_bytes()).hexdigest(),unique_tests_passed=8,full_t_gate='deferred to T06-W12',application_proofs_pending=987)
(reports/'final-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+chr(10))
with tarfile.open(fileobj=__import__('sys').stdout.buffer,mode='w|gz') as tar:
    for name in ['status.json','source-manifest.json','launch.json','final-audit.json','supervisor.log']+[s['stage']+'.log' for s in state['stages']]:tar.add(reports/name,arcname=name)
