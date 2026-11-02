"""Deduplicate identical actual Rust output bytes; retain each original namespace."""
from pathlib import Path
import datetime,hashlib,io,json,stat,tarfile
OUT=Path(__file__).resolve().parent;RUN=OUT/'rust-originals';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
sources={
 'original-paired':Path('/private/tmp/mpk-go-lex-original-paired-rust-paired-outputs'),
 'original-smoke':Path('/private/tmp/mpk-go-lex-original-paired-rust-smoke-outputs'),
 'byte-bound-paired':Path('/private/tmp/mpk-go-lex-byte-bound-rust-paired-outputs'),
 'byte-bound-smoke':Path('/private/tmp/mpk-go-lex-byte-bound-ready-rust-smoke-outputs'),
};rows=[];blobs={}
def seal(path):
 raw=Path(path).read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
for namespace,root in sources.items():
 for path in sorted(root.rglob('*')):
  if not path.is_file():continue
  assert not path.is_symlink();row={**seal(path),'member':namespace+'/'+str(path.relative_to(root)),'mode':stat.S_IMODE(path.stat().st_mode)};rows.append(row);blobs.setdefault(row['sha256'],path)
manifest=RUN/'manifest.json';record={'status':'running','actual_driver_prelaunch':seal(RUN/'driver.py'),'original_inputs':rows,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};manifest.write_text(json.dumps(record,indent=2)+'\n')
packed=Path('/private/tmp/mpk-go-lex-rust-originals.tar.xz')
with tarfile.open(packed,'w:xz',preset=6)as archive:
 for digest,path in sorted(blobs.items()):
  raw=path.read_bytes();assert hashlib.sha256(raw).hexdigest()==digest;item=tarfile.TarInfo('blobs/'+digest);item.size=len(raw);item.mode=0o600;archive.addfile(item,io.BytesIO(raw))
raw=packed.read_bytes();chunks=[]
for n,start in enumerate(range(0,len(raw),4*1024*1024)):
 buf=raw[start:start+4*1024*1024];path=RUN/f'part-{n:03}.hex';path.write_text(buf.hex()+'\n');chunks.append({'encoded':seal(path),'raw_bytes':len(buf),'raw_sha256':hashlib.sha256(buf).hexdigest()})
assert all(seal(row['path'])=={k:row[k]for k in ['path','bytes','sha256']}for row in rows)
record.update(status='passed',chunks=chunks,archive_bytes=len(raw),archive_sha256=hashlib.sha256(raw).hexdigest(),complete_original_files=len(rows),complete_original_bytes=sum(r['bytes']for r in rows),distinct_original_byte_blobs=len(blobs),not_regenerated_or_recompiled=True,original_modes_retained_in_namespace=True,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());manifest.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','complete_original_files','complete_original_bytes','distinct_original_byte_blobs','archive_bytes']}))
