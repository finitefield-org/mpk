"""Compare every staged blob and mode with its current reviewed file bytes."""
from pathlib import Path
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def main():
    head = subprocess.check_output(['/usr/bin/git', 'rev-parse', 'HEAD'], cwd=REPO).decode().strip()
    assert head == '7d37e98882edf86b6f2b537ff4592eca8b589ba4'
    raw = subprocess.check_output(['/usr/bin/git', 'diff', '--cached', '--raw', '--no-abbrev', '--no-renames', '-z'], cwd=REPO)
    tokens = raw.split(b'\0')
    assert tokens[-1] == b'' and (len(tokens)-1) % 2 == 0
    aggregate = hashlib.sha256()
    count = total = 0
    identifiers = set()
    for index in range(0, len(tokens)-1, 2):
        header, encoded = tokens[index:index+2]
        oldmode, newmode, oldhash, oid, status = header.decode().lstrip(':').split()
        assert status in ['A', 'M'] and newmode in ['100644', '100755']
        path = REPO/os.fsdecode(encoded)
        assert path.is_file() and not path.is_symlink()
        metadata = path.stat()
        expected_mode = '100755' if metadata.st_mode & 0o111 else '100644'
        assert newmode == expected_mode
        blob = hashlib.sha1()
        blob.update(f'blob {metadata.st_size}\0'.encode())
        sha = hashlib.sha256()
        count_bytes = 0
        with path.open('rb') as stream:
            while block := stream.read(1024*1024):
                blob.update(block)
                sha.update(block)
                count_bytes += len(block)
        assert count_bytes == metadata.st_size and blob.hexdigest() == oid
        aggregate.update(encoded+b'\0'+str(count_bytes).encode()+b'\0'+sha.digest())
        total += count_bytes
        count += 1
        identifiers.add(oid)
    result = {'status':'passed', 'basis_commit':head, 'staged_files':count, 'staged_bytes':total, 'unique_git_blobs':len(identifiers), 'path_size_sha256_aggregate':aggregate.hexdigest(), 'scope':'Every staged original source, compiler byte, test output, code file and mode; comparison uses the Git blob framing and an independent raw SHA-256 aggregate. The active download directory is not staged.', 'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'audit-partial-stage.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ['status','staged_files','staged_bytes','unique_git_blobs']}))


if __name__ == '__main__':
    main()
