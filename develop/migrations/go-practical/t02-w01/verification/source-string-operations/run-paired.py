"""Pair complete terminal original Rust streams, preserving all Go inputs first."""
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
epoch, group, *reuse_groups = sys.argv[1:]
reused={}
reuse_inputs=[]
for old_group in reuse_groups:
    assert old_group.replace('-', '').isalnum()
    old_path=ROOT/old_group/'result.json';old=json.loads(old_path.read_bytes())
    assert old['status']=='passed' and old['whole_inputs_unchanged'] and old['go_binary']==binary and old['go_production_sources']==prior['go_production_sources']
    reuse_inputs.append(old_path)
    for case in old['cases']:
        assert case['name'] not in reused
        reused[case['name']]=(case,old_path)
        reuse_inputs.extend([Path(case['stdout']['path']),Path(case['stderr']['path'])])
assert group.replace('-', '').isalnum()
for epoch in [epoch]:
    path=ROOT/epoch/'result.json';run=json.loads(path.read_bytes())
    assert run['status']=='passed' and run['whole_source_inputs_unchanged']
    assert seal(run['source_receipt']['path'])==run['source_receipt']
    executions.append(seal(path))
    files.extend(sorted((ROOT/epoch/'certificates').glob('*.mpcert')))
assert files and len({p.name for p in files})==len(files)
out=ROOT/group;out.mkdir(exist_ok=False)
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
inputs=[Path(binary['path']),Path(__file__),prior_path,*reuse_inputs]
inputs.extend(Path(r['path']) for r in prior['go_production_sources'])
for row in executions:
    path=Path(row['path']);inputs.append(path)
    inputs.append(Path(json.loads(path.read_bytes())['source_receipt']['path']))
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
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'complete_prelaunch_inputs':seal(receipt),'original_Rust_executions':executions,'go_binary':binary,'go_production_sources':prior['go_production_sources'],'actual_unchanged_Go_build_evidence':seal(prior_path),'cases':[],'scope':'Whole original source-specific bounded string certificate streams, with all tags, active payloads and inactive zeros. Every original input and the unchanged Go binary/source closure is captured before checking. Private source-key boxing is not source-runtime validity or canonical context/program authority. No arbitrary-input validity/canonical capture/context/program/release claim.'}


def save():
    (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')


save()
try:
    for path in files:
        report_path=path.with_suffix('.rust.json');rust=json.loads(report_path.read_bytes())
        before,report_before=seal(path),seal(report_path)
        command=[binary['path'],'verify',str(path)];start=time.monotonic()
        if path.stem in reused:
            from types import SimpleNamespace
            previous,previous_path=reused[path.stem]
            assert previous['certificate']['sha256']==before['sha256'] and previous['certificate']['bytes']==before['bytes'] and previous['rust_observation']==rust
            assert seal(previous['stdout']['path'])==previous['stdout'] and seal(previous['stderr']['path'])==previous['stderr']
            result=SimpleNamespace(stdout=Path(previous['stdout']['path']).read_bytes(),stderr=Path(previous['stderr']['path']).read_bytes(),returncode=previous['exit_code'])
            command=previous['command'];reused_receipt=seal(previous_path);seconds=previous['seconds']
        else:
            result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
            reused_receipt=None;seconds=time.monotonic()-start
        stdout,stderr=out/f'{path.stem}.stdout.log',out/f'{path.stem}.stderr.log'
        stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr);go=json.loads(result.stdout)
        record['cases'].append({'name':path.stem,'certificate':before,'rust_report':report_before,'command':command,'exit_code':result.returncode,'seconds':seconds,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go,'reused_exact_terminal_execution':reused_receipt});save()
        assert not result.stderr
        if rust['verdict']=='accepted':
            assert result.returncode==0 and go['verdict']=='accepted'
            assert rust['axiom_count']==0 and not rust['axiom_report']['entries']
            assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
        else:
            assert result.returncode!=0 and rust['verdict']==go['verdict']=='rejected'
            assert rust['error_code']=='KERNEL_CORE_CHECK' and go['error_kind']=='core_check'
            assert json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH'
        assert seal(path)==before and seal(report_path)==report_before
        print(json.dumps({'name':path.stem,'verdict':go['verdict'],'seconds':record['cases'][-1]['seconds']}),flush=True)
    for row in receipts:
        assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']}
        assert Path(row['path']).stat().st_mode&0o777==row['mode']
        assert seal(row['complete_prelaunch_snapshot']['path'])==row['complete_prelaunch_snapshot']
    record.update(status='passed',same_byte_streams=len(files),zero_axiom_acceptances=sum(r['rust_observation']['verdict']=='accepted' for r in record['cases']),genuine_type_rejections=sum(r['rust_observation']['verdict']=='rejected' for r in record['cases']),whole_inputs_unchanged=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
except BaseException:
    record['status']='failed';save();raise
