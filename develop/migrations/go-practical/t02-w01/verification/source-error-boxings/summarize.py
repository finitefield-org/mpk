"""Audit terminal original streams, scoped checks and retained failed epochs."""
from pathlib import Path
import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def same(row):
    assert seal(row['path']) == {k: row[k] for k in ['path', 'bytes', 'sha256']}, row['path']


streams = {}
pair_runs = []
go_preimages = 0
for epoch in ['current-paired', 'current-large-paired', 'current-controls-paired', 'current-large-controls-paired']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_inputs_unchanged']
    for key in ['actual_driver_prelaunch', 'complete_prelaunch_inputs', 'go_binary']:
        same(run[key])
    for row in run['go_production_sources']:
        same(row)
    inputs = json.loads(Path(run['complete_prelaunch_inputs']['path']).read_bytes())
    go_preimages += len(inputs)
    for row in inputs:
        same(row)
        same(row['complete_prelaunch_snapshot'])
        original, copy = Path(row['path']), Path(row['complete_prelaunch_snapshot']['path'])
        assert original.read_bytes() == copy.read_bytes()
        assert original.stat().st_mode & 0o777 == copy.stat().st_mode & 0o777 == row['mode']
    for row in run['cases']:
        for key in ['certificate', 'rust_report', 'stdout', 'stderr']:
            same(row[key])
        digest = row['certificate']['sha256']
        assert digest not in streams
        rust, go = row['rust_observation'], row['go_observation']
        assert rust['verdict'] == go['verdict']
        if rust['verdict'] == 'accepted':
            assert row['exit_code'] == 0 and rust['axiom_count'] == 0 and not rust['axiom_report']['entries']
            assert rust['hashes'] == go['hashes'] and rust['declaration_count'] == go['declaration_count']
        else:
            assert row['exit_code'] != 0 and rust['error_code'] == 'KERNEL_CORE_CHECK' and go['error_kind'] == 'core_check'
            detail = json.loads(rust['error_detail'])
            assert detail['code'] == 'CORE_TYPE_MISMATCH' and detail['details']['kind'] == 'check_type_mismatch'
        streams[digest] = row
    pair_runs.append(seal(path))
assert len(streams) == 14
assert sum(r['rust_observation']['verdict'] == 'accepted' for r in streams.values()) == 6
assert sum(r['rust_observation']['verdict'] == 'rejected' for r in streams.values()) == 8

checks = {}
for epoch in ['current', 'current-large', 'current-controls', 'current-large-controls', 'current-replay', 'current-lint', 'current-format', 'current-oracle-fixed', 'current-large-oracle', 'current-owned-oracle']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_source_inputs_unchanged']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    checks[epoch] = seal(path)

source_map = ROOT/'current-replay/source-literal-map.json'
generic_map = ROOT/'current-replay/generic-literal-map.json'
sources, generic = json.loads(source_map.read_bytes()), json.loads(generic_map.read_bytes())
assert len(sources) == 5 and all(r['complete_current_original_bytes_literal_equal'] for r in sources)
assert len(generic) == 71 and all(r['complete_original_and_current_bytes_literal_equal'] for r in generic)
large = ROOT/'current-large/certificates/source-boxing-N70.mpcert'
assert large.read_bytes() == (ROOT/'box-recipe-probe/certificates/complete-unverified-N70.mpcert').read_bytes()

oracles = [ROOT/name/'structural-comparison.json' for name in ['current-oracle-fixed', 'current-large-oracle']]
observed = []
for path in oracles:
    data = json.loads(path.read_bytes())
    assert data['status'] == 'frozen_boxing_structure_passed'
    observed.extend(data['streams'])
assert len(observed) == 5 and sum(len(r['cases']) for r in observed) == 78
assert sum(r['actual_tag_bit_observations'] for r in observed) == 4992
for row in observed:
    raw = Path(row['certificate']).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['raw_sha256']
    assert row['full_source_U_preserved']
    assert all(c['all64_tag_bits_and_whole_active_inactive_payload_structure_equal'] for c in row['cases'])
owned = ROOT/'current-owned-oracle/structural-comparison.json'
ownership = json.loads(owned.read_bytes())
assert ownership['status'] == 'passed' and ownership['coherent_tag_collision_is_valid_math'] and ownership['exact_source_tag_body_mismatch_detected']

epochs, failures = [], []
for path in sorted(ROOT.glob('*/inputs.json')):
    inputs = json.loads(path.read_bytes())
    run = json.loads(path.with_name('result.json').read_bytes())
    assert run['status'] in {'passed', 'failed'}
    assert run['whole_source_inputs_unchanged'] or run['formatter_mutation_expected']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    record = {'result': seal(path.with_name('result.json')), 'inputs': len(inputs['sources']), 'status': run['status'], 'formatter_mutation_expected': run['formatter_mutation_expected']}
    epochs.append(record)
    if run['status'] == 'failed':
        failures.append({**record, 'counted_as_semantic_negative': False})

