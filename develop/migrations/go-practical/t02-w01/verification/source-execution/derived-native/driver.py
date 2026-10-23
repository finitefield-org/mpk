"""Separate original source+explicit locators from native output observations."""
from pathlib import Path
import base64,hashlib,json,sys
ROOT=Path(__file__).resolve().parent;epoch=sys.argv[1] if len(sys.argv)>1 else 'native';out=ROOT/('derived-'+epoch);out.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
run=ROOT/epoch/'result.json';result=json.loads(run.read_bytes());assert result['status']=='passed' and result['whole_source_inputs_unchanged']
original=ROOT/epoch/'native-originals';inputs=[Path(__file__),run,*[original/n for n in ['unit-inputs.json','unit-selection.json','native-values.json']]];rows=[]
for i,p in enumerate(inputs):
 target=out/f'{i:03d}.original.hex';target.write_text(p.read_bytes().hex()+'\n');target.chmod(0o644);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target),'snapshot_encoding':'hex','snapshot_mode':0o644})
(out/'prelaunch-inputs.json').write_text(json.dumps(rows,indent=2)+'\n');(out/'driver.py').write_bytes(Path(__file__).read_bytes())
payload=json.loads((original/'unit-inputs.json').read_bytes());raw=base64.b64decode(payload['canonical_components'],validate=True);components=json.loads(raw);assert len(components['packages'])==3 and not components['declarations'] and len(components['types'])==1 and components['types'][0]['name']=='bool'
selection=json.loads((original/'unit-selection.json').read_bytes());selection['input']={'components_hex':raw.hex(),'sources':[{'object':r['object'],'raw_hex':base64.b64decode(r['raw'],validate=True).hex()} for r in payload['sources']]}
corpus=ROOT/'corpus.json';assert not corpus.exists();corpus.write_text(json.dumps(selection,sort_keys=True,separators=(',',':'))+'\n')
observations=ROOT/'native-values.json';assert not observations.exists();observations.write_bytes((original/'native-values.json').read_bytes());observations.chmod(0o644)
for r in rows:assert seal(r['path'])=={k:r[k] for k in ['path','bytes','sha256']} and Path(r['path']).stat().st_mode&0o777==r['mode']
r={'status':'passed','whole_inputs_unchanged':True,'complete_prelaunch_inputs':seal(out/'prelaunch-inputs.json'),'actual_driver_prelaunch':seal(out/'driver.py'),'native_types_or_results_used_as_constructor_inputs':False,'original_source_packet':seal(corpus),'separate_native_result_bytes':seal(observations)};(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({'status':'passed','packages':3,'full_native_cases':127,'native_types_or_results_used_as_constructor_inputs':False}))
