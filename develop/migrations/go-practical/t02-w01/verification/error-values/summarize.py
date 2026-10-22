"""Bind terminal targeted checks and retain every original failed epoch."""
from pathlib import Path
import collections
import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


checks = {}
for epoch in ['final', 'affected', 'lint', 'format']:
    path = ROOT/epoch/'result.json'
    data = json.loads(path.read_bytes())
    assert data['status'] == 'passed' and data['exit_code'] == 0 and data['whole_source_inputs_unchanged']
    for key in ['stdout', 'stderr', 'source_receipt', 'actual_driver_prelaunch']:
        assert seal(data[key]['path']) == data[key]
    for row in json.loads(Path(data['source_receipt']['path']).read_bytes())['sources']:
        assert seal(row['path']) == {k:row[k] for k in ['path', 'bytes', 'sha256']}
        assert Path(row['path']).stat().st_mode & 0o777 == row['mode']
    checks[epoch] = seal(path)
pair_path = ROOT/'final-paired/result.json'
paired = json.loads(pair_path.read_bytes())
assert paired['status'] == 'passed'
assert (paired['same_byte_streams'], paired['zero_axiom_positives'], paired['genuine_core_type_mismatches']) == (169, 2, 167)
mismatches = collections.Counter()
for case in paired['cases']:
    for key in ['certificate', 'rust_report', 'stdout', 'stderr']:
        assert seal(case[key]['path']) == case[key]
    rust, go = case['rust_observation'], case['go_observation']
    assert rust['verdict'] == go['verdict']
    if rust['verdict'] == 'accepted':
        assert rust['axiom_count'] == 0 and not rust['axiom_report']['entries']
        assert rust['hashes'] == go['hashes']
    else:
        detail = json.loads(rust['error_detail'])
        assert detail['code'] == 'CORE_TYPE_MISMATCH'
        mismatches[detail['details']['kind']] += 1
foundation = ROOT/'final/certificates/error-values.mpcert'
initial_success = ROOT/'computed-majors/certificates/error-values.mpcert'
assert foundation.read_bytes() == initial_success.read_bytes()
assert foundation.with_suffix('.rust.json').read_bytes() == initial_success.with_suffix('.rust.json').read_bytes()
assert foundation.stat().st_size == 471001
report = json.loads(foundation.with_suffix('.rust.json').read_bytes())
assert report['declaration_count'] == 2050 and report['axiom_count'] == 0
observations = ROOT/'final/certificates/finite-observations.json'
observed = json.loads(observations.read_bytes())
assert len(observed['rows']) == 528
assert {r['tag_hex'] for r in observed['rows']} == {f'{n:016x}' for n in [0, (1 << 64)-1, *[1 << i for i in range(64)]]}
assert {tuple(r['payload']) for r in observed['rows']} == {(a, b) for a in [False, True] for b in [False, True]}
assert all(r['nil'] == (int(r['tag_hex'], 16) == 0) and r['non_nil'] != r['nil']
           and r['entire_tag_and_both_payload_fields_retained']
           and all(c['matched'] == (c['case_hex'] == r['tag_hex']) for c in r['cases']) for r in observed['rows'])
frozen_path = ROOT.parent/'product-relations/frozen-design-preservation.json'
frozen = json.loads(frozen_path.read_bytes())
assert len(frozen['inputs']) == 255
for row in frozen['inputs']:
    assert seal(row['path']) == row
epochs = []
for path in sorted(ROOT.glob('*/inputs.json')):
    data = json.loads(path.read_bytes())
    result_path = path.parent/'result.json'
    result = json.loads(result_path.read_bytes())
    assert result['status'] in {'passed', 'failed'}
    assert result['source_receipt'] == seal(path) and data['actual_driver_prelaunch'] == seal(path.parent/'driver.py')
    assert result['whole_source_inputs_unchanged'] or result['formatter_mutation_expected']
    assert all(seal(result[key]['path']) == result[key] for key in ['stdout', 'stderr'])
    epochs.append({'result': seal(result_path), 'status': result['status'], 'command': result['command'], 'preimages': len(data['sources'])})
failed = [row for row in epochs if row['status'] == 'failed']
assert {Path(row['result']['path']).parent.name for row in failed} == {'first', 'diagnosis', 'congruence', 'nil-laws', 'bit-separations', 'current'}
classification = {
    'first': 'Original full64 projection/comparator conversion fuel failure; actual Go observation also retained. Not a semantic negative.',
    'diagnosis': 'Diagnostic prefix observations locate IsNilOfMake fuel failure. Original full stream and complete sources retained; diagnostic prefix streams reconstruct from the retained driver/source but were not individually persisted.',
    'congruence': 'Diagnostic prefix observations locate NilIsNil: direct reflexivity omits required explicit computed-major rewriting. Not a passing/semantic-negative candidate.',
    'nil-laws': 'Diagnostic prefix observations locate closed nil alias conversion fuel failure. Not a semantic negative.',
    'bit-separations': 'Diagnostic prefix observations locate TagOneIsNonzero: direct reflexivity omits computed-major rewriting at the actual low bit. Not a passing/semantic-negative candidate.',
    'current': 'Original appended test declaration names violated certificate naming syntax. Both failed test roots and original malformed candidates remain; no canonical rejection is counted as a semantic control.',
}
for row in failed:
    row['classification'] = classification[Path(row['result']['path']).parent.name]
