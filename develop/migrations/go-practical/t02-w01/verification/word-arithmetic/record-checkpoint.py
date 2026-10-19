"""Bind the actual scoped arithmetic observations without repeating checks."""
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

original_path = root / 'final/rust-tests/result.json'
original = read(original_path)
assert original['status'] == 'failed'
assert '3 passed; 1 failed' in (root / 'final/rust-tests/stdout.log').read_text()
focused_path = root / 'corrected/rust-controls/result.json'
focused = read(focused_path)
assert focused['status'] == 'passed' and len(focused['certificates']) == 5
control = 'crates/mpk-vc/tests/support/go_word_arithmetic_controls.rs'
for source in original['sources']:
    snapshot = seal(root / 'final/rust-tests/inputs' / source['path'])
    assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
    if source['path'] != control:
        assert seal(source['path']) == source
for source in focused['sources']:
    assert seal(source['path']) == source
paired_path = root / 'paired-final/result.json'
paired = read(paired_path)
assert paired['status'] == 'passed' and len(paired['cases']) == 26
comparison_path = root / 'native-comparison-final/result.json'
comparison = read(comparison_path)
assert comparison['status'] == 'passed' and comparison['actual_fixed_shift_and_product_stages'] == 1440
certificate = root / 'final/rust-tests/certificates/word-arithmetic-foundation.mpcert'
report = read(certificate.with_suffix('.rust.json'))
assert report['verdict'] == 'accepted' and report['declaration_count'] == 3828 and report['axiom_count'] == 0
assert len(certificate.read_bytes()) == 1358764
assert certificate.read_bytes() == (root / 'literal-values/rust-construction/certificates/word-arithmetic-foundation.mpcert').read_bytes()
assert certificate.read_bytes() == (root / 'final/rust-tests/model/certificate.mpcert').read_bytes()
old = root / 'final/rust-tests/certificates/retained-numeric-basis.mpcert'
previous_path = root.parent / 'word-numeric/verification.json'
previous = read(previous_path)
assert old.read_bytes() == Path(previous['fresh_component']['certificate']['path']).read_bytes()
counts = collections.Counter()
for case in paired['cases']:
    observation = case['rust_observation']
    if observation['verdict'] == 'rejected':
        counts[json.loads(observation['error_detail'])['details']['kind']] += 1
assert counts == {'check_type_mismatch': 22, 'lambda_domain_mismatch': 1}
check_paths = {
    'all_width_smoke': 'literal-values/rust-construction/result.json',
    'current_focused_controls': 'corrected/rust-controls/result.json',
    'current_clippy': 'corrected/clippy/result.json',
    'current_Rust_format': 'corrected/format/result.json',
    'Go_observer_format': 'final/go-format/result.json',
}
checks = {name: read(root / path) for name, path in check_paths.items()}
assert all(check['status'] == 'passed' for check in checks.values())
for name in ['current_clippy', 'current_Rust_format']:
    for source in checks[name]['sources']:
        assert seal(source['path']) == source
