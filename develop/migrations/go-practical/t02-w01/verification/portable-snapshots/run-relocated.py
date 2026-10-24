from pathlib import Path
import datetime,hashlib,json,shutil,subprocess,time
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
SOURCE=REPO/'develop/migrations/go-practical/t02-w01/verification/snapshot-inputs'
TARGET=Path('/private/tmp/mpk-portable-snapshot-183-fixed')
def seal(path):
    raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def main():
    out=ROOT/'relocated';out.mkdir(exist_ok=False);TARGET.mkdir(exist_ok=False)
    files=[SOURCE/'preserve-originals.py',*sorted((SOURCE/'snapshot-storage').glob('*'))]
    originals=[seal(p) for p in files]
    shutil.copy2(files[0],TARGET/'preserve-originals.py')
    shutil.copytree(SOURCE/'snapshot-storage',TARGET/'snapshot-storage',copy_function=shutil.copy2)
    pre={'basis_commit':'2e1330a6dc4e155399f836002491fd1e2dc13200','command':['python3','-B',str(TARGET/'preserve-originals.py'),'--restore'],'cwd':str(TARGET),'driver':seal(Path(__file__).resolve()),'inputs':originals,'source_scope':'All original storage parts and the entire immutable77162-file snapshot manifest, with no missing original member or rewritten receipt.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out/'prelaunch.json').write_text(json.dumps(pre,indent=2)+'\n')
    start=time.monotonic()
    with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
        code=subprocess.run(pre['command'],cwd=TARGET,stdout=stdout,stderr=stderr,check=False).returncode
    unchanged=originals==[seal(p) for p in files]
    result={'status':'passed' if code==0 and unchanged else 'failed','exit_code':code,'seconds':time.monotonic()-start,'command':pre['command'],'prelaunch':seal(out/'prelaunch.json'),'stdout':seal(out/'stdout.log'),'stderr':seal(out/'stderr.log'),'consumed_inputs_unchanged':unchanged,'relocated_manifest_byte_identical':(TARGET/'snapshot-storage/manifest.json').read_bytes()==(SOURCE/'snapshot-storage/manifest.json').read_bytes(),'whole_T_gate_run':False}
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged']}));raise SystemExit(result['status']!='passed')
if __name__=='__main__':main()
