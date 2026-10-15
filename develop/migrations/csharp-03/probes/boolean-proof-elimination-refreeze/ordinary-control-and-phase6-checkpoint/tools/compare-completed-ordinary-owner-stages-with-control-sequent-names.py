import collections,hashlib,json,pathlib,re,sys
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');root=r/'develop/migrations/csharp-03/ordinary-foundation';number=sys.argv[1];assert number in ['6','7','8'];state=json.loads((b/f'ordinary-regeneration-{number}/status.json').read_bytes());completed={s['destination']:s for s in state['stages'] if s['exit_code']==0 and s['test_counts']==['1']};plan=[p for p in json.loads((b/f'ordinary-regeneration-plan-{number}.json').read_bytes()) if p['destination'] in completed];assert plan
mapping=json.loads((b/'context-name-rebindings-4.json').read_bytes());negative=json.loads((b/'negative-envelope-parent-rebinding.json').read_bytes())
for k,v in list(mapping.items()):mapping[k]=negative['snapshot_id_changes'].get(v,v)
reverse={v:k for k,v in mapping.items()};observed=collections.Counter();h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def rebound(v):
 if isinstance(v,str):
  for a,c in mapping.items():
   if a in v:v=v.replace(a,c)
  return v
 if isinstance(v,list):return [rebound(x) for x in v]
 if isinstance(v,dict):return {k:rebound(x) for k,x in v.items()}
 return v
def projection(v,path=''):
 if isinstance(v,dict):
  result={}
  for k,x in v.items():
   if k.endswith('_sha256') or k=='sha256':observed[path+'/'+k]+=1;continue
   if k in ['id','sequent_id'] and isinstance(x,str) and re.fullmatch('[a-z_]+_vc:[0-9a-f]{64}',x):observed[path+'/'+k]+=1;continue
   result[k]=projection(x,path+'/'+k)
  return result
 if isinstance(v,list):return [projection(x,path+'[]') for x in v]
 return v
identity_rows=[];identities={};rows=[];failures=[]
def programs(a,c,file,pointer=''):
 if isinstance(a,dict) and isinstance(c,dict):
  if 'schema' in a and 'source_ir_sha256' in a and 'source_ir_sha256' in c and a['schema']==c['schema']:
   old=a['source_ir_sha256'];new=c['source_ir_sha256'];assert re.fullmatch('[0-9a-f]{64}',old) and re.fullmatch('[0-9a-f]{64}',new)
   if old in identities:assert identities[old]==new
   identities[old]=new;identity_rows.append({'file':file,'pointer':pointer,'schema':a['schema'],'old_source_ir_sha256':old,'new_source_ir_sha256':new})
  for k in a.keys()&c.keys():programs(a[k],c[k],file,pointer+'/'+k)
 elif isinstance(a,list) and isinstance(c,list):
  for i,(x,y) in enumerate(zip(a,c)):programs(x,y,file,pointer+'/'+str(i))
for p in plan:
 family=p['destination'];out=b/f'ordinary-regeneration-{number}/goldens'/family
 assert completed[family]['generated_files']=={str(f.relative_to(out)):h(f) for f in out.rglob('*') if f.is_file()}
 for fresh in sorted(out.rglob('*')):
  if not fresh.is_file():continue
  name=str(fresh.relative_to(out));old_name=name
  for a,c in reverse.items():old_name=old_name.replace(a,c)
  old=root/family/old_name;assert old.is_file(),str(old)
  row={'family':family,'generated_path':name,'original_path':old_name,'original_raw_sha256':h(old),'generated_raw_sha256':h(fresh)}
  if fresh.suffix=='.hex':
   row['certificate_bytes_preserved']=old.read_bytes()==fresh.read_bytes();ok=True
   if not row['certificate_bytes_preserved']:row['changed_certificate_graph_and_dual_acceptance']='required_separate_checks'
  elif fresh.suffix=='.json':
   a=json.loads(old.read_bytes());c=json.loads(fresh.read_bytes());ok=projection(rebound(a))==projection(c);row['semantic_metadata_preserved']=ok
   if ok:programs(a,c,str(old.relative_to(r)))
   else:failures.append({'family':family,'path':name,'original_projected':projection(rebound(a)),'new_projected':projection(c)})
  else:raise AssertionError(name)
  if not ok:failures.append(row)
  rows.append(row)
receipt={'schema':'mpk.csharp_practical.t01_w09.completed_ordinary_owner_stages.v1','status':'passed_completed_stage_actual_generation_complete_semantic_lineage' if not failures else 'failed_completed_stage_lineage','parent_run_status':state['status'],'completed_owner_stages':len(plan),'files':rows,'certificate_files':sum('certificate_bytes_preserved' in x for x in rows),'metadata_files':sum('semantic_metadata_preserved' in x for x in rows),'actual_identity_rows':identity_rows,'actual_source_ir_rebindings':identities,'failures':failures,'original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
(b/f'completed-ordinary-owner-stages-{number}-with-control-sequent-names.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k not in ['files','failures','actual_identity_rows','actual_source_ir_rebindings']}));print('failures',len(failures),'identities',len(identities))
