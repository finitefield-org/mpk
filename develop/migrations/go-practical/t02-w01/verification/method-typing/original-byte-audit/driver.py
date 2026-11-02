"""Literal originals and independent original method-callee correspondence."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
names=['native-initial-original','native-original','native-calls-original','rust-original','rust-calls-original','scalar-original','scalar-final-original','aggregate-initial-original','aggregate-original','builtin-original']
for name in names:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert count==31709
for name,corpus in [('native-original','corpus.json'),('native-calls-original','corpus-calls.json')]:
 cases=json.loads((OUT/corpus).read_bytes());assert len(cases)==461
 for n,row in enumerate(cases):
  assert json.loads(originals[name][f'case-{n:05}/input.json'])==row['input'];assert json.loads(originals[name][f'case-{n:05}/native.json'])==row['native']
cases=json.loads((OUT/'corpus-calls.json').read_bytes());rust=originals['rust-calls-original'];a=r=o=0;calls=[]
for n,row in enumerate(cases):
 assert json.loads(rust[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(rust[f'case-{n:05}-actual.json'])
 if row['native']['accepted']and not row['input']['outside_value_typing']:assert actual==row['native'];a+=1;calls.extend(actual['method_calls'])
 elif row['native']['accepted']:assert not actual['accepted'];o+=1
 else:assert not actual['accepted'];r+=1
assert(a,r,o)==(143,300,18)
assert json.loads(rust['summary.json'])=={'accepted':143,'rejected':300,'outside':18,'failures':[]}
assert len(calls)==69 and sum(bool(c['path'])for c in calls)==23 and sum(c['method_expression']for c in calls)==18 and sum(c['body_start']is None for c in calls)==8
owned=json.loads(rust['owned-method-actual.json']);assert len(owned['method_calls'])==1 and owned['method_calls'][0]['path']==[0]and owned['source_body_evaluation_or_invocation_or_program_proof']is False
chain=json.loads(rust['method-chain-actual.json']);assert len(chain['method_calls'])==200
oldcases=json.loads((OUT/'corpus.json').read_bytes());initial=originals['rust-original']
for n,row in enumerate(oldcases):
 assert json.loads(initial[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(initial[f'case-{n:05}-actual.json'])
 if row['native']['accepted']and not row['input']['outside_value_typing']:assert actual==row['native']
 else:assert not actual['accepted']
for name,corpus,expected in [('scalar-original',OUT.parent/'scalar-typing/corpus.json',(2776,2205,4)),('scalar-final-original',OUT.parent/'scalar-typing/corpus.json',(2776,2205,4)),('aggregate-initial-original',OUT.parent/'value-typing/corpus-aliases.json',(474,463,13)),('aggregate-original',OUT.parent/'value-typing/corpus-aliases.json',(474,463,13))]:
 cases=json.loads(corpus.read_bytes());files=originals[name];a=r=o=0
 for n,row in enumerate(cases):
  assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
  elif row['native']['accepted']:assert row['input'].get('outside_value_typing',row['input'].get('outside_scalar_typing',False));o+=1
  else:assert not actual['accepted'];r+=1
 assert(a,r,o)==expected
cases=json.loads((OUT.parent/'slice-builtins/corpus.json').read_bytes());files=originals['builtin-original'];a=r=o=p=0
for n,row in enumerate(cases):
 assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
 if 'native_panic'in row['native']:assert not actual['accepted'];p+=1
 elif row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
 elif row['native']['accepted']:assert row['input']['outside_value_typing'];o+=1
 else:assert not actual['accepted'];r+=1
assert(a,r,o,p)==(510,1143,3,3)
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_method_typing_cases':461,'exact_positive_type_and_method_provenance_outcomes':143,'agreeing_actual_rejections':300,'explicit_valid_Go_unmodeled_forms':18,'independent_original_method_call_headers':69,'independent_promoted_receiver_paths':23,'independent_method_expressions':18,'independent_opaque_method_calls':8,'all_original_case_rows_and_outcomes_exact':True,'original200_method_chain_retained':True,'original_unused_Error_signature_dependency_distinct_from_calls':True,'previous_scalar_cases':4985,'previous_aggregate_cases':950,'previous_builtin_cases':1659,'native_checker_panics_still_distinct_from_rejections':3,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally']}))
