"""Independently audit all native input bytes, identities and scoped results."""
from pathlib import Path
import hashlib
import json
import tarfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    native = ROOT / 'native'
    rows = json.loads((native / 'native.json').read_bytes())
    assert len(rows) == 36 and len({row['name'] for row in rows}) == 36
    assert sum(row['accepted'] for row in rows) == 10
    raw_files = set()
    for row in rows:
        path = native / 'original-bytes' / f"{row['sha256']}.raw"
        raw = path.read_bytes()
        assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
        raw_files.add(path.name)
        obj = {'schema': 'mpk.go.source_object.v1', 'capability': 'analysis-frontend', 'path': 'vendor/modules.txt', 'kind': row['kind'], 'raw_sha256': row['sha256'], 'bytes': len(raw)}
        canonical = json.dumps(obj, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0' + canonical).hexdigest() == row['source_id']
        if row['accepted']:
            assert row['packages'] == sorted(set(row['packages']))
        else:
            assert row['packages'] is None
    assert raw_files == {p.name for p in (native / 'original-bytes').glob('*.raw')}
    by_name = {row['name']: row for row in rows}
    original = (REPO / 'go-tools/go2vir/vendor/modules.txt').read_bytes()
    first = by_name['original-complete-vendor-manifest']
    assert (native / 'original-bytes' / f"{first['sha256']}.raw").read_bytes() == original
    packages = sorted(line for line in original.decode().splitlines() if line and not line.startswith('#'))
    assert len(packages) == 29 and first['packages'] == packages
    limit = by_name['complete-original-plus-maximum-packages']
    over = by_name['complete-original-plus-one-over-maximum']
    limit_raw = (native / 'original-bytes' / f"{limit['sha256']}.raw").read_bytes()
    over_raw = (native / 'original-bytes' / f"{over['sha256']}.raw").read_bytes()
    assert limit_raw.startswith(original) and len(limit['packages']) == 32768 and limit['accepted']
    assert over_raw == limit_raw + b'golang.org/x/tools/testing/over_limit\n' and not over['accepted']
    before = json.loads((ROOT / 'prelaunch/result.json').read_bytes())
    assert before['status'] == 'passed' and before['inputs_unchanged'] and len(before['registered_sdk']) == 5577
    with tarfile.open(ROOT / 'prelaunch/inputs.tar.gz') as archive:
        retained = archive.extractfile('go-tools/go2vir/vendor/modules.txt').read()
    assert retained == original
    for name in ['native', 'vendor-selected', 'vendor-clippy-check', 'go-checks', 'rust-format-check']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result.get('consumed_inputs_unchanged', True)
    results = (ROOT / 'vendor-selected/stdout.log').read_text()
    assert 'test result: ok. 1 passed;' in results and 'test result: ok. 8 passed;' in results
    events = [json.loads(line) for line in (native / 'stdout.log').read_bytes().splitlines()]
    passed = {event.get('Test') for event in events if event.get('Action') == 'pass'}
    assert {'TestPracticalAnalysisVendorDifferential', 'TestPracticalAnalysisVendorManifestRequiresCompleteExactOwners'} <= passed
    summary = {'status': 'passed', 'driver': seal(Path(__file__).resolve()), 'native_observations': seal(native / 'native.json'), 'native_result': seal(native / 'result.json'), 'rust_result': seal(ROOT / 'vendor-selected/result.json'), 'original_manifest': seal(REPO / 'go-tools/go2vir/vendor/modules.txt'), 'cases': 36, 'accepted': 10, 'rejected': 26, 'distinct_original_byte_files': len(raw_files), 'original_packages': 29, 'maximum_packages_accepted': 32768, 'one_over_maximum_rejected': True, 'complete_original_manifest_preserved_in_limit_case': True, 'all_native_source_ids_independently_recomputed': True, 'source_to_binary_correspondence_claimed': False, 'registered_context_claimed': False, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT / 'inspection.json'
    assert not out.exists()
    out.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: summary[key] for key in ['status', 'cases', 'accepted', 'rejected', 'distinct_original_byte_files', 'original_packages', 'maximum_packages_accepted']}))


if __name__ == '__main__':
    main()
