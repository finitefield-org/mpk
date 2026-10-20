"""Join the complete original controls and current accepted ownership positives."""
from pathlib import Path
import datetime,hashlib,json,runpy,subprocess,sys,time
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());assert sys.argv[1:] == ['final'];out=root/'paired-final';out.mkdir(exist_ok=False)
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
execution_path=root/'final/rust-tests/result.json';ex=read(execution_path);assert ex['status']=='passed' and ex['exit_code']==0 and not ex['changed_inputs_during_execution'];assert '2 passed; 0 failed' in (execution_path.parent/'stdout.log').read_text()
for s in ex['sources']:
 assert seal(s['path'])==s
 copied=seal(execution_path.parent/'inputs'/s['path']);assert(copied['sha256'],copied['bytes'])==(s['sha256'],s['bytes'])
expected=runpy.run_path(str(root/'planned-controls.py'))['names']();controls=read(root/'go-controls/result.json');assert controls['status']=='passed' and {c['name']for c in controls['cases']}==expected
previous_path=root.parent/'core-interfaces/paired/result.json';previous=read(previous_path);binary=previous['go_binary'];assert seal(binary['path'])==binary
for s in previous['go_production_sources']:assert seal(s['path'])==s
cases=[]
for row in controls['cases']:
 for key in ['certificate','rust_report','stdout','stderr']:assert seal(row[key]['path'])==row[key]
 assert row['same_original_bytes'] and row['rust_observation']['verdict']==row['go_observation']['verdict']=='rejected' and json.loads(row['rust_observation']['error_detail'])['code']=='CORE_TYPE_MISMATCH';cases.append(row)
positive={'word-shift-counts','honest-different-count-shifts'}
certificates=sorted((execution_path.parent/'certificates').glob('*.mpcert'));assert len(certificates)==85 and {p.stem for p in certificates}==expected|positive
assert [seal(p)for p in certificates]==ex['certificates']
assert (execution_path.parent/'certificates/word-shift-counts.mpcert').read_bytes()==(root/'actual-signed-interpretations/rust-construction/certificates/word-shift-counts.mpcert').read_bytes()
record={'schema':'mpk.go_practical.paired_full64_shift_proofs.v1','status':'running','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'Rust_execution':seal(execution_path),'original_Go_control_execution':seal(root/'go-controls/result.json'),'go_binary':binary,'unchanged_go_build_evidence':seal(previous_path),'unchanged_go_production_sources':previous['go_production_sources'],'outer_observation_wait_seconds':300,'cases':cases,'scope':'Two scoped Rust roots and all83 complete original typed controls plus2 current positives. Every full canonical byte stream is identical in both unchanged production cores. No checker/axiom/fuel/artifact-cap widening, prefix acceptance, source/project/profile/Linux-release claim or CSharp tests. Full T02-T11 and all169 application demands remain.','full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
def save():(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 for certificate in certificates:
  if certificate.stem not in positive:continue
  name=certificate.stem;rust_path=certificate.with_suffix('.rust.json');rust=read(rust_path);assert rust['verdict']=='accepted' and rust['axiom_count']==0
  cmd=[binary['path'],'verify',str(certificate)];start=time.monotonic();run=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,timeout=300)
  stdout,stderr=out/(name+'.stdout.log'),out/(name+'.stderr.log');stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr);go=json.loads(run.stdout)
  row={'name':name,'certificate':seal(certificate),'rust_report':seal(rust_path),'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go,'same_original_bytes':True};record['cases'].append(row);save()
  assert not run.stderr and run.returncode==0 and go['verdict']=='accepted' and go.get('axiom_count',0)==0 and go['hashes']==rust['hashes'] and go['declaration_count']==rust['declaration_count']
 assert len(record['cases'])==85;record['status']='passed';record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save();print(json.dumps({'status':'passed','same_byte_streams':85,'accepted':2,'genuine_core_type_mismatches':83}))
except BaseException:record['status']='failed';save();raise
