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
assert len(streams) == 87
assert sum(r['rust_observation']['verdict'] == 'accepted' for r in streams.values()) == 80
assert sum(r['rust_observation']['verdict'] == 'rejected' for r in streams.values()) == 7

checks = {}
for epoch in ['first', 'first-large', 'controls', 'large-controls', 'final-lint', 'final-format', 'oracle-small', 'oracle-large', 'oracle-owned']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_source_inputs_unchanged']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    checks[epoch] = seal(path)

literal_replays=[]
for epoch in ['first','first-large']:
    for path in sorted((ROOT/epoch/'certificates').glob('*.catalogue.json')):
        metadata=json.loads(path.read_bytes());assert metadata['original_complete_operations_bytes_literal_equal']
        original=Path(metadata['original_complete_operations_stream'])
        current=path.with_name(metadata['source_unit']+'.prior-operations.original-bytes')
        assert current.read_bytes()==original.read_bytes()
        assert len(current.read_bytes())==metadata['prior_operations_bytes']
        assert hashlib.sha256(current.read_bytes()).hexdigest()==metadata['prior_operations_raw_sha256']
        literal_replays.append({'catalogue_metadata':seal(path),'complete_current_operations_bytes':seal(current),'complete_original_operations_bytes':seal(original),'literal_bytes_equal':True})
assert len(literal_replays)==5
large_rows=sorted((ROOT/'first-large/certificates').glob('*.sources.json'))
assert len(large_rows)==70

oracles = [ROOT/name/'structural-comparison.json' for name in ['oracle-small', 'oracle-large']]
observed = []
for path in oracles:
    data = json.loads(path.read_bytes())
    assert data['status'] == 'frozen_error_case_structure_passed'
    observed.extend(data['streams'])
assert len(observed) == 78 and sum(len(r['cases']) for r in observed) == 4926
assert sum(r['actual_tag_bit_observations'] for r in observed) == 315264
assert sum(r['actual_nil_tag_bit_observations'] for r in observed) == 4992
assert sum(r['complete_operation_definitions_compared'] for r in observed) == 390
assert sum(r['complete_operation_theorem_types_compared'] for r in observed) == 10944
assert sum(r['all_copy_payload_fields_preserved'] for r in observed) == 4926
assert sum(r['complete_case_row_theorem_types_compared'] for r in observed)==9930
assert sum(len(r['case_comparisons']) for r in observed)==4926
for row in observed:
    raw = Path(row['certificate']).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['raw_sha256']
    assert row['full_source_U_preserved']
    assert all(c['all64_tag_bits_and_whole_active_inactive_payload_structure_equal'] for c in row['cases'])
owned = ROOT/'oracle-owned/structural-comparison.json'
ownership = json.loads(owned.read_bytes())
assert ownership['status'] == 'passed' and ownership['prior_duplicate_tag_math_is_accepted_but_frozen_tag_body_mismatch'] and ownership['prior_constant_false_math_is_accepted_but_frozen_comparison_body_mismatch'] and ownership['different_accepted_case_row_owner_mismatch_detected']

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

production = ['crates/mpk-vc/src/go_practical_ordinary.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_operations.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_cases.rs']
for epoch in ['first', 'first-large', 'controls', 'large-controls', 'final-lint']:
    rows = json.loads((ROOT/epoch/'inputs.json').read_bytes())['sources']
    for path in production:
        same(next(r for r in rows if r['path'] == path))
frozen_path = ROOT.parent/'product-relations/frozen-design-preservation.json'
frozen = json.loads(frozen_path.read_bytes())
assert len(frozen['inputs']) == 255
for row in frozen['inputs']:
    same(row)
large_stats=[]
for path in large_rows:
    metadata=json.loads(path.read_bytes());cert=path.with_name(path.name.removesuffix('.sources.json')+'.mpcert');report=json.loads(cert.with_suffix('.rust.json').read_bytes())
    assert len(metadata['case_tags'])==70 and len(metadata['types'])==73
    assert metadata['declarations']==report['declaration_count']<=4096
    assert metadata['terms']<=131072 and metadata['bytes']==cert.stat().st_size<=2097152
    large_stats.append({k:metadata[k] for k in ['boxed_source_type_id','declarations','terms','bytes']})

