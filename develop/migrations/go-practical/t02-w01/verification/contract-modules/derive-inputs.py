"""Keep original Go/sidecar bytes and separate native comparison observations."""
from pathlib import Path
import base64
import hashlib
import json

OUT = Path(__file__).resolve().parent
RUN = OUT/'derived-receivers'
RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
assert json.loads((OUT/'native-receivers/result.json').read_bytes())['status'] == 'passed'


def seal(path):
    raw = path.read_bytes()
    return {'path':str(path), 'bytes':len(raw), 'sha256':hashlib.sha256(raw).hexdigest(), 'mode':path.stat().st_mode & 0o777}


paths = sorted(p for p in (OUT/'native-receivers').rglob('*') if p.is_file())
before = [seal(p) for p in paths]
(RUN/'inputs.json').write_text(json.dumps(before,indent=2)+'\n')
native = OUT/'native-receivers/observations/contract-modules'
original = json.loads((native/'inputs.json').read_bytes())
raw = base64.b64decode(original['canonical_components'],validate=True)
wire = json.loads(raw)
assert len(wire['types']) == 1 and wire['types'][0]['name'] == 'bool' and not wire['declarations']
packet = {'components_hex':raw.hex(), 'sources':[
    {'object':s['object'], 'raw_hex':base64.b64decode(s['raw'],validate=True).hex()} for s in original['sources']]}
control = json.loads((native/'controls.json').read_bytes())
assert control['cases'] == 17 and control['original_sidecar_namespace_removed']
cases = []
for n in range(control['cases']):
    value = json.loads((native/f'case-{n:02}.json').read_bytes())
    for overlay in value['overlays']:
        overlay['raw_hex'] = base64.b64decode(overlay.pop('raw'),validate=True).hex()
    cases.append(value)
output = OUT/'corpus-receivers.json'
assert not output.exists()
output.write_text(json.dumps({'input':packet, 'cases':cases},ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n')
assert before == [seal(p) for p in paths]
record = {'status':'passed', 'actual_driver_prelaunch':seal(RUN/'driver.py'), 'source_receipt':seal(RUN/'inputs.json'),
          'original_files_unchanged':len(paths), 'original_bytes_unchanged':sum(r['bytes'] for r in before),
          'cases':len(cases), 'growth_cases':70, 'helpers':512, 'native_facts_passed_to_constructor':False, 'output':seal(output)}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','original_files_unchanged','cases']}))
