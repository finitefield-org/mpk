"""Combine exact original cases; no source, candidate or outcome is regenerated."""
from pathlib import Path
import hashlib,json,sys
OUT=Path(__file__).resolve().parent;SOURCE=Path(sys.argv[1]);RUN=OUT/'derived-indices-native';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
check=json.loads((OUT/'indices-formatted/go-oracle/result.json').read_bytes());assert check['status']=='passed'and check['environment']['MPK_GO_VALUE_TYPING_NATIVE_OUTPUT']==str(SOURCE)
files=sorted(p for p in SOURCE.rglob('*')if p.is_file());inputs=[seal(p)for p in files];(RUN/'inputs.json').write_text(json.dumps({'actual_driver_prelaunch':seal(RUN/'driver.py'),'original_inputs':inputs},indent=2)+'\n')
summary=json.loads((SOURCE/'summary.json').read_bytes());assert summary['cases']==944 and summary['runtime']=='go1.25.0'and summary['applications_executed']is False
cases=[]
for n in range(summary['cases']):
 case=SOURCE/f'case-{n:05d}';cases.append({'input':json.loads((case/'input.json').read_bytes()),'native':json.loads((case/'native.json').read_bytes())})
p=OUT/'corpus-indices.json';assert not p.exists();p.write_text(json.dumps(cases,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n');assert all(seal(r['path'])==r for r in inputs)
(RUN/'result.json').write_text(json.dumps({'status':'passed','actual_driver_prelaunch':seal(RUN/'driver.py'),'original_inputs':inputs,'output':seal(p),'actual_cases':summary['cases']},indent=2)+'\n');print(json.dumps({'status':'passed','cases':summary['cases'],'bytes':p.stat().st_size}))
