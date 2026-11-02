"""Restore every original registered-source file without changing its bytes."""
from pathlib import Path
import hashlib
import json
import os
import sys
import tarfile

ROOT=Path(__file__).resolve().parent
INSTALLED=ROOT.parent/'installed-compiler'
OLD_ROOT=Path('/private/tmp/mpk-installed-compiler-205-go-source')


def main():
    dest=Path(sys.argv[1]);assert dest.is_absolute() and not dest.exists();dest.mkdir(mode=0o700)
    receipt=json.loads((INSTALLED/'original-go-source.json').read_bytes())
    assert receipt['basis_commit']=='ed67a4c388defe8babdbc55f29851552562121d9'
    archive=INSTALLED/'original-go-source.tar';raw=archive.read_bytes();assert hashlib.sha256(raw).hexdigest()==receipt['archive_sha256']
    original={str(Path(row['path']).relative_to(OLD_ROOT)):row for row in receipt['source_files']};assert len(original)==269
    assert sum(name.startswith('go-tools/go2vir/') for name in original)==242
    seen=set();directories={};archive_modes_differ=0
    with tarfile.open(archive) as tar:
        for member in tar:
            path=Path(member.name);assert not path.is_absolute() and all(part not in ('','.','..') for part in path.parts) and str(path)==member.name
            output=dest/path
            if member.isdir():
                assert member.name not in directories;directories[member.name]=member.mode & 0o777;output.mkdir(parents=True,exist_ok=True);continue
            assert member.isfile() and member.name in original and member.name not in seen
            row=original[member.name];data=tar.extractfile(member).read();assert len(data)==member.size==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
            archive_modes_differ += (member.mode & 0o777)!=row['mode']
            output.parent.mkdir(parents=True,exist_ok=True);output.write_bytes(data);output.chmod(row['mode']);seen.add(member.name)
    assert seen==set(original) and len(directories)==61
    for name,mode in sorted(directories.items(),key=lambda x:len(Path(x[0]).parts),reverse=True):(dest/name).chmod(mode)
    for name,row in original.items():
        p=dest/name;data=p.read_bytes();assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'] and p.stat().st_mode & 0o777==row['mode']
    assert {str(p.relative_to(dest)) for p in dest.rglob('*') if p.is_file()}==set(original)
    result={'status':'passed','scope':'Every registered original frontend/reference-checker/release-source file, path and receipt mode. The original tar and its own modes remain intact.','basis_commit':receipt['basis_commit'],'source_files':len(original),'frontend_files':242,'directories':len(directories),'archive_file_modes_distinct_from_receipt':archive_modes_differ,'archive_sha256':receipt['archive_sha256'],'native_Linux_release_acceptance':False}
    print(json.dumps(result),flush=True)


if __name__=='__main__':main()
