import hashlib,json,os,pathlib,subprocess,time
from datetime import datetime,timezone
root=pathlib.Path('/private/tmp/mpk-w09-context-application-reports');source=root/'local/certificates';out=root/'checks-2';out.mkdir()
bins={'rust':pathlib.Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk'),'go':pathlib.Path('/private/tmp/mpk-w09-scoped-construction/go-checker')}
expected={'rust':'b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91','go':'b953943f226faf8e516c6677277c75b73607aa600f27cd0f04b761f097e719a7'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert {k:sha(p) for k,p in bins.items()}==expected
cases=['true','false','wrong','nested-true','nested-false','nested-wrong'];pins={str(p):sha(p) for p in [*bins.values(),*(source/(c+'.mpcert') for c in cases),pathlib.Path(__file__)]};(out/'inputs.json').write_text(json.dumps(pins,indent=2)+'\n')
prior=root/'checks';prior_state=json.loads((prior/'status.json').read_text());prior_rows={(x['case'],x['backend']):x for x in prior_state['stages']};assert prior_state['binaries']==expected
state=dict(status='running',supervisor_pid=os.getpid(),stages=[],binaries=expected,application_scope_pending=True,selection_reason='Check same bytes for true/false original context cases and nested application regressions with both unchanged backends. Wrong actual equality proofs must fail at core checking. Shared DAG bytes are compared to prior evidence, not rechecked solely for this syntax fix.',full_t_gate='deferred to T06-W12')
def save():
 p=out/'status.tmp';p.write_text(json.dumps(state,indent=2)+'\n');p.replace(out/'status.json')
try:
 for case in cases:
  accepted={};value=(source/(case+'.mpcert')).read_bytes();h=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+value).hexdigest()
  for backend,binary in bins.items():
   command=[str(binary),'check' if backend=='rust' else 'verify',str(source/(case+'.mpcert'))];start=time.monotonic();state['stage']=case+'-'+backend
   reuse=prior_rows.get((case,backend))
   if reuse:
    assert reuse['input_file_sha256']==hashlib.sha256(value).hexdigest()
    assert reuse['report_sha256']==sha(prior/(case+'-'+backend+'.json')) and reuse['stderr_sha256']==sha(prior/(case+'-'+backend+'.stderr.txt'))
    for suffix in ('.json','.stderr.txt'):(out/(case+'-'+backend+suffix)).write_bytes((prior/(case+'-'+backend+suffix)).read_bytes())
    code=reuse['exit_code']
   else:
    with (out/(case+'-'+backend+'.json')).open('wb') as o,(out/(case+'-'+backend+'.stderr.txt')).open('wb') as e:
     p=subprocess.Popen(command,stdout=o,stderr=e);state['process_pid']=p.pid;save();code=p.wait()
   report=json.loads((out/(case+'-'+backend+'.json')).read_bytes());wrong='wrong' in case;assert code==(1 if wrong else 0);assert report['verdict']==('rejected' if wrong else 'accepted')
   if not wrong or backend=='rust':assert report['hashes']['certificate']==h
   if wrong:assert report['error_code' if backend=='rust' else 'error_kind']==('KERNEL_CORE_CHECK' if backend=='rust' else 'core_check')
   else:
    assert report.get('axiom_count',0)==0
    if backend=='rust':assert all(x==0 for x in report['axiom_report']['summary'].values())
    accepted[backend]=(report['module'],report['declaration_count'],report['hashes'])
   state['stages'].append(dict(case=case,backend=backend,reused_from=str(prior) if reuse else None,command=command,exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),input_file_sha256=hashlib.sha256(value).hexdigest(),report_sha256=sha(out/(case+'-'+backend+'.json')),stderr_sha256=sha(out/(case+'-'+backend+'.stderr.txt'))));save();print(case+'-'+backend+' passed',flush=True)
  if not wrong:assert accepted['rust']==accepted['go']
 assert all(sha(pathlib.Path(p))==h for p,h in pins.items());state.update(status='passed',stage_count=12);save()
except BaseException as e:state.update(status='failed',error=repr(e));save();raise
