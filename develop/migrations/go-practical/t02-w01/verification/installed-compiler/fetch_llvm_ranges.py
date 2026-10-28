"""Resume the failed fixed LLVM archive retrieval using independently checked ranges."""
from pathlib import Path
import concurrent.futures
import hashlib
import json
import os
import sys
import time
import traceback
import urllib.request

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
WORK = Path('/private/tmp/mpk-go-installed-205-fs/llvm-range-download')
DEST = Path('/private/tmp/mpk-go-installed-205-fs/originals/downloads')
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import rust_build_inputs as rust


def main():
    out = ROOT / 'llvm-ranged-retrieval'
    out.mkdir(exist_ok=False)
    WORK.mkdir(exist_ok=False)
    vector_path = REPO / 'develop/specs/vectors/rust-build-inputs-v0.json'
    vector = json.loads(vector_path.read_bytes())
    expected = vector['native_origins']['llvm']
    inputs = [seal(p) for p in [Path(__file__).resolve(), ROOT/'run_scoped.py', REPO/'scripts/rust_build_inputs.py', vector_path, ROOT/'rust-case-sensitive-reproduction/result.json', ROOT/'rust-case-sensitive-reproduction/downloads.json']]
    url = rust.LLVM_URL
    request = urllib.request.Request(url, method='HEAD', headers={'User-Agent':'mpk-rust-build-inputs-v0', 'Accept-Encoding':'identity'})
    with urllib.request.urlopen(request, timeout=90) as response:
        size = int(response.headers['Content-Length'])
        head = {'status': response.status, 'headers': dict(response.headers.items()), 'final_url': response.url}
    assert 0 < size <= 2_147_483_648
    chunk_size = 8 * 1024 * 1024
    ranges = [(i, start, min(size, start + chunk_size) - 1) for i, start in enumerate(range(0, size, chunk_size))]
    write(out/'inputs.json', {'inputs': inputs, 'url':url, 'expected_sha256':expected['sha256'], 'expected_archive':expected['archive'], 'head':head, 'size':size, 'ranges':ranges, 'scope':'Resume after a terminal socket timeout. Every range is retained, content-range checked, reassembled in exact order and checked against the original frozen full-archive SHA-256 before use. Range size is transfer framing, not a compiler, source, proof or artifact limit.'})
    report = {'status':'running', 'inputs':seal(out/'inputs.json'), 'native_Linux_release_acceptance':False}
    write(out/'result.json', report)
    start_time = time.monotonic()
    results = []

    def fetch(row):
        index, start, end = row
        final = WORK/f'{index:04d}.part'
        attempts = []
        for attempt in range(1, 6):
            temporary = WORK/f'{index:04d}.{attempt}.partial'
            try:
                req = urllib.request.Request(url, headers={'User-Agent':'mpk-rust-build-inputs-v0', 'Accept-Encoding':'identity', 'Range':f'bytes={start}-{end}'})
                with urllib.request.urlopen(req, timeout=90) as response, temporary.open('xb') as output:
                    assert response.status == 206 and response.headers.get('Content-Range') == f'bytes {start}-{end}/{size}'
                    observed = 0
                    while True:
                        block = response.read(1024*1024)
                        if not block:
                            break
                        observed += len(block)
                        assert observed <= end-start+1
                        output.write(block)
                assert observed == end-start+1
                os.replace(temporary, final)
                return {'index':index, 'start':start, 'end':end, 'status':'passed', 'attempts':attempts, 'part':seal(final)}
            except BaseException as error:
                attempts.append({'attempt':attempt, 'error':repr(error)})
                if temporary.exists():
                    temporary.unlink()
        return {'index':index, 'start':start, 'end':end, 'status':'failed', 'attempts':attempts}

    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            futures = [pool.submit(fetch, row) for row in ranges]
            for future in concurrent.futures.as_completed(futures):
                results.append(future.result())
                write(out/'ranges.json', sorted(results, key=lambda r:r['index']))
        assert len(results) == len(ranges) and all(r['status']=='passed' for r in results)
        temporary = DEST/(expected['archive']+'.verified-partial')
        final = DEST/expected['archive']
        assert not temporary.exists() and not final.exists()
        digest = hashlib.sha256()
        count = 0
        with temporary.open('xb') as output:
            for row in sorted(results, key=lambda r:r['index']):
                part = Path(row['part']['path'])
                assert seal(part) == row['part']
                with part.open('rb') as stream:
                    while block := stream.read(1024*1024):
                        count += len(block)
                        digest.update(block)
                        output.write(block)
        assert count == size and digest.hexdigest() == expected['sha256']
        os.replace(temporary, final)
        report.update(status='passed', complete_original_archive=seal(final), part_count=len(results))
    except BaseException as error:
        report.update(status='failed', error=repr(error), traceback=traceback.format_exc())
        raise
    finally:
        report.update(seconds=time.monotonic()-start_time, inputs_unchanged=inputs == [seal(p['path']) for p in inputs], ranges=results)
        write(out/'result.json',report)
    print(json.dumps({k:report[k] for k in ['status','seconds','inputs_unchanged','complete_original_archive']}))


if __name__ == '__main__':
    main()
