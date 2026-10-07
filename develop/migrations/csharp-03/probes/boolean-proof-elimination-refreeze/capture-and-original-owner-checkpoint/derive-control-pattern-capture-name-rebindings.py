import copy,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');oldroot=r/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-captures';newroot=b/'ordinary-regeneration-12/goldens/control-predicates/with-pattern-captures';compact=lambda x:json.dumps(x,ensure_ascii=False,separators=(',',':')).encode();sha=lambda x:hashlib.sha256(compact(x)).hexdigest();mapping=json.loads((b/'context-name-rebindings-9.json').read_bytes());names={};rows=[]
state=json.loads((b/'ordinary-regeneration-12/status.json').read_bytes());stage=state['stages'][0];assert state['status']=='passed_selected_source_bound_ordinary_owners' and stage['exit_code']==0
for fresh in sorted(newroot.glob('*.json')):
 old=json.loads((oldroot/fresh.name).read_bytes());new=json.loads(fresh.read_bytes());assert len(old.get('pattern_captures',[]))==len(new.get('pattern_captures',[]))
 for a,c in zip(old.get('pattern_captures',[]),new.get('pattern_captures',[])):
  aa=a['definition'];cc=c['definition'];x=copy.deepcopy(a);y=copy.deepcopy(c);x['definition']=y['definition']=None;assert x==y,(fresh.name,'capture body');
  if aa is None or cc is None:assert aa==cc;continue
  assert aa=='Mpk.CSharp.Ordinary.ControlPredicate.PatternGoverningCapture.H'+sha([old['source_ir_sha256'],x]);assert cc=='Mpk.CSharp.Ordinary.ControlPredicate.PatternGoverningCapture.H'+sha([new['source_ir_sha256'],y]);
  if aa in names:assert names[aa]==cc
  names[aa]=cc;rows.append({'file':fresh.name,'role':'PatternGoverningCapture','old_name':aa,'new_name':cc,'old_source_ir_sha256':old['source_ir_sha256'],'new_source_ir_sha256':new['source_ir_sha256'],'exact_complete_capture_body_sha256':sha(x)})
 assert len(old.get('pattern_capture_scopes',[]))==len(new.get('pattern_capture_scopes',[]))
 for a,c in zip(old.get('pattern_capture_scopes',[]),new.get('pattern_capture_scopes',[])):
  assert a['source_sequent_id']==c['source_sequent_id'] and len(a['executions'])==len(c['executions'])
  for x,y in zip(a['executions'],c['executions']):
   aa=x['definition'];cc=y['definition'];assert x['source_execution']==y['source_execution'];assert x['arguments']==y['arguments'] and x['native_argument_indices']==y['native_argument_indices']
   if aa is None or cc is None:assert aa==cc;continue
   e=x['source_execution'];role='PatternExecutionScopeWithCapture';old_id=[old['source_ir_sha256'],a['source_sequent_id'],e['edge_id'],e['component_map_sha256']];new_id=[new['source_ir_sha256'],c['source_sequent_id'],e['edge_id'],e['component_map_sha256']];assert aa=='Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H'+sha(old_id);assert cc=='Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H'+sha(new_id)
   if aa in names:assert names[aa]==cc
   names[aa]=cc;rows.append({'file':fresh.name,'role':role,'old_name':aa,'new_name':cc,'old_identity':old_id,'new_identity':new_id,'exact_source_execution_sha256':sha(e),'exact_ordered_arguments_sha256':sha(x['arguments'])})
for a,c in names.items():
 if a in mapping:assert mapping[a]==c
 mapping[a]=c
(b/'context-name-rebindings-10.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n');receipt={'schema':'mpk.csharp_practical.t01_w09.pattern_capture_name_identity_rebinding.v1','status':'passed_exact_complete_capture_bodies_and_actual_producer_name_hashes','producer_capture_owner':'crates/mpk-vc/src/csharp_practical_ordinary_control_pattern_captures.rs:255','producer_scope_owner':'crates/mpk-vc/src/csharp_practical_ordinary_control_pattern_scopes.rs:546','rule':'Governing capture names use the exact ordered serialized (VIR hash, complete capture with definition=None) tuple; every other capture field, execution alternative, native/source binding and argument/component order compares exactly. Capture scope names use the independently checked unchanged native execution tuple. Complete metadata lineage, certificate graphs and both checker acceptances remain separate.','actual_source_owner':stage,'capture_names':sum('PatternGoverningCapture.H' in k for k in names),'capture_scope_names':sum('PatternExecutionScopeWithCapture.H' in k for k in names),'identities':rows,'original_application_proof_ids_pending':987};(b/'control-pattern-capture-name-rebinding-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print({k:v for k,v in receipt.items() if k not in ['identities','actual_source_owner']})
