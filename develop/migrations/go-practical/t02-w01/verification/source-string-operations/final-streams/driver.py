"""Retain exact complete terminal subtest streams; never relabel a failed run."""
from pathlib import Path
import datetime,hashlib,json
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'final-streams';OUT.mkdir(exist_ok=False)
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
selected={'remaining':['length-only','control-zero-result','control-erased-domain','control-extra-theorem'],'index-fixed':['index-only'],'index-matrix':None}
# The first run failed in four Index roots, but these two independent subtests
# actually completed successfully. Preserve that whole failed result/stdout.
old=json.loads((ROOT/'remaining/result.json').read_bytes());assert old['status']=='failed' and old['whole_source_inputs_unchanged']
stdout=(ROOT/'remaining/stdout.log').read_text()
for test in ['source_operations_length_without_error_or_byte','source_operations_controls','source_operations_incomplete_source_packet_rejects']:assert 'test '+test+' ... ok' in stdout
inputs=[Path(__file__)];files=[];executions=[]
for epoch,names in selected.items():
 folder=ROOT/epoch;run=json.loads((folder/'result.json').read_bytes());assert run['whole_source_inputs_unchanged']
 assert run['status']==('failed' if epoch=='remaining' else 'passed')
 assert seal(run['source_receipt']['path'])==run['source_receipt']
 inputs.extend([folder/n for n in ['result.json','inputs.json','driver.py','stdout.log','stderr.log']]);executions.append({'epoch':epoch,'actual_result':seal(folder/'result.json'),'failed_parent_run_preserved':epoch=='remaining','selected_completed_subtests_only':True})
 candidates=sorted((folder/'certificates').glob('*.mpcert'))
 if names is not None:candidates=[p for p in candidates if p.stem in names];assert {p.stem for p in candidates}==set(names)
 for p in candidates:
  report=json.loads(p.with_suffix('.rust.json').read_bytes())
  if p.stem.startswith('control-') and p.stem!='control-extra-theorem':assert report['verdict']=='rejected' and json.loads(report['error_detail'])['code']=='CORE_TYPE_MISMATCH'
  else:assert report['verdict']=='accepted' and report['axiom_count']==0
  files.extend([p,p.with_suffix('.rust.json')]);meta=p.with_suffix('.sources.json')
  if meta.exists():files.append(meta)
 if epoch=='index-matrix':files.extend(sorted((folder/'certificates').glob('*-inventory.json')))
assert len({p.name for p in files})==len(files)
assert sum(p.suffix=='.mpcert' for p in files)==28
inputs.extend(files);rows=[]
for i,p in enumerate(inputs):
 q=OUT/'inputs'/f'{i:03d}.original';q.parent.mkdir(exist_ok=True);q.write_bytes(p.read_bytes());q.chmod(p.stat().st_mode&0o777)
 rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(q)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');driver=OUT/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
(OUT/'certificates').mkdir()
for p in files:
 q=OUT/'certificates'/p.name;q.write_bytes(p.read_bytes());q.chmod(p.stat().st_mode&0o777);assert q.read_bytes()==p.read_bytes()
for r in rows:assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode']
record={'status':'passed','assembly_not_new_kernel_execution':True,'normal_rows':25,'complete_streams':28,'original_executions':executions,'actual_driver_prelaunch':seal(driver),'source_receipt':seal(receipt),'complete_prelaunch_inputs':seal(receipt),'whole_source_inputs_unchanged':True,'whole_inputs_unchanged':True,'scope':'Exact full candidate/report/metadata copies from the completed selected subtests. The original failed parent run remains failed and is retained. Source operands/execution/effects/context are not certified. Index-only generic proof fix does not change the Length branch or its passing controls; those exact old streams are reused without repeating unaffected tests.','finished_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','normal_rows':25,'streams':28}))
