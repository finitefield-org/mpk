"""Read original bytes from verified archives without extraction or execution."""
from pathlib import Path,PurePosixPath
import hashlib,io,json,tarfile
ROOT=Path(__file__).resolve().parents[6]
OUT=Path(__file__).resolve().parent

def read(name='native'):
 manifest=json.loads((OUT/name/'manifest.json').read_bytes());assert manifest['status']=='passed'
 chunks=[]
 for row in manifest['chunks']:
  p=ROOT/row['encoded']['path'];encoded=p.read_bytes();assert len(encoded)==row['encoded']['bytes']and hashlib.sha256(encoded).hexdigest()==row['encoded']['sha256']
  raw=bytes.fromhex(encoded.decode());assert len(raw)==row['raw_bytes']and hashlib.sha256(raw).hexdigest()==row['raw_sha256'];chunks.append(raw)
 raw=b''.join(chunks);assert len(raw)==manifest['archive_bytes']and hashlib.sha256(raw).hexdigest()==manifest['archive_sha256']
 expected={r['member']:r for r in manifest['original_inputs']};assert len(expected)==len(manifest['original_inputs']);result={}
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:xz')as archive:
  for member in archive:
   path=PurePosixPath(member.name);assert member.isfile()and not path.is_absolute()and all(p not in {'.','..',''}for p in path.parts)and member.name in expected and member.name not in result
   value=archive.extractfile(member).read();row=expected[member.name];assert member.mode==row['mode']and len(value)==row['bytes']and hashlib.sha256(value).hexdigest()==row['sha256'];result[member.name]=value
 assert set(result)==set(expected)and len(result)==manifest['complete_original_files']and sum(map(len,result.values()))==manifest['complete_original_bytes']
 return manifest,result
if __name__=='__main__':
 import sys
 manifest,files=read(sys.argv[1]if len(sys.argv)>1 else'native')
 if len(sys.argv)>2:sys.stdout.buffer.write(files[sys.argv[2]])
 else:print(json.dumps({'status':'passed','files':len(files),'bytes':sum(map(len,files.values()))}))
