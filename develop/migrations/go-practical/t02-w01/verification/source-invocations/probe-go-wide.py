"""Full original wide stream in the unchanged Go core; no prefix or cap change."""
from pathlib import Path
import datetime,hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'go-wide-probe';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
prior_path=ROOT.parent/'core-interfaces/paired/result.json';prior=json.loads(prior_path.read_bytes());binary=prior['go_binary'];assert seal(binary['path'])==binary
for row in prior['go_production_sources']:assert seal(row['path'])==row
certificate=ROOT/'rust-entries/certificates/wide-000.mpcert';inputs=[Path(__file__),prior_path,Path(binary['path']),certificate,*[Path(r['path']) for r in prior['go_production_sources']]];rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());command=[binary['path'],'verify',str(certificate)]
r={'status':'running','complete_prelaunch_inputs':seal(receipt),'actual_driver_prelaunch':seal(OUT/'driver.py'),'go_binary':binary,'command':command,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(r,indent=2)+'\n')
save();start=time.monotonic()
with (OUT/'stdout.log').open('wb') as stdout,(OUT/'stderr.log').open('wb') as stderr:code=subprocess.run(command,stdout=stdout,stderr=stderr,check=False).returncode
for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
r.update(status='passed' if code==0 else 'failed',whole_inputs_unchanged=True,exit_code=code,seconds=time.monotonic()-start,certificate=seal(certificate),stdout=seal(OUT/'stdout.log'),stderr=seal(OUT/'stderr.log'),go_observation=json.loads((OUT/'stdout.log').read_bytes()),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({k:r[k] for k in ['status','exit_code','seconds']}))
