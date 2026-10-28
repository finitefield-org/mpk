"""Restore the two exact archives and scan report into a new directory."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
OLD = ROOT.parent/'analysis-build-list'


def restore(encoded, expected_bytes, expected_sha256, output, mode):
    raw = bytes.fromhex(encoded.read_text())
    assert len(raw) == expected_bytes and hashlib.sha256(raw).hexdigest() == expected_sha256
    output.parent.mkdir(parents=True, exist_ok=True)
    assert not output.exists()
    output.write_bytes(raw)
    output.chmod(mode)
    assert output.read_bytes() == raw
    return {'path': str(output.resolve()), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def main():
    destination = Path(sys.argv[1]); assert destination.is_absolute() and not destination.exists(); destination.mkdir(mode=0o700)
    data = json.loads((ROOT/'archive-storage.json').read_bytes()); assert data['status'] == 'passed'
    outputs = []
    for name, record in zip(['go-checks', 'prelaunch'], data['files']):
        encoded = OLD/name/'inputs.tar.gz.hex'
        encoded_raw = encoded.read_bytes(); assert len(encoded_raw) == record['encoded']['bytes'] and hashlib.sha256(encoded_raw).hexdigest() == record['encoded']['sha256']
        original = record['historical_receipt']
        outputs.append(restore(encoded, original['bytes'], original['sha256'], destination/name/'inputs.tar.gz', original['mode']))
    findings = data['findings']; encoded = ROOT/'archive-scan-findings.json.hex'
    outputs.append(restore(encoded, findings['raw_bytes'], findings['raw_sha256'], destination/'archive-scan-findings.json', 0o600))
    print(json.dumps({'status': 'passed', 'outputs': outputs, 'exact_recovery': True}))


if __name__ == '__main__':
    main()
