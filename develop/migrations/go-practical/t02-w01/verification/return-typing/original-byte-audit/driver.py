"""Compare literal originals and independently observed return/signature facts."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
names=['native-original','native-import-original','rust-initial-original','rust-original','scalar-original','aggregate-original','builtin-original','method-original','assignment-original']
for name in names:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
old=json.loads((OUT/'corpus.json').read_bytes());cases=json.loads((OUT/'corpus-import.json').read_bytes());assert len(old)==len(cases)==630
initial=originals['rust-initial-original'];current=originals['rust-original'];counts={'supported':0,'rejected':0,'outside':0,'signature_slots':0,'result_slots':0,'named_results':0,'implicit_returns':0,'nested_function_returns':0,'method_returns':0,'calls':0,'opaque_calls':0,'max_parameters':0}
for n,(previous,row)in enumerate(zip(old,cases)):
 for name,entry in [('native-original',previous),('native-import-original',row)]:
  assert json.loads(originals[name][f'case-{n:05}/input.json'])==entry['input'];assert json.loads(originals[name][f'case-{n:05}/native.json'])==entry['native']
 assert previous['input']['label']==row['input']['label']and previous['input']['outside_return_typing']==row['input']['outside_return_typing']
 assert json.loads(initial[f'case-{n:05}-input.json'])==previous['input'];initial_actual=json.loads(initial[f'case-{n:05}-actual.json'])
 if previous['native']['accepted']and not previous['input']['outside_return_typing']:assert initial_actual==previous['native']
 elif n==583:
  assert previous['input']['label']=='foreign/error/nil'and not previous['native']['accepted']and initial_actual['accepted']and 'not used'in previous['native']['error']and not initial_actual['body_state_or_program_proof']
 else:assert not initial_actual['accepted']
 if n==583:assert not previous['native']['accepted']and row['native']['accepted']
 else:assert previous['native']['accepted']==row['native']['accepted']
 assert json.loads(current[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(current[f'case-{n:05}-actual.json'])
 if row['native']['accepted']and not row['input']['outside_return_typing']:
  native=row['native'];assert actual==native;counts['supported']+=1
  signature=native['function'];slots=signature['slots'];counts['signature_slots']+=len(slots);counts['result_slots']+=len(native['values']);counts['named_results']+=sum(b is not None for b in native['bindings']);counts['implicit_returns']+=native['implicit_results'];counts['nested_function_returns']+=signature['kind']=='FuncLit';counts['method_returns']+=any(s['role']=='receiver'for s in slots);counts['calls']+=len(native['calls']);counts['opaque_calls']+=sum(c['body_start']is None for c in native['calls']);counts['max_parameters']=max(counts['max_parameters'],sum(s['role']=='parameter'for s in slots))
 elif row['native']['accepted']:assert not actual['accepted'];counts['outside']+=1
 else:assert not actual['accepted'];counts['rejected']+=1
assert(counts['supported'],counts['rejected'],counts['outside'])==(150,470,10)and counts['max_parameters']==70
s=json.loads(initial['summary.json']);assert (s['accepted'],s['rejected'],s['outside'])==(149,471,10)and len(s['failures'])==1 and s['failures'][0].startswith('case 583 ')
assert json.loads(current['summary.json'])=={'accepted':150,'rejected':470,'outside':10,'failures':[]}
owned=json.loads(current['owned-return-actual.json']);assert owned['inner']['function']['kind']=='FuncLit'and owned['implicit']['implicit_results']and owned['shadowed_naked_rejected']and owned['unrelated_assignment_not_typechecked']and len(owned['joint']['calls'])==1 and len(owned['joint']['values'])==2 and not owned['body_state_or_program_proof']
assert 'return-deep-actual.json'in current and 'return-deep-actual.json'in initial
assert not list(Path('/private/tmp/mpk-go-return-typing-formatted-native').iterdir())
for name,corpus,expected in [('scalar-original',OUT.parent/'scalar-typing/corpus.json',(2776,2205,4)),('aggregate-original',OUT.parent/'value-typing/corpus-aliases.json',(474,463,13)),('method-original',OUT.parent/'method-typing/corpus-calls.json',(143,300,18)),('assignment-original',OUT.parent/'assignment-typing/corpus.json',(475,951,7))]:
 files=originals[name];a=r=o=0
 for n,row in enumerate(json.loads(corpus.read_bytes())):
  assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
  elif row['native']['accepted']:assert row['input'].get('outside_assignment_typing',row['input'].get('outside_value_typing',row['input'].get('outside_scalar_typing',False)));o+=1
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
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_return_cases':630,'independent_observations':counts,'first_reference_build_failed_before_any_native_cases':True,'initial_unused_import_rejection_not_return_type_rejection':{'index':583,'label':'foreign/error/nil','original_native_accepted':False,'original_partial_return_typed':True,'original_program_authority':False,'corrected_original_source_references_import_by_type':True,'original_inputs_outputs_and_failure_retained':True},'original_static_and_nested_function_ownership_retained':True,'flat_contextual_unary_nodes':20000,'function_results_boundary16_and17':True,'previous_scalar_cases':4985,'previous_aggregate_cases':950,'previous_builtin_cases':1659,'previous_method_cases':461,'previous_assignment_cases':1433,'native_checker_panics_still_distinct_from_rejections':3,'body_state_or_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','independent_observations']}))
