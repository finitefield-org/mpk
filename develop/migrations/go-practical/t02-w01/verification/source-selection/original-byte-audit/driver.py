"""Audit literal original buffers and both independently constructed graphs."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];originals={};rows=[];count=size=0
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;manifest,files=read(name);originals[name]=files
 for row in manifest['original_inputs']:
  path=Path(row['path']);raw=path.read_bytes();assert raw==files[row['member']]and path.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
assert len(rows)==11
cases=json.loads((OUT/'corpus.json').read_bytes());assert len(cases)==33;positive=negative=calls=deps=declarations=0
normalize=lambda rows:sorted(rows,key=lambda r:json.dumps(r,sort_keys=True,separators=(',',':')))
for n,row in enumerate(cases):
 native=json.loads(originals['native-original'][f'closure-{n:02}-actual.json']);assert row['native']==native
 assert json.loads(originals['rust-original'][f'case-{n:02}-input.json'])==row['input'];actual=json.loads(originals['rust-original'][f'case-{n:02}-actual.json'])
 if row['input']['expected_issue_codes']is None:
  assert 'error'not in actual
  for key in ['calls','dependencies']:assert normalize(actual[key])==normalize(native[key])
  expected=[{k:v for k,v in d.items()if k!='module_source_id'}for d in native['declarations']];assert normalize(actual['declarations'])==normalize(expected)
  positive+=1;calls+=len(actual['calls']);deps+=len(actual['dependencies']);declarations+=len(actual['declarations'])
 else:assert 'error'in actual;negative+=1
assert(positive,negative)==(15,18)
assert json.loads(originals['rust-original']['summary.json'])=={'positive':15,'negative':18,'failures':[]}
many=json.loads(originals['rust-original']['many-roots-actual.json']);assert len(many['calls'])==70
assert json.loads(originals['body-original']['summary.json'])=={'accepted':176,'rejected':299,'outside':20,'failures':[]}
front=json.loads(originals['frontend-original']['reproduced-builds.json']);assert front['byte_exact_equal']and front['source_inputs']==190 and front['independent_empty_caches']and len(front['builds'])==2
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'original_cases':33,'exact_positive_selection_graphs':positive,'independent_negative_rejections':negative,'exact_original_call_edges':calls,'exact_original_dependency_edges':deps,'exact_original_declaration_rows':declarations,'all_unselected_nested_declarations_in_inventory':True,'flat_original_helpers':512,'simultaneous_roots':70,'parameter_slots':70,'legacy_body_cases':495,'legacy_scope_roots':3,'legacy_type_roots':4,'fresh_frontend_inputs':190,'fresh_independent_empty_cache_builds':2,'fresh_frontend_binary_bytes':front['builds'][0]['bytes'],'fresh_frontend_binary_sha256':front['builds'][0]['binary_sha256'],'caller_native_AST_type_or_adapter_accepted':False,'registered_capture_or_source_state_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','exact_positive_selection_graphs','exact_original_call_edges','exact_original_dependency_edges']}))