production = ['crates/mpk-vc/src/go_practical_ordinary.rs', 'crates/mpk-vc/src/go_practical_ordinary/composition.rs', 'crates/mpk-vc/src/go_practical_ordinary/error_layout.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_layout.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_boxing.rs']
for epoch in ['current', 'current-large', 'current-controls', 'current-large-controls', 'current-replay', 'current-lint']:
    rows = json.loads((ROOT/epoch/'inputs.json').read_bytes())['sources']
    for path in production:
        same(next(r for r in rows if r['path'] == path))
frozen_path = ROOT.parent/'product-relations/frozen-design-preservation.json'
frozen = json.loads(frozen_path.read_bytes())
assert len(frozen['inputs']) == 255
for row in frozen['inputs']:
    same(row)
metadata = json.loads(large.with_suffix('.sources.json').read_bytes())
report = json.loads(large.with_suffix('.rust.json').read_bytes())
assert len(metadata['case_tags']) == 70 and len(metadata['types']) == 73
assert metadata['declarations'] == report['declaration_count'] <= 4096
assert metadata['terms'] <= 131072 and metadata['bytes'] == large.stat().st_size <= 2097152

verification = {
    'schema': 'mpk.go_practical.source_error_boxing_checkpoint.v1',
    'status': 'scoped_checks_passed',
    'basis_commit': '310acba813149c17ccba76ba283b351efc1f42f6',
    'language_scope': ['Go', 'Rust'],
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'distinct_full_streams_checked_in_both_cores': len(streams),
    'zero_axiom_mathematical_acceptances': 6,
    'genuine_core_type_mismatches': 8,
    'paired_executions': pair_runs,
    'Go_complete_prelaunch_inputs': go_preimages,
    'normal_original_streams': 5,
    'independent_complete_source_box_constructors': 78,
    'actual_tag_bit_observations': 4992,
    'structural_comparisons': [seal(p) for p in oracles],
    'coherent_tag_collision_is_math_but_fails_exact_source_import_and_oracle': seal(owned),
    'current_source_literal_replay': seal(source_map),
    'old_generic71_literal_replay': seal(generic_map),
    'source_epochs': len(epochs),
    'original_input_rows': sum(e['inputs'] for e in epochs),
    'actual_source_executions': epochs,
    'retained_failed_epochs': failures,
    'pairing_driver_epoch_selection_failure': {
        'record': seal(ROOT/'paired-driver-selection-failure/record.json'),
        'complete_after_terminal_driver': seal(ROOT/'paired-driver-selection-failure/after-terminal-driver.py'),
        'prelaunch_archive_claimed': False,
        'native_Go_execution_started': False,
        'corrected_full_prelaunch_pairing_passed': True,
    },
    'current_checks': checks,
    'production_sources': [seal(p) for p in production],
    'N70': {'cases': 70, 'source_type_recipes': 73, 'declarations': metadata['declarations'], 'terms': metadata['terms'], 'bytes': metadata['bytes'], 'imports': 0, 'axioms': 0},
    'frozen_design_inputs': {'manifest': seal(frozen_path), 'count': 255},
    'selection_reason': 'A new consumer of complete source U and source zeros requires original small and70-case full streams, active/inactive payload and nil/tag controls, source ownership and all64 tag-bit observations. The compiled dependency closure retains all core prerequisites and all source type/zero identities. Unused generic adapter laws remain literally unchanged in their separate full0..70 library; complete5/source and71/generic replay checks the refactor without repeating expensive unchanged checker matrices. Go frontend/core sources are unchanged. Scoped lint/format and changed-input review complete this component; unrelated suites and the intermediate whole gate are not repeated.',
    'packaging': 'All core/error-value declarations and all checked source Carrier/Zero/ZeroRecipe/NilFromSourceZeros declarations are roots. The generic Nil and exact closed dependencies are copied with unchanged named types/bodies/proofs/family metadata. Complete payload equality uses checked leaf congruence once and a checked whole BoxRecipe per source case; no function extensionality or smaller U is assumed.',
    'all_actual_validation_handles_terminal': True,
    'limits_unchanged': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152, 'conversion_fuel': 1024},
    'no_new_case_root_helper_parameter_cap': True,
    'Go_frontend_changed': False,
    'kernel_or_axiom_changes': False,
    'complete_R01_R06_W01': False,
    'public_profile_activated': False,
    'full_T02_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed': False,
    'remaining': 'Source-Go boxing/zero-evaluation and arbitrary-input/inactive-case validity; canonical capture/module-role/U/tag/boxing bindings; entry/receiver/dispatch/precondition/result/store/frame/lifetime/loop/runtime/program proofs; full56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings remain for the final strict gate. Private source tags and recipes confer no canonical context or program authority.'
}
(ROOT/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps({k: verification[k] for k in ['status', 'distinct_full_streams_checked_in_both_cores', 'source_epochs', 'original_input_rows', 'Go_complete_prelaunch_inputs', 'N70']}))
