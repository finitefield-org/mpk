from pathlib import Path
from hashlib import sha256
import concurrent.futures,json,subprocess,time
root=Path('/private/tmp/mpk-w09-scoped-construction')
source=root/'full-1/certificates'; out=root/'checks';out.mkdir(exist_ok=True)
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
manifest=json.loads((root/'source-manifest.json').read_bytes())
for group in ('source_hashes','fixture_hashes'):
 for n,h in manifest[group].items(): assert sha256((repo/n).read_bytes()).hexdigest()==h,n
prior=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/construction-storage-types/source-manifest.json'
assert manifest['fixture_hashes']==json.loads(prior.read_bytes())['fixture_hashes']
old=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-allocation-proofs/attempt-5/certificates'
def catalog(p): return {x.name:sha256(x.read_bytes()).hexdigest() for x in p.iterdir() if x.is_file()}
assert catalog(root/'legacy-1/certificates')==catalog(old)
bins={'go':root/'go-checker','rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
binary_hashes={k:sha256(p.read_bytes()).hexdigest() for k,p in bins.items()}
assert binary_hashes['rust']=='b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'
positive=sorted(p for p in source.glob('*.hex') if not p.stem.endswith('-wrong'))
assert len(positive)==12
roots=[json.loads(p.read_bytes()) for p in source.glob('*-program.json')]
assert len(roots)==45 and sum(len(m['candidates']) for m in roots)==12
assert sum(len(m['pending_source_operation_ids']) for m in roots)==0
assert sum(len(m['generic_pending_operations']) for m in roots)==25
assert sum(len(m['pending_proof_ids']) for m in roots)==987
state=dict(status='running',stages=[],binaries=binary_hashes,scoped_candidates=12,source_contexts=45,source_operations_pending=0,generic_operations_pending=25,application_proofs_pending=987,full_t_gate='deferred to T06-W12')
def persist(): (out/'status.json').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n')
def run(b,stem):
 p=out/(stem+'.mpcert'); command=[str(bins[b]),'verify' if b=='go' else 'check',str(p)]
 start=time.monotonic()
 with (out/(stem+'-'+b+'.json')).open('wb') as o,(out/(stem+'-'+b+'.stderr.txt')).open('wb') as e: proc=subprocess.run(command,stdout=o,stderr=e)
 report=json.loads((out/(stem+'-'+b+'.json')).read_bytes())
 return b,proc.returncode,report,round(time.monotonic()-start,3),command
persist()
for p in positive:
 data=bytes.fromhex(p.read_text()); jobs=[('positive',data),('hash',data[:-1]+bytes([data[-1]^1]))]
 wrong=source/(p.stem+'-wrong.hex')
 if wrong.exists(): jobs.append(('wrong',bytes.fromhex(wrong.read_text())))
 for kind,data in jobs:
  stem=p.stem+'-'+kind; (out/(stem+'.mpcert')).write_bytes(data)
  state['stage']=stem;persist(); accepted={}
  with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool: results=list(pool.map(lambda b:run(b,stem),bins))
  h=sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
  for b,code,r,elapsed,command in results:
   assert code==(0 if kind=='positive' else 1),(stem,b,code)
   assert r['verdict']==('accepted' if kind=='positive' else 'rejected')
   if kind=='positive':
    assert r.get('axiom_count',0)==0 and r['hashes']['certificate']==h
    if b=='rust': assert all(v==0 for v in r['axiom_report']['summary'].values())
    accepted[b]=(r['module'],r['declaration_count'],r.get('axiom_count',0),r['hashes'])
   else:
    assert r['error_kind' if b=='go' else 'error_code']==({'hash':'hash_mismatch','wrong':'core_check'}[kind] if b=='go' else {'hash':'KERNEL_HASH_MISMATCH','wrong':'KERNEL_CORE_CHECK'}[kind])
   state['stages'].append(dict(backend=b,case=p.stem,kind=kind,command=command,exit_code=code,elapsed_seconds=elapsed,input_file_sha256=sha256(data).hexdigest(),certificate_sha256=h,binary_sha256=binary_hashes[b],report_sha256=sha256((out/(stem+'-'+b+'.json')).read_bytes()).hexdigest(),stderr_sha256=sha256((out/(stem+'-'+b+'.stderr.txt')).read_bytes()).hexdigest()))
  if kind=='positive': assert accepted['go']==accepted['rust'],stem
  persist();print(stem,'passed both checkers',flush=True)
assert len(state['stages'])==50
state.update(status='passed',stage_count=50,positive_stages=24,hash_rejections=24,typed_core_rejections=2,legacy_exported_files_unchanged=92)
persist();(out/'verification.json').write_bytes((out/'status.json').read_bytes())
