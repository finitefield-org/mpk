import collections,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');root=r/'develop/migrations/csharp-03/ordinary-foundation';identities=json.loads((b/'vir-hash-replay-audit.json').read_bytes())['identities'];corpora={p['request_path']:p for p in json.loads((b/'response-candidate-plan.json').read_bytes())};sources={}
for x in identities:sources.setdefault(x['old_source_ir_sha256'],dict(request_path=x['corpus'],response_path=corpora[x['corpus']]['response_path'],id=x['new_id'],new_source_ir_sha256=x['new_source_ir_sha256']))
producers={name:'emitted' for name in ['json_envelopes','json_depth_guarded_envelopes','json_typed_guarded_envelopes','json_limits_guarded_envelopes','json_boundary_fields','json_typed_nodes','json_typed_depth']};targets={};unmatched=[]
def walk(v,p,ptr,container):
 if isinstance(v,dict):
  schema=v.get('schema','');short=schema.removeprefix('mpk.csharp.ordinary_').removesuffix('.v1')
  if schema==f'mpk.csharp.ordinary_{short}.v1' and short in producers:
   old=v.get('source_ir_sha256',v.get('parsers',{}).get('source_ir_sha256'))
   if old:
    sha=v.get('certificate_sha256',container.get('certificate_sha256'))
    assert sha,(str(p),ptr)
    occ={'file':str(p.relative_to(r)),'pointer':ptr,'original_program':v,'original_certificate_sha256':sha}
    if old in sources:
     key=(schema,old);target=targets.setdefault(key,dict(schema=schema,producer=short,context=producers[short],old_source_ir_sha256=old,source=sources[old],occurrences=[]));target['occurrences'].append(occ)
    else:unmatched.append(occ)
  for k,x in v.items():
   if not k.startswith(('prior_','previous_','before_')):walk(x,p,ptr+'/'+k.replace('~','~0').replace('/','~1'),v)
 elif isinstance(v,list):
  for i,x in enumerate(v):walk(x,p,ptr+'/'+str(i),container)
for p in sorted(root.rglob('*.json')):
 if any(x=='previous' or x.startswith(('previous-','pre-')) for x in p.parts) or 'verification-logs' in p.parts:continue
 v=json.loads(p.read_bytes());walk(v,p,'',{})
rows=[]
for (schema,old),v in sorted(targets.items()):v['label']=v['producer']+'-'+old[:16];rows.append(v)
plan={'schema':'mpk.csharp_practical.t01_w09.actual_wrapper_program_regeneration_plan.v1','source_commit':'957bd9c8db1adfa4b29594a4ef8b822b57d2193c','selection_reason':'Reconcile only seven JSON wrapper metadata schemas whose old/new source VIR identity is independently replayed. Regenerate complete wrapper canonical bytes and strict imports from original captures, including the nested parser/decoder/depth/node programs. Compare old certificates and complete non-digest metadata before promotion; preserve explicit historical fields and directories. This is linkage reconciliation, not application proof discharge.','producers':producers,'targets':rows,'unmatched_occurrences':unmatched,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'};(b/'wrapper-program-plan.json').write_text(json.dumps(plan,sort_keys=True,indent=2)+'\n');print(json.dumps({'programs':len(rows),'occurrences':sum(len(x['occurrences']) for x in rows),'contexts':len({x['old_source_ir_sha256'] for x in rows}),'schemas':dict(collections.Counter(x['producer'] for x in rows)),'unmatched':len(unmatched)}))
