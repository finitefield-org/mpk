"""Losslessly retain every native original output and its directory metadata."""
from pathlib import Path
import hashlib
import json
import os
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
sys.dont_write_bytecode=True
sys.path.insert(0,str(ROOT.parent/'installed-compiler'))
from run_scoped import seal,write


def main():
    epoch=sys.argv[1];assert epoch.replace('-','').isalnum()
    run=ROOT/epoch;result=json.loads((run/'result.json').read_bytes());assert result['status']=='passed'
    prior=json.loads((run/'original-evidence-manifest.json').read_bytes());source=Path(prior['root']);files=prior['files']
    assert files==[seal(Path(row['path'])) for row in files]
    directories=sorted(p for p in source.rglob('*') if p.is_dir())
    assert not any(p.is_symlink() for p in source.rglob('*'))
    dir_rows=[{'path':str(p.relative_to(source)),'mode':p.stat().st_mode & 0o777} for p in directories]
    temp=source.parent/'original-evidence.tar.gz';assert not temp.exists()
    started=time.monotonic()
    with tarfile.open(temp,'x:gz',dereference=True) as archive:
        for p in directories:archive.add(p,arcname=str(p.relative_to(source)),recursive=False)
        for row in files:
            path=Path(row['path']);archive.add(path,arcname=str(path.relative_to(source)),recursive=False)
    original={str(Path(r['path']).relative_to(source)):r for r in files};expected_dirs={r['path']:r['mode'] for r in dir_rows};seen=set();seen_dirs={}
    with tarfile.open(temp) as archive:
        for item in archive:
            if item.isdir():
                assert item.name not in seen_dirs;seen_dirs[item.name]=item.mode & 0o777;continue
            assert item.isfile() and item.name not in seen and item.name in original
            row=original[item.name];digest=hashlib.sha256();size=0
            stream=archive.extractfile(item)
            for block in iter(lambda:stream.read(1024*1024),b''):digest.update(block);size+=len(block)
            assert size==item.size==row['bytes'] and digest.hexdigest()==row['sha256'] and (item.mode & 0o777)==row['mode'];seen.add(item.name)
    assert seen==set(original) and seen_dirs==expected_dirs and files==[seal(Path(row['path'])) for row in files]
    folder=run/'evidence-storage';folder.mkdir(exist_ok=False);chunks=[]
    with temp.open('rb') as stream:
        index=0
        while block:=stream.read(4*1024*1024):
            path=folder/f'{index:05d}.hex';path.write_bytes(block.hex().encode('ascii'))
            assert bytes.fromhex(path.read_text())==block
            chunks.append({'path':str(path.relative_to(run)),'raw_bytes':len(block),'raw_sha256':hashlib.sha256(block).hexdigest(),'encoded':seal(path)})
            index+=1
    record={'status':'passed','selection_reason':'Retain all original Go application, SDK, compiler and export evidence files with exact bytes, paths and modes, including every unselected input; the compressed stream is losslessly stored as hexadecimal chunks.','original_manifest':seal(run/'original-evidence-manifest.json'),'original_files':len(files),'original_bytes':sum(row['bytes'] for row in files),'original_root_mode':source.stat().st_mode & 0o777,'original_directories':dir_rows,'archive':seal(temp),'verified_archive_members':len(seen),'all_directories_and_modes_verified':True,'chunks':chunks,'elapsed_seconds':time.monotonic()-started,'native_Linux_release_acceptance':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(run/'evidence-storage.json',record);print(json.dumps({'status':record['status'],'files':len(files),'bytes':record['original_bytes'],'chunks':len(chunks),'archive_bytes':record['archive']['bytes']}),flush=True)


if __name__=='__main__':main()
