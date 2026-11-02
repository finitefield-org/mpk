"""Retain complete original source packets; native comparison facts stay separate."""
from pathlib import Path
import base64,hashlib,json
ROOT=Path(__file__).resolve().parent;out=ROOT/'derived';out.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
r=ROOT/'native/result.json';run=json.loads(r.read_bytes());assert run['status']=='failed' and run['whole_source_inputs_unchanged']
inputs=[Path(__file__),r]+[ROOT/f'native/native-originals/{unit}/{name}' for unit in ['pure','wide'] for name in ['unit-inputs.json','unit-selection.json','native-closure.json','storage-original-exports.json']]
rows=[]
for i,p in enumerate(inputs):
 target=out/f'{i:03d}.original.hex';target.write_text(p.read_bytes().hex()+'\n');target.chmod(0o644);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target),'snapshot_encoding':'hex','snapshot_mode':0o644})
receipt=out/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(out/'driver.py').write_bytes(Path(__file__).read_bytes());units=[]
for unit in ['pure','wide']:
 folder=ROOT/f'native/native-originals/{unit}';exports=json.loads((folder/'storage-original-exports.json').read_bytes());assert exports['error']=='<nil>'
 packet=json.loads((folder/'unit-inputs.json').read_bytes());raw=base64.b64decode(packet['canonical_components'],validate=True);c=json.loads(raw);assert len(c['packages'])==3 and not c['declarations'] and len(c['types'])==1 and c['types'][0]['name']=='bool'
 selection=json.loads((folder/'unit-selection.json').read_bytes());assert selection['name']==unit
 selection['input']={'components_hex':raw.hex(),'sources':[{'object':r['object'],'raw_hex':base64.b64decode(r['raw'],validate=True).hex()} for r in packet['sources']]};units.append(selection)
p=ROOT/'corpus.json';assert not p.exists();p.write_text(json.dumps({'units':units},sort_keys=True,separators=(',',':'))+'\n')
for r in rows:assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode']
r={'status':'passed','whole_inputs_unchanged':True,'complete_prelaunch_inputs':seal(receipt),'actual_driver_prelaunch':seal(out/'driver.py'),'original_parent_result_stays_failed':True,'source_capture_and_original_Go_typecheck_succeeded':True,'parent_failure':'Named-payload-only native comparison helper rejects builtin string after complete source/export/closure capture. No native type comparison output is used as constructor input.','native_type_facts_used_as_constructor_inputs':False,'complete_original_source_packet':seal(p)};(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({'status':'passed','complete_packages_per_unit':3,'original_parent_stays_failed':True,'native_type_facts_used_as_constructor_inputs':False}))
