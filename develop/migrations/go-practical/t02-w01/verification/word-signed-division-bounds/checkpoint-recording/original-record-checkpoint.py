"""Bind actual signed division bounds to complete, scoped local evidence."""
from pathlib import Path
import collections
import datetime
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())


def read(path):
    return json.loads(Path(path).read_text())


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def current_sources(path, production_only=False):
    record = read(path)
    assert record['status'] == 'passed' and record['exit_code'] == 0
    assert record['changed_inputs_during_execution'] == []
    for source in record['sources']:
        snapshot = seal(path.parent / 'inputs' / source['path'])
        assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
        if not production_only or source['path'].startswith('crates/mpk-vc/src/'):
            assert seal(source['path']) == source
    return record


execution_path = root / 'final/rust-tests/result.json'
execution = current_sources(execution_path)
stdout = (execution_path.parent / 'stdout.log').read_text()
assert '4 passed; 0 failed' in stdout and 'matched complete prior named refinement=3897' in stdout
certificate = execution_path.parent / 'certificates/word-division-bounds.mpcert'
report = read(certificate.with_suffix('.rust.json'))
assert report['verdict'] == 'accepted' and report['declaration_count'] == 4093 and report['axiom_count'] == 0
assert report['hashes']['certificate'] == 'f847c610dd931b6cb2a6b7b99a56da0419938d819b04e1bf94577e40aaf7f047'
assert report['hashes']['export'] == '6ca64bd168c29a27eb19a5e17039ae1d6d2bb2634d617423acaf909dddeefab7'
assert seal(certificate)['sha256'] == 'a0158c8449e9f44c36279dd735166fe84f5b30f1d30a8caa337d32e256927e56'
assert certificate.stat().st_size == 1465556
assert certificate.read_bytes() == (root / 'actual-minimum-classification/rust-construction/certificates/word-division-bounds.mpcert').read_bytes()
assert certificate.read_bytes() == (execution_path.parent / 'model/certificate.mpcert').read_bytes()
construction_path = root / 'actual-minimum-classification/rust-construction/result.json'
current_sources(construction_path, production_only=True)
candidate_path = root / 'actual-minimum-classification/go-candidate/result.json'
candidate = read(candidate_path)
assert candidate['exit_code'] == 0
assert candidate['Rust_observation']['hashes'] == candidate['Go_observation']['hashes'] == report['hashes']
assert candidate['Rust_observation']['verdict'] == candidate['Go_observation']['verdict'] == 'accepted'

paired_path = root / 'paired-final/result.json'
paired = read(paired_path)
assert paired['status'] == 'passed' and len(paired['cases']) == 70
expected = {'FalseStrictUpper', 'FakeStrictOrder', 'FalseQuotientGap', 'FalseNegationSign',
            'FakeNegationGap', 'FalseScalarCorrection', 'FalseHalfUnit', 'FakeDivisorPositive',
            'FakeHalfPositive', 'FalseUnitQuotient', 'FakeUnitRemainderBound', 'FalseNegativeUnitReflect'}
expected |= {f'{name}-{width}' for width in [8, 16, 32, 64] for name in [
    'FakeWordGap', 'FalseQuotientHalf', 'FalseRemainderHalf', 'FalseMinimumCharacterization',
    'WrongNegativeUnitDefinition', 'WrongMinimumDefinition', 'FalseNegativeUnitSound',
    'FalseNegativeUnitComplete', 'FakeExactGuard', 'FalseMinimumSourceQuotient',
    'FakeMinimumPremise', 'FalseMinimumSourceRemainder', 'FalseClassifiedConservation', 'FakeSourceDivisor']}
observed, positives, kinds = set(), set(), collections.Counter()
for row in paired['cases']:
    rust, go = row['rust_observation'], row['go_observation']
    assert rust['verdict'] == go['verdict']
    if rust['verdict'] == 'accepted':
        assert rust['axiom_count'] == go.get('axiom_count', 0) == 0 and rust['hashes'] == go['hashes']
        positives.add(row['name'])
    else:
        detail = json.loads(rust['error_detail'])
        assert detail['code'] == 'CORE_TYPE_MISMATCH'
        kinds[detail['details']['kind']] += 1
        observed.add(row['name'])
assert observed == expected and len(observed) == 68
assert positives == {'word-division-bounds', 'honest-different-division-bounds'}

checks = {}
for label, directory in [('strict_clippy', root / 'current/clippy'), ('Rust_format', root / 'current/format')]:
    path = directory / 'result.json'
    checked = current_sources(path)
    checks[label] = {'record': seal(path), 'status': 'passed', 'seconds': checked['wall_seconds']}
