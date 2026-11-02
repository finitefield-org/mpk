"""Preserve exact original Go output; derive byte-for-byte Rust input files."""
from pathlib import Path
import base64,datetime,hashlib,io,json,stat,tarfile
OUT=Path(__file__).resolve().parent
SOURCE=Path('/private/tmp/mpk-go-type-keys-formatted-go-selected-native')
RUN=OUT/'native';RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
def seal(path):
 raw=Path(path).read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
files=sorted(p for p in SOURCE.rglob('*')if p.is_file());assert all(not p.is_symlink()for p in files)
rows=[{**seal(p),'member':str(p.relative_to(SOURCE)),'mode':stat.S_IMODE(p.stat().st_mode)}for p in files]
receipt={'status':'running','actual_driver_prelaunch':seal(RUN/'driver.py'),'original_directory':str(SOURCE),'original_inputs':rows,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
manifest=RUN/'manifest.json';manifest.write_text(json.dumps(receipt,indent=2)+'\n')
packed=Path('/private/tmp/mpk-go-type-keys-original-native.tar.xz')
with tarfile.open(packed,'w:xz',preset=6)as archive:
 for p,row in zip(files,rows):
  raw=p.read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256']
  item=tarfile.TarInfo(row['member']);item.size=len(raw);item.mode=row['mode'];item.mtime=int(p.stat().st_mtime);archive.addfile(item,io.BytesIO(raw))
raw=packed.read_bytes();chunks=[]
for n,start in enumerate(range(0,len(raw),4*1024*1024)):
 buf=raw[start:start+4*1024*1024];p=RUN/f'part-{n:03}.hex';p.write_text(buf.hex()+'\n');chunks.append({'encoded':seal(p),'raw_bytes':len(buf),'raw_sha256':hashlib.sha256(buf).hexdigest()})
components=OUT/'native-components';components.mkdir(exist_ok=False);cases=[]
for n in range(5):
 p=SOURCE/f'type-keys-{n:02}.json';record=json.loads(p.read_bytes());assert record['source_typing_or_registered_context_proof']is False
 case=components/f'case-{n:02}';(case/'sources').mkdir(parents=True)
 wire=base64.b64decode(record['canonical_components'],validate=True);(case/'components.json').write_bytes(wire)
 obj=json.loads(wire);expected={s['id']:s for s in obj['source_objects']};assert len(expected)==len(record['sources'])
 for source in record['sources']:
  s=source['object'];data=base64.b64decode(source['raw'],validate=True);assert expected[s['id']]==s and len(data)==s['bytes']and hashlib.sha256(data).hexdigest()==s['raw_sha256']
  identity={k:v for k,v in s.items()if k!='id'};pre=json.dumps(identity,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+pre).hexdigest()==s['id']
  (case/'sources'/(s['id']+'.json')).write_text(json.dumps(s,sort_keys=True,separators=(',',':'),ensure_ascii=False))
  (case/'sources'/(s['id']+'.raw')).write_bytes(data)
 cases.append({'original':seal(p),'canonical':seal(case/'components.json'),'files':[seal(f)for f in sorted(case.rglob('*'))if f.is_file()]})
assert all(seal(row['path'])=={k:row[k]for k in ['path','bytes','sha256']}for row in rows)
receipt.update(status='passed',chunks=chunks,archive_bytes=len(raw),archive_sha256=hashlib.sha256(raw).hexdigest(),complete_original_files=len(rows),complete_original_bytes=sum(r['bytes']for r in rows),all_original_bytes_and_modes_exact=True,not_regenerated_or_recompiled=True,derived_Rust_cases=cases,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
manifest.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','complete_original_files','complete_original_bytes','archive_bytes']}))
