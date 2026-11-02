"""Audit the original snapshot and the exact shared-budget controls."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('prior_snapshot_inspection', ROOT.parent / 'analysis-namespace/inspect_snapshot.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)


def seal(path):
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def main():
    native = ROOT / 'native'
    packet = json.loads((native / 'native.json').read_bytes())
    previous = json.loads((ROOT.parent / 'analysis-namespace/native/native.json').read_bytes())
    archive = json.loads((ROOT.parent / 'analysis-archive/native/native.json').read_bytes())
    assert packet['original_sources'] == 213 and packet['vendor_files'] == 133
    assert packet['source_ids'] == [row['object']['id'] for row in previous['original_sources']]
    assert packet['original_root_authenticated'] is False
    assert sum(item['namespace_entries'] for item in archive['archives']) == 2403
    cases = {}
    for case in packet['cases']:
        raw = (native / case['snapshot_file']).read_bytes()
        wire = json.loads(raw)
        assert prior.canonical(wire) == raw and prior.native_digest(wire) == wire['snapshot_sha256']
        body = {key: value for key, value in wire.items() if key != 'content_sha256'}
        assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0' + prior.canonical(body)).hexdigest() == wire['content_sha256']
        assert [obj['id'] for obj in wire['objects']] == packet['source_ids']
        assert case['directory_entries'] == sum(len(d['entries']) for d in wire['directories'])
        assert case['archive_entries'] == 2403
        cases[case['name']] = (case, wire)
    assert {name:case['accepted'] for name, (case, _) in cases.items()} == {'original': True, 'exact-shared-limit': True, 'one-over-shared-limit': False}
    def directories(wire):
        return {(row['capability'], row['path']): [(entry['name_hex'], entry['type_bits']) for entry in row['entries']] for row in wire['directories']}
    original = directories(cases['original'][1]); key = ('analysis-frontend', '')
    for name in ['exact-shared-limit', 'one-over-shared-limit']:
        case, wire = cases[name]; current = directories(wire)
        assert {k:v for k,v in current.items() if k != key} == {k:v for k,v in original.items() if k != key}
        before, after = set(original[key]), set(current[key])
        padding = case['directory_entries'] - cases['original'][0]['directory_entries']
        assert before <= after and after - before == {(f'zz_mpk_budget_{i:06d}.data'.encode().hex(), 0) for i in range(padding)}
        assert case['directory_entries'] + 2403 == 131072 + (name == 'one-over-shared-limit')
    for name in ['prelaunch', 'native', 'go-format-write', 'go-checks', 'budget-check', 'budget-tests', 'budget-clippy', 'rust-format']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result.get('consumed_inputs_unchanged', True)
    output = (ROOT / 'budget-tests/stdout.log').read_text()
    assert output.count('test result: ok. 2 passed;') == 2 and 'test result: ok. 1 passed;' in output
    summary = {'status': 'passed', 'driver': seal(Path(__file__).resolve()), 'native': seal(native / 'native.json'), 'go_result': seal(native / 'result.json'), 'rust_result': seal(ROOT / 'budget-tests/result.json'), 'original_sources': 213, 'vendor_files': 133, 'archive_namespace_entries': 2403, 'controls': {name:{'directory_entries':case['directory_entries'],'archive_entries':case['archive_entries'],'accepted':case['accepted']} for name,(case,_) in cases.items()}, 'all_snapshot_digests_recomputed': True, 'all_original_source_objects_retained': True, 'extra_padding_is_control_only': True, 'all_other_capture_archive_budgets_proved': False, 'original_root_authenticated': False, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT / 'inspection.json'; assert not out.exists()
    out.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'status': summary['status'], 'original_sources': 213, 'controls': len(cases), 'archive_namespace_entries': 2403}))


if __name__ == '__main__':
    main()
