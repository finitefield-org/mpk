"""New actual top-bit observations and half-range scalar counterparts."""
from pathlib import Path
import datetime,hashlib,json,sys
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1];assert attempt.replace('-','').isalnum()
out=root/('native-comparison-'+attempt);out.mkdir(exist_ok=False)
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
native_path=root/'native/result.json';native=read(native_path);assert native['status']=='passed'and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries']:assert seal(native[key]['path'])==native[key]
for cmd in native['commands']:
 assert cmd['exit_code']==0
 for key in ['stdout','stderr']:assert seal(cmd[key]['path'])==cmd[key]
queries=read(native['original_queries']['path']);observations=read(root/'native/run1.stdout.log')
data=root/attempt/'rust-tests/model';model=read(data/'model.json');metrics=read(data/'metrics.json')
assert len(queries)==len(observations)==len(model)==32 and observations==model
assert metrics['Nat_half_low_complement_gap_guard_or_theorem_bodies_executed']is False
assert (data/'certificate.mpcert').read_bytes()==(root/attempt/'rust-tests/certificates/word-signed-bounds-foundation.mpcert').read_bytes()
origins=read(root/'query-origins.json');assert seal(origins['original_input']['path'])==origins['original_input']
assert metrics['original_frozen_query_sha256']==origins['original_input']['sha256']
assert queries==[q for q in read(origins['original_input']['path'])if q['operation']=='Neg']
assert [q['id']for q in queries]==origins['original_IDs']
oracle=[];minima=0;zeros=0;nonnegative=0
for q,actual in zip(queries,model,strict=True):
 w=q['width'];m=1<<w;half=m//2;x=int(q['x_hex'],16);assert 0<=x<m
 sx=x-m if x>=half else x;neg=(-x)%m;magnitude=abs(sx)
 # Remainders, subtraction and half minus absolute value are independent of
 # both the model's balanced recurrence and Go's masking/conditional gap.
 low=x%half;complement=half-low-1;gap=half-magnitude
 expected={'id':q['id'],'word_hex':format(x,'0'+str(w//4)+'x'),'neg_hex':format(neg,'0'+str(w//4)+'x'),'abs_hex':format(magnitude,'0'+str(w//4)+'x'),'signed_x':str(sx),'half':str(half),'modulus':str(m),'low':str(low),'lower_complement':str(complement),'gap':str(gap),'source_sign':sx<0,'negation_sign':neg>=half,'magnitude_sign':magnitude>=half,'unsigned_below_half':x<half,'magnitude_half_bound':magnitude<=half,'magnitude_equals_half':magnitude==half,'input_is_minimum':x==half}
 assert expected==actual,(q,expected,actual);oracle.append(expected)
 assert low+complement+1==half and magnitude+gap==half and x==low+half*int(sx<0)
 assert (magnitude==half)==(x==half)==(magnitude>=half)
 if sx>=0:assert magnitude<half;nonnegative+=1
 if x==half:assert gap==0 and neg==half;minima+=1
 zeros+=int(x==0)
assert minima==metrics['minimum_magnitude_cases']==4 and zeros==metrics['zero_cases']==4
assert nonnegative==metrics['nonnegative_cases']
assert metrics['actual_word_observations']==metrics['actual_Top_definition_applications']==96 and metrics['actual_word_bit_observations']==2880
write(out/'independent-integer-oracle.json',oracle)
record={'schema':'mpk.go_practical.signed_bounds_native_comparison.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_execution':seal(native_path),'query_origins':seal(root/'query-origins.json'),'actual_certificate':seal(data/'certificate.mpcert'),'actual_Boolean_model':seal(data/'model.json'),'model_metrics':seal(data/'metrics.json'),'independent_integer_oracle':seal(out/'independent-integer-oracle.json'),'selected_original_queries':32,'actual_word_observations':96,'actual_Top_definition_applications':96,'actual_word_bit_observations':2880,'minimum_magnitude_cases':4,'zero_cases':4,'nonnegative_cases':nonnegative,'scope':metrics['scope'],'CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
write(out/'result.json',record);print(json.dumps({'status':'passed','queries':32,'actual_words':96,'actual_Top_calls':96,'bits':2880,'minimum':4,'zero':4,'nonnegative':nonnegative}))
