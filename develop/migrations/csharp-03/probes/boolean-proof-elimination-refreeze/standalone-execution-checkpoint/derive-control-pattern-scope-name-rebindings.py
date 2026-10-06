import copy,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');oldroot=r/'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-scopes';newroot=b/'ordinary-regeneration-11/goldens/control-predicates/with-pattern-scopes';compact=lambda x:json.dumps(x,ensure_ascii=False,separators=(',',':')).encode();sha=lambda x:hashlib.sha256(compact(x)).hexdigest();mapping=json.loads((b/'context-name-rebindings-8.json').read_bytes());rows=[];names={};unchanged=0
state=json.loads((b/'ordinary-regeneration-11/status.json').read_bytes());stage=next(s for s in state['stages'] if s['stage']=='control-predicates--with-pattern-scopes');assert stage['exit_code']==0 and stage['test_counts']==['1']
for fresh in sorted(newroot.glob('*.json')):
 old=json.loads((oldroot/fresh.name).read_bytes());new=json.loads(fresh.read_bytes());assert len(old.get('pattern_scopes',[]))==len(new.get('pattern_scopes',[]))
 for scope_a,scope_c in zip(old.get('pattern_scopes',[]),new.get('pattern_scopes',[])):
  assert scope_a['source_sequent_id']==scope_c['source_sequent_id'];assert len(scope_a['executions'])==len(scope_c['executions'])
  for a,c in zip(scope_a['executions'],scope_c['executions']):
   aa=a['definition'];cc=c['definition'];assert a['source_execution']==c['source_execution'];assert a['arguments']==c['arguments'];assert a['native_argument_indices']==c['native_argument_indices']
   if aa is None or cc is None:assert aa==cc;continue
   role='PatternExecutionScope';assert aa.startswith('Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H') and cc.startswith('Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H'),(fresh.name,aa,cc)
   execution=a['source_execution'];old_id=[old['source_ir_sha256'],scope_a['source_sequent_id'],execution['edge_id'],execution['component_map_sha256']];new_id=[new['source_ir_sha256'],scope_c['source_sequent_id'],execution['edge_id'],execution['component_map_sha256']]
   assert aa=='Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H'+sha(old_id);assert cc=='Mpk.CSharp.Ordinary.ControlPredicate.'+role+'.H'+sha(new_id)
   if aa in names:assert names[aa]==cc
   names[aa]=cc;unchanged+=aa==cc;rows.append({'file':fresh.name,'source_sequent_id':scope_a['source_sequent_id'],'old_name':aa,'new_name':cc,'old_identity':old_id,'new_identity':new_id,'source_execution_sha256':sha(execution),'ordered_arguments_sha256':sha(a['arguments']),'native_argument_indices':a['native_argument_indices']})
for a,c in names.items():
 if a in mapping:assert mapping[a]==c
 mapping[a]=c
(b/'context-name-rebindings-9.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n');receipt={'schema':'mpk.csharp_practical.t01_w09.pattern_scope_name_identity_rebinding.v1','status':'passed_actual_producer_name_hashes_exact_native_execution_and_ordered_arguments','producer_name_owner':'crates/mpk-vc/src/csharp_practical_ordinary_control_pattern_scopes.rs:546','rule':'Both old and fresh names must equal SHA256 of the exact ordered tuple (VIR hash, source sequent ID, native edge ID, unchanged execution component-map hash). Native execution records, argument order and native argument indices are compared exactly. Complete metadata lineage, full certificate graphs and both checker verdicts are required separately.','actual_source_owner':stage,'execution_scope_names':len(names),'changed_names':sum(a!=c for a,c in names.items()),'unchanged_names':unchanged,'identities':rows,'original_application_proof_ids_pending':987}
(b/'control-pattern-scope-name-rebinding-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print({k:v for k,v in receipt.items() if k not in ['identities','actual_source_owner']})
