"""Preserve terminal Linux evidence after all independent dual-checker stages passed."""
from hashlib import sha256
import json
from pathlib import Path
import shutil
import sys

commit = sys.argv[1]
assert len(commit) == 40 and all(c in '0123456789abcdef' for c in commit)
repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
temporary = Path('/tmp/mpk-w09-concrete-allocation-proofs')
launch = temporary / ('server-launch-' + commit[:8])
final = temporary / ('server-final-' + commit[:8])
base = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-allocation-proofs'
out = base / 'server-linux'
assert not out.exists()
audit = json.loads((final / 'final-audit.json').read_bytes())
assert audit['status'] == 'passed' and audit['source_commit'] == commit
assert audit['unique_tests_passed'] == 3 and audit['git_blob_files_verified'] == 917
assert audit['exported_files_verified'] == 92 and audit['exported_files_match_local']
assert audit['application_proofs_pending'] == 987 and audit['concrete_operations_pending'] == 25
assert audit['supplied_concrete_operation_proofs'] == 442
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
                   (base.parent / 'concrete-operation-transport/transport-attempt-2/certificates').iterdir() if p.is_file()}
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
                    linux_source_commit=commit, linux_git_blob_files_verified=917,
                    linux_harness_git_blobs_verified=3, linux_unique_tests_passed=3,
                    linux_exported_files_verified=92, linux_exported_files_match_local=True,
                    linux_receipt='server-linux/final/final-audit.json',
                    terminal_dual_checker_stages=184, dual_checker_agreement=True,
                    linux_legacy_exported_files_unchanged=91)
verification_path.write_text(json.dumps(verification, indent=2, sort_keys=True) + '\n')
review_path = repo / 'develop/migrations/csharp-03/ordinary-foundation/unit-7-concrete-allocation-proofs-review.md'
review = review_path.read_text()
before = """Requested Linux verification
remains pending an exact published-source terminal receipt."""
after = f"""The requested Linux server
passes all three targeted tests afresh, Clippy and both format checks at fixed
public source `{commit[:8]}`. Its clean detached checkout matches all 917
source/fixture Git blobs and all three harness blobs. The terminal audit verifies
all test log and test binary hashes, all 92 new-route outputs matching local
bytes and all 91 legacy outputs matching the prior published corpus exactly.
The fetched archive, SSH execution receipt and independent audit script are
retained in `verification-logs/concrete-allocation-proofs/server-linux/`."""
assert review.count(before) == 1
review_path.write_text(review.replace(before, after))
todo_path = repo / 'develop/docs/08_csharp_practical_subset_design-todo.md'
todo = todo_path.read_text()
before = """and both typed wrong-normal core rejections. Exact published-source Linux replay
remains pending. The 25 remaining operations, six internal type instances, seven"""
after = f"""and both typed wrong-normal core rejections. Fixed public source `{commit[:8]}`
also passes all three targeted tests, Clippy and both format checks on the
requested Linux server. Its terminal audit verifies all 917 source/fixture Git
blobs, 92 exact new-route exports and all 91 unchanged legacy exports.
The 25 remaining operations, six internal type instances, seven"""
assert todo.count(before) == 1
todo_path.write_text(todo.replace(before, after))
inventory = {str(p.relative_to(out)): sha256(p.read_bytes()).hexdigest()
             for p in sorted(out.rglob('*')) if p.is_file()}
(out / 'local-transfer-audit.json').write_text(json.dumps(dict(
    status='passed', source_commit=commit, copied_bytes_unchanged=True,
    source_and_fixture_hashes_still_match=True, raw_receipt_hashes=inventory,
    archive_sha256=execution['archive_sha256'], full_t_gate='deferred to T06-W12',
    application_proofs_pending=987, dual_checker_verification='passed all 184 audited stages (26 fresh, 158 exact retained)',
), indent=2, sort_keys=True) + '\n')
print(json.dumps(dict(status='passed', copied_files=len(inventory), source_commit=commit,
                     targeted_tests=3, git_blobs=917, exported_files_match_local=92), sort_keys=True))
