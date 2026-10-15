import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
out = b / 'pattern-consumer-checkpoint-evidence-2'
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())

owners = read(b / 'ordinary-regeneration-13/status.json')
assert owners['status'] == 'passed_all_four_affected_original_pattern_source_and_proof_type_owners'
assert len(owners['stages']) == 4
assert all(s['exit_code'] == 0 and s['test_counts'] == ['1'] for s in owners['stages'])
promotion = read(b / 'pattern-consumer-promotion.json')
assert promotion['status'].startswith('passed_promoted_54_')
capture = read(b / 'capture-promotion.json')
assert capture['status'].startswith('promoted_completed_capture_owner_')
from verified_server_result_provenance import load_result_provenance
duals, duals_folder = load_result_provenance(b, 'pattern')
shutil.copytree(b / 'pattern-checkpoint-evidence', out)

def copy_file(src, relative):
    assert src.is_file(), src
    dst = out / relative
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    assert h(src) == h(dst)

for folder in ['ordinary-regeneration-12', 'ordinary-regeneration-13', 'completed-pattern-result-provenance']:
    for src in sorted((b / folder).rglob('*')):
        if not src.is_file() or 'goldens' in src.relative_to(b / folder).parts:
            continue
        copy_file(src, pathlib.Path(folder) / src.relative_to(b / folder))

for name in [
    'ordinary-source-manifest-12.json', 'ordinary-source-manifest-13.json',
    'run-ordinary-regeneration-12.py', 'run-ordinary-regeneration-13.py',
    'pattern-consumer-promotion.json', 'apply-pattern-consumer-checkpoint-5.py', 'apply-pattern-consumer-checkpoint-4.py', 'verified_server_result_provenance.py', 'assemble-server-result-provenance.py', 'audit-server-remaining-results.py',
    'post-capture-relation-source-manifest-deltas.json',
    'published-capture-relation-review-append-audit.json', 'verify_registration_order_source_correspondence.py', 'registration-order-source-correspondence-guard-review.json', 'registration-order-fresh-environment-correspondence-review.json', 'wrapper-promotion-source-manifest-correspondence-review.json', 'wrapper-completed-230-checker-final-second-review.json',
    'finalize-pattern-checkpoint-evidence-6.py', 'finalize-pattern-checkpoint-evidence-5.py', 'finalize-pattern-checkpoint-evidence-4.py', 'pattern-finalizer-5-preparation-review.json', 'pattern-checkpoint-second-review.json',
    'pending-foundation-current-overlay-inventory-8.json',
    'audit-current-foundation-overlay-5.py', 'audit-current-foundation-overlay-6.py',
    'audit-current-foundation-overlay-7.py', 'audit-current-foundation-overlay-8.py', 'core-specification-correspondence-second-review.json', 'unused-refreeze-incremental-cache-cleanup-3.json',
    'full-semantic-finalizer-preparation-disk-failure.json',
    'audit-completed-checker-report-prefix.py', 'completed-checker-report-prefix-120-30-second-review.json', 'completed-checker-report-prefix-154-30-second-review.json', 'completed-checker-report-prefix-172-30-second-review.json', 'completed-checker-report-prefix-200-30-second-review.json', 'serial-w10-republication-plan.json',
    'current-foundation-overlay-5-failure.json', 'current-foundation-overlay-6-failure.json',
    'current-foundation-overlay-7-failure.json',
]:
    copy_file(b / name, name)

changes = promotion['changes']
assert len(changes) == 126 and len({x['path'] for x in changes}) == 126
# Capture36 was already published in fd8683e5 and is only prior evidence here.
prior_capture = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/capture-and-original-owner-checkpoint'
assert prior_capture.is_dir()
for row in capture['changes']:
    assert h(r / row['path']) == row['raw_sha256']
archive = r / 'develop/migrations/csharp-03/ordinary-foundation/previous-context-6-pattern-consumers'
archive.mkdir(exist_ok=False)
before_manifest = {}
for receipt, backup in [(promotion, b / 'pattern-consumer-before')]:
    for row in receipt['changes']:
        assert h(r / row['path']) == row['raw_sha256'], row['path']
        src = backup / row['path']
        assert h(src) == row['before_raw_sha256'], row['path']
        relative = pathlib.Path(row['path']).relative_to('develop/migrations/csharp-03/ordinary-foundation')
        dst = archive / relative
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)
        assert h(dst) == row['before_raw_sha256']
        before_manifest[str(relative)] = h(dst)
