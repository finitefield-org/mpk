from pathlib import Path
import hashlib
import json
import subprocess
import re

root = Path('/root/mpk-w09-bool-cases-97559877-linux')
repo = Path('/root/mpk-w09-bool-cases-97559877')
control = Path('/root/mpk-w09-bool-cases-97559877-control')
commit = '9755987798ec65b266035f1d0418fc7f2890182a'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=repo, text=True).strip()


state = json.loads((root / 'status.json').read_text())
launch = json.loads((root / 'launch.json').read_text())
manifest = json.loads((control / 'source-manifest.json').read_text())
assert state['status'] == 'passed_exact_public_source_linux_tests_and_dual_checkers'
assert state['source_commit'] == launch['commit'] == git('rev-parse', 'HEAD') == commit
assert not git('status', '--porcelain')
assert not (Path('/proc') / str(launch['supervisor_pid'])).exists()
assert state['rust_tests'] == 86 and state['checker_stages'] == 44
assert len(state['stages']) == 61
files = 0
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        assert sha(repo / name) == digest, name
        blob = subprocess.check_output(['/usr/bin/git', 'show', 'HEAD:' + name], cwd=repo)
        assert hashlib.sha256(blob).hexdigest() == digest, name
        files += 1
assert files == launch['git_blob_files_verified'] == 1088
for name, digest in launch['control_hashes'].items():
    assert sha(control / name) == digest, name
for stage in state['stages']:
    label = stage['stage']
    if 'log_sha256' in stage:
        assert stage['exit_code'] == 0
        assert sha(root / (label + '.log.txt')) == stage['log_sha256']
    else:
        assert sha(root / (label + '.json')) == stage['report_sha256']
        assert sha(root / (label + '.stderr.txt')) == stage['stderr_sha256']
        case = label.rsplit('-', 1)[0]
        assert sha(root / (case + '.mpcert')) == stage['input_sha256']
for path, digest in state['test_binaries'].items():
    assert sha(Path(path)) == digest, path
for binary in state['binaries'].values():
    assert sha(Path(binary['path'])) == binary['sha256']
go_tests = len(re.findall(r'^--- PASS: Test', (root / 'go-core-defeq-tests.log.txt').read_text(), re.M))
assert go_tests == 16
result = dict(status='passed_exact_public_source_linux_audit', source_commit=commit,
              rust_tests=86, go_top_level_tests=16, cases=22, checker_stages=44,
              git_blob_files_verified=files, source_clean=True,
              source_manifest_sha256=sha(control / 'source-manifest.json'),
              runner_status_sha256=sha(root / 'status.json'), launch_sha256=sha(root / 'launch.json'),
              control_hashes=launch['control_hashes'], binaries=state['binaries'],
              test_binaries=state['test_binaries'],
              reports={p.name: sha(p) for p in root.iterdir() if p.is_file()},
              full_t01_gate='deferred to renewed T01-W10',
              full_t06_gate='deferred to T06-W12', application_scope_pending=True)
(root / 'final-audit.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
print(json.dumps(result))
