from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
from concurrent.futures import ThreadPoolExecutor
import json,subprocess,time,threading
out=Path('/tmp/mpk-w09-context-refinement/shared-dag-checks');out.mkdir(exist_ok=True)
data=Path('/tmp/mpk-w09-context-refinement/context-certificates/shared-dag.mpcert').read_bytes()
expected_hash=sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
binaries={'go':Path('/tmp/mpk-w09-default-closed-proofs/go-checker'),'rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
expected={'go':'d631f01eb0beaab7b83bebd53e9f66b02f3a6a8c04e173b3af0acb48b9b305a2','rust':'b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'}
for key,binary in binaries.items():assert sha256(binary.read_bytes()).hexdigest()==expected[key]
lock=threading.Lock();state={'status':'running','selection_reason':'Verify the actual canonical shared-DAG certificate after saturating the expanded sharing cost. Both unchanged checkers must accept the same byte sequence with zero axioms and matching export/certificate/axiom hashes. Compiler profile checks alone are not acceptance. Original application proofs remain pending.','stages':{},'launches':{},'full_t_gate':'deferred to T06-W12'}
def persist():
 (out/'status.tmp').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n');(out/'status.tmp').replace(out/'status.json')
def run(backend):
 binary=binaries[backend];cert=out/(backend+'.mpcert');cert.write_bytes(data)
 command=[str(binary)]+(['check'] if backend=='rust' else [])+[str(cert)];started=datetime.now(timezone.utc).isoformat();start=time.monotonic()
 with (out/(backend+'.json')).open('wb') as report,(out/(backend+'.stderr')).open('wb') as stderr:
  process=subprocess.Popen(command,stdout=report,stderr=stderr)
  with lock:state['launches'][backend]={'process_pid':process.pid,'command':command,'started_at':started,'input_file_sha256':sha256(data).hexdigest(),'binary_sha256':expected[backend]};persist()
  code=process.wait()
 row={'backend':backend,'exit_code':code,'started_at':started,'finished_at':datetime.now(timezone.utc).isoformat(),'elapsed_seconds':round(time.monotonic()-start,3),'input_file_sha256':sha256(data).hexdigest(),'certificate_sha256':expected_hash,'report_sha256':sha256((out/(backend+'.json')).read_bytes()).hexdigest(),'stderr_sha256':sha256((out/(backend+'.stderr')).read_bytes()).hexdigest()}
 with lock:state['stages'][backend]=row;persist()
 print(backend,code,flush=True);return backend
with ThreadPoolExecutor(max_workers=2) as pool:list(pool.map(run,binaries))
try:
 hashes={}
 for backend in binaries:
  row=state['stages'][backend];assert row['exit_code']==0
  r=json.loads((out/(backend+'.json')).read_bytes());assert r['verdict']=='accepted'
  if backend=='go':
   core=r['report'];assert core['AxiomCount']==0;hashes[backend]={k:bytes(core[v]).hex() for k,v in [('export','ExportHash'),('axiom_report','AxiomReportHash'),('certificate','CertificateHash')]}
  else:assert r['axiom_count']==0;hashes[backend]=r['hashes']
  assert hashes[backend]['certificate']==expected_hash
 assert hashes['go']==hashes['rust'];state['status']='passed';state['matching_hashes']=hashes['go']
except BaseException as error:state['status']='failed';state['error']=repr(error);persist();raise
persist()
