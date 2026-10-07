import copy,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration/develop/migrations/csharp-03/ordinary-foundation/control-predicates');root=b/'control-pattern-consumer-regeneration/generated';mapping=json.loads((b/'context-name-rebindings-10.json').read_bytes());compact=lambda x:json.dumps(x,ensure_ascii=False,separators=(',',':')).encode();sha=lambda x:hashlib.sha256(compact(x)).hexdigest();rows=[];names={}
def bind(a,c,role,old_id,new_id,file):
 if a is None or c is None:assert a==c,(file,role);return
 prefix='Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H';assert a==prefix+sha(old_id),(file,role,a,prefix+sha(old_id));assert c==prefix+sha(new_id),(file,role,c,prefix+sha(new_id))
 if a in mapping:assert mapping[a]==c,(file,role)
 if a in names:assert names[a]==c,(file,role)
 names[a]=c;rows.append({'file':file,'role':role,'old_name':a,'new_name':c,'old_identity_sha256':sha(old_id),'new_identity_sha256':sha(new_id)})
for folder in ['with-pattern-primitives','with-pattern-routes','with-packed-pattern-proof-types']:
 for fresh in sorted((root/folder).glob('*.json')):
  if fresh.name.endswith('.control.json'):continue
  old=json.loads((r/folder/fresh.name).read_bytes());new=json.loads(fresh.read_bytes());id=fresh.stem;oldctrl=json.loads((r/'with-pattern-routes'/(id+'.control.json')).read_bytes());newctrl=json.loads((root/'with-pattern-routes'/(id+'.control.json')).read_bytes());oldflows={x['function_id']:x for x in oldctrl['functions']};newflows={x['function_id']:x for x in newctrl['functions']};file=folder+'/'+fresh.name
  assert len(old.get('pattern_sources',[]))==len(new.get('pattern_sources',[]))
  for a,c in zip(old.get('pattern_sources',[]),new.get('pattern_sources',[])):
   assert a['source']==c['source'] and a['source_step']==c['source_step'];step=a['source_step'];obs=a['source'];op=oldflows[obs['function_id']]['source_graph']['operations'][step['source_ordinal']];freshop=newflows[obs['function_id']]['source_graph']['operations'][step['source_ordinal']];assert op==freshop
   bind(a['definition'],c['definition'],'PatternSource',[old['source_ir_sha256'],step,obs,op],[new['source_ir_sha256'],step,obs,freshop],file)
  for key,role in [('pattern_scopes','PatternExecutionScopeWithSourceObservations'),('pattern_capture_scopes','PatternExecutionScopeWithCaptureAndSourceObservations')]:
   assert len(old.get(key,[]))==len(new.get(key,[]))
   for a,c in zip(old.get(key,[]),new.get(key,[])):
    assert a['source_sequent_id']==c['source_sequent_id'] and len(a['executions'])==len(c['executions'])
    for x,y in zip(a['executions'],c['executions']):
     assert x['source_execution']==y['source_execution'];assert x['arguments']==y['arguments'] and x['native_argument_indices']==y['native_argument_indices'];e=x['source_execution'];tail=[a['source_sequent_id'],e['edge_id'],e['component_map_sha256']]
     bind(x['definition'],y['definition'],role,[old['source_ir_sha256'],old['control_vc_sha256']]+tail,[new['source_ir_sha256'],new['control_vc_sha256']]+tail,file)
  for a,c in zip(old.get('pattern_proof_types',[]),new.get('pattern_proof_types',[])):
   assert a['source_sequent_id']==c['source_sequent_id'] and a['native_edge_id']==c['native_edge_id'];assert a['arguments']==c['arguments'] and a['packed_environment']==c['packed_environment'] and a['goal_argument_indices']==c['goal_argument_indices'];oldid=[a['source_sequent_id'],a['native_edge_id'],a['components'],a['arguments'],a['packed_environment']];newid=[c['source_sequent_id'],c['native_edge_id'],c['components'],c['arguments'],c['packed_environment']]
   bind(a['scope_proposition_definition'],c['scope_proposition_definition'],'PackedPatternScopeProposition',oldid,newid,file);bind(a['refinement_proposition_definition'],c['refinement_proposition_definition'],'PackedPatternSourceRefinementType',[oldid,a['goal_definition'],a['goal_argument_indices']],[newid,c['goal_definition'],c['goal_argument_indices']],file)
   assert len(a['premise_proofs'])==len(c['premise_proofs'])
   for i,(x,y) in enumerate(zip(a['premise_proofs'],c['premise_proofs'])):bind(x['theorem'],y['theorem'],'PackedPatternScopePremiseProof',[oldid,i],[newid,i],file)
for a,c in names.items():mapping[a]=c
(b/'context-name-rebindings-11.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n');roles={role:sum(row['role']==role for row in rows) for role in sorted({x['role'] for x in rows})};receipt={'schema':'mpk.csharp_practical.t01_w09.pattern_consumer_name_identity_rebinding.v1','status':'passed_actual_hash_rules_exact_source_operations_native_execution_and_environment_layouts','actual_generation_status_sha256':hashlib.sha256((b/'control-pattern-consumer-regeneration/status.json').read_bytes()).hexdigest(),'names':len(names),'roles':roles,'identities':rows,'source_operation_rule':'Both old/new PatternSource names hash the exact typed tuple (VIR hash, source step, source observation, operation), with complete source steps/observations and typed operations compared exactly. Native execution/argument indices and packed environments are unchanged. Scope and packed proposition/premise names follow their exact ordered producer tuples. Full metadata comparison, full graphs and both checker acceptances remain separately required.','original_application_proof_ids_pending':987};(b/'control-pattern-consumer-name-rebinding-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print({k:v for k,v in receipt.items() if k!='identities'})
