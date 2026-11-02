from pathlib import Path
import datetime,hashlib,io,json,lzma,stat,tarfile
OUT=Path('develop/migrations/go-practical/t02-w01/verification/storage-source')
run=OUT/'native-archive-alias-observations';run.mkdir(exist_ok=False)
(run/'archive.py').write_bytes(Path(__file__).read_bytes())
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
for epoch in ['alias-observations']:
 source=Path('/private/tmp/mpk-go-practical-storage-source-'+epoch+'-selected-evidence')
 target=run/epoch;target.mkdir()
 files=sorted(p for p in source.rglob('*')if p.is_file());assert all(not p.is_symlink()for p in files)
 rows=[{**seal(p),'member':str(p.relative_to(source)),'mode':stat.S_IMODE(p.stat().st_mode)}for p in files]
 receipt={'status':'running','actual_driver_prelaunch':seal(run/'archive.py'),'original_directory':str(source),'original_inputs':rows,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
 manifest=target/'manifest.json';manifest.write_text(json.dumps(receipt,indent=2)+'\n')
 packed=Path('/private/tmp/mpk-go-storage-source-'+epoch+'-native-originals.tar.xz')
 with tarfile.open(packed,'w:xz',preset=6)as archive:
  for p,row in zip(files,rows):
   raw=p.read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256']
   item=tarfile.TarInfo(row['member']);item.size=len(raw);item.mode=row['mode'];item.mtime=int(p.stat().st_mtime);archive.addfile(item,io.BytesIO(raw))
 raw=packed.read_bytes();chunks=[]
 for n,start in enumerate(range(0,len(raw),4*1024*1024)):
  buf=raw[start:start+4*1024*1024];p=target/f'part-{n:03}.hex';p.write_text(buf.hex()+'\n');chunks.append({'encoded':seal(p),'raw_bytes':len(buf),'raw_sha256':hashlib.sha256(buf).hexdigest()})
 decoded=b''.join(bytes.fromhex(Path(r['encoded']['path']).read_text())for r in chunks);assert decoded==raw
 expected={r['member']:r for r in rows};seen=set()
 with tarfile.open(fileobj=io.BytesIO(decoded),mode='r:xz')as archive:
  for item in archive:
   assert item.isfile()and item.name in expected and item.name not in seen;seen.add(item.name)
   buf=archive.extractfile(item).read();r=expected[item.name];assert item.mode==r['mode']and len(buf)==r['bytes']and hashlib.sha256(buf).hexdigest()==r['sha256']
 assert seen==set(expected)and all(seal(r['path'])=={k:r[k]for k in ['path','sha256','bytes']}for r in rows)
 receipt.update(status='passed',whole_exact_archive=seal(packed),chunks=chunks,complete_original_files=len(rows),complete_original_bytes=sum(r['bytes']for r in rows),all_original_bytes_and_modes_exact=True,not_regenerated_or_recompiled=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());manifest.write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({'epoch':epoch,'status':'passed','files':len(rows),'original_bytes':receipt['complete_original_bytes'],'archive_bytes':len(raw),'chunks':len(chunks)}),flush=True)
