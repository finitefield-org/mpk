"""Retain actual native inputs; export source-only buffers and comparison rows."""
from pathlib import Path
import base64,hashlib,json,sys
OUT=Path(__file__).resolve().parent;RUN=OUT/'derived';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());SOURCE=Path(sys.argv[1])
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
paths=sorted(p for p in SOURCE.rglob('*')if p.is_file());inputs=[seal(p)for p in paths];(RUN/'inputs.json').write_text(json.dumps({'actual_original_directory':str(SOURCE),'actual_driver_prelaunch':seal(RUN/'driver.py'),'files':inputs},indent=2)+'\n')
original=json.loads((SOURCE/'boxings-inputs.json').read_bytes());components=base64.b64decode(original['canonical_components'],validate=True);c=json.loads(components);assert len(c['types'])==1 and c['types'][0]['name']=='bool'and not c['declarations']
packet={'components_hex':components.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()}for s in original['sources']]}
cases=[];count=json.loads((SOURCE/'boxings-controls.json').read_bytes())['cases']
for n in range(count):
 before=json.loads((SOURCE/f'boxing-{n:02}-input.json').read_bytes());actual=json.loads((SOURCE/f'boxing-{n:02}-actual.json').read_bytes());cases.append({'name':before['name'],'root':before['root'],'native':actual})
p=OUT/'corpus.json';assert not p.exists();p.write_text(json.dumps({'input':packet,'cases':cases},ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n');assert inputs==[seal(p)for p in paths]
record={'status':'passed','actual_driver_prelaunch':seal(RUN/'driver.py'),'actual_original_directory':str(SOURCE),'original_input_receipt':seal(RUN/'inputs.json'),'cases':count,'only_bool_candidate_type':True,'native_types_AST_expected_targets_or_boxing_rows_passed_to_constructor':False,'original_sources_modified':False,'output':seal(p)};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','cases':count,'bytes':record['output']['bytes']}))
