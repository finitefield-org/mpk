"""Check staged bytes without rewriting complete original source/log bytes.
Run outside the sandbox; all Git calls use /usr/bin/git.
"""
from pathlib import Path
import datetime, hashlib, json, re, subprocess, sys, types

OUT = Path(__file__).resolve().parents[1]
RUN = Path(__file__).resolve().parent

def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args])

def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}

report = json.loads((OUT / 'input-review.json').read_bytes())
assert report['status'] == 'passed' and report['source_inputs_unchanged']
for row in report['inputs']:
    assert seal(row['path']) == row
raw = (RUN / 'stdout.log').read_bytes()
err = (RUN / 'stderr.log').read_bytes()
assert not err
warnings = re.findall(rb'^(.+):([0-9]+): (trailing whitespace\.|new blank line at EOF\.)$', raw, re.M)
assert len(warnings) == 3
preserved = []
for name, line, kind in warnings:
    path = Path(name.decode())
    value = path.read_bytes()
    line = int(line)
    if kind == b'trailing whitespace.':
        original = (OUT / 'native-components/case-00/sources' / path.name).read_bytes()
        assert path.suffix == '.raw' and value == original
        assert value.split(b'\n')[line - 1].endswith(b'\r')
        reason = 'Exact original Go source includes CR in a raw tag; complete bytes retained.'
    else:
        assert path.name == 'stdout.log' and value.endswith(b'\n\n')
        result = json.loads((path.parent / 'result.json').read_bytes())
        assert seal(path.resolve()) == result['stdout']
        reason = 'Complete actual Cargo stdout ends with its original blank line.'
    preserved.append({**seal(path), 'line': line, 'reason': reason})
assert sum(row['path'].endswith('.raw') for row in preserved) == 0
command = ['/usr/bin/git', 'diff', '--cached', '--check', '--', '.',
           *[':(exclude)' + row['path'] for row in preserved]]
check = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
assert check.returncode == 0 and not check.stdout and not check.stderr
original = Path('/private/tmp/mpk-go-source-signatures-staged-whitespace-original')
original.mkdir(exist_ok=False)
(RUN / 'stdout.log').rename(original / 'stdout.log')
(RUN / 'stderr.log').rename(original / 'stderr.log')
(RUN / 'stdout.hex').write_text(raw.hex() + '\n')
(RUN / 'stderr.hex').write_text(err.hex() + '\n')
assert bytes.fromhex((RUN / 'stdout.hex').read_text()) == (original / 'stdout.log').read_bytes()
record = {
    'status': 'passed', 'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_driver_prelaunch': seal(Path(__file__).resolve()),
    'basis_commit': git('rev-parse', 'HEAD').decode().strip(),
    'input_review': seal(OUT / 'input-review.json'), 'all_reviewed_inputs_unchanged': True,
    'complete_staged_whitespace_command': ['/usr/bin/git', 'diff', '--cached', '--check'],
    'complete_staged_whitespace_exit_code': 2,
    'complete_staged_whitespace_stdout_hex': seal(RUN / 'stdout.hex'),
    'complete_staged_whitespace_stderr_hex': seal(RUN / 'stderr.hex'),
    'exact_preserved_original_whitespace': preserved,
    'scoped_code_and_metadata_whitespace_command': command,
    'scoped_code_and_metadata_whitespace_exit_code': 0,
    'scoped_stdout_hex': check.stdout.hex(), 'scoped_stderr_hex': check.stderr.hex(),
    'new_scanner_exclusions': False, 'public_profile_activated': False,
    'full_T_gate': 'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
}
(RUN / 'result.json').write_text(json.dumps(record, indent=2) + '\n')
scanner = Path('scripts/check-no-active-gir.sh')
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
module = types.ModuleType('mpk_source_signatures_final_stage')
module.__file__ = str(Path.cwd() / scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(Path.cwd()), 'strict']
exec(compile(code, str(scanner), 'exec'), module.__dict__)
matchers = module.load_matchers()
assert len(matchers) == 127
for path in sorted(RUN.iterdir()):
    assert not module.detect(str(path.relative_to(Path.cwd())), path.read_text(), matchers)
assert not module.detect(str(RUN / 'stdout.hex') + '#decoded-original', raw.decode(), matchers)
subprocess.run(['/usr/bin/git', 'add', '--', str(RUN)], check=True)
assert not git('diff', '--name-only') and not git('ls-files', '--others', '--exclude-standard')
changed = git('diff', '--cached', '--name-only', '-z').decode().strip('\0').split('\0')
index = {}
for row in git('ls-files', '--stage', '-z').split(b'\0'):
    if row:
        meta, name = row.split(b'\t', 1)
        mode, oid, stage = meta.decode().split()
        assert stage == '0'
        index[name.decode()] = (mode, oid)
total = 0
for name in changed:
    path = Path(name)
    assert path.is_file() and not path.is_symlink()
    value = path.read_bytes()
    mode, oid = index[name]
    assert hashlib.sha1(b'blob ' + str(len(value)).encode() + b'\0' + value).hexdigest() == oid
    total += len(value)
assert record['basis_commit'] == '323fa842a02ef1d1160a06cd02fccb92fc2eb89d'
print(json.dumps({'status': 'passed', 'reviewed_inputs': len(report['inputs']) + 1,
                  'final_receipt_files': len(list(RUN.iterdir())), 'staged_files': len(changed),
                  'staged_bytes': total, 'preserved_raw_CR_source_copies': 0,
                  'preserved_Cargo_stdout_EOFs': 3, 'working_tree_matches_index': True,
                  'all_reviewed_source_inputs_unchanged': True}))
