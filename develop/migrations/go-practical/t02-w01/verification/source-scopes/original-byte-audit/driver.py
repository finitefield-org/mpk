"""Compare every decoded archive file literally to its retained original."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent
RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
read=runpy.run_path(str(OUT/'read-native.py'))['read'];files=0;size=0
archives=[]
for name in ['native-count-failure','native-dot-conflict','native-original']:
 manifest,decoded=read(name)
 for row in manifest['original_inputs']:
  p=Path(row['path']);raw=p.read_bytes();assert raw==decoded[row['member']]
  assert p.stat().st_mode&0o777==row['mode'];files+=1;size+=len(raw)
 archives.append({'name':name,'files':manifest['complete_original_files'],'bytes':manifest['complete_original_bytes']})
rust=json.loads((OUT/'rust-originals/manifest.json').read_bytes())
for row in rust['files']:
 a,b=Path(row['original']),Path(row['retained']);raw=a.read_bytes();assert raw==b.read_bytes()
 assert a.stat().st_mode&0o777==b.stat().st_mode&0o777==row['mode']
 assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256'];files+=1;size+=len(raw)
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'native_archives':archives,'actual_original_files_compared_literally':files,'actual_original_bytes_compared_literally':size,'all_original_file_modes_compared':True,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
