import gzip
import hashlib
import json
import pathlib
import shutil

b = pathlib.Path(__file__).parent
out = b / 'pattern-checkpoint-evidence'
out.mkdir(exist_ok=False)
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

def copy_file(src, relative):
    assert src.is_file(), src
    dst = out / relative
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    assert h(src) == h(dst)

for folder, count in [
    ('control-pattern-capture-name-graphs', 7),
    ('control-pattern-capture-dual-checkers', 14),
    ('control-pattern-consumer-name-graphs', 21),
]:
    state = json.loads((b / folder / 'status.json').read_bytes())
    assert state['status'].startswith('passed_all_')
    assert len(state['stages']) == count
    assert all(row['exit_code'] == 0 for row in state['stages'])
    for src in sorted((b / folder).rglob('*')):
        if src.is_file() and src.suffix != '.mpcert':
            copy_file(src, pathlib.Path(folder) / src.relative_to(b / folder))

for folder in [
    'historical-pattern-condition-closure-review',
    'pattern-consumer-test-source-review',
    'pattern-consumer-selected-quality',
    'control-pattern-consumer-regeneration',
]:
    for src in sorted((b / folder).rglob('*')):
        if not src.is_file() or 'generated' in src.relative_to(b / folder).parts:
            continue
        copy_file(src, pathlib.Path(folder) / src.relative_to(b / folder))
copy_file(
    b / 'control-pattern-consumer-regeneration/generated/generation.json',
    'control-pattern-consumer-regeneration/generation.json',
)

names = [
    'capture-promotion.json',
    'control-pattern-capture-changed-certificates.json',
    'control-pattern-consumer-changed-certificates.json',
    'completed-ordinary-owner-stages-12-with-derived-pattern-capture-names.json',
    'completed-control-pattern-consumers-with-producer-names.json',
    'control-pattern-capture-name-rebinding-audit.json',
    'control-pattern-consumer-name-rebinding-audit.json',
    'context-name-rebindings-10.json',
    'context-name-rebindings-11.json',
    'derive-control-pattern-capture-name-rebindings.py',
    'derive-control-pattern-consumer-name-rebindings.py',
    'compare-completed-control-captures.py',
    'compare-completed-control-captures-2.py',
    'compare-completed-control-pattern-consumers.py',
    'compare-completed-control-pattern-consumers-2.py',
    'regenerate-control-pattern-consumers.rs',
    'control-pattern-consumer-compile-command.json',
    'control-pattern-consumer-compile.log.txt',
    'run-control-pattern-consumer-regeneration.py',
    'run-control-pattern-capture-name-graphs.py',
    'run-control-pattern-capture-dual-checkers.py',
    'run-control-pattern-consumer-name-graphs.py',
    'run-control-pattern-consumer-dual-checkers.py',
    'compare-historical-pattern-condition-closures.rs',
    'historical-pattern-condition-closure-compile-command-draft.json',
    'historical-pattern-condition-closure-compile-command.json',
    'historical-pattern-condition-closure-compile.log.txt',
    'run-historical-pattern-condition-closure-review.py',
    'run-pattern-consumer-selected-quality.py',
    'prepare-pattern-consumer-promotion.py',
    'prepare-pattern-checkpoint-evidence.py',
    'pattern-primitives-original-owner-equivalence.json',
    'pattern-routes-original-owner-equivalence.json',
    'unused-main-incremental-cache-cleanup-2.json',
]
for name in names:
    copy_file(b / name, name)

failures = []
for name in [
    'completed-ordinary-owner-stages-12-with-pattern-capture-names.json',
    'completed-control-pattern-consumers-with-capture-names.json',
]:
    raw = (b / name).read_bytes()
    data = json.loads(raw)
    assert data['failures'], name
    dst = out / (name + '.gz')
    dst.write_bytes(gzip.compress(raw, mtime=0))
    assert gzip.decompress(dst.read_bytes()) == raw
    failures.append({
        'original_path': name,
        'original_raw_sha256': hashlib.sha256(raw).hexdigest(),
        'archive': dst.name,
        'archive_sha256': h(dst),
        'original_bytes': len(raw),
        'failures': len(data['failures']),
        'real_status': data['status'],
        'lossless': True,
    })
(out / 'initial-metadata-comparison-failures.json').write_text(
    json.dumps(failures, sort_keys=True, indent=2) + '\n'
)
manifest = {
    str(src.relative_to(out)): h(src)
    for src in sorted(out.rglob('*')) if src.is_file()
}
(out / 'preparation-manifest.json').write_text(json.dumps({
    'status': 'completed_evidence_only_pending_consumer_duals_and_remaining_original_owners',
    'files': manifest,
    'application_proof_ids_pending': 987,
    'whole_gate': 'deferred_to_final_T01_W10_and_T06_W12',
}, sort_keys=True, indent=2) + '\n')
print('prepared completed evidence files', len(manifest))
