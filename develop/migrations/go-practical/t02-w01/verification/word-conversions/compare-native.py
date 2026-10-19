"""Original full conversion rows: actual Bool circuits, typed Go and integers."""
from pathlib import Path
import datetime
import hashlib
import json
import sys

root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt=sys.argv[1]
assert attempt.replace('-','').isalnum()
out=root/('native-comparison-'+attempt);out.mkdir(exist_ok=False)
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
native_path=root/'native/result.json';native=read(native_path)
assert native['status']=='passed' and native['native_rows']==256 and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries','complete_original_prepared_image']:
 assert seal(native[key]['path'])==native[key]
for check in native['complete_original_image_checks']:assert check['status']=='passed' and check['files']==5577
for command in native['commands']:
 assert command['exit_code']==0
 for key in ['stdout','stderr']:assert seal(command[key]['path'])==command[key]
assert (root/'native/run1.stdout.log').read_bytes()==(root/'native/run2.stdout.log').read_bytes()
origins=read(root/'query-origins.json');original=origins['original_input'];assert seal(original['path'])==original
queries=read(root/'queries.json');assert queries==[q for q in read(original['path']) if 'To' in q['operation']]
assert [q['id'] for q in queries]==origins['original_IDs']
execution_path=root/attempt/'rust-tests/result.json';execution=read(execution_path)
assert execution['status']=='passed' and execution['changed_inputs_during_execution']==[]
for source in execution['sources']:assert seal(source['path'])==source
model_dir=execution_path.parent/'model';model=read(model_dir/'model.json');metrics=read(model_dir/'metrics.json')
assert metrics['Nat_Int_masks_ratios_quotients_or_theorem_bodies_executed'] is False
assert metrics['original_frozen_query_sha256']==original['sha256']
observations=read(root/'native/run1.stdout.log');assert len(queries)==len(model)==len(observations)==256
assert observations==model
assert (model_dir/'certificate.mpcert').read_bytes()==(execution_path.parent/'certificates/word-conversions.mpcert').read_bytes()
oracle=[];faults=exact_s=exact_u=0
for q,actual in zip(queries,model,strict=True):
 w,t=q['width'],q['target_width'];ms,mt=1<<w,1<<t;x=int(q['x_hex'],16)
 assert 0<=x<ms and q['operation'] in [f'UnsignedTo{t}',f'SignedTo{t}']
 sg=q['operation'].startswith('SignedTo');source_sign=x>=ms//2
 signed_x=x-ms if source_sign else x;source=signed_x if sg else x
 # Python uses arbitrary-precision modular conversion and signed differences.
 result=source%mt;target_sign=result>=mt//2;target_s=result-mt if target_sign else result
 cu=(source-result)//mt;cs=(source-target_s)//mt
 assert source==result+mt*cu==target_s+mt*cs
 discard=x//mt if w>=t else 0
 exu=not sg and w<=t;exs=sg and w<=t
 if exu:exact_u+=1;assert result==source
 if exs:exact_s+=1;assert target_s==source
 count_word=source%(1<<64) if sg else x
 defined=not sg or signed_x>=0
 if defined:assert count_word==x
 else:faults+=1
 expected={'id':q['id'],'operation':q['operation'],'result_hex':format(result,'0'+str(t//4)+'x'),
 'source_value':str(source),'target_unsigned':str(result),'target_signed':str(target_s),
 'source_modulus':str(ms),'target_modulus':str(mt),'discarded':str(discard),
 'extension_mask':str(mt-ms) if w<t else None,'target_sign':target_sign,'source_sign':source_sign,
 'correction_unsigned':str(cu),'correction_signed':str(cs),'unsigned_refinement':True,'signed_refinement':True,
 'exact_unsigned':exu,'exact_signed':exs,'count_word_hex':format(count_word,'016x'),
 'count_defined':defined,'native_count_panicked':not defined}
 assert expected==actual,(q,expected,actual)
 oracle.append(expected)
assert faults==64 and exact_s==exact_u==80
assert metrics['actual_false_signed_count_guards']==faults and metrics['defined_counts']==192
assert metrics['actual_source_conversion_results']==metrics['actual_count_normalizations']==256
write(out/'independent-integer-oracle.json',oracle)
record={'schema':'mpk.go_practical.word_conversions_native_comparison.v1','status':'passed',
'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_execution':seal(native_path),
'query_origins':seal(root/'query-origins.json'),'Rust_execution':seal(execution_path),
'certificate':seal(model_dir/'certificate.mpcert'),'actual_Bool_model':seal(model_dir/'model.json'),
'model_metrics':seal(model_dir/'metrics.json'),'independent_integer_oracle':seal(out/'independent-integer-oracle.json'),
'original_queries':256,'actual_source_conversion_results':256,'actual_count_normalizations':256,
'defined_counts':192,'actual_false_signed_count_guards':64,'signed_exact_type_cases':80,'unsigned_exact_type_cases':80,
'prepared_image_files':5577,'two_native_outputs_identical':True,
'Nat_Int_masks_ratios_quotients_or_theorem_bodies_executed':False,
'scope':'All32 operations retain all256 original complete rows. Actual decoded Bool conversion/sign/To64/count guards match actual typed Go conversions and real negative-count panics. Python uses arbitrary-precision modular casts, quotient by integer division and corrections by exact signed differences, independently of the product/mask proof recipe. Exactness flags identify theorem applicability by types, not an iff predicate over finite inputs. Nat/Int/ratio/quotient/mask/theorem bodies and variable shift source bodies do not execute in the Bool model. Both unchanged cores separately check universal conversion/count laws; full variable shifts/source/context/foundation/application/profile binding remains due.',
'CSharp_tests_executed':False,'T02_W01_complete':False,'public_profile_activated':False,
'full_T02_gate':'deferred to W04: check-fast.sh --skip-csharp'}
write(out/'result.json',record)
print(json.dumps({k:record[k] for k in ['status','original_queries','defined_counts','actual_false_signed_count_guards']}))
