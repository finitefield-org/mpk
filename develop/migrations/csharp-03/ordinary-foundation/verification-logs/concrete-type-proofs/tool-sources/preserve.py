"""Preserve exact local receipts and prepare an isolated Linux source replay."""
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import shutil


repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
source = Path('/tmp/mpk-w09-concrete-type-proofs')
out = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-type-proofs'
assert not out.exists()
tracked = set(source.joinpath('tracked-paths.bin').read_bytes().decode().split('\0')) - {''}
final = json.loads((source / 'attempt-3/status.json').read_bytes())
assert final['status'] == 'passed_targeted_tests_lint_format'
assert final['unique_tests_passed'] == 5
source_hashes = json.loads((source / 'attempt-3/source-manifest.json').read_bytes())
assert set(source_hashes).issubset(tracked)
for name, digest in source_hashes.items():
    assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
for attempt in ('attempt-1', 'attempt-2', 'attempt-3'):
    state = json.loads((source / attempt / 'status.json').read_bytes())
    for row in state['stages']:
        assert sha256((source / attempt / (row['stage'] + '.log')).read_bytes()).hexdigest() == row['log_sha256']
retained = final['retained_stages'][0]
assert retained['stage'] == 'pattern-candidate-units' and retained['exit_code'] == 0
assert sha256((source / 'attempt-1/pattern-candidate-units.log').read_bytes()).hexdigest() == retained['log_sha256']
assert set(final['retained_source_delta']) == {
    'crates/mpk-vc/src/csharp_practical_ordinary_concrete_types.rs',
    'crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_type_tests.rs',
}

dual = json.loads((source / 'checks/verification.json').read_bytes())
assert dual['status'] == 'passed' and len(dual['stages']) == 182
assert Counter((s['backend'], s['kind']) for s in dual['stages']) == Counter({
    ('go', 'positive'): 45, ('rust', 'positive'): 45,
    ('go', 'hash'): 45, ('rust', 'hash'): 45,
    ('go', 'wrong'): 1, ('rust', 'wrong'): 1,
})
positive = {}
for row in dual['stages']:
    stem = row['case'] + '-' + row['kind']
    data = (source / 'checks' / (stem + '.mpcert')).read_bytes()
    report_bytes = (source / 'checks' / (stem + '-' + row['backend'] + '.json')).read_bytes()
    stderr = (source / 'checks' / (stem + '-' + row['backend'] + '.stderr')).read_bytes()
    assert sha256(data).hexdigest() == row['input_file_sha256']
    assert sha256(report_bytes).hexdigest() == row['report_sha256']
    assert sha256(stderr).hexdigest() == row['stderr_sha256']
    certificate_hash = sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest()
    assert certificate_hash == row['certificate_sha256']
    assert row['binary_sha256'] == dual['binaries'][row['backend']]
    report = json.loads(report_bytes)
    good = row['kind'] == 'positive'
    assert row['exit_code'] == (0 if good else 1)
    assert report['verdict'] == ('accepted' if good else 'rejected')
    if good:
        if row['backend'] == 'go':
            core = report['report']
            hashes = {k: bytes(core[f]).hex() for k, f in (
                ('export', 'ExportHash'), ('axiom_report', 'AxiomReportHash'), ('certificate', 'CertificateHash'))}
            counts = (core['Module'], core['DeclarationCount'], core['AxiomCount'])
            assert all(v == 0 for v in core['AxiomReport']['Summary'].values())
        else:
            hashes = report['hashes']
            counts = (report['module'], report['declaration_count'], report['axiom_count'])
            assert all(v == 0 for v in report['axiom_report']['summary'].values())
        assert counts[2] == 0 and hashes['certificate'] == certificate_hash
        positive.setdefault(row['case'], {})[row['backend']] = (hashes, counts)
    else:
        assert (report['certificate'] if row['backend'] == 'go' else report['hashes']['certificate']) == certificate_hash
        if row['kind'] == 'wrong':
            assert (report.get('error_kind') == 'core_check' if row['backend'] == 'go' else report.get('error_code') == 'KERNEL_CORE_CHECK')
for case, reports in positive.items():
    assert reports['go'] == reports['rust'], case

certificates = source / 'attempt-3/certificates'
metadata = [json.loads(p.read_bytes()) for p in sorted(certificates.glob('*.json'))]
assert len(metadata) == 45
assert sum(len(m['proofs']) for m in metadata) == 81
assert sum(not m['proofs'] for m in metadata) == 10
assert sum(len(m['pending_type_instances']) for m in metadata) == 6
assert sum(len(m['pending_proof_ids']) for m in metadata) == 987
assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
assert len(list(certificates.glob('*.hex'))) == 46

