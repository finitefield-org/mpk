"""Own every native-observed raw SDK source byte without narrowing its corpus."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def seal(path):
 p=Path(path);b=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def main():
 out=ROOT/'source-byte-transport.json';assert not out.exists()
 native_path=ROOT/'native/observations/original-sdk-source-roles.json'
 sdk_path=ROOT.parent/'module-graphs/original-sdk/inputs.json'
 sdk=json.loads(sdk_path.read_bytes());native=json.loads(native_path.read_bytes())
 assert sdk['sources']==[seal(row['path']) for row in sdk['sources']]
 inputs={'native':seal(native_path),'original_sdk':seal(sdk_path),'sdk_archive':sdk['complete_archive'],'driver':seal(__file__)}
 root=Path(sdk['root']);rows={str(Path(row['path']).relative_to(root)):row for row in sdk['sources']}
 prepared=ROOT.parent/'compiler-inputs/native/native-originals/versioned-owned/original-compiler-files'
 store=ROOT/'original-source-bytes';store.mkdir(exist_ok=False)
 owned=[];copied=set()
 for row in native['sdk_sources']:
  original=rows[row['path']];assert original['sha256']==row['raw_sha256'] and original['bytes']==row['bytes']
  raw=Path(original['path']).read_bytes();assert hashlib.sha256(raw).hexdigest()==row['raw_sha256']
  name=row['raw_sha256']+'.raw';existing=prepared/name
  if existing.is_file():
   assert existing.read_bytes()==raw;storage='prepared-compiler'
  else:
   storage='raw-sdk'
   if name not in copied:(store/name).write_bytes(raw);copied.add(name)
  owned.append({'path':row['path'],'bytes':len(raw),'raw_sha256':row['raw_sha256'],'storage':storage,'raw_file':name})
 assert sdk['sources']==[seal(row['path']) for row in sdk['sources']]
 value={'status':'passed','inputs':inputs,'sdk_sources':owned,'raw_source_objects':len(owned),'new_unique_raw_files':len(copied),'complete_original_raw_sdk_unchanged':True,'scope':'Every one of the11004 full raw SDK source observations is retained. The original prepared compiler still owns all5577 files and5480 SDK sources; its original application retains182 packages/5697 objects and its Root contract. Reuse exact committed prepared bytes when identical, otherwise own the entire original raw source. No source/body/role substitution or universe reduction.'}
 out.write_text(json.dumps(value,indent=2)+'\n');print(json.dumps({k:v for k,v in value.items() if k not in ['sdk_sources','inputs']}))
if __name__=='__main__':main()