fuel_path = root / 'final/rust-tests/certificates/WrongMulDiscard-64.rust.json'
assert json.loads(read(fuel_path)['error_detail'])['code'] == 'CORE_FUEL_EXHAUSTED'
sources = [source['path'] for source in focused['sources']]
phase_scopes = {
    'initial/rust-tests/result.json': 'Original PowerTwoWeight1 intended-positive typing failure, independently reproduced in Go; repair needs explicit numeral-sum congruence.',
    'initial/rust-diagnostic/result.json': 'Temporary diagnostic uses wrong package-relative path; no proof observation.',
    'corrected-path/rust-diagnostic/result.json': 'Corrected path locates original first failed declaration; prefixes are diagnostic only.',
    'product/rust-tests/result.json': 'Complete draft reaches construction limit before certificate emission.',
    'envelope/rust-construction/result.json': 'Temporary diagnostic finds3736 declarations/131072 terms atW64.DigitsBlock0At35; limits stay fixed.',
    'shared-vectors/rust-construction/result.json': 'Shared vector conversions fit3812 declarations/89187 terms/1228117 bytes; scalar projection conversion fuel-exhausts atPushConservation4.',
    'shared-vectors/rust-diagnostic/result.json': 'Temporary diagnostic compilation moves owned input path; original compiler output retained.',
    'shared-vectors-focused/rust-diagnostic/result.json': 'Borrow correction locatesPushConservation4; no complete canonical acceptance.',
    'explicit-projections/rust-construction/result.json': 'Explicit projection/Top/Zero lifting advances toW32.SelectZeroValue fuel failure.',
    'explicit-projections/rust-diagnostic/result.json': 'Diagnostic prefixes locate selection failure;8/16-bit declarations check, not complete certificates.',
    'explicit-selection/rust-construction/result.json': 'Explicit word selection lifting advances through32-bit arithmetic toW64.LeftStep1 full-vector conversion fuel failure in both cores.',
    'explicit-selection/rust-diagnostic/result.json': 'Diagnostic prefixes locateW64.LeftStep1; shared literal-bit value congruence repairs it without raising fuel/bounds.',
}
record = {
    'schema': 'mpk.go_practical.actual_word_arithmetic_checkpoint.v1',
    'status': 'component_checks_passed_work_item_in_progress',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'entry_commit': '51c43828140acb6095c27a6edc5980e36ab28bbb',
    'production_sources': [seal(path) for path in sources if path.startswith('crates/mpk-vc/src/')],
    'current_test_and_model_sources': [seal(path) for path in sources if path.startswith('crates/mpk-vc/tests/')],
    'standard_raw_inputs': [seal(path) for path in sources if path.startswith('proofs/std/')],
    'fresh_component': {
        'certificate': seal(certificate), 'hashes': report['hashes'],
        'declarations': 3828, 'terms': 101398, 'bytes': 1358764, 'axioms': 0,
        'new_ordinary_declarations': 1249, 'widths': [8, 16, 32, 64],
        'frozen_candidate_bounds': {'bytes': 2097152, 'declarations': 4096, 'terms': 131072},
        'all_artifact_bounds_unchanged_and_met': True,
        'unbounded_Nat_exponent_addition': True, 'word_width_weight_bridges': 7,
        'actual_multiplication_steps': 120, 'balanced_digit_prefix_blocks': 236,
        'literal_bit_value_congruence_laws': 4,
        'fixed_left_shift_scope': 'All fixed0<=i<width, actual Left_i/actual dropped highest bits and genuine PowerTwo(i). General variable-count/signed/right-shift numeric/source laws remain due.',
        'product_law': 'Arbitrary full words and unbounded initial Nat: initial+Ux*Uy=initial+(U(actual Mul(x,y))+modulus*actual computed Nat MulDiscard_width(x,y)).',
        'witness': 'Ordinary Nat recurrence of actual selected shift-discard amounts and actual ripple carries; all earlier definitions remain exact.',
        'proof_construction': 'Explicit word projection/selection/definition and Bool Top equalities lift through Value/Bit before scalar arithmetic. One generic address-elimination congruence law per width consumes actual independently checked literal-bit equalities. Whole operands are arbitrary, never enumerated; no function extensionality, arithmetic opcode, old seed or caller quotient authority.',
        'exact_import': 'Complete deterministic regeneration; changed/oversized/honestly different valid components reject. Full foundation/profile/registry/context remains due.',
    },
    'retained_numeric_basis': {
        'original_verification': seal(previous_path), 'current_bytes': seal(old),
        'hashes': read(old.with_suffix('.rust.json'))['hashes'],
        'declarations_retained': 2579, 'term_nodes_retained': 55105,
        'complete_original_bytes_identical': True,
    },
    'Rust_verification_scope': {
        'original_execution': seal(original_path), 'original_passing_roots': 3,
        'affected_root_stopped_on_resource_failure': 1,
        'focused_corrective_execution': seal(focused_path), 'focused_passed_roots': 1,
        'focused_changed_or_unreached_cases': 5, 'unaffected_typed_rejections_retained': 18,
        'logical_roots_covered': 4,
        'test_only_source_change': seal(root / 'review/control-source-change.json'),
        'reason': 'Complete/import/basis, three generic controls and48-query/1440-stage model roots pass. Preserve one64-bit altered-witness fuel failure. Change only four witness mutations to constant0 and run those plus the unreached64-bit shift control. Eighteen unaffected typed rejections and all passing producer/model/native/standard bytes remain exact; do not rerun unrelated roots.',
    },
    'paired_current_checkers': seal(paired_path), 'same_byte_cases': 26,
    'accepted_zero_axiom_with_exact_three_hashes': 3, 'rejected_core_type_mismatch': 23,
    'core_mismatch_subtypes': dict(counts),
    'negative_scope': {
        'wrong_unbounded_power_or_push': 2, 'ignored_actual_shift_premise': 1,
        'omitted_shift_contribution': 4, 'omitted_product_contribution': 4,
        'wrong_actual_Left1': 4, 'wrong_computed_shift_discard': 4,
        'wrong_computed_multiplication_discard_constant_zero': 4,
        'original_dropped_witness_addend_64_fuel_failure_counted': False,
    },
    'original_failed_control': {
        'certificate': seal(root / 'final/rust-tests/certificates/WrongMulDiscard-64.mpcert'),
        'Rust_report': seal(fuel_path), 'code': 'CORE_FUEL_EXHAUSTED',
        'semantic_negative_counted': False,
        'proof_checker_producer_model_or_budget_change_for_repair': False,
    },
    'initial_observations': [{'record': seal(root / path), 'scope': scope} for path, scope in phase_scopes.items()],
    'original_Go_rejection': seal(root / 'initial/go-rejection/result.json'),
    'original_explicit_selection_Go_fuel_observation': seal(root / 'explicit-selection/go-candidate/result.json'),
    'same_final_byte_Go_smoke_acceptance': seal(root / 'literal-values/go-candidate/result.json'),
    'diagnostic_harness_archived_exact_source': seal(root / 'diagnostics/declaration-prefix-harness.rs'),
    'diagnostic_harness_removed_from_active_discovery': True,
    'temporary_construction_stderr_diagnostic_removed': True,
    'checks': {name: {'record': seal(root / path), 'status': checks[name]['status'],
                     'wall_seconds': checks[name].get('wall_seconds', checks[name].get('seconds'))}
               for name, path in check_paths.items()},
    'actual_finite_observations': {
        'comparison': seal(comparison_path), 'native_execution': seal(root / 'native/result.json'),
        'query_origins': seal(root / 'query-origins.json'),
        'new_native_source': seal('develop/probes/go-practical/arithmetic_observer.go'),
        'model_metrics': seal(root / 'final/rust-tests/model/metrics.json'),
        'selected_original_queries': 48, 'actual_fixed_shift_and_product_stages': 1440,
        'observed_true_carries': 107, 'AST_steps': 181613956,
        'maximum_query_AST_steps': 13336376, 'two_native_outputs_byte_identical': True,
        'complete_prepared_image_files': 5577,
        'actual_Boolean_certificate_terms_executed': True, 'Nat_witness_declarations_executed': False,
        'scope': 'Actual Boolean-only Left_i/Mul_i/carry terms execute. Host derives scalar witness recurrence values; independent Python integers and native typed Go multiplication/shifts/carries/math-big products/quotients agree at every stage. Universal actual Nat witness equations check separately in both cores. No enormous unary execution, arithmetic model callback, whole Go source/program or Linux release claim.',
    },
    'frozen_design_preservation': seal(root / 'frozen-design-preservation.json'),
    'strict_changed_input_review': 'input-review/changed-inputs-review.json',
    'CSharp_tests_executed': False, 'T02_W01_complete': False, 'public_profile_activated': False,
    'source_program_proof_claimed': False, 'authoritative_immutable_application_replay_claimed': False,
    'native_Linux_gate_claimed': False, 'all_Go_application_demands_remaining': 169,
    'kernel_reduction_axiom_fuel_or_artifact_bounds_changed': False,
    'remaining_work': 'Word bounds/order/modulo/division/remainder; variable/signed/right shifts; signed Int multiplication and complete source-typed nominal/operation correspondence; full carriers/strings/products/errors/slices/storage/lifetimes/definedness/program foundation,56-family descriptor/private registry/context and authoritative immutable source/export/selected-closure replay. Later T02-T11 gates remain.',
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
}
(root / 'verification.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'status': record['status'], 'raw_certificate': record['fresh_component']['certificate'],
                  'paired_cases': 26, 'core_negatives': 23, 'finite_stages': 1440}))