fixture_paths = set()
for name in tracked:
    p = Path(name)
    if name.startswith('develop/migrations/csharp-03/') and 'verification-logs' not in p.parts:
        if p.suffix == '.json' and (any(part.endswith('-sources') for part in p.parts) or p.parent.name == 'data-phase'):
            fixture_paths.add(name)
        if 'ordinary-foundation/concrete-types/' in name and (p.suffix == '.hex' or p.name == 'certificates.json'):
            fixture_paths.add(name)
    if any(name.startswith(prefix) for prefix in ('develop/specs/', 'proofs/std/', 'fixtures/csharp/')) and p.suffix in ('.json', '.hex', '.cs', '.toml'):
        fixture_paths.add(name)
for family in ('binding-vc', 'construction-vc', 'exception-vc'):
    for filename in ('requests.json', 'responses.json'):
        fixture_paths.add(f'develop/migrations/csharp-03/{family}/{filename}')
fixture_paths.update((
    'develop/migrations/csharp-03/foundation/foundation-descriptor.json',
    'develop/migrations/csharp-03/foundation/foundation-definitions.json',
    'develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md',
))
assert fixture_paths.issubset(tracked)
fixture_hashes = {name: sha256((repo / name).read_bytes()).hexdigest() for name in sorted(fixture_paths)}

renames = {}
for folder in ('attempt-1', 'attempt-2', 'attempt-3', 'checks'):
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
        renames[str(relative)] = str(target.relative_to(out))

tools = out / 'tool-sources'
tools.mkdir()
for name in ('run.py', 'check.py', 'run-targeted.py', 'preserve.py'):
    shutil.copyfile(source / name, tools / name)
prior = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/with-context-refinement-candidates/tool-sources/launch-linux.py'
launch = prior.read_text().replace('mpk-w09-context-refinement-candidates-', 'mpk-w09-concrete-type-proofs-')
launch = launch.replace('verification-logs/control-predicates/with-context-refinement-candidates', 'verification-logs/concrete-type-proofs')
(tools / 'launch-linux.py').write_text(launch)

def write_json(name, value):
    (out / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

write_json('source-manifest.json', dict(source_hashes=source_hashes, fixture_hashes=fixture_hashes))
write_json('expected-certificates.json', {
    p.name: sha256(p.read_bytes()).hexdigest() for p in sorted(certificates.iterdir()) if p.is_file()
})
write_json('receipt-path-map.json', renames)
inventory = {
    str(p.relative_to(out)): sha256(p.read_bytes()).hexdigest()
    for p in sorted(out.rglob('*')) if p.is_file()
}
write_json('local-receipt-audit.json', dict(
    status='passed', recorded_at=datetime.now(timezone.utc).isoformat(),
    current_source_hashes_verified=len(source_hashes), fixture_hashes=len(fixture_hashes),
    fixture_bytes=sum((repo / name).stat().st_size for name in fixture_hashes),
    raw_receipt_hashes=inventory, exact_report_input_log_hashes_verified=True,
    dual_checker_reports_agree=True, all_axiom_counts_zero=True,
    terminal_dual_checker_stages=182, original_contexts=45, supplied_public_type_proofs=81,
    contexts_with_empty_type_group=10, internal_type_instances_pending=6,
    original_application_proof_ids_retained=987, unique_targeted_tests_passed=5,
    retained_stages=final['retained_stages'], retained_source_delta=final['retained_source_delta'],
    retention_reason=final['retention_reason'], full_t_gate='deferred to T06-W12',
))
write_json('verification.json', dict(
    status='passed_local_targeted_verification', source_contexts=45,
    supplied_public_type_proofs=81, contexts_with_empty_type_group=10,
    internal_type_instances_pending=6, application_proofs_pending=987,
    complete_application_assembly_pending=True, unique_targeted_tests_passed=5,
    terminal_dual_checker_stages=182, dual_checker_agreement=True,
    total_axiom_count=0, wrong_proof_core_rejection=True,
    selection_reason=final['selection_reason'], retained_source_delta=final['retained_source_delta'],
    retention_reason=final['retention_reason'], full_t_gate='deferred to T06-W12',
    linux_verification='pending fixed public source commit',
    review='../../unit-7-concrete-type-proofs-review.md',
))
print(json.dumps(dict(receipt_files=len(inventory), source_files=len(source_hashes),
                     fixture_files=len(fixture_hashes), supplied_proofs=81, dual_stages=182), sort_keys=True))
