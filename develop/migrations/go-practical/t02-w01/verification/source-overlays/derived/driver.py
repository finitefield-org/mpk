"""Keep actual source and sidecar captures, with native comparisons separated."""
from pathlib import Path
import base64,hashlib,json,sys
OUT=Path(__file__).resolve().parent;RUN=OUT/'derived';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());SOURCE=Path(sys.argv[1])
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
paths=sorted(p for p in SOURCE.rglob('*')if p.is_file());rows=[seal(p)for p in paths];(RUN/'inputs.json').write_text(json.dumps(rows,indent=2)+'\n')
original=json.loads((SOURCE/'source-overlays/overlays-inputs.json').read_bytes());raw=base64.b64decode(original['canonical_components'],validate=True);c=json.loads(raw);assert len(c['types'])==1 and c['types'][0]['name']=='bool'and not c['declarations'];packet={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()}for s in original['sources']]}
control=json.loads((SOURCE/'source-overlays/overlay-controls.json').read_bytes());assert control['cases']==13 and control['original_sidecar_namespace_removed'];cases=[]
for n in range(13):
 before=json.loads((SOURCE/f'source-overlays/overlay-{n:02}-input.json').read_bytes());before['raw_hex']=base64.b64decode(before.pop('raw'),validate=True).hex();cases.append({'input':before,'native':json.loads((SOURCE/f'source-overlays/overlay-{n:02}-actual.json').read_bytes())})
p=OUT/'corpus.json';assert not p.exists();p.write_text(json.dumps({'input':packet,'cases':cases},ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n');assert rows==[seal(p)for p in paths]
record={'status':'passed','actual_driver_prelaunch':seal(RUN/'driver.py'),'input_receipt':seal(RUN/'inputs.json'),'actual_original_files':len(rows),'actual_original_bytes':sum(r['bytes']for r in rows),'cases':13,'original_source_and_sidecar_bytes':True,'compiler_AST_types_or_expected_binding_rows_passed_to_constructor':False,'output':seal(p)};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