go_format_path = root / 'current/go-format/result.json'
go_format = read(go_format_path)
assert go_format['status'] == 'passed' and seal(go_format['source']['path']) == go_format['source']
checks['new_Go_format'] = {'record': seal(go_format_path), 'status': 'passed'}

identities_path = root / 'retained-identities.json'
identities = read(identities_path)
assert identities['all_original_refinement_named_identities'] == 3897
assert identities['additional_original_half_named_identities'] == 8
assert identities['new_division_bounds_declarations'] == 188
assert identities['certificate']['sha256'] == seal(certificate)['sha256']
old_certificate = identities['original_refinement_certificate']
assert seal(old_certificate['path']) == old_certificate
assert old_certificate['sha256'] == 'a38c1903fb7add8bf7156b484d747612b52a985899227453a772f3a3b6498487'

comparison_path = root / 'native-comparison-final/result.json'
comparison = read(comparison_path)
metrics_path = root / 'final/rust-tests/model/metrics.json'
metrics = read(metrics_path)
assert comparison['status'] == 'passed'
for key, count in [('actual_source_operator_results', 176), ('defined_pairs', 88),
                   ('source_zero_divisor_faults', 8), ('minimum_minus_one_pairs', 8),
                   ('negative_unit_divisors', 8), ('exact_source_reconstructions', 80),
                   ('corrected_source_reconstructions', 88)]:
    assert comparison[key] == metrics[key] == count
assert comparison['original_queries'] == metrics['original_signed_queries'] == 96
assert metrics['Nat_gap_half_predicate_Int_or_theorem_bodies_executed'] is False
assert metrics['universal_classification_supplied_by_certificate_not_finite_model'] is True
native_path = root / 'native/result.json'
native = read(native_path)
assert native['status'] == 'passed' and native['native_rows'] == 96 and native['two_native_outputs_byte_identical']
assert seal(native['original_source']['path']) == native['original_source']
recovery = native['metadata_recovery']
assert recovery['original_wrapper_exit_code'] == 1
assert recovery['native_build_or_observation_repeated'] is False
assert recovery['original_timing_fields_recreated'] is False
for key in ['original_runner', 'fixed_runner', 'recovery_script', 'observed_tool_failure_text']:
    assert seal(recovery[key]['path']) == recovery[key]
assert (root / 'native/original-runner.py').read_text().replace('== 128', '== 96') == (root / 'run-native.py').read_text()
assert native['complete_original_image_checks'][0]['status'] == 'passed_by_reached_control_flow'
assert native['complete_original_image_checks'][0]['seconds'] is None
assert native['complete_original_image_checks'][1]['status'] == 'passed'
assert native['complete_original_image_checks'][1]['files'] == 5577
for command in native['commands']:
    assert command['exit_code'] == 0 and command['seconds'] is None
    for key in ['stdout', 'stderr']:
        assert seal(command[key]['path']) == command[key]

previous = read(root.parent / 'word-signed-refinement/frozen-design-preservation.json')
assert previous['status'] == 'passed' and len(previous['inputs']) == 255
for source in previous['inputs']:
    assert seal(source['path']) == source
