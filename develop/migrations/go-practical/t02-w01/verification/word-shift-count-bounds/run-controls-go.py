"""Inspect completed full original control streams once; never use prefixes."""
from pathlib import Path
import datetime,hashlib,json,re,runpy,subprocess,time
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());out=root/'go-controls';out.mkdir(exist_ok=False)
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
expected=runpy.run_path(str(root/'planned-controls.py'))['names']();previous_path=root.parent/'core-interfaces/paired/result.json';previous=json.loads(previous_path.read_text());binary=previous['go_binary'];assert seal(binary['path'])==binary
for s in previous['go_production_sources']:assert seal(s['path'])==s
record={'schema':'mpk.go_practical.full_count_shift_control_check.v1','status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'go_binary':binary,'unchanged_go_build_evidence':seal(previous_path),'unchanged_go_production_sources':previous['go_production_sources'],'expected_names':sorted(expected),'outer_observation_wait_seconds':300,'cases':[],'scope':'Each complete original Rust-emitted canonical control stream is checked once by the unchanged Go checker. Process whichever completed report is available, retaining all raw bytes and exact source snapshots. No prefix acceptance/resource admission/format failure is counted as semantic rejection.','CSharp_tests_executed':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
def save():(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save();pending=set(expected)
try:
 while pending:
  ready=[]
  for name in sorted(pending):
   p=root/'final/rust-tests/certificates'/f'{name}.rust.json'
   if not p.exists():continue
   try:r=json.loads(p.read_text())
   except json.JSONDecodeError:continue
   ready.append((name,p,r))
  if not ready:
   state=json.loads((root/'final/rust-tests/result.json').read_text());assert state['status']=='running',('missing controls',pending,state['status']);time.sleep(2);continue
  for name,rust_path,rust in ready:
   certificate=rust_path.with_suffix('').with_suffix('.mpcert');assert certificate.name==name+'.mpcert'
   before=seal(certificate);report=seal(rust_path);assert rust['verdict']=='rejected' and rust['error_code']=='KERNEL_CORE_CHECK';detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH'
   cmd=[binary['path'],'verify',str(certificate)];start=time.monotonic();run=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,timeout=300)
   stdout,stderr=out/(name+'.stdout.log'),out/(name+'.stderr.log');stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr);go=json.loads(run.stdout)
   row={'name':name,'certificate':before,'rust_report':report,'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go,'same_original_bytes':True};record['cases'].append(row);save()
   assert seal(certificate)==before and seal(rust_path)==report
   assert run.returncode==1 and not run.stderr and go['verdict']=='rejected' and go['error_kind']=='core_check',row
   if detail['details']['kind']=='check_type_mismatch':
    a,b=(detail['details'][k]for k in ['inferred_kind','expected_kind']);assert re.fullmatch(r'term [0-9]+ inferred '+re.escape(a)+' but expected '+re.escape(b),go['error_detail']),row
   else:assert detail['details']['kind']=='lambda_domain_mismatch' and go['error_detail']=='lambda domain mismatch',row
   pending.remove(name)
 assert len(record['cases'])==83 and seal(binary['path'])==binary
 for s in previous['go_production_sources']:assert seal(s['path'])==s
 record['status']='passed';record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save();print(json.dumps({'status':'passed','same_original_byte_semantic_rejections':83}))
except BaseException:record['status']='failed';save();raise