affected_out = (ROOT/'affected/stdout.log').read_text()
assert '4 passed; 0 failed' in affected_out and affected_out.count('4 passed; 0 failed') == 2
verification = {
    'schema': 'mpk.go_practical.error_value_checkpoint.v1', 'status': 'scoped_checks_passed',
    'basis_commit': 'ab353e38981c28949f3d09a6f4b54099c1ed06bc', 'language_scope': ['Go', 'Rust'],
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_prelaunch_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'component': {'certificate': seal(foundation), 'declarations': 2050, 'terms': 33772, 'bytes': 471001,
                  'axioms': 0, 'imports': 0, 'new_definitions': 9, 'new_laws': 90,
                  'logical_hashes': report['hashes'], 'complete_first_success_and_final_bytes_reports_equal': True},
    'retained_entire_named_libraries': {'word': {'declarations': 1892, 'raw_sha256': '23c88fb260bf9d27c2defb64a892f275653cd1741ed030f00a911fed1b2aa11c'},
                                      'product_relations': {'declarations': 94, 'raw_sha256': '584cbf2e2a70ce0d86f7703b754e9a22fd84dcd59ffd42e6df00e22fdb2a0d1d'}},
    'current_checks': checks, 'new_Rust_roots': 4, 'affected_old_Rust_roots': 8,
    'paired': seal(pair_path), 'same_byte_library_streams': 169, 'zero_axiom_positives': 2,
    'genuine_core_type_mismatches': 167, 'mismatch_subtypes': dict(mismatches),
    'false_law_controls': 90, 'erased_actual_premises': 69, 'definition_corruptions': 5, 'false_concrete_claims': 3,
    'finite_actual_term_observations': seal(observations), 'finite_observations': 528,
    'single_bit_positions': 64, 'complete_two_field_payload_patterns': 4,
    'finite_interpreter_steps': observed['total_interpreter_steps'],
    'source_epochs': len(epochs), 'source_preimages': sum(row['preimages'] for row in epochs), 'actual_executions': epochs,
    'original_failed_epochs': failed, 'first_actual_Go_failure_observation': seal(ROOT/'first-go-observation/result.json'),
    'original_review_bootstrap_failure': seal(ROOT/'input-review-first/result.json'),
    'unchanged_frozen_inputs': {'manifest': seal(frozen_path), 'count': 255},
    'selection_reason': 'New ordinary tagged observation definitions/laws and complete Word/Product.Relation dependencies. New tests verify all laws, all actual premises, definition corruption, full64/whole payload observations and exact ownership. All old named identities and raw producer hashes are pinned; affected old producer/import/guard consumers pass. The unchanged1640-query word-operation matrix is retained without rerunning it; no word operation changed. No unrelated semantic tests or equivalent intermediate full gate.',
    'limits': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152, 'conversion_fuel': 1024,
               'ordinary_interpreter_steps': 50000000, 'ordinary_interpreter_values': 2000000},
    'kernel_evaluator_axiom_import_or_budget_changes': False,
    'exact_frozen_Error_U_layout_or_validity_proved': False, 'source_universe_or_dispatch_proved': False,
    'Go_production_frontend_changed': False, 'frontend_rebuild_repeated': False,
    'agents_used': False, 'GitHub_Actions_used': False,
    'complete_R01_R06_W01': False, 'public_profile_activated': False,
    'all_actual_validation_handles_terminal': True,
    'full_T02_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed': False,
    'remaining': 'The pair is an observation view, not the frozen balanced n-case carrier. Checked layout adapters, exact source U/tag assignment, canonical nil/inactive-field zeros and case payload validity, backing-store and dynamic dispatch safety/receiver/precondition/frame/result obligations remain. Full56 ordinary foundation/private registry/descriptor/context, canonical source/capture/module roles and scope/contract wire, source/runtime/state/program proofs, native Linux and all169 application demands/T02-T11 remain due. The169 local library streams are distinct from the application demands. Earlier historical-scope scanner findings remain for final-gate resolution.',
}
(ROOT/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps({k:verification[k] for k in ['status', 'source_epochs', 'source_preimages', 'new_Rust_roots', 'affected_old_Rust_roots', 'same_byte_library_streams', 'genuine_core_type_mismatches']}))
