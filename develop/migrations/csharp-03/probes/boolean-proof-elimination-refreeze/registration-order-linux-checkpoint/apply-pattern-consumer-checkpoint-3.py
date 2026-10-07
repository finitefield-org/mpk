import hashlib
import json
import pathlib
import shutil
from verify_registration_order_source_correspondence import verify_source_or_reviewed_registration_delta

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())

owners = read(b / 'ordinary-regeneration-13/status.json')
assert owners['status'] == 'passed_all_four_affected_original_pattern_source_and_proof_type_owners'
assert len(owners['stages']) == 4
assert all(s['exit_code'] == 0 and s['test_counts'] == ['1'] for s in owners['stages'])
duals = read(b / 'control-pattern-consumer-dual-checkers/status.json')
assert duals['status'] == 'passed_all_21_distinct_consumer_definition_certificates_both_checkers_zero_axioms'
assert len(duals['stages']) == 42
assert all(s['exit_code'] == 0 and s['verdict'] == 'accepted' for s in duals['stages'])

# The completed owner runner pins all production/test sources and its inputs.
reviewed_prior_changes = {}
prior_receipt = b / 'relation-metric-promotion.json'
if prior_receipt.exists():
    prior = read(prior_receipt)
    assert prior['status'].startswith('passed_promoted_exact_actual_21_')
    reviewed_prior_changes = {row['path']: row for row in prior['changes']}
review_append = read(b / 'published-capture-relation-review-append-audit.json')
assert review_append['status'] == 'passed_exact_published_review_append_only'
assert review_append['published_commit'] == 'fd8683e54cebdb898cfc9769f3cf9f7dbde7c6c6'
assert review_append['old_review_bytes_retained_exactly'] and not review_append['production_and_test_changes']
reviewed_prior_changes[review_append['path']] = review_append
assert set(reviewed_prior_changes) == {
    'develop/migrations/csharp-03/ordinary-foundation/relations/metrics.json',
    'develop/migrations/csharp-03/ordinary-foundation/unit-7-boolean-elimination-review.md',
}
for path, sha in read(b / 'ordinary-source-manifest-13.json')['hashes'].items():
    if path in reviewed_prior_changes:
        row = reviewed_prior_changes[path]
        assert sha == row['before_raw_sha256']
        assert h(r / path) == row['raw_sha256']
    else:
        verify_source_or_reviewed_registration_delta(r, path, sha)

equivalent = []
for family in ['with-pattern-primitives', 'with-pattern-routes', 'with-packed-pattern-proof-types']:
    stage = next(s for s in owners['stages'] if s['destination'] == family)
    original = b / 'ordinary-regeneration-13/goldens' / family
    actual = b / 'control-pattern-consumer-regeneration/generated' / family
    for path, sha in stage['generated_files'].items():
        a, z = original / path, actual / path
        assert h(a) == sha
        assert a.read_bytes() == z.read_bytes(), (family, path)
        equivalent.append({'family': family, 'file': path, 'raw_sha256': sha, 'byte_identical': True})
assert len(equivalent) == 126
unpacked = next(s for s in owners['stages'] if s['destination'] == 'unpacked-proof-types')
assert len(unpacked['generated_files']) == 36
for path, sha in unpacked['generated_files'].items():
    assert h(b / 'ordinary-regeneration-13/goldens/unpacked-proof-types' / path) == sha

plan = read(b / 'pattern-consumer-promotion-plan/plan.json')
assert len(plan['changes']) == 126
for row in plan['changes']:
    assert h(r / row['path']) == row['before_raw_sha256'], row['path']
    assert h(b / 'pattern-consumer-promotion-plan/files' / row['path']) == row['raw_sha256']

before = b / 'pattern-consumer-before'
before.mkdir(exist_ok=False)
for row in plan['changes']:
    dst = before / row['path']
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(r / row['path'], dst)
    assert h(dst) == row['before_raw_sha256']

for row in plan['changes']:
    shutil.copy2(b / 'pattern-consumer-promotion-plan/files' / row['path'], r / row['path'])
    assert h(r / row['path']) == row['raw_sha256']

receipt = dict(plan)
receipt.update(
    status='passed_promoted_54_actual_programs_four_original_owners_full_graphs_and_dual_acceptance',
    selected_original_owners_passed=4,
    source_correspondence_review="registration-order-checkpoint/receipt.json: exact old source pins plus explicitly reviewed canonical cases registration-only deltas; old checker execution remains attributed to its original binaries",
    selected_original_owner_equivalent_files=equivalent,
    original_unpacked_proof_type_files_verified=unpacked['generated_files'],
    fresh_checker_stages_passed=42,
    normal_pin_equivalence='All126 producer JSON/control/hex output files exactly match passed original-owner output files; promoted fixture bytes exactly match those output files. The OUTPUT branch executes the unchanged source/type/value/import/mutation checks and writes the exact bytes otherwise compared by the normal owner pin branch. No unaffected owner is repeated.',
)
(b / 'pattern-consumer-promotion.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
print('promoted exact126 files; all4 original owners and42 fresh checker stages pass')
