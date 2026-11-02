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
for epoch in ['paired-small', 'paired-large', 'controls-paired', 'large-controls-paired']:
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
assert len(streams) == 16
assert sum(r['rust_observation']['verdict'] == 'accepted' for r in streams.values()) == 6
assert sum(r['rust_observation']['verdict'] == 'rejected' for r in streams.values()) == 10

checks = {}
for epoch in ['explicit-bit', 'first-large', 'controls', 'large-controls', 'final-lint', 'final-format', 'oracle-small-complete', 'oracle-large', 'oracle-owned']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_source_inputs_unchanged']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    checks[epoch] = seal(path)

large = ROOT/'first-large/certificates/source-operations-N70.mpcert'
literal_replays=[]
for epoch in ['explicit-bit','first-large']:
    for path in sorted((ROOT/epoch/'certificates').glob('*.sources.json')):
        metadata=json.loads(path.read_bytes())
        assert metadata['prior_boxing_complete_literal_bytes_equal']
        original_name='source-boxing-N70' if path.stem=='source-operations-N70.sources' else path.name.removesuffix('.sources.json')
        original_epoch='current-large' if len(metadata['case_tags'])==70 else 'current'
        original=ROOT.parent/'source-error-boxings'/original_epoch/'certificates'/f'{original_name}.mpcert'
        literal_replays.append({'current_metadata':seal(path),'original_complete_boxing_bytes':seal(original),'literal_comparison_asserted_in_original_saved_Rust_targeted_test':True})
assert len(literal_replays)==5

oracles = [ROOT/name/'structural-comparison.json' for name in ['oracle-small-complete', 'oracle-large']]
observed = []
for path in oracles:
    data = json.loads(path.read_bytes())
    assert data['status'] == 'frozen_error_operation_structure_passed'
    observed.extend(data['streams'])
assert len(observed) == 5 and sum(len(r['cases']) for r in observed) == 78
assert sum(r['actual_tag_bit_observations'] for r in observed) == 4992
assert sum(r['actual_nil_tag_bit_observations'] for r in observed) == 320
assert sum(r['complete_operation_definitions_compared'] for r in observed) == 25
assert sum(r['complete_operation_theorem_types_compared'] for r in observed) == 226
assert sum(r['all_copy_payload_fields_preserved'] for r in observed) == 78
for row in observed:
    raw = Path(row['certificate']).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['raw_sha256']
    assert row['full_source_U_preserved']
    assert all(c['all64_tag_bits_and_whole_active_inactive_payload_structure_equal'] for c in row['cases'])
owned = ROOT/'oracle-owned/structural-comparison.json'
ownership = json.loads(owned.read_bytes())
assert ownership['status'] == 'passed' and ownership['constant_false_case_test_is_valid_math'] and ownership['exact_frozen_case_comparison_body_mismatch_detected']

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

production = ['crates/mpk-vc/src/go_practical_ordinary.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_boxing.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_operations.rs']
for epoch in ['explicit-bit', 'first-large', 'controls', 'large-controls', 'final-lint']:
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
    'schema': 'mpk.go_practical.source_error_operations_checkpoint.v1',
    'status': 'scoped_checks_passed',
    'basis_commit': 'd558832ef7a290a97f04ef6cd88e26afb46b12c8',
    'language_scope': ['Go', 'Rust'],
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'distinct_full_streams_checked_in_both_cores': len(streams),
    'zero_axiom_mathematical_acceptances': 6,
    'genuine_core_type_mismatches': 10,
    'paired_executions': pair_runs,
    'Go_complete_prelaunch_inputs': go_preimages,
    'normal_original_streams': 5,
    'independent_complete_source_box_constructors': 78,
    'actual_tag_bit_observations': 4992,
    'structural_comparisons': [seal(p) for p in oracles],
    'constant_false_case_test_is_math_but_fails_exact_source_import_and_oracle': seal(owned),
    'prior_boxing5_literal_replays': literal_replays,
    'prior_source5_generic71_evidence_unchanged': seal(ROOT.parent/'source-error-boxings/verification.json'),
    'actual_nil_tag_bit_observations': sum(r['actual_nil_tag_bit_observations'] for r in observed),
    'complete_operation_definitions_compared': sum(r['complete_operation_definitions_compared'] for r in observed),
    'complete_operation_theorem_statements_compared': sum(r['complete_operation_theorem_types_compared'] for r in observed),
    'every_source_copy_payload_field_compared': sum(r['all_copy_payload_fields_preserved'] for r in observed),
    'source_epochs': len(epochs),
    'original_input_rows': sum(e['inputs'] for e in epochs),
    'actual_source_executions': epochs,
    'retained_failed_epochs': failures,
    'current_checks': checks,
    'production_sources': [seal(p) for p in production],
    'N70': {'cases': 70, 'source_type_recipes': 73, 'declarations': metadata['declarations'], 'terms': metadata['terms'], 'bytes': metadata['bytes'], 'imports': 0, 'axioms': 0},
    'frozen_design_inputs': {'manifest': seal(frozen_path), 'count': 255},
    'selection_reason': 'New source-U nil/copy/comparison equations require full original small/70-case streams, every copied field, nil tag/concrete/inactive-payload controls and source-owned comparison definitions. Both actual full cores check the complete libraries. The independent frozen oracle compares all operation definitions/statements and every case/nil tag bit. A coherently healed constant-false case test is mathematical acceptance but exact-import and structural mismatch. Five complete old boxing producers replay literally in the original saved target tests after the builder refactor. Existing source/generic/core/Go frontend code is unchanged, so earlier expensive unrelated suites and full intermediate gate are not repeated.',
    'packaging': 'The default boxing builder is unchanged. The nil builder additionally retains the compiled FrozenZeroIsZero root with exact closed named dependencies. Nil keeps the frozen constant-false64-bit function, separately from the Word.Zero decision tree; no function extensionality is assumed. Generic checked tag/comparison transfer laws avoid concrete fold normalization, and each source case supplies explicit checked bit rewrites. Every full old boxing named type/body/proof/family identity remains unchanged in the new complete stream.',
    'all_actual_validation_handles_terminal': True,
    'limits_unchanged': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152, 'conversion_fuel': 1024},
    'no_new_case_root_helper_parameter_cap': True,
    'Go_frontend_changed': False,
    'kernel_or_axiom_changes': False,
    'complete_R01_R06_W01': False,
    'public_profile_activated': False,
    'full_T02_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed': False,
    'remaining': 'Source-Go nil/copy/boxing/comparison execution and zero-evaluation and arbitrary-input/inactive-case validity; canonical capture/module-role/U/tag/boxing bindings; entry/receiver/dispatch/precondition/result/store/frame/lifetime/loop/runtime/program proofs; full56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings remain for the final strict gate. Private source tags and recipes confer no canonical context or program authority.'
}
(ROOT/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps({k: verification[k] for k in ['status', 'distinct_full_streams_checked_in_both_cores', 'source_epochs', 'original_input_rows', 'Go_complete_prelaunch_inputs', 'N70']}))
