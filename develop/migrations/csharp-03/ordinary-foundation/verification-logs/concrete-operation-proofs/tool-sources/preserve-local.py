"""Preserve exact operation-proof test receipts; dual-checker replay stays pending."""
from datetime import datetime, timezone
from hashlib import sha256
import ast
import json
from pathlib import Path
import shutil

repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
source = Path('/tmp/mpk-w09-concrete-operation-proofs')
out = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-operation-proofs'
assert not out.exists()
final = json.loads((source / 'attempt-3/status.json').read_bytes())
assert final['status'] == 'passed_targeted_tests_lint_format' and final['unique_tests_passed'] == 2
source_hashes = json.loads((source / 'attempt-3/source-manifest.json').read_bytes())
assert len(source_hashes) == 743
tracked = set((source / 'tracked-paths.bin').read_bytes().decode().split('\0')) - {''}
assert set(source_hashes) - tracked == {'crates/mpk-vc/src/csharp_practical_ordinary_concrete_operation_proofs.rs'}
for name, digest in source_hashes.items():
    assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
for attempt in ('attempt-1', 'attempt-2', 'attempt-3'):
    state = json.loads((source / attempt / 'status.json').read_bytes())
    for row in state['stages']:
        assert sha256((source / attempt / (row['stage'] + '.log')).read_bytes()).hexdigest() == row['log_sha256']
assert json.loads((source / 'attempt-1/status.json').read_bytes())['status'] == 'failed'
correction = json.loads((source / 'attempt-2/generation-correction.json').read_bytes())
assert correction['status'] == 'interrupted_for_proof_generation_correction'
assert sha256((source / 'attempt-2/core-check-sample.txt').read_bytes()).hexdigest() == correction['sample_sha256']
certificates = source / 'attempt-3/certificates'
metadata = [json.loads(p.read_bytes()) for p in certificates.glob('*.json')]
assert len(metadata) == 45
assert sum(len(m['proofs']) for m in metadata) == 436
assert sum(len(m['pending_operations']) for m in metadata) == 31
assert sum(len(m['pending_proof_ids']) for m in metadata) == 987
assert sum(not m['proofs'] for m in metadata) == 10
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
assert len(list(certificates.iterdir())) == 91
for p in certificates.glob('*.json'):
    m = json.loads(p.read_bytes())
    data = bytes.fromhex(p.with_suffix('.hex').read_text())
    assert sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest() == m['certificate_sha256']
prior = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-type-proofs/source-manifest.json'
fixture_paths = {name for name in json.loads(prior.read_bytes())['fixture_hashes'] if '/concrete-types/' not in name}
fixture_paths.update(str(p.relative_to(repo)) for p in (repo / 'develop/migrations/csharp-03/ordinary-foundation/concrete-operations').iterdir() if p.suffix == '.hex' or p.name == 'certificates.json')
assert fixture_paths.issubset(tracked)
fixture_hashes = {name: sha256((repo / name).read_bytes()).hexdigest() for name in sorted(fixture_paths)}
path_map = {}
for folder in ('attempt-1', 'attempt-2', 'attempt-3'):
    for old in sorted((source / folder).rglob('*')):
        if not old.is_file():
            continue
        relative = old.relative_to(source)
        target = out / relative
        if old.suffix in ('.log', '.stderr'):
            target = target.with_name(target.name + '.txt')
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(old, target)
        assert old.read_bytes() == target.read_bytes()
        path_map[str(relative)] = str(target.relative_to(out))
tools = out / 'tool-sources'
tools.mkdir()
for name in ('run.py', 'check.py', 'run-targeted.py', 'launch-linux.py', 'launch-wrapper.py', 'final-linux-fetch.py', 'preserve-local.py'):
    ast.parse((source / name).read_text())
    shutil.copyfile(source / name, tools / name)

def write(name, value):
    (out / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

write('source-manifest.json', dict(source_hashes=source_hashes, fixture_hashes=fixture_hashes))
write('expected-certificates.json', {p.name: sha256(p.read_bytes()).hexdigest() for p in certificates.iterdir()})
write('receipt-path-map.json', path_map)
inventory = {str(p.relative_to(out)): sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
write('local-receipt-audit.json', dict(status='passed_local_targeted_tests_audit', recorded_at=datetime.now(timezone.utc).isoformat(), current_source_hashes_verified=len(source_hashes), fixture_hashes_verified=len(fixture_hashes), raw_receipt_hashes=inventory, copied_bytes_unchanged=True, unique_targeted_tests_passed=2, source_contexts=45, supplied_concrete_operation_proofs=436, concrete_operations_pending=31, original_application_proof_ids_retained=987, full_t_gate='deferred to T06-W12'))
write('verification.json', dict(status='passed_local_targeted_tests_lint_format', source_contexts=45, supplied_concrete_operation_proofs=436, contexts_with_empty_operation_group=10, concrete_operations_pending=31, application_proofs_pending=987, complete_application_assembly_pending=True, unique_targeted_tests_passed=2, local_rust_kernel_all_certificates_accepted=True, total_axiom_count=0, wrong_normal_value_core_rejection=True, wrong_value_definition_only_accepted=True, selection_reason=final['selection_reason'], dual_checker_verification='pending terminal replay of identical original-source certificate bytes', linux_verification='pending fixed public source commit', review='../../unit-7-concrete-operation-proofs-review.md', full_t_gate='deferred to T06-W12'))
print(json.dumps(dict(receipt_files=len(inventory), source_files=len(source_hashes), fixture_files=len(fixture_hashes), supplied_proofs=436, local_tests=2), sort_keys=True))
