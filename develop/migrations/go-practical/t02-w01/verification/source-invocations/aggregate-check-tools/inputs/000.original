"""Link a reporting-only wrapper to the exact original compiled Rust kernel."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'aggregate-check-tools';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
original=ROOT/'generation-linked/result.json';run=json.loads(original.read_bytes());assert run['status']=='passed'
libs=run['exact_compiled_library_identities'];library=libs['mpk_kernel'];assert seal(library['path'])==library
source=ROOT/'verify-aggregate.rs';binary=Path('/private/tmp/mpk-go-source-invocations-whole-kernel-174');command=['rustc','--edition=2021','-C','opt-level=1','-C','debug-assertions=yes','-C','overflow-checks=yes','-L','dependency=target/debug/deps','--extern','mpk_kernel='+library['path'],str(source),'-o',str(binary)]
inputs=[Path(__file__),source,original,ROOT/'generation-linked/prelaunch-inputs.json',ROOT/'rust-entries/inputs.json'];rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','command':command,'actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'exact_original_kernel_library':library,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
with (OUT/'stdout.log').open('wb') as out,(OUT/'stderr.log').open('wb') as err:code=subprocess.run(command,stdout=out,stderr=err,check=False).returncode
unchanged=all(seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode'] for r in rows) and seal(library['path'])==library
record.update(status='passed' if code==0 and unchanged else 'failed',exit_code=code,whole_inputs_unchanged=unchanged,stdout=seal(OUT/'stdout.log'),stderr=seal(OUT/'stderr.log'),binary=seal(binary) if binary.exists() else None,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':record['status'],'exit_code':code}));raise SystemExit(record['status']!='passed')
