"""Preserve exact transport-proof receipts and source promotion evidence."""
from datetime import datetime, timezone
from hashlib import sha256
import ast
import json
from pathlib import Path
import shutil

repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
source = Path('/tmp/mpk-w09-concrete-operation-proofs')
out = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-operation-transport'
assert not out.exists()
final = json.loads((source / 'transport-attempt-2/status.json').read_bytes())
assert final['status'] == 'passed_targeted_tests_lint_format' and final['unique_tests_passed'] == 2
source_hashes = json.loads((source / 'transport-attempt-2/source-manifest.json').read_bytes())
assert len(source_hashes) == 743
for name, digest in source_hashes.items():
    assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
for attempt in ('transport-attempt-1', 'transport-attempt-2'):
    state = json.loads((source / attempt / 'status.json').read_bytes())
    for row in state['stages']:
        assert sha256((source / attempt / (row['stage'] + '.log')).read_bytes()).hexdigest() == row['log_sha256']
assert json.loads((source / 'transport-attempt-1/status.json').read_bytes())['status'] == 'failed'
retained = final['retained_stages'][0]
assert retained['stage'] == 'original-operation-pins' and retained['exit_code'] == 0
assert sha256((source / 'transport-attempt-1/original-operation-pins.log').read_bytes()).hexdigest() == retained['log_sha256']
certificates = source / 'transport-attempt-2/certificates'
metadata = [json.loads(p.read_bytes()) for p in certificates.glob('*.json')]
assert len(metadata) == 45
assert sum(len(m['proofs']) for m in metadata) == 436
assert sum(len(m['pending_operations']) for m in metadata) == 31
assert sum(len(m['pending_proof_ids']) for m in metadata) == 987
assert sum(not m['proofs'] for m in metadata) == 10
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
assert len(list(certificates.iterdir())) == 91
for p in certificates.glob('*.json'):
    current = json.loads(p.read_bytes())
    old = json.loads((source / 'attempt-3/certificates' / p.name).read_bytes())
    data = bytes.fromhex(p.with_suffix('.hex').read_text())
    assert sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest() == current['certificate_sha256']
    for m in (old, current):
        m.pop('certificate_sha256')
    assert current == old, p.name
    assert p.read_bytes() == (source / 'transport-attempt-1/certificates' / p.name).read_bytes()
    assert p.with_suffix('.hex').read_bytes() == (source / 'transport-attempt-1/certificates' / p.with_suffix('.hex').name).read_bytes()
old_base = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-operation-proofs'
fixture_hashes = json.loads((old_base / 'source-manifest.json').read_bytes())['fixture_hashes']
for name, digest in fixture_hashes.items():
    assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
probe = source / 'transport-money-go-probe'
receipt = json.loads((probe / 'receipt.json').read_bytes())
assert receipt['status'] == 'accepted_by_unchanged_go_kernel'
assert sha256((probe / 'money.mpcert').read_bytes()).hexdigest() == receipt['input_sha256']
assert sha256((probe / 'report.json').read_bytes()).hexdigest() == receipt['report_sha256']
report = json.loads((probe / 'report.json').read_bytes())
assert report['verdict'] == 'accepted' and report['report']['AxiomCount'] == 0
assert (probe / 'money.mpcert').read_bytes() == bytes.fromhex((certificates / 'binding-vc-money.hex').read_text())
path_map = {}
for folder in ('transport-attempt-1', 'transport-attempt-2', 'transport-money-go-probe'):
    for old in sorted((source / folder).rglob('*')):
        if not old.is_file():
            continue
        target = out / old.relative_to(source)
        if old.suffix in ('.log', '.stderr'):
            target = target.with_name(target.name + '.txt')
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(old, target)
        assert old.read_bytes() == target.read_bytes()
        path_map[str(old.relative_to(source))] = str(target.relative_to(out))
shutil.copyfile(source / 'transport-promotion.json', out / 'promotion.json')
tools = out / 'tool-sources'
tools.mkdir()
for name in ('run-transport.py', 'check-transport.py', 'preserve-transport-local.py'):
    ast.parse((source / name).read_text())
    shutil.copyfile(source / name, tools / name)
for name in ('run-targeted.py', 'launch-linux.py', 'launch-wrapper.py', 'final-linux-fetch.py'):
    text = (old_base / 'tool-sources' / name).read_text().replace('concrete-operation-proofs', 'concrete-operation-transport')
    ast.parse(text)
    (tools / name).write_text(text)

def write(name, value):
    (out / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

write('source-manifest.json', dict(source_hashes=source_hashes, fixture_hashes=fixture_hashes))
write('expected-certificates.json', {p.name: sha256(p.read_bytes()).hexdigest() for p in certificates.iterdir()})
write('receipt-path-map.json', path_map)
inventory = {str(p.relative_to(out)): sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
write('local-receipt-audit.json', dict(status='passed_local_targeted_tests_audit', recorded_at=datetime.now(timezone.utc).isoformat(), current_source_hashes_verified=len(source_hashes), fixture_hashes_verified=len(fixture_hashes), raw_receipt_hashes=inventory, copied_bytes_unchanged=True, unique_targeted_tests_passed=2, source_contexts=45, supplied_concrete_operation_proofs=436, concrete_operations_pending=31, original_application_proof_ids_retained=987, original_metadata_same_except_certificate_hash=True, outputs_unchanged_after_lint_correction=91, retained_stages=final['retained_stages'], retention_reason=final['retention_reason'], full_t_gate='deferred to T06-W12'))
write('verification.json', dict(status='passed_local_targeted_tests_lint_format', source_contexts=45, supplied_concrete_operation_proofs=436, contexts_with_empty_operation_group=10, concrete_operations_pending=31, application_proofs_pending=987, complete_application_assembly_pending=True, unique_targeted_tests_passed=2, local_rust_kernel_all_certificates_accepted=True, total_axiom_count=0, wrong_normal_value_core_rejection=True, wrong_value_definition_only_accepted=True, original_metadata_same_except_certificate_hash=True, original_operation_pins_unchanged=45, selection_reason=final['selection_reason'], retained_stages=final['retained_stages'], retention_reason=final['retention_reason'], dual_checker_verification='pending terminal replay of identical original-source certificate bytes', linux_verification='pending fixed public source commit', review='../../unit-7-concrete-operation-transport-review.md', full_t_gate='deferred to T06-W12'))
print(json.dumps(dict(receipt_files=len(inventory), source_files=len(source_hashes), fixture_files=len(fixture_hashes), supplied_proofs=436, local_tests=2), sort_keys=True))
