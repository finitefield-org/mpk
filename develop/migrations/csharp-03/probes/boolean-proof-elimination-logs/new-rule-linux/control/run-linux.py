import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time

parser = argparse.ArgumentParser()
for name in ('repo', 'reports', 'target', 'control', 'commit'):
    parser.add_argument('--' + name, required=True)
a = parser.parse_args()
repo, reports, target, control = map(Path, (a.repo, a.reports, a.target, a.control))
manifest = json.loads((control / 'source-manifest.json').read_text())
expected = json.loads((control / 'expected.json').read_text())
pins = json.loads((control / 'control-hashes.json').read_text())
state = dict(status='running', supervisor_pid=os.getpid(), source_commit=a.commit,
             started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
             stages=[], selection_reason=manifest['selection_reason'],
             full_t01_gate='deferred to renewed T01-W10',
             full_t06_gate='deferred to T06-W12', application_scope_pending=True)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save():
    temporary = reports / 'status.json.tmp'
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    temporary.replace(reports / 'status.json')


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=repo, text=True).strip()


def verify():
    assert git('rev-parse', 'HEAD') == a.commit
    assert not git('status', '--porcelain')
    for group in ('source_hashes', 'fixture_hashes'):
        for name, digest in manifest[group].items():
            assert sha(repo / name) == digest, name
    for name, digest in pins.items():
        assert sha(control / name) == digest, name


def run(stage, command, cwd, env):
    state['stage'] = stage
    save()
    log = reports / (stage + '.log.txt')
    started = time.monotonic()
    with log.open('wb') as stream:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=stream, stderr=subprocess.STDOUT)
        state['process_pid'] = process.pid
        save()
        code = process.wait()
    record = dict(stage=stage, command=command, exit_code=code,
                  log_sha256=sha(log), elapsed_seconds=round(time.monotonic() - started, 3))
    state['stages'].append(record)
    save()
    assert code == 0, stage
    return log


save()
try:
    verify()
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
               CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
               CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',
               GOTOOLCHAIN='go1.23.2')
    jobs = []
    for filter_name in ('bool_cases::', 'env::', 'infer::', 'reduce_inductive::', 'defeq::', 'inductive_gen::'):
        jobs.append(('core-' + filter_name.split(':')[0], ['test', '-p', 'mpk-core', '--lib', filter_name, '--', '--nocapture']))
    for filter_name in ('bool_cases', 'decl_driver::', 'proof_check::'):
        jobs.append(('kernel-' + filter_name.split(':')[0], ['test', '-p', 'mpk-kernel', '--lib', filter_name, '--', '--nocapture']))
    jobs.append(('t01-feasibility', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_spec', 'csharp_03_t01_w09_boolean_proof_elimination_feasibility', '--', '--nocapture']))
    for package in ('mpk-core', 'mpk-kernel'):
        jobs.append((package + '-clippy', ['clippy', '-p', package, '--lib', '--tests', '--', '-D', 'warnings']))
    jobs.extend([
        ('vc-spec-clippy', ['clippy', '-p', 'mpk-vc', '--test', 'csharp_practical_spec', '--', '-D', 'warnings']),
        ('format', ['fmt', '-p', 'mpk-core', '-p', 'mpk-kernel', '-p', 'mpk-vc', '--', '--check']),
        ('rust-checker-build', ['build', '-p', 'mpk-cli', '--bin', 'mpk']),
    ])
    tests = 0
    test_binaries = {}
    for label, arguments in jobs:
        log = run(label, ['/usr/bin/cargo', *arguments], repo, env)
        if arguments[0] == 'test':
            text = log.read_text()
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', text)
            assert len(counts) == 1 and int(counts[0]) > 0, label
            tests += int(counts[0])
            for name in re.findall(r'Running .+ \(([^)]+)\)', text):
                path = Path(name)
                if not path.is_absolute():
                    path = repo / path
                test_binaries[str(path)] = sha(path)
        verify()
    go = shutil.which('go')
    assert go
    run('go-core-defeq-tests', [go, 'test', '-count=1', '-v', '-run', '^Test(BoolCases|Defeq|Core|CheckCore)', './...'], repo / 'go-tools/mpk-checker-ref', env)
    run('go-checker-build', [go, 'build', '-o', str(reports / 'go-checker'), './cmd/mpk-checker-ref'], repo / 'go-tools/mpk-checker-ref', env)
    binaries = {'rust': target / 'debug/mpk', 'go': reports / 'go-checker'}
    state['binaries'] = {k: dict(path=str(v), sha256=sha(v)) for k, v in binaries.items()}
    for case, wanted in expected.items():
        if case == 'predecessor-std-bool':
            data = (repo / 'develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/base.mpcert').read_bytes()
        else:
            path = (repo / 'fixtures/cert-basic' / (case.removeprefix('predecessor-') + '.hex')
                    if case.startswith('predecessor-') else repo / 'fixtures/core-bool-cases' / (case + '.hex'))
            data = bytes.fromhex(path.read_text())
        path = reports / (case + '.mpcert')
        path.write_bytes(data)
        assert sha(path) == wanted['input_sha256'], case
        accepted = []
        for backend, binary in binaries.items():
            label = case + '-' + backend
            state['stage'] = label
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
            local = wanted['reports'][backend]
            assert report['verdict'] == local['verdict'], (label, report)
            good = report['verdict'] == 'accepted'
            assert process.returncode == (0 if good else 1)
            if good:
                assert report.get('axiom_count', 0) == local.get('axiom_count', 0)
                for key in ('module', 'hashes', 'declaration_count', 'axiom_report'):
                    if key in local:
                        assert report[key] == local[key], (label, key)
                accepted.append({k: report[k] for k in ('module', 'hashes')})
            else:
                key = 'error_code' if backend == 'rust' else 'error_kind'
                assert report[key] == local[key] == ('KERNEL_CORE_CHECK' if backend == 'rust' else 'core_check')
            state['stages'].append(dict(stage=label, command=command, exit_code=process.returncode,
                                       input_sha256=sha(path), report_sha256=sha(report_path),
                                       stderr_sha256=sha(stderr_path),
                                       elapsed_seconds=round(time.monotonic() - started, 3)))
            save()
        if accepted:
            assert accepted[0] == accepted[1], case
    verify()
    assert all(sha(v) == state['binaries'][k]['sha256'] for k, v in binaries.items())
    state.update(status='passed_exact_public_source_linux_tests_and_dual_checkers',
                 rust_tests=tests, test_binaries=test_binaries, cases=22, checker_stages=44,
                 verified_source_files=len(manifest['source_hashes']),
                 verified_fixture_files=len(manifest['fixture_hashes']), source_clean_after=True)
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    state['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
