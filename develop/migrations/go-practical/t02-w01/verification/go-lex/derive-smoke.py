"""Retain exact original native edge cases for default Rust test execution."""
from pathlib import Path
import hashlib,json
OUT=Path(__file__).resolve().parent;source=Path('/private/tmp/mpk-go-lex-formatted-native');rows=[];originals=[]
for case in sorted(source.glob('case-*')):
 native=json.loads((case/'native.json').read_bytes())
 if native['label'].startswith('original/'):continue
 obj=json.loads((case/'source.json').read_bytes());raw=(case/'source.raw').read_bytes();assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
 rows.append({'object':obj,'raw_hex':raw.hex(),'native':native});originals.append({'native_path':str(case/'native.json'),'sha256':hashlib.sha256((case/'native.json').read_bytes()).hexdigest(),'source_id':obj['id']})
target=OUT/'smoke.json';assert not target.exists();target.write_text(json.dumps(rows,separators=(',',':'),ensure_ascii=False)+'\n');(OUT/'smoke-origins.json').write_text(json.dumps({'cases':len(rows),'complete_original_native_entries':originals,'raw_bytes_not_normalized':True},indent=2)+'\n');print(json.dumps({'cases':len(rows),'bytes':target.stat().st_size}))
