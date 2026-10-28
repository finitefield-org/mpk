"""Restore and verify the complete original live-capture evidence elsewhere."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile

ROOT=Path(__file__).resolve().parent


def main():
    epoch=sys.argv[1];destination=Path(sys.argv[2]);assert epoch.replace('-','').isalnum() and destination.is_absolute() and not destination.exists()
    run=ROOT/epoch;record=json.loads((run/'evidence-storage.json').read_bytes());manifest=json.loads((run/'original-evidence-manifest.json').read_bytes());assert record['status']=='passed'
    source=Path(manifest['root']);files={str(Path(row['path']).relative_to(source)):row for row in manifest['files']};directories={row['path']:row['mode'] for row in record['original_directories']}
    destination.mkdir(mode=0o700);archive_path=destination/'original-evidence.tar.gz';digest=hashlib.sha256();total=0
    with archive_path.open('xb') as archive:
        for chunk in record['chunks']:
            encoded=(run/chunk['path']).read_bytes();assert len(encoded)==chunk['encoded']['bytes'] and hashlib.sha256(encoded).hexdigest()==chunk['encoded']['sha256']
            raw=bytes.fromhex(encoded.decode('ascii'));assert len(raw)==chunk['raw_bytes'] and hashlib.sha256(raw).hexdigest()==chunk['raw_sha256']
            archive.write(raw);digest.update(raw);total+=len(raw)
    assert total==record['archive']['bytes'] and digest.hexdigest()==record['archive']['sha256']
    target=destination/'originals';target.mkdir(mode=0o700);seen=set();seen_dirs={}
    with tarfile.open(archive_path) as archive:
        for item in archive:
            name=Path(item.name);assert not name.is_absolute() and all(part not in ('','.','..') for part in name.parts)
            if item.isdir():
                assert item.name in directories and item.name not in seen_dirs and (item.mode & 0o777)==directories[item.name]
                (target/name).mkdir(parents=True,exist_ok=True);seen_dirs[item.name]=item.mode & 0o777;continue
            assert item.isfile() and item.name in files and item.name not in seen
            row=files[item.name];path=target/name;path.parent.mkdir(parents=True,exist_ok=True);h=hashlib.sha256();size=0
            with archive.extractfile(item) as incoming,path.open('xb') as outgoing:
                for block in iter(lambda:incoming.read(1024*1024),b''):outgoing.write(block);h.update(block);size+=len(block)
            path.chmod(row['mode']);assert size==item.size==row['bytes'] and h.hexdigest()==row['sha256'] and (item.mode & 0o777)==row['mode'];seen.add(item.name)
    assert seen==set(files) and seen_dirs==directories
    for name,mode in sorted(directories.items(),key=lambda x:len(Path(x[0]).parts),reverse=True):(target/name).chmod(mode)
    target.chmod(record['original_root_mode'])
    result={'status':'passed','restored_files':len(seen),'restored_bytes':sum(r['bytes'] for r in files.values()),'restored_directories':len(seen_dirs),'archive_bytes':total,'archive_sha256':digest.hexdigest(),'all_original_files_and_modes_match':True,'destination':str(destination)}
    (destination/'restore-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)


if __name__=='__main__':main()
