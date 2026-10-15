import hashlib
import json
from pathlib import Path
b=Path(__file__).parent
r=Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
f=r/'develop/migrations/csharp-03/ordinary-foundation'
old='230c708601f4b89feeae28af23da10ccb11eec998d990b5133cf9336369e15a2'
new='99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844'
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
plans=['wrapper-program-promotion-plan-2','pattern-consumer-promotion-plan']
overlay={}; planned={}
for folder in plans:
 d=json.loads((b/folder/'plan.json').read_text())
 for row in d['changes']:
  src=b/folder/'files'/row['path']
  assert h(src)==row['raw_sha256']
  assert h(r/row['path'])==row['before_raw_sha256'],row['path']
  assert row['path'] not in planned
  planned[row['path']]=row
  if src.suffix=='.json': overlay[row['path']]=src
old_current=[]; old_historical=[]; excluded=0; current_new=[]
def scan(node,pointer,rel):
 global excluded
 if isinstance(node,dict):
  if 'foundation_sha256' in node:
   value=node['foundation_sha256']
   if value==old:
    row={'file':rel,'pointer':pointer,'schema':node.get('schema'),'source_ir_sha256':node.get('source_ir_sha256')}
    parts=Path(rel).parts
    if 'verification-logs' in parts: excluded+=1
    elif any(x.startswith(('previous-','before-')) for x in parts): pass
    elif 'with-pattern-observations' in parts: old_historical.append(row)
    else: old_current.append(row)
   elif value==new: current_new.append({'file':rel,'pointer':pointer})
  for k,v in node.items():scan(v,pointer+'/'+str(k).replace('~','~0').replace('/','~1'),rel)
 elif isinstance(node,list):
  for i,v in enumerate(node):scan(v,pointer+'/'+str(i),rel)
for p in sorted(f.rglob('*.json')):
 rel=str(p.relative_to(r)); q=overlay.get(rel,p)
 scan(json.loads(q.read_text()),'',rel)
assert not old_current,old_current
assert len(old_historical)==18
metrics=json.loads((f/'relations/metrics.json').read_text())
assert len(metrics)==21 and all(x['metadata']['foundation_sha256']==new for x in metrics)
receipt={'status':'passed_zero_old_current_foundation_identity_fields_after_explicit_unapplied_plans',
 'selection_reason':'Read-only direct foundation identity inventory after the published relation metric refresh; candidate wrapper288 and pattern126 files are overlaid explicitly and are not promoted or accepted by this audit. Historical61-condition fixtures and immutable predecessor/verification evidence are separately classified.',
 'published_relation_metric_rows':21,'unapplied_plan_files':len(planned),
 'overlaid_json_files':len(overlay),'current_old_foundation_fields':old_current,
 'retained_historical_61_condition_documents':old_historical,
 'excluded_verification_evidence_occurrences':excluded,
 'current_new_foundation_field_count':len(current_new),
 'application_proof_ids_pending':987,'whole_gate':'deferred_to_final_T01_W10_and_T06_W12'}
(b/'pending-foundation-current-overlay-inventory-5.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if not isinstance(v,list)},indent=2))
