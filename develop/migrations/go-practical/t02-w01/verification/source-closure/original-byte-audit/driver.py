"""Literal original-byte/mode audit and native dependency graph correspondence."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];counts=[];total=size=0;native=None
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;manifest,files=read(name)
 for row in manifest['original_inputs']:
  path=Path(row['path']);raw=path.read_bytes();assert raw==files[row['member']]and path.stat().st_mode&0o777==row['mode'];total+=1;size+=len(raw)
 counts.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
 if name=='native-original':native=files
assert native is not None and len(counts)==9
control=json.loads(native['closure-final-controls.json']);assert control['cases']==31 and control['roots']==70 and control['cross_reader_helper_links']==512 and control['parameter_slots']==70 and control['source_ownership']and not control['application_initializers_executed']
def key(ref):return(ref['package'],ref['source'],ref['start'],ref['end'])
positive=negative=calls=edges=issues=0;roles={};constant_shadow_cases=0
for n in range(31):
 before=json.loads(native[f'closure-{n:02}-input.json']);actual=json.loads(native[f'closure-{n:02}-actual.json']);root=key(before['root']);assert[key(r)for r in actual['roots']]==[root]
 declarations={key(d['ref']):d for d in actual['declarations']};assert len(declarations)==len(actual['declarations'])and root in declarations
 dependencies={(key(e['from']),key(e['to']),e['role'])for e in actual['dependencies']};assert len(dependencies)==len(actual['dependencies'])
 for a,b,role in dependencies:assert a in declarations and b in declarations and role in declarations[b]['roles']
 for call in actual['calls']:
  a,b=key(call['caller']),key(call['callee']);assert a in declarations and b in declarations and(a,b,'body_dependency')in dependencies
  assert call['source']==a[1]and a[2]<=call['start']<call['end']<=a[3]
 codes={e['code']for e in actual['issues']};expected=before['expected_issue_codes']or[]
 if expected:assert set(expected)<=codes;negative+=1
 else:assert not codes;positive+=1
 for issue in actual['issues']:
  path=[key(r)for r in issue['path']];assert path and path[0]==root
  if issue['code']=='call_cycle':
   assert path[-1]in path[:-1]
   for a,b in zip(path,path[1:]):assert(a,b,'body_dependency')in dependencies
  else:
   assert path[-1]==key(issue['declaration'])
   for a,b in zip(path,path[1:]):assert any(x==a and y==b for x,y,_ in dependencies)
 sources={s['object']['id']:base64.b64decode(s['raw'],validate=True)for s in actual['sources']}
 for d in actual['declarations']:
  r=key(d['ref']);assert r[1]in sources and 0<=r[2]<r[3]<=len(sources[r[1]])
  for role in d['roles']:roles[role]=roles.get(role,0)+1
 if b'func Context()'in sources[root[1]][root[2]:root[3]]or b'func ContextBlank()'in sources[root[1]][root[2]:root[3]]:
  shadow=[d for d in actual['declarations']if d['kind']=='const'and d['names']==['Shadow']and'constant_dependency'in d['roles']];assert len(shadow)==2;constant_shadow_cases+=1
 assert actual['source_admission_or_execution_or_program_proof']is False
 calls+=len(actual['calls']);edges+=len(dependencies);issues+=len(actual['issues'])
assert(positive,negative,constant_shadow_cases)==(13,18,2)
many=json.loads(native['closure-many-roots.json']);assert len(many['roots'])==70 and len(many['calls'])==70 and not many['issues']
frontend=json.loads(read('frontend-original')[1]['reproduced-builds.json']);assert frontend['source_inputs']==190 and frontend['byte_exact_equal']and frontend['independent_empty_caches']and len(frontend['builds'])==2
assert frontend['builds'][0]['binary_sha256']==frontend['builds'][1]['binary_sha256']=='589e82c3c73b95df654501cc71776d9a1a3095371a0ee11170ac2dff4a816eea'
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':counts,'actual_original_files_compared_literally':total,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'original_selected_cases':31,'positive_metadata_cases':positive,'negative_metadata_cases':negative,'original_call_edges':calls,'original_dependency_edges':edges,'original_issues':issues,'declaration_roles':roles,'contextual_shadow_cases':constant_shadow_cases,'selected_roots_control':70,'parameter_slots_control':70,'flat_original_helper_declarations':512,'actual_native_object_and_physical_call_correspondence_checked':True,'original_initializers_or_application_executed':False,'complete_source_admission_or_contract_closure_or_source_program_proof':False,'current_frontend_inputs':190,'current_frontend_empty_cache_builds':2,'current_frontend_binary_sha256':frontend['builds'][0]['binary_sha256'],'native_Linux_acceptance':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','positive_metadata_cases','negative_metadata_cases','original_call_edges','original_dependency_edges']}))
