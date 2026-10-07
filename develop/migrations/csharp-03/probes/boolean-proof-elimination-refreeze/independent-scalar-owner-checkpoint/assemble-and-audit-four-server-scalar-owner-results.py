"""Join only four actually complete owner results and audit on the server.

Neither original runner status is edited or relabeled as terminal success.
"""
import hashlib
import json
from pathlib import Path
import platform
import re
import shutil
import subprocess


assert platform.system() == 'Linux'
repo = Path('/root/mpk-w09-server-e89c2a0f')
control = Path('/root/mpk-w09-server-e89c2a0f-control')
serial = Path('/root/mpk-w09-server-e89c2a0f-scalar-reports')
parallel = Path('/root/mpk-w09-server-e89c2a0f-independent-scalar-reports')
out = Path('/root/mpk-w09-server-e89c2a0f-joined-scalar-reports')
audit_path = Path('/root/mpk-w09-server-e89c2a0f-scalar-joined-audit.json')
assert not out.exists() and not audit_path.exists()
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
manifest = read(control / 'server-source-manifest.json')
plan = read(control / 'remaining-server-test-plan.json')
original = read(serial / 'status.json')
independent = read(parallel / 'status.json')
assert independent['status'] == 'passed_all_three_independent_original_scalar_owners_and_exact_current_output_bytes_on_server'
assert len(independent['stages']) == 3 and not independent['active']
assert original['source_commit'] == independent['source_commit'] == manifest['source_commit'] == plan['source_commit']
assert original['execution_host'] == independent['execution_host'] == 'root@162.43.92.154'
assert original['binaries'] == plan['validated_checker_binaries']
assert subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip() == manifest['source_commit']
assert not subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'status', '--porcelain'], text=True).strip()
for path, sha in manifest['hashes'].items():
    assert h(repo / path) == sha, path
for path, sha in read(control / 'control-hashes.json').items():
    assert h(control / path) == sha, path
for binary in original['binaries'].values():
    assert h(Path(binary['path'])) == binary['sha256']
binary = independent['original_owner_binary']
assert h(Path(binary['path'])) == binary['raw_sha256'] == 'c6c381c36d49c95d1a1d2c13cad4011ebba2e88a8c3f06b447815ff240cebb5d'
assert independent['source_manifest_raw_sha256'] == h(control / 'server-source-manifest.json')
first = [row for row in original['stages'] if row['stage'] == 'decimal-fixed-formats']
assert len(first) == 1, 'The complete original decimal fixed-format owner must finish first.'
rows = {first[0]['stage']: (first[0], serial)}
for row in independent['stages']:
    assert row['stage'] not in rows
    rows[row['stage']] = (row, parallel)
assert len(rows) == 4
stages, copies, verified = [], [], []
for label, selector, variable in plan['scalar_owner_jobs']:
    row, folder = rows[label]
    assert row['stage'] == row['destination'] == label and row['selector'] == selector
    assert row['exit_code'] == 0 and row['test_counts'] == ['1']
    assert row['execution_host'] == 'root@162.43.92.154'
    log = folder / (label + '.log.txt')
    assert h(log) == row['log_sha256']
    assert re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text()) == ['1']
    copies.append((log, Path(log.name), h(log)))
    generated = folder / 'goldens' / label
    actual = {p.relative_to(generated).as_posix(): h(p) for p in sorted(generated.rglob('*')) if p.is_file()}
    assert actual and actual == row['generated_files']
    for path, sha in actual.items():
        src = generated / path
        current = repo / 'develop/migrations/csharp-03/ordinary-foundation' / label / path
        assert h(current) == sha and src.read_bytes() == current.read_bytes(), (label, path)
        copies.append((src, Path('goldens') / label / path, sha))
    # Keep every original row field and add only its actual report location.
    stages.append({**row, 'actual_original_report_directory': str(folder)})
    verified.append({'owner': label, 'selector': selector, 'original_complete_test_passed': True,
                     'exact_current_output_files': actual, 'actual_original_report_directory': str(folder)})
for folder, label in [(serial, 'serial'), (parallel, 'independent')]:
    p = folder / 'status.json'
    copies.append((p, Path('original-runner-statuses') / (label + '-status.json'), h(p)))
out.mkdir()
for source, path, sha in copies:
    target = out / path
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)
    assert h(target) == sha
state = {
    'status': 'assembled_four_complete_actual_server_owner_results_with_original_runner_statuses_retained',
    'kind': 'scalar', 'stages': stages, 'source_commit': manifest['source_commit'],
    'execution_host': 'root@162.43.92.154', 'binaries': original['binaries'],
    'original_owner_binary': binary, 'application_proof_ids_pending': 987,
    'selection_reason': 'One complete original decimal fixed-format owner plus three complete independently executed original structural/scalar/range owners. Every original row field, assertion, log, source/input/control/binary SHA and complete output bytes remain. Neither original status is modified.',
    'actual_serial_status_raw_sha256': h(out / 'original-runner-statuses/serial-status.json'),
    'actual_independent_status_raw_sha256': h(out / 'original-runner-statuses/independent-status.json'),
    'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server',
}
(out / 'status.json').write_text(json.dumps(state, sort_keys=True, indent=2) + '\n')
files = {p.relative_to(out).as_posix(): h(p) for p in sorted(out.rglob('*')) if p.is_file()}
audit = {
    'status': 'passed_independent_server_scalar_terminal_results_review', 'kind': 'scalar',
    'execution_host': state['execution_host'], 'source_commit': manifest['source_commit'],
    'source_manifest_raw_sha256': h(control / 'server-source-manifest.json'),
    'verified_source_files': len(manifest['hashes']), 'runner_status_raw_sha256': h(out / 'status.json'),
    'checker_binaries': original['binaries'], 'checker_source_commit': plan['validated_checker_source_commit'],
    'verified_stages': 4, 'verified': verified, 'report_files': files,
    'actual_original_runner_status_files': ['original-runner-statuses/serial-status.json', 'original-runner-statuses/independent-status.json'],
    'original_owner_binary': binary, 'original_stage_fields_retained': True,
    'historical_interrupted_statuses_modified': False,
    'historical_results_attributed_to_new_source_or_binaries': False,
    'local_tests_executed': 0, 'application_proof_ids_pending': 987,
    'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server',
}
audit_path.write_text(json.dumps(audit, sort_keys=True, indent=2) + '\n')
print(json.dumps({'status': audit['status'], 'verified_stages': 4,
                  'report_files': len(files), 'audit_path': str(audit_path),
                  'audit_raw_sha256': h(audit_path)}, sort_keys=True))
