"""Compare actual Bool DAGs and native stages with exact prefix division."""
from pathlib import Path
import datetime
import hashlib
import json
import sys

root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt=sys.argv[1];assert attempt.replace('-','').isalnum()
out=root/('native-comparison-'+attempt);out.mkdir(exist_ok=False)
def read(path):return json.loads(Path(path).read_text())
def seal(path):
 p=Path(path);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
native_path=root/'native/result.json';native=read(native_path)
assert native['status']=='passed' and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries']:assert seal(native[key]['path'])==native[key]
for command in native['commands']:
 assert command['exit_code']==0
 for key in ['stdout','stderr']:assert seal(command[key]['path'])==command[key]
queries=read(native['original_queries']['path']);native_rows=read(root/'native/run1.stdout.log')
data=root/attempt/'rust-tests/model';model=read(data/'model.json');metrics=read(data/'metrics.json')
assert model==native_rows
assert (data/'certificate.mpcert').read_bytes()==(root/attempt/'rust-tests/certificates/word-division-foundation.mpcert').read_bytes()
assert metrics['query_sha256']==seal(native['original_queries']['path'])['sha256']
assert metrics['Nat_prefix_denotation_order_or_theorem_bodies_executed'] is False
origins=read(root/'query-origins.json');assert seal(origins['original_input']['path'])==origins['original_input']
original=read(origins['original_input']['path']);by_id={q['id']:q for q in original}
assert queries['divisions']==[q for q in original if q['operation'] in ['UnsignedDiv','UnsignedRem']]
assert len(queries['divisions'])==len(model['divisions'])==96
assert len(queries['steps'])==len(model['steps'])==34
assert [q['id'] for q in queries['divisions']]==origins['division_original_IDs']
for q,origin in zip(queries['steps'],origins['arbitrary_restore_origins'],strict=True):
 old=by_id[origin['original_id']]
 assert old['operation']=='Sub' and origin['step_id']==q['id'] and origin['low']==q['low']
 assert (old['width'],old['x_hex'],old['y_hex'])==(q['width'],q['r_hex'],q['d_hex'])
def expected_step(w,r,d,low):
 modulus=1<<w;mask=modulus-1;s=(2*r+int(low))&mask;sub=(s-d)&mask
 exact=2*r+int(low);digit=exact//d;remainder=exact%d
 assert 0<=r<d<modulus and digit in [0,1]
 high=r>=modulus//2;carry=s>=d
 assert bool(digit)==(high or carry) and s+modulus*int(high)==exact
 assert s+modulus==sub+d+modulus*int(carry)
 def hx(x):return format(x,'0'+str(w//4)+'x')
 return {'prior_hex':hx(r),'shifted_hex':hx(s),'sub_hex':hx(sub),'remainder_hex':hx(remainder),
         'low':low,'high':high,'carry':carry,'decision':bool(digit)}
division_oracle=[];step_oracle=[];stages=0;failures=0;high_bits=0
for q,actual in zip(queries['divisions'],model['divisions'],strict=True):
 w=q['width'];x,y=int(q['x_hex'],16),int(q['y_hex'],16);assert 0<=x<1<<w and 0<=y<1<<w
 if y==0:
  expected={'id':q['id'],'kind':'runtime_safety_failure','reason':'division_by_zero','guard':False};failures+=1
 else:
  rows=[]
  for i in range(w):
   # Compute each expected remainder/quotient directly from exact prefix / and
   # %, independently of the native or ordinary restoring implementation.
   before=x>>(w-i);after=x>>(w-1-i);r=before%y;low=bool(after&1)
   row=expected_step(w,r,y,low)
   assert int(row['remainder_hex'],16)==after%y
   row.update(index=i,dividend_prefix=str(after),quotient_prefix=str(after//y))
   rows.append(row);stages+=1
  result=x//y if q['operation']=='UnsignedDiv' else x%y
  expected={'id':q['id'],'kind':'word','word_hex':format(result,'0'+str(w//4)+'x'),'guard':True,'stages':rows}
 assert expected==actual,(q,expected,actual)
 division_oracle.append(expected)
for q,actual in zip(queries['steps'],model['steps'],strict=True):
 expected=expected_step(q['width'],int(q['r_hex'],16),int(q['d_hex'],16),q['low']);expected['id']=q['id']
 assert expected==actual,(q,expected,actual)
 high_bits+=int(expected['high']);step_oracle.append(expected)
assert stages==metrics['actual_source_division_stages']==2640
assert failures==metrics['runtime_domain_failures']==8
assert high_bits==metrics['observed_arbitrary_overflow_bits']==10
write(out/'independent-integer-oracle.json',{'divisions':division_oracle,'steps':step_oracle})
record={'schema':'mpk.go_practical.word_division_native_comparison.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'native_execution':seal(native_path),'query_origins':seal(root/'query-origins.json'),'actual_certificate':seal(data/'certificate.mpcert'),
 'actual_Boolean_certificate_model':seal(data/'model.json'),'model_metrics':seal(data/'metrics.json'),'independent_integer_oracle':seal(out/'independent-integer-oracle.json'),
 'original_division_queries':96,'actual_division_stages':stages,'native_runtime_division_by_zero_rows':failures,'arbitrary_restore_queries':34,'arbitrary_overflow_bits':high_bits,
 'scope':'All actual retained source stage/carry/output bits and additional full-width arbitrary-remainder restore bits agree with two identical fresh typed native Go runs and independent Python exact prefix division/remainder. Eight zero-divisor rows match actual Go runtime panics and actual false circuit guards; totalized zero bits are not accepted Go division results. The Bool-only model does not execute unary Nat prefixes/denotations/order/theorem bodies. Universal genuine Nat laws check separately in both cores. No source-program, authoritative immutable application replay or native Linux release claim.',
 'CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
write(out/'result.json',record)
print(json.dumps({'status':'passed','original_queries':96,'stages':stages,'runtime_panics':failures,'arbitrary_steps':34,'overflow_bits':high_bits}))
