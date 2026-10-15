import datetime
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import time

root = pathlib.Path('/root/mpk-w09-bool-proof-capability-e2774c23-3')
control = root / 'control'
reports = root / 'reports'
source = pathlib.Path('/root/mpk-w09-context-application-f02dcb32')
target = pathlib.Path('/root/mpk-w09-context-application-f02dcb32-target')
commit = 'f02dcb32ee0ff420a86083ea95067879455dec81'
reports.mkdir()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=source, text=True).strip()


manifest = json.loads((control / 'source-manifest.json').read_text())
pins = json.loads((control / 'control-hashes.json').read_text())
expected = json.loads((control / 'expected.json').read_text())
state = dict(status='running', supervisor_pid=os.getpid(), source_commit=commit,
             started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
             stages=[], diagnostic_only=True, application_scope_pending=True,
             full_t_gate='deferred to T06-W12')


def save():
    temporary = reports / 'status.json.tmp'
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    temporary.replace(reports / 'status.json')


def check_inputs():
    assert git('rev-parse', 'HEAD') == commit
    assert not git('status', '--porcelain')
    for group in ('source_hashes', 'fixture_hashes'):
        for name, digest in manifest[group].items():
            assert sha(source / name) == digest, name
    for name, digest in pins.items():
        assert sha(control / name) == digest, name


def build(stage, command, cwd, env):
    state['stage'] = stage
    save()
    started = time.monotonic()
    log = reports / (stage + '.log.txt')
    with log.open('wb') as stream:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=stream,
                                   stderr=subprocess.STDOUT)
        state['process_pid'] = process.pid
        save()
        result = process.wait()
    state['stages'].append(dict(stage=stage, command=command, exit_code=result,
                                elapsed_seconds=round(time.monotonic() - started, 3),
                                log_sha256=sha(log)))
    save()
    assert result == 0, stage


save()
try:
    check_inputs()
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(target)
    env['GOTOOLCHAIN'] = 'go1.23.2'
    build('rust-checker-build', ['/usr/bin/cargo', 'build', '-p', 'mpk-cli', '--bin', 'mpk'], source, env)
    go = shutil.which('go')
    assert go
    build('go-checker-build', [go, 'build', '-o', str(reports / 'go-checker'), './cmd/mpk-checker-ref'],
          source / 'go-tools/mpk-checker-ref', env)
    binaries = {'rust': target / 'debug/mpk', 'go': reports / 'go-checker'}
    state['binaries'] = {backend: dict(path=str(path), sha256=sha(path))
                         for backend, path in binaries.items()}
    accepted = {}
    for case, wanted in expected.items():
        path = control / (case + '.mpcert')
        data = path.read_bytes()
        assert sha(path) == wanted['input_sha256']
        for backend, binary in binaries.items():
            label = case + '-' + backend
            state['stage'] = label
            save()
            command = [str(binary), 'check' if backend == 'rust' else 'verify', str(path)]
            started = time.monotonic()
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            state['process_pid'] = process.pid
            save()
            stdout, stderr = process.communicate()
            report_path = reports / (label + '.json')
            stderr_path = reports / (label + '.stderr.txt')
            report_path.write_bytes(stdout)
            stderr_path.write_bytes(stderr)
            report = json.loads(stdout)
            good = wanted['accepted']
            assert process.returncode == (0 if good else 1)
            assert report['verdict'] == ('accepted' if good else 'rejected')
            if good:
                assert report.get('axiom_count', 0) == 0
                if backend == 'rust':
                    assert all(v == 0 for v in report['axiom_report']['summary'].values())
                assert report['hashes']['certificate'] == hashlib.sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest()
                fields = ('module', 'declaration_count', 'hashes')
                summary = {key: report[key] for key in fields}
                assert summary == {key: wanted['local_reports'][backend][key] for key in fields}
                accepted[(case, backend)] = summary
            else:
                assert report['error_code' if backend == 'rust' else 'error_kind'] == ('KERNEL_CORE_CHECK' if backend == 'rust' else 'core_check')
            state['stages'].append(dict(stage=label, command=command, exit_code=process.returncode,
                                        input_sha256=sha(path), report_sha256=sha(report_path),
                                        stderr_sha256=sha(stderr_path),
                                        elapsed_seconds=round(time.monotonic() - started, 3)))
            save()
    for case, wanted in expected.items():
        if wanted['accepted']:
            assert accepted[(case, 'rust')] == accepted[(case, 'go')]
    check_inputs()
    assert all(sha(path) == state['binaries'][backend]['sha256'] for backend, path in binaries.items())
    state.update(status='completed_capability_diagnostic', cases=5, checker_stages=10,
                 verified_source_files=len(manifest['source_hashes']),
                 verified_fixture_files=len(manifest['fixture_hashes']),
                 source_clean_after=True, control_hashes=pins)
except BaseException as error:
    state.update(status='failed_capability_diagnostic', error=repr(error))
    raise
finally:
    state['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
