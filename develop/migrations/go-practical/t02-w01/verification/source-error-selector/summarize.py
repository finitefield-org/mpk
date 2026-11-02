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
assert len(streams) == 99
assert sum(r['rust_observation']['verdict'] == 'accepted' for r in streams.values()) == 93
assert sum(r['rust_observation']['verdict'] == 'rejected' for r in streams.values()) == 6

checks = {}
for epoch in ['entries', 'first-large', 'controls-corrected', 'large-controls', 'lint-corrected', 'format-corrected', 'oracle-small', 'oracle-large', 'oracle-owned', 'replay']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_source_inputs_unchanged']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    checks[epoch] = seal(path)

replay=ROOT/'replay/certificates/case-row-literal-map.json'
literal_replays=json.loads(replay.read_bytes())
assert len(literal_replays)==78 and all(r['complete_current_original_bytes_literal_equal'] for r in literal_replays)
for row in literal_replays:
    original=Path(row['original']);assert original.stat().st_size==row['bytes']
large_rows=sorted((ROOT/'first-large/certificates').glob('*.sources.json'))
assert len(large_rows)==71

oracles = [ROOT/name/'structural-comparison.json' for name in ['oracle-small', 'oracle-large']]
observed = []
for path in oracles:
    data = json.loads(path.read_bytes())
    assert data['status'] == 'frozen_error_selector_structure_passed'
    observed.extend(data['streams'])
assert len(observed) == 92 and sum(len(r['cases']) for r in observed) == 5030
assert sum(r['actual_tag_bit_observations'] for r in observed) == 321920
assert sum(r['actual_nil_tag_bit_observations'] for r in observed) == 5888
assert sum(r['complete_operation_definitions_compared'] for r in observed) == 460
assert sum(r['complete_operation_theorem_types_compared'] for r in observed) == 11348
assert sum(r['all_copy_payload_fields_preserved'] for r in observed) == 5030
assert sum(r['complete_case_row_theorem_types_compared'] for r in observed)==9975
assert sum(len(r['case_comparisons']) for r in observed)==4945
assert sum(r['selector_definitions_compared'] for r in observed)==10336
assert sum(r['selector_statements_compared'] for r in observed)==15274
assert len([r for r in observed if r['boxed_source_type_id'] is None])==7
for row in observed:
    raw = Path(row['certificate']).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['raw_sha256']
    assert row['full_source_U_preserved']
    assert all(c['all64_tag_bits_and_whole_active_inactive_payload_structure_equal'] for c in row['cases'])
owned = ROOT/'oracle-owned/structural-comparison.json'
ownership = json.loads(owned.read_bytes())
assert ownership['status'] == 'passed' and ownership['coherent_wrong_ordinal_is_valid_math_but_fails_exact_target_definition'] and ownership['different_accepted_original_call_site_owner_mismatch_detected'] and ownership['same_complete_U_and_method_targets_preserved_for_wrong_site_control']

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

