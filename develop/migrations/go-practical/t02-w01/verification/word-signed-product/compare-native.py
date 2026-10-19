"""Actual multiplication stages, native signed Go and exact Python integers."""
from pathlib import Path
import datetime,hashlib,json,sys

root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1];assert attempt.replace('-','').isalnum()
out=root/('native-comparison-'+attempt);out.mkdir(exist_ok=False)
def seal(path):
 p=Path(path);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
def read(path):return json.loads(Path(path).read_text())
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
native_path=root/'native/result.json';native=read(native_path);assert native['status']=='passed' and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries']:assert seal(native[key]['path'])==native[key]
for command in native['commands']:
 assert command['exit_code']==0
 for key in ['stdout','stderr']:assert seal(command[key]['path'])==command[key]
queries=read(native['original_queries']['path']);observations=read(root/'native/run1.stdout.log')
data=root/attempt/'rust-tests/model';model=read(data/'model.json');metrics=read(data/'metrics.json')
assert len(queries)==len(observations)==len(model)==48 and observations==model
assert metrics['Nat_witness_correction_guard_or_theorem_bodies_executed'] is False
assert (data/'certificate.mpcert').read_bytes()==(root/attempt/'rust-tests/certificates/word-signed-product-foundation.mpcert').read_bytes()
origins=read(root/'query-origins.json');assert seal(origins['original_input']['path'])==origins['original_input']
assert metrics['original_frozen_query_sha256']==origins['original_input']['sha256']
assert queries==[q for q in read(origins['original_input']['path']) if q['operation']=='Mul']
assert [q['id']for q in queries]==origins['original_IDs']
oracle=[];stage_count=0;carries=0;exact_count=0;min_minus_one=0
for q,actual in zip(queries,model,strict=True):
 w=q['width'];m=1<<w;mask=m-1;x,y=int(q['x_hex'],16),int(q['y_hex'],16);assert 0<=x<m and 0<=y<m
 acc,discard,stages=0,0,[]
 for i in range(w):
  left,lost=(x<<i)%m,(x<<i)//m;flag=(y>>i)&1;selected=left if flag else 0;carry=acc+selected>=m;acc=(acc+selected)%m
  discard+=flag*lost+int(carry)
  assert acc+m*discard==x*(y&((1<<(i+1))-1))
  stages.append({'index':i,'left_hex':format(left,'0'+str(w//4)+'x'),'shift_discard':str(lost),'carry':carry,'acc_hex':format(acc,'0'+str(w//4)+'x')});stage_count+=1;carries+=int(carry)
 unsigned=x*y;bits=unsigned&mask;sx,sy,sr=x>=m//2,y>=m//2,bits>=m//2
 signed_x=x-(m if sx else 0);signed_y=y-(m if sy else 0);signed_result=bits-(m if sr else 0);signed_product=signed_x*signed_y
 positive=unsigned//m+int(sr)+m*int(sx and sy);negative=x*int(sy)+y*int(sx)
 correction=(signed_product-signed_result)//m
 assert (signed_product-signed_result)%m==0 and positive-negative==correction
 assert signed_product==signed_result+m*correction and discard==unsigned//m
 exact=signed_product==signed_result;assert exact==(positive==negative);exact_count+=int(exact)
 if x==m//2 and y==m-1:
  assert not exact and correction==1 and signed_product==m//2 and signed_result==-m//2;min_minus_one+=1
 expected={'id':q['id'],'word_hex':format(bits,'0'+str(w//4)+'x'),'go_signed_result':str(signed_result),'unsigned_product':str(unsigned),'discarded_quotient':str(unsigned//m),'signed_x':str(signed_x),'signed_y':str(signed_y),'signed_product':str(signed_product),'correction_positive':str(positive),'correction_negative':str(negative),'signed_correction':str(correction),'no_overflow':exact,'stages':stages}
 assert expected==actual,(q,expected,actual);oracle.append(expected)
assert stage_count==1440 and carries==metrics['observed_true_carries']==107
assert exact_count==metrics['exact_signed_products']==21 and 48-exact_count==metrics['overflowed_signed_products']==27
assert min_minus_one==metrics['minimum_times_minus_one_cases']==4
write(out/'independent-integer-oracle.json',oracle)
record={'schema':'mpk.go_practical.signed_product_native_comparison.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'native_execution':seal(native_path),'query_origins':seal(root/'query-origins.json'),'actual_certificate':seal(data/'certificate.mpcert'),'actual_Boolean_model':seal(data/'model.json'),'model_metrics':seal(data/'metrics.json'),'independent_integer_oracle':seal(out/'independent-integer-oracle.json'),
 'selected_original_queries':48,'actual_stages':1440,'actual_true_carries':107,'exact_signed_products':21,'overflowed_signed_products':27,'minimum_times_minus_one_cases':4,
 'scope':'Actual decoded Boolean multiplication/fixed shifts/carries/sign bits agree with two identical fresh typed signed/unsigned Go runs, math/big products/corrections and independent Python exact integers. Host derives the ordinary unsigned-discard recurrence and both correction parts from actual bits/carries. The Nat witness/correction/guard or theorem bodies are not executed by the Bool-only model; universal actual guard equivalence checks separately in both cores. Minimum signed value times minus one wraps in Go, has correction1 and false exact-result guard. No source-program, authoritative immutable application replay or native Linux release claim.',
 'CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
write(out/'result.json',record);print(json.dumps({'status':'passed','queries':48,'stages':1440,'exact':21,'overflow':27,'minimum_minus_one':4}))
