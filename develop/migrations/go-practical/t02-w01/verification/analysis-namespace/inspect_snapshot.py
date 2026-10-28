"""Independently audit the original snapshot, controls and scoped results."""
from pathlib import Path
import hashlib
import json
import tarfile

ROOT = Path(__file__).resolve().parent


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def native_digest(wire):
    digest = hashlib.sha256(b'MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0')
    def num(value):
        digest.update(value.to_bytes(8, 'little'))
    def string(value):
        num(len(value))
        digest.update(value)
    num(len(wire['capabilities']))
    for capability in wire['capabilities']:
        string(capability.encode())
    num(len(wire['directories']))
    for directory in wire['directories']:
        string(directory['capability'].encode())
        string(directory['path'].encode())
        num(len(directory['entries']))
        for entry in directory['entries']:
            string(bytes.fromhex(entry['name_hex']))
            num(entry['type_bits'])
    num(len(wire['objects']))
    for obj in wire['objects']:
        string(obj['id'].encode())
    return digest.hexdigest()


def main():
    native = ROOT / 'native'
    packet = json.loads((native / 'native.json').read_bytes())
    records = packet['original_sources']
    assert len(records) == 213 and len(packet['archive_source_ids']) == 3
    assert packet['authenticated_original_root_claimed'] is False
    sources, raw_files = {}, set()
    with tarfile.open(ROOT / 'prelaunch/inputs.tar.gz') as original:
        for row in records:
            obj = row['object']
            raw = (native / 'original-bytes' / row['raw_file']).read_bytes()
            assert len(raw) == obj['bytes'] and hashlib.sha256(raw).hexdigest() == obj['raw_sha256']
            assert row['raw_file'] == obj['raw_sha256'] + '.raw'
            identity = {key: value for key, value in obj.items() if key != 'id'}
            assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0' + canonical(identity)).hexdigest() == obj['id']
            assert obj['id'] not in sources
            sources[obj['id']] = obj
            raw_files.add(row['raw_file'])
            if obj['capability'] == 'analysis-frontend':
                original_path = 'go-tools/go2vir/' + obj['path']
            else:
                assert obj['capability'] == 'analysis-archives'
                original_path = 'go-tools/go2vir/testdata/practical-analysis-cache/' + obj['path']
            assert original.extractfile(original_path).read() == raw
    assert raw_files == {path.name for path in (native / 'original-bytes').glob('*.raw')}
    old = json.loads((ROOT.parent / 'analysis-archive/native/native.json').read_bytes())
    assert {row['object']['id'] for row in old['snapshot_sources']} == set(sources)
    assert all((native / 'original-bytes' / name).read_bytes() == (ROOT.parent / 'analysis-archive/native/original-bytes' / name).read_bytes() for name in raw_files)
    cases = {}
    for case in packet['cases']:
        raw = bytes.fromhex(case['snapshot_hex'])
        wire = json.loads(raw)
        assert canonical(wire) == raw
        assert native_digest(wire) == wire['snapshot_sha256']
        body = {key: value for key, value in wire.items() if key != 'content_sha256'}
        assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0' + canonical(body)).hexdigest() == wire['content_sha256']
        assert [obj['id'] for obj in wire['objects']] == case['source_ids']
        assert all(sources[obj['id']] == obj for obj in wire['objects'])
        cases[case['name']] = (case, wire)
    assert {key: (value[0]['accepted'], value[0]['files']) for key, value in cases.items()} == {
        'original-complete-snapshot': (True, 133), 'unread-file': (False, 0),
        'unread-directory': (False, 0), 'unknown-unread-file': (False, 0),
        'symbolic-link': (False, 0), 'coherently-pruned-snapshot': (True, 132),
    }
    original = cases['original-complete-snapshot'][1]
    original_ids = {obj['id'] for obj in original['objects']}
    target = 'vendor/golang.org/x/tools/go/packages/doc.go'
    directory = target.rsplit('/', 1)[0]
    missing = {key for key, obj in sources.items() if obj['capability'] == 'analysis-frontend' and obj['path'] == target}
    assert len(missing) == 1
    removed_directory = {key for key, obj in sources.items() if obj['capability'] == 'analysis-frontend' and obj['path'].startswith(directory + '/')}
    def directories(wire):
        return {(d['capability'], d['path']): [(e['name_hex'], e['type_bits']) for e in d['entries']] for d in wire['directories']}
    base = directories(original)
    key = ('analysis-frontend', directory)
    for name, (_, wire) in cases.items():
        current_ids = {obj['id'] for obj in wire['objects']}
        expected_removed = missing if name in ['unread-file', 'coherently-pruned-snapshot'] else removed_directory if name == 'unread-directory' else set()
        assert original_ids - current_ids == expected_removed and not current_ids - original_ids
        current = directories(wire)
        if name == 'unread-directory':
            assert key in base and key not in current and {k:v for k,v in base.items() if k != key} == current
        elif name in ['unknown-unread-file', 'symbolic-link', 'coherently-pruned-snapshot']:
            assert {k:v for k,v in base.items() if k != key} == {k:v for k,v in current.items() if k != key}
            old_entries, new_entries = set(base[key]), set(current[key])
            removed = {(b'doc.go'.hex(), 0)} if name == 'coherently-pruned-snapshot' else set()
            added = {(b'ghost.go'.hex(), 0)} if name == 'unknown-unread-file' else {(b'symlink.go'.hex(), 1 << 27)} if name == 'symbolic-link' else set()
            assert old_entries - new_entries == removed and new_entries - old_entries == added
        else:
            assert base == current
    for name in ['prelaunch', 'native', 'go-format-write', 'go-checks', 'rust-format', 'namespace-check', 'namespace-clippy', 'namespace-tests', 'snapshot-tests']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result.get('consumed_inputs_unchanged', True)
    rust = (ROOT / 'namespace-tests/stdout.log').read_text()
    assert rust.count('test result: ok. 2 passed;') == 2
    snapshot_output = (ROOT / 'snapshot-tests/stdout.log').read_text()
    assert 'test result: ok. 3 passed;' in snapshot_output
    summary = {'status': 'passed', 'driver': seal(Path(__file__).resolve()), 'native': seal(native / 'native.json'), 'native_result': seal(native / 'result.json'), 'rust_result': seal(ROOT / 'namespace-tests/result.json'), 'original_sources': len(records), 'distinct_original_raw_files': len(raw_files), 'vendor_files': 133, 'archives': 3, 'archive_members_verified_by_rust': 1751, 'native_go_rust_observations': len(cases), 'controls': {name: {'accepted': case['accepted'], 'files': case['files']} for name, (case, _) in cases.items()}, 'all_snapshot_and_source_digests_independently_recomputed': True, 'all_original_bytes_match_prelaunch_and_previous_capture': True, 'coherent_pruning_authenticates_original_root': False, 'source_syntax_or_binary_proof_claimed': False, 'registered_context_claimed': False, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT / 'inspection.json'
    assert not out.exists()
    out.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: summary[key] for key in ['status', 'original_sources', 'distinct_original_raw_files', 'vendor_files', 'native_go_rust_observations']}))


if __name__ == '__main__':
    main()
