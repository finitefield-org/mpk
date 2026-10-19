"""Bind actual signed correction proofs and their exact scoped observations."""
from pathlib import Path
import collections
import datetime
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
diagnostic_source = 'crates/mpk-vc/tests/go_practical_word_signed_refinement_diagnostic.rs'

def read(path):
    return json.loads(Path(path).read_text())

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

lifecycle_path = root / 'auxiliary-diagnostic/lifecycle.json'
lifecycle = read(lifecycle_path)
assert lifecycle['status'] == 'ignored_by_default_after_all_original_executions_terminal'
assert seal(diagnostic_source) == lifecycle['after']

def current_sources(record, directory, production_only=False):
    assert record['status'] == 'passed' and record['exit_code'] == 0
    assert record['changed_inputs_during_execution'] == []
    adjusted_auxiliary = []
    for source in record['sources']:
        snapshot = seal(directory / 'inputs' / source['path'])
        assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
        if production_only and not source['path'].startswith('crates/mpk-vc/src/'):
            continue
        if seal(source['path']) == source:
            continue
        # Runtime/lint compiled the separate main target; the earlier formatter
        # also saw this auxiliary file. A later targeted format covers its tag.
        assert source['path'] == diagnostic_source and source == lifecycle['before']
        if record['command'][0] == 'rustfmt':
            assert diagnostic_source in record['command']
        else:
            assert '--test' in record['command'] and 'go_practical_word_signed_refinement' in record['command']
            assert 'go_practical_word_signed_refinement_diagnostic' not in record['command']
        before = (directory / 'inputs' / source['path']).read_text()
        assert Path(source['path']).read_text() == before.replace('#[test]\n', lifecycle['ignore_attribute'], 1)
        adjusted_auxiliary.append(source['path'])
    return adjusted_auxiliary

execution_path = root / 'final/rust-tests/result.json'
execution = read(execution_path)
auxiliary_in_execution = current_sources(execution, execution_path.parent)
stdout = (execution_path.parent / 'stdout.log').read_text()
assert '4 passed; 0 failed' in stdout and 'matched complete named prerequisites=3483' in stdout
certificate = execution_path.parent / 'certificates/word-signed-refinement.mpcert'
report = read(certificate.with_suffix('.rust.json'))
assert report['verdict'] == 'accepted' and report['declaration_count'] == 3897 and report['axiom_count'] == 0
assert report['hashes']['certificate'] == '6462496c5aa8708f1c03c3fa4f772c07a98287063733d91a5e9ccf027fd17127'
assert certificate.stat().st_size == 1390896
assert certificate.read_bytes() == (root / 'explicit-flag-transport/rust-construction/certificates/word-signed-refinement.mpcert').read_bytes()
assert certificate.read_bytes() == (execution_path.parent / 'model/certificate.mpcert').read_bytes()
paired_path = root / 'paired-final/result.json'
paired = read(paired_path)
assert paired['status'] == 'passed' and len(paired['cases']) == 62
expected = {'FalseSignedNatProjection', 'FalseScalarConservation', 'FakeScalarBalance', 'FalseScalarSign'}
expected |= {f'{name}-{width}' for width in [8, 16, 32, 64] for name in [
    'FalseCorrectionPositive', 'FalseCorrectionNegative', 'FalseSelectionRefinement',
    'FalseMagnitudeConservation', 'FakeMagnitudeGuard', 'FalseMagnitudeRemainderBound',
    'FalseQuotientRefinement', 'FalseRemainderRefinement', 'FalseMathematicalConservation',
    'FakeMathematicalGuard', 'FalseSourceConservation', 'FakeSourceGuard',
    'ChangedSignedDivSource', 'ChangedSignedRemSource']}
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
assert observed == expected and len(observed) == 60
assert positives == {'word-signed-refinement', 'honest-different-signed-refinement'}
checks = {}
for label, directory in [('strict_clippy', root / 'current/clippy'), ('Rust_format', root / 'current/format')]:
    path = directory / 'result.json'
    checked = read(path)
    auxiliary = current_sources(checked, directory)
    checks[label] = {'record': seal(path), 'status': 'passed', 'seconds': checked['wall_seconds'],
                     'captured_non_target_auxiliary_ignore_transition': auxiliary}
go_format_path = root / 'current/go-format/result.json'
go_format = read(go_format_path)
assert go_format['status'] == 'passed' and seal(go_format['source']['path']) == go_format['source']
checks['new_Go_format'] = {'record': seal(go_format_path), 'status': 'passed'}
for mode in ['diagnostic-clippy', 'diagnostic-format', 'diagnostic-list']:
    path = root / 'auxiliary-diagnostic' / mode / 'result.json'
    checked = read(path)
    assert current_sources(checked, path.parent) == []
    checks[mode] = {'record': seal(path), 'status': 'passed', 'seconds': checked['wall_seconds']}
