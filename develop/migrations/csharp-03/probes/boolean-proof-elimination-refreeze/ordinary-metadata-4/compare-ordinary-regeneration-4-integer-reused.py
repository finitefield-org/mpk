import collections,hashlib,json,re
from pathlib import Path
base=Path(__file__).parent;repo=Path('/private/tmp/mpk-w09-refreeze-metadata-integration');root=repo/'develop/migrations/csharp-03/ordinary-foundation'
plan=json.loads((base/'ordinary-regeneration-plan-4-integer-reused.json').read_bytes());state=json.loads((base/'ordinary-regeneration-4-integer-reused/status.json').read_bytes());assert state['status']=='passed_selected_source_bound_ordinary_owners'
mapping=json.loads((base/'context-name-rebindings.json').read_bytes());negative=json.loads((base/'negative-envelope-parent-rebinding.json').read_bytes())
for k,v in list(mapping.items()):mapping[k]=negative['snapshot_id_changes'].get(v,v)
reverse={v:k for k,v in mapping.items()};observed=collections.Counter();h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def replace_ids(v):
 if isinstance(v,str):
  for a,b in mapping.items():
   if a in v:v=v.replace(a,b)
  return v
 if isinstance(v,list):return [replace_ids(x) for x in v]
 if isinstance(v,dict):return {k:replace_ids(x) for k,x in v.items()}
 return v
def projection(v,path=''):
 if isinstance(v,dict):
  out={}
  for k,x in v.items():
   if k.endswith('_sha256') or k=='sha256':observed[path+'/'+k]+=1;continue
   if k in ['id','sequent_id'] and isinstance(x,str) and re.fullmatch('[a-z_]+_vc:[0-9a-f]{64}',x):observed[path+'/'+k]+=1;continue
   out[k]=projection(x,path+'/'+k)
  return out
 if isinstance(v,list):return [projection(x,path+'[]') for x in v]
 return v
rows=[];fail=[]
for p in plan:
 family=p['destination'];out=base/'ordinary-regeneration-4-integer-reused/goldens'/family
 sourcefiles={}
 for generated in sorted(out.rglob('*')):
  if not generated.is_file():continue
  name=str(generated.relative_to(out));old_name=name
  for a,b in reverse.items():old_name=old_name.replace(a,b)
  old=root/family/old_name;assert old.exists(),str(old)
  row={'family':family,'generated_path':name,'original_path':old_name,'original_raw_sha256':h(old),'generated_raw_sha256':h(generated)}
  if generated.suffix=='.hex':
   row['certificate_bytes_preserved']=old.read_bytes()==generated.read_bytes()
   if not row['certificate_bytes_preserved']:fail.append(row)
  elif generated.suffix=='.json':
   a=projection(replace_ids(json.loads(old.read_bytes())));b=projection(json.loads(generated.read_bytes()));row['semantic_metadata_preserved']=a==b
   if a!=b:fail.append({'family':family,'path':name,'original_projected':a,'new_projected':b})
  else:raise AssertionError(name)
  rows.append(row)
receipt={'schema':'mpk.csharp_practical.t01_w09.ordinary_metadata_regeneration.v1','status':'passed_actual_generation_and_exact_definition_bytes' if not fail else 'failed_lineage_comparison','selection_reason':state['selection_reason'],'producer_tests':len(state['stages']),'files':rows,'certificate_files':sum('certificate_bytes_preserved' in x for x in rows),'metadata_files':sum('semantic_metadata_preserved' in x for x in rows),'changed_metadata_digest_paths':dict(sorted(observed.items())),'failures':fail,'original_application_proof_ids_pending':987,'full_t01_gate':'deferred to final W10','full_t06_gate':'deferred to W12'}
(base/'ordinary-regeneration-comparison-4-integer-reused.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({'status':receipt['status'],'files':len(rows),'certificates':receipt['certificate_files'],'metadata':receipt['metadata_files'],'failures':len(fail)},ensure_ascii=False))
