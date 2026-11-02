"""Join original same-byte controls and the narrowly repaired metadata root."""
from pathlib import Path
import datetime,hashlib,json,subprocess,sys,time
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1];assert attempt=='final'
out=root/'paired-final';out.mkdir(exist_ok=False)
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
original=root/'final/rust-tests/result.json';old=read(original)
assert old['status']=='failed' and old['exit_code']==101 and old['changed_inputs_during_execution']==[]
assert '2 passed; 1 failed' in (original.parent/'stdout.log').read_text()
assert 'left: 2669' in (original.parent/'stderr.log').read_text() and 'right: 2665' in (original.parent/'stderr.log').read_text()
for s in old['sources']:
 captured=original.parent/'inputs'/s['path'];a=seal(captured);assert(a['sha256'],a['bytes'])==(s['sha256'],s['bytes'])
 if s['path']=='crates/mpk-vc/tests/go_practical_word_shifts.rs':
  raw=captured.read_bytes();assert raw.count(b'assert_eq!(retained, 2665);')==1
  assert Path(s['path']).read_bytes()==raw.replace(b'assert_eq!(retained, 2665);',b'assert_eq!(retained, 2669);').replace(b'matched exact2665 original named shift prerequisites',b'matched exact2669 original named shift prerequisites')
 else:assert seal(s['path'])==s
fixed_path=root/'metadata-corrected/rust-tests/result.json';fixed=read(fixed_path)
assert fixed['status']=='passed' and fixed['exit_code']==0 and fixed['changed_inputs_during_execution']==[]
assert '1 passed; 0 failed' in (fixed_path.parent/'stdout.log').read_text()
assert 'matched exact2669 original named shift prerequisites' in (fixed_path.parent/'stdout.log').read_text()
for s in fixed['sources']:assert seal(s['path'])==s
controls_path=root/'go-controls-complete/result.json';controls=read(controls_path);assert controls['status']=='passed' and len(controls['cases'])==101
previous_path=root.parent/'core-interfaces/paired/result.json';previous=read(previous_path);binary=previous['go_binary'];assert seal(binary['path'])==binary
for s in previous['go_production_sources']:assert seal(s['path'])==s
cases=[]
for row in controls['cases']:
 for k in ['certificate','rust_report','stdout','stderr']:assert seal(row[k]['path'])==row[k]
 assert row['same_original_bytes'] and row['rust_observation']['verdict']==row['go_observation']['verdict']=='rejected'
 assert json.loads(row['rust_observation']['error_detail'])['code']=='CORE_TYPE_MISMATCH';cases.append(row)
positive={'word-shifts','honest-different-shifts'};certificates=sorted((fixed_path.parent/'certificates').glob('*.mpcert'));assert {p.stem for p in certificates}==positive
assert (fixed_path.parent/'certificates/word-shifts.mpcert').read_bytes()==(original.parent/'certificates/word-shifts.mpcert').read_bytes()==(root/'applied-count-cases/rust-construction/certificates/word-shifts.mpcert').read_bytes()
record={'schema':'mpk.go_practical.paired_word_shifts.v1','status':'running','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_combined_Rust_execution':seal(original),'repaired_single_affected_Rust_root':seal(fixed_path),'controls_original_Go_execution':seal(controls_path),'go_binary':binary,'unchanged_go_build_evidence':seal(previous_path),'unchanged_go_production_sources':previous['go_production_sources'],'cases':cases,'selection_reason':'Three affected roots validated across the original combined101-control/504-model run and the one metadata-root rerun. Original combined failure is only a retained-declaration count expectation: the existing Builder seed adds4 original Std.Bool definitions to2665 planned root dependencies, yielding2669 originals and436 new laws. Only the assertion and its report-text count literals change2665 to2669. Production proof bytes and all helpers/model/query inputs stay exact. Repeating unaffected101controls/model is avoided. Two accepted complete positives and101 genuine same-byte type rejections. Full variable-shift numeric/source/profile/application proof remains due.','CSharp_tests_executed':False,'T02_W01_complete':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
def save():(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 for certificate in certificates:
  name=certificate.stem;rust_path=certificate.with_suffix('.rust.json');rust=read(rust_path);assert rust['verdict']=='accepted' and rust['axiom_count']==0
  cmd=[binary['path'],'verify',str(certificate)];start=time.monotonic();run=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,timeout=60)
  stdout,stderr=out/(name+'.stdout.log'),out/(name+'.stderr.log');stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr);go=json.loads(run.stdout)
  row={'name':name,'certificate':seal(certificate),'rust_report':seal(rust_path),'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go,'same_original_bytes':True};record['cases'].append(row);save()
  assert run.returncode==0 and not run.stderr and go['verdict']=='accepted' and go.get('axiom_count',0)==0 and go['hashes']==rust['hashes'] and go['declaration_count']==rust['declaration_count']
 assert len(record['cases'])==103 and len({r['name']for r in record['cases']})==103
 record['status']='passed';record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save();print(json.dumps({'status':'passed','same_byte_cases':103,'accepted':2,'genuine_type_rejections':101}))
except BaseException:record['status']='failed';save();raise
