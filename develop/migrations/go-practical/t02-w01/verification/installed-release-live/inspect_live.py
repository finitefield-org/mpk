"""Recompute the staged installed-root and original capture evidence."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
spec = importlib.util.spec_from_file_location('go_snapshot_prior', ROOT.parent / 'analysis-namespace/inspect_snapshot.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)


def seal(path):
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def wire(packet):
    raw = bytes.fromhex(packet['snapshot_hex'])
    data = json.loads(raw)
    assert prior.canonical(data) == raw and prior.native_digest(data) == data['snapshot_sha256']
    body = {key: value for key, value in data.items() if key != 'content_sha256'}
    assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0' + prior.canonical(body)).hexdigest() == data['content_sha256']
    return data


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum()
    restored_evidence = Path(sys.argv[2])
    restored_binary = Path(sys.argv[3])
    assert restored_evidence.is_absolute() and restored_binary.is_absolute()
    run = ROOT / epoch
    result = json.loads((run / 'result.json').read_bytes())
    assert result['status'] == 'passed' and result['original_inputs_unchanged'] and result['staged_binary_unchanged'] and result['registered_sdk_unchanged']
    assert result['complete_staged_bundles_unchanged'] and result['original_go_cache_unchanged']
    evidence = restored_evidence / 'originals/versioned-owned'
    full_packet = json.loads((evidence / 'complete-original-module-graph-inputs.json').read_bytes())
    full = wire(full_packet)
    full_rows = {row['object']['id']: row for row in full_packet['snapshot_sources']}
    assert len(full_rows) == len(full['objects']) == 5697
    parts = []
    for name, expected in [('source-coord', {'original-go-sdk', 'workspace/dep', 'workspace/main'}), ('export-coord', {'original-go-package-exports'})]:
        coord = run / name
        start = json.loads((coord / 'roots-ready.json').read_bytes())
        opened = json.loads((coord / 'opened.json').read_bytes())
        matched = json.loads((coord / 'matched.json').read_bytes())
        packet = json.loads((coord / 'snapshot-ready.json').read_bytes())
        snapshot = wire(packet)
        assert start['capture_started'] is False and {row['Capability'] for row in start['roots']} == expected
        if name == 'export-coord':
            assert start['compiler_output_prepared'] is True
        assert opened['status'] == matched['status'] == 'passed' and opened['opened_before_capture'] and matched['opened_before_capture']
        assert set(snapshot['capabilities']) == expected
        assert snapshot['directories'] == [row for row in full['directories'] if row['capability'] in expected]
        assert snapshot['objects'] == [row for row in full['objects'] if row['capability'] in expected]
        assert matched['original_source_objects'] == len(snapshot['objects']) and matched['native_snapshot_sha256'] == snapshot['snapshot_sha256']
        assert all(full_rows[row['object']['id']] == row for row in packet['snapshot_sources'])
        parts.append(snapshot)
    assert len(parts[0]['objects']) == 5516 and len(parts[1]['objects']) == 181
    assert set(full['capabilities']) == set(parts[0]['capabilities']) | set(parts[1]['capabilities'])
    total = 0
    for obj in full['objects']:
        row = full_rows[obj['id']]
        assert row['object'] == obj and row['raw_file'] == obj['id'] + '.raw'
        raw = (evidence / 'original-source-objects' / row['raw_file']).read_bytes()
        assert len(raw) == obj['bytes'] and hashlib.sha256(raw).hexdigest() == obj['raw_sha256']
        identity = {key: value for key, value in obj.items() if key != 'id'}
        assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0' + prior.canonical(identity)).hexdigest() == obj['id']
        total += len(raw)
    compiler = json.loads(bytes.fromhex(full_packet['compiler_inputs_hex']))
    release = json.loads((REPO / 'release/bundles/candidates/go.json').read_bytes())['toolchain_bundles'][0]
    assert compiler['inventory'] == release['inventory'] and compiler['host'] == 'linux_amd64' and len(full_packet['compiler_files']) == 5577
    compiler_bytes = 0
    for row, expected in zip(full_packet['compiler_files'], release['inventory']['files']):
        assert row['entry'] == expected and row['raw_file'] == expected['sha256'] + '.raw'
        raw = (evidence / 'original-compiler-files' / row['raw_file']).read_bytes()
        assert len(raw) == expected['size_bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
        compiler_bytes += len(raw)
    assert compiler_bytes == 166354960
    selection = json.loads(bytes.fromhex(full_packet['selection_hex']))
    exports = json.loads(bytes.fromhex(full_packet['export_inputs_hex']))
    assert len(selection['roots']) == 1 and len(full_packet['overlays']) == 0 and len(exports['packages']) == 182
    ready = json.loads((run / 'combined-coord/owner-ready.json').read_bytes())
    owner = json.loads((run / 'combined-coord/owner-matched.json').read_bytes())
    assert ready['original_roots_retired'] and ready['export_cache_retired']
    assert owner['status'] == 'passed' and owner['both_root_sets_retained'] and owner['compiler_inputs_imported']
    assert owner['staged_installed_root_opened'] is True and owner['staged_test_executable'] is True
    assert (owner['source_objects'], owner['export_objects'], owner['total_objects']) == (5516, 181, 5697)
    assert (owner['registered_sdk_files'], owner['registered_sdk_bytes']) == (5577, 166354960)
    assert not owner['launcher_authority_claimed'] and not owner['compiler_to_source_proved'] and not owner['application_proof_claimed']
    checked_before_binding = owner.get('export_cache_retirement_checked_before_binding') is True
    assert checked_before_binding
    if checked_before_binding:
        start = json.loads((run / 'export-coord/roots-ready.json').read_bytes())
        assert ready['export_cache_directory'] == start['roots'][0]['Directory']
    output = (run / 'go.stdout').read_text()
    assert output.index('independent observer mutated only copied export inputs') < output.index('both independently held original root sets match the complete original export owner') < output.index('--- PASS: TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages')
    storage = json.loads((run / 'evidence-storage.json').read_bytes())
    restore = json.loads((run / 'restore-check.json').read_bytes())
    assert storage['status'] == restore['status'] == 'passed' and storage['original_files'] == restore['restored_files']
    retirement = json.loads((run / 'retirement-check.json').read_bytes())
    assert retirement['status'] == 'passed' and retirement['all_paths_absent_after_run'] and not retirement['original_volume_modified']
    inputs = json.loads((run / 'inputs.json').read_bytes())
    driver = Path(inputs['driver']['path'])
    assert seal(driver)['sha256'] == inputs['driver']['sha256'] and seal(driver)['bytes'] == inputs['driver']['bytes'] and driver.stat().st_mode & 0o777 == inputs['driver']['mode']
    stage = json.loads((ROOT / 'installed-stage-retry/result.json').read_bytes())
    assert stage['status'] == 'passed' and len(stage['complete_bundle_ids']) == 4
    binary = stage['installed_executable']
    actual_binary = restored_binary / 'mpk-test-observer'
    assert seal(actual_binary)['sha256'] == binary['sha256'] and seal(actual_binary)['bytes'] == binary['bytes'] and actual_binary.stat().st_mode & 0o777 == binary['mode']
    audit = {'status': 'passed', 'selection_reason': 'Independently recompute every source object and SDK file and verify both capture-time root matches and the staged installed-root compiler binding.', 'inspector': seal(Path(__file__).resolve()), 'live_driver': seal(driver), 'installed_stage': seal(ROOT / 'installed-stage-retry/result.json'), 'live_result': seal(run / 'result.json'), 'source_snapshot': seal(run / 'source-coord/snapshot-ready.json'), 'export_snapshot': seal(run / 'export-coord/snapshot-ready.json'), 'complete_owner': seal(evidence / 'complete-original-module-graph-inputs.json'), 'evidence_storage': seal(run / 'evidence-storage.json'), 'restoration': seal(run / 'restore-check.json'), 'post_run_retirement': seal(run / 'retirement-check.json'), 'source_objects': 5516, 'export_objects': 181, 'complete_original_objects': 5697, 'original_source_bytes': total, 'original_packages': 182, 'registered_sdk_files': 5577, 'registered_sdk_bytes': compiler_bytes, 'both_matched_root_handles_retained_through_binding': True, 'staged_installed_root_opened': True, 'staged_test_executable': True, 'root_matches_are_capture_time_observations': True, 'cache_retirement_checked_before_binding': checked_before_binding, 'original_paths_absent_after_run': True, 'launcher_authority_proved': False, 'application_proof_claimed': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = run / 'independent-inspection.json'
    assert not out.exists()
    out.write_text(json.dumps(audit, indent=2) + '\n')
    print(json.dumps({key: audit[key] for key in ['status', 'source_objects', 'export_objects', 'complete_original_objects', 'registered_sdk_files']}), flush=True)


if __name__ == '__main__':
    main()
