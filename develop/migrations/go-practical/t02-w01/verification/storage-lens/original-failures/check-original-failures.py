"""Independently inspect the retained original failed foundation inputs."""
from pathlib import Path
import datetime,hashlib,json,subprocess,time
OUT=Path(__file__).resolve().parent
PRIOR=OUT.parent/'core-interfaces/paired/result.json'
def seal(path):
 raw=Path(path).read_bytes();return {'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
prior=json.loads(PRIOR.read_bytes());binary=prior['go_binary']
assert seal(binary['path'])==binary and all(seal(r['path'])==r for r in prior['go_production_sources'])
run=OUT/'original-failures';run.mkdir(exist_ok=False)
driver=run/'check-original-failures.py';driver.write_bytes(Path(__file__).read_bytes())
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'go_binary':binary,'go_production_sources':prior['go_production_sources'],'cases':[],'original_Rust_outputs_reconstructed':False,'current_resource_limits_changed':False}
def save():(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 for epoch,code in [('fields-before-frames','CORE_TYPE_MISMATCH'),('hygienic-transport','CORE_FUEL_EXHAUSTED')]:
  candidate=OUT/f'epochs/{epoch}/foundation/candidates/lens-foundation.mpcert';rust_path=candidate.with_suffix('.rust.json');rust=json.loads(rust_path.read_bytes())
  assert rust['verdict']=='rejected' and json.loads(rust['error_detail'])['code']==code
  command=[binary['path'],'verify',str(candidate)];start=time.monotonic();result=subprocess.run(command,capture_output=True,check=False)
  stdout=run/(epoch+'.stdout.log');stderr=run/(epoch+'.stderr.log');stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr);go=json.loads(result.stdout)
  record['cases'].append({'epoch':epoch,'input':seal(candidate),'original_Rust_report':seal(rust_path),'original_source_inputs':seal(OUT/f'epochs/{epoch}/inputs.json'),'original_Rust_execution':seal(OUT/f'epochs/{epoch}/foundation/result.json'),'Rust':rust,'Go':go,'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr)});save()
  assert result.returncode!=0 and not result.stderr and go['verdict']=='rejected' and go['error_kind']=='core_check'
 assert seal(binary['path'])==binary and all(seal(r['path'])==r for r in prior['go_production_sources'])
 record.update(status='passed',original_inputs_rejected=2,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','original_inputs_rejected':2}))
except BaseException:
 record['status']='failed';save();raise
