from pathlib import Path
import hashlib,json,collections
r=Path('/Users/kazuyoshitoshiya/mpk/develop/migrations/csharp-03/ordinary-foundation');current=r/'concrete-types/certificates.json';structural=r/'structural-foundations/certificates.json';types=json.loads(current.read_text())['sources'];old={x['id']:x for x in json.loads(structural.read_text())};result=[];counts=collections.Counter()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def operations(metadata):
 ops={}
 def add(id,normal,args,result,failures=(),**extra):
  assert id not in ops
  ops[id]=dict(normal_definition=normal,argument_type_ids=args,result_type_id=result,failures=list(failures),**extra)
 for family in ['constructions','outcomes','collections','money']:
  for d in metadata[family]:
   for op in d['operations']:
    add(op['operation_id'],op['normal_definition'],op['argument_type_ids'],op['result_type_id'],op['failures'],family=family,currency_predicate_argument_type_id=op.get('currency_predicate_argument_type_id'))
 for d in metadata['sequences']:
  id=d['carrier']['type_id'];add(id+'.length',d['length_definition'],[id],'mpk.csharp.value.u32.v1',family='sequences')
  add(id+'.read',d['read_definition'],[id,'mpk.csharp.value.i32.v1'],d['element_type_id'],[dict(label='index_range',definition=d['index_range_definition'],argument_indices=[0,1])],family='sequences')
  for suffix,field,type in [('equal','equality_definition','bool'),('compare','compare_definition','i32')]:
   if d[field]is not None:add(id+'.'+suffix,d[field],[id,id],'mpk.csharp.value.'+type+'.v1',family='sequences')
 for d in metadata['entries']:
  id=d['carrier']['type_id'];add(id+'.make',d['make_definition'],[d['key_type_id'],d['value_type_id']],id,family='entries')
  for suffix in ['key','value']:add(id+'.'+suffix,d[suffix+'_definition'],[id],d[suffix+'_type_id'],family='entries')
  for suffix,field,type in [('equal','equality_definition','bool'),('compare','compare_definition','i32')]:
   if d[field]is not None:add(id+'.'+suffix,d[field],[id,id],'mpk.csharp.value.'+type+'.v1',family='entries')
 for d in metadata.get('transitions',[]):
  id=d['carrier']['type_id'];add(id+'.make',d['make_definition'],[d['state_type_id'],d['events_type_id'],d['response_type_id']],id,[dict(label='event_bound',definition=d['event_bound_definition'],argument_indices=[d['event_bound_argument_index']])],family='transitions')
  for suffix in ['state','events','response']:add(id+'.'+suffix,d[suffix+'_definition'],[id],d[suffix+'_type_id'],family='transitions')
  for suffix,field,type in [('equal','equality_definition','bool'),('compare','compare_definition','i32')]:
   if d[field]is not None:add(id+'.'+suffix,d[field],[id,id],'mpk.csharp.value.'+type+'.v1',family='transitions')
 return ops
for row in types:
 m=row['metadata'];pin=old.get(row['id']);ops=operations(pin['metadata'])if pin else{};pending_types={d['instance_id']for d in m['pending_type_instances']}
 if pin:assert pin['metadata']['source_ir_sha256']==m['source_ir_sha256']and pin['metadata']['foundation_sha256']==m['foundation_sha256']
 for i in [*[d['instance']for d in m['definitions']],*m['pending_type_instances']]:
  for recipe in i['operation_definitions']:
   op=ops.get(recipe['id']);notes=[]
   if op:
    assert op['argument_type_ids']==recipe['argument_type_ids']and op['result_type_id']==recipe['normal_result_type_id'],(row['id'],recipe['id'])
    if [f['label']for f in op['failures']]!=recipe['error_precedence']:notes.append('Reconcile original frozen error order with the standalone failure metadata.')
    if any(f.get('definition')is None for f in op['failures']):notes.append('Resolve ownership/use-point failure conditions; some predicates have no standalone definition.')
    if op.get('currency_predicate_argument_type_id'):notes.append('Supply the exact currency public predicate at the implicit predicate argument.')
    if pending_types.intersection([*recipe['argument_type_ids'],recipe['normal_result_type_id']]):notes.append('Internal construction-state domain is still pending.')
    status='metadata_link_found'
   else:status='no_same_context_structural_pin';notes.append('Generate the structural component under this original source context; do not reuse a foreign context pin.')
   counts[status]+=1
   result.append(dict(source_context=row['id'],source_ir_sha256=m['source_ir_sha256'],operation_recipe=recipe,structural_metadata=op,status=status,remaining=notes+['Lower both original equivalence goals and preserve every outcome; supply checked application proofs separately.']))
assert len(result)==467
out=dict(status='metadata_discovery_only_not_definition_or_proof_completion',scope='Read-only mapping of original 467 operation recipes to same-context existing structural metadata. This does not validate the referenced certificate bytes, supply an adapter, or discharge any condition.',inputs={str(current.relative_to(r)):sha(current),str(structural.relative_to(r)):sha(structural)},counts=dict(counts),original_operation_conditions=len(result),operations=result,full_gate='deferred_to_T06_W12')
p=r/'verification-logs/concrete-types/operation-link-discovery.json';p.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items()if k!='operations'}))
