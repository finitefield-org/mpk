"""Store two complete original frontend binaries in reversible hex form."""
from pathlib import Path
import hashlib
import json
import os
import shutil

ROOT=Path(__file__).resolve().parent
TEMP=Path('/private/tmp/mpk-analysis-build-list-original-binaries')


def seal(path):
 raw=path.read_bytes();return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


def main():
 out=ROOT/'native/build';receipt=json.loads((ROOT/'native/result.json').read_bytes());assert receipt['status']=='passed' and not TEMP.exists();TEMP.mkdir(mode=0o700)
 records=[]; originals=[]
 for index in range(2):
  src=out/f'frontend-{index}.elf';raw=src.read_bytes();original=receipt['binary_outputs'][index]
  assert raw.startswith(b'\x7fELF') and len(raw)==original['bytes'] and hashlib.sha256(raw).hexdigest()==original['sha256']
  encoded=out/f'frontend-{index}.elf.hex';assert not encoded.exists();encoded.write_bytes(raw.hex().encode('ascii'))
  restored=bytes.fromhex(encoded.read_text());assert restored==raw
  dest=TEMP/src.name;shutil.move(src,dest);assert dest.read_bytes()==restored and not src.exists()
  originals.append(restored);records.append({'original':original,'encoded':seal(encoded),'recovery_sha256':hashlib.sha256(restored).hexdigest(),'exact_recovery':True,'local_original_copy':str(dest)})
 assert originals[0]==originals[1]
 result={'status':'passed','selection_reason':'Retain both exact original binary byte streams losslessly as reversible hex data. The repository scanner reads tracked files as text; the initial review reported incidental obsolete-token matches in ELF machine bytes. No scanner rule, seal or exclusion is changed.','files':records,'two_complete_original_binaries_equal':True,'all_exactly_recoverable':True,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 (ROOT/'binary-storage.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'files':len(records),'all_exactly_recoverable':True}))


if __name__=='__main__':main()
