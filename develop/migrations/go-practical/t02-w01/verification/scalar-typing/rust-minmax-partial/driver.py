"""Losslessly preserve original outputs after their consumer is terminal."""
from pathlib import Path
import datetime,hashlib,io,json,stat,sys,tarfile
OUT=Path(__file__).resolve().parent
name,source=sys.argv[1:];assert name.replace('-','').isalnum();SOURCE=Path(source)
RUN=OUT/name;RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
def seal(path):
 raw=Path(path).read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
files=sorted(p for p in SOURCE.rglob('*')if p.is_file());assert all(not p.is_symlink()for p in files)
rows=[{**seal(p),'member':str(p.relative_to(SOURCE)),'mode':stat.S_IMODE(p.stat().st_mode)}for p in files]
receipt={'status':'running','actual_driver_prelaunch':seal(RUN/'driver.py'),'original_directory':str(SOURCE),'original_inputs':rows,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};manifest=RUN/'manifest.json';manifest.write_text(json.dumps(receipt,indent=2)+'\n')
packed=Path('/private/tmp')/('mpk-go-scalar-typing-'+name+'-original.tar.xz')
with tarfile.open(packed,'w:xz',preset=6)as archive:
 for p,row in zip(files,rows):
  raw=p.read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256']
  item=tarfile.TarInfo(row['member']);item.size=len(raw);item.mode=row['mode'];item.mtime=int(p.stat().st_mtime);archive.addfile(item,io.BytesIO(raw))
raw=packed.read_bytes();chunks=[]
for n,start in enumerate(range(0,len(raw),4*1024*1024)):
 buf=raw[start:start+4*1024*1024];p=RUN/f'part-{n:03}.hex';p.write_text(buf.hex()+'\n');chunks.append({'encoded':seal(p),'raw_bytes':len(buf),'raw_sha256':hashlib.sha256(buf).hexdigest()})
assert all(seal(r['path'])=={k:r[k]for k in ['path','bytes','sha256']}for r in rows)
receipt.update(status='passed',chunks=chunks,archive_bytes=len(raw),archive_sha256=hashlib.sha256(raw).hexdigest(),complete_original_files=len(rows),complete_original_bytes=sum(r['bytes']for r in rows),all_original_bytes_and_modes_exact=True,not_regenerated_or_recompiled=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());manifest.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','complete_original_files','complete_original_bytes','archive_bytes']}))
