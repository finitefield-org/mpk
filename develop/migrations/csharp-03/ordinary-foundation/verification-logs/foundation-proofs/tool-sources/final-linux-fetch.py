from datetime import datetime,timezone
from hashlib import sha256
import io,json,subprocess,sys,tarfile,time
from pathlib import Path
commit=sys.argv[1]
assert len(commit)==40 and all(c in '0123456789abcdef' for c in commit)
out=Path('/private/tmp/mpk-w09-foundation-proofs/server-final-'+commit[:8]);assert not out.exists();out.mkdir()
remote=r'''from datetime import datetime,timezone
from hashlib import sha256
from pathlib import Path
import json,subprocess,sys,tarfile
commit='SOURCE_COMMIT'
repo=Path('/root/mpk-w09-foundation-proofs-'+commit[:8]);reports=repo.with_name(repo.name+'-linux')
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/foundation-proofs'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args],cwd=repo)
assert git('rev-parse','HEAD').decode().strip()==commit
assert git('status','--porcelain').decode().strip()==''
state=json.loads((reports/'status.json').read_bytes())
assert state['status']=='passed_targeted_tests_lint_format' and state['unique_tests_passed']==3
assert [x['stage'] for x in state['stages']]==['foundation-proofs','original-allocation-proofs','original-storage-type-proofs','clippy','format','support-format']
assert (state['supplied_type_proofs'],state['supplied_operation_proofs'],state['supplied_binding_sequents'],state['remaining_binding_sequents'],state['original_application_proofs_pending'],state['generic_operations_pending'])==(87,442,529,458,987,25)
assert state['exported_certificates_match_local'] is True
for stage in state['stages']:
 assert stage['exit_code']==0
 assert sha256((reports/(stage['stage']+'.log')).read_bytes()).hexdigest()==stage['log_sha256']
assert len(state['test_binaries'])==1
for path,digest in state['test_binaries'].items():assert sha256(Path(path).read_bytes()).hexdigest()==digest
manifest=json.loads((reports/'source-manifest.json').read_bytes())
assert (reports/'source-manifest.json').read_bytes()==(base/'source-manifest.json').read_bytes()
assert (len(manifest['source_hashes']),len(manifest['fixture_hashes']))==(751,173)
verified={}
for group in ('source_hashes','fixture_hashes'):
 for name,digest in manifest[group].items():
  data=(repo/name).read_bytes();assert sha256(data).hexdigest()==digest,name
  assert git('show',commit+':'+name)==data,name
  assert name not in verified;verified[name]=digest
assert len(verified)==924
for name in ('source-manifest.json','expected-certificates.json','tool-sources/run-targeted.py'):
 path=base/name;assert git('show',commit+':'+str(path.relative_to(repo)))==path.read_bytes()
def catalog(folder):return {p.name:sha256(p.read_bytes()).hexdigest() for p in folder.iterdir() if p.is_file()}
actual=catalog(reports/'certificates')
assert len(actual)==91 and actual==json.loads((base/'expected-certificates.json').read_bytes())
assert {'certificates/'+name:digest for name,digest in actual.items()}==state['exported_certificate_hashes']
legacy={}
for folder,golden in [('type-certificates','construction-storage-types/attempt-2/certificates'),('operation-certificates','concrete-allocation-proofs/attempt-5/certificates')]:
 exports=catalog(reports/folder)
 assert len(exports)==92 and exports==state[folder.replace('-','_')+'_hashes']
 assert exports==catalog(repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs'/golden)
 legacy[folder]=exports
metadata=[json.loads(p.read_bytes()) for p in sorted((reports/'certificates').glob('*.json'))]
assert len(metadata)==45
assert sum(len(m['types']['proofs']) for m in metadata)==87
assert sum(len(m['operations']['proofs']) for m in metadata)==442
assert sum(len(m['supplied_binding_sequent_ids']) for m in metadata)==529
assert sum(len(m['remaining_binding_sequent_ids']) for m in metadata)==458
assert sum(len(m['pending_proof_ids']) for m in metadata)==987
assert sum(len(m['operations']['pending_operations']) for m in metadata)==25
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
for m in metadata:
 supplied={p['sequent']['id'] for p in m['types']['proofs']+m['operations']['proofs']}
 assert len(supplied)==len(m['types']['proofs'])+len(m['operations']['proofs'])
 assert m['supplied_binding_sequent_ids']==[p for p in m['pending_proof_ids'] if p in supplied]
 assert m['remaining_binding_sequent_ids']==[p for p in m['pending_proof_ids'] if p not in supplied]
 assert all(d['private_storage_only'] and d['ownership_pending'] for d in m['types'].get('construction_storage_domains',[]))
launch=json.loads((reports/'launch.json').read_bytes())
assert launch['commit']==commit and launch['clean_checkout'] and launch['public_git_source'] and launch['git_blob_files_verified']==924
assert launch['manifest_sha256']==sha256((base/'source-manifest.json').read_bytes()).hexdigest()
assert launch['runner_sha256']==sha256((base/'tool-sources/run-targeted.py').read_bytes()).hexdigest()
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),source_commit=commit,clean_checkout=True,git_blob_files_verified=924,harness_git_blobs_verified=3,test_log_hashes_verified=True,test_binary_hashes_verified=True,source_contexts=45,unique_tests_passed=3,supplied_type_proofs=87,supplied_operation_proofs=442,supplied_binding_sequents=529,remaining_binding_sequents=458,original_application_proofs_pending=987,generic_operations_pending=25,exported_files_verified=91,exported_files_match_local=True,legacy_type_exported_files_verified=92,legacy_operation_exported_files_verified=92,legacy_certificate_bytes_unchanged=True,complete_application_assembly_pending=True,full_t_gate='deferred to T06-W12')
(reports/'final-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
with tarfile.open(fileobj=sys.stdout.buffer,mode='w|gz') as tar:
 for name in ['status.json','source-manifest.json','launch.json','final-audit.json','supervisor.log']+[s['stage']+'.log' for s in state['stages']]:tar.add(reports/name,arcname=name)
 for folder,files in [('certificates',actual),*legacy.items()]:
  for name in sorted(files):tar.add(reports/folder/name,arcname=folder+'/'+name)
'''.replace('SOURCE_COMMIT',commit)
(out/'remote-audit.py').write_text(remote)
command=['ssh','-i','/Users/kazuyoshitoshiya/ffvps.pem','-o','BatchMode=yes','-o','ConnectTimeout=20','root@162.43.92.154','python3','-']
start=datetime.now(timezone.utc).isoformat();tick=time.monotonic()
p=subprocess.run(command,input=remote.encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
(out/'receipt.tar.gz').write_bytes(p.stdout);(out/'stderr.log.txt').write_bytes(p.stderr)
r=dict(command=command,exit_code=p.returncode,started_at=start,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-tick,3),source_script_sha256=sha256(remote.encode()).hexdigest(),archive_sha256=sha256(p.stdout).hexdigest(),stderr_sha256=sha256(p.stderr).hexdigest())
(out/'execution.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n')
if p.returncode:print(p.stderr.decode());raise SystemExit(p.returncode)
with tarfile.open(fileobj=io.BytesIO(p.stdout),mode='r:gz') as tar:tar.extractall(out,filter='data')
audit=json.loads((out/'final-audit.json').read_bytes());assert audit['status']=='passed' and audit['source_commit']==commit
print(json.dumps(audit,sort_keys=True))
