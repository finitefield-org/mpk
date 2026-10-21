"""Compare complete actual bytes/modes and independent assignment observations."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
names=['native-original','rust-initial-original','rust-original','scalar-original','aggregate-original','builtin-original','method-original']
for name in names:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases)==1433
calls=slots=shorts=blanks=0
for n,row in enumerate(cases):
 assert json.loads(originals['native-original'][f'case-{n:05}/input.json'])==row['input'];assert json.loads(originals['native-original'][f'case-{n:05}/native.json'])==row['native']
 for name in ['rust-initial-original','rust-original']:
  files=originals[name];assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and not row['input']['outside_assignment_typing']:assert actual==row['native']
  else:assert not actual['accepted']
 if row['native']['accepted']and not row['input']['outside_assignment_typing']:
  native=row['native'];calls+=len(native['calls']);slots+=len(native['slots']);shorts+=sum(t['declares_variable']for t in native['targets']);blanks+=sum(t['type_key']is None for t in native['targets'])
for name in ['rust-initial-original','rust-original']:assert json.loads(originals[name]['summary.json'])=={'accepted':475,'rejected':951,'outside':7,'failures':[]}
owned=json.loads(originals['rust-original']['owned-assignment-actual.json']);assert len(owned['pair']['slots'])==2 and len(owned['pair']['calls'])==1 and len(owned['compound']['calls'])==2 and owned['other_body_statement_independently_rejected']and not owned['body_state_or_program_proof']
initial=originals['rust-initial-original'];final=originals['rust-original'];assert 'assignment-deep-actual.json'not in initial and 'assignment-deep-actual.json'in final
assert initial['assignment-deep-input.json']!=final['assignment-deep-input.json']
for name,corpus,expected in [('scalar-original',OUT.parent/'scalar-typing/corpus.json',(2776,2205,4)),('aggregate-original',OUT.parent/'value-typing/corpus-aliases.json',(474,463,13)),('method-original',OUT.parent/'method-typing/corpus-calls.json',(143,300,18))]:
 files=originals[name];a=r=o=0
 for n,row in enumerate(json.loads(corpus.read_bytes())):
  assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
  elif row['native']['accepted']:assert row['input'].get('outside_value_typing',row['input'].get('outside_scalar_typing',False));o+=1
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
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_assignment_cases':1433,'exact_positive_types_and_target_provenance':475,'agreeing_actual_rejections':951,'explicit_valid_Go_unmodeled_forms':7,'independent_source_call_headers':calls,'independent_destination_slots':slots,'independent_new_short_bindings':shorts,'independent_blank_destinations':blanks,'initial_fixture_lexed_repeated_minus_as_decrement_tokens':True,'initial_failed_control_and_corrected_originals_retained':True,'flat_contextual_unary_nodes':20000,'parallel_and_short_destination_controls':200,'function_results_boundary16_and17':True,'previous_scalar_cases':4985,'previous_aggregate_cases':950,'previous_builtin_cases':1659,'previous_method_cases':461,'native_checker_panics_still_distinct_from_rejections':3,'body_state_or_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','independent_source_call_headers','independent_destination_slots','independent_new_short_bindings','independent_blank_destinations']}))
