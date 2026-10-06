import hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');oldroot=r/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-execution';newroot=b/'ordinary-regeneration-10/goldens/control-predicates/with-execution';compact=lambda x:json.dumps(x,ensure_ascii=False,separators=(',',':')).encode();sha=lambda x:hashlib.sha256(compact(x)).hexdigest();mapping={};names={};rows=[]
state=json.loads((b/'ordinary-regeneration-10/status.json').read_bytes());stage=next(s for s in state['stages'] if s['stage']=='control-predicates--with-execution');assert stage['exit_code']==0 and stage['test_counts']==['1']
for fresh in sorted(newroot.glob('*.json')):
 old=json.loads((oldroot/fresh.name).read_bytes());new=json.loads(fresh.read_bytes());assert len(old['sequents'])==len(new['sequents'])
 for a,c in zip(old['sequents'],new['sequents']):
  x=a['source'];y=c['source'];assert {k:v for k,v in x.items() if k!='attachment_ids'}=={k:v for k,v in y.items() if k!='attachment_ids'};assert len(x['attachment_ids'])==len(y['attachment_ids'])
  for aa,cc in zip(x['attachment_ids'],y['attachment_ids']):
   if aa in mapping:assert mapping[aa]==cc
   mapping[aa]=cc
  aa=a['logical_implication_definition'];cc=c['logical_implication_definition']
  if aa is None or cc is None:assert aa==cc;continue
  assert aa=='Mpk.CSharp.Ordinary.ControlPredicate.LogicalImplication.H'+sha(x);assert cc=='Mpk.CSharp.Ordinary.ControlPredicate.LogicalImplication.H'+sha(y)
  if aa in names:assert names[aa]==cc
  names[aa]=cc;rows.append({'file':fresh.name,'source_sequent_id':x['id'],'old_name':aa,'new_name':cc,'source_body_without_attachment_ids_sha256':sha({k:v for k,v in x.items() if k!='attachment_ids'}),'old_attachment_ids':x['attachment_ids'],'new_attachment_ids':y['attachment_ids']})
allmap=json.loads((b/'context-name-rebindings-6.json').read_bytes());allmap.update(mapping);allmap.update(names);(b/'context-name-rebindings-7.json').write_text(json.dumps(allmap,sort_keys=True,indent=2)+'\n');receipt={'schema':'mpk.csharp_practical.t01_w09.control_sequent_name_identity_rebinding.v1','status':'passed_complete_typed_source_sequent_bodies_and_actual_identity_name_hashes','actual_source_owner':stage,'sequent_names':len(names),'attachment_rebindings':len(mapping),'source_body_rule':'Compare every source field, assumption/goal term and order exactly; only the attachment identity strings differ. Both old/new generated names must equal the producer SHA256 of the exact ordered typed source serialization. Full certificate graph comparison and dual acceptance remain separate requirements.','producer_name_owner':'crates/mpk-vc/src/csharp_practical_ordinary_control_predicates.rs:141','identities':rows,'original_application_proof_ids_pending':987};(b/'control-execution-sequent-name-rebinding-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k not in ['identities','actual_source_owner']}))
