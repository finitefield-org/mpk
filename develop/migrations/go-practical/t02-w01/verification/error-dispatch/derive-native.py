"""Keep actual producer files; export only original buffers to the constructor."""
from pathlib import Path
import base64,hashlib,json,sys
OUT=Path(__file__).resolve().parent;RUN=OUT/'derived';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());SOURCE=Path(sys.argv[1])
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
paths=sorted(p for p in SOURCE.rglob('*')if p.is_file());rows=[seal(p)for p in paths];(RUN/'inputs.json').write_text(json.dumps(rows,indent=2)+'\n')
original=json.loads((SOURCE/'error-dispatch/dispatch-inputs.json').read_bytes());raw=base64.b64decode(original['canonical_components'],validate=True);c=json.loads(raw);assert len(c['types'])==1 and c['types'][0]['name']=='bool'and not c['declarations'];packet={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()}for s in original['sources']]}
count=json.loads((SOURCE/'error-dispatch/dispatch-controls.json').read_bytes())['cases'];assert count==23
cases=[{'input':json.loads((SOURCE/f'error-dispatch/dispatch-{n:02}-input.json').read_bytes()),'native':json.loads((SOURCE/f'error-dispatch/dispatch-{n:02}-actual.json').read_bytes())}for n in range(count)]
corpus={'input':packet,'declarations':json.loads((SOURCE/'error-dispatch/dispatch-declarations.json').read_bytes()),'cases':cases};p=OUT/'corpus.json';assert not p.exists();p.write_text(json.dumps(corpus,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n');assert rows==[seal(p)for p in paths]
record={'status':'passed','actual_driver_prelaunch':seal(RUN/'driver.py'),'input_receipt':seal(RUN/'inputs.json'),'actual_original_files':len(rows),'actual_original_bytes':sum(r['bytes']for r in rows),'cases':count,'compiler_AST_or_type_facts_passed_to_constructor':False,'output':seal(p)};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
