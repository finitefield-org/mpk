"""Audit the whole Go capture, ZIP members and scoped comparison results."""
from pathlib import Path
import base64
import hashlib
import io
import json
import tarfile
import zipfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    native = ROOT / 'native'
    packet = json.loads((native / 'native.json').read_bytes())
    records = packet['snapshot_sources']
    assert len(records) == 213 and len(packet['vendor_source_ids']) == 133 and len(packet['archives']) == 3
    sources, paths, used_files = {}, {}, set()
    with tarfile.open(ROOT / 'prelaunch/inputs.tar.gz') as original:
        for row in records:
            obj = row['object']
            raw = (native / 'original-bytes' / row['raw_file']).read_bytes()
            assert len(raw) == obj['bytes'] and hashlib.sha256(raw).hexdigest() == obj['raw_sha256']
            assert row['raw_file'] == obj['raw_sha256'] + '.raw'
            identity = {key: value for key, value in obj.items() if key != 'id'}
            canonical = json.dumps(identity, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
            assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0' + canonical).hexdigest() == obj['id']
            assert obj['id'] not in sources
            sources[obj['id']] = (obj, raw)
            paths[(obj['capability'], obj['path'])] = (obj, raw)
            used_files.add(row['raw_file'])
            if obj['capability'] == 'analysis-frontend':
                original_path = 'go-tools/go2vir/' + obj['path']
            else:
                assert obj['capability'] == 'analysis-archives'
                original_path = 'go-tools/go2vir/testdata/practical-analysis-cache/' + obj['path']
            assert original.extractfile(original_path).read() == raw
    assert packet['manifest_source_id'] in sources
    assert len(set(packet['vendor_source_ids'])) == 133 and all(key in sources for key in packet['vendor_source_ids'])
    archive_members = 0
    for row in packet['archives']:
        obj, raw = sources[row['source_id']]
        assert obj['kind'] == 'auxiliary'
        with zipfile.ZipFile(io.BytesIO(raw)) as zipped:
            members = {}
            for item in zipped.infolist():
                data = zipped.read(item)
                assert len(data) == item.file_size and item.filename.startswith(f"{row['module']}@{row['version']}/") and not item.is_dir()
                path = item.filename.removeprefix(f"{row['module']}@{row['version']}/")
                assert path not in members
                members[path] = {'Path': path, 'Bytes': len(data), 'RawSHA256': hashlib.sha256(data).hexdigest()}
            assert [members[path] for path in sorted(members)] == row['members']
            summary = ''.join(f"{members[path]['RawSHA256']}  {row['module']}@{row['version']}/{path}\n" for path in sorted(members)).encode()
            assert 'h1:' + base64.b64encode(hashlib.sha256(summary).digest()).decode() == row['checksum']
            archive_members += len(members)
    assert archive_members == 1751 and packet['original_differences'] == 1
    changed = packet['changed_vendor_source']
    changed_raw = (native / 'original-bytes' / changed['raw_file']).read_bytes()
    original_raw = paths[('analysis-frontend', 'vendor/golang.org/x/tools/go/packages/packages.go')][1]
    assert changed_raw == original_raw + b'\n' and hashlib.sha256(changed_raw).hexdigest() == changed['object']['raw_sha256']
    used_files.add(changed['raw_file'])
    changed = packet['changed_archive_source']
    changed_raw = (native / 'original-bytes' / changed['raw_file']).read_bytes()
    original_raw = paths[('analysis-archives', 'golang.org/x/tools/@v/v0.36.0.zip')][1]
    assert changed_raw == original_raw[:-1] + bytes([original_raw[-1] ^ 1]) and hashlib.sha256(changed_raw).hexdigest() == changed['object']['raw_sha256']
    used_files.add(changed['raw_file'])
    assert used_files == {p.name for p in (native / 'original-bytes').glob('*.raw')}
    controls = {row['name']: row['accepted'] for row in packet['controls']}
    assert controls == {'changed-vendor-go': False, 'missing-only-package-file': False, 'unknown-asset': False, 'omitted-one-of-several-go-files': True, 'changed-todo': True, 'changed-archive': False}
    for name in ['prelaunch', 'native', 'archive-selected', 'archive-clippy-final', 'go-format-write', 'go-checks', 'rust-format-final-check']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result.get('consumed_inputs_unchanged', True)
    output = (ROOT / 'archive-selected/stdout.log').read_text()
    assert 'test result: ok. 2 passed;' in output and 'test result: ok. 1 passed;' in output
    summary = {'status': 'passed', 'driver': seal(Path(__file__).resolve()), 'native': seal(native / 'native.json'), 'go_result': seal(native / 'result.json'), 'rust_result': seal(ROOT / 'archive-selected/result.json'), 'original_sources': len(records), 'distinct_original_and_control_raw_files': len(used_files), 'vendor_files': 133, 'archives': 3, 'complete_archive_members': archive_members, 'original_asset_differences': 1, 'controls': controls, 'all_source_ids_independently_recomputed': True, 'all_captured_original_bytes_match_prelaunch': True, 'supplied_file_list_proves_complete_namespace': False, 'go_syntax_or_source_binary_proof_claimed': False, 'registered_context_claimed': False, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT / 'inspection.json'
    assert not out.exists()
    out.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: summary[key] for key in ['status', 'original_sources', 'distinct_original_and_control_raw_files', 'vendor_files', 'archives', 'complete_archive_members']}))


if __name__ == '__main__':
    main()
