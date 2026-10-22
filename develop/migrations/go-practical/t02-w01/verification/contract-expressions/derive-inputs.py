"""Extract original source buffers; keep native observations comparison-only.

The frozen historical scope is not an input to the Rust constructor. Its70
source-expression witnesses are extracted with one explicit package renaming.
"""
from pathlib import Path
import base64
import hashlib
import json

OUT = Path(__file__).resolve().parent
RUN = OUT/'derived'
RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
native = OUT/'native/observations/contract-expressions'
fixed = Path('develop/migrations/go-practical/t01-w04/contract-wire/expression-witnesses.json')


def seal(path):
    raw = path.read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777}


paths = sorted(p for p in (OUT/'native').rglob('*') if p.is_file()) + [fixed]
before = [seal(p) for p in paths]
(RUN/'inputs.json').write_text(json.dumps(before, indent=2)+'\n')
inputs = json.loads((native/'inputs.json').read_bytes())
components = base64.b64decode(inputs['canonical_components'], validate=True)
wire = json.loads(components)
assert len(wire['types']) == 1 and wire['types'][0]['name'] == 'bool' and not wire['declarations']
packet = {'components_hex': components.hex(), 'sources': [
    {'object': row['object'], 'raw_hex': base64.b64decode(row['raw'], validate=True).hex()}
    for row in inputs['sources']]}
cases = []
for n in range(4):
    row = json.loads((native/f'overlay-{n:02}.json').read_bytes())
    row['raw_hex'] = base64.b64decode(row.pop('raw'), validate=True).hex()
    cases.append(row)
assert json.loads((native/'controls.json').read_bytes())['original_sidecar_namespace_removed']
corpus = OUT/'corpus.json'
corpus.write_text(json.dumps({'input': packet, 'cases': cases}, ensure_ascii=False, sort_keys=True, separators=(',', ':'))+'\n')


def rename(v):
    if isinstance(v, dict):
        return {k: rename(x) for k, x in v.items()}
    if isinstance(v, list):
        return [rename(x) for x in v]
    if isinstance(v, str) and v.startswith('fixture/policy.'):
        return 'example.com/policy.' + v.removeprefix('fixture/policy.')
    return v


witnesses = json.loads(fixed.read_bytes())['witnesses']
assert len(witnesses) == 70 and len({w['op'] for w in witnesses}) == 70
source_only = [{'op': w['op'], 'phase': w['phase'], 'ordinal': w['ordinal'], 'source': rename(w['source']),
                'expected_type_kind': w['wire']['type']['kind'],
                'expected_logical_type': w['wire']['type'].get('name')} for w in witnesses]
output = OUT/'source-witnesses.json'
output.write_text(json.dumps({'original': seal(fixed), 'transformation': 'Extract all70 source witnesses, phase/ordinal and logical-vs-source result kind; rename fixture/policy. to example.com/policy. No historical scope/native type IDs are constructor inputs.', 'witnesses': source_only}, ensure_ascii=False, sort_keys=True, separators=(',', ':'))+'\n')
assert before == [seal(p) for p in paths]
result = {'status': 'passed', 'actual_driver_prelaunch': seal(RUN/'driver.py'), 'source_receipt': seal(RUN/'inputs.json'),
          'original_files_unchanged': len(paths), 'cases': 4, 'witnesses': 70, 'outputs': [seal(corpus), seal(output)]}
(RUN/'result.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps({k: result[k] for k in ['status', 'original_files_unchanged', 'cases', 'witnesses']}))
