"""Bind scoped current observations; no test or checker is rerun here."""
from pathlib import Path
import collections
import datetime
import hashlib
import json
import sys

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt = sys.argv[1]
assert attempt.replace('-', '').isalnum()

def read(path):
    return json.loads(Path(path).read_text())

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

execution_path = root / attempt / 'rust-tests/result.json'
execution = read(execution_path)
assert execution['status'] == 'passed'
assert '4 passed; 0 failed' in (root / attempt / 'rust-tests/stdout.log').read_text()
for source in execution['sources']:
    assert seal(source['path']) == source
    snapshot = seal(root / attempt / 'rust-tests/inputs' / source['path'])
    assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
paired_path = root / ('paired-' + attempt) / 'result.json'
paired = read(paired_path)
assert paired['status'] == 'passed' and len(paired['cases']) == 31
comparison_path = root / ('native-comparison-' + attempt) / 'result.json'
comparison = read(comparison_path)
assert comparison['status'] == 'passed' and comparison['selected_original_queries'] == 272
certificate = root / attempt / 'rust-tests/certificates/word-order-foundation.mpcert'
report = read(certificate.with_suffix('.rust.json'))
assert report['verdict'] == 'accepted' and report['declaration_count'] == 3437 and report['axiom_count'] == 0
assert len(certificate.read_bytes()) == 985945
assert certificate.read_bytes() == (root / 'equality-explicit-blocks/rust-construction/certificates/word-order-foundation.mpcert').read_bytes()
assert certificate.read_bytes() == (root / attempt / 'rust-tests/model/certificate.mpcert').read_bytes()
previous_path = root.parent / 'word-numeric/verification.json'
previous = read(previous_path)
old = root / attempt / 'rust-tests/certificates/retained-numeric-basis.mpcert'
assert old.read_bytes() == Path(previous['fresh_component']['certificate']['path']).read_bytes()
negative_names = {'WrongNotLeaf', 'FalseTotal', 'WrongOppositeCarry', 'IgnoredBound'}
negative_names |= {f'{name}-{width}' for width in [8, 16, 32, 64]
                   for name in ['FalseBound', 'WrongSub', 'FalseLess', 'FalseEqual',
                                'FalseZero', 'FakeDivisorGuard']}
counts = collections.Counter()
observed_negatives = set()
for case in paired['cases']:
    observation = case['rust_observation']
    if observation['verdict'] == 'rejected':
        observed_negatives.add(case['name'])
        detail = json.loads(observation['error_detail'])
        assert detail['code'] == 'CORE_TYPE_MISMATCH'
        counts[detail['details']['kind']] += 1
assert negative_names == observed_negatives and sum(counts.values()) == 28
checks = {}
for name, path in [('strict_clippy', root / attempt / 'clippy/result.json'),
                   ('Rust_format', root / attempt / 'format/result.json')]:
    check = read(path)
    assert check['status'] == 'passed'
    for source in check['sources']:
        assert seal(source['path']) == source
    checks[name] = {'record': seal(path), 'status': check['status'],
                    'wall_seconds': check['wall_seconds']}
