import concurrent.futures,hashlib,json,pathlib,subprocess,time,os
from datetime import datetime,timezone
root=pathlib.Path('/private/tmp/mpk-w09-identity-proofs');repo=pathlib.Path('/private/tmp/mpk-w09-packed-pattern-proofs');source=root/'local/certificates';out=root/'checks';out.mkdir(exist_ok=True)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def now():return datetime.now(timezone.utc).isoformat()
m=json.loads((root/'source-manifest.json').read_bytes())
for group in ('source_hashes','fixture_hashes'):
 for n,h in m[group].items():assert digest(repo/n)==h,n
bins={'go':pathlib.Path('/private/tmp/mpk-w09-scoped-construction/go-checker'),'rust':pathlib.Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
hashes={k:digest(v) for k,v in bins.items()};assert hashes==dict(go='b953943f226faf8e516c6677277c75b73607aa600f27cd0f04b761f097e719a7',rust='b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91')
state=dict(status='running',supervisor_pid=os.getpid(),started_at=now(),stages=[],binaries=hashes,selection_reason='Check the one changed identity proof context and typed/hash mutations with both unchanged backends; reuse prior same-byte checker evidence for the 44 unchanged certificates.',full_t_gate='deferred to T06-W12')
def save():
 p=out/'status.tmp';p.write_text(json.dumps(state,indent=2,sort_keys=True)+'\n');p.replace(out/'status.json')
def run(backend,stem):
 command=[str(bins[backend]),'verify' if backend=='go' else 'check',str(out/(stem+'.mpcert'))];start=time.monotonic()
 with (out/(stem+'-'+backend+'.json')).open('wb') as o,(out/(stem+'-'+backend+'.stderr.txt')).open('wb') as e:r=subprocess.run(command,stdout=o,stderr=e)
 return backend,r.returncode,json.loads((out/(stem+'-'+backend+'.json')).read_bytes()),round(time.monotonic()-start,3),command
save()
try:
 pos=source/'float-make-commutation.hex';data=bytes.fromhex(pos.read_text());wrong=bytes.fromhex((source/'float-make-commutation-wrong.hex').read_text())
 for kind,value in [('positive',data),('hash',data[:-1]+bytes([data[-1]^1])),('wrong',wrong)]:
  stem='float-make-commutation-'+kind;(out/(stem+'.mpcert')).write_bytes(value);state['stage']=stem;save();accepted={}
  with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:results=list(pool.map(lambda b:run(b,stem),bins))
  for b,code,r,elapsed,command in results:
   assert code==(0 if kind=='positive' else 1) and r['verdict']==('accepted' if kind=='positive' else 'rejected')
   if kind=='positive':
    assert r.get('axiom_count',0)==0 and r['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+value).hexdigest()
    accepted[b]=(r['module'],r['declaration_count'],r.get('axiom_count',0),r['hashes'])
    if b=='rust':assert all(v==0 for v in r['axiom_report']['summary'].values())
   else:assert r['error_kind' if b=='go' else 'error_code']==({'hash':'hash_mismatch','wrong':'core_check'}[kind] if b=='go' else {'hash':'KERNEL_HASH_MISMATCH','wrong':'KERNEL_CORE_CHECK'}[kind])
   state['stages'].append(dict(backend=b,kind=kind,command=command,exit_code=code,elapsed_seconds=elapsed,input_file_sha256=hashlib.sha256(value).hexdigest(),binary_sha256=hashes[b],report_sha256=digest(out/(stem+'-'+b+'.json')),stderr_sha256=digest(out/(stem+'-'+b+'.stderr.txt'))))
  if kind=='positive':assert accepted['go']==accepted['rust']
  save();print(stem+' passed both checkers',flush=True)
 prior=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/foundation-proofs'
 receipt=json.loads((prior/'checks/verification.json').read_bytes());assert receipt['status']=='passed'
 reused={}
 for p in source.glob('*.json'):
  meta=json.loads(p.read_bytes())
  if meta['proofs']:continue
  old=prior/'local/certificates'/(p.stem+'.hex');raw=bytes.fromhex(old.read_text());h=hashlib.sha256(raw).hexdigest();assert meta['certificate_sha256']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
  aliases=receipt['source_certificate_aliases'][h];case=aliases[0]
  positive=[s for s in receipt['stages'] if s['case']==case and s['kind']=='positive'];assert {s['backend'] for s in positive}==set(bins)
  for s in positive:
   assert s['exit_code']==0 and s['input_file_sha256']==h
   assert s['report_sha256']==digest(prior/'checks'/(case+'-positive-'+s['backend']+'.json'))
  reused[p.stem]=dict(input_file_sha256=h,original_hex_sha256=digest(old),prior_case=case)
 assert len(reused)==44 and len(state['stages'])==6
 state.update(status='passed',finished_at=now(),stage_count=6,reused_unchanged_certificate_contexts=reused,prior_dual_receipt_sha256=digest(prior/'checks/verification.json'),fresh_positive_stages=2,hash_rejections=2,typed_core_rejections=2);save();(out/'verification.json').write_bytes((out/'status.json').read_bytes())
except BaseException as e:
 state.update(status='failed',error=repr(e),finished_at=now());save();raise
