"""Preserve terminal original native observations with literal byte/mode checks."""
from pathlib import Path
import hashlib
import io
import json
import shutil
import sys
import tarfile
import tempfile

OUT = Path(__file__).resolve().parent
epoch = sys.argv[1]
assert epoch in {'native','native-receivers'}
SOURCE = OUT/epoch/'observations'
ARCHIVE = OUT/(epoch+'-original')
ARCHIVE.mkdir(exist_ok=False)
(ARCHIVE/'driver.py').write_bytes(Path(__file__).read_bytes())
assert json.loads((OUT/epoch/'result.json').read_bytes())['status'] == 'passed'
derived = 'derived' if epoch == 'native' else 'derived-receivers'
assert json.loads((OUT/derived/'result.json').read_bytes())['status'] == 'passed'
files = sorted(p for p in SOURCE.rglob('*') if p.is_file())
rows = [{'path':str(p),'member':p.relative_to(SOURCE).as_posix(),'bytes':p.stat().st_size,
         'mode':p.stat().st_mode & 0o777,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
(ARCHIVE/'prelaunch.json').write_text(json.dumps({'source':str(SOURCE),'files':rows},indent=2)+'\n')
with tempfile.TemporaryDirectory(prefix='mpk-module-original-',dir='/private/tmp') as temporary:
    path = Path(temporary)/'originals.tar.xz'
    with tarfile.open(path,'w:xz',preset=6) as archive:
        for row in rows:
            raw = Path(row['path']).read_bytes()
            assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
            member = tarfile.TarInfo(row['member'])
            member.size,member.mode = row['bytes'],row['mode']
            archive.addfile(member,io.BytesIO(raw))
    chunks = []
    with path.open('rb') as stream:
        while raw := stream.read(2_097_152):
            chunk = ARCHIVE/f'originals-{len(chunks):03}.tar.xz.hex'
            chunk.write_text(raw.hex()+'\n')
            chunks.append({'path':str(chunk),'bytes':chunk.stat().st_size,'sha256':hashlib.sha256(chunk.read_bytes()).hexdigest()})
    packed = b''.join(bytes.fromhex(Path(r['path']).read_text()) for r in chunks)
    assert packed == path.read_bytes()
    with tarfile.open(fileobj=io.BytesIO(packed),mode='r:xz') as archive:
        members = archive.getmembers()
        assert [m.name for m in members] == [r['member'] for r in rows]
        for member,row in zip(members,rows):
            original = Path(row['path'])
            raw = archive.extractfile(member).read()
            assert raw == original.read_bytes() and member.mode == original.stat().st_mode & 0o777 == row['mode']
            assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
    record = {'status':'passed','epoch':epoch,'encoding':'xz-tar split into lowercase hexadecimal chunks',
              'chunks':chunks,'files':rows,'actual_files_compared_literally':len(rows),'actual_bytes_compared_literally':sum(r['bytes'] for r in rows),
              'actual_driver_prelaunch_sha256':hashlib.sha256((ARCHIVE/'driver.py').read_bytes()).hexdigest()}
    (ARCHIVE/'manifest.json').write_text(json.dumps(record,indent=2)+'\n')
shutil.rmtree(SOURCE)
print(json.dumps({k:record[k] for k in ['status','epoch','actual_files_compared_literally','actual_bytes_compared_literally']}))
