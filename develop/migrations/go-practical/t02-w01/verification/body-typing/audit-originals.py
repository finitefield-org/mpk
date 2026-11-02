"""Audit exact original buffers/modes and compiler-derived body correspondence."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
names=['native-original','rust-original','scalar-original','aggregate-original','builtin-original','method-original','assignment-original','return-original','constant-original','source-type-original']
for name in names:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert(count,size)==(32576,92971348)
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases)==495;files=originals['rust-original'];counts={'supported':0,'rejected':0,'outside':0,'signature_slots':0,'statements':0,'expressions':0,'branches':0,'const_specs':0,'const_members':0,'locals':0,'syntactically_terminating':0,'maximum_statements':0}
for n,row in enumerate(cases):
 assert json.loads(originals['native-original'][f'case-{n:05}/input.json'])==row['input'];assert json.loads(originals['native-original'][f'case-{n:05}/native.json'])==row['native']
 assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
 if row['native']['accepted']and not row['input']['outside_body_typing']:
  assert actual==row['native'];counts['supported']+=1
  for key in ['statements','expressions','branches','locals']:counts[key]+=len(actual[key])
  counts['signature_slots']+=len(actual['function']['slots']);counts['const_specs']+=len(actual['constants']);counts['const_members']+=sum(len(c['values'])for c in actual['constants']);counts['syntactically_terminating']+=actual['terminating'];counts['maximum_statements']=max(counts['maximum_statements'],len(actual['statements']))
 elif row['native']['accepted']:assert not actual['accepted'];counts['outside']+=1
 else:assert not actual['accepted'];counts['rejected']+=1
assert counts=={'supported':174,'rejected':299,'outside':22,'signature_slots':1665,'statements':1972,'expressions':442,'branches':113,'const_specs':14,'const_members':16,'locals':57,'syntactically_terminating':28,'maximum_statements':1001}
assert json.loads(files['summary.json'])=={'accepted':174,'rejected':299,'outside':22,'failures':[]}
owned=json.loads(files['owned-body-actual.json']);assert len(owned['probe']['branches'])==2 and len(owned['probe']['locals'])==3 and owned['unsupported_callee_body_rejected_separately']and not owned['callee_closure_or_evaluation_state_program_authority']
assert 'body-deep-actual.json'in files and json.loads(files['blank-constant-spec-actual.json'])['values']==[['0'],['1'],['2'],['3','3']]
for name,corpus,expected,flag in [('scalar-original','scalar-typing/corpus.json',(2776,2205,4),'outside_scalar_typing'),('aggregate-original','value-typing/corpus-aliases.json',(474,463,13),'outside_value_typing'),('method-original','method-typing/corpus-calls.json',(143,300,18),'outside_value_typing'),('assignment-original','assignment-typing/corpus.json',(475,951,7),'outside_assignment_typing'),('return-original','return-typing/corpus-import.json',(150,470,10),'outside_return_typing'),('constant-original','source-constants/corpus.json',(4832,281,4),'outside_scalar_reader')]:
 files=originals[name];a=r=o=0
 for n,row in enumerate(json.loads((OUT.parent/corpus).read_bytes())):
  assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
  elif row['native']['accepted']:assert row['input'].get(flag,row['input'].get('outside_value_typing',False));o+=1
  else:assert not actual['accepted'];r+=1
 assert(a,r,o)==expected
files=originals['builtin-original'];a=r=o=p=0
for n,row in enumerate(json.loads((OUT.parent/'slice-builtins/corpus.json').read_bytes())):
 assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
 if 'native_panic'in row['native']:assert not actual['accepted'];p+=1
 elif row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
 elif row['native']['accepted']:assert row['input']['outside_value_typing'];o+=1
 else:assert not actual['accepted'];r+=1
assert(a,r,o,p)==(510,1143,3,3)
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_body_cases':495,'independent_observations':counts,'flat_contextual_unary_nodes':20000,'source_statement_width_control':1001,'parameter_control':70,'blank_repeated_constant_values_exact':True,'unreachable_unused_missing_return_and_branch_error_origins_retained':True,'typed_caller_is_not_selected_callee_closure':True,'known_v1_aggregate_array_constant_len_cap_not_yet_supported':True,'previous_scalar_cases':4985,'previous_aggregate_cases':950,'previous_builtin_cases':1659,'previous_method_cases':461,'previous_assignment_cases':1433,'previous_return_cases':630,'previous_constant_cases':5117,'old_source_type_roots':4,'native_checker_panics_still_distinct_from_rejections':3,'evaluation_state_or_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','independent_observations']}))