production = ['crates/mpk-vc/src/go_practical_ordinary.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_cases.rs', 'crates/mpk-vc/src/go_practical_ordinary/source_error_dispatch.rs']
for epoch in ['entries', 'first-large', 'controls-corrected', 'large-controls', 'lint-corrected', 'replay']:
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
    'schema': 'mpk.go_practical.source_error_selector_checkpoint.v1',
    'status': 'scoped_checks_passed',
    'basis_commit': '1a0058ffe62998822209223f109276b5b6ef2f39',
    'language_scope': ['Go', 'Rust'],
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'distinct_full_streams_checked_in_both_cores': len(streams),
    'zero_axiom_mathematical_acceptances': 93,
    'genuine_core_type_mismatches': 6,
    'paired_executions': pair_runs,
    'Go_complete_prelaunch_inputs': go_preimages,
    'normal_original_streams': 92,
    'independent_complete_source_box_constructors': 5030,
    'actual_tag_bit_observations': 321920,
    'structural_comparisons': [seal(p) for p in oracles],
    'wrong_target_ordinal_math_and_wrong_call_owner_fail_frozen_comparison': seal(owned),
    'prior_case78_literal_replay': {'map':seal(replay),'rows':78},
    'prior_complete_case_rows_evidence_unchanged': seal(ROOT.parent/'source-error-cases/verification.json'),
    'actual_nil_tag_bit_observations': sum(r['actual_nil_tag_bit_observations'] for r in observed),
    'complete_operation_definitions_compared': sum(r['complete_operation_definitions_compared'] for r in observed),
    'complete_operation_theorem_statements_compared': sum(r['complete_operation_theorem_types_compared'] for r in observed),
    'every_source_copy_payload_field_compared': sum(r['all_copy_payload_fields_preserved'] for r in observed),
    'complete_source_case_row_theorem_statements_compared':sum(r['complete_case_row_theorem_types_compared'] for r in observed),
    'case_comparison_pairs':sum(len(r['case_comparisons']) for r in observed),
    'selector_definitions_compared':sum(r['selector_definitions_compared'] for r in observed),
    'selector_theorem_statements_compared':sum(r['selector_statements_compared'] for r in observed),
    'actual_decoded_selector_target_observations':len(observed),
    'nil_without_selected_target_rows':sum(r['boxed_source_type_id'] is None for r in observed),
    'source_epochs': len(epochs),
    'original_input_rows': sum(e['inputs'] for e in epochs),
    'actual_source_executions': epochs,
    'retained_failed_epochs': failures,
    'current_checks': checks,
    'production_sources': [seal(p) for p in production],
    'N70': {'cases':70,'source_type_recipes':73,'rows':71,'concrete_target_rows':70,'nil_no_target_rows':1,'row_sizes':large_stats,'maximum_declarations':max(r['declarations'] for r in large_stats),'maximum_terms':max(r['terms'] for r in large_stats),'maximum_bytes':max(r['bytes'] for r in large_stats),'imports':0,'axioms':0},
    'frozen_design_inputs': {'manifest': seal(frozen_path), 'count': 255},
    'selection_reason':'Exact finite target selection must preserve the original checked full U/method table and each original receiver/call site. Verify all92 rows across source entry, nil-only, method-expression, receiver-call and multiple-root sites plus all70 large concrete types and nil. Both complete cores check the actual original full streams; independent frozen definitions/statements/hash preimages and actual decoded selector observations compare to exact target ordinals and prior original Go call/receiver spans/target tables. Five small and one large false-target/branch/nil claims must type reject. A coherent changed named ordinal is mathematical acceptance but exact-import and frozen-body mismatch; different accepted call-site substitution must reject despite same U/targets. Replay all78 prior case rows literally without rerunning their unchanged expensive proof matrix. Scoped lint/format/review only; no intermediate full gate or unrelated suites.',
    'packaging':'Every original complete source case row and all source-U type/zero/payload/nil/copy/comparison identities remain in concrete selector rows. Nil rows use the same complete source-specific basis. Each ordinary suffix definition scans every target, and checked generic true/false branch transfer plus exact old Matches/NilHasNoCase witnesses prove every suffix and final selected target. Private preparation names bind all original call/receiver/method signatures; NoTarget Nat0 is not a nil safety or callee correctness proof. No target or case admission cap, caller comparator/root list, new axiom/reduction/compiler behavior or canonical context identity is introduced.',
    'all_actual_validation_handles_terminal': True,
    'limits_unchanged': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152, 'conversion_fuel': 1024},
    'no_new_case_root_helper_parameter_cap': True,
    'Go_frontend_changed': False,
    'kernel_or_axiom_changes': False,
    'complete_R01_R06_W01': False,
    'public_profile_activated': False,
    'full_T02_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed': False,
    'remaining': 'Original-source validity and full receiver-once/nil-safety/callee/precondition/joint-result/effect/frame execution, nil/copy/boxing/comparison execution and zero-evaluation and arbitrary-input/inactive-case validity; canonical capture/module-role/U/tag/boxing bindings; entry/receiver/dispatch/precondition/result/store/frame/lifetime/loop/runtime/program proofs; full56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings remain for the final strict gate. Private source tags and recipes confer no canonical context or program authority.'
}
(ROOT/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps({k:verification[k] for k in ['status','distinct_full_streams_checked_in_both_cores','source_epochs','original_input_rows','Go_complete_prelaunch_inputs']}))
