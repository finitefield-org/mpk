import json,os,subprocess,sys
from datetime import datetime,timezone
from hashlib import sha256
from pathlib import Path
commit=sys.argv[1]
repo=Path('/root/mpk-w09-pattern-routes-'+commit[:8])
reports=repo.with_name(repo.name+'-linux')
state=json.loads((reports/'status.json').read_bytes())
assert state['status']=='passed_targeted_tests_lint_format',state
assert state['unique_tests_passed']==7
assert [s['stage'] for s in state['stages']]==['routes','source-conditions','capture-compatibility','observations','scope-units','clippy','format']
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True)==''
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/with-pattern-routes'
manifest_path=base/'source-manifest.json'
manifest=json.loads(manifest_path.read_bytes())
assert (reports/'source-manifest.json').read_bytes()==manifest_path.read_bytes()
launch=json.loads((reports/'launch.json').read_bytes())
assert launch['commit']==commit
assert launch['manifest_sha256']==sha256(manifest_path.read_bytes()).hexdigest()
runner=base/'tool-sources/run-targeted.py'
assert launch['runner_sha256']==sha256(runner.read_bytes()).hexdigest()
unique={}
for group in ('source_hashes','fixture_hashes'):
    for name,digest in manifest[group].items():
        data=(repo/name).read_bytes()
        assert sha256(data).hexdigest()==digest,name
        assert subprocess.check_output(['/usr/bin/git','show',f'{commit}:{name}'],cwd=repo)==data,name
        unique[name]=digest
for stage in state['stages']:
    assert stage['exit_code']==0,stage
    assert sha256((reports/(stage['stage']+'.log')).read_bytes()).hexdigest()==stage['log_sha256']
for name,digest in state['test_binaries'].items():
    assert sha256(Path(name).read_bytes()).hexdigest()==digest,name
assert len(state['test_binaries'])==2
result={'status':'passed_independent_checkout_source_log_binary_audit','checked_at':datetime.now(timezone.utc).isoformat(),'commit':commit,'clean_checkout':True,'source_hashes_verified':len(manifest['source_hashes']),'fixture_hashes_verified':len(manifest['fixture_hashes']),'git_blob_files_verified':len(unique),'actual_stage_exit_codes':{s['stage']:s['exit_code'] for s in state['stages']},'unique_tests_passed':state['unique_tests_passed'],'test_binaries_verified':state['test_binaries'],'manifest_sha256':sha256(manifest_path.read_bytes()).hexdigest(),'runner_sha256':sha256(runner.read_bytes()).hexdigest(),'supervisor_status_sha256':sha256((reports/'status.json').read_bytes()).hexdigest(),'application_proofs_pending':987,'w09_complete':False,'full_t_gate':'deferred to T06-W12'}
(reports/'checkout-verification.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
print(json.dumps(result,sort_keys=True))
