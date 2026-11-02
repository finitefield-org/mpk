"""Retain exact diagnostic source, rejected stream and compiled dependencies."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,time,sys
ROOT=Path(__file__).resolve().parent
epoch,source_epoch=sys.argv[1:] if len(sys.argv)>1 else ('fuel-diagnosis','heterogeneous')
out=ROOT/epoch;out.mkdir(exist_ok=False)
source=ROOT/'diagnose-prefix.rs'
certificate=next((ROOT/source_epoch/'certificates').glob('*.mpcert'))
profile=Path('target/debug/.fingerprint/mpk-vc-d9b46e74a77037b9/test-integration-test-go_practical_source_value_relations.json')
test=json.loads(profile.read_bytes())
index={}
for p in Path('target/debug/.fingerprint').glob('*/lib-*'):
    if p.suffix or not p.is_file():continue
    try:index[(p.name.removeprefix('lib-'),int.from_bytes(bytes.fromhex(p.read_text()),'little'))]=p
    except ValueError:pass
selected={};pending=[(n,h) for _,n,_,h in test['deps'] if n in ['mpk_kernel','mpk_cert']]
while pending:
    name,h=pending.pop()
    if name in selected:continue
    p=index[(name,h)];suffix=p.parent.name.rsplit('-',1)[1]
    matches=list(Path('target/debug/deps').glob('lib'+name+'-'+suffix+'.rlib'))
    assert len(matches)==1,(name,p)
    selected[name]=matches[0]
    meta=json.loads(p.with_suffix('.json').read_bytes())
    pending.extend((n,h) for _,n,_,h in meta['deps'] if (n,h) in index)
inputs=[Path(__file__),source,certificate,profile,*selected.values()]
def seal(p):
    p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
rows=[]
for i,p in enumerate(inputs):
    target=out/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());mode=p.stat().st_mode&0o777;target.chmod(mode)
    rows.append({**seal(p),'mode':mode,'complete_prelaunch_snapshot':seal(target)})
receipt=out/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n')
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
binary=out/'diagnosis'
compile=['rustc','--edition=2021',str(source),'-L','dependency=target/debug/deps','-o',str(binary)]
for name in ['mpk_cert','mpk_kernel']:compile+=['--extern',name+'='+str(selected[name])]
record={'status':'running','scope':'Diagnostic prefix localization only, preserving the full original rejected certificate; native core caps unchanged. No prefix counts as a normal row/source-U proof.','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'complete_prelaunch_inputs':seal(receipt),'commands':[compile],'tools':{n:seal(shutil.which(n)) for n in ['rustc']}}
result=out/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    code=subprocess.run(compile,stdout=stdout,stderr=stderr,check=False).returncode
    if code==0:
        record['compiled_diagnostic_binary_before_execution']=seal(binary)
        cmd=[str(binary),str(certificate)];record['commands'].append(cmd);result.write_text(json.dumps(record,indent=2)+'\n')
        code=subprocess.run(cmd,stdout=stdout,stderr=stderr,check=False).returncode
unchanged=all(seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} for r in rows)
record.update(status='passed' if code==0 and unchanged else 'failed',whole_inputs_unchanged=unchanged,exit_code=code,seconds=time.monotonic()-start,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record['status']));raise SystemExit(record['status']!='passed')
