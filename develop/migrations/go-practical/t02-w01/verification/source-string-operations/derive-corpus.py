"""Pass only original source inputs and explicit source selections to Rust."""
from pathlib import Path
import base64,hashlib,json
root=Path(__file__).resolve().parent
r=json.loads((root/'native-complete/result.json').read_bytes());assert r['status']=='passed' and r['whole_source_inputs_unchanged']
units=[]
for name in ['pure','wide']:
 p=root/'native-complete/native-originals'/name
 original=json.loads((p/'unit-inputs.json').read_bytes());raw=base64.b64decode(original['canonical_components'],validate=True);c=json.loads(raw);assert len(c['packages'])==3 and not c['declarations'] and len(c['types'])==1 and c['types'][0]['name']=='bool'
 selection=json.loads((p/'unit-selection.json').read_bytes());assert selection['name']==name
 selection['input']={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()} for s in original['sources']]}
 units.append(selection)
p=root/'corpus.json';assert not p.exists();p.write_text(json.dumps({'units':units},sort_keys=True,separators=(',',':'))+'\n')
print(json.dumps({'status':'passed','units':len(units),'native_facts_used_as_constructor_inputs':False,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}))