listing = (root / 'auxiliary-diagnostic/diagnostic-list/stdout.log').read_text()
assert 'identify_original_first_failed_declaration_without_changing_the_checker: test' in listing
assert 'prefix=' not in listing
comparison_path = root / 'native-comparison-final/result.json'
comparison = read(comparison_path)
assert comparison['status'] == 'passed' and comparison['observations'] == 160
metrics_path = root / 'final/rust-tests/model/metrics.json'
metrics = read(metrics_path)
assert metrics['Nat_Int_correction_or_theorem_bodies_executed'] is False
assert metrics['universal_correction_classification_claimed'] is False
identities_path = root / 'retained-identities.json'
identities = read(identities_path)
assert identities['all_original_prerequisite_named_identities'] == 3483
assert identities['additional_original_Int_named_identities'] == 166
assert identities['new_signed_refinement_declarations'] == 248
assert identities['certificate']['sha256'] == seal(certificate)['sha256']
initial_path = root / 'actual-source-corrections/rust-construction/result.json'
initial = read(initial_path)
assert initial['status'] == 'failed' and initial['exit_code'] == 101 and initial['changed_inputs_during_execution'] == []
initial_go_path = root / 'actual-source-corrections/go-candidate/result.json'
assert read(initial_go_path)['Rust_observation']['verdict'] == read(initial_go_path)['Go_observation']['verdict'] == 'rejected'
diagnostic_path = root / 'first-mismatch/rust-diagnostic/diagnostic.json'
diagnostic = read(diagnostic_path)
assert diagnostic['first_failed_declaration_index'] == 3708
assert diagnostic['first_failed_declaration_name'] == 'Mpk.Go.Practical.Word.SignedDivision.ScalarSignedDivision'
construction_path = root / 'explicit-flag-transport/rust-construction/result.json'
construction = read(construction_path)
current_sources(construction, construction_path.parent, production_only=True)
candidate_path = root / 'explicit-flag-transport/go-candidate/result.json'
assert read(candidate_path)['Go_observation']['verdict'] == 'accepted'
previous = read(root.parent / 'word-signed-division/frozen-design-preservation.json')
assert previous['status'] == 'passed' and len(previous['inputs']) == 255
for source in previous['inputs']:
    assert seal(source['path']) == source
