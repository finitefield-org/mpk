"""Literal original byte/mode audit, including the runtime-pinned native rerun."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent
RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];count=size=0;native=None;runtime=None;rows=[]
for name in ['native-build-failure','native-original','native-runtime','rust-original','scope-original']:
 manifest,files=read(name)
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==files[row['member']]and p.stat().st_mode&0o777==row['mode'];count+=1;size+=len(raw)
 rows.append({'name':name,'files':manifest['complete_original_files'],'bytes':manifest['complete_original_bytes']})
 if name=='native-original':native=files
 if name=='native-runtime':runtime=files
assert native==runtime and len(native)==10235
corpus=json.loads((OUT/'corpus.json').read_bytes());assert len(corpus)==5117
for n,row in enumerate(corpus):
 assert json.loads(native[f'case-{n:05d}/input.json'])==row['input']
 assert json.loads(native[f'case-{n:05d}/native.json'])==row['native']
assert count==30978 and size==66409565
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':rows,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'native_runtime_rerun_all10235_original_files_exact':True,'all5117_corpus_rows_equal_actual_original_files':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
