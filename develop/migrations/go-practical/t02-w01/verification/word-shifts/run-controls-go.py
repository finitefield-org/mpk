"""Inspect each completed original Rust control stream once with the Go core."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,time
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());out=root/'go-controls-complete';out.mkdir(exist_ok=False)
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
previous_path=root.parent/'core-interfaces/paired/result.json';previous=json.loads(previous_path.read_text());binary=previous['go_binary'];assert seal(binary['path'])==binary
for s in previous['go_production_sources']:assert seal(s['path'])==s
expected=set()
for n in ['PowerMinusBalance','PowerMinusAdd','PowerPositive','PopConservation6','CubeValueBound6','RightCompose','CountSumZero','CountNonzeroPositive','WeightedBitZero','CountDeMorgan']:expected.add('False'+n)
for n,i in [('RightCompose',11),('RightCompose',12),('WeightedBitZero',2),('CountPairZero',4),('CountPairZero',5)]:expected.add(f'FakePremise-{n}-{i}')
for w in [8,16,32,64]:
 for j in range(w.bit_length()-1):
  for n in ['UnsignedRightConservation','SignedRightConservation','RemainderBound']:expected.add(f'False{n}-W{w}-{1<<j}')
 for n in ['FalseSignedRightTop','FalseFilledRight','WrongActualRight','WrongActualSignedRight']:expected.add(f'{n}-W{w}')
for d in range(3,7):
 for n in ['FalseCountBalance','FalseCountHighZero','WrongCountHigh','WrongCountHighValue']:expected.add(f'{n}-{d}')
assert len(expected)==101
record={'schema':'mpk.go_practical.word_shift_original_control_observation.v1','status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'unchanged_go_binary':binary,'unchanged_go_build_evidence':seal(previous_path),'expected_original_control_names':sorted(expected),'cases':[],'preserved_original_observer_failure':seal(root/'go-controls/result.json'),'scope':'Each complete original immutable certificate/report pair is read after Rust closes its report, then the same complete bytes are checked once by unchanged Go. No prefix acceptance, source exclusion or inferred Rust test success is used; final combined scoped-test result and the separately repaired metadata assertion remain independently required.','CSharp_tests_executed':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
def save():(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
prior=json.loads((root/'go-controls/result.json').read_text());assert prior['status']=='failed' and len(prior['cases'])==1
row=prior['cases'][0]
for k in ['certificate','rust_report','stdout','stderr']:assert seal(row[k]['path'])==row[k]
detail=json.loads(row['rust_observation']['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH' and detail['details']['kind']=='lambda_domain_mismatch' and row['go_observation']['error_detail']=='lambda domain mismatch' and row['go_observation']['verdict']=='rejected' and row['exit_code']==1
record['cases'].append(row)
save()
try:
 for name in sorted(expected-{r['name'] for r in record['cases']}):
  certificate=root/'final/rust-tests/certificates'/f'{name}.mpcert';rust_path=certificate.with_suffix('.rust.json')
  while True:
   if rust_path.exists():
    try:rust=json.loads(rust_path.read_text());break
    except json.JSONDecodeError:pass
   state=json.loads((root/'final/rust-tests/result.json').read_text())
   assert state['status']=='running',('missing original complete control',name,state['status'])
   time.sleep(2)
  rust_seal,cert_seal=seal(rust_path),seal(certificate)
  assert rust['verdict']=='rejected' and rust['error_code']=='KERNEL_CORE_CHECK'
  detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH'
  cmd=[binary['path'],'verify',str(certificate)];start=time.monotonic();run=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,timeout=60)
  stdout,stderr=out/(name+'.stdout.log'),out/(name+'.stderr.log');stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr)
  go=json.loads(run.stdout)
  row={'name':name,'certificate':cert_seal,'rust_report':rust_seal,'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go,'same_original_bytes':True};record['cases'].append(row);save()
  assert seal(certificate)==cert_seal and seal(rust_path)==rust_seal
  assert run.returncode==1 and not run.stderr and go['verdict']=='rejected' and go['error_kind']=='core_check',row
  if detail['details']['kind']=='check_type_mismatch':
   inferred,expected_kind=(detail['details'][k] for k in ['inferred_kind','expected_kind']);assert re.fullmatch(r'term [0-9]+ inferred '+re.escape(inferred)+' but expected '+re.escape(expected_kind),go['error_detail']),row
  else:assert detail['details']['kind']=='lambda_domain_mismatch' and go['error_detail']=='lambda domain mismatch',row
 assert seal(binary['path'])==binary
 for s in previous['go_production_sources']:assert seal(s['path'])==s
 record['status']='passed';record['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save();print(json.dumps({'status':'passed','same_byte_genuine_controls':101}))
except BaseException:
 record['status']='failed';save();raise
