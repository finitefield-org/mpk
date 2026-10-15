import collections,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');root=r/'develop/migrations/csharp-03/ordinary-foundation';plan=json.loads((b/'program-alias-plan.json').read_bytes());out=b/'program-alias-regeneration';state=json.loads((out/'status.json').read_bytes());assert state['status']=='passed' and state['exit_code']==0 and state['source_input_binary_bytes_unchanged'];generation=json.loads((out/'generated/generation.json').read_bytes());assert generation['status']=='passed_actual_generation_and_strict_import'
actual=json.loads((b/'context-name-rebindings-3.json').read_bytes());actual.update(json.loads((b/'actual-vir-hash-rebindings.json').read_bytes()));negative=json.loads((b/'negative-envelope-parent-rebinding.json').read_bytes())
for k,v in list(actual.items()):actual[k]=negative['snapshot_id_changes'].get(v,v)
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();wanted={occ['original_program']['certificate_sha256'] for target in plan['targets'] for occ in target['occurrences']};index=collections.defaultdict(list)
for p in sorted(root.rglob('*.hex')):
 if any(x=='previous' or x.startswith(('previous-','pre-')) for x in p.parts) or 'verification-logs' in p.parts:continue
 raw=bytes.fromhex(p.read_text());h=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
 if h in wanted:index[h].append(str(p.relative_to(r)))
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
 label=target['label'];meta=out/'generated'/f'{label}.program.json';cert=out/'generated'/f'{label}.hex';fresh=json.loads(meta.read_bytes());raw=bytes.fromhex(cert.read_text());new_hash=hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest();assert fresh['certificate_sha256']==new_hash
 row={'label':label,'schema':target['schema'],'old_source_ir_sha256':target['old_source_ir_sha256'],'source_ir_sha256':target['source']['new_source_ir_sha256'],'new_metadata_raw_sha256':digest(meta),'new_certificate_raw_sha256':digest(cert),'occurrences':[]}
 for occ in target['occurrences']:
  if occ['pointer'].endswith('/parsers'):continue
  old=occ['original_program'];paths=index[old['certificate_sha256']];retained=old['certificate_sha256']==new_hash;comparison=project(reb(old))==project(fresh);item={k:v for k,v in occ.items() if k!='original_program'};item.update(original_certificate_files=paths,certificate_bytes_preserved=retained,semantic_metadata_preserved=comparison)
  if paths and retained:assert all(bytes.fromhex((r/p).read_text())==raw for p in paths)
  if not comparison:failures.append({'label':label,'file':occ['file'],'pointer':occ['pointer'],'old_projected':project(reb(old)),'new_projected':project(fresh)})
  if not paths:failures.append({'label':label,'file':occ['file'],'pointer':occ['pointer'],'reason':'original_certificate_bytes_not_found'})
  if not retained:item['changed_certificate_graph_and_dual_acceptance']='required_separate_checks'
  row['occurrences'].append(item)
 rows.append(row)
receipt={'schema':'mpk.csharp_practical.t01_w09.actual_program_alias_regeneration_receipt.v1','status':'passed_standalone_actual_generation_strict_import_and_complete_semantic_metadata_lineage' if not failures else 'failed_lineage_comparison','selection_reason':plan['selection_reason'],'actual_programs':len(rows),'program_occurrences':sum(len(x['occurrences']) for x in rows),'original_certificate_files_checked':sum(len(v) for v in index.values()),'programs':rows,'digest_paths':dict(observed),'failures':failures,'original_application_proof_ids_pending':987,'full_t01_gate':plan['full_t01_gate'],'full_t06_gate':plan['full_t06_gate']};(b/'program-alias-comparison-2.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k not in ['programs','failures','digest_paths','selection_reason']},ensure_ascii=False));print('failures',len(failures))
