"""Aggregate complete actual native input/output pairs without regeneration."""
from pathlib import Path
import hashlib,json,sys
OUT=Path(__file__).resolve().parent
SOURCE=Path(sys.argv[1]);CHECK=Path(sys.argv[2])
check=json.loads(CHECK.read_bytes());assert check['status']=='passed'and check['environment']['MPK_GO_CONSTANT_NATIVE_OUTPUT']==str(SOURCE)
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
run=OUT/'derived-native';run.mkdir(exist_ok=False);(run/'driver.py').write_bytes(Path(__file__).read_bytes())
inputs=[seal(p)for p in sorted(SOURCE.rglob('*'))if p.is_file()]
(run/'inputs.json').write_text(json.dumps({'actual_driver_prelaunch':seal(run/'driver.py'),'inputs':inputs},indent=2)+'\n')
summary=json.loads((SOURCE/'summary.json').read_bytes());cases=[]
for p in sorted(SOURCE.glob('case-*')):
 a=json.loads((p/'input.json').read_bytes());b=json.loads((p/'native.json').read_bytes());assert a['index']==len(cases)
 for source in a['sources']:
  raw=bytes.fromhex(source['raw_hex']);obj=source['object'];assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
 cases.append({'input':a,'native':b})
assert len(cases)==summary['cases']
target=OUT/'corpus.json';assert not target.exists();target.write_text(json.dumps(cases,ensure_ascii=False,separators=(',',':'))+'\n')
assert all(seal(row['path'])==row for row in inputs)
(run/'result.json').write_text(json.dumps({'status':'passed','actual_driver_prelaunch':seal(run/'driver.py'),'original_inputs':inputs,'corpus':seal(target),'summary':summary},indent=2)+'\n')
print(json.dumps(summary))