fuel = read(root / 'equality/rust-construction/certificates/word-order-foundation.rust.json')
assert json.loads(fuel['error_detail'])['code'] == 'CORE_FUEL_EXHAUSTED'
diagnosis = read(root / 'equality-first-declaration/rust-diagnostic/prefix-diagnosis.json')
assert diagnosis['first_failed_declaration_name'] == 'Mpk.Go.Practical.Word.Order.W64.CarryPairEquality'
assert not Path('crates/mpk-vc/tests/go_practical_word_order_diagnostic.rs').exists()
metrics_path = root / attempt / 'rust-tests/model/metrics.json'
metrics = read(metrics_path)
assert metrics['Nat_denotation_order_or_theorem_bodies_executed'] is False
phases = {
    'initial/rust-construction/result.json': 'Initial complement/bounds/subtraction/unsigned order component checks in Rust; equality/zero/guard laws not yet attached. Diagnostic unoptimized profile only; existing checker rules/fuel unchanged.',
    'equality/rust-construction/result.json': 'Complete first equality/zero/guard candidate emits3201 declarations/67993 terms/934695 bytes but exhausts conversion fuel at64-bit root equality; not a semantic negative or acceptance.',
    'equality-first-declaration/rust-diagnostic/result.json': 'Direct in-memory declaration-prefix diagnosis locates W64.CarryPairEquality; no complete canonical-certificate acceptance claim.',
    'equality-explicit-blocks/rust-construction/result.json': 'Explicit236 balanced block-fold correspondence laws avoid implicit complete-tree conversion; the same final3437-declaration bytes pass without changed bounds/fuel.',
    'final/rust-tests/result.json': 'First final harness compilation fails on borrowed temporary argument arrays in the observation helper; no test root executes. Original exact helper/source/compiler output retained. Named argument storage fixes only that helper.',
}
record = {
    'schema': 'mpk.go_practical.actual_word_order_checkpoint.v1',
    'status': 'component_checks_passed_work_item_in_progress',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'entry_commit': '126552426eea2d97aefb9b327c6c6ca58081e12e',
    'production_sources': [s for s in execution['sources'] if s['path'].startswith('crates/mpk-vc/src/')],
    'current_test_and_model_sources': [s for s in execution['sources'] if s['path'].startswith('crates/mpk-vc/tests/')],
    'standard_raw_inputs': [s for s in execution['sources'] if s['path'].startswith('proofs/std/')],
    'fresh_component': {
        'certificate': seal(certificate), 'hashes': report['hashes'],
        'declarations': 3437, 'terms': 71143, 'bytes': 985945, 'axioms': 0,
        'new_ordinary_declarations': 858, 'widths': [8, 16, 32, 64],
        'frozen_candidate_bounds': {'bytes': 2097152, 'declarations': 4096, 'terms': 131072},
        'all_artifact_bounds_unchanged_and_met': True,
        'complement_law': 'U(x)+U(actual Not(x))+1=modulus.',
        'strict_value_bound': 'Nat.Less(U(x),modulus)=true for every full word.',
        'actual_subtraction_law': 'U(x)+modulus=(U(actual Sub(x,y))+U(y))+modulus*Bit(actual final Carry(x,Not(y),true)).',
        'comparison_laws': 'Actual unsigned < and <= equal genuine Nat.Less and Nat.LessEqual of the same unbounded natural values.',
        'equality_laws': 'Conjunction of opposite actual subtraction carries equals actual bitwise Word.Equal, which equals genuine Nat.Equal of the word values. Word.NotEqual reflects its negation.',
        'balanced_equality_blocks': 236, 'explicit_literal_block_correspondence_laws': 236,
        'zero_and_actual_divisor_guard': 'Actual Word.Zero has value0; actual IsZero equals Nat.Equal(value,0); actual DivideDefined equals Nat.Less(0,value). Its actual equality-to-true premise implies positive value.',
        'proof_construction': 'Direct pointwise complement and explicit word projection/definition equalities lift through Value. Checked unbounded Nat semiring/cancellation/translation laws derive the actual carry order. Double Nat induction proves total equality; finite Bool cases concern local carry flags or selectors only. Small named block folds connect to the literal actual Equal tree. No function extensionality, caller truth substitution, arithmetic opcode, old seed or larger reduction budget.',
        'exact_import': 'Complete deterministic regeneration; altered, oversized, retained other component and honestly different valid component reject. This owning-byte component supplies no complete profile/registry/context authority.',
    },
    'retained_numeric_basis': {
        'original_verification': seal(previous_path), 'current_bytes': seal(old),
        'hashes': read(old.with_suffix('.rust.json'))['hashes'],
        'declarations_retained': 2579, 'term_nodes_retained': 55105,
        'complete_original_bytes_identical': True,
    },
    'retained_arithmetic_generation': {
        'original_verification': seal(root.parent / 'word-arithmetic/verification.json'),
        'raw_sha256': '2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c',
        'scope': 'The changed shared producer is regenerated by the new root and its complete preceding arithmetic bytes remain exact. The already passed unaffected arithmetic proof/model/native suites are not rerun.',
    },
    'Rust_execution': seal(execution_path), 'passed_affected_roots': 4,
    'paired_current_checkers': seal(paired_path), 'same_byte_cases': 31,
    'accepted_zero_axiom_with_exact_three_hashes': 3, 'rejected_core_type_mismatch': 28,
    'core_mismatch_subtypes': dict(counts), 'negative_case_names': sorted(negative_names),
    'initial_observations': [{'record': seal(root / p), 'scope': scope} for p, scope in phases.items()],
    'first_complete_candidate_Go_fuel_observation': seal(root / 'equality/go-candidate/result.json'),
    'same_final_byte_Go_smoke_acceptance': seal(root / 'equality-explicit-blocks/go-candidate/result.json'),
    'diagnostic_harness_archived_exact_source': seal(root / 'diagnostics/declaration-prefix-harness.rs'),
    'diagnostic_harness_removed_from_active_discovery': True,
    'unconnected_control_draft_retained_without_execution_claim': seal(root / 'diagnostics/unconnected-controls-first-draft.rs'),
    'checks': checks,
    'actual_finite_observations': {
        'comparison': seal(comparison_path), 'native_execution': seal(root / 'native/result.json'),
        'query_origins': seal(root / 'query-origins.json'), 'model_metrics': seal(metrics_path),
        'selected_original_queries': 272, 'word_results': 80, 'boolean_results': 192,
        'observed_zero_divisors': metrics['observed_zero_divisors'],
        'observed_borrows': metrics['observed_borrows'],
        'AST_steps': metrics['total_AST_steps'], 'maximum_query_AST_steps': metrics['maximum_query_AST_steps'],
        'two_native_outputs_byte_identical': True, 'complete_prepared_image_files': 5577,
        'actual_Boolean_certificate_terms_executed': True,
        'Nat_denotation_order_or_theorem_bodies_executed': False,
        'scope': comparison['scope'],
    },
    'frozen_design_preservation': seal(root / 'frozen-design-preservation.json'),
    'strict_changed_input_review': 'input-review/changed-inputs-review.json',
    'CSharp_tests_executed': False, 'T02_W01_complete': False, 'public_profile_activated': False,
    'source_program_proof_claimed': False, 'authoritative_immutable_application_replay_claimed': False,
    'native_Linux_gate_claimed': False, 'all_Go_application_demands_remaining': 169,
    'kernel_reduction_axiom_fuel_or_artifact_bounds_changed': False,
    'remaining_work': 'Actual modulo/division/remainder, variable/signed/right-shift numeric laws, signed Int multiplication and complete source-typed nominal/operation correspondence. Full carriers/strings/products/errors/slices/storage/lifetimes/definedness/program foundation,56-family descriptor/private registry/context and authoritative immutable source/export/selected-closure replay. This separate component does not establish that the entire future foundation fits all frozen bounds; complete bounded assembly remains due.',
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
}
(root / 'verification.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'status': record['status'], 'raw_certificate': record['fresh_component']['certificate'],
                  'paired_cases': 31, 'core_negatives': 28, 'finite_queries': 272}))
