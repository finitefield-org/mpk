from pathlib import Path
import hashlib
import json
import subprocess

root = Path('/root/mpk-w09-bool-proof-capability-e2774c23-3')
control = root / 'control'
reports = root / 'reports'
source = Path('/root/mpk-w09-context-application-f02dcb32')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=source, text=True).strip()


launch = json.loads((root / 'launch.json').read_text())
state = json.loads((reports / 'status.json').read_text())
manifest = json.loads((control / 'source-manifest.json').read_text())
assert state['status'] == 'completed_capability_diagnostic'
assert git('rev-parse', 'HEAD') == state['source_commit'] == 'f02dcb32ee0ff420a86083ea95067879455dec81'
assert not git('status', '--porcelain')
assert not (Path('/proc') / str(launch['supervisor_pid'])).exists()
assert len(state['stages']) == 12
for name, digest in launch['control_hashes'].items():
    assert sha(control / name) == digest, name
blob_files = 0
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        assert sha(source / name) == digest, name
        blob = subprocess.check_output(['/usr/bin/git', 'show', 'HEAD:' + name], cwd=source)
        assert hashlib.sha256(blob).hexdigest() == digest, name
        blob_files += 1
for stage in state['stages']:
    label = stage['stage']
    if 'log_sha256' in stage:
        assert stage['exit_code'] == 0
        assert sha(reports / (label + '.log.txt')) == stage['log_sha256']
    else:
        assert sha(reports / (label + '.json')) == stage['report_sha256']
        assert sha(reports / (label + '.stderr.txt')) == stage['stderr_sha256']
for binary in state['binaries'].values():
    assert sha(Path(binary['path'])) == binary['sha256']
result = dict(status='passed_exact_public_source_linux_audit',
              source_commit=state['source_commit'], source_clean=True,
              git_blob_files_verified=blob_files, checker_stages=10,
              diagnostic_only=True, application_scope_pending=True,
              binaries=state['binaries'],
              reports={p.name: sha(p) for p in reports.iterdir() if p.is_file()},
              control_hashes=launch['control_hashes'],
              source_manifest_sha256=sha(control / 'source-manifest.json'),
              runner_status_sha256=sha(reports / 'status.json'),
              launch_sha256=sha(root / 'launch.json'))
(root / 'final-audit.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
print(json.dumps(result))