frozen = {**previous, 'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'scope': 'All255 original frozen design/specification/vector inputs remain exact. Actual quotient/remainder half bounds and minimum/-one correction classification fit unchanged4096/131072/2097152 limits. No core/axiom/fuel/source policy widening.'}
write(root / 'frozen-design-preservation.json', frozen)
record = {
    'schema': 'mpk.go_practical.actual_signed_division_bounds_checkpoint.v1',
    'status': 'component_checks_passed_work_item_in_progress',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'entry_commit': 'a652e77b74819dbf6c931c2d29543c012b0b1aef',
    'production_sources': [s for s in execution['sources'] if s['path'].startswith('crates/mpk-vc/src/')],
    'current_target_test_model_and_inputs': [s for s in execution['sources'] if not s['path'].startswith('crates/mpk-vc/src/')],
    'fresh_component': {'certificate': seal(certificate), 'hashes': report['hashes'], 'declarations': 4093,
                        'terms': 99499, 'bytes': 1465556, 'axioms': 0, 'widths': [8, 16, 32, 64],
                        'retained_complete_prior_named_declarations': 3897, 'additional_original_half_proofs': 8,
                        'new_division_bounds_declarations': 188, 'remaining_declaration_slots': 3,
                        'frozen_candidate_bounds': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152},
                        'all_bounds_unchanged_and_met': True,
                        'exact_import': 'Private owning deterministic complete regeneration rejects altered/oversized/honestly different valid components. Complete prior refinement generator/raw/named identities stay exact. No complete source descriptor/context/registry or profile activation.'},
    'proof_scope': {
        'actual_half_bounds': 'For actual magnitude division q/r, QGap=(d-1)*q+r+MagnitudeGap(x) and q+QGap=H, hence q<=H. Actual r<d and d+MagnitudeGap(y)=H yield r<H. All premises supplied by actual source/magnitude proofs.',
        'actual_sign_correction': 'Any actual word with U+gap=H has sign=Nat.Equal(U,H) and sign(Neg)=actual DivideDefined. The prior actual selection correction is Equivalent to FromNat(Bit(And(Not(flag),Equal(U,H)))). Actual remainder correction is zero.',
        'negative_unit': 'NegativeUnit(y)=And(actual sign(y),Nat.Equal(U(actual AbsBits y),1)) is sound and complete for actual signed y Equivalent mathematical -1.',
        'minimum_classification': 'Under actual y-nonzero source guard, positive-sign q=H iff actual minimum input x and NegativeUnit(y). Actual quotient correction Equivalent FromNat(Bit(MinimumMinusOne)); remainder correction Equivalent zero.',
        'source_conservation': 'Actual guarded Sx Equivalent Sy*(S(actual SignedDiv)+M*Bit(MinimumMinusOne))+S(actual SignedRem). Exact remainder holds universally; optional QuotientNoOverflow=Not(MinimumMinusOne) yields exact quotient and source conservation. This optional exactness guard does not restrict source definedness.',
        'minimum_source_results': 'Minimum/-one remains a normal source operation. Actual SignedDiv unsigned value is H; actual SignedRem signed value Equivalent zero. Actual magnitude quotient/remainder equal H/zero.'},
    'retained_identity_observation': seal(identities_path), 'prior_complete_raw_certificate': old_certificate,
    'Rust_execution': seal(execution_path), 'passed_affected_roots': 4,
    'same_byte_paired_checkers': seal(paired_path), 'same_byte_cases': 70,
    'accepted_zero_axiom_exact_hashes': 2, 'rejected_core_type_mismatches': 68,
    'core_mismatch_subtypes': dict(kinds), 'negative_case_names': sorted(expected), 'checks': checks,
    'passing_fixed_source_construction': seal(construction_path), 'passing_same_byte_initial_Go_inspection': seal(candidate_path),
    'construction_scope': 'Initial one-root fixed-source construction checks exact import and every prior3897 named declaration. Its complete production inputs and emitted certificate bytes equal the final four-root bytes. Test-only controls/model/honest component expand after the construction terminal handle; original captured inputs remain preserved.',
    'captured_inputs_fixed_until_actual_terminal_handles': True,
    'finite_observations': {'comparison': seal(comparison_path), 'native_execution': seal(native_path),
                            'native_source': native['original_source'], 'query_origins': seal(root / 'query-origins.json'),
                            'model_metrics': seal(metrics_path), 'observations': metrics,
                            'two_native_outputs_identical': True, 'prepared_image_files': 5577,
                            'Nat_gap_half_predicate_Int_or_theorem_bodies_executed': False,
                            'universal_classification_supplied_by_certificate_not_finite_model': True},
    'native_wrapper_metadata_recovery': {'original_count_assertion_exit_code': 1, 'native_build_and_runs_completed': True,
                                         'record': seal(native_path), 'native_build_or_observation_repeated': False,
                                         'original_timings_recreated': False, 'semantic_negative_counted': False,
                                         'scope': 'Only copied unique-ID expected count128 changes to96. Preserve exact original runner, source, queries, raw native logs and observed terminal tool failure text. Original pre-build image validation/command zero status have reached-control-flow provenance; original elapsed times are null. A fresh full5577-file image revalidation runs during metadata recovery. No raw OS stderr or original elapsed/after-image check is fabricated.'},
    'frozen_design_preservation': seal(root / 'frozen-design-preservation.json'),
    'strict_changed_input_review': 'input-review/changed-inputs-review.json',
    'CSharp_tests_executed': False, 'T02_W01_complete': False, 'all_application_demands_remaining': 169,
    'public_profile_activated': False, 'whole_source_program_proof_claimed': False,
    'authoritative_immutable_application_replay_claimed': False, 'native_Linux_gate_claimed': False,
    'kernel_axiom_fuel_or_artifact_bounds_changed': False,
    'remaining_work': 'Remaining variable/signed/right-shift numeric/source laws, full source/carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All T02-T11 scope/gates remain.',
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
}
write(root / 'verification.json', record)
print(json.dumps({'status': record['status'], 'declarations': 4093, 'terms': 99499, 'bytes': 1465556,
                  'same_byte_cases': 70, 'semantic_negatives': 68, 'source_observations': 96, 'subtypes': dict(kinds)}))
