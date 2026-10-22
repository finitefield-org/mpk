"""Check completed small endpoint streams while the original large test runs."""
from pathlib import Path
import hashlib
import json
import subprocess
import time

ROOT=Path(__file__).resolve().parent


def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


prior_path=ROOT.parent/'core-interfaces/paired/result.json'
prior=json.loads(prior_path.read_bytes())
binary=prior['go_binary']
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:
    assert seal(row['path'])==row
out=ROOT/'first-large'
out.mkdir(exist_ok=False)
driver=out/'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
live=ROOT/'first/result.json'
(out/'original_Rust_state.json').write_bytes(live.read_bytes())
record={'status':'running','actual_driver_prelaunch':seal(driver),
        'initial_Rust_observation_snapshot':seal(out/'original_Rust_state.json'),
        'go_binary':binary,'go_production_sources':prior['go_production_sources'],
        'actual_unchanged_Go_build_evidence':seal(prior_path),'cases':[],
        'scope':'Original completed N70 whole stream only. Both independent cores inspect its unchanged entire bytes; no full-suite or source correspondence claim.'}


def save():
    (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')


save()
for n in [70]:
    certificate=ROOT/f'first/certificates/layout-N{n}.mpcert'
    rust_path=certificate.with_suffix('.rust.json')
    before,rust_before=seal(certificate),seal(rust_path)
    rust=json.loads(rust_path.read_bytes())
    assert rust['verdict']=='accepted' and rust['axiom_count']==0
    command=[binary['path'],'verify',str(certificate)]
    start=time.monotonic()
    completed=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
    stdout,stderr=out/f'N{n}.stdout.log',out/f'N{n}.stderr.log'
    stdout.write_bytes(completed.stdout);stderr.write_bytes(completed.stderr)
    go=json.loads(completed.stdout)
    record['cases'].append({'case_count':n,'certificate':before,'rust_report':rust_before,
                            'command':command,'exit_code':completed.returncode,'seconds':time.monotonic()-start,
                            'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go})
    save()
    assert completed.returncode==0 and not completed.stderr and go['verdict']=='accepted'
    assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
    assert seal(certificate)==before and seal(rust_path)==rust_before
    print(json.dumps({'case_count':n,'verdict':go['verdict']}),flush=True)
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:
    assert seal(row['path'])==row
record['status']='passed'
save()
