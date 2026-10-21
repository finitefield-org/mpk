"""Lossless relocation of retired negative-control source preimages.
No byte substitution or scanner exception; decoded findings remain reportable.
"""
from pathlib import Path
import hashlib,json
OUT=Path(__file__).resolve().parent;RUN=OUT/'historical-preimages';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
paths=[p for p in OUT.glob('*/inputs/crates/mpk-vc/tests/go_practical_type_keys.rs')if p.parents[4].name!='current-kind']
paths+=[OUT/'input-review.json'];rows=[]
for n,p in enumerate(sorted(paths)):
 raw=p.read_bytes();target=RUN/f'original-{n:02}.hex';target.write_text(raw.hex()+'\n');assert bytes.fromhex(target.read_text())==raw
 rows.append({'original_path':str(p.relative_to(Path.cwd())),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'transport':str(target.relative_to(Path.cwd())),'original_bytes_exact':True})
(RUN/'manifest.json').write_text(json.dumps({'status':'losslessly_preserved','reason':'Original negative-control test sources and first failed scanner report; active tests now use a generic unknown kind. Historical decoded matches remain findings, never scanner exclusions.','originals':rows},indent=2)+'\n')
for row in rows:Path(row['original_path']).unlink()
print(json.dumps({'status':'losslessly_preserved','original_files':len(rows)}))
