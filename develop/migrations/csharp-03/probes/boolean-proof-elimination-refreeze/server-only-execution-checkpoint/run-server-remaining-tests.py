import argparse
from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--kind', choices=['pattern', 'scalar'], required=True)
args = parser.parse_args()
control = Path(__file__).parent
repo = Path('/root/mpk-w09-server-e89c2a0f')
target = Path('/root/mpk-w09-bool-cases-fd99c03c-target')
reports = Path('/root/mpk-w09-server-e89c2a0f-' + args.kind + '-reports')
reports.mkdir(exist_ok=False)
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
manifest = read(control / 'server-source-manifest.json')
plan = read(control / 'remaining-server-test-plan.json')
pins = read(control / 'control-hashes.json')
prior_linux = read(Path('/root/mpk-w09-bool-cases-fd99c03c-linux-2/status.json'))
assert prior_linux['status'] == 'passed_exact_public_source_linux_tests_and_dual_checkers'
assert prior_linux['source_commit'] == plan['validated_checker_source_commit']
binaries = prior_linux['binaries']
assert binaries == plan['validated_checker_binaries']
state = {'status': 'running', 'kind': args.kind, 'stages': [],
         'source_commit': manifest['source_commit'], 'supervisor_pid': os.getpid(),
         'selection_reason': plan['selection_reason'], 'execution_host': 'root@162.43.92.154',
         'binaries': binaries, 'application_proof_ids_pending': 987,
         'full_t01_gate': 'deferred_to_final_W10_on_server',
         'full_t06_gate': 'deferred_to_W12_on_server'}


def save():
    temporary = reports / 'status.json.tmp'
    temporary.write_text(json.dumps(state, sort_keys=True, indent=2) + '\n')
    temporary.replace(reports / 'status.json')


def verify():
    assert subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip() == manifest['source_commit']
    assert not subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'status', '--porcelain'], text=True).strip()
    for path, sha in manifest['hashes'].items():
        assert h(repo / path) == sha, path
    for path, sha in pins.items():
        assert h(control / path) == sha, path
    for binary in binaries.values():
        assert h(Path(binary['path'])) == binary['sha256'], binary['path']


save()
try:
    verify()
    env = os.environ.copy()
    for key in list(env):
        if key.startswith(('MPK_W09_', 'MPK_T06_')) or key == 'MPK_W14_REQUEST_OUTPUT':
            env.pop(key)
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
               CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
               CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',
               PYTHONDONTWRITEBYTECODE='1', GOTOOLCHAIN='go1.23.2')
    if args.kind == 'pattern':
        for case in plan['pattern_cases']:
            source = control / 'pattern-inputs' / case['file']
            assert h(source) == case['input_sha256']
            data = source.read_bytes()
            matches = []
            for backend in ['rust', 'go']:
                label = case['case'] + '-' + backend
                state['stage'] = label
                save()
                command = [binaries[backend]['path'], 'check' if backend == 'rust' else 'verify', str(source)]
                stdout = reports / (label + '.json')
                stderr = reports / (label + '.stderr.txt')
                started = time.monotonic()
                with stdout.open('wb') as so, stderr.open('wb') as se:
                    process = subprocess.Popen(command, cwd=repo, env=env, stdout=so, stderr=se)
                    state['process_pid'] = process.pid
                    save()
                    code = process.wait()
                report = read(stdout)
                row = {'stage': label, 'case': case['case'], 'backend': backend, 'command': command,
                       'exit_code': code, 'verdict': report['verdict'],
                       'input_sha256': h(source), 'report_sha256': h(stdout), 'stderr_sha256': h(stderr),
                       'elapsed_seconds': time.monotonic() - started,
                       'execution_host': state['execution_host'], 'source_commit': manifest['source_commit'],
                       'checker_source_commit': prior_linux['source_commit']}
                state['stages'].append(row)
                save()
                assert code == 0 and report['verdict'] == 'accepted', row
                assert report['hashes']['certificate'] == hashlib.sha256(b'MPK-MODULE-CERT-0.1\0' + data).hexdigest()
                assert report.get('axiom_count', 0) == 0
                if backend == 'rust':
                    assert all(v == 0 for v in report['axiom_report']['summary'].values())
                matches.append({k: report[k] for k in ['module', 'hashes']})
                verify()
            assert matches[0] == matches[1], case['case']
            print(case['case'] + ': both accepted with matching hashes and zero axioms', flush=True)
        assert len(state['stages']) == 12
        state['status'] = 'passed_all_twelve_remaining_pattern_checker_stages_on_server'
    else:
        for label, selector, variable in plan['scalar_owner_jobs']:
            generated = reports / 'goldens' / label
            generated.parent.mkdir(parents=True, exist_ok=True)
            command = ['/usr/bin/cargo', 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc',
                       selector, '--', '--nocapture', '--test-threads=1']
            state['stage'] = label
            save()
            log = reports / (label + '.log.txt')
            started = time.monotonic()
            with log.open('wb') as stream:
                process = subprocess.Popen(command, cwd=repo, env={**env, variable: str(generated)},
                                           stdout=stream, stderr=subprocess.STDOUT)
                state['process_pid'] = process.pid
                save()
                code = process.wait()
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
            row = {'stage': label, 'selector': selector, 'destination': label, 'command': command,
                   'exit_code': code, 'test_counts': counts, 'log_sha256': h(log),
                   'elapsed_seconds': time.monotonic() - started, 'execution_host': state['execution_host']}
            state['stages'].append(row)
            save()
            assert code == 0 and counts == ['1'] and generated.is_dir(), row
            row['generated_files'] = {p.relative_to(generated).as_posix(): h(p)
                                      for p in sorted(generated.rglob('*')) if p.is_file()}
            save()
            verify()
            print(label + ': original complete source/value/import/mutation owner passed', flush=True)
        assert len(state['stages']) == 4
        state['status'] = 'passed_all_four_remaining_original_scalar_codec_owners_on_server'
    verify()
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    save()
