"""All original shift rows: actual Boolean circuits, typed Go, integer oracle."""
from pathlib import Path
import datetime,hashlib,json,sys
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1]
assert attempt.replace('-','').isalnum()
out=root/('native-comparison-'+attempt);out.mkdir(exist_ok=False)
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
native=read(root/'native/result.json');assert native['status']=='passed' and native['native_rows']==504 and native['two_native_outputs_byte_identical']
for k in ['original_source','owned_source','original_queries','owned_queries','complete_original_prepared_image']:assert seal(native[k]['path'])==native[k]
for r in native['complete_original_image_checks']:assert r['status']=='passed' and r['files']==5577
for r in native['commands']:
 assert r['exit_code']==0
 for k in ['stdout','stderr']:assert seal(r[k]['path'])==r[k]
assert (root/'native/run1.stdout.log').read_bytes()==(root/'native/run2.stdout.log').read_bytes()
origins=read(root/'query-origins.json');source=origins['original_input'];assert seal(source['path'])==source
qs=read(root/'queries.json');assert qs==[q for q in read(source['path']) if q['operation'] in ['ShiftLeft','ShiftRight','SignedShiftRight']]
assert [q['id']for q in qs]==origins['original_IDs']
execution_path=root/attempt/'rust-tests/result.json';ex=read(execution_path);assert ex['status']=='failed' and ex['exit_code']==101 and ex['changed_inputs_during_execution']==[]
assert '2 passed; 1 failed' in (execution_path.parent/'stdout.log').read_text()
assert 'left: 2669' in (execution_path.parent/'stderr.log').read_text() and 'right: 2665' in (execution_path.parent/'stderr.log').read_text()
for r in ex['sources']:
 captured=execution_path.parent/'inputs'/r['path'];saved=captured.read_bytes();assert hashlib.sha256(saved).hexdigest()==r['sha256'] and len(saved)==r['bytes']
 if r['path']=='crates/mpk-vc/tests/go_practical_word_shifts.rs':assert Path(r['path']).read_bytes()==saved.replace(b'assert_eq!(retained, 2665);',b'assert_eq!(retained, 2669);').replace(b'matched exact2665 original named shift prerequisites',b'matched exact2669 original named shift prerequisites')
 else:assert seal(r['path'])==r
model_path=execution_path.parent/'model/model.json';metrics_path=model_path.parent/'metrics.json'
model,metrics=read(model_path),read(metrics_path)
assert metrics['Nat_Int_witness_or_theorem_bodies_executed'] is False
assert metrics['original_frozen_query_sha256']==source['sha256']
assert (model_path.parent/'certificate.mpcert').read_bytes()==(execution_path.parent/'certificates/word-shifts.mpcert').read_bytes()
observed=read(root/'native/run1.stdout.log');assert len(qs)==len(model)==len(observed)==504 and observed==model
oracle=[];faults=high_defined=0
for q,actual in zip(qs,model,strict=True):
 w=q['width'];modulus=2**w;x=int(q['x_hex'],16);c=int(q['count_hex'],16);d=w.bit_length()-1
 assert q['target_width']==0 and 0<=x<modulus and 0<=c<2**64
 defined=not q['signed_count'] or c<2**63;faults+=not defined
 high=c>=w;lo=c%w;hi=c//w;assert c==lo+w*hi and 0<=lo<w and high==(hi!=0)
 result=stages=signed_left=None
 if defined:
  high_defined+=high
  sx=x if x<modulus//2 else x-modulus
  op=q['operation']
  if op=='ShiftLeft':r=0 if high else (x*2**c)%modulus;signed_left=format(r,'0'+str(w//4)+'x')
  elif op=='ShiftRight':r=0 if high else x//2**c
  else:r=(int(sx<0)*(modulus-1)) if high else (sx//2**c)%modulus
  result=format(r,'0'+str(w//4)+'x')
  v=x;stages=[format(v,'0'+str(w//4)+'x')]
  for i in range(d):
   k=2**i
   if (c//2**i)%2:
    if op=='ShiftLeft':v=(v*2**k)%modulus
    elif op=='ShiftRight':v=v//2**k
    else:
     s=v if v<modulus//2 else v-modulus
     v=(s//2**k)%modulus
   stages.append(format(v,'0'+str(w//4)+'x'))
 expected={'id':q['id'],'operation':q['operation'],'count_defined':defined,'native_count_panicked':not defined,'count_high':high,'low_count':str(lo),'high_value':str(hi),'large_count_excess':str(c-w) if high else None,'result_hex':result,'signed_left_result_hex':signed_left,'stage_hex':stages}
 assert expected==actual,(q,expected,actual)
 oracle.append(expected)
assert faults==24 and high_defined==309
write(out/'independent-integer-oracle.json',oracle)
record={'schema':'mpk.go_practical.word_shifts_native_comparison.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_execution':seal(root/'native/result.json'),'query_origins':seal(root/'query-origins.json'),'Rust_execution':seal(execution_path),'actual_Bool_model':seal(model_path),'model_metrics':seal(metrics_path),'independent_integer_oracle':seal(out/'independent-integer-oracle.json'),'original_queries':504,'defined_counts':480,'negative_count_native_panics':24,'defined_high_counts':309,'two_native_outputs_identical':True,'prepared_image_files':5577,'Nat_Int_witness_or_theorem_bodies_executed':False,'scope':'The original combined execution completed the model/control roots and failed only its separate retained-count assertion, then the assertion/report literals were narrowly repaired and the affected root rerun. All original complete rows. The model evaluates actual source Bool shifts/stages/guards and the new exact CountHigh bit tree; native Go separately observes negative-count panic and real typed full64 shifts. Python uses modular multiplication and floor integer division; for huge counts it compares zero/sign-fill output without materializing enormous powers. Finite observations do not supply any universal Nat/Int theorem. Full variable-shift conservation, source/application/profile binding and Linux release remain due.','CSharp_tests_executed':False,'T02_W01_complete':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
write(out/'result.json',record);print(json.dumps({k:record[k]for k in ['status','original_queries','defined_counts','negative_count_native_panics','defined_high_counts']}))
