"""Execute three independent original owners using the already built binary.

Every original assertion and input remains. Outputs are separate from the
unchanged original serial controller and decimal fixed-format observation.
"""
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import time


assert platform.system() == 'Linux'
repo = Path('/root/mpk-w09-server-e89c2a0f')
control = Path('/root/mpk-w09-server-e89c2a0f-control')
out = Path('/root/mpk-w09-server-e89c2a0f-independent-scalar-reports')
assert not out.exists()
binary = Path('/root/mpk-w09-bool-cases-fd99c03c-target/debug/deps/csharp_practical_vc-18f294cebe272f30')
binary_sha = 'c6c381c36d49c95d1a1d2c13cad4011ebba2e88a8c3f06b447815ff240cebb5d'
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
manifest = read(control / 'server-source-manifest.json')
pins = read(control / 'control-hashes.json')
plan = read(control / 'remaining-server-test-plan.json')
jobs = plan['scalar_owner_jobs'][1:]
assert len(jobs) == 3
assert [row[0] for row in jobs] == ['structural-storage', 'scalar-domains', 'scalar-domain-ranges']
original = read(Path('/root/mpk-w09-server-e89c2a0f-scalar-reports/status.json'))
assert original['source_commit'] == manifest['source_commit'] == 'e89c2a0f24e1f4a1c562c6a60c3199473781f101'
assert original['stage'] == 'decimal-fixed-formats' and not original['stages']
assert h(binary) == binary_sha
state = {'status': 'running', 'stages': [], 'active': {}, 'supervisor_pid': os.getpid(),
         'execution_host': 'root@162.43.92.154', 'source_commit': manifest['source_commit'],
         'original_owner_binary': {'path': str(binary), 'raw_sha256': binary_sha},
         'source_manifest_raw_sha256': h(control / 'server-source-manifest.json'),
         'original_serial_controller_modified': False, 'application_proof_ids_pending': 987,
         'selection_reason': 'The three unfinished structural/scalar/range owners read independent immutable inputs, keep process-local state and write only their separately selected output directories. Execute the original compiled test selectors concurrently; no source assertion changes, rebuild, fixture overwrite or unrelated owner rerun.',
         'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server'}


def save():
    p = out / 'status.json.tmp'
    p.write_text(json.dumps(state, sort_keys=True, indent=2) + '\n')
    p.replace(out / 'status.json')


def verify():
    assert subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip() == manifest['source_commit']
    assert not subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'status', '--porcelain'], text=True).strip()
    for path, sha in manifest['hashes'].items():
        assert h(repo / path) == sha, path
    for path, sha in pins.items():
        assert h(control / path) == sha, path
    assert h(binary) == binary_sha


verify()
env = os.environ.copy()
for key in list(env):
    if key.startswith(('MPK_W09_', 'MPK_T06_')) or key == 'MPK_W14_REQUEST_OUTPUT':
        env.pop(key)
env.update(PYTHONDONTWRITEBYTECODE='1', GOTOOLCHAIN='go1.23.2')
listed = subprocess.run([str(binary), '--list'], cwd=repo, env=env, check=True,
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
names = [line.removesuffix(': test') for line in listed.stdout.splitlines() if line.endswith(': test')]
matches = {}
for label, selector, variable in jobs:
    selected = [name for name in names if selector in name]
    assert len(selected) == 1, (label, selected)
    matches[label] = selected[0]
out.mkdir()
(out / 'original-test-list.stdout.txt').write_text(listed.stdout)
(out / 'original-test-list.stderr.txt').write_text(listed.stderr)
state['original_selected_test_names'] = matches
state['original_test_list_stdout_sha256'] = h(out / 'original-test-list.stdout.txt')
state['original_test_list_stderr_sha256'] = h(out / 'original-test-list.stderr.txt')
save()
active = {}
try:
    for label, selector, variable in jobs:
        generated = out / 'goldens' / label
        generated.parent.mkdir(exist_ok=True)
        command = [str(binary), matches[label], '--exact', '--nocapture', '--test-threads=1']
        log = out / (label + '.log.txt')
        stream = log.open('wb')
        process = subprocess.Popen(command, cwd=repo, env={**env, variable: str(generated)},
                                   stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        active[label] = (process, stream, time.monotonic(), selector, command, generated, log)
        state['active'][label] = {'process_pid': process.pid, 'command': command}
        save()
    while active:
        completed = []
        for label, (process, stream, start, selector, command, generated, log) in active.items():
            code = process.poll()
            if code is None:
                continue
            stream.close()
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
            row = {'stage': label, 'destination': label, 'selector': selector, 'command': command,
                   'exit_code': code, 'test_counts': counts, 'log_sha256': h(log),
                   'elapsed_seconds': time.monotonic() - start, 'execution_host': state['execution_host'],
                   'source_commit': manifest['source_commit'], 'original_test_binary_raw_sha256': binary_sha}
            state['stages'].append(row)
            state['active'].pop(label)
            save()
            assert code == 0 and counts == ['1'] and generated.is_dir(), row
            files = {p.relative_to(generated).as_posix(): h(p) for p in sorted(generated.rglob('*')) if p.is_file()}
            assert files
            for path, sha in files.items():
                current = repo / 'develop/migrations/csharp-03/ordinary-foundation' / label / path
                assert h(current) == sha and current.read_bytes() == (generated / path).read_bytes(), (label, path)
            row['generated_files'] = files
            save()
            verify()
            completed.append(label)
            print(label + ': complete original source/value/import/mutation owner and exact current output passed', flush=True)
        for label in completed:
            active.pop(label)
        if active:
            time.sleep(1)
    assert len(state['stages']) == 3
    state['status'] = 'passed_all_three_independent_original_scalar_owners_and_exact_current_output_bytes_on_server'
except BaseException as error:
    state.update(status='failed', error=repr(error))
    raise
finally:
    save()
