"""Remove redundant copies only after literal archive/producer correspondence."""
from pathlib import Path
import hashlib,json,runpy,shutil
OUT=Path(__file__).resolve().parent;RUN=OUT/'native-copy-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());read=runpy.run_path(str(OUT/'read-originals.py'))['read'];m,files=read('native-archive');rows=json.loads((OUT/'derived/inputs.json').read_bytes());assert len(files)==len(rows)==10977
for r in rows:
 p=Path(r['copy']['path']);member=str(p.relative_to(OUT/'native-original'));raw=files[member];q=Path(r['original']['path']);assert p.read_bytes()==q.read_bytes()==raw and p.stat().st_mode&0o777==q.stat().st_mode&0o777==r['copy']['mode']
shutil.rmtree(OUT/'native-original')
record={'status':'passed','actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'literal_files_compared':len(files),'literal_bytes_compared':sum(map(len,files.values())),'modes_compared':True,'redundant_copy_removed':True,'producer_originals_preserved':True,'earlier_cleanup_helper':'An inline postprocessing helper failed before deletion because relative_to mixed absolute and relative paths. This dedicated prelaunch driver uses the actual absolute directory.'};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
