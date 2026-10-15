import collections,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');root=r/'develop/migrations/csharp-03/ordinary-foundation';plan=json.loads((b/'wrapper-program-plan.json').read_bytes());out=b/'wrapper-program-regeneration';state=json.loads((out/'status.json').read_bytes());assert state['status']=='passed' and state['exit_code']==0 and state['source_input_binary_bytes_unchanged'];generation=json.loads((out/'generated/generation.json').read_bytes());assert generation['status']=='passed_actual_generation_and_strict_import'
actual=json.loads((b/'context-name-rebindings-5.json').read_bytes());actual.update(json.loads((b/'actual-vir-hash-rebindings.json').read_bytes()));negative=json.loads((b/'negative-envelope-parent-rebinding.json').read_bytes())
for k,v in list(actual.items()):actual[k]=negative['snapshot_id_changes'].get(v,v)
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();wanted={occ.get('original_certificate_sha256',occ['original_program'].get('certificate_sha256')) for target in plan['targets'] for occ in target['occurrences']};index=collections.defaultdict(list)
for p in sorted(root.rglob('*.hex')):
 if any(x=='previous' or x.startswith(('previous-','pre-')) for x in p.parts) or 'verification-logs' in p.parts:continue
 raw=bytes.fromhex(p.read_text());h=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
 if h in wanted:index[h].append(str(p.relative_to(r)))
audit_dir=root/'verification-logs/unit-4-count-pin-refresh/additional-consumers'
prior_promotion=json.loads((audit_dir/'promotion.json').read_bytes())
prior_audit=json.loads((audit_dir/'declaration-audit.json').read_bytes())
prior_rows={x['old_certificate_sha256']:x for x in prior_promotion['programs']}
prior_graphs={x['file']:x for x in prior_audit['programs']}
stale_aliases=[]
def differences(a,c,p=''):
 if a==c:return []
 if isinstance(a,dict) and isinstance(c,dict):
  return [q for k in sorted(a.keys()|c.keys()) for q in (differences(a[k],c[k],p+'/'+k) if k in a and k in c else [p+'/'+k])]
 if isinstance(a,list) and isinstance(c,list) and len(a)==len(c):
  return [q for i,(x,y) in enumerate(zip(a,c)) for q in differences(x,y,p+'/'+str(i))]
 return [p]
observed=collections.Counter()
def reb(v):
 if isinstance(v,str):
  for old,new in actual.items():
   if old in v:v=v.replace(old,new)
  return v
 if isinstance(v,list):return [reb(x) for x in v]
 if isinstance(v,dict):return {k:reb(x) for k,x in v.items()}
 return v
def project(v,path=''):
 if isinstance(v,dict):
  result={}
  for k,x in v.items():
   if k.endswith('_sha256') or k=='sha256':observed[path+'/'+k]+=1;continue
   if k in ['id','sequent_id'] and isinstance(x,str) and x.split(':',1)[0].endswith('_vc') and len(x.split(':')[-1])==64:observed[path+'/'+k]+=1;continue
   result[k]=project(x,path+'/'+k)
  return result
 if isinstance(v,list):return [project(x,path+'[]') for x in v]
 return v
rows=[];failures=[]
for target in plan['targets']:
 label=target['label'];meta=out/'generated'/f'{label}.program.json';cert=out/'generated'/f'{label}.hex';fresh=json.loads(meta.read_bytes());raw=bytes.fromhex(cert.read_text());new_hash=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();assert fresh.get('certificate_sha256',new_hash)==new_hash
 row={'label':label,'schema':target['schema'],'old_source_ir_sha256':target['old_source_ir_sha256'],'source_ir_sha256':target['source']['new_source_ir_sha256'],'new_metadata_raw_sha256':digest(meta),'new_certificate_raw_sha256':digest(cert),'occurrences':[]}
 for occ in target['occurrences']:
  old=occ['original_program'];old_hash=occ.get('original_certificate_sha256',old.get('certificate_sha256'));paths=index[old_hash];provenance=None
  if not paths:
   assert old_hash in prior_rows,(label,old_hash)
   prior=prior_rows[old_hash];graph=prior_graphs[prior['file']]
   assert prior['audit_status']==graph['status']=='known_count_helper_only'
   assert graph['declarations']['added']==graph['declarations']['changed_types']==[]
   assert len(graph['declarations']['changed_bodies'])==1 and 'DomainCount.' in graph['declarations']['changed_bodies'][0]
   assert graph['old_hex_sha256']==prior['old_hex_sha256'] and graph['new_hex_sha256']==prior['hex_sha256']
   paths=index[prior['certificate_sha256']];assert paths
   assert all(digest(r/p)==prior['hex_sha256'] for p in paths)
   canonical=[v for v in target['occurrences'] if v.get('original_certificate_sha256')==prior['certificate_sha256']]
   assert canonical and all(v['original_program']==canonical[0]['original_program'] for v in canonical)
   current=canonical[0]['original_program'];diffs=differences(old,current)
   assert set(diffs)==set(graph['metadata_changed_paths']),(label,diffs,graph['metadata_changed_paths'])
   assert all(p.endswith(('/certificate_sha256','/static_transformers')) for p in diffs)
   provenance={'file':occ['file'],'pointer':occ['pointer'],'stale_certificate_sha256':old_hash,'already_promoted_certificate_sha256':prior['certificate_sha256'],'prior_promotion_file':str((audit_dir/'promotion.json').relative_to(r)),'prior_declaration_audit_file':str((audit_dir/'declaration-audit.json').relative_to(r)),'prior_certificate_file':prior['file'],'exact_stale_metadata_paths':diffs,'prior_aggregate_record':{k:v for k,v in occ.items() if k!='original_program'}}
   stale_aliases.append(provenance);old=current;old_hash=prior['certificate_sha256']
  retained=old_hash==new_hash;comparison=project(reb(old))==project(fresh);item={k:v for k,v in occ.items() if k!='original_program'};item.update(original_certificate_files=paths,certificate_bytes_preserved=retained,semantic_metadata_preserved=comparison)
  if provenance:item['stale_aggregate_count_refresh_provenance']=provenance
  if paths and retained:assert all(bytes.fromhex((r/p).read_text())==raw for p in paths)
  if not comparison:failures.append({'label':label,'file':occ['file'],'pointer':occ['pointer'],'old_projected':project(reb(old)),'new_projected':project(fresh)})
  if not paths:failures.append({'label':label,'file':occ['file'],'pointer':occ['pointer'],'reason':'original_certificate_bytes_not_found'})
  if not retained:item['changed_certificate_graph_and_dual_acceptance']='required_separate_checks'
  row['occurrences'].append(item)
 rows.append(row)
receipt={'schema':'mpk.csharp_practical.t01_w09.actual_program_alias_regeneration_receipt.v1','status':'passed_actual_generation_strict_import_complete_semantic_lineage_and_reviewed_stale_alias_reconciliation' if not failures else 'failed_lineage_comparison','selection_reason':plan['selection_reason'],'actual_programs':len(rows),'program_occurrences':sum(len(x['occurrences']) for x in rows),'original_certificate_files_checked':sum(len(v) for v in index.values()),'programs':rows,'stale_aggregate_aliases_reconciled_from_existing_reviewed_count_refresh':stale_aliases,'digest_paths':dict(observed),'failures':failures,'original_application_proof_ids_pending':987,'full_t01_gate':plan['full_t01_gate'],'full_t06_gate':plan['full_t06_gate']};(b/'wrapper-program-comparison-3.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k not in ['programs','failures','digest_paths','selection_reason']},ensure_ascii=False));print('failures',len(failures))
