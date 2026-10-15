import hashlib,json,pathlib
b=pathlib.Path(__file__).parent;plan=json.loads((b/'wrapper-program-plan.json').read_bytes());mapping=json.loads((b/'context-name-rebindings-5.json').read_bytes());canonical=json.loads((b/'context-name-rebindings.json').read_bytes());names={};rows=[]
for t in plan['targets']:
 if 'envelopes' not in t['producer']:continue
 fresh=json.loads((b/'wrapper-program-regeneration/generated'/(t['label']+'.program.json')).read_bytes())
 for o in t['occurrences']:
  defs=o['original_program']['definitions'];newdefs=fresh['definitions'];assert len(defs)==len(newdefs)
  for old,new in zip(defs,newdefs):
   a=old['contract_sha256'];c=new['contract_sha256'];assert canonical.get(a,a)==c
   assert old['arguments_shape']==new['arguments_shape'] and old['arguments_depth']==new['arguments_depth'] and old['packet_depth']==new['packet_depth']
   old_state=f'Mpk.CSharp.Ordinary.JsonEnvelope.H{a}.State';new_state=f'Mpk.CSharp.Ordinary.JsonEnvelope.H{c}.State'
   role='Mpk.CSharp.Ordinary.Structural.T';old_name=role+old_state.encode().hex();new_name=role+new_state.encode().hex()
   if old_name in names:assert names[old_name]==new_name
   names[old_name]=new_name;rows.append({'label':t['label'],'file':o['file'],'pointer':o['pointer'],'old_contract_sha256':a,'new_contract_sha256':c,'old_state_type_id':old_state,'new_state_type_id':new_state,'old_structural_name_prefix':old_name,'new_structural_name_prefix':new_name,'argument_shape_and_depths_preserved':True})
mapping.update(names);(b/'context-name-rebindings-6.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n');receipt={'status':'passed_actual_canonical_contract_and_hex_encoded_state_name_derivation','distinct_structural_name_prefixes':len(names),'occurrences':len(rows),'producer_owners':['crates/mpk-vc/src/csharp_practical_ordinary_json_envelopes.rs:393','crates/mpk-vc/src/csharp_practical_ordinary_structural.rs:328'],'identity_rule':'Structural.T followed by the UTF-8 hex encoding of JsonEnvelope.H<independently verified canonical contract SHA256>.State','identities':rows,'complete_body_and_graph_checks':'required_separately_before_promotion','original_application_proof_ids_pending':987};(b/'json-state-name-rebinding-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(len(names),len(rows))
