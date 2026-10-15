import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
owners = read(b / 'wrapper-owner-validation-2/status.json')
assert owners['status'] == 'passed_all_14_affected_original_wrapper_owner_dependency_and_import_tests'
assert len(owners['stages']) == 14
assert all(x['exit_code'] == 0 and x['test_counts'] == ['1'] for x in owners['stages'])
duals = read(b / 'wrapper-program-dual-checkers/status.json')
assert duals['status'] == 'passed_all_115_changed_definition_certificates_both_checkers_zero_axioms'
assert len(duals['stages']) == 230
assert all(x['exit_code'] == 0 and x['verdict'] == 'accepted' for x in duals['stages'])
promotion = read(b / 'wrapper-consumer-promotion.json')
assert promotion['status'].startswith('passed_promoted_161_actual_wrapper_programs_')
assert len(promotion['changes']) == 288
out = b / 'wrapper-final-checkpoint-evidence'
shutil.copytree(b / 'wrapper-checkpoint-evidence', out)
def copy_file(src, rel):
    assert src.is_file(), src
    dst = out / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    assert h(dst) == h(src)
for folder in ['wrapper-owner-validation-2', 'wrapper-program-dual-checkers']:
    for src in sorted((b / folder).rglob('*')):
        if src.is_file() and src.suffix != '.mpcert':
            copy_file(src, pathlib.Path(folder) / src.relative_to(b / folder))
for name in ['wrapper-parent-identity-second-review.json', 'wrapper-original-owner-completed-prefix-2-review.json', 'wrapper-consumer-promotion.json',
             'apply-wrapper-consumer-checkpoint-2.py', 'finalize-wrapper-checkpoint-evidence-2.py',
             'verify_registration_order_source_correspondence.py', 'registration-order-source-correspondence-guard-review.json', 'pending-foundation-current-overlay-inventory-8.json', 'audit-current-foundation-overlay-8.py', 'core-specification-correspondence-second-review.json', 'unused-refreeze-incremental-cache-cleanup-3.json',
    'full-semantic-finalizer-preparation-disk-failure.json',
    'audit-completed-checker-report-prefix.py', 'completed-checker-report-prefix-120-30-second-review.json']:
    copy_file(b / name, name)
archive = r / 'develop/migrations/csharp-03/ordinary-foundation/previous-context-7-wrapper-consumers'
archive.mkdir(exist_ok=False)
before_manifest = {}
for row in promotion['changes']:
    assert h(r / row['path']) == row['raw_sha256'], row['path']
    src = b / 'wrapper-consumer-before' / row['path']
    assert h(src) == row['before_raw_sha256'], row['path']
    rel = pathlib.Path(row['path']).relative_to('develop/migrations/csharp-03/ordinary-foundation')
    dst = archive / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    assert h(dst) == row['before_raw_sha256']
    before_manifest[str(rel)] = h(dst)
(archive / 'manifest.json').write_text(json.dumps({
    'status': 'immutable_complete_predecessor_wrapper_bytes_retained',
    'files': before_manifest,
}, sort_keys=True, indent=2) + '\n')
receipt = {
    'schema': 'mpk.csharp_practical.t01_w09.wrapper_consumer_checkpoint.v1',
    'status': 'passed_actual_generation_complete_lineage_full_graphs_14_original_owners_and_230_checker_stages',
    'selection_reason': 'The288 affected wrapper fixture files cover161 actually regenerated wrapper programs. Both checkers accept all115 changed certificate byte sets; the fourteen original source/dependency/import owners use normal candidate pins with all original value, mutation and complete declaration closure checks. Unaffected scalar/native matrices are excluded. Whole gates remain final T01-W10/T06-W12.',
    'newly_promoted_files': 288,
    'actual_wrapper_programs': 161,
    'full_changed_definition_graph_pairs': 115,
    'fresh_matching_zero_axiom_checker_acceptances': 230,
    'original_wrapper_owner_dependency_and_import_tests_passed': 14,
    'exact_current_production_and_test_sources': False,
    'exact_original_owner_source_pins_and_reviewed_registration_only_deltas': promotion['exact_original_owner_source_pins_and_reviewed_registration_only_deltas'],
    'parent_identity_changes': '198 independently replayed request identity occurrences across105 JSON files;115 certificate outputs and every other metadata field remain exact.',
    'initial_failures_preserved': 'Complete original graph failure101, failed initial metadata comparisons, original boundary-field owner failure101, and optional-case repair harness failure1 are retained with actual status/log/input hashes. None is counted as pass.',
    'second_review': 'wrapper-parent-identity-second-review.json',
    'unresolved': 'Ongoing scalar/codec semantic observations, any not-yet-promoted pattern consumers, final approved-freeze/publication closure and all original application proofs remain pending.',
    'historical_core_review_superseded_by': '../registration-order-checkpoint/receipt.json',
    'application_proof_ids_pending': 987,
    'w09_status': 'In progress', 'w10_status': 'Blocked',
    'practical_profile': 'inactive',
    'full_t01_gate': 'deferred_to_final_W10', 'full_t06_gate': 'deferred_to_W12',
    'changes': promotion['changes'],
}
(out / 'receipt.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
manifest = {str(p.relative_to(out)): h(p) for p in sorted(out.rglob('*'))
            if p.is_file() and p.name != 'file-manifest.json'}
(out / 'file-manifest.json').write_text(json.dumps({
    'schema': 'mpk.evidence_file_manifest.v1', 'files': manifest,
}, sort_keys=True, indent=2) + '\n')
destination = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/wrapper-consumer-checkpoint'
shutil.copytree(out, destination)
for path, sha in manifest.items():
    assert h(destination / path) == sha, path
print('finalized exact288 promoted wrapper files; evidence', len(manifest), 'immutable predecessors', len(before_manifest))
