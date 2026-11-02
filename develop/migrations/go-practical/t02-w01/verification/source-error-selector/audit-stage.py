"""Check exact staged bytes/modes and preserve original libtest EOF records.

Run outside the sandbox; every Git invocation uses /usr/bin/git.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parent


def git(*args, data=None):
    return subprocess.check_output(['/usr/bin/git', *args], input=data)


def run(*args):
    command = ['/usr/bin/git', *args]
    r = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    return {'command': command, 'exit_code': r.returncode, 'stdout': r.stdout.decode(), 'stderr': r.stderr.decode()}


mode = sys.argv[1]
assert git('rev-parse', 'HEAD').decode().strip() == '1a0058ffe62998822209223f109276b5b6ef2f39'
if mode == 'whitespace':
    initial = run('diff', '--cached', '--check')
    assert initial['exit_code'] in {0, 2} and not initial['stderr']
    originals = []
    lines = initial['stdout'].splitlines()
    for row in lines:
        assert row.endswith(': new blank line at EOF.'), row
        name = row.rsplit(':', 2)[0]
        path = Path(name)
        assert path.name == 'stdout.log' and path.is_relative_to(ROOT.relative_to(Path.cwd()))
        result = json.loads(path.with_name('result.json').read_bytes())
        raw = path.read_bytes()
        assert hashlib.sha256(raw).hexdigest() == result['stdout']['sha256'] and len(raw) == result['stdout']['bytes']
        originals.append({'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
    names = [r['path'] for r in originals]
    other = run('diff', '--cached', '--check', '--', '.', *[':(top,literal,exclude)'+n for n in names])
    original_check = run('-c', 'core.whitespace=-blank-at-eof', 'diff', '--cached', '--check', '--', *names) if names else None
    assert other['exit_code'] == 0 and (original_check is None or original_check['exit_code'] == 0)
    result = {'status': 'passed_with_exact_immutable_libtest_stdout_eof_records', 'initial_default_check': initial, 'preserved_original_stdout': originals, 'default_check_all_other_files': other, 'raw_stdout_other_whitespace_check': original_check, 'persistent_git_or_scanner_configuration_weakened': False, 'scope': 'Only exact receipt-bound original libtest stdout EOF blank lines are retained. Every other staged file receives the default whitespace check. No persistent Git or repository scanner setting changes.'}
    (ROOT/'whitespace-audit.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status': result['status'], 'preserved_stdout_files': len(originals)}))
elif mode == 'bytes':
    names = set(git('diff', '--cached', '--name-only', '-z', '--diff-filter=ACMR').decode().split('\0')) - {''}
    rows = []
    for row in git('ls-files', '--stage', '-z', '--', *sorted(names)).split(b'\0'):
        if row:
            meta, name = row.split(b'\t', 1)
            permissions, oid, stage = meta.decode().split()
            assert stage == '0'
            rows.append((name.decode(), oid, permissions))
    assert {r[0] for r in rows} == names
    oids = sorted({r[1] for r in rows})
    batch = git('cat-file', '--batch', data=('\n'.join(oids)+'\n').encode())
    blobs, cursor = {}, 0
    for oid in oids:
        end = batch.index(b'\n', cursor)
        actual, kind, size = batch[cursor:end].decode().split()
        size = int(size)
        assert actual == oid and kind == 'blob'
        blobs[oid] = batch[end+1:end+1+size]
        cursor = end+size+2
    assert cursor == len(batch)
    files = []
    for name, oid, permissions in rows:
        path = Path(name)
        raw = path.read_bytes()
        assert raw == blobs[oid] and path.stat().st_mode & 0o777 == int(permissions, 8) & 0o777
        files.append({'path': name, 'staged_blob': oid, 'mode': permissions, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'literal_staged_bytes_equal_worktree': True})
    deleted = set(git('diff', '--cached', '--name-only', '-z', '--diff-filter=D').decode().split('\0')) - {''}
    assert not deleted
    report = {'status': 'all_staged_data_bytes_and_modes_exact', 'files': files, 'file_count': len(files), 'bytes': sum(r['bytes'] for r in files), 'authorized_deletion': sorted(deleted), 'report_added_after_data_audit': True, 'report_staged_bytes_checked_separately': True, 'source_originals_audited': 'input-review.json', 'actual_audit_driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    output = ROOT/'staged-preservation.json'
    output.write_text(json.dumps(report, indent=2)+'\n')
    git('add', '--', str(output.relative_to(Path.cwd())))
    assert git('show', ':'+str(output.relative_to(Path.cwd()))) == output.read_bytes()
    print(json.dumps({k: report[k] for k in ['status', 'file_count', 'bytes']}))
else:
    raise AssertionError(mode)
