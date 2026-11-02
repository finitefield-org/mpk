"""Store and independently restore the exact staged test executable."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
OUT = ROOT / 'observer-binary'
RESTORE = Path('/private/tmp/mpk-installed-release-live-restored-observer')
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    build = json.loads((ROOT / 'observer-final/result.json').read_bytes())
    run = json.loads((ROOT / 'live-original/result.json').read_bytes())
    assert build['status'] == run['status'] == 'passed' and run['staged_binary_unchanged']
    source = Path(build['executable']['path'])
    assert seal(source) == build['executable'] and not OUT.exists() and not RESTORE.exists()
    raw = source.read_bytes()
    assert raw[:6] == b'\x7fELF\x02\x01' and int.from_bytes(raw[18:20], 'little') == 62
    OUT.mkdir()
    RESTORE.mkdir(mode=0o700)
    chunks = []
    for index, offset in enumerate(range(0, len(raw), 4 * 1024 * 1024)):
        block = raw[offset:offset + 4 * 1024 * 1024]
        path = OUT / f'{index:05d}.hex'
        path.write_bytes(block.hex().encode('ascii'))
        assert bytes.fromhex(path.read_text()) == block
        chunks.append({'path': str(path.relative_to(ROOT)), 'raw_bytes': len(block),
                       'raw_sha256': hashlib.sha256(block).hexdigest(), 'encoded': seal(path)})
    restored = RESTORE / 'mpk-test-observer'
    with restored.open('xb') as output:
        for chunk in chunks:
            path = ROOT / chunk['path']
            assert seal(path) == chunk['encoded']
            block = bytes.fromhex(path.read_text())
            assert len(block) == chunk['raw_bytes'] and hashlib.sha256(block).hexdigest() == chunk['raw_sha256']
            output.write(block)
    restored.chmod(build['executable']['mode'])
    actual = seal(restored)
    assert all(actual[key] == build['executable'][key] for key in ['bytes', 'sha256', 'mode'])
    assert seal(source) == build['executable']
    receipt = {'status': 'passed', 'selection_reason': 'Losslessly retain and restore the exact '
               'Linux x86 test observer and its original mode; this is not the production CLI.',
               'original': build['executable'], 'restored': actual, 'chunks': chunks,
               'original_binary_unchanged': True, 'native_Linux_release_acceptance': False}
    write(OUT / 'receipt.json', receipt)
    print(json.dumps({'status': receipt['status'], 'bytes': len(raw), 'chunks': len(chunks)}), flush=True)


if __name__ == '__main__':
    main()
