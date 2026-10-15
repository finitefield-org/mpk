from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import sys

repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
root = Path('/private/tmp/mpk-w09-foundation-proofs')
reports = root / 'local'
state = json.loads((reports / 'status.json').read_bytes())
assert state['status'] == 'passed_targeted_tests_lint_format'
reused = state.get('reused_unchanged_component_receipt')
if reused:
    assert state['unique_tests_passed'] == 1
    assert [s['stage'] for s in state['stages']] == ['foundation-proofs', 'clippy', 'format', 'support-format']
    prior = Path(reused['path'])
    previous = json.loads((prior/'status.json').read_bytes())
    assert sha256((prior/'status.json').read_bytes()).hexdigest() == reused['status_sha256']
    assert sha256((prior/'source-manifest.json').read_bytes()).hexdigest() == reused['source_manifest_sha256']
    assert previous['status'] == 'passed_targeted_tests_lint_format' and previous['unique_tests_passed'] == 3
    previous_audit = json.loads((prior/'final-audit.json').read_bytes())
    assert previous_audit['status'] == 'passed' and previous_audit['terminal_exec_session'] == 29072
    for binary,digest in previous['test_binaries'].items():
        assert sha256((prior/'test-binaries'/Path(binary).name).read_bytes()).hexdigest() == digest
    previous_manifest = json.loads((prior/'source-manifest.json').read_bytes())
    current_manifest = json.loads((root/'source-manifest.json').read_bytes())
    changed = [name for name,digest in current_manifest['source_hashes'].items() if previous_manifest['source_hashes'][name] != digest]
    assert set(changed) == set(reused['changed_files']) == {'crates/mpk-vc/src/csharp_practical_ordinary_foundation_proofs.rs', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_foundation_proof_tests.rs'}
else:
    assert state['unique_tests_passed'] == 3
    assert [s['stage'] for s in state['stages']] == ['foundation-proofs', 'original-allocation-proofs', 'original-storage-type-proofs', 'clippy', 'format', 'support-format']
for stage in state['stages']:
    assert stage['exit_code'] == 0
    assert sha256((reports / (stage['stage'] + '.log')).read_bytes()).hexdigest() == stage['log_sha256']
for name, digest in state['test_binaries'].items():
    assert sha256(Path(name).read_bytes()).hexdigest() == digest
assert len(state['test_binaries']) == 1
manifest = json.loads((reports / 'source-manifest.json').read_bytes())
assert (reports / 'source-manifest.json').read_bytes() == (root / 'source-manifest.json').read_bytes()
assert (len(manifest['source_hashes']), len(manifest['fixture_hashes'])) == (751, 173)
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
old = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs'
assert manifest['fixture_hashes'] == json.loads((old / 'scoped-construction-proofs/source-manifest.json').read_bytes())['fixture_hashes']
def catalog(p):
    return {x.name: sha256(x.read_bytes()).hexdigest() for x in p.iterdir() if x.is_file()}
exports = catalog(reports / 'certificates')
assert len(exports) == 91 and exports == json.loads((reports / 'expected-certificates.json').read_bytes())
if reused: assert exports == catalog(prior/'certificates')
assert {'certificates/' + name: digest for name, digest in exports.items()} == state['exported_certificate_hashes']
for folder, golden in [('type-certificates', 'construction-storage-types/attempt-2/certificates'), ('operation-certificates', 'concrete-allocation-proofs/attempt-5/certificates')]:
    actual = catalog(reports / folder)
    assert len(actual) == 92 and actual == catalog(old / golden)
    assert actual == state[folder.replace('-', '_') + '_hashes']
contexts = []
for path in sorted((reports / 'certificates').glob('*.json')):
    m = json.loads(path.read_bytes())
    types, operations = m['types'], m['operations']
    for p in (types, operations):
        assert p['source_ir_sha256'] == m['source_ir_sha256']
        assert p['foundation_sha256'] == m['foundation_sha256']
        assert p['binding_vc_sha256'] == m['binding_vc_sha256']
        assert p['pending_proof_ids'] == m['pending_proof_ids']
        assert p['proof_check_pending'] and p['application_scope_pending']
    assert m['proof_check_pending'] and m['application_scope_pending']
    assert not types['pending_type_instances']
    assert all(p['sequent']['kind'] == 'concrete_type_equivalence' for p in types['proofs'])
    assert all(p['sequent']['kind'] == 'concrete_definition_equivalence' for p in operations['proofs'])
    supplied = {p['sequent']['id'] for p in types['proofs'] + operations['proofs']}
    assert len(supplied) == len(types['proofs']) + len(operations['proofs'])
    assert m['supplied_binding_sequent_ids'] == [p for p in m['pending_proof_ids'] if p in supplied]
    assert m['remaining_binding_sequent_ids'] == [p for p in m['pending_proof_ids'] if p not in supplied]
    assert len(set(m['pending_proof_ids'])) == len(m['pending_proof_ids'])
    assert all(d['private_storage_only'] and d['ownership_pending'] for d in types.get('construction_storage_domains', []))
    original_types = json.loads((reports / 'type-certificates' / path.name).read_bytes())
    assert [p['sequent'] for p in types['proofs']] == [p['sequent'] for p in original_types['proofs']]
    original_operations = (reports / 'operation-certificates' / path.name).read_bytes()
    assert operations == json.loads(original_operations)
    assert m['original_operation_proofs_sha256'] == sha256(original_operations).hexdigest()
    assert m['original_operation_certificate_sha256'] == operations['certificate_sha256']
    certificate = bytes.fromhex(path.with_suffix('.hex').read_text())
    assert sha256(b'MPK-MODULE-CERT-0.1\0' + certificate).hexdigest() == types['certificate_sha256']
    contexts.append(dict(id=path.stem, type_proofs=len(types['proofs']), operation_proofs=len(operations['proofs']), supplied_binding_sequents=len(supplied), remaining_binding_sequents=len(m['remaining_binding_sequent_ids']), original_application_proofs_pending=len(m['pending_proof_ids']), generic_operations_pending=len(operations['pending_operations']), certificate_sha256=types['certificate_sha256']))
assert len(contexts) == 45
counts = {k: sum(c[k] for c in contexts) for k in ('type_proofs', 'operation_proofs', 'supplied_binding_sequents', 'remaining_binding_sequents', 'original_application_proofs_pending', 'generic_operations_pending')}
assert list(counts.values()) == [87, 442, 529, 458, 987, 25]
audit = dict(status='passed', recorded_at=datetime.now(timezone.utc).isoformat(), terminal_exec_session=int(sys.argv[1]), source_files_verified=751, fixture_files_verified=173, manifest_sha256=sha256((root/'source-manifest.json').read_bytes()).hexdigest(), unique_tests_passed=state['unique_tests_passed'], unique_unchanged_component_tests_reused=2 if reused else 0, test_log_hashes_verified=True, test_binary_hashes_verified=True, source_contexts=45, contexts=contexts, counts=counts, exported_files_verified=91, legacy_type_exported_files_verified=92, legacy_operation_exported_files_verified=92, legacy_certificate_bytes_unchanged=True, application_scope_pending=True, complete_application_assembly_pending=True, full_t_gate='deferred to T06-W12')
(reports/'final-audit.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
print(json.dumps({k:v for k,v in audit.items() if k != 'contexts'},sort_keys=True))
