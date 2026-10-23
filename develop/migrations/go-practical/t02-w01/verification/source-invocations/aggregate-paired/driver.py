"""Bind both real kernels to all original call definitions and exact source imports."""
from pathlib import Path
import datetime,hashlib,json
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'aggregate-paired';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
names=['aggregate-union','oracle-generation','native-comparison-generation','generation-linked','linkage','paired-pure','aggregate-rust-pure','aggregate-rust-wide','aggregate-go-pure','aggregate-go-wide']
runs={n:json.loads((ROOT/n/'result.json').read_bytes()) for n in names};assert all(r['status']=='passed' and r['whole_inputs_unchanged'] for r in runs.values())
inputs=[Path(__file__)]+[ROOT/n/'result.json' for n in names]
for name in ['aggregate-rust-pure','aggregate-rust-wide','aggregate-go-pure','aggregate-go-wide']:
 r=runs[name];inputs.extend(Path(r[k]['path']) for k in ['stdout','stderr','complete_prelaunch_inputs'])
inputs+=sorted((ROOT/'aggregation-interned/certificates').iterdir())+sorted((ROOT/'generation-linked/certificates').iterdir())+sorted((ROOT/'linkage/certificates').iterdir())
rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 units=[]
 for unit in ['pure','wide']:
  rust,go=[runs[f'aggregate-{engine}-{unit}'] for engine in ['rust','go']]
  assert rust['certificate']==go['certificate'] and seal(rust['certificate']['path'])==rust['certificate']
  a,b=rust['observation'],go['observation'];assert a['verdict']==b['verdict']=='accepted' and a['axiom_count']==0 and not a['axiom_report']['entries']
  assert a['hashes']==b['hashes'] and a['module']==b['module'] and a['declaration_count']==b['declaration_count']
  raw=Path(rust['certificate']['path']).read_bytes();assert a['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
  originals=sorted((ROOT/'generation-linked/certificates').glob(unit+'-*.mpcert'))
  for p in originals:
   q=ROOT/'linkage/certificates'/p.name;assert q.read_bytes()==p.read_bytes()
   assert q.with_suffix('.sources.json').read_bytes()==p.with_suffix('.sources.json').read_bytes()
  assert (ROOT/'generation-linked/certificates'/f'{unit}-inventory.json').read_bytes()==(ROOT/'linkage/certificates'/f'{unit}-inventory.json').read_bytes()
  metadata=json.loads(Path(rust['certificate']['path']).with_suffix('.sources.json').read_bytes());assert metadata['declarations']==a['declaration_count'] and len(metadata['components'])==len(originals)
  assert metadata['declarations']<=4096 and metadata['terms']<=131072 and metadata['bytes']<=2097152
  units.append({'unit':unit,'original_call_interfaces':len(originals),'certificate':rust['certificate'],'declarations':a['declaration_count'],'terms':metadata['terms'],'bytes':len(raw),'both_real_kernels_accept_identical_complete_union':True,'zero_axioms':True,'every_original_call_definition_and_proof_retained':True,'every_original_site_exact_import_and_predecessor_identity_checked':True})
 controls=[case for case in runs['paired-pure']['cases'] if case['name'].startswith('control-')];assert {c['name'] for c in controls}=={'control-extra-theorem','control-stale-post'}
 for c in controls:
  assert c['rust_observation']['verdict']==c['go_observation']['verdict']==('accepted' if c['name']=='control-extra-theorem' else 'rejected')
  if c['name']=='control-extra-theorem':assert c['rust_observation']['axiom_count']==0 and c['rust_observation']['hashes']==c['go_observation']['hashes']
  else:assert json.loads(c['rust_observation']['error_detail'])['code']=='CORE_TYPE_MISMATCH' and c['go_observation']['error_kind']=='core_check'
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,units=units,complete_original_call_interfaces=sum(u['original_call_interfaces'] for u in units),complete_same_byte_union_streams=len(units),both_kernel_controls=controls,individual_row_full_matrix_acceptance_claimed=False,source_execution_callee_or_context_proof=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','complete_original_call_interfaces':record['complete_original_call_interfaces'],'complete_same_byte_union_streams':len(units)}))
except BaseException:
 record['status']='failed';save();raise
