import hashlib,json
from pathlib import Path
b=Path(__file__).parent;r=Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');f=r/'develop/migrations/csharp-03/ordinary-foundation'
read=lambda p:json.loads(p.read_bytes());h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for name,prefix in [('wrapper-consumer-promotion.json','passed_promoted_161_'),('pattern-consumer-promotion.json','passed_promoted_54_')]:
 receipt=read(b/name);assert receipt['status'].startswith(prefix)
 for row in receipt['changes']:assert h(r/row['path'])==row['raw_sha256'],row['path']
old_foundation='230c708601f4b89feeae28af23da10ccb11eec998d990b5133cf9336369e15a2';new_foundation='99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844'
identity=read(b/'vir-hash-replay-audit.json');request_map={}
for row in identity['identities']:
 old,new=row['old_id'],row['new_id']
 if old!=new:
  assert old not in request_map or request_map[old]==new
  request_map[old]=new
assert len(request_map)==38
old_fields=[];historical=[];new_fields=[];old_requests=[];prior_references=[];files={};excluded=[]
def scan(node,pointer,path,historical_conditions=False,prior=False):
 if isinstance(node,dict):
  if 'foundation_sha256' in node:
   value=node['foundation_sha256'];row={'path':path,'pointer':pointer+'/foundation_sha256','value':value}
   if value==old_foundation:
    (historical if historical_conditions else prior_references if prior else old_fields).append(row)
   elif value==new_foundation:new_fields.append(row)
  for k,v in node.items():
   here=pointer+'/'+str(k).replace('~','~0').replace('/','~1');previous=prior or k.startswith(('prior_','previous_','before_'))
   if k in ['source_snapshot_sha256','source_request_id'] and isinstance(v,str) and v in request_map:
    row={'path':path,'pointer':here,'old':v,'current':request_map[v]}
    (prior_references if previous or historical_conditions else old_requests).append(row)
   scan(v,here,path,historical_conditions,previous)
 elif isinstance(node,list):
  for i,v in enumerate(node):scan(v,pointer+'/'+str(i),path,historical_conditions,prior)
for p in sorted(f.rglob('*.json')):
 parts=p.relative_to(f).parts;rel=p.relative_to(r).as_posix()
 if 'verification-logs' in parts or any(x=='previous' or x.startswith(('previous-','before-','pre-')) for x in parts):excluded.append(rel);continue
 files[rel]=h(p);scan(read(p),'',rel,'with-pattern-observations' in parts)
assert not old_fields,old_fields
assert not old_requests,old_requests
assert len(historical)==18,len(historical)
metrics=read(f/'relations/metrics.json');assert len(metrics)==21 and all(x['metadata']['foundation_sha256']==new_foundation for x in metrics)
receipt={'status':'passed_actual_promoted_ordinary_inventory_without_candidate_overlays','selection_reason':'Audit only the actual current ordinary JSON fixture bytes after both completed promotions. Direct foundation and request identity fields must contain no predecessor identities; complete predecessor/verification archives, historical61-condition documents and explicit prior references retain their actual bytes. No candidate files are overlaid.','candidate_overlay_files':0,'actual_current_files':files,'current_old_foundation_fields':old_fields,'current_old_request_fields':old_requests,'distinct_changed_request_ids':len(request_map),'current_new_foundation_fields':len(new_fields),'retained_historical_condition_documents':historical,'retained_explicit_prior_references':prior_references,'excluded_immutable_evidence_and_predecessors':excluded,'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
(b/'final-actual-ordinary-identity-inventory.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n')
print('PASS actual ordinary identity inventory:',len(files),'files;',len(new_fields),'new foundation fields; zero candidate overlays')
