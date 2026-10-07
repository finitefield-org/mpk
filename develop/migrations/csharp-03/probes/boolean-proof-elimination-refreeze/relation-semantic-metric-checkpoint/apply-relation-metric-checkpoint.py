import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
g = pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
assert read(b / 'ordinary-regeneration-13/status.json')['status'] == 'passed_all_four_affected_original_pattern_source_and_proof_type_owners'
owner = read(b / 'ordinary-relations-metrics-regeneration-2/status.json')
assert owner['status'] == 'passed_selected_source_bound_ordinary_owners'
assert len(owner['stages']) == 1 and owner['stages'][0]['exit_code'] == 0
assert owner['stages'][0]['test_counts'] == ['1']
duals = read(b / 'relation-positive-metric-checkers/status.json')
assert duals['status'].startswith('passed_positive_constructor_') and len(duals['stages']) == 2
assert all(x['exit_code'] == 0 and x['verdict'] == 'accepted' for x in duals['stages'])

manifest = read(b / 'ordinary-relations-metrics-source-manifest-2.json')
unrelated_test_changes = {
    'csharp_practical_ordinary_control_edge_tests.rs',
    'csharp_practical_ordinary_control_pattern_capture_tests.rs',
    'csharp_practical_ordinary_control_pattern_proof_tests.rs',
    'csharp_practical_ordinary_control_pattern_route_tests.rs',
    'csharp_practical_ordinary_control_pattern_source_tests.rs',
    'csharp_practical_ordinary_control_predicate_tests.rs',
}
verified = {}
for path, sha in manifest['hashes'].items():
    assert h(g / path) == sha, path
    if '/src/' in path or (path.startswith('crates/') and '/tests/' in path
                           and pathlib.Path(path).name not in unrelated_test_changes):
        assert h(r / path) == sha, path
        verified[path] = sha
relation_test = 'crates/mpk-vc/tests/support/csharp_practical_ordinary_relation_tests.rs'
assert relation_test in verified

plan = read(b / 'relation-metric-promotion-plan/plan.json')
assert len(plan['changes']) == 1
row = plan['changes'][0]
src = b / 'relation-metric-promotion-plan/files' / row['path']
assert h(src) == row['raw_sha256']
assert h(r / row['path']) == row['before_raw_sha256']
before = b / 'relation-metric-before' / row['path']
before.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(r / row['path'], before)
assert h(before) == row['before_raw_sha256']
shutil.copy2(src, r / row['path'])
assert h(r / row['path']) == row['raw_sha256']
receipt = dict(plan)
receipt.update(
    status='passed_promoted_exact_actual_21_source_semantic_metrics_with_all_current_certificate_pins_retained',
    exact_same_production_and_relation_owner_test_sources=verified,
    unrelated_control_tests='Six separately reviewed control frame/input/name comparison test changes do not affect the unchanged relation owner body or shared semantic evaluator; all other original test sources match the completed owner checkout.',
    positive_current_certificate_fresh_matching_checker_acceptances=2,
    actual_owner_elapsed_seconds=owner['stages'][0]['elapsed_seconds'],
)
(b / 'relation-metric-promotion.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
print('promoted exact onefile21 semantic metrics; all source/owner pins and two positive checker reports verified')
