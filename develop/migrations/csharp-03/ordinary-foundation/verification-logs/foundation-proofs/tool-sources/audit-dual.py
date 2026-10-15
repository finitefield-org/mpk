from datetime import datetime,timezone
from hashlib import sha256
import json,shutil
from pathlib import Path
import sys
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs');root=Path('/private/tmp/mpk-w09-foundation-proofs');out=root/'checks'
state=json.loads((out/'status.json').read_bytes());assert state['status']=='passed' and len(state['stages'])==182
assert (state['positive_stages'],state['hash_rejections'],state['typed_core_rejections'])==(90,90,2)
assert state['distinct_positive_certificates']==45 and len(state['source_certificate_aliases'])==45
manifest=json.loads((root/'source-manifest.json').read_bytes())
for group in ('source_hashes','fixture_hashes'):
 for p,h in manifest[group].items():assert sha256((repo/p).read_bytes()).hexdigest()==h,p
bins={'go':Path('/private/tmp/mpk-w09-scoped-construction/go-checker'),'rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
for backend,path in bins.items():assert sha256(path.read_bytes()).hexdigest()==state['binaries'][backend]
assert state['binaries']=={'go':'b953943f226faf8e516c6677277c75b73607aa600f27cd0f04b761f097e719a7','rust':'b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'}
expected=[]
for p in sorted((root/'local/certificates').glob('*.hex')):
 if p.stem.endswith('-wrong'):continue
 data=bytes.fromhex(p.read_text());assert state['source_certificate_aliases'][sha256(data).hexdigest()]==[p.stem]
 jobs=[('positive',data),('hash',data[:-1]+bytes([data[-1]^1]))]
 wrong=p.with_name(p.stem+'-wrong.hex')
 if wrong.exists():jobs.append(('wrong',bytes.fromhex(wrong.read_text())))
 for kind,data in jobs:
  stem=p.stem+'-'+kind
  assert (out/(stem+'.mpcert')).read_bytes()==data
  accepted={}
  for backend in bins:
   row=state['stages'][len(expected)];expected.append((p.stem,kind,backend))
   assert (row['case'],row['kind'],row['backend'])==expected[-1]
   assert row['command']==[str(bins[backend]),'verify' if backend=='go' else 'check',str(out/(stem+'.mpcert'))]
   assert row['exit_code']==(0 if kind=='positive' else 1)
   assert row['binary_sha256']==state['binaries'][backend]
   assert row['input_file_sha256']==sha256(data).hexdigest()
   assert row['certificate_sha256']==sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
   raw=out/(stem+'-'+backend+'.json');stderr=out/(stem+'-'+backend+'.stderr.txt')
   assert sha256(raw.read_bytes()).hexdigest()==row['report_sha256']
   assert sha256(stderr.read_bytes()).hexdigest()==row['stderr_sha256']
   r=json.loads(raw.read_bytes());assert r['verdict']==('accepted' if kind=='positive' else 'rejected')
   if kind=='positive':
    assert r.get('axiom_count',0)==0 and r['hashes']['certificate']==row['certificate_sha256']
    if backend=='rust':assert all(v==0 for v in r['axiom_report']['summary'].values())
    accepted[backend]=(r['module'],r['declaration_count'],r.get('axiom_count',0),r['hashes'])
   else:
    assert r['error_kind' if backend=='go' else 'error_code']==({'hash':'hash_mismatch','wrong':'core_check'}[kind] if backend=='go' else {'hash':'KERNEL_HASH_MISMATCH','wrong':'KERNEL_CORE_CHECK'}[kind])
  if kind=='positive':assert accepted['go']==accepted['rust']
assert len(expected)==182
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/foundation-proofs'
assert not (base/'checks').exists();shutil.copytree(out,base/'checks')
audit=dict(status='passed',recorded_at=datetime.now(timezone.utc).isoformat(),terminal_exec_session=int(sys.argv[1]),source_commit='1e5338e94f792a84b8ebe22ed2452bef896791c8',fresh_stages_verified=182,source_contexts=45,distinct_positive_certificates=45,positive_stages=90,hash_rejections=90,typed_core_rejections=2,input_bytes_match_final_source_exports=True,all_positive_reports_match=True,all_positive_axiom_counts_zero=True,binary_hashes=state['binaries'],raw_report_and_stderr_hashes_verified=True,source_files_verified=751,fixture_files_verified=173,supplied_type_proofs=87,supplied_operation_proofs=442,supplied_binding_sequents=529,remaining_binding_sequents=458,original_application_proofs_pending=987,generic_operations_pending=25,complete_application_assembly_pending=True,full_t_gate='deferred to T06-W12')
(base/'dual-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
print(json.dumps(audit,sort_keys=True))
