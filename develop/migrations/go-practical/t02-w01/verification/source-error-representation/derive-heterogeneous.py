"""Derive only original buffers; native AST/type observations stay comparison data."""
from pathlib import Path
import base64,hashlib,json
root=Path(__file__).resolve().parent
run=json.loads((root/'native-corrected/result.json').read_bytes())
assert run['status']=='passed' and run['whole_source_inputs_unchanged']
source=root/'native-corrected/native-originals'
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
paths=sorted(p for p in source.rglob('*') if p.is_file());before=[seal(p) for p in paths]
original=json.loads((source/'dispatch-inputs.json').read_bytes());raw=base64.b64decode(original['canonical_components'],validate=True);components=json.loads(raw)
assert len(components['types'])==1 and components['types'][0]['name']=='bool' and not components['declarations']
packet={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()} for s in original['sources']]}
assert json.loads((source/'dispatch-controls.json').read_bytes())['cases']==1
corpus={'input':packet,'declarations':json.loads((source/'dispatch-declarations.json').read_bytes()),'cases':[{'input':json.loads((source/'dispatch-00-input.json').read_bytes()),'native':json.loads((source/'dispatch-00-actual.json').read_bytes())}]}
p=root/'corpus-heterogeneous.json';assert not p.exists();p.write_text(json.dumps(corpus,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n')
assert before==[seal(p) for p in paths]
print(json.dumps({'status':'passed','full_original_files':len(paths),'native_facts_passed_as_constructor_authority':False,'output':seal(p)}))
