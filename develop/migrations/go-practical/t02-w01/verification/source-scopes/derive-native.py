"""Decode actual saved Go observations into explicit Rust input files."""
from pathlib import Path
import base64,hashlib,json,sys
OUT=Path(__file__).resolve().parent
SOURCE=Path(sys.argv[1])
check=json.loads((OUT/'dot-corrected/go-oracle/result.json').read_bytes())
assert check['status']=='passed'and check['environment']['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']==str(SOURCE)
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
run=OUT/'derived-native';run.mkdir(exist_ok=False);(run/'driver.py').write_bytes(Path(__file__).read_bytes())
inputs=[seal(SOURCE/name)for name in ['scope-components.json','scope-uses.json']]
(run/'inputs.json').write_text(json.dumps({'actual_driver_prelaunch':seal(run/'driver.py'),'inputs':inputs},indent=2)+'\n')
observed=json.loads((SOURCE/'scope-components.json').read_bytes())
canonical=base64.b64decode(observed['canonical_components'],validate=True)
wire=json.loads(canonical);raws={}
target=OUT/'native-components';target.mkdir(exist_ok=False);(target/'sources').mkdir()
(target/'components.json').write_bytes(canonical)
for row in observed['sources']:
 obj=row['object'];raw=base64.b64decode(row['whole_original_bytes'],validate=True)
 assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
 raws[obj['id']]=raw
 (target/'sources'/(obj['id']+'.json')).write_text(json.dumps(obj,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n')
 (target/'sources'/(obj['id']+'.raw')).write_bytes(raw)
uses_raw=(SOURCE/'scope-uses.json').read_bytes();uses=json.loads(uses_raw)['uses']
for row in uses:
 assert raws[row['source_object_id']][row['start_byte']:row['end_byte']]==row['name'].encode()
 if row['definition_source_object_id']is not None:assert 0<=row['definition_start_byte']<len(raws[row['definition_source_object_id']])
(target/'uses.json').write_bytes(uses_raw)
record={'status':'passed','actual_driver_prelaunch':seal(run/'driver.py'),'original_inputs':inputs,'whole_original_source_objects':len(raws),'actual_lexical_uses':len(uses),'outputs':[seal(p)for p in sorted(target.rglob('*'))if p.is_file()],'source_typing_or_context_or_program_authority':False}
(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
assert all(seal(row['path'])==row for row in inputs)
print(json.dumps({'status':'passed','source_objects':len(raws),'uses':len(uses)}))
