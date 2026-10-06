"""Original identity sequents, unchanged foundation contexts, targeted lint/format; defer full T06 gate to W12."""
import argparse, hashlib, json, os, pathlib, re, subprocess, time
from datetime import datetime, timezone

def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def now(): return datetime.now(timezone.utc).isoformat()
def main():
 a=argparse.ArgumentParser(description=__doc__)
 for n in ('repo','reports','manifest','target'): a.add_argument('--'+n,type=pathlib.Path,required=True)
 a.add_argument('--cargo',default='cargo'); a.add_argument('--expected-certificates',type=pathlib.Path)
 x=a.parse_args(); x.reports.mkdir(parents=True,exist_ok=True); manifest=json.loads(x.manifest.read_bytes())
 state=dict(status='preparing',supervisor_pid=os.getpid(),started_at=now(),stages=[],selection_reason=manifest['selection_reason'],full_t_gate='deferred to T06-W12',application_scope_pending=True)
 def save():
  p=x.reports/'status.tmp';p.write_text(json.dumps(state,indent=2,sort_keys=True)+'\n');p.replace(x.reports/'status.json')
 def verify():
  for group in ('source_hashes','fixture_hashes'):
   for n,h in manifest[group].items(): assert digest(x.repo/n)==h,n
 save()
 try:
  verify(); (x.reports/'source-manifest.json').write_bytes(x.manifest.read_bytes())
  env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(x.target),CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',CARGO_PROFILE_TEST_OPT_LEVEL='2',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',MPK_W09_IDENTITY_PROOFS_OUT=str(x.reports/'certificates'))
  env.pop('MPK_W09_IDENTITY_SOURCE_FILTER',None)
  jobs=[('identity-proofs',['test','-p','mpk-vc','--test','csharp_practical_vc','csharp_03_t06_w09_identity_projection_proofs_original_source','--','--nocapture','--test-threads=1']),('clippy',['clippy','-p','mpk-vc','--lib','--tests','--','-D','warnings']),('format',['fmt','-p','mpk-vc','--','--check'])]
  binaries={}
  for name,args in jobs:
   verify();state.update(status='running',stage=name);start=time.monotonic();log=x.reports/(name+'.log.txt')
   with log.open('wb') as o:
    p=subprocess.Popen([x.cargo,*args],cwd=x.repo,env=env,stdin=subprocess.DEVNULL,stdout=o,stderr=subprocess.STDOUT);state['process_pid']=p.pid;save();code=p.wait()
   state['stages'].append(dict(stage=name,command=[x.cargo,*args],exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=digest(log)));save();print(json.dumps(state['stages'][-1]),flush=True);assert code==0,(name,code)
   if name=='identity-proofs':
    text=log.read_text();assert 'test result: ok. 1 passed; 0 failed;' in text
    for n in re.findall(r'Running [^\n]*\(([^\n]+)\)',text):binaries[n]=digest(pathlib.Path(n))
   verify()
  command=['rustfmt','--edition','2021','--check','crates/mpk-vc/tests/support/csharp_practical_ordinary_binding_relation_tests.rs','crates/mpk-vc/tests/support/csharp_practical_ordinary_foundation_proof_tests.rs','crates/mpk-vc/tests/support/csharp_practical_ordinary_identity_proof_tests.rs']
  log=x.reports/'support-format.log.txt'
  with log.open('wb') as o:p=subprocess.run(command,cwd=x.repo,env=env,stdout=o,stderr=subprocess.STDOUT)
  state['stages'].append(dict(stage='support-format',command=command,exit_code=p.returncode,log_sha256=digest(log)));assert p.returncode==0
  exports=x.reports/'certificates';metadata=[json.loads(p.read_bytes()) for p in exports.glob('*.json')];assert len(metadata)==45
  counts=dict(source_contexts=45,identity_proofs=sum(len(m['proofs']) for m in metadata),supplied_binding_sequents=sum(len(m['supplied_binding_sequent_ids']) for m in metadata),remaining_binding_sequents=sum(len(m['remaining_binding_sequent_ids']) for m in metadata),original_application_proofs_pending=sum(len(m['pending_proof_ids']) for m in metadata),generic_operations_pending=sum(len(m['foundation']['operations']['pending_operations']) for m in metadata))
  assert counts==dict(source_contexts=45,identity_proofs=2,supplied_binding_sequents=531,remaining_binding_sequents=456,original_application_proofs_pending=987,generic_operations_pending=25)
  assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
  catalog={p.name:digest(p) for p in exports.iterdir() if p.is_file()};assert len(catalog)==47
  if x.expected_certificates:assert catalog==json.loads(x.expected_certificates.read_bytes())
  (x.reports/'expected-certificates.json').write_text(json.dumps(catalog,indent=2,sort_keys=True)+'\n')
  verify();state.update(counts,status='passed_targeted_tests_lint_format',finished_at=now(),test_binaries=binaries,exported_hashes=catalog,verified_source_hashes=len(manifest['source_hashes']),verified_fixture_hashes=len(manifest['fixture_hashes']),unchanged_foundation_certificates=44,original_foundation_programs_unchanged=45);save()
 except BaseException as error:
  state.update(status='failed',error=repr(error),finished_at=now());save();raise
if __name__=='__main__':main()
