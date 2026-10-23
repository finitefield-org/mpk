"""Independently reconstruct every complete generated candidate, before proofs finish."""
from pathlib import Path
import datetime,hashlib,json,subprocess,sys,time
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'oracle-generation';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
source=ROOT/'generation-linked';run=json.loads((source/'result.json').read_bytes());assert run['status']=='passed' and run['whole_inputs_unchanged'] and run['kernel_proofs_rechecked'] is False
inputs=[Path(__file__),ROOT/'structural-oracle.py',ROOT/'observe-structural.py',ROOT.parent/'source-error-layouts/structural-oracle.py',Path('develop/probes/go-practical/carrier_terms.py'),source/'result.json',source/'prelaunch-inputs.json',ROOT/'rust-entries/certificates/control-extra-theorem.mpcert',ROOT/'rust-entries/certificates/control-extra-theorem.rust.json']+sorted((source/'certificates').iterdir());rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes())
command=[sys.executable,str(ROOT/'observe-structural.py'),str(source/'certificates'),str(OUT/'structural-comparison.json'),'--generation-only']
record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'command':command,'normal_original_Rust_kernel_reports_required':False,'complete_final_kernel_matrix':False,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save();start=time.monotonic()
try:
 with (OUT/'stdout.log').open('wb') as out,(OUT/'stderr.log').open('wb') as err:code=subprocess.run(command,stdout=out,stderr=err,check=False).returncode
 record.update(exit_code=code,seconds=time.monotonic()-start,stdout=seal(OUT/'stdout.log'),stderr=seal(OUT/'stderr.log'));save();assert code==0
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print((OUT/'stdout.log').read_text(),end='')
except BaseException:
 record['status']='failed';save();raise
