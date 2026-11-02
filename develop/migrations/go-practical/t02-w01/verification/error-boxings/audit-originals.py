"""Literal original output audit and independent body-boxing correspondence."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];archives=[];kept={};count=size=0
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;manifest,files=read(name)
 for row in manifest['original_inputs']:
  path=Path(row['path']);assert path.read_bytes()==files[row['member']]and path.stat().st_mode&0o777==row['mode'];count+=1;size+=len(files[row['member']])
 archives.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
 if name=='native-original':kept[name]={k:v for k,v in files.items()if '/boxing' in k}
 if name in {'rust-original','selection-original','frontend-original'}:kept[name]=files
assert len(archives)==14 and count==54753 and size==385437972
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases['cases'])==46
rust=kept['rust-original'];normalize=lambda v:sorted(v,key=lambda r:json.dumps(r,sort_keys=True,separators=(',',':')))
assert json.loads(rust['input.json'])==cases['input'];typed=body=go_typed=0;details=[]
for n,case in enumerate(cases['cases']):
 native=json.loads(kept['native-original'][f'error-boxings/boxing-{n:02}-actual.json']);assert case['native']==native
 actual=json.loads(rust[f'case-{n:02}-actual.json']);assert actual['name']==case['name']and not actual['global_error_universe_complete']and not actual['all_compiler_typed_sites_claimed']
 assert normalize(actual['body'])==normalize(native['body'])
 assert all(site in native['typed']for site in actual['typed'])
 assert actual['body_type_union']==sorted({site['type_id']for site in native['body']})
 typed+=len(actual['typed']);body+=len(actual['body']);go_typed+=len(native['typed']);details.append({'name':case['name'],'checked_value_sites':len(actual['typed']),'compiler_typed_sites':len(native['typed']),'body_sites':len(actual['body']),'body_types':len(actual['body_type_union'])})
assert(typed,go_typed,body)==(128,129,125)
assert [r['name']for r in details if r['checked_value_sites']!=r['compiler_typed_sites']]==['ConstLen']
assert all(row['body_sites']==0 for row in details if row['name']in ['Nil','Rebox','BlankDefault','ConstLen','DirectLen','GlobalLen','GlobalRange','RangeZero','RangeOne','RangeBlank'])
assert next(r for r in details if r['name']=='ManyCaller')['body_sites']==70
assert next(r for r in details if r['name']=='Wide')['body_sites']==16
assert next(r for r in details if r['name']=='RangeAssign')['body_sites']==1
assert next(r for r in details if r['name']=='Branches')['body_types']==2
selected=kept['selection-original'];assert json.loads(selected['current-summary.json'])=={'positive':24,'negative':27,'failures':[]}and json.loads(selected['baseline-summary.json'])=={'positive':15,'negative':18,'failures':[]}
legacy=json.loads((OUT/'isolated-native/go-oracle/result.json').read_bytes());assert legacy['status']=='failed'and legacy['whole_source_inputs_unchanged']
assert not any(name.startswith('TestPracticalSourceClosure')for name in legacy['failed_tests'])
old_passes=[name for name in legacy['passed_tests']if name.startswith('TestPracticalSourceClosure')];assert len(old_passes)==53
assert legacy['failed_tests']==['TestPracticalErrorBoxingsOriginalContexts']
new=json.loads((OUT/'box-native/go-boxings/result.json').read_bytes());assert new['status']=='passed'and len(new['passed_tests'])==47
front=json.loads(kept['frontend-original']['reproduced-builds.json']);assert front['byte_exact_equal']and front['independent_empty_caches']and front['source_inputs']==190
inputs=json.loads(kept['frontend-original']['captured-inputs.json'])['build_inputs'];assert len(inputs)==190
for obj in inputs:
 raw=(Path('go-tools/go2vir')/obj['path']).read_bytes();assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':archives,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'original_cases':46,'checked_value_sites':128,'original_compiler_typed_sites':129,'exact_body_sites':125,'typed_evidence_difference':'ConstLen is delegated to the separate constant reader. Its compile-only site is not emitted by value-engine evidence. All128 emitted sites are original Go typed sites; all125 body sites and every body type union match completely.','cases':details,'prior_original_selection_cases':84,'prior_selection_exact_graphs':39,'prior_selection_rejections':45,'legacy_native_source_closure_passes':old_passes,'legacy_native_run_overall_status':'failed due exclusively to new observer output-directory setup; not a whole-run pass','fresh_native_boxing_passes':47,'empty_cache_frontend_builds':2,'frontend_inputs':190,'frontend_binary_bytes':front['builds'][0]['bytes'],'frontend_binary_sha256':front['builds'][0]['binary_sha256'],'registered_capture_full_error_universe_Root_Plain_or_source_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','exact_body_sites','frontend_binary_sha256']}))
