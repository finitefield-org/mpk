"""Read complete recorded native originals without extracting or executing them."""
from pathlib import Path,PurePosixPath
import hashlib,io,json,tarfile
OUT=Path(__file__).resolve().parent
ARCHIVES={'original-owned':OUT/'native-archives/original-owned/manifest.json','complete-shapes':OUT/'native-archives/complete-shapes/manifest.json','source-budget':OUT/'native-archive-source-budget/source-budget/manifest.json','alias-observations':OUT/'native-archive-alias-observations/alias-observations/manifest.json','exact-locators':OUT/'native-archive-exact-locators/exact-locators/manifest.json'}
def seal(path):
 raw=Path(path).read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
def read(epoch):
 manifest=json.loads(ARCHIVES[epoch].read_bytes());assert manifest['status']=='passed'and manifest['all_original_bytes_and_modes_exact']and manifest['not_regenerated_or_recompiled']
 assert seal(manifest['actual_driver_prelaunch']['path'])==manifest['actual_driver_prelaunch']
 parts=[]
 for row in manifest['chunks']:
  assert seal(row['encoded']['path'])==row['encoded'];raw=bytes.fromhex(Path(row['encoded']['path']).read_text());assert len(raw)==row['raw_bytes']and hashlib.sha256(raw).hexdigest()==row['raw_sha256'];parts.append(raw)
 packed=b''.join(parts);authority=manifest['whole_exact_archive'];assert len(packed)==authority['bytes']and hashlib.sha256(packed).hexdigest()==authority['sha256']
 expected={r['member']:r for r in manifest['original_inputs']};assert len(expected)==len(manifest['original_inputs'])==manifest['complete_original_files']
 result={}
 with tarfile.open(fileobj=io.BytesIO(packed),mode='r:xz')as archive:
  for entry in archive:
   name=PurePosixPath(entry.name);assert entry.isfile()and not name.is_absolute()and '..'not in name.parts and str(name)==entry.name and entry.name in expected and entry.name not in result
   row=expected[entry.name];raw=archive.extractfile(entry).read();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256']and entry.mode==row['mode'];result[entry.name]=raw
 assert set(result)==set(expected)and sum(map(len,result.values()))==manifest['complete_original_bytes']
 return manifest,result
if __name__=='__main__':
 for epoch in ARCHIVES:
  manifest,files=read(epoch);print(json.dumps({'epoch':epoch,'status':'passed','whole_original_files':len(files),'whole_original_bytes':sum(map(len,files.values()))}))
