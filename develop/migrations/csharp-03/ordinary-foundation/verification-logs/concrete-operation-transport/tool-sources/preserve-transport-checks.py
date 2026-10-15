"""Independently audit every terminal same-byte operation transport check."""
from collections import Counter
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import shutil

repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
source = Path('/tmp/mpk-w09-concrete-operation-proofs/transport-checks')
base = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-operation-transport'
out = base / 'checks'
assert not out.exists()
dual = json.loads((source / 'verification.json').read_bytes())
assert dual['status'] == 'passed' and len(dual['stages']) == 182
assert Counter((s['backend'], s['kind']) for s in dual['stages']) == Counter({
    ('go', 'positive'): 45, ('rust', 'positive'): 45,
    ('go', 'hash'): 45, ('rust', 'hash'): 45,
    ('go', 'wrong'): 1, ('rust', 'wrong'): 1,
})
positive = {}
for row in dual['stages']:
    stem = row['case'] + '-' + row['kind']
    data = (source / (stem + '.mpcert')).read_bytes()
    report_bytes = (source / (stem + '-' + row['backend'] + '.json')).read_bytes()
    stderr = (source / (stem + '-' + row['backend'] + '.stderr')).read_bytes()
    assert sha256(data).hexdigest() == row['input_file_sha256']
    assert sha256(report_bytes).hexdigest() == row['report_sha256']
    assert sha256(stderr).hexdigest() == row['stderr_sha256']
    certificate_hash = sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest()
    assert certificate_hash == row['certificate_sha256']
    assert row['binary_sha256'] == dual['binaries'][row['backend']]
    expected_input = bytes.fromhex((base / 'transport-attempt-2/certificates' / (row['case'] + ('-wrong' if row['kind'] == 'wrong' else '') + '.hex')).read_text())
    if row['kind'] == 'hash':
        expected_input = expected_input[:-1] + bytes([expected_input[-1] ^ 1])
    assert data == expected_input
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
assert len(positive) == 45
for case, reports in positive.items():
    assert reports['go'] == reports['rust'], case
manifest = json.loads((base / 'source-manifest.json').read_bytes())
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
path_map = {}
for old in sorted(source.iterdir()):
    if not old.is_file():
        continue
    target = out / old.name
    if old.suffix in ('.log', '.stderr'):
        target = target.with_name(target.name + '.txt')
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(old, target)
    assert old.read_bytes() == target.read_bytes()
    path_map[old.name] = str(target.relative_to(out))
inventory = {str(p.relative_to(out)): sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.is_file()}
(out / 'receipt-audit.json').write_text(json.dumps(dict(status='passed', recorded_at=datetime.now(timezone.utc).isoformat(), terminal_stages=182, source_contexts=45, reports_agree=True, all_axiom_counts_zero=True, exact_inputs_reports_stderr_verified=True, current_source_and_fixture_hashes_match=True, raw_receipt_hashes=inventory, receipt_path_map=path_map, full_t_gate='deferred to T06-W12'), indent=2, sort_keys=True) + '\n')
(base / 'tool-sources/preserve-transport-checks.py').write_bytes(Path(__file__).read_bytes())
verification_path = base / 'verification.json'
verification = json.loads(verification_path.read_bytes())
verification.update(status='passed_local_targeted_and_same_byte_dual_checker_verification', dual_checker_verification='passed all 182 terminal positive/hash/wrong-proof stages', dual_checker_agreement=True, terminal_dual_checker_stages=182, dual_checker_receipt='checks/receipt-audit.json')
verification_path.write_text(json.dumps(verification, indent=2, sort_keys=True) + '\n')
review_path = repo / 'develop/migrations/csharp-03/ordinary-foundation/unit-7-concrete-operation-transport-review.md'
review = review_path.read_text()
before = '''Both fixed unchanged Go/Rust binaries are replaying the exact new certificate
bytes. The separate Money probe has already been accepted by the unchanged Go
kernel with zero axioms; it is not a substitute for the 45-context replay.
The full result remains pending until all 182 positive/hash/wrong-proof stages
terminate and their raw reports are independently audited.'''
after = '''Both fixed unchanged Go/Rust binaries accept all 45 exact new certificates
with matching module/declaration/axiom counts and all three hashes. All 182
stages are terminal: 90 positive backend executions, 90 hash-corruption
rejections and two wrong-normal-value core rejections. Every axiom summary is
zero. The independent audit verifies all exact inputs, fixed binary hashes,
raw reports and stderr, and confirms current source/fixture hashes still match.
The separate earlier Money probe is retained as historical evidence; the full
45-context replay is authoritative. See
`verification-logs/concrete-operation-transport/checks/receipt-audit.json`.'''
assert review.count(before) == 1
review_path.write_text(review.replace(before, after))
todo_path = repo / 'develop/docs/08_csharp_practical_subset_design-todo.md'
todo = todo_path.read_text()
before = '''743-source/173-fixture byte audit. Same-byte dual checking and Linux replay of
this new recipe remain pending terminal receipts. All 31 pending operations and'''
after = '''743-source/173-fixture byte audit. Both unchanged Go/Rust checkers accept all
45 certificates with zero axioms and matching hashes/counts; all 182 positive,
hash-corruption and wrong-normal-value stages are terminal and independently
audited. Linux replay of this new recipe remains pending its terminal receipt.
All 31 pending operations and'''
assert todo.count(before) == 1
todo_path.write_text(todo.replace(before, after))
print(json.dumps(dict(status='passed', stages=182, contexts=45, copied_files=len(inventory)), sort_keys=True))
