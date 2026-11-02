"""Actual Boolean negation/magnitude, typed native Go and independent integers."""
from pathlib import Path
import datetime,hashlib,json,sys
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1]
assert attempt.replace('-','').isalnum()
out=root/('native-comparison-'+attempt);out.mkdir(exist_ok=False)
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def read(p):return json.loads(Path(p).read_text())
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
native_path=root/'native/result.json';native=read(native_path)
assert native['status']=='passed' and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries']:assert seal(native[key]['path'])==native[key]
for cmd in native['commands']:
 assert cmd['exit_code']==0
 for key in ['stdout','stderr']:assert seal(cmd[key]['path'])==cmd[key]
queries=read(native['original_queries']['path']);observations=read(root/'native/run1.stdout.log')
data=root/attempt/'rust-tests/model';model=read(data/'model.json');metrics=read(data/'metrics.json')
assert len(queries)==len(observations)==len(model)==32 and observations==model
assert metrics['Nat_correction_guard_or_theorem_bodies_executed'] is False
assert (data/'certificate.mpcert').read_bytes()==(root/attempt/'rust-tests/certificates/word-signed-magnitude-foundation.mpcert').read_bytes()
origins=read(root/'query-origins.json');assert seal(origins['original_input']['path'])==origins['original_input']
assert metrics['original_frozen_query_sha256']==origins['original_input']['sha256']
assert queries==[q for q in read(origins['original_input']['path'])if q['operation']=='Neg']
assert [q['id']for q in queries]==origins['original_IDs']
oracle=[];stage_count=0;minima=0;zeros=0;exact=0
for q,actual in zip(queries,model,strict=True):
 w=q['width'];m=1<<w;x=int(q['x_hex'],16);assert 0<=x<m
 neg=(-x)%m;sx=x-(m if x>=m//2 else 0);sn=neg-(m if neg>=m//2 else 0)
 magnitude=abs(sx);cp=int(x>=m//2)+int(neg>=m//2);cn=int(x!=0)
 correction=(-sx-sn)//m;assert (-sx-sn)%m==0 and correction==cp-cn
 guard=-sx==sn;assert guard==(cp==cn)
 stages=[]
 for i in range(w):
  stages.append({'index':i,'complement_bit':((x>>i)&1)==0,'carry_in':x%(1<<i)==0,'result_bit':((neg>>i)&1)!=0,'carry_out':x%(1<<(i+1))==0});stage_count+=1
 expected={'id':q['id'],'neg_hex':format(neg,'0'+str(w//4)+'x'),'abs_hex':format(magnitude,'0'+str(w//4)+'x'),'signed_x':str(sx),'go_signed_negation':str(sn),'exact_negation':str(-sx),'absolute_value':str(magnitude),'carry':x==0,'input_zero':x==0,'negation_zero':x==0,'magnitude_zero':x==0,'input_defined':x!=0,'negation_defined':x!=0,'magnitude_defined':x!=0,'correction_positive':cp,'correction_negative':cn,'signed_correction':str(correction),'no_overflow':guard,'stages':stages}
 assert expected==actual,(q,expected,actual);oracle.append(expected)
 assert x+neg+m*int(x==0)==m
 if x==m//2:assert not guard and correction==1 and magnitude==m//2;minima+=1
 zeros+=int(x==0);exact+=int(guard)
assert stage_count==metrics['actual_full_adder_stages']==960
assert minima==metrics['minimum_negation_wrap_cases']==4
assert zeros==metrics['zero_cases']==4 and exact==metrics['exact_negations']==28
write(out/'independent-integer-oracle.json',oracle)
record={'schema':'mpk.go_practical.signed_magnitude_native_comparison.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_execution':seal(native_path),'query_origins':seal(root/'query-origins.json'),'actual_certificate':seal(data/'certificate.mpcert'),'actual_Boolean_model':seal(data/'model.json'),'model_metrics':seal(data/'metrics.json'),'independent_integer_oracle':seal(out/'independent-integer-oracle.json'),'selected_original_queries':32,'actual_stages':960,'exact_negations':28,'minimum_negation_wrap_cases':4,'zero_cases':4,'scope':'Actual decoded Boolean Neg, AbsBits, every complement-plus-one carry and IsZero/DivideDefined agree with two identical fresh typed signed/unsigned Go runs and math/big exact negation/absolute value. Independent Python integers derive carries using divisibility of the original low-bit prefixes, rather than repeating the adder. Host computes both correction parts and the exact-result guard from actual Boolean observations. Nat/correction/guard/theorem bodies are not executed; universal meaning checks separately in both cores. Minimum negation wraps, while its unsigned magnitude is the positive half modulus. No source-program, authoritative immutable application replay or native Linux release claim.','CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
write(out/'result.json',record);print(json.dumps({'status':'passed','queries':32,'stages':960,'exact':28,'minimum_wrap':4,'zero':4}))
