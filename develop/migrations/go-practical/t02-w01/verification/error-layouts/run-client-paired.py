"""Pair the actual named-carrier client and exact-ownership boundary examples."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import time

ROOT=Path(__file__).resolve().parent


def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


rust_path=ROOT/'carrier-client/result.json'
rust_run=json.loads(rust_path.read_bytes())
assert rust_run['status']=='passed' and rust_run['whole_source_inputs_unchanged']
inputs_path=Path(rust_run['source_receipt']['path'])
assert seal(inputs_path)==rust_run['source_receipt']
sources=json.loads(inputs_path.read_bytes())['sources']
for row in sources:
    assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']}
prior_path=ROOT.parent/'core-interfaces/paired/result.json'
prior=json.loads(prior_path.read_bytes())
binary=prior['go_binary']
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:
    assert seal(row['path'])==row
out=ROOT/'carrier-client-paired'
out.mkdir(exist_ok=False)
driver=out/'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
record={'status':'running','actual_driver_prelaunch':seal(driver),
        'terminal_Rust_execution':seal(rust_path),'complete_Rust_inputs':seal(inputs_path),
        'go_binary':binary,'go_production_sources':prior['go_production_sources'],
        'actual_unchanged_Go_build_evidence':seal(prior_path),'cases':[],
        'scope':'Mathematical carrier-name definitions and actual typed client only. Both accepted alternatives fail exact ownership; no source/profile/program authority.'}


def save():
    (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')


save()
try:
    cases={'honest-named-carrier-client':True,'unreferenced-carrier-is-only-math':True,
           'empty-carrier-extra-slot-client':False}
    files=sorted((ROOT/'carrier-client/certificates').glob('*.mpcert'))
    assert {p.stem for p in files}==set(cases)
    for certificate in files:
        name=certificate.stem
        report_path=certificate.with_suffix('.rust.json')
        before,report_before=seal(certificate),seal(report_path)
        rust=json.loads(report_path.read_bytes())
        command=[binary['path'],'verify',str(certificate)]
        start=time.monotonic()
        result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
        stdout,stderr=out/(name+'.stdout.log'),out/(name+'.stderr.log')
        stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
        go=json.loads(result.stdout)
        record['cases'].append({'name':name,'certificate':before,'rust_report':report_before,
            'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,
            'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go})
        save()
        assert seal(certificate)==before and seal(report_path)==report_before and not result.stderr
        if cases[name]:
            assert result.returncode==0 and rust['verdict']==go['verdict']=='accepted'
            assert rust['axiom_count']==0 and not rust['axiom_report']['entries']
            assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
            assert go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
        else:
            assert result.returncode!=0 and rust['verdict']==go['verdict']=='rejected'
            assert rust['error_code']=='KERNEL_CORE_CHECK' and go['error_kind']=='core_check'
            detail=json.loads(rust['error_detail'])
            assert detail['code']=='CORE_TYPE_MISMATCH'
            if detail['details']['kind']=='check_type_mismatch':
                inferred,expected=[detail['details'][k] for k in ['inferred_kind','expected_kind']]
                assert re.fullmatch(r'term [0-9]+ inferred '+re.escape(inferred)+' but expected '+re.escape(expected),go['error_detail'])
            else:
                assert detail['details']['kind']=='lambda_domain_mismatch' and go['error_detail']=='lambda domain mismatch'
        print(json.dumps({'name':name,'verdict':go['verdict']}),flush=True)
    for row in sources:
        assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']}
    assert seal(binary['path'])==binary
    for row in prior['go_production_sources']:
        assert seal(row['path'])==row
    record.update(status='passed',same_byte_streams=3,zero_axiom_mathematical_acceptances=2,genuine_core_type_mismatches=1)
    save()
except BaseException:
    record['status']='failed';save();raise
