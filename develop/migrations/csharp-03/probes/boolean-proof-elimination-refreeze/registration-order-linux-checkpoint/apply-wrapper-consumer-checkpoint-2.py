import hashlib
import json
import pathlib
import shutil
from verify_registration_order_source_correspondence import verify_source_or_reviewed_registration_delta

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
j = pathlib.Path('/private/tmp/mpk-w09-wrapper-owner-validation')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
assert read(b / 'ordinary-regeneration-13/status.json')['status'] == 'passed_all_four_affected_original_pattern_source_and_proof_type_owners'
owners = read(b / 'wrapper-owner-validation-2/status.json')
assert owners['status'] == 'passed_all_14_affected_original_wrapper_owner_dependency_and_import_tests'
assert len(owners['stages']) == 14
assert all(x['exit_code'] == 0 and x['test_counts'] == ['1'] for x in owners['stages'])
duals = read(b / 'wrapper-program-dual-checkers/status.json')
assert duals['status'] == 'passed_all_115_changed_definition_certificates_both_checkers_zero_axioms'
assert len(duals['stages']) == 230
assert all(x['exit_code'] == 0 and x['verdict'] == 'accepted' for x in duals['stages'])
graphs = read(b / 'wrapper-program-name-graphs-2/status.json')
assert graphs['status'].startswith('passed_all_115_') and len(graphs['stages']) == 115
assert all(x['exit_code'] == 0 for x in graphs['stages'])

for path, sha in read(b / 'wrapper-owner-validation-2/source-manifest.json')['hashes'].items():
    assert h(j / path) == sha, path
    if path.startswith('crates/') or path in ['Cargo.toml', 'Cargo.lock']:
        verify_source_or_reviewed_registration_delta(r, path, sha)

plan = read(b / 'wrapper-program-promotion-plan-2/plan.json')
assert len(plan['changes']) == 288
old_plan = read(b / 'wrapper-program-promotion-plan/plan.json')
old_certificates = {x['path']: x for x in old_plan['changes'] if x['kind'] == 'certificate'}
new_certificates = {x['path']: x for x in plan['changes'] if x['kind'] == 'certificate'}
assert old_certificates == new_certificates and len(new_certificates) == 115
for row in plan['changes']:
    assert h(r / row['path']) == row['before_raw_sha256'], row['path']
    assert h(j / row['path']) == row['raw_sha256'], row['path']
    assert h(b / 'wrapper-program-promotion-plan-2/files' / row['path']) == row['raw_sha256']

before = b / 'wrapper-consumer-before'
before.mkdir(exist_ok=False)
for row in plan['changes']:
    dst = before / row['path']
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(r / row['path'], dst)
    assert h(dst) == row['before_raw_sha256']

for row in plan['changes']:
    shutil.copy2(b / 'wrapper-program-promotion-plan-2/files' / row['path'], r / row['path'])
    assert h(r / row['path']) == row['raw_sha256']

receipt = dict(plan)
receipt.update(
    status='passed_promoted_161_actual_wrapper_programs_14_original_owner_tests_115_full_graphs_and_230_checker_stages',
    normal_original_owner_tests_passed=14,
    fresh_matching_checker_acceptance_stages=230,
    complete_definition_graph_pairs=115,
    unchanged_actual_certificate_outputs_after_parent_identity_repair=115,
    original_owner_review_checkout='wrapper-owner-validation-checkout.json',
    exact_same_production_and_test_sources=False,
    exact_original_owner_source_pins_and_reviewed_registration_only_deltas="registration-order-checkpoint/receipt.json; owner source/tests unchanged in the frozen J checkout; candidate certificates lack the reserved cases interface",
    initial_original_owner_failure_retained='wrapper-owner-validation/status.json',
)
(b / 'wrapper-consumer-promotion.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
print('promoted exact288 wrapper files; all14 original owners115graphs230fresh checker stages pass')
