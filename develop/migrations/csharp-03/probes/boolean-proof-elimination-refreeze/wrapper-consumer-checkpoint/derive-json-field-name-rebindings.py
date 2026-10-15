import hashlib,json,pathlib
b=pathlib.Path(__file__).parent;plan=json.loads((b/'wrapper-program-plan.json').read_bytes());mapping=json.loads((b/'context-name-rebindings-4.json').read_bytes());base_mapping=json.loads((b/'context-name-rebindings.json').read_bytes());names={};rows=[]
for target in plan['targets']:
 fresh=json.loads((b/'wrapper-program-regeneration/generated'/(target['label']+'.program.json')).read_bytes());newfields=fresh.get('fields',fresh.get('field_decoders',[]))
 for occ in target['occurrences']:
  oldfields=occ['original_program'].get('fields',occ['original_program'].get('field_decoders',[]));assert len(oldfields)==len(newfields)
  for old,new in zip(oldfields,newfields):
   assert old['field_id']==new['field_id'];a=old['contract_sha256'];c=new['contract_sha256'];assert base_mapping.get(a,a)==c
   role='Mpk.CSharp.Ordinary.JsonBoundaryFields.H';old_name=role+hashlib.sha256((a+':'+old['field_id']).encode()).hexdigest()+'.Parse';new_name=role+hashlib.sha256((c+':'+new['field_id']).encode()).hexdigest()+'.Parse';assert old['parse_definition']==old_name and new['parse_definition']==new_name
   if old_name in names:assert names[old_name]==new_name
   names[old_name]=new_name;rows.append({'file':occ['file'],'pointer':occ['pointer'],'field_id':old['field_id'],'old_contract_sha256':a,'new_contract_sha256':c,'old_name':old_name,'new_name':new_name})
mapping.update(names);(b/'context-name-rebindings-5.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n');receipt={'schema':'mpk.csharp_practical.t01_w09.json_field_name_identity_rebinding.v1','status':'passed_actual_canonical_contract_hash_and_producer_name_identity_checks','distinct_field_name_rebindings':len(names),'field_occurrences':len(rows),'identities':rows,'identity_rule':'Mpk.CSharp.Ordinary.JsonBoundaryFields.H<SHA256(contract_sha256 + colon + field_id)>.Parse','producer_owner':'crates/mpk-vc/src/csharp_practical_ordinary_json_boundary_fields.rs:457','body_and_graph_checks':'required_separately_before_promotion','original_application_proof_ids_pending':987};(b/'json-field-name-rebinding-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k!='identities'}))
