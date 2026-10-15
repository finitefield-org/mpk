from pathlib import Path
import hashlib
import json
import re
import shutil

b = Path(__file__).parent
f = Path('/private/tmp/mpk-w09-context-application-integration')
r = Path('/private/tmp/mpk-w09-refreeze-consumer-continuation')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
status_path = b / 'ordinary-regeneration-4b/status.json'
status_bytes = status_path.read_bytes()
status = json.loads(status_bytes)
assert len(status['stages']) >= 6
expected = ['integer-parsers', 'integer-formats', 'hex-codecs', 'calendar-codecs', 'decimal-parsers', 'decimal-formats']
stages = status['stages'][:6]
assert [s['stage'] for s in stages] == expected
assert all(s['exit_code'] == 0 and s['test_counts'] == ['1'] for s in stages)
source = read(b / 'ordinary-source-manifest-4b.json')['hashes']
for path, sha in source.items():
    assert h(f / path) == sha, path
production = {}
for path, sha in source.items():
    if path in ['Cargo.toml', 'Cargo.lock'] or (path.startswith('crates/') and '/src/' in path):
        assert h(r / path) == sha, path
        production[path] = sha
assert len(production) == 194
files = []
for stage in stages:
    log = b / 'ordinary-regeneration-4b' / (stage['stage'] + '.log.txt')
    assert h(log) == stage['log_sha256']
    assert re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text()) == ['1']
    for path, sha in stage['generated_files'].items():
        original = b / 'ordinary-regeneration-4b/goldens' / stage['destination'] / path
        current = r / 'develop/migrations/csharp-03/ordinary-foundation' / stage['destination'] / path
        assert h(original) == sha and original.read_bytes() == current.read_bytes(), (stage['stage'], path)
        files.append({'owner': stage['stage'], 'path': current.relative_to(r).as_posix(), 'raw_sha256': sha})
assert len(files) == 140
out = b / 'six-semantic-owner-checkpoint-evidence'
out.mkdir(exist_ok=False)
for name in ['ordinary-source-manifest-4b.json', 'run-ordinary-regeneration-4b.py',
             'decimal-formats-completed-owner-pin-review.json',
             'full-semantic-completed-prefix-pin-comparison.json',
             'verify_registration_order_source_correspondence.py',
             'registration-order-source-correspondence-guard-review.json',
             'registration-order-fresh-environment-correspondence-review.json',
             'finalize-six-semantic-owner-checkpoint.py',
             'run-final-w09-selected-validation.py', 'audit-final-actual-ordinary-identities.py',
             'final-boolean-elimination-proposal-reconciliation-plan.json',
             'final-boolean-elimination-proposal-candidate.md',
             'pattern-finalizer-5-preparation-review.json',
             'finalize-pattern-checkpoint-evidence-5.py']:
    shutil.copy2(b / name, out / name)
    assert h(b / name) == h(out / name)
(out / 'running-source-owner-status-snapshot.json').write_bytes(status_bytes)
for stage in stages:
    src = b / 'ordinary-regeneration-4b' / (stage['stage'] + '.log.txt')
    shutil.copy2(src, out / src.name)
    assert h(src) == h(out / src.name)
receipt = {
    'status': 'passed_exact_six_completed_original_scalar_codec_owners_remaining_four_not_claimed',
    'selection_reason': 'Retain the six actual completed source/value/import/mutation owners affected by the foundation refreeze. Each complete generated file is compared byte for byte with its current fixture. The remaining four owner results are deliberately not claimed by this checkpoint; no additional unaffected tests execute.',
    'original_owner_tests_passed': 6,
    'completed_stages': stages,
    'exact_current_output_files': files,
    'exact_current_output_file_count': len(files),
    'semantic_owner_owned_production_sources_exactly_equal': production,
    'source_provenance': 'The immutable original F source manifest and actual execution logs are retained. Current production ownership covers the exact 194 source paths; reviewed canonical-cases registration deltas are recorded separately and are not attributed to this older execution.',
    'decimal_formats_native_model_observations': 72,
    'remaining_original_owner_tests_not_claimed_passed': 4,
    'remaining_original_owner_tests': ['decimal-fixed-formats', 'structural-storage', 'scalar-domains', 'scalar-domain-ranges'],
    'newly_promoted_fixture_files': 0,
    'application_proof_ids_pending': 987,
    'w09_status': 'In progress', 'w10_status': 'Blocked', 'practical_profile': 'inactive',
    'full_t01_gate': 'deferred_to_final_W10', 'full_t06_gate': 'deferred_to_W12',
}
(out / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
manifest = {p.relative_to(out).as_posix(): h(p) for p in sorted(out.rglob('*')) if p.is_file()}
(out / 'file-manifest.json').write_text(json.dumps({'schema': 'mpk.evidence_file_manifest.v1', 'files': manifest}, sort_keys=True, indent=2) + '\n')
destination = r / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/six-semantic-owner-checkpoint'
shutil.copytree(out, destination)
for path, sha in manifest.items():
    assert h(destination / path) == sha, path
print('Retained six complete original owners and 140 exact current output hashes; remaining four owner results not claimed.')
