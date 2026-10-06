"""Independently hash the terminal local tests, all inputs and preserved exports."""
from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
import json
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/scoped-construction-proofs'
def digest(p):return sha256(p.read_bytes()).hexdigest()
def catalog(p):return {f.name:digest(f) for f in p.iterdir() if f.is_file()}
s=json.loads((base/'local/status.json').read_bytes())
assert s['status']=='passed_targeted_tests_lint_format' and s['unique_tests_passed']==3
assert [r['stage'] for r in s['stages']]==['scoped-construction-proofs','original-allocation-proofs','original-operation-pins','clippy','format','support-format']
for r in s['stages']:
 assert r['exit_code']==0 and digest(base/'local'/(r['stage']+'.log.txt'))==r['log_sha256']
for p,h in s['test_binaries'].items():assert digest(Path(p))==h
assert len(s['test_binaries'])==1
m=json.loads((base/'source-manifest.json').read_bytes())
assert (base/'source-manifest.json').read_bytes()==(base/'local/source-manifest.json').read_bytes()
assert (len(m['source_hashes']),len(m['fixture_hashes']))==(749,173)
for group in ('source_hashes','fixture_hashes'):
 for name,h in m[group].items():assert digest(repo/name)==h,name
assert m['fixture_hashes']==json.loads((base.parent/'construction-storage-types/source-manifest.json').read_bytes())['fixture_hashes']
c=catalog(base/'local/certificates')
assert len(c)==70 and c==json.loads((base/'expected-certificates.json').read_bytes())
assert {'certificates/'+n:h for n,h in c.items()}==s['exported_certificate_hashes']
old=catalog(base/'local/legacy-certificates')
assert len(old)==92 and old==s['legacy_certificate_hashes']==catalog(base.parent/'concrete-allocation-proofs/attempt-5/certificates')
roots=[json.loads(p.read_bytes()) for p in (base/'local/certificates').glob('*-program.json')]
assert len(roots)==45 and sum(bool(m['candidates']) for m in roots)==6
assert sum(len(m['candidates']) for m in roots)==12
assert sum(len(m['pending_source_operation_ids']) for m in roots)==0
assert sum(len(m['generic_pending_operations']) for m in roots)==25
assert sum(len(m['pending_proof_ids']) for m in roots)==987
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in roots)
kinds=set()
for root in roots:
 for i,candidate in enumerate(root['candidates']):
  stem=next(p.stem[:-8] for p in (base/'local/certificates').glob('*-program.json') if json.loads(p.read_bytes())==root)+'-'+str(i)
  exported=json.loads((base/'local/certificates'/(stem+'.json')).read_bytes())
  assert exported==candidate
  program=candidate['program']
  assert program['proof_check_pending'] and program['application_scope_pending']
  assert candidate['ownership']['receiver_id']==candidate['source']['subjects'][0]['id']
  assert candidate['ownership']['function_id']==candidate['source']['function_id']
  assert candidate['ownership']['node_id']==candidate['source']['node_id']
  domains=program['construction_storage_domains']
  assert domains and all(d['private_storage_only'] and d['ownership_pending'] for d in domains)
  generic={o['component']['operation_id'] for o in root['generic_pending_operations']}
  pending={o['component']['operation_id'] for o in program['pending_operations']}
  assert len(generic-pending)==1 and not pending-generic
  kinds.update(op.rsplit('.',1)[-1] for op in generic-pending)
  certificate=bytes.fromhex((base/'local/certificates'/(stem+'.hex')).read_text())
  assert sha256(b'MPK-MODULE-CERT-0.1\0'+certificate).hexdigest()==program['certificate_sha256']
assert kinds=={'fill','freeze'}
assert 'scoped construction: 45 contexts, 12 candidates, operation kinds {"fill", "freeze"}' in (base/'local/scoped-construction-proofs.log.txt').read_text()
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),source_commit='2c4613aab498c2ec07f8663502a0c30316b63669',source_files_verified=749,fixture_files_verified=173,test_log_hashes_verified=True,test_binary_hashes_verified=True,unique_tests_passed=3,source_contexts=45,contexts_with_scoped_candidates=6,scoped_candidates=12,source_operations_pending=0,observed_operation_kinds=sorted(kinds),generic_operations_pending=25,application_proofs_pending=987,exported_files_verified=70,legacy_exported_files_verified=92,legacy_certificate_bytes_unchanged=True,manifest_sha256=digest(base/'source-manifest.json'),expected_certificates_sha256=digest(base/'expected-certificates.json'),full_t_gate='deferred to T06-W12')
(base/'local/final-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
print(json.dumps(audit,sort_keys=True))
