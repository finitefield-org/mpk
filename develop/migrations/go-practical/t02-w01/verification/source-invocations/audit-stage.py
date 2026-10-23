"""Check exact staged bytes/modes and preserve original terminal EOF records.

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
assert git('rev-parse', 'HEAD').decode().strip() == '75cc4973b1411337707b3c096215648309e827c8'
if mode == 'whitespace':
    initial = run('diff', '--cached', '--check')
    assert initial['exit_code'] in {0, 2} and not initial['stderr']
    originals = []
    lines = initial['stdout'].splitlines()
    diagnostics = []
    trailing_names = []
    at = 0
    while at < len(lines):
        row = lines[at]
        at += 1
        if row.endswith(': trailing whitespace.'):
            name, number, _ = row.rsplit(':', 2)
            path = Path(name)
            assert path == ROOT.relative_to(Path.cwd())/'rust-entries/stdout.log'
            actual = path.read_bytes().splitlines()[int(number)-1]
            assert actual == b'test source_invocations_wide ... '
            assert at < len(lines) and lines[at] == '+'+actual.decode()
            at += 1
            stopped = json.loads(path.with_name('result.json').read_bytes())
            replacement = json.loads((ROOT/'individual-supersession/result.json').read_bytes())
            assert stopped['status']=='failed' and stopped['whole_source_inputs_unchanged']
            assert replacement['status']=='passed' and replacement['complete_replacement_obligations_checked_before_supersession']
            trailing_names.append(name)
        else:
            assert row.endswith(': new blank line at EOF.'), row
        diagnostics.append(row)
    for row in diagnostics:
        name = row.rsplit(':', 2)[0]
        path = Path(name)
        assert path.is_relative_to(ROOT.relative_to(Path.cwd()))
        raw = path.read_bytes()
        if path.name in ['stdout.log','stderr.log','compile.stdout.log','compile.stderr.log']:
            result = json.loads(path.with_name('result.json').read_bytes())
            field={'stdout.log':'stdout','stderr.log':'stderr','compile.stdout.log':'compile_stdout','compile.stderr.log':'compile_stderr'}[path.name]
            assert hashlib.sha256(raw).hexdigest() == result[field]['sha256'] and len(raw) == result[field]['bytes']
        else:
            matched=[]
            for receipt in ROOT.glob('*/prelaunch-inputs.json'):
                for original in json.loads(receipt.read_bytes()):
                    saved=Path(original['complete_prelaunch_snapshot']['path'])
                    logical=saved.relative_to(Path.cwd()) if saved.is_absolute() else saved
                    if logical==path:
                        assert Path(original['path']).name in ['stdout.log','stderr.log','compile.stdout.log','compile.stderr.log']
                        assert hashlib.sha256(raw).hexdigest()==original['sha256']==original['complete_prelaunch_snapshot']['sha256']
                        assert len(raw)==original['bytes']==original['complete_prelaunch_snapshot']['bytes']
                        source=Path(original['path']);field={'stdout.log':'stdout','stderr.log':'stderr','compile.stdout.log':'compile_stdout','compile.stderr.log':'compile_stderr'}[source.name];actual=json.loads(source.with_name('result.json').read_bytes())[field]
                        assert actual['sha256']==original['sha256'] and actual['bytes']==original['bytes']
                        assert source.read_bytes()==raw
                        matched.append(original)
            assert len(matched)==1,('not exact receipt-bound original terminal output',name)
        originals.append({'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
    names = [r['path'] for r in originals]
    other = run('diff', '--cached', '--check', '--', '.', *[':(top,literal,exclude)'+n for n in names])
    eof_names = [name for name in names if name not in trailing_names]
    original_check = run('-c', 'core.whitespace=-blank-at-eof', 'diff', '--cached', '--check', '--', *eof_names) if eof_names else None
    interrupted_check = run('-c', 'core.whitespace=-blank-at-eof,-blank-at-eol', 'diff', '--cached', '--check', '--', *trailing_names) if trailing_names else None
    assert other['exit_code'] == 0 and (original_check is None or original_check['exit_code'] == 0) and (interrupted_check is None or interrupted_check['exit_code']==0)
    result = {'status': 'passed_with_exact_immutable_terminal_output_records', 'initial_default_check': initial, 'preserved_original_terminal_output': originals, 'default_check_all_other_files': other, 'raw_output_other_whitespace_check': original_check, 'exact_interrupted_libtest_status_files': trailing_names, 'interrupted_libtest_other_whitespace_check': interrupted_check, 'persistent_git_or_scanner_configuration_weakened': False, 'scope': 'Exact result-bound original compiler/test EOF blank lines and one exact interrupted libtest status line are retained without altering raw bytes. The sole trailing-space exception is byte-for-byte the original rust-entries test status after complete replacement proofs passed. All other original logs still receive the trailing-space check. Every other staged file receives the default whitespace check. No persistent Git or repository scanner setting changes.'}
    (ROOT/'whitespace-audit.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status': result['status'], 'preserved_terminal_output_files': len(originals)}))
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
