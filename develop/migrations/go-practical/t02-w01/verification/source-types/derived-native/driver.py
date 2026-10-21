"""Decode complete actual original Go type streams for Rust source checking."""
from pathlib import Path
import base64,hashlib,json,sys
OUT=Path(__file__).resolve().parent;SOURCE=Path(sys.argv[1])
check=json.loads((OUT/'formatted/go-oracle/result.json').read_bytes());assert check['status']=='passed'and check['environment']['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']==str(SOURCE)
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
run=OUT/'derived-native';run.mkdir(exist_ok=False);(run/'driver.py').write_bytes(Path(__file__).read_bytes())
inputs=[seal(SOURCE/f'source-types-{n:02}.json')for n in range(7)]
(run/'inputs.json').write_text(json.dumps({'actual_driver_prelaunch':seal(run/'driver.py'),'inputs':inputs},indent=2)+'\n')
target=OUT/'native-components';target.mkdir(exist_ok=False)
for n in range(7):
 observed=json.loads((SOURCE/f'source-types-{n:02}.json').read_bytes());case=target/f'case-{n:02}';case.mkdir();(case/'sources').mkdir()
 raw=base64.b64decode(observed['canonical_components'],validate=True);(case/'components.json').write_bytes(raw)
 for row in observed['sources']:
  obj=row['object'];raw=base64.b64decode(row['raw'],validate=True);assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
  (case/'sources'/(obj['id']+'.json')).write_text(json.dumps(obj,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n')
  (case/'sources'/(obj['id']+'.raw')).write_bytes(raw)
assert all(seal(r['path'])==r for r in inputs)
(run/'result.json').write_text(json.dumps({'status':'passed','actual_driver_prelaunch':seal(run/'driver.py'),'original_inputs':inputs,'outputs':[seal(p)for p in sorted(target.rglob('*'))if p.is_file()],'actual_Go_streams':7},indent=2)+'\n');print(json.dumps({'status':'passed','streams':7}))
