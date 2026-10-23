"""Compare all original source calls, typed groups, slots and possible targets."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy,sys
ROOT=Path(__file__).resolve().parent;epoch=sys.argv[1] if len(sys.argv)>1 else 'rust-entries';generation_only='--generation-only' in sys.argv[2:];OUT=ROOT/('native-comparison-generation' if generation_only else 'native-comparison');OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
run=ROOT/epoch/'result.json';r=json.loads(run.read_bytes());assert r['status']=='passed'
if generation_only:
 assert r['whole_inputs_unchanged'] and r['kernel_proofs_rechecked'] is False
 input_receipt=ROOT/epoch/'prelaunch-inputs.json';assert seal(input_receipt)==r['complete_prelaunch_inputs']
else:
 assert r['whole_source_inputs_unchanged'];assert seal(r['source_receipt']['path'])==r['source_receipt'];input_receipt=ROOT/epoch/'inputs.json'
manifest=ROOT/'native-output-original/manifest.json';archive=json.loads(manifest.read_bytes());assert archive['status']=='passed'
inputs=[Path(__file__),ROOT/'read-originals.py',manifest,ROOT/'corpus.json',ROOT/'native-complete/result.json',run,input_receipt,ROOT/'derived/result.json']+[Path(c['encoded']['path']) for c in archive['chunks']]+sorted((ROOT/epoch/'certificates').glob('*.sources.json'))+sorted((ROOT/epoch/'certificates').glob('*-inventory.json'))
rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','complete_prelaunch_inputs':seal(receipt),'actual_driver_prelaunch':seal(OUT/'driver.py'),'original_Rust_kernel_epoch_terminal_required':not generation_only,'candidate_generation_epoch':seal(run) if generation_only else None,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 _,files=runpy.run_path(str(ROOT/'read-originals.py'))['read']('native-output-original');corpus=json.loads((ROOT/'corpus.json').read_bytes());units=[]
 for unit in corpus['units']:
  name=unit['name'];packet=json.loads(files[name+'/unit-inputs.json']);raw=base64.b64decode(packet['canonical_components'],validate=True);assert raw.hex()==unit['input']['components_hex'];components=json.loads(raw);assert len(components['packages'])==3 and not components['declarations'] and len(components['types'])==1 and components['types'][0]['name']=='bool'
  originals={s['object']['id']:(s['object'],base64.b64decode(s['raw'],validate=True)) for s in packet['sources']}
  for s in unit['input']['sources']:assert originals[s['object']['id']]==(s['object'],bytes.fromhex(s['raw_hex']))
  native=json.loads(files[name+'/native-calls.json']);closure=json.loads(files[name+'/native-closure.json']);inventory=json.loads((ROOT/epoch/f'certificates/{name}-inventory.json').read_bytes());sites=inventory['sites'];assert len(sites)==len(native);actual={(s['source'],s['start'],s['end']):s for s in sites};assert len(actual)==len(sites)
  params=returns=groups=projections=0;dispatches=0
  for observed in native:
   key=tuple(observed[k] for k in ['source','start','end']);s=actual[key];assert s['caller']==[observed['source'],observed['caller_start'],observed['caller_end']]
   assert s['inputs']==observed['inputs'] and s['results']==observed['results'];params=max(params,len(s['inputs']));returns=max(returns,len(s['results']))
   assert len(s['arguments'])==len(observed['arguments'])
   for got,want in zip(s['arguments'],observed['arguments']):
    for field in ['start','end','receiver','values']:assert got[field]==want[field],(name,key,field,got,want)
    groups+=1;projections+=len(got['values'])
   dispatch=[d for d in closure['dispatches'] if (d['source'],d['start'],d['end'])==key];assert s['dispatch']==bool(dispatch)
   if dispatch:
    assert len(dispatch)==1;expected=sorted((t['type_id'],t['callee']['source'],t['callee']['start'],t['callee']['end']) for t in dispatch[0]['targets']);dispatches+=1
    assert sorted((t['dynamic_type'],*t['callee']) for t in s['targets'])==expected
    assert all(t['receiver_type']==t['dynamic_type'] and not t['receiver_path'] for t in s['targets'])
   else:
    targets=[d for d in closure['calls'] if (d['source'],d['start'],d['end'])==key];assert len(targets)==len(s['targets'])==1
    assert s['targets'][0]['callee']==[targets[0]['callee'][k] for k in ['source','start','end']] and s['targets'][0]['dynamic_type'] is None
    if s['targets'][0]['receiver_type'] is not None:assert s['targets'][0]['receiver_type']==s['inputs'][0] and not s['targets'][0]['receiver_path']
  assert params==65 and returns==16 and min(len(s['results']) for s in sites)==0
  assert inventory['cases']==closure['global_cases'] and len(inventory['cases'])==(70 if name=='wide' else 0)
  units.append({'unit':name,'complete_calls':len(sites),'max_inputs':params,'max_results':returns,'argument_groups':groups,'value_projections':projections,'error_dispatches':dispatches,'complete_error_cases':len(inventory['cases']),'all_callers_groups_types_and_targets_equal':True})
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,units=units,complete_calls=sum(u['complete_calls'] for u in units),source_execution_callee_or_context_proof=False,native_type_facts_used_as_constructor_inputs=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','complete_calls':record['complete_calls'],'units':units}))
except BaseException:
 record['status']='failed';save();raise
