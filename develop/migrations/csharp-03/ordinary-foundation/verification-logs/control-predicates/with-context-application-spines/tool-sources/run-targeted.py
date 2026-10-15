"""Context application spine regression and direct consumers; defer the T06 gate to W12."""
import argparse,hashlib,json,os,pathlib,re,subprocess,time
from datetime import datetime,timezone
p=argparse.ArgumentParser(description=__doc__)
for n in ('repo','reports','manifest','target'):p.add_argument('--'+n,type=pathlib.Path,required=True)
p.add_argument('--cargo',default='cargo');a=p.parse_args();a.reports.mkdir(parents=True,exist_ok=True)
m=json.loads(a.manifest.read_bytes())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def verify():
 for group in ('source_hashes','fixture_hashes'):
  for n,h in m[group].items():assert sha(a.repo/n)==h,n
def now():return datetime.now(timezone.utc).isoformat()
state=dict(status='running',supervisor_pid=os.getpid(),started_at=now(),selection_reason=m['selection_reason'],stages=[],application_scope_pending=True,full_t_gate='deferred to T06-W12')
def save():
 t=a.reports/'status.tmp';t.write_text(json.dumps(state,indent=2,sort_keys=True)+'\n');t.replace(a.reports/'status.json')
try:
 verify();(a.reports/'source-manifest.json').write_bytes(a.manifest.read_bytes())
 env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(a.target),CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',CARGO_PROFILE_TEST_OPT_LEVEL='2',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',MPK_W09_CONTEXT_BOOLEAN_UNIT_OUTPUT=str(a.reports/'certificates'))
 jobs=[('context-units',['test','-p','mpk-vc','--lib','context_boolean_normalization_','--','--nocapture'],3),('candidate-units',['test','-p','mpk-vc','--lib','pattern_refinement_candidate_','--','--nocapture'],3),('substitution-unit',['test','-p','mpk-vc','--lib','ownership_normalization_preserves_free_selector_under_binder','--','--nocapture'],1),('clippy',['clippy','-p','mpk-vc','--lib','--tests','--','-D','warnings'],0),('format',['fmt','-p','mpk-vc','--','--check'],0)]
 binaries={}
 for name,args,count in jobs:
  verify();state.update(stage=name);log=a.reports/(name+'.log.txt');start=time.monotonic()
  with log.open('wb') as o:
   process=subprocess.Popen([a.cargo,*args],cwd=a.repo,env=env,stdout=o,stderr=subprocess.STDOUT);state['process_pid']=process.pid;save();code=process.wait()
  state['stages'].append(dict(stage=name,command=[a.cargo,*args],exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha(log)));save();assert code==0,(name,code)
  if count:
   text=log.read_text();assert f'test result: ok. {count} passed; 0 failed;' in text
   for binary in re.findall(r'Running [^\n]*\(([^\n]+)\)',text):binaries[binary]=sha(pathlib.Path(binary))
  verify();print(name+' passed',flush=True)
 exports={p.name:sha(p) for p in (a.reports/'certificates').iterdir() if p.is_file()};assert len(exports)==7
 state.update(status='passed_targeted_tests_lint_format',finished_at=now(),tests=7,test_binaries=binaries,exported_hashes=exports,verified_source_hashes=len(m['source_hashes']),verified_fixture_hashes=len(m['fixture_hashes']));save()
except BaseException as e:state.update(status='failed',error=repr(e),finished_at=now());save();raise
