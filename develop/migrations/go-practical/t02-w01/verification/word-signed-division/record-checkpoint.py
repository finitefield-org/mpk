"""Bind scoped zero/composition checks without repeating unaffected suites."""
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

def current_sources(record):
    assert record['status'] == 'passed' and record['exit_code'] == 0
    assert record['changed_inputs_during_execution'] == []
    for source in record['sources']:
        assert seal(source['path']) == source

previous = read(root.parent / 'word-signed-bounds/frozen-design-preservation.json')
assert previous['status'] == 'passed' and len(previous['inputs']) == 255
for source in previous['inputs']:
    assert seal(source['path']) == source
frozen = {**previous, 'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'scope': 'All255 original frozen design/specification/vector members remain exact. Direct ordinary zero proofs and compact same-type proved substitution preserve all required signed-division prerequisites under the unchanged4096/131072/2097152 bounds; no core/axiom/fuel widening.'}
write(root / 'frozen-design-preservation.json', frozen)
comparison_path = root / 'native-comparison-final/result.json'
comparison = read(comparison_path)
assert comparison['status'] == 'passed' and comparison['original_queries'] == 128
go_format_path = root / 'current/go-format/result.json'
go_format = read(go_format_path)
assert go_format['status'] == 'passed' and seal(go_format['source']['path']) == go_format['source']

zero_negatives = {'FalseSumZero', 'FalsePositiveProductZero', 'IgnoredPositive', 'FalsePositiveNatural'}
zero_negatives |= {f'{name}-{depth}' for depth in range(7)
                   for name in ['AlwaysTrueBits', 'FalseCubeNatural', 'FalseCubeSign']}
zero_negatives |= {f'{name}-{width}' for width in [8, 16, 32, 64]
                   for name in ['FalseActualZero', 'FalseDefinedNatural', 'FakeDefinedPremise', 'FalseActualSign']}
division_negatives = {'IgnoredScopedPositive', 'FalseScopedHalf'}
division_negatives |= {f'{name}-{width}' for width in [8, 16, 32, 64]
                       for name in ['FalseUnsignedConservation', 'FakeUnsignedGuard', 'FalseRemainderBound',
                                    'FalseHalfBound', 'FalseScopedZero', 'FakeScopedDivisor', 'FalseMagnitudeSign']}

for component, name, construction, declarations, terms, byte_count, raw_hash, positives, negatives in [
    (root.parent / 'word-zero', 'word-zero-foundation', 'direct-balanced-zero', 274, 10538, 108827,
     '6cdac50b1b10d56baefa679b71add664d2dc3129af78552737233a3c5da4157e',
     {'word-zero-foundation', 'honest-different-zero-component'}, zero_negatives),
    (root, 'signed-division-prerequisites', 'joint-prerequisites', 3483, 85883, 1255288,
     '2d23c58db425fc11d653c8c7966db69a0f71495d614712cd7b8abc0f2246e11d',
     {'signed-division-prerequisites', 'honest-different-division-prerequisites', 'honest-different-signed-div-definition'}, division_negatives),
]:
    execution_path = component / 'final/rust-tests/result.json'
    execution = read(execution_path)
    current_sources(execution)
    for source in execution['sources']:
        snapshot = seal(component / 'final/rust-tests/inputs' / source['path'])
        assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
    stdout = (component / 'final/rust-tests/stdout.log').read_text()
    assert '4 passed; 0 failed' in stdout
    certificate = component / 'final/rust-tests/certificates' / (name + '.mpcert')
    report = read(certificate.with_suffix('.rust.json'))
    assert report['verdict'] == 'accepted' and report['declaration_count'] == declarations and report['axiom_count'] == 0
    assert seal(certificate)['sha256'] == raw_hash and certificate.stat().st_size == byte_count
    assert certificate.read_bytes() == (component / construction / 'rust-construction/certificates' / (name + '.mpcert')).read_bytes()
    assert certificate.read_bytes() == (component / 'final/rust-tests/model/certificate.mpcert').read_bytes()
    paired_path = component / 'paired-final/result.json'
    paired = read(paired_path)
    assert paired['status'] == 'passed' and len(paired['cases']) == len(positives) + len(negatives)
    observed_positive, observed_negative = set(), set()
    kinds = collections.Counter()
    for row in paired['cases']:
        rust, go = row['rust_observation'], row['go_observation']
        assert rust['verdict'] == go['verdict']
        if rust['verdict'] == 'accepted':
            assert rust['axiom_count'] == go.get('axiom_count', 0) == 0 and rust['hashes'] == go['hashes']
            observed_positive.add(row['name'])
        else:
            detail = json.loads(rust['error_detail'])
            assert detail['code'] == 'CORE_TYPE_MISMATCH'
            kinds[detail['details']['kind']] += 1
            observed_negative.add(row['name'])
    assert observed_positive == positives and observed_negative == negatives
    checks = {}
    for label, path in [('strict_clippy', component / 'current/clippy/result.json'),
                        ('Rust_format', component / 'current/format/result.json')]:
        checked = read(path)
        current_sources(checked)
        checks[label] = {'record': seal(path), 'status': 'passed', 'seconds': checked['wall_seconds']}
    checks['new_Go_format'] = {'record': seal(go_format_path), 'status': 'passed', 'seconds': go_format['seconds']}
    initial_path = component / construction / 'rust-construction/result.json'
    initial = read(initial_path)
    assert initial['status'] == 'passed' and not initial['changed_inputs_during_execution']
    later_production_changes = []
    for source in initial['sources']:
        if source['path'].startswith('crates/mpk-vc/src/'):
            snapshot = seal(component / construction / 'rust-construction/inputs' / source['path'])
            assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
            if seal(source['path']) != source:
                later_production_changes.append(source['path'])
    assert later_production_changes == (['crates/mpk-vc/src/go_practical_ordinary.rs'] if component.name == 'word-zero' else [])
    candidate_path = component / construction / 'go-candidate/result.json'
    assert read(candidate_path)['Go_observation']['verdict'] == 'accepted'
    metrics_path = component / 'final/rust-tests/model/metrics.json'
    record = {
        'schema': 'mpk.go_practical.' + component.name.replace('-', '_') + '_checkpoint.v1',
        'status': 'component_checks_passed_work_item_in_progress',
        'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'entry_commit': '1e994f48627bd165ac27cdc7d45099f5938bb36c',
        'production_sources': [source for source in execution['sources'] if source['path'].startswith('crates/mpk-vc/src/')],
        'test_model_and_original_inputs': [source for source in execution['sources'] if not source['path'].startswith('crates/mpk-vc/src/')],
        'fresh_component': {'certificate': seal(certificate), 'hashes': report['hashes'],
                            'declarations': declarations, 'terms': terms, 'bytes': byte_count, 'axioms': 0,
                            'widths': [8, 16, 32, 64], 'cube_depths': list(range(7)),
                            'frozen_candidate_bounds': {'bytes': 2097152, 'terms': 131072, 'declarations': 4096},
                            'all_bounds_unchanged_and_met': True,
                            'exact_import': 'Private owning deterministic full regeneration rejects altered/oversized/honestly different valid components. No complete descriptor/context/registry or activated source profile is claimed.'},
        'Rust_execution': seal(execution_path), 'passed_affected_roots': 4,
        'same_byte_paired_checkers': seal(paired_path), 'same_byte_cases': len(paired['cases']),
        'accepted_zero_axiom_exact_hashes': len(positives), 'rejected_core_type_mismatches': len(negatives),
        'core_mismatch_subtypes': dict(kinds), 'negative_case_names': sorted(negatives), 'checks': checks,
        'initial_passing_construction': seal(initial_path), 'same_final_byte_initial_Go_acceptance': seal(candidate_path),
        'construction_scope': 'One-root fixed-source construction preserves complete original bytes/raw diagnostics. Later controls/models and the additive composition API are covered by the final current-source verification; the emitted complete component bytes match the initial bytes exactly. No captured Rust inputs change before the terminal execution result.',
        'production_sources_changed_after_terminal_construction': later_production_changes,
        'later_change_scope': 'The direct-zero main module gains only the separate composition module declaration, owning prerequisite type/getter and signed-division-prerequisite generate/import functions. The original zero generator/import and every other original producer body remain exact. The joint-prerequisite initial production inputs already match current sources.',
        'finite_observations': {'comparison': seal(comparison_path), 'native_execution': seal(root / 'native/result.json'),
                                'native_source': seal('develop/probes/go-practical/division_prerequisite_observer.go'),
                                'query_origins': seal(root / 'query-origins.json'), 'metrics': seal(metrics_path),
                                'observations': read(metrics_path), 'native_original_queries': 128,
                                'two_fresh_native_outputs_identical': True, 'prepared_image_files': 5577},
        'frozen_design_preservation': seal(root / 'frozen-design-preservation.json'),
        'strict_changed_input_review': str(root / 'input-review/changed-inputs-review.json'),
        'CSharp_tests_executed': False, 'T02_W01_complete': False,
        'all_application_demands_remaining': 169, 'public_profile_activated': False,
        'universal_signed_division_or_remainder_theorem_claimed': False,
        'whole_source_program_proof_claimed': False, 'authoritative_immutable_application_replay_claimed': False,
        'native_Linux_gate_claimed': False, 'kernel_axiom_fuel_or_artifact_bounds_changed': False,
        'remaining_work': 'Actual signed division/remainder with minimum/-one behavior; remaining variable/signed/right-shift numeric/source laws, full source-typed carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All T02-T11 gates remain.',
        'full_T02_gate': 'deferred to T02-W04: check-fast.sh --skip-csharp',
    }
    if component.name == 'word-zero':
        assert 'matched named prerequisites=209' in stdout
        record['proof_scope'] = {
            'retained_named_prerequisites': 209, 'new_zero_declarations': 65,
            'retained_signed_bounds_raw_sha256': '8895b359ea61393aaf241c82e26366498f06cd12a0645e5ba37e3cbe14c41a31',
            'definitions': 'ZeroBits0(x)=Not(x); ZeroBits_d=And(ZeroBits_half(low),ZeroBits_half(high)). Exact actual IsZero bit equality to source zero reduces by explicit leaf congruences; no general word order/carry closure.',
            'unbounded_laws': 'SumZero, PositiveProductZero under an actual positive multiplier, and NaturalNonzeroPositive use ordinary Nat induction/cancellation. Existing complement balance proves actual weight positivity at every depth without giant unary literals.',
            'actual_word_laws': 'IsZeroBits, IsZeroNatural, DivideDefinedNatural, DefinedDivisorPositive and ZeroSign for all four actual widths. Every arbitrary bit participates; source false/true premises remain exact.',
        }
    else:
        assert 'every mapped declaration matches; actual substitutions=12' in stdout
        analysis_path = root / 'prerequisite-analysis/result.json'
        analysis = read(analysis_path)
        assert analysis['selected_name_union'] == 4173
        record['proof_scope'] = {
            'structural_planning_only': seal(analysis_path),
            'original_whole_union_declarations': 4570, 'original_selected_closed_union_declarations': 4173,
            'remaining_declaration_slots': 613, 'proved_same_type_substitutions': 12,
            'input_components': ['complete original unsigned division', 'complete original signed bounds', 'direct proved zero', 'complete original word/source definitions and Int library'],
            'substitution': 'Only12 compiled-in theorem interfaces: IsZeroNatural, DefinedDivisorPositive and ZeroSign for each width. Exact named-DAG types match; each replacement supplies its full real checked proof. Original complete generators and certificate bytes remain unchanged.',
            'composition': 'Private untrusted fixed roots. All common type/body/proof/level/family fingerprints must agree. Closed type/value/proof/primitive-family dependencies rebuild in a fresh explicit signed-division prerequisite namespace; iterative declaration order avoids recursion through64 stages. Every mapped output declaration independently matches rewritten original full metadata. All resulting proofs check normally in both cores.',
            'retained_actual_laws': 'Unsigned conservation/remainder bounds; actual signed magnitude equivalence/divisor transfer/negation value sum; half bound, half/minimum, magnitude-sign/minimum and unsigned-half/sign correspondence; actual SignedPositive/Negative and SignedDiv/SignedRem definitions; Int representations/projections.',
            'scope_control': 'An otherwise valid component replacing actual W8.SignedDiv by zero is accepted by both cores, demonstrating that no signed-division theorem is included yet. The exact owning importer rejects those different bytes. Actual signed arithmetic/source linkage remains required.',
        }
    write(component / 'verification.json', record)
    print(json.dumps({'component': component.name, 'status': record['status'],
                      'declarations': declarations, 'terms': terms, 'bytes': byte_count,
                      'paired_cases': len(paired['cases']), 'semantic_negatives': len(negatives),
                      'subtypes': dict(kinds)}))
