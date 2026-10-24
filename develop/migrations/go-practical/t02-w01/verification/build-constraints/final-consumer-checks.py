"""After the test-only repair, check only its changed consumer and Python harnesses."""
from pathlib import Path
import hashlib,json,os,subprocess,time
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def main():
    out=ROOT/'final-consumer-checks';out.mkdir(exist_ok=False)
    previous=ROOT/'standalone/inputs.json';inputs=json.loads(previous.read_bytes())
    for row in inputs['sources']:assert row==seal(Path(row['path']))
    before=[seal(Path(row['path'])) for row in inputs['sources']]
    commands=[['rustfmt','--edition=2021','--check','--config','skip_children=true','crates/mpk-vc/tests/go_practical_source_build.rs'],['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_source_build','--','-D','warnings']]
    env=os.environ.copy();env.update({'CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0'})
    pre={'driver':seal(Path(__file__).resolve()),'complete_current_source_epoch':seal(previous),'sources':before,'commands':commands,'selection_reason':'Only the direct test reader changed after all production checks passed. Check that consumer format and strict Clippy; its two tests run separately at a new absolute path from zero original snapshots. Preserve earlier13 unaffected Rust roots and both unchanged Go oracle roots without repeating them.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out/'prelaunch.json').write_text(json.dumps(pre,indent=2)+'\n');start=time.monotonic();runs=[]
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        for command in commands:
            code=subprocess.run(command,cwd=REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
            runs.append({'command':command,'exit_code':code})
            if code:break
    unchanged=before==[seal(Path(row['path'])) for row in inputs['sources']]
    for path in ROOT.glob('*.py'):compile(path.read_bytes(),str(path),'exec')
    result={'status':'passed' if all(r['exit_code']==0 for r in runs) and len(runs)==2 and unchanged else 'failed','seconds':time.monotonic()-start,'prelaunch':seal(out/'prelaunch.json'),'runs':runs,'consumed_sources_unchanged':unchanged,'stdout':seal(out/'stdout.log'),'stderr':seal(out/'stderr.log'),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'seconds':result['seconds'],'checks':len(runs)}));raise SystemExit(result['status']!='passed')
if __name__=='__main__':main()
