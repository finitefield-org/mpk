"""Read exact original Rust output objects and complete per-run namespaces."""
from pathlib import Path
import hashlib,io,json,tarfile
ROOT=Path(__file__).resolve().parents[6];OUT=Path(__file__).resolve().parent

def read():
 manifest=json.loads((OUT/'rust-originals/manifest.json').read_bytes());assert manifest['status']=='passed';chunks=[]
 for row in manifest['chunks']:
  raw=(ROOT/row['encoded']['path']).read_bytes();assert len(raw)==row['encoded']['bytes']and hashlib.sha256(raw).hexdigest()==row['encoded']['sha256'];buf=bytes.fromhex(raw.decode());assert len(buf)==row['raw_bytes']and hashlib.sha256(buf).hexdigest()==row['raw_sha256'];chunks.append(buf)
 packed=b''.join(chunks);assert len(packed)==manifest['archive_bytes']and hashlib.sha256(packed).hexdigest()==manifest['archive_sha256'];blobs={}
 expected={r['sha256']for r in manifest['original_inputs']}
 with tarfile.open(fileobj=io.BytesIO(packed),mode='r:xz')as archive:
  for item in archive:
   assert item.isfile()and item.name.startswith('blobs/')and item.mode==0o600;digest=item.name[6:];assert digest in expected and digest not in blobs;raw=archive.extractfile(item).read();assert hashlib.sha256(raw).hexdigest()==digest;blobs[digest]=raw
 assert set(blobs)==expected and len(blobs)==manifest['distinct_original_byte_blobs']
 files={}
 for row in manifest['original_inputs']:
  assert row['member']not in files and row['mode']in {0o600,0o644};raw=blobs[row['sha256']];assert len(raw)==row['bytes'];files[row['member']]=raw
 assert len(files)==manifest['complete_original_files']and sum(map(len,files.values()))==manifest['complete_original_bytes']
 return manifest,files
if __name__=='__main__':
 import sys
 manifest,files=read()
 if len(sys.argv)>1:sys.stdout.buffer.write(files[sys.argv[1]])
 else:print(json.dumps({'status':'passed','files':len(files),'distinct_byte_objects':manifest['distinct_original_byte_blobs']}))
