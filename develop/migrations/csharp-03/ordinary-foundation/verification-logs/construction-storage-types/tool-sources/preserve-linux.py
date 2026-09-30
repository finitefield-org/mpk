"""Preserve terminal Linux evidence after all independent dual-checker stages passed."""
from hashlib import sha256
import json
from pathlib import Path
import shutil
import sys

commit = sys.argv[1]
assert len(commit) == 40 and all(c in '0123456789abcdef' for c in commit)
repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
temporary = Path('/tmp/mpk-w09-construction-storage-types')
launch = temporary / ('server-launch-' + commit[:8])
final = temporary / ('server-final-' + commit[:8])
base = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/construction-storage-types'
out = base / 'server-linux'
assert not out.exists()
audit = json.loads((final / 'final-audit.json').read_bytes())
assert audit['status'] == 'passed' and audit['source_commit'] == commit
assert audit['unique_tests_passed'] == 4 and audit['git_blob_files_verified'] == 920
assert audit['exported_files_verified'] == 92 and audit['exported_files_match_local']
assert audit['application_proofs_pending'] == 987 and audit['concrete_type_instances_pending'] == 0 and audit['private_storage_domains'] == 6 and audit['source_ownership_pending'] is True
assert audit['supplied_concrete_type_proofs'] == 87
execution = json.loads((final / 'execution.json').read_bytes())
assert execution['exit_code'] == 0
assert sha256((final / 'receipt.tar.gz').read_bytes()).hexdigest() == execution['archive_sha256']
assert sha256((final / 'stderr.txt').read_bytes()).hexdigest() == execution['stderr_sha256']
assert sha256((final / 'remote-audit.py').read_bytes()).hexdigest() == execution['source_script_sha256']
state = json.loads((final / 'status.json').read_bytes())
for stage in state['stages']:
    assert stage['exit_code'] == 0
    assert sha256((final / (stage['stage'] + '.log')).read_bytes()).hexdigest() == stage['log_sha256']
expected = json.loads((base / 'expected-certificates.json').read_bytes())
assert {p.name: sha256(p.read_bytes()).hexdigest() for p in (final / 'certificates').iterdir()} == expected
assert audit['legacy_exported_files_verified'] == 91 and audit['legacy_certificate_bytes_unchanged']
legacy_expected = {p.name: sha256(p.read_bytes()).hexdigest() for p in
                   (base.parent / 'concrete-type-proofs/attempt-3/certificates').iterdir() if p.is_file()}
assert len(legacy_expected) == 91
assert {p.name: sha256(p.read_bytes()).hexdigest() for p in (final / 'legacy-certificates').iterdir()} == legacy_expected

manifest = json.loads((base / 'source-manifest.json').read_bytes())
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        assert sha256((repo / name).read_bytes()).hexdigest() == digest, name
path_map = {}
for folder, old_root in (('launch', launch), ('final', final)):
    for old in sorted(old_root.rglob('*')):
        if not old.is_file():
            continue
        relative = old.relative_to(old_root)
        target = out / folder / relative
        if old.suffix == '.log':
            target = target.with_name(target.name + '.txt')
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(old, target)
        assert old.read_bytes() == target.read_bytes()
        path_map[str(Path(folder) / relative)] = str(target.relative_to(out))
(out / 'receipt-path-map.json').write_text(json.dumps(path_map, indent=2, sort_keys=True) + '\n')
(base / 'tool-sources/preserve-linux.py').write_bytes(Path(__file__).read_bytes())
verification_path = base / 'verification.json'
verification = json.loads(verification_path.read_bytes())
dual = json.loads((base / 'dual-audit.json').read_bytes())
assert dual['status'] == 'passed' and dual['stage_count'] == 184
assert dual['new_stages'] == 26 and dual['retained_stages'] == 158
verification.update(status='passed_local_and_requested_linux_targeted_and_dual_checker_verification',
                    linux_verification='passed targeted tests, lint, format and terminal audit',
                    linux_source_commit=commit, linux_git_blob_files_verified=920,
                    linux_harness_git_blobs_verified=3, linux_unique_tests_passed=4,
                    linux_exported_files_verified=92, linux_exported_files_match_local=True,
                    linux_receipt='server-linux/final/final-audit.json',
                    terminal_dual_checker_stages=184, dual_checker_agreement=True,
                    linux_legacy_exported_files_unchanged=91)
verification_path.write_text(json.dumps(verification, indent=2, sort_keys=True) + '\n')
review_path=repo/'develop/migrations/csharp-03/ordinary-foundation/unit-7-construction-storage-types-review.md'
review=review_path.read_text()
before='Requested Linux verification remains pending an exact published-source terminal receipt.'
after=f'The requested Linux server passes all four targeted tests afresh, Clippy\nand both format checks at fixed public source `{commit[:8]}`. Its clean detached\ncheckout matches all 920 source/fixture Git blobs and all three harness blobs.\nThe terminal audit verifies every test log and binary hash, all 92 new-route\noutputs matching local bytes and all 91 unchanged legacy outputs. The fetched\narchive, SSH execution receipt and audit scripts are retained in\nverification-logs/construction-storage-types/server-linux/.'
assert review.count(before)==1
review_path.write_text(review.replace(before,after))
todo_path=repo/'develop/docs/08_csharp_practical_subset_design-todo.md'
todo=todo_path.read_text()
before='typed core rejections. Exact published-source Linux replay remains pending.'
after=f'typed core rejections. Fixed public source `{commit[:8]}` also passes all four\ntargeted tests, Clippy and both format checks on the requested Linux server. Its\nterminal audit verifies all 920 source/fixture Git blobs, all 92 exact new-route\nexports and all 91 unchanged legacy exports.'
assert todo.count(before)==1
todo_path.write_text(todo.replace(before,after))
inventory={str(p.relative_to(out)):sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
(out/'local-transfer-audit.json').write_text(json.dumps(dict(
 status='passed',source_commit=commit,copied_bytes_unchanged=True,
 source_and_fixture_hashes_still_match=True,raw_receipt_hashes=inventory,
 archive_sha256=execution['archive_sha256'],full_t_gate='deferred to T06-W12',
 application_proofs_pending=987,source_ownership_pending=True,
 dual_checker_verification='passed all 184 audited stages (26 fresh, 158 exact retained)',
),indent=2,sort_keys=True)+chr(10))
print(json.dumps(dict(status='passed',copied_files=len(inventory),source_commit=commit,
 targeted_tests=4,git_blobs=920,exported_files_match_local=92),sort_keys=True))
