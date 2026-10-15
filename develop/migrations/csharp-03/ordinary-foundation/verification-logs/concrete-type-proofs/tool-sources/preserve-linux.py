"""Preserve the terminal Linux receipt without changing tested source bytes."""
from hashlib import sha256
import json
from pathlib import Path
import shutil
import sys


commit = sys.argv[1]
assert len(commit) == 40 and all(c in '0123456789abcdef' for c in commit)
repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
temporary = Path('/tmp/mpk-w09-concrete-type-proofs')
launch = temporary / ('server-launch-' + commit[:8])
final = temporary / ('server-final-' + commit[:8])
base = repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-type-proofs'
out = base / 'server-linux'
assert not out.exists()
audit = json.loads((final / 'final-audit.json').read_bytes())
assert audit['status'] == 'passed' and audit['source_commit'] == commit
assert audit['unique_tests_passed'] == 5 and audit['git_blob_files_verified'] == 915
assert audit['exported_files_verified'] == 91 and audit['exported_files_match_local']
assert audit['application_proofs_pending'] == 987 and audit['internal_type_instances_pending'] == 6
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
verification.update(status='passed_local_and_requested_linux_targeted_verification',
                    linux_verification='passed targeted tests, lint, format and terminal audit',
                    linux_source_commit=commit, linux_git_blob_files_verified=915,
                    linux_harness_git_blobs_verified=3, linux_unique_tests_passed=5,
                    linux_exported_files_verified=91, linux_exported_files_match_local=True,
                    linux_receipt='server-linux/final/final-audit.json')
verification_path.write_text(json.dumps(verification, indent=2, sort_keys=True) + '\n')
review_path = repo / 'develop/migrations/csharp-03/ordinary-foundation/unit-7-concrete-type-proofs-review.md'
review = review_path.read_text()
before = '''The requested Linux replay is prepared for an exact public source commit in a
new detached checkout. It runs all five targeted tests afresh, Clippy and both
format checks, verifies source/fixture hashes before and after execution, and
requires all 91 exported metadata/positive/negative files to match the local
bytes. Its result remains pending until a terminal server receipt is fetched
and audited. See `verification-logs/concrete-type-proofs/verification.json`.'''
after = f'''The requested Linux replay passes all five targeted tests afresh, Clippy and
both format checks at fixed public source `{commit[:8]}`. Its detached checkout
matches all 915 source/fixture Git blobs and all three harness blobs. The terminal
audit verifies every test log and both test binaries, requires a clean checkout,
and confirms all 91 exported metadata/positive/negative files match the local
bytes. The exact fetched archive, SSH execution receipt and independent audit
script are retained separately. See
`verification-logs/concrete-type-proofs/server-linux/final/final-audit.json` and
`verification-logs/concrete-type-proofs/verification.json`.'''
assert review.count(before) == 1
review_path.write_text(review.replace(before, after))
todo_path = repo / 'develop/docs/08_csharp_practical_subset_design-todo.md'
todo = todo_path.read_text()
before = '''and all 987 application proof IDs remain explicitly pending. Exact public-source
Linux replay is prepared but its terminal receipt is still pending. This is a
partial unit 7 checkpoint; W09 remains In progress, W10-W12 remain Blocked and'''
after = f'''and all 987 application proof IDs remain explicitly pending. The fixed public
source `{commit[:8]}` also passes all five targeted tests, Clippy and both format
checks on the requested Linux server; its terminal audit verifies 915 source/
fixture Git blobs, all log/test-binary hashes and all 91 exact exported files.
This is a partial unit 7 checkpoint; W09 remains In progress, W10-W12 remain Blocked and'''
assert todo.count(before) == 1
todo_path.write_text(todo.replace(before, after))
inventory = {str(p.relative_to(out)): sha256(p.read_bytes()).hexdigest()
             for p in sorted(out.rglob('*')) if p.is_file()}
(out / 'local-transfer-audit.json').write_text(json.dumps(dict(
    status='passed', source_commit=commit, copied_bytes_unchanged=True,
    source_and_fixture_hashes_still_match=True, raw_receipt_hashes=inventory,
    archive_sha256=execution['archive_sha256'], full_t_gate='deferred to T06-W12',
    application_proofs_pending=987,
), indent=2, sort_keys=True) + '\n')
print(json.dumps(dict(status='passed', copied_files=len(inventory), source_commit=commit,
                     targeted_tests=5, git_blobs=915, exported_files_match_local=91), sort_keys=True))
