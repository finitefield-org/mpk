"""Preserve the exact completed test-only observer binary and its mode."""
from pathlib import Path
import hashlib
import json
import shutil
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
binary = Path('/private/tmp/mpk-go-source-root-matching/target/debug/examples/go_practical_live_combined_roots')
preflight = json.loads((ROOT / 'scoped-final/result.json').read_bytes())
assert preflight['status'] == 'passed' and preflight['original_inputs_unchanged'] and preflight['checks']['test-observer-build']['status'] == 'passed'
raw = binary.read_bytes()
mode = binary.stat().st_mode & 0o777
assert raw[:6] == b'\x7fELF\x02\x01' and int.from_bytes(raw[18:20], 'little') == 183 and mode & 0o111
out = ROOT / 'binary-final'
work = Path('/private/tmp/mpk-installed-combined-test-observer-final')
assert not out.exists() and not work.exists()
out.mkdir(mode=0o700)
work.mkdir(mode=0o700)
copy = work / binary.name
shutil.copyfile(binary, copy)
copy.chmod(mode)
assert copy.read_bytes() == raw and copy.stat().st_mode & 0o777 == mode
chunks = []
for index, offset in enumerate(range(0, len(raw), 4 * 1024 * 1024)):
    block = raw[offset:offset + 4 * 1024 * 1024]
    path = out / (f'{index:05d}.hex')
    path.write_bytes(block.hex().encode('ascii'))
    assert bytes.fromhex(path.read_text()) == block
    encoded = path.read_bytes()
    chunks.append({'path': str(path.relative_to(ROOT)), 'raw_bytes': len(block), 'raw_sha256': hashlib.sha256(block).hexdigest(), 'encoded_bytes': len(encoded), 'encoded_sha256': hashlib.sha256(encoded).hexdigest()})
assert binary.read_bytes() == raw and binary.stat().st_mode & 0o777 == mode
result = {'status': 'passed', 'selection_reason': 'Retain the exact Linux arm64 observer built with test cfg, including original binary bytes and mode; the test snapshot does not establish installed-root authority.', 'original_path': str(binary), 'runtime_copy': str(copy), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': mode, 'chunks': chunks, 'original_binary_unchanged': True, 'native_Linux_release_acceptance': False}
(out / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'bytes': len(raw), 'chunks': len(chunks)}), flush=True)
