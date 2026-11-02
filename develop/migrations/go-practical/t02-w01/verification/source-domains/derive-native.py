"""Export only the intact original source packet; keep compiler comparisons separate."""
from pathlib import Path
import base64,hashlib,json,shutil,sys
OUT=Path(__file__).resolve().parent;RUN=OUT/'derived';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());SOURCE=Path(sys.argv[1]);COPY=OUT/'native-original';COPY.mkdir(exist_ok=False)
def seal(p):
 raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777}
paths=sorted(p for p in SOURCE.rglob('*')if p.is_file());rows=[]
for p in paths:
 q=COPY/p.relative_to(SOURCE);q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);assert p.read_bytes()==q.read_bytes()and(p.stat().st_mode&0o777)==(q.stat().st_mode&0o777);rows.append({'original':seal(p),'copy':seal(q)})
(RUN/'inputs.json').write_text(json.dumps(rows,indent=2)+'\n')
original=json.loads((COPY/'source-domains/domains-inputs.json').read_bytes());raw=base64.b64decode(original['canonical_components'],validate=True);c=json.loads(raw);assert len(c['types'])==1 and c['types'][0]['name']=='bool'and not c['declarations']
packet={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()}for s in original['sources']]}
corpus={'input':packet,**{key:json.loads((COPY/f'source-domains/domains-{key}.json').read_bytes())for key in ['roots','declarations','observations']}}
p=OUT/'corpus.json';assert not p.exists();p.write_text(json.dumps(corpus,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n')
assert all(seal(Path(r['original']['path']))==r['original']and seal(Path(r['copy']['path']))==r['copy']for r in rows)
record={'status':'passed','actual_driver_prelaunch':seal(RUN/'driver.py'),'input_receipt':seal(RUN/'inputs.json'),'copied_original_files':len(rows),'copied_original_bytes':sum(r['original']['bytes']for r in rows),'comparison_observations':len(corpus['observations']),'source_only_constructor_input':True,'compiler_AST_or_type_facts_passed_to_constructor':False,'output':seal(p)};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
