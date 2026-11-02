"""Pair each immutable completed Rust row without restarting the live Rust run.

Common Go inputs and each complete row are preserved BEFORE its native Go
invocation. The evolving Rust result is observed, never consumed by a core;
its terminal result is recorded only after the original run actually ends.
"""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import sys
import time
ROOT=Path(__file__).resolve().parent

def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}

epoch,group=sys.argv[1:]
assert epoch.replace('-','').isalnum() and group.replace('-','').isalnum()
original=ROOT/epoch;state_path=original/'result.json';initial=json.loads(state_path.read_bytes())
assert initial['status'] in ['running','passed']
assert seal(initial['source_receipt']['path'])==initial['source_receipt']
prior_path=ROOT.parent/'core-interfaces/paired/result.json';prior=json.loads(prior_path.read_bytes());binary=prior['go_binary']
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:assert seal(row['path'])==row
metadata=next((original/'certificates').glob('*-nil.sources.json'))
meta=json.loads(metadata.read_bytes());assert meta['unit']=='seventy-cases' and len(meta['case_tags'])==70
stems=[f'seventy-cases-{id}' for id in ['nil',*[r[0] for r in meta['case_tags']]]]
out=ROOT/group;out.mkdir(exist_ok=False)
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
observed=out/'rust-run-state-before-first-pair.json';observed.write_text(json.dumps(initial,indent=2)+'\n')
receipts=[];saved=set();receipt=out/'prelaunch-inputs.json'
def preserve(paths):
    for path in paths:
        path=Path(path)
        if path in saved:continue
        raw=path.read_bytes();mode=path.stat().st_mode&0o777
        target=out/'inputs'/f'{len(receipts):03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(raw);target.chmod(mode)
        receipts.append({**seal(path),'mode':mode,'complete_prelaunch_snapshot':seal(target)})
        saved.add(path)
    receipt.write_text(json.dumps(receipts,indent=2)+'\n')
preserve([Path(binary['path']),Path(__file__),prior_path,Path(initial['source_receipt']['path']),observed,metadata,*[Path(r['path']) for r in prior['go_production_sources']]])
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'complete_prelaunch_inputs':seal(receipt),'original_Rust_executions':[],'go_binary':binary,'go_production_sources':prior['go_production_sources'],'actual_unchanged_Go_build_evidence':seal(prior_path),'Rust_state_before_first_pair':seal(observed),'per_row_prelaunch_receipts':[],'cases':[],'scope':'Each original completed complete Rust row is paired exactly once, with all common Go sources/binary and that row certificate/report/metadata saved before its native Go invocation. No incomplete certificate is consumed. The original live Rust test is never restarted or changed; its evolving execution result is observed only, then terminal provenance is recorded separately. Original same-U whole source-value relation/nil/copy/boxing mathematics only; no source validity or canonical context/program authority.'}
def save():
    record['complete_prelaunch_inputs']=seal(receipt)
    (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
def ready(stem):
    path=original/'certificates'/f'{stem}.mpcert'
    while not path.with_suffix('.rust.json').exists():
        state=json.loads(state_path.read_bytes());assert state['status']!='failed'
        if state['status']=='passed':raise AssertionError(('terminal original missing row',stem))
        time.sleep(1)
    return path
save()
try:
    for stem in stems:
        path=ready(stem);report_path=path.with_suffix('.rust.json');meta_path=path.with_suffix('.sources.json')
        rust=json.loads(report_path.read_bytes());assert rust['verdict']=='accepted' and rust['axiom_count']==0
        before,report_before=seal(path),seal(report_path)
        preserve([path,report_path,meta_path])
        row_receipt=out/f'{stem}.prelaunch.json'
        row_receipt.write_text(json.dumps({'common_and_completed_rows':seal(receipt),'certificate':before,'rust_report':report_before,'metadata':seal(meta_path),'saved_before_native_Go_invocation':True},indent=2)+'\n')
        record['per_row_prelaunch_receipts'].append(seal(row_receipt));save()
        command=[binary['path'],'verify',str(path)];start=time.monotonic();run=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
        stdout,stderr=out/f'{stem}.stdout.log',out/f'{stem}.stderr.log';stdout.write_bytes(run.stdout);stderr.write_bytes(run.stderr);go=json.loads(run.stdout)
        record['cases'].append({'name':stem,'certificate':before,'rust_report':report_before,'prelaunch_row_receipt':seal(row_receipt),'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go});save()
        assert run.returncode==0 and not run.stderr and go['verdict']=='accepted' and not rust['axiom_report']['entries']
        assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
        assert seal(path)==before and seal(report_path)==report_before
        print(json.dumps({'name':stem,'accepted':True,'seconds':record['cases'][-1]['seconds']}),flush=True)
    while True:
        final=json.loads(state_path.read_bytes())
        if final['status']!='running':break
        time.sleep(1)
    assert final['status']=='passed' and final['whole_source_inputs_unchanged']
    assert final['source_receipt']==initial['source_receipt']
    assert {p.stem for p in (original/'certificates').glob('*.mpcert')}==set(stems)
    for row in receipts:
        assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']}
        assert Path(row['path']).stat().st_mode&0o777==row['mode']
        assert seal(row['complete_prelaunch_snapshot']['path'])==row['complete_prelaunch_snapshot']
    record.update(status='passed',same_byte_streams=len(stems),zero_axiom_acceptances=len(stems),whole_inputs_unchanged=True,original_Rust_executions=[seal(state_path)],Rust_terminal_result_observed_after_pairing=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
except BaseException:
    record['status']='failed';save();raise
