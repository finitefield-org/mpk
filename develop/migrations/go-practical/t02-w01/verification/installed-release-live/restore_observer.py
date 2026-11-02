"""Restore the captured observer from the retained, individually sealed chunks."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    destination = Path(sys.argv[1])
    assert destination.is_absolute() and not destination.exists()
    receipt_path = ROOT / 'observer-binary/receipt.json'
    receipt = json.loads(receipt_path.read_bytes())
    assert receipt['status'] == 'passed' and receipt['original_binary_unchanged']
    destination.mkdir(mode=0o700)
    restored = destination / 'mpk-test-observer'
    with restored.open('xb') as output:
        for chunk in receipt['chunks']:
            path = ROOT / chunk['path']
            assert seal(path) == chunk['encoded']
            raw = bytes.fromhex(path.read_text())
            assert len(raw) == chunk['raw_bytes'] and hashlib.sha256(raw).hexdigest() == chunk['raw_sha256']
            output.write(raw)
    restored.chmod(receipt['original']['mode'])
    observed = seal(restored)
    assert all(observed[key] == receipt['original'][key] for key in ['bytes', 'sha256', 'mode'])
    result = {'status': 'passed', 'receipt': seal(receipt_path), 'restored': observed,
              'all_original_bytes_and_mode_match': True, 'native_Linux_release_acceptance': False}
    write(destination / 'restore-result.json', result)
    print(json.dumps(result), flush=True)


if __name__ == '__main__':
    main()