frozen = {**previous, 'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'scope': 'All255 original frozen design/specification/vector inputs remain exact. Actual signed quotient/remainder numeric refinement with computed corrections fits the unchanged4096/131072/2097152 limits. No core/axiom/fuel or source policy is widened.'}
write(root / 'frozen-design-preservation.json', frozen)
record = {
    'schema': 'mpk.go_practical.actual_signed_division_refinement_checkpoint.v1',
    'status': 'component_checks_passed_work_item_in_progress',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'entry_commit': '3c2d5b74b1a0844209c5b8ecc57704b28896299e',
    'production_sources': [source for source in execution['sources'] if source['path'].startswith('crates/mpk-vc/src/')],
    'current_target_test_model_and_inputs': [source for source in execution['sources'] if not source['path'].startswith('crates/mpk-vc/src/') and source['path'] != diagnostic_source],
    'fresh_component': {'certificate': seal(certificate), 'hashes': report['hashes'], 'declarations': 3897,
                        'terms': 94497, 'bytes': 1390896, 'axioms': 0, 'widths': [8, 16, 32, 64],
                        'retained_complete_named_prerequisites': 3483, 'additional_original_Int_proof_dependencies': 166,
                        'new_signed_refinement_declarations': 248, 'remaining_declaration_slots': 199,
                        'frozen_candidate_bounds': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152},
                        'all_bounds_unchanged_and_met': True,
                        'exact_import': 'Private owning deterministic complete regeneration rejects altered/oversized/honestly different valid components. No complete source descriptor/context/registry or practical-profile activation.'},
    'proof_scope': {
        'generic_signed_nat': 'SignedNat(flag,n)=Make(choose(flag,0,n),choose(flag,n,0)); exact Nat projections. Four local sign cases prove signed reconstruction from an actual Nat a=b*q+r premise, with Xor quotient sign and dividend remainder sign.',
        'actual_sign_selection': 'For arbitrary actual word q and Bool flag: desired +/-Uq is Equivalent to S(actual Select(flag,actual Neg(q),q))+M*SelectionCorrection(q,flag). Positive part chooses actual Neg(q) sign or actual q sign; negative part chooses actual DivideDefined(q) bit or0. Actual NegationValueSum, explicit word/projection equations and checked Nat algebra prove both branches.',
        'actual_magnitude_division': 'Actual Abs/DivideDefined transfer supplies the unsigned divisor premise; actual unsigned quotient/remainder satisfy UAbsx=UAbsy*Uq+Ur and Ur<UAbsy under the actual original source guard.',
        'mathematical_signed_reconstruction': 'Actual signed input magnitude equivalence and the generic scalar theorem yield Sx Equivalent to Sy*QM+RM, where QM/RM are computed from actual unsigned magnitude division and actual source flags.',
        'actual_source_refinement': 'Explicit exact Word SignedDiv/SignedRem definition theorems bind the actual selected words. QM Equivalent to Sq+M*CQ, RM Equivalent to Sr+M*CR, with ordinary computed balanced corrections. Original Int congruence/transitivity yields actual guarded Sx Equivalent to Sy*(Sq+M*CQ)+(Sr+M*CR). No caller quotient or truth flag.',
        'correction_classification_remaining': 'Universal quotient-half bounds, zero remainder correction and exact minimum/-one quotient correction classification remain required. Finite observations are not that proof.'},
    'retained_identity_observation': seal(identities_path),
    'Rust_execution': seal(execution_path), 'passed_affected_roots': 4,
    'same_byte_paired_checkers': seal(paired_path), 'same_byte_cases': 62,
    'accepted_zero_axiom_exact_hashes': 2, 'rejected_core_type_mismatches': 60,
    'core_mismatch_subtypes': dict(kinds), 'negative_case_names': sorted(expected),
    'changed_source_definitions_now_reject': [f'{name}-{width}' for width in [8, 16, 32, 64] for name in ['ChangedSignedDivSource', 'ChangedSignedRemSource']],
    'checks': checks,
    'original_intended_positive_rejection': {'Rust_execution': seal(initial_path), 'same_byte_Go_observation': seal(initial_go_path),
                                            'first_declaration_diagnostic': seal(diagnostic_path),
                                            'semantic_negative_counted': False,
                                            'recipe_repair': 'Computed Xor majors require a proved Bool literal result transported by actual Eq.congr through the Nat choice motive before literal reduction. Add only this explicit proof transport; preserve original failed full candidate/source/raw diagnostics and all diagnostic prefix observations. No checker or budget change.'},
    'passing_fixed_source_construction': seal(construction_path), 'passing_same_byte_initial_Go_inspection': seal(candidate_path),
    'construction_scope': 'The one-root fixed-source construction checks exact import and all original3483 named prerequisites. Its production inputs match the final producer exactly and its complete emitted bytes equal the expanded final4-root bytes. Test-only controls/model/extra honest component were added after the construction handle terminated. Original captured test sources remain preserved.',
    'captured_inputs_fixed_until_actual_terminal_handles': True,
    'auxiliary_diagnostic_lifecycle': {'record': seal(lifecycle_path), 'captured_non_target_input_adjusted_after_terminal': auxiliary_in_execution,
                                     'scope': 'Only the standalone diagnostic ignore attribute changes after all original Rust/Go executions terminate. Main target/library/control/model dependencies remain byte exact. Targeted diagnostic listing/lint/format validate its default-ignore lifecycle without repeating original prefix work or unaffected main tests. Strict changed-input scan still includes every current/source/snapshot byte; no source exclusion.'},
    'finite_observations': {'comparison': seal(comparison_path), 'native_execution': seal(root / 'native/result.json'),
                            'native_source': seal('develop/probes/go-practical/signed_refinement_observer.go'),
                            'query_origins': seal(root / 'query-origins.json'), 'model_metrics': seal(metrics_path),
                            'observations': metrics, 'two_fresh_native_outputs_identical': True, 'prepared_image_files': 5577,
                            'Nat_Int_correction_or_theorem_bodies_executed': False, 'universal_correction_classification_claimed': False},
    'frozen_design_preservation': seal(root / 'frozen-design-preservation.json'),
    'strict_changed_input_review': 'input-review/changed-inputs-review.json',
    'CSharp_tests_executed': False, 'T02_W01_complete': False, 'all_application_demands_remaining': 169,
    'public_profile_activated': False, 'whole_source_program_proof_claimed': False,
    'authoritative_immutable_application_replay_claimed': False, 'native_Linux_gate_claimed': False,
    'kernel_axiom_fuel_or_artifact_bounds_changed': False,
    'remaining_work': 'Universal correction/minimum classification and quotient/half bounds; remaining variable/signed/right-shift numeric/source laws, full source/carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All T02-T11 scope/gates remain.',
    'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
}
write(root / 'verification.json', record)
print(json.dumps({'status': record['status'], 'declarations': 3897, 'terms': 94497, 'bytes': 1390896,
                  'same_byte_cases': 62, 'semantic_negatives': 60, 'source_observations': 160,
                  'subtypes': dict(kinds)}))