verification = {
    'schema': 'mpk.go_practical.source_error_cases_checkpoint.v1',
    'status': 'scoped_checks_passed',
    'basis_commit': '6df96aa3f074af997eeb234355def1394a4f82c4',
    'language_scope': ['Go', 'Rust'],
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'distinct_full_streams_checked_in_both_cores': len(streams),
    'zero_axiom_mathematical_acceptances': 80,
    'genuine_core_type_mismatches': 7,
    'paired_executions': pair_runs,
    'Go_complete_prelaunch_inputs': go_preimages,
    'normal_original_streams': 78,
    'independent_complete_source_box_constructors': 4926,
    'actual_tag_bit_observations': 315264,
    'structural_comparisons': [seal(p) for p in oracles],
    'prior_math_alternatives_and_wrong_case_row_owner_fail_frozen_comparison': seal(owned),
    'prior_operations5_literal_replays': literal_replays,
    'prior_value_operations_evidence_unchanged': seal(ROOT.parent/'source-error-operations/verification.json'),
    'actual_nil_tag_bit_observations': sum(r['actual_nil_tag_bit_observations'] for r in observed),
    'complete_operation_definitions_compared': sum(r['complete_operation_definitions_compared'] for r in observed),
    'complete_operation_theorem_statements_compared': sum(r['complete_operation_theorem_types_compared'] for r in observed),
    'every_source_copy_payload_field_compared': sum(r['all_copy_payload_fields_preserved'] for r in observed),
    'complete_source_case_row_theorem_statements_compared':sum(r['complete_case_row_theorem_types_compared'] for r in observed),
    'case_comparison_pairs':sum(len(r['case_comparisons']) for r in observed),
    'source_epochs': len(epochs),
    'original_input_rows': sum(e['inputs'] for e in epochs),
    'actual_source_executions': epochs,
    'retained_failed_epochs': failures,
    'current_checks': checks,
    'production_sources': [seal(p) for p in production],
    'N70': {'cases':70,'source_type_recipes':73,'rows':70,'comparison_pairs':4900,'row_sizes':large_stats,'maximum_declarations':max(r['declarations'] for r in large_stats),'maximum_terms':max(r['terms'] for r in large_stats),'maximum_bytes':max(r['bytes'] for r in large_stats),'imports':0,'axioms':0},
    'frozen_design_inputs': {'manifest': seal(frozen_path), 'count': 255},
    'selection_reason':'Complete finite case selection requires every source constructor to match exactly its own target among the full same U. Check all78 original rows/4926 pairs including all70 large rows/4900 pairs in both full cores; compare every retained source constructor/operation statement/tag bit and all new row statements independently. Six small and one large false-claim/collision/constant-false-row controls must genuinely type reject. Two earlier coherent prior-math alternatives remain mathematical acceptance and fail exact source import/frozen bodies. Different accepted row substitution must fail exact source-owner import and independent scope comparison. Replay all5 old full operations streams literally after builder refactor. Scoped lint/format/review only; no unchanged unrelated suites or intermediate whole gate.',
    'packaging':'An immutable private catalogue closes every original source-specific definition AND theorem once, retaining all complete U carriers, zeros, payloads, constructors and their nil/copy/observation laws. Unused general math stays unchanged in its full old library. Each row clones this exact closed basis and adds comparisons against every source target, with actual checked bit rewrites and HasCase transfer; there is no source-universe/root-list selector, smaller case cap, axiom or caller-supplied comparison oracle.',
    'all_actual_validation_handles_terminal': True,
    'limits_unchanged': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152, 'conversion_fuel': 1024},
    'no_new_case_root_helper_parameter_cap': True,
    'Go_frontend_changed': False,
    'kernel_or_axiom_changes': False,
    'complete_R01_R06_W01': False,
    'public_profile_activated': False,
    'full_T02_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed': False,
    'remaining': 'Source-Go finite dispatch/receiver-once/callee/effect execution, nil/copy/boxing/comparison execution and zero-evaluation and arbitrary-input/inactive-case validity; canonical capture/module-role/U/tag/boxing bindings; entry/receiver/dispatch/precondition/result/store/frame/lifetime/loop/runtime/program proofs; full56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings remain for the final strict gate. Private source tags and recipes confer no canonical context or program authority.'
}
(ROOT/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps({k: verification[k] for k in ['status', 'distinct_full_streams_checked_in_both_cores', 'source_epochs', 'original_input_rows', 'Go_complete_prelaunch_inputs', 'N70']}))
