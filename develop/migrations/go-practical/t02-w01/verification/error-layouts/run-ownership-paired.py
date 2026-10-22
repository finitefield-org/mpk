"""Pair the completed honest extension; original failed control stays visible."""
from pathlib import Path
import hashlib
import json
import subprocess
import time

ROOT=Path(__file__).resolve().parent


def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


execution=ROOT/'final/result.json'
original=json.loads(execution.read_bytes())
assert original['status']=='failed' and original['whole_source_inputs_unchanged']
output=(ROOT/'final/stdout.log').read_text()
assert '2 passed; 1 failed' in output
assert 'reject_wrong_layout_nil_field_order_or_incomplete_adapters' in output
assert 'exact_layout_ownership_and_unchanged_artifact_resource_boundaries' in output
certificate=ROOT/'final/certificates/honest-extension.mpcert'
rust_path=certificate.with_suffix('.rust.json')
rust=json.loads(rust_path.read_bytes())
assert rust['verdict']=='accepted' and rust['axiom_count']==0
prior_path=ROOT.parent/'core-interfaces/paired/result.json'
prior=json.loads(prior_path.read_bytes())
binary=prior['go_binary']
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:
    assert seal(row['path'])==row
out=ROOT/'ownership-paired'
out.mkdir(exist_ok=False)
driver=out/'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
record={'status':'running','actual_driver_prelaunch':seal(driver),'original_Rust_execution':seal(execution),
        'go_binary':binary,'go_production_sources':prior['go_production_sources'],
        'actual_unchanged_Go_build_evidence':seal(prior_path),'cases':[],
        'scope':'Only the honest extension from the passing ownership root; the original separate unreferenced-type control failed and remains retained. Both valid extended alternatives fail exact component ownership. No source/profile/program authority.'}
result_path=out/'result.json'
result_path.write_text(json.dumps(record,indent=2)+'\n')
before,report_before=seal(certificate),seal(rust_path)
command=[binary['path'],'verify',str(certificate)]
start=time.monotonic()
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
stdout,stderr=out/'honest-extension.stdout.log',out/'honest-extension.stderr.log'
stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
go=json.loads(result.stdout)
record['cases'].append({'name':'honest-extension','certificate':before,'rust_report':report_before,
    'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,
    'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go})
valid=result.returncode==0 and not result.stderr and go['verdict']==rust['verdict']=='accepted'
valid=valid and rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
record.update(status='passed' if valid else 'failed',same_byte_streams=1,zero_axiom_mathematical_acceptances=1)
result_path.write_text(json.dumps(record,indent=2)+'\n')
assert seal(certificate)==before and seal(rust_path)==report_before
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:
    assert seal(row['path'])==row
assert valid
print(json.dumps({'status':'passed','name':'honest-extension','verdict':'accepted'}))