(archive / 'manifest.json').write_text(json.dumps({
    'status': 'immutable_complete_predecessor_bytes_retained',
    'source_checkpoint_commit': 'fd8683e54cebdb898cfc9769f3cf9f7dbde7c6c6',
    'files': before_manifest,
}, sort_keys=True, indent=2) + '\n')

receipt = {
    'schema': 'mpk.csharp_practical.t01_w09.pattern_consumer_checkpoint.v1',
    'status': 'passed_actual_generation_complete_lineage_full_graphs_original_owners_and_both_checkers',
    'selection_reason': 'Only the126 remaining primitive/route/packed pattern fixture files are promoted. Capture36 was already published by fd8683e5 and is not counted again. All actual producer JSON/control/hex bytes match the passed original eighteen-source owner outputs; all types, ordered original goals/premises/environments and value/mutation observations remain. Independently replayed ordered hash tuples account for name changes. Unpacked proof types execute their original owner but are not a newly published corpus. Unaffected scalar/native matrices are excluded. Whole gates remain final T01-W10/T06-W12.',
    'capture_previously_published_evidence': '../capture-and-original-owner-checkpoint/',
    'newly_promoted_fixture_files': 126,
    'capture_original_source_owner_cases': 18,
    'capture_complete_changed_graph_pairs': 7,
    'capture_fresh_matching_checker_acceptances': 14,
    'capture_exact_original_fresh_pair_reuses': 11,
    'current_consumer_actual_programs_generated': 54,
    'current_consumer_original_source_owners_passed': 4,
    'current_consumer_full_changed_graph_pairs': 21,
    'current_consumer_fresh_matching_checker_acceptances': 42,
    'historical_completed_checker_stages_before_server_only_instruction': 30,
    'new_server_checker_stages': 12,
    'checker_execution_provenance': 'completed-pattern-result-provenance/receipt.json; each actual stage keeps its source, host, input and original binary SHA',
    'current_consumer_exact_original_fresh_pair_reuses': 33,
    'source_goal_and_premise_checks': 'Unpacked113 paths/703 premises/108 refinements/664 projections/5 binder-limited paths; packed113 paths/703 complete premises/projections with all original field-bit observations. Source refinement, execution establishment and application proof assembly remain pending.',
    'historical_61_condition_complete_typed_closures_preserved': 18,
    'three_test_changes': 'SHA-pinned independently derived historical name mapping and two optional exact predecessor input directories; all other original test bytes remain unchanged.',
    'selected_vc_test_clippy_and_format': 'passed on exactly the current three test sources',
    'normal_fixture_pin_equivalence': promotion['normal_pin_equivalence'],
    'source_correspondence_review': promotion['source_correspondence_review'],
    'failed_initial_metadata_comparisons': 'Both failures retained losslessly with real failure counts and hashes; neither is counted as pass.',
    'accepted_parent_report_evidence': '../standalone-execution-checkpoint/',
    'wrapper_previously_published_evidence': '../wrapper-consumer-checkpoint/',
    'unresolved': 'Ongoing scalar/codec semantic observations, approved-freeze final closure/publication and all original application proofs remain pending.',
    'historical_core_review_superseded_by': '../registration-order-checkpoint/receipt.json',
    'application_proof_ids_pending': 987,
    'w09_status': 'In progress', 'w10_status': 'Blocked',
    'practical_profile': 'inactive',
    'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server',
    'relation_metrics_previously_published_evidence': '../relation-semantic-metric-checkpoint/',
    'changes': changes,
}
(out / 'receipt.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
manifest = {
    str(src.relative_to(out)): h(src)
    for src in sorted(out.rglob('*')) if src.is_file() and src.name != 'file-manifest.json'
}
(out / 'file-manifest.json').write_text(json.dumps({
    'schema': 'mpk.evidence_file_manifest.v1', 'files': manifest,
}, sort_keys=True, indent=2) + '\n')
destination = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/pattern-consumer-checkpoint'
shutil.copytree(out, destination)
for path, sha in manifest.items():
    assert h(destination / path) == sha, path
print('finalized exact126 newly promoted consumer files (capture36 previously published), complete evidence', len(manifest), 'immutable predecessors', len(before_manifest))
