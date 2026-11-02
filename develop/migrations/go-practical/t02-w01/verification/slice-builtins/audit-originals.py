"""Compare original transports literally; native crashes are not rejections."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};count=size=0;rows=[]
names=['native-unrecovered-partial','native-original','rust-copy-partial','rust-original','scalar-original','aggregate-original']
for name in names:
 manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases)==1659;native=originals['native-original'];rust=originals['rust-original'];accepted=rejected=outside=panicked=0
for n,row in enumerate(cases):
 assert json.loads(native[f'case-{n:05}/input.json'])==row['input'];assert json.loads(native[f'case-{n:05}/native.json'])==row['native'];assert json.loads(rust[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(rust[f'case-{n:05}-actual.json'])
 if 'native_panic'in row['native']:assert not actual['accepted']and row['native']['native_panic']['stack'];panicked+=1
 elif row['input']['outside_value_typing']and row['native']['accepted']:assert not actual['accepted'];outside+=1
 elif row['native']['accepted']:assert actual==row['native'];accepted+=1
 else:assert not actual['accepted'];rejected+=1
assert(accepted,rejected,outside,panicked)==(510,1143,3,3)
assert json.loads(rust['summary.json'])=={'accepted':510,'rejected':1143,'outside':3,'native_panics':3,'failures':[]}
headers=json.loads(rust['owned-headers-actual.json']);assert len(headers['headers'])==3 and headers['allocation_access_effect_or_body_authority']is False
partial=originals['native-unrecovered-partial'];assert 'case-01147/input.json'in partial and 'case-01147/native.json'not in partial and 'case-01148/input.json'not in partial
partial=originals['rust-copy-partial'];assert len(json.loads(partial['summary.json'])['failures'])==2
for name,corpus,expected in [('scalar-original',OUT.parent/'scalar-typing/corpus.json',(2776,2205,4)),('aggregate-original',OUT.parent/'value-typing/corpus-aliases.json',(474,463,13))]:
 old=json.loads(corpus.read_bytes());files=originals[name];a=r=o=0
 for n,row in enumerate(old):
  assert json.loads(files[f'case-{n:05}-input.json'])==row['input'];actual=json.loads(files[f'case-{n:05}-actual.json'])
  if row['native']['accepted']and actual['accepted']:assert actual==row['native'];a+=1
  elif row['native']['accepted']:assert row['input'].get('outside_value_typing',row['input'].get('outside_scalar_typing',False));o+=1
  else:assert not actual['accepted'];r+=1
 assert(a,r,o)==expected
compiler=json.loads((OUT/'compiler-controls/result.json').read_bytes());assert compiler['status']=='observed'and not compiler['applications_executed']
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
for row in [compiler['corpus'],compiler['actual_driver_prelaunch']]:assert seal(row['path'])==row
internal_errors=0
for control in compiler['original_controls']:
 for key in ['actual_driver_prelaunch','original_input','original_source','compiler_source','stdout','stderr']:assert seal(control[key]['path'])==control[key]
 assert control['source_bytes_unchanged']and not control['applications_executed']
 if control['label']=='edge-append(PairBytesString()...)':assert control['status']=='accepted'and control['exit_code']==0
 else:
  assert control['status']=='not_accepted'and control['exit_code']!=0
  assert b'internal compiler error'in Path(control['stdout']['path']).read_bytes()+Path(control['stderr']['path']).read_bytes();internal_errors+=1
assert internal_errors==3
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'actual_slice_builtin_cases':1659,'exact_positive_type_value_addressability_and_nil_context_outcomes':510,'agreeing_actual_rejections':1143,'explicit_valid_Go_unmodeled_forms':3,'actual_native_checker_panics_not_rejections':3,'compiler_observations':4,'compiler_internal_errors_not_language_rejections':3,'previous_scalar_cases':4985,'previous_aggregate_cases':950,'all_original_case_rows_and_outcomes_exact':True,'original_two_copy_context_differences_preserved':True,'original_unrecovered_native_crash_and_missing_outputs_preserved':True,'partial_missing_outputs_not_reconstructed':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally']}))
