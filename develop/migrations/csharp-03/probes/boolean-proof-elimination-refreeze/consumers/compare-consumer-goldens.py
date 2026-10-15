import collections,hashlib,json,re
from pathlib import Path
base=Path(__file__).parent
paths=['construction-vc/goldens.json','data-vc/goldens.json','control-vc/goldens.json','exception-vc/goldens.json','binding-vc/goldens.json','boundary-vc/goldens.json','boundary-vc/run-goldens.json','transition-vc/goldens.json','boundary-input/conformance.json','boundary-output/conformance.json']
snapshot_map=json.loads((base/'snapshot-id-map.json').read_bytes())
negative=json.loads((base/'negative-envelope-parent-rebinding.json').read_bytes())
for k,v in list(snapshot_map.items()):snapshot_map[k]=negative['snapshot_id_changes'].get(v,v)
observed=collections.Counter()
context_hashes={'registry_sha256','profile_entry_sha256','content_sha256'}
def projection(v,key='',parent=''):
 if isinstance(v,dict):
  result={}
  for k,x in v.items():
   if k.endswith('_sha256') or k=='sha256':
    observed[parent+'/'+k]+=1
    continue
   if k=='revision' and v.get('schema')=='mpk.semantic_profile.registry.v2':
    observed[parent+'/'+k]+=1
    continue
   if k=='capture_id':
    assert isinstance(x,str) and re.fullmatch('[0-9a-f]{64}',x)
    observed[parent+'/'+k]+=1
    continue
   if k=='id' and isinstance(x,str) and x.startswith(('construction_vc:','data_vc:','control_vc:','exception_vc:','binding_vc:','boundary_vc:','transition_vc:')):
    assert re.fullmatch('[a-z_]+_vc:[0-9a-f]{64}',x)
    observed[parent+'/'+k]+=1
    continue
   result[k]=projection(x,k,parent+'/'+k)
  return result
 if isinstance(v,list):
  p=[projection(x,key,parent+'[]') for x in v]
  if key in ('','sequents','assumptions','goals'):p.sort(key=lambda x:json.dumps(x,sort_keys=True,separators=(',',':')))
  return p
 if isinstance(v,str):
  if v in snapshot_map:return snapshot_map[v]
  if v.startswith('{'):
   try:return projection(json.loads(v),key,parent+'(json)')
   except json.JSONDecodeError:pass
 return v
rows=[];differences=[]
def diffs(a,b,path,out):
 if type(a)!=type(b):out.append([path,a,b]);return
 if isinstance(a,dict):
  if a.keys()!=b.keys():out.append([path+'/keys',list(a),list(b)]);return
  for k in a:diffs(a[k],b[k],path+'/'+k,out)
 elif isinstance(a,list):
  if len(a)!=len(b):out.append([path+'/length',len(a),len(b)]);return
  for i,(x,y) in enumerate(zip(a,b)):diffs(x,y,path+'/'+str(i),out)
 elif a!=b:out.append([path,a,b])
for path in paths:
 owner='consumer-regeneration-2' if path in paths[6:] else 'consumer-regeneration'
 old=base/'goldens-before'/path;new=base/owner/'goldens'/path
 a=json.loads(old.read_bytes());b=json.loads(new.read_bytes())
 p=projection(a);q=projection(b);d=[];diffs(p,q,path,d)
 rows.append({'path':path,'rows':len(a),'original_raw_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'regenerated_raw_sha256':hashlib.sha256(new.read_bytes()).hexdigest(),'semantic_projection_equal':not d,'differences':d[:12],'difference_count':len(d)})
 differences.extend(d)
receipt={'schema':'mpk.csharp_practical.t01_w09.consumer_regeneration.v1','status':'passed_semantic_projection' if not differences else 'failed_projection','selection_reason':'Compare the ten actually regenerated context-bound VC and boundary owner corpora; retain source clauses, argument/value data, terms, operations, checks, route/edge roles, definition names and sequent counts. Ignore only declared digest fields, actual snapshot-ID rebinding and generated VC digest IDs; compare hash-sorted sequent sets as multisets. All normal owner tests must separately pin the final corpora.','observed_digest_paths':dict(sorted(observed.items())),'corpora':rows,'difference_count':len(differences)}
(base/'consumer-semantic-comparison.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n')
print(json.dumps({'status':receipt['status'],'corpora':len(rows),'differences':len(differences),'examples':[x for x in rows if x['difference_count']][:3]},ensure_ascii=False))
