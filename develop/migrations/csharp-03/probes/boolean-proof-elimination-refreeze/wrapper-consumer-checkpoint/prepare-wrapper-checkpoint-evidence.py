import gzip
import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
out = b / 'wrapper-checkpoint-evidence'
out.mkdir(exist_ok=False)
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
graphs = read(b / 'wrapper-program-name-graphs-2/status.json')
assert graphs['status'].startswith('passed_all_115_') and len(graphs['stages']) == 115
assert all(x['exit_code'] == 0 for x in graphs['stages'])
comparison = read(b / 'wrapper-program-comparison-3.json')
assert not comparison['failures']

def copy_file(src, relative):
    assert src.is_file(), src
    dst = out / relative
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    assert h(src) == h(dst)

for folder in [
    'wrapper-program-name-graphs', 'wrapper-program-name-graphs-2',
    'wrapper-program-regeneration', 'wrapper-owner-validation',
    'wrapper-owner-boundary-field-failure', 'wrapper-parent-repair-first-harness-failure',
]:
    for src in sorted((b / folder).rglob('*')):
        if not src.is_file() or 'generated' in src.relative_to(b / folder).parts or src.suffix == '.mpcert':
            continue
        copy_file(src, pathlib.Path(folder) / src.relative_to(b / folder))
copy_file(b / 'wrapper-program-regeneration/generated/generation.json', 'wrapper-program-regeneration/generation.json')
copy_file(b / 'wrapper-program-promotion-plan-2/plan.json', 'wrapper-program-promotion-plan-2.json')
for name in [
    'wrapper-program-comparison-3.json', 'wrapper-program-compile-command.json',
    'wrapper-program-compile.log.txt', 'context-name-rebindings-5.json',
    'context-name-rebindings-6.json', 'json-field-name-rebinding-audit.json',
    'json-state-name-rebinding-audit.json', 'derive-json-field-name-rebindings.py',
    'derive-json-state-name-rebindings.py', 'prepare-wrapper-program-plan.py',
    'build-wrapper-program-driver.py', 'regenerate-wrapper-programs.rs',
    'run-wrapper-program-regeneration.py', 'compare-wrapper-program-regeneration.py',
    'compare-wrapper-program-regeneration-with-attachments.py',
    'compare-wrapper-program-regeneration-2.py', 'compare-wrapper-program-regeneration-3.py',
    'run-wrapper-program-name-graphs.py', 'run-wrapper-program-name-graphs-2.py',
    'run-wrapper-program-dual-checkers.py', 'prepare-wrapper-program-promotion.py',
    'repair-wrapper-parent-request-identities.py', 'wrapper-owner-validation-checkout.json',
    'run-wrapper-owner-validation.py', 'run-wrapper-owner-validation-2.py',
    'pending-current-owner-request-identity-inventory.json', 'prepare-wrapper-checkpoint-evidence.py',
]:
    copy_file(b / name, name)

archives = []
for name in ['wrapper-program-plan.json', 'wrapper-program-comparison.json', 'wrapper-program-comparison-2.json']:
    raw = (b / name).read_bytes()
    dst = out / (name + '.gz')
    dst.write_bytes(gzip.compress(raw, mtime=0))
    assert gzip.decompress(dst.read_bytes()) == raw
    data = json.loads(raw)
    archives.append({
        'original_file': name, 'original_raw_sha256': hashlib.sha256(raw).hexdigest(),
        'archive_sha256': h(dst), 'original_bytes': len(raw), 'lossless': True,
        'real_status': data.get('status', 'original_actual_source_generation_plan'),
        'failed_comparison_count': len(data.get('failures', [])),
    })
(out / 'large-original-input-and-comparison-archives.json').write_text(json.dumps(archives, sort_keys=True, indent=2) + '\n')
manifest = {str(p.relative_to(out)): h(p) for p in sorted(out.rglob('*')) if p.is_file()}
(out / 'preparation-manifest.json').write_text(json.dumps({
    'status': 'completed_graph_and_lineage_evidence_only_pending_duals_and_original_owner_retry',
    'files': manifest, 'application_proof_ids_pending': 987,
    'whole_gate': 'deferred_to_final_T01_W10_and_T06_W12',
}, sort_keys=True, indent=2) + '\n')
print('completed-only wrapper evidence files', len(manifest))
