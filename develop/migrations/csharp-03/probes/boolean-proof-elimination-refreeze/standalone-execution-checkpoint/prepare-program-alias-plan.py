import collections,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');root=r/'develop/migrations/csharp-03/ordinary-foundation';mapping=json.loads((b/'actual-vir-hash-rebindings.json').read_bytes());identities=json.loads((b/'vir-hash-replay-audit.json').read_bytes())['identities'];corpora={p['request_path']:p for p in json.loads((b/'response-candidate-plan.json').read_bytes())}
producers={name:'vir' for name in ['json_products','source_clauses','structural_public','structural_boundary','contract_expressions','calendar_data','integer_data','decimal_data','floating_data','lifted_data','source_value_data','string_data','structural_data']}
producers.update({name:'emitted' for name in ['json_depth_guarded_envelopes','json_limits_guarded_envelopes','json_boundary_fields','json_typed_nodes','json_envelopes','json_typed_guarded_envelopes']})
sources={}
for x in identities:
 sources.setdefault(x['old_source_ir_sha256'],dict(request_path=x['corpus'],response_path=corpora[x['corpus']]['response_path'],id=x['new_id'],new_source_ir_sha256=x['new_source_ir_sha256']))
targets={};unmatched=[]
def walk(v,p,ptr):
 if isinstance(v,dict):
  schema=v.get('schema','');short=schema.removeprefix('mpk.csharp.ordinary_').removesuffix('.v1')
  if schema==f'mpk.csharp.ordinary_{short}.v1' and short in producers and v.get('foundation_sha256')=='230c708601f4b89feeae28af23da10ccb11eec998d990b5133cf9336369e15a2' and 'certificate_sha256' in v:
   old=v['source_ir_sha256'];occ={'file':str(p.relative_to(r)),'pointer':ptr,'original_program':v}
   if old in sources:
    key=(schema,old);target=targets.setdefault(key,dict(schema=schema,producer=short,context=producers[short],old_source_ir_sha256=old,source=sources[old],occurrences=[]))
    target['occurrences'].append(occ)
   else:unmatched.append(occ)
  for k,x in v.items():walk(x,p,ptr+'/'+k.replace('~','~0').replace('/','~1'))
 elif isinstance(v,list):
  for i,x in enumerate(v):walk(x,p,ptr+'/'+str(i))
for p in sorted(root.rglob('*.json')):
 if any(x=='previous' or x.startswith(('previous-','pre-')) for x in p.parts) or 'verification-logs' in p.parts:continue
 walk(json.loads(p.read_bytes()),p,'')
rows=[]
for (schema,old),v in sorted(targets.items()):
 v['label']=v['producer']+'-'+old[:16];rows.append(v)
plan={'schema':'mpk.csharp_practical.t01_w09.actual_program_alias_regeneration_plan.v1','source_commit':'957bd9c8db1adfa4b29594a4ef8b822b57d2193c','selection_reason':'Only still-current metadata and aliases for deterministic producers whose old/new VIR identity was independently replayed from original captures. Generate and strictly import actual new canonical program bytes; compare complete original metadata and certificate bytes before any fixture mutation. Historical directories remain unchanged. This is linkage reconciliation, not application proof discharge.','producers':producers,'targets':rows,'unmatched_occurrences':unmatched,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
(b/'program-alias-plan.json').write_text(json.dumps(plan,sort_keys=True,indent=2)+'\n');print(json.dumps({'programs':len(rows),'occurrences':sum(len(x['occurrences']) for x in rows),'contexts':len({x['old_source_ir_sha256'] for x in rows}),'schemas':dict(collections.Counter(x['producer'] for x in rows)),'unmatched':len(unmatched)}))
