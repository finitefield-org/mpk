"""Exercise the independent decoder on both complete source/U shapes early."""
from pathlib import Path
import datetime,hashlib,importlib.util,json,sys
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'oracle-pilot';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
files=[ROOT/'rust-entries/certificates'/f'{name}.mpcert' for name in ['pure-000','wide-000']]
inputs=[Path(__file__),ROOT/'structural-oracle.py',ROOT.parent/'source-error-layouts/structural-oracle.py',Path('develop/probes/go-practical/carrier_terms.py')]
for p in files:inputs.extend([p,p.with_suffix('.rust.json'),p.with_suffix('.sources.json')])
rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n')
record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'complete_final_matrix':False,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 spec=importlib.util.spec_from_file_location('independent_invocations',ROOT/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 observed=[m.inspect(p) for p in files]
 assert {r['original_cases'] for r in observed}=={0,70}
 for r in rows:
  assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode']
 record.update(status='passed',whole_inputs_unchanged=True,streams=observed,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','normal_streams':len(observed),'original_cases':[r['original_cases'] for r in observed]}))
except BaseException:
 record['status']='failed';save();raise
