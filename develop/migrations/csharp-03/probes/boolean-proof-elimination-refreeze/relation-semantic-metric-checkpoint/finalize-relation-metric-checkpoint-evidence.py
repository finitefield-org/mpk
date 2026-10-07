import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
r = pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
out = b / 'relation-metric-checkpoint-evidence'
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
receipt = json.loads((b / 'relation-metric-promotion.json').read_bytes())
assert receipt['status'].startswith('passed_promoted_exact_actual_21_')
assert len(receipt['changes']) == 1
row = receipt['changes'][0]
assert h(r / row['path']) == row['raw_sha256']
before = b / 'relation-metric-before' / row['path']
assert h(before) == row['before_raw_sha256']
archive = r / 'develop/migrations/csharp-03/ordinary-foundation/previous-context-6-relation-metrics'
archive.mkdir(exist_ok=False)
shutil.copy2(before, archive / 'metrics.json')
(archive / 'manifest.json').write_text(json.dumps({
    'status': 'immutable_complete_predecessor_metric_bytes_retained',
    'files': {'metrics.json': h(before)},
}, sort_keys=True, indent=2) + '\n')
for name in ['relation-metric-promotion.json', 'apply-relation-metric-checkpoint.py',
             'finalize-relation-metric-checkpoint-evidence.py']:
    shutil.copy2(b / name, out / name)
(out / 'receipt.json').write_text(json.dumps({
    'schema': 'mpk.csharp_practical.t01_w09.relation_semantic_metric_checkpoint.v1',
    'status': 'passed_promoted_exact_original_semantic_owner_metrics_and_matching_source_free_checkers',
    'selection_reason': receipt['selection_reason'],
    'original_source_cases': 21,
    'actual_owner_elapsed_seconds': receipt['actual_owner_elapsed_seconds'],
    'current_complete_certificate_bytes_retained': 21,
    'unchanged_complete_old_non_identity_observation_records': 20,
    'positive_constructor_original_carriers_retained': 2,
    'positive_constructor_actual_added_bool_carriers': 1,
    'positive_constructor_actual_observations': 24,
    'positive_constructor_previous_observations': 20,
    'positive_constructor_fresh_matching_checker_acceptances': 2,
    'original_failed_manifest_guard': 'ordinary-relations-metrics-regeneration/status.json',
    'normal_pin_equivalence': 'Every complete fresh semantic-owner metadata record and certificate byte set matches the current certificate-owner pins; the promoted metrics bytes equal the actual full original-owner output. No source/value/import/mutation observation is omitted or rerun solely for elapsed time.',
    'production_and_relation_owner_sources': 'relation-metric-promotion.json',
    'changes': receipt['changes'],
    'application_proof_ids_pending': 987,
    'w09_status': 'In progress', 'w10_status': 'Blocked',
    'practical_profile': 'inactive',
    'full_t01_gate': 'deferred_to_final_W10', 'full_t06_gate': 'deferred_to_W12',
}, sort_keys=True, indent=2) + '\n')
manifest = {str(p.relative_to(out)): h(p) for p in sorted(out.rglob('*'))
            if p.is_file() and p.name != 'file-manifest.json'}
(out / 'file-manifest.json').write_text(json.dumps({
    'schema': 'mpk.evidence_file_manifest.v1', 'files': manifest,
}, sort_keys=True, indent=2) + '\n')
dest = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/relation-semantic-metric-checkpoint'
shutil.copytree(out, dest)
assert all(h(dest / path) == sha for path, sha in manifest.items())
print('exact21case relation checkpoint completed; onefile promoted, predecessor and evidence verified')
