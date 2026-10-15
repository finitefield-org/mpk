import json,subprocess,sys,tarfile
from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
commit='69c32d10503676ae6947f8ababa09e9f84720c8d'
repo=Path('/root/mpk-w09-concrete-type-proofs-'+commit[:8])
reports=repo.with_name(repo.name+'-linux')
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-type-proofs'
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True).strip()==''
state=json.loads((reports/'status.json').read_bytes())
assert state['status']=='passed_targeted_tests_lint_format' and state['unique_tests_passed']==5
assert [s['stage'] for s in state['stages']]==['candidate-units','original-type-proofs','original-type-pins','clippy','format','support-format']
assert state['supplied_concrete_type_proofs']==81
assert state['internal_type_instances_pending']==6 and state['application_proofs_pending']==987
assert state['exported_certificates_match_local'] is True
for stage in state['stages']:
    assert stage['exit_code']==0
    assert sha256((reports/(stage['stage']+'.log')).read_bytes()).hexdigest()==stage['log_sha256']
for binary,digest in state['test_binaries'].items():
    assert sha256(Path(binary).read_bytes()).hexdigest()==digest
assert len(state['test_binaries'])==2
manifest=json.loads((reports/'source-manifest.json').read_bytes())
assert (reports/'source-manifest.json').read_bytes()==(base/'source-manifest.json').read_bytes()
all_files={}
for group in ('source_hashes','fixture_hashes'):
    for name,digest in manifest[group].items():
        data=(repo/name).read_bytes()
        assert sha256(data).hexdigest()==digest,name
        assert subprocess.check_output(['/usr/bin/git','show',commit+':'+name],cwd=repo)==data,name
        assert name not in all_files or all_files[name]==digest
        all_files[name]=digest
for name in ('source-manifest.json','expected-certificates.json','tool-sources/run-targeted.py'):
    path=base/name
    assert subprocess.check_output(['/usr/bin/git','show',commit+':'+str(path.relative_to(repo))],cwd=repo)==path.read_bytes()
catalog=json.loads((base/'expected-certificates.json').read_bytes())
assert len(catalog)==91
actual={p.name:sha256(p.read_bytes()).hexdigest() for p in (reports/'certificates').iterdir() if p.is_file()}
assert actual==catalog
assert {'certificates/'+name:digest for name,digest in actual.items()}==state['exported_certificate_hashes']
metadata=[json.loads(p.read_bytes()) for p in sorted((reports/'certificates').glob('*.json'))]
assert len(metadata)==45
assert sum(len(m['proofs']) for m in metadata)==81
assert sum(not m['proofs'] for m in metadata)==10
assert sum(len(m['pending_type_instances']) for m in metadata)==6
assert sum(len(m['pending_proof_ids']) for m in metadata)==987
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
launch=json.loads((reports/'launch.json').read_bytes())
assert launch['commit']==commit and launch['public_git_source'] is True and launch['clean_checkout'] is True
assert launch['manifest_sha256']==sha256((base/'source-manifest.json').read_bytes()).hexdigest()
assert launch['runner_sha256']==sha256((base/'tool-sources/run-targeted.py').read_bytes()).hexdigest()
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),source_commit=commit,
           clean_checkout=True,git_blob_files_verified=len(all_files),
           harness_git_blobs_verified=3,test_log_hashes_verified=True,test_binary_hashes_verified=True,
           manifest_sha256=sha256((reports/'source-manifest.json').read_bytes()).hexdigest(),
           expected_certificates_sha256=sha256((base/'expected-certificates.json').read_bytes()).hexdigest(),
           unique_tests_passed=5,source_contexts=45,supplied_public_type_proofs=81,
           contexts_with_empty_type_group=10,internal_type_instances_pending=6,
           application_proofs_pending=987,complete_application_assembly_pending=True,
           exported_files_verified=len(actual),exported_files_match_local=True,
           full_t_gate='deferred to T06-W12')
(reports/'final-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+chr(10))
with tarfile.open(fileobj=sys.stdout.buffer,mode='w|gz') as tar:
    for name in ['status.json','source-manifest.json','launch.json','final-audit.json','supervisor.log']+[s['stage']+'.log' for s in state['stages']]:
        tar.add(reports/name,arcname=name)
    for name in sorted(actual):
        tar.add(reports/'certificates'/name,arcname='certificates/'+name)
