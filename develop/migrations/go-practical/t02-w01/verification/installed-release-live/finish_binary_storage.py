"""Retire the generated binary copy after verifying its lossless restoration."""
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    receipt_path = ROOT / 'observer-binary/receipt.json'
    receipt = json.loads(receipt_path.read_bytes())
    stage = json.loads((ROOT / 'installed-stage-retry/result.json').read_bytes())
    assert receipt['status'] == stage['status'] == 'passed'
    original = Path(receipt['original']['path'])
    restored = Path(receipt['restored']['path'])
    installed = Path(stage['installed_executable']['path'])
    assert seal(original) == receipt['original'] and seal(restored) == receipt['restored']
    assert seal(installed) == stage['installed_executable']
    for key in ['bytes', 'sha256', 'mode']:
        assert receipt['original'][key] == receipt['restored'][key] == stage['installed_executable'][key]
    assert all(seal(ROOT / chunk['path']) == chunk['encoded'] for chunk in receipt['chunks'])
    out = ROOT / 'observer-binary/storage.json'
    assert not out.exists()
    original.unlink()
    assert not original.exists() and seal(restored) == receipt['restored'] and seal(installed) == stage['installed_executable']
    write(out, {'status': 'passed', 'selection_reason': 'Retire only the generated local copy after '
          'lossless encoded storage and verified restoration of its exact bytes, original path '
          'and mode; the installed test copy remains intact.', 'receipt': seal(receipt_path),
          'original_path': str(original), 'restored': receipt['restored'],
          'installed_copy': stage['installed_executable'], 'original_copy_retired': True})
    print(json.dumps({'status': 'passed', 'original_copy_retired': True}), flush=True)


if __name__ == '__main__':
    main()
