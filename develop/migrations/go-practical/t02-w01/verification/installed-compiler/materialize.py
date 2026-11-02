from pathlib import Path
import sys,json,traceback
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
sys.dont_write_bytecode=True;sys.path.insert(0,str(REPO/'scripts'))
import successor_release_bundles as release
out=ROOT/'materialize-original';out.mkdir(exist_ok=False)
work=Path('/private/tmp/mpk-installed-compiler-205-original');work.mkdir(exist_ok=False)
try:
 registry,candidates=release.validate_release_models()
 roots=release.build_roots(work,candidates)
 result={'status':'passed','roots':{k:str(v) for k,v in roots.items()},'registry_sha256':registry['registry_sha256']}
except BaseException as error:
 result={'status':'failed','error':repr(error)}
 (out/'failure.log').write_text(traceback.format_exc())
 (out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps(result));raise
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
