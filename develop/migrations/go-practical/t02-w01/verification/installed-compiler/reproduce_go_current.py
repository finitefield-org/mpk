"""Reproduce the registered original Go producer from its exact Go/Rust-only epoch."""
from pathlib import Path
import json,hashlib,subprocess,sys,traceback
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
SOURCE=Path('/private/tmp/mpk-installed-compiler-205-go-source')
sys.dont_write_bytecode=True;sys.path.insert(0,str(SOURCE/'scripts'))
import release_bundles as original
out=ROOT/'reproduced-go-current';out.mkdir(exist_ok=False)
output=Path('/private/tmp/mpk-installed-compiler-205-original-go-output')
receipt=json.loads((ROOT/'original-go-source.json').read_bytes())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
assert receipt['source_files']==[seal(r['path']) for r in receipt['source_files']]
(out/'inputs.json').write_text(json.dumps({'source_receipt':seal(ROOT/'original-go-source.json'),'driver':seal(__file__)},indent=2)+'\n')
run=subprocess.run;calls=[]
def observed(*args,**kw):
 result=run(*args,**kw);n=len(calls);row={'command':args[0],'exit_code':result.returncode}
 for k in ['stdout','stderr']:
  p=out/f'{n:02d}-{k}.log';p.write_bytes(getattr(result,k) or b'');row[k]=seal(p)
 calls.append(row);(out/'commands.json').write_text(json.dumps(calls,indent=2)+'\n');return result
subprocess.run=observed
try:
 original.build_release_outputs(output)
 candidate=json.loads((REPO/'release/bundles/candidates/go.json').read_bytes())
 frontend=candidate['frontend_bundles'][0];actual=original.inventory(output/'frontend')
 assert actual==frontend['inventory']['files'], 'exact original registered frontend bytes'
 toolchain=candidate['toolchain_bundles'][0];assert original.inventory(output/'toolchain')==toolchain['inventory']['files']
 assert receipt['source_files']==[seal(r['path']) for r in receipt['source_files']]
 result={'status':'passed','source_epoch':receipt['basis_commit'],'frontend':seal(output/'frontend/bin/go2vir'),'toolchain_root':str(output/'toolchain'),'original_source_inputs_unchanged':True,'native_Linux_release_acceptance':False}
except BaseException as error:
 result={'status':'failed','error':repr(error)};(out/'failure.log').write_text(traceback.format_exc());(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');raise
finally:subprocess.run=run
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
