"""Recover the exact pinned Rust inputs on a separate case-sensitive volume.

The resulting bytes are build inputs for a local Linux-emulation observer; they
do not establish a native Linux release, launcher authority or program proof.
"""
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
import hashlib
import json
import os
import shutil
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-combined-recovery-20261029-fs')
WORK = VOLUME / 'original-recovery'
DOWNLOADS = WORK / 'downloads'
PARTS = WORK / 'llvm-ranges'
OUT = ROOT / 'recovery-downloads'
PRIOR = ROOT.parent / 'installed-compiler'
sys.dont_write_bytecode = True
sys.path.insert(0, str(PRIOR))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import rust_build_inputs as rust


def main():
    assert Path.cwd() == REPO and VOLUME.is_mount()
    assert not WORK.exists() and not OUT.exists()
    WORK.mkdir()
    DOWNLOADS.mkdir()
    PARTS.mkdir()
    OUT.mkdir()
    vector_path = REPO / 'develop/specs/vectors/rust-build-inputs-v0.json'
    prior_path = PRIOR / 'llvm-ranged-retrieval/inputs.json'
    crate_path = PRIOR / 'rust-crate-cache-seed/result.json'
    vector = json.loads(vector_path.read_bytes())
    prior = json.loads(prior_path.read_bytes())
    crate_receipt = json.loads(crate_path.read_bytes())
    assert crate_receipt['status'] == 'passed'
    files = [Path(__file__).resolve(), vector_path, prior_path, crate_path,
             REPO / 'scripts/rust_build_inputs.py', PRIOR / 'run_scoped.py']
    before = [seal(path) for path in files]
    llvm = vector['native_origins']['llvm']
    assert prior['expected_sha256'] == llvm['sha256']
    write(OUT / 'inputs.json', {
        'inputs': before, 'volume': str(VOLUME), 'download_root': str(DOWNLOADS),
        'reason': 'Recover the full pinned Rust archives and local crate cache '
                  'without replacing registered files, changing source or '
                  'weakening any compiler, source or proof limits.',
        'llvm_size': prior['size'], 'llvm_ranges': prior['ranges'],
    })
    results = []
    started = time.monotonic()
    try:
        for row in crate_receipt['archives']:
            source = Path(row['source']['path'])
            assert seal(source) == row['source']
            destination = DOWNLOADS / source.name
            assert not destination.exists()
            shutil.copyfile(source, destination)
            assert seal(destination)['sha256'] == row['expected_sha256']
            assert seal(source) == row['source']
        print(json.dumps({'phase': 'crates_verified', 'count': len(crate_receipt['archives'])}), flush=True)

        def archive(record):
            name = {'clippy-preview': 'clippy', 'llvm-tools-preview': 'llvm-tools',
                    'rustfmt-preview': 'rustfmt'}.get(record['name'], record['name'])
            filename = f"{name}-nightly-{record['target']}.tar.xz"
            destination = DOWNLOADS / filename
            rust.download(f'{rust.RUST_DIST_ROOT}/{filename}', record['sha256'], destination)
            return {'kind': 'rust', 'filename': filename, 'file': seal(destination)}

        def part(row):
            index, first, last = row
            destination = PARTS / f'{index:04d}.part'
            errors = []
            for attempt in range(1, 6):
                temporary = PARTS / f'{index:04d}.{attempt}.partial'
                try:
                    request = urllib.request.Request(
                        prior['url'], headers={'User-Agent': 'mpk-rust-build-inputs-v0',
                        'Accept-Encoding': 'identity', 'Range': f'bytes={first}-{last}'})
                    count = 0
                    with urllib.request.urlopen(request, timeout=90) as response, temporary.open('xb') as output:
                        assert response.status == 206
                        assert response.headers.get('Content-Range') == f'bytes {first}-{last}/{prior["size"]}'
                        while True:
                            block = response.read(1024 * 1024)
                            if not block:
                                break
                            count += len(block)
                            assert count <= last - first + 1
                            output.write(block)
                    assert count == last - first + 1
                    os.replace(temporary, destination)
                    return {'kind': 'llvm-range', 'index': index, 'first': first,
                            'last': last, 'attempt_errors': errors, 'file': seal(destination)}
                except Exception as error:
                    errors.append(repr(error))
            raise RuntimeError(f'fixed LLVM range {index} failed: {errors}')

        jobs = []
        with ThreadPoolExecutor(max_workers=8) as pool:
            jobs.extend(pool.submit(archive, row) for row in vector['rust_distribution']['archives'])
            jobs.extend(pool.submit(part, row) for row in prior['ranges'])
            for future in as_completed(jobs):
                results.append(future.result())
                if len(results) % 10 == 0 or len(results) == len(jobs):
                    print(json.dumps({'phase': 'pinned_downloads', 'finished': len(results),
                                      'total': len(jobs), 'seconds': round(time.monotonic()-started)}), flush=True)
        rows = sorted((r for r in results if r['kind'] == 'llvm-range'), key=lambda r: r['index'])
        assert len(rows) == len(prior['ranges'])
        destination = DOWNLOADS / llvm['archive']
        digest = hashlib.sha256()
        size = 0
        with destination.open('xb') as output:
            for row in rows:
                path = Path(row['file']['path'])
                assert seal(path) == row['file']
                with path.open('rb') as stream:
                    while block := stream.read(1024 * 1024):
                        size += len(block)
                        digest.update(block)
                        output.write(block)
        assert size == prior['size'] and digest.hexdigest() == llvm['sha256']
        assert before == [seal(path) for path in files]
        write(OUT / 'result.json', {
            'status': 'passed', 'seconds': time.monotonic()-started,
            'input_files_unchanged': True, 'original_crates': len(crate_receipt['archives']),
            'rust_archives': [r for r in results if r['kind'] == 'rust'],
            'llvm_ranges': rows, 'complete_llvm': seal(destination),
            'native_Linux_release_acceptance': False,
            'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
        })
        print(json.dumps({'status': 'passed', 'seconds': round(time.monotonic()-started)}), flush=True)
    except BaseException as error:
        write(OUT / 'result.json', {'status': 'failed', 'error': repr(error),
              'seconds': time.monotonic()-started, 'completed': results,
              'input_files_unchanged': before == [seal(path) for path in files],
              'native_Linux_release_acceptance': False})
        raise


if __name__ == '__main__':
    main()
