"""Preserve every native observation byte/mode in a verified lossless archive.

Consumers have terminated before archival. The original comparison corpus and
receipts remain distinct from this storage encoding.
"""
from pathlib import Path
import gzip
import hashlib
import json
import shutil
import tarfile
import tempfile

OUT = Path(__file__).resolve().parent
SOURCE = OUT/'native/observations'
ARCHIVE = OUT/'native-original'
ARCHIVE.mkdir(exist_ok=False)
(ARCHIVE/'driver.py').write_bytes(Path(__file__).read_bytes())
assert json.loads((OUT/'native/result.json').read_bytes())['status'] == 'passed'
assert json.loads((OUT/'derived/result.json').read_bytes())['status'] == 'passed'
files = sorted(p for p in SOURCE.rglob('*') if p.is_file())
rows = [{'path': str(p), 'member': p.relative_to(SOURCE).as_posix(), 'bytes': p.stat().st_size,
         'mode': p.stat().st_mode & 0o777, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
(ARCHIVE/'prelaunch.json').write_text(json.dumps({'source':str(SOURCE), 'files':rows}, indent=2)+'\n')
with tempfile.TemporaryDirectory(prefix='mpk-contract-native-', dir='/private/tmp') as temporary:
    packed = Path(temporary)/'originals.tar.gz'
    with packed.open('wb') as stream, gzip.GzipFile(filename='', mode='wb', fileobj=stream, mtime=0) as zipped, tarfile.open(fileobj=zipped, mode='w|') as tar:
        for row in rows:
            raw = Path(row['path']).read_bytes()
            assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
            info = tarfile.TarInfo(row['member'])
            info.size, info.mode = row['bytes'], row['mode']
            import io
            tar.addfile(info, io.BytesIO(raw))
    chunks = []
    with packed.open('rb') as stream:
        while raw := stream.read(2_097_152):
            path = ARCHIVE/f'originals-{len(chunks):03}.tar.gz.hex'
            path.write_text(raw.hex()+'\n')
            chunks.append({'path':str(path), 'bytes':path.stat().st_size, 'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    reconstructed = Path(temporary)/'reconstructed.tar.gz'
    with reconstructed.open('wb') as stream:
        for chunk in chunks:
            stream.write(bytes.fromhex(Path(chunk['path']).read_text()))
    assert reconstructed.read_bytes() == packed.read_bytes()
    with tarfile.open(reconstructed, 'r:gz') as tar:
        members = tar.getmembers()
        assert [m.name for m in members] == [r['member'] for r in rows]
        for member, row in zip(members, rows):
            raw = tar.extractfile(member).read()
            original = Path(row['path'])
            assert raw == original.read_bytes() and member.mode == original.stat().st_mode & 0o777 == row['mode']
            assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
    manifest = {'status':'passed', 'encoding':'gzip-tar split into lowercase hexadecimal chunks', 'chunks':chunks,
                'files':rows, 'actual_files_compared_literally':len(rows), 'actual_bytes_compared_literally':sum(r['bytes'] for r in rows),
                'actual_driver_prelaunch_sha256':hashlib.sha256((ARCHIVE/'driver.py').read_bytes()).hexdigest()}
    (ARCHIVE/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
shutil.rmtree(SOURCE)
print(json.dumps({k:manifest[k] for k in ['status', 'actual_files_compared_literally', 'actual_bytes_compared_literally']}))
