"""Retain and decode actual whole original native signature observations."""
from pathlib import Path
import base64,hashlib,json,sys
OUT=Path(__file__).resolve().parent;SOURCE=Path(sys.argv[1])
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
check=json.loads((OUT/'span-formatted/go-oracle/result.json').read_bytes());assert check['status']=='passed'and check['environment']['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']==str(SOURCE)
RUN=OUT/'derived-native';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
inputs=[seal(SOURCE/name)for name in ['signature-components.json','signature-observations.json']];(RUN/'inputs.json').write_text(json.dumps({'actual_driver_prelaunch':seal(RUN/'driver.py'),'inputs':inputs},indent=2)+'\n')
components=json.loads((SOURCE/'signature-components.json').read_bytes());observed=json.loads((SOURCE/'signature-observations.json').read_bytes());assert observed['runtime']=='go1.25.0'and observed['applications_executed']is False
wire=base64.b64decode(components['canonical_components'],validate=True);sources=[]
for row in components['sources']:
 obj=row['object'];raw=base64.b64decode(row['raw'],validate=True);assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];sources.append({'object':obj,'whole_original_bytes':list(raw)})
p=OUT/'corpus.json';assert not p.exists();p.write_text(json.dumps({'components':json.loads(wire),'sources':sources,'observations':observed},sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n')
assert all(seal(row['path'])==row for row in inputs)
(RUN/'result.json').write_text(json.dumps({'status':'passed','actual_driver_prelaunch':seal(RUN/'driver.py'),'original_inputs':inputs,'outputs':[seal(p)],'actual_function_signatures':len(observed['functions']),'actual_variable_bindings':len(observed['variables'])},indent=2)+'\n');print(json.dumps({'status':'passed','functions':len(observed['functions']),'variables':len(observed['variables'])}))
