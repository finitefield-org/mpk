"""Audit the terminal same-byte Go/Rust reports against the final source exports."""
from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
import json,shutil
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
root=Path('/private/tmp/mpk-w09-scoped-construction')
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/scoped-construction-proofs'
checks=root/'checks'
def digest(p):return sha256(p.read_bytes()).hexdigest()
s=json.loads((checks/'status.json').read_bytes())
assert s['status']=='passed' and len(s['stages'])==50
assert (checks/'status.json').read_bytes()==(checks/'verification.json').read_bytes()
assert s['stage_count']==50 and (s['positive_stages'],s['hash_rejections'],s['typed_core_rejections'])==(24,24,2)
bins={'go':root/'go-checker','rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
assert {b:digest(p) for b,p in bins.items()}==s['binaries']
assert s['binaries']['rust']=='b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'
manifest=json.loads((base/'source-manifest.json').read_bytes())
assert (len(manifest['source_hashes']),len(manifest['fixture_hashes']))==(749,173)
for group in ['source_hashes','fixture_hashes']:
 for name,h in manifest[group].items():assert digest(repo/name)==h,name
exports=base/'local/certificates'
positive=sorted(p.stem for p in exports.glob('*.hex') if not p.stem.endswith('-wrong'))
assert len(positive)==12
expected=[]
for name in positive:
 expected.extend((name,k,b) for k in ['positive','hash'] for b in ['go','rust'])
 if (exports/(name+'-wrong.hex')).exists():expected.extend((name,'wrong',b) for b in ['go','rust'])
assert [(r['case'],r['kind'],r['backend']) for r in s['stages']]==expected
counts={'positive':0,'hash':0,'wrong':0};accepted={}
for r in s['stages']:
 name,kind,b=r['case'],r['kind'],r['backend'];stem=name+'-'+kind
 path=checks/(stem+'.mpcert');data=path.read_bytes()
 if kind=='positive':assert data==bytes.fromhex((exports/(name+'.hex')).read_text())
 elif kind=='hash':
  original=bytes.fromhex((exports/(name+'.hex')).read_text());assert data==original[:-1]+bytes([original[-1]^1])
 else:assert data==bytes.fromhex((exports/(name+'-wrong.hex')).read_text())
 assert sha256(data).hexdigest()==r['input_file_sha256']
 assert sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()==r['certificate_sha256']
 assert r['binary_sha256']==s['binaries'][b]
 assert r['command']==[str(bins[b]),'verify' if b=='go' else 'check',str(path)]
 report_path=checks/(stem+'-'+b+'.json');stderr_path=checks/(stem+'-'+b+'.stderr.txt')
 assert digest(report_path)==r['report_sha256'] and digest(stderr_path)==r['stderr_sha256']
 report=json.loads(report_path.read_bytes())
 assert r['exit_code']==(0 if kind=='positive' else 1)
 assert report['verdict']==('accepted' if kind=='positive' else 'rejected')
 if kind=='positive':
  # The checked Go CLI source explicitly omits a uint64 axiom_count when zero.
  assert report.get('axiom_count',0)==0 and report['hashes']['certificate']==r['certificate_sha256']
  if b=='rust':assert all(v==0 for v in report['axiom_report']['summary'].values())
  value=(report['module'],report['declaration_count'],report.get('axiom_count',0),report['hashes'])
  if name in accepted:assert accepted[name]==value,name
  else:accepted[name]=value
 else:
  expected_error={'go':{'hash':'hash_mismatch','wrong':'core_check'},'rust':{'hash':'KERNEL_HASH_MISMATCH','wrong':'KERNEL_CORE_CHECK'}}[b][kind]
  assert report['error_kind' if b=='go' else 'error_code']==expected_error
 counts[kind]+=1
assert counts=={'positive':24,'hash':24,'wrong':2} and len(accepted)==12
assert not (base/'checks').exists()
shutil.copytree(checks,base/'checks')
for path in checks.rglob('*'):
 if path.is_file():assert path.read_bytes()==(base/'checks'/path.relative_to(checks)).read_bytes()
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),source_commit='2c4613aab498c2ec07f8663502a0c30316b63669',source_files_verified=749,fixture_files_verified=173,scoped_candidates=12,source_contexts=45,source_operations_pending=0,generic_operations_pending=25,application_proofs_pending=987,stages_verified=50,positive_stages=24,hash_rejections=24,typed_core_rejections=2,input_bytes_match_final_local_exports=True,input_file_hashes_verified=True,checker_binary_hashes_verified=True,raw_report_hashes_verified=True,stderr_hashes_verified=True,all_positive_hashes_and_counts_agree=True,all_axiom_counts_zero=True,binary_hashes=s['binaries'],go_binary_build_command=['go','build','-o',str(bins['go']),'./cmd/mpk-checker-ref'],go_binary_build_workdir=str(repo/'go-tools/mpk-checker-ref'),go_binary_build_terminal_session=16308,go_binary_build_exit_code=0,unchanged_kernel_and_checker_source=True,full_t_gate='deferred to T06-W12')
(base/'dual-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
verification=json.loads((base/'verification.json').read_bytes());verification['dual_checker_verification']='passed independent terminal audit; 50 fresh stages';verification['dual_checker_audit']='dual-audit.json';verification['same_byte_positive_cases_verified']=12
(base/'verification.json').write_text(json.dumps(verification,indent=2,sort_keys=True)+'\n')
print(json.dumps(audit,sort_keys=True))
