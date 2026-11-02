"""Pair all complete original control streams, retaining the first setup failure, preserving all Go inputs first."""
from pathlib import Path
import sys
import datetime
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
for row in prior['go_production_sources']:assert seal(row['path'])==row
files=[]
executions=[]
group, = sys.argv[1:]
assert group.replace('-', '').isalnum()
for epoch in ['controls','controls-small-fixed']:
    path=ROOT/epoch/'result.json';run=json.loads(path.read_bytes())
    assert run['whole_source_inputs_unchanged']
    if epoch=='controls':
        assert run['status']=='failed' and run['exit_code']==101
        stdout=(ROOT/epoch/'stdout.log').read_text();stderr=(ROOT/epoch/'stderr.log').read_text()
        assert 'heterogeneous_relation_observes_string_bytes_slice_owner_and_bool ... ok' in stdout
        assert 'last_large_relation_keeps_first_inactive_payload ... ok' in stdout
        assert '2 passed; 1 failed' in stdout and 'NotFound' in stderr
        assert {p.stem for p in (ROOT/epoch/'certificates').glob('*.mpcert')}=={'source-bool-zero-is-true','string-bytes-replaced-by-length-observation','slice-owner-observation-erased','last-large-wrong-inactive-copy-relation'}
    else:
        assert run['status']=='passed'

    assert seal(run['source_receipt']['path'])==run['source_receipt']
    executions.append(seal(path))
    files.extend(sorted((ROOT/epoch/'certificates').glob('*.mpcert')))
assert files and len({p.name for p in files})==len(files)
assert {p.stem for p in files}=={'nonzero-source-byte-related-to-zero','arbitrary-copy-related-to-nil','related-box-is-nil','boxed-relation-forces-wrong-active-value','constant-byte-view-is-only-math','source-bool-zero-is-true','string-bytes-replaced-by-length-observation','slice-owner-observation-erased','last-large-wrong-inactive-copy-relation'}
out=ROOT/group;out.mkdir(exist_ok=False)
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
inputs=[Path(binary['path']),Path(__file__),prior_path]
inputs.extend(Path(r['path']) for r in prior['go_production_sources'])
for row in executions:
    path=Path(row['path']);inputs.append(path)
    inputs.append(Path(json.loads(path.read_bytes())['source_receipt']['path']))
    inputs.extend([path.with_name('stdout.log'),path.with_name('stderr.log')])
for p in files:
    inputs.extend([p,p.with_suffix('.rust.json')])
    if p.with_suffix('.sources.json').exists():inputs.append(p.with_suffix('.sources.json'))
receipts=[]
for i,path in enumerate(sorted(set(inputs))):
    raw=path.read_bytes();mode=path.stat().st_mode&0o777
    target=out/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True)
    target.write_bytes(raw);target.chmod(mode)
    receipts.append({**seal(path),'mode':mode,'complete_prelaunch_snapshot':seal(target)})
receipt=out/'prelaunch-inputs.json';receipt.write_text(json.dumps(receipts,indent=2)+'\n')
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'complete_prelaunch_inputs':seal(receipt),'original_Rust_executions':executions,'original_failed_suite_retained':True,'failed_suite_reason':'First control root lacked its positive entry fixture; the other two complete roots passed. Only the first affected root is rerun after the nil-only setup fix.','go_binary':binary,'go_production_sources':prior['go_production_sources'],'actual_unchanged_Go_build_evidence':seal(prior_path),'cases':[],'scope':'Whole original source-specific nil/copy certificate streams, with all tags, active payloads and inactive zeros. Every original input and the unchanged Go binary/source closure is captured before checking. Private source-key boxing is not source-runtime validity or canonical context/program authority. No arbitrary-input validity/canonical capture/context/program/release claim.'}


def save():
    (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')


save()
try:
    for path in files:
        report_path=path.with_suffix('.rust.json');rust=json.loads(report_path.read_bytes())
        before,report_before=seal(path),seal(report_path)
        command=[binary['path'],'verify',str(path)];start=time.monotonic()
        result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
        stdout,stderr=out/f'{path.stem}.stdout.log',out/f'{path.stem}.stderr.log'
        stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr);go=json.loads(result.stdout)
        record['cases'].append({'name':path.stem,'certificate':before,'rust_report':report_before,'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go});save()
        assert not result.stderr
        if path.stem=='constant-byte-view-is-only-math':
            assert result.returncode==0 and rust['verdict']==go['verdict']=='accepted'
            assert rust['axiom_count']==0 and not rust['axiom_report']['entries']
            assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
        else:
            assert result.returncode!=0 and rust['verdict']==go['verdict']=='rejected'
            assert rust['error_code']=='KERNEL_CORE_CHECK' and go['error_kind']=='core_check'
            detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH'
            assert detail['details']['kind'] in ['check_type_mismatch','lambda_domain_mismatch']
        assert seal(path)==before and seal(report_path)==report_before
        print(json.dumps({'name':path.stem,'verdict':go['verdict'],'seconds':record['cases'][-1]['seconds']}),flush=True)
    for row in receipts:
        assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']}
        assert Path(row['path']).stat().st_mode&0o777==row['mode']
        assert seal(row['complete_prelaunch_snapshot']['path'])==row['complete_prelaunch_snapshot']
    record.update(status='passed',same_byte_streams=len(files),zero_axiom_acceptances=sum(x['rust_observation']['verdict']=='accepted' for x in record['cases']),genuine_type_rejections=sum(x['rust_observation']['verdict']=='rejected' for x in record['cases']),whole_inputs_unchanged=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
except BaseException:
    record['status']='failed';save();raise
