"""Restore every recorded file of the complete later frontend source epoch."""
from pathlib import Path
import hashlib
import json
import sys
import tarfile

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];NATIVE=ROOT.parent/'analysis-build-list/native'


def main():
    destination=Path(sys.argv[1]);assert destination.is_absolute() and not destination.exists();destination.mkdir(mode=0o700)
    receipt=json.loads((NATIVE/'inputs.json').read_bytes());archive=NATIVE/'inputs.tar.gz';raw=archive.read_bytes();entry=receipt['source_archive']
    assert len(raw)==entry['bytes'] and hashlib.sha256(raw).hexdigest()==entry['sha256']
    expected={str(Path(row['path']).relative_to(REPO)):row for row in receipt['inputs']};assert len(expected)==331
    assert sum(name.startswith('go-tools/go2vir/') for name in expected)==328
    seen=set()
    with tarfile.open(archive) as tar:
        for member in tar:
            name=Path(member.name);assert not name.is_absolute() and all(part not in ('','.','..') for part in name.parts) and str(name)==member.name
            assert member.isfile() and member.name in expected and member.name not in seen
            row=expected[member.name];data=tar.extractfile(member).read();assert len(data)==member.size==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'] and member.mode & 0o777==row['mode']
            output=destination/name;output.parent.mkdir(parents=True,exist_ok=True);output.write_bytes(data);output.chmod(row['mode']);seen.add(member.name)
    assert seen==set(expected)
    for name,row in expected.items():
        p=destination/name;data=p.read_bytes();assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'] and p.stat().st_mode & 0o777==row['mode']
    assert {str(p.relative_to(destination)) for p in destination.rglob('*') if p.is_file()}==set(expected)
    print(json.dumps({'status':'passed','scope':'Every captured file, relative path and mode in the complete later source archive, including all frontend files.','archive_sha256':entry['sha256'],'all_original_files':len(seen),'frontend_files':328,'native_Linux_release_acceptance':False}),flush=True)


if __name__=='__main__':main()
