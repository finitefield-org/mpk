import datetime
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import time

root = pathlib.Path('/private/tmp/mpk-w09-bool-cases-reports')
repo = pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
target = pathlib.Path('/private/tmp/mpk-w09-bool-cases-target')
reports = root / 'local-4'
manifest = json.loads((root / 'source-manifest-4.json').read_text())
state = dict(status='running', supervisor_pid=os.getpid(), stages=[],
             selection_reason=manifest['selection_reason'],
             full_t01_gate='deferred to renewed T01-W10',
             full_t06_gate='deferred to T06-W12', application_scope_pending=True)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save():
    path = reports / 'status.json'
    temporary = path.with_suffix('.json.tmp')
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    temporary.replace(path)


def verify():
    for group in ('source_hashes', 'fixture_hashes', 'document_hashes'):
        for name, digest in manifest[group].items():
            assert sha(repo / name) == digest, name


save()
try:
    verify()
    cargo = shutil.which('cargo')
    assert cargo
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(target)
    jobs = []
    for filter_name in ('bool_cases::', 'env::', 'infer::'):
        jobs.append(('core-' + filter_name.split(':')[0], ['test', '-p', 'mpk-core', '--lib', filter_name, '--', '--nocapture']))
    for filter_name in ('bool_cases', 'decl_driver::'):
        jobs.append(('kernel-' + filter_name.split(':')[0], ['test', '-p', 'mpk-kernel', '--lib', filter_name, '--', '--nocapture']))
    jobs.append(('t01-feasibility', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_spec', 'csharp_03_t01_w09_boolean_proof_elimination_feasibility', '--', '--nocapture']))
    jobs.extend([
        ('core-clippy', ['clippy', '-p', 'mpk-core', '--lib', '--tests', '--', '-D', 'warnings']),
        ('kernel-clippy', ['clippy', '-p', 'mpk-kernel', '--lib', '--tests', '--', '-D', 'warnings']),
        ('vc-spec-clippy', ['clippy', '-p', 'mpk-vc', '--test', 'csharp_practical_spec', '--', '-D', 'warnings']),
        ('format', ['fmt', '-p', 'mpk-core', '-p', 'mpk-kernel', '-p', 'mpk-vc', '--', '--check']),
        ('rust-checker-build', ['build', '-p', 'mpk-cli', '--bin', 'mpk']),
    ])
    binaries = {}
    total = 0
    for label, arguments in jobs:
        state['stage'] = label
        save()
        command = [cargo, *arguments]
        log = reports / (label + '.log.txt')
        started = time.monotonic()
        with log.open('wb') as stream:
            process = subprocess.Popen(command, cwd=repo, env=env, stdout=stream, stderr=subprocess.STDOUT)
            state['process_pid'] = process.pid
            save()
            result = process.wait()
        text = log.read_text()
        counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', text)
        if arguments[0] == 'test' and result == 0:
            assert len(counts) == 1 and int(counts[0]) > 0, label
            total += int(counts[0])
            for name in re.findall(r'Running .+ \(([^)]+)\)', text):
                path = pathlib.Path(name)
                if not path.is_absolute():
                    path = repo / path
                if path.is_file():
                    digest = sha(path)
                    assert str(path) not in binaries or binaries[str(path)] == digest
                    binaries[str(path)] = digest
        state['stages'].append(dict(stage=label, command=command, exit_code=result,
                                    elapsed_seconds=round(time.monotonic() - started, 3),
                                    log_sha256=sha(log), test_counts=counts))
        save()
        print(label + ': ' + str(result), flush=True)
        assert result == 0, label
    verify()
    state.update(status='passed_selected_tests_lint_format_build', tests=total,
                 test_binaries=binaries, source_files=len(manifest['source_hashes']),
                 fixture_files=len(manifest['fixture_hashes']), document_files=len(manifest['document_hashes']),
                 rust_checker_sha256=sha(target / 'debug/mpk'))
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    state['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
