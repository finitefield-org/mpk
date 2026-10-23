"""Apply exactly the isolated tested sources after original Rust consumers end."""
from pathlib import Path
import datetime,hashlib,json,subprocess,time
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
copy=Path(json.loads((ROOT/'isolated-copy.json').read_bytes())['copy'])
original=json.loads((ROOT/'final-large/inputs.json').read_bytes())['sources']
changed=json.loads((ROOT/'isolated-copy.json').read_bytes())['overlaid_sources']
while True:
    path=ROOT/'producer-replay/result.json'
    if path.exists():
        replay=json.loads(path.read_bytes())
        if replay['status']!='running':break
    time.sleep(1)
assert replay['status']=='passed' and replay['whole_source_inputs_unchanged']
large=json.loads((ROOT/'final-large/result.json').read_bytes());assert large['status']=='passed' and large['whole_source_inputs_unchanged']
latest=json.loads((ROOT/'producer-replay/inputs.json').read_bytes())['sources']
def seal(p):
    p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
for name in changed:
    before=next(r for r in original if r['path']==name)
    after=next(r for r in latest if r['path']==name)
    assert seal(name)['sha256']==before['sha256'] and Path(name).stat().st_mode&0o777==before['mode'],('concurrent original source edit',name)
    assert seal(copy/name)['sha256']==after['sha256'] and (copy/name).stat().st_mode&0o777==after['mode']
out=ROOT/'source-application';out.mkdir(exist_ok=False)
(out/'driver.py').write_bytes(Path(__file__).read_bytes())
rows=[]
for name in changed:
    source=Path(name);target=copy/name
    for label,path in [('before',source),('tested-after',target)]:
        saved=out/label/name;saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(path.read_bytes());saved.chmod(path.stat().st_mode&0o777)
    rows.append({'path':name,'before':seal(source),'tested_after':seal(target),'before_mode':source.stat().st_mode&0o777,'after_mode':target.stat().st_mode&0o777})
receipt={'status':'prepared_exact_tested_source_application','original_Rust_terminal':seal(ROOT/'final-large/result.json'),'isolated_replay_terminal':seal(ROOT/'producer-replay/result.json'),'actual_driver':seal(out/'driver.py'),'sources':rows,'Go_streaming_inputs_mutated':False,'reason':'Original Rust source consumers have ended. Any continuing Go checker reads only its unchanged binary/13 Go sources, accepted certificate/report/metadata and immutable receipt JSON; these four Rust files are not its inputs.'}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
for name in changed:
    Path(name).write_bytes((copy/name).read_bytes());Path(name).chmod((copy/name).stat().st_mode&0o777)
for row in rows:assert seal(row['path'])['sha256']==row['tested_after']['sha256']
receipt.update(status='applied_exact_isolated_tested_source_bytes',finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'status':receipt['status'],'sources':len(rows)}),flush=True)
for epoch,mode in [('format-final','format'),('lint','clippy')]:
    command=['python3',str(ROOT/'run-complete.py'),epoch,mode]
    code=subprocess.run(command,check=False).returncode
    assert code==0,(epoch,code)
