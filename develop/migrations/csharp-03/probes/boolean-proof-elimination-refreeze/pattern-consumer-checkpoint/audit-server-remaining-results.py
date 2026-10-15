"""Run only on the approved SSH host, after the selected runner terminates."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--kind', choices=['pattern', 'scalar'], required=True)
args = parser.parse_args()
control = Path('/root/mpk-w09-server-e89c2a0f-control')
repo = Path('/root/mpk-w09-server-e89c2a0f')
reports = Path('/root/mpk-w09-server-e89c2a0f-' + args.kind + '-reports')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
plan = read(control / 'remaining-server-test-plan.json')
manifest = read(control / 'server-source-manifest.json')
state = read(reports / 'status.json')
terminal = {
    'pattern': 'passed_all_twelve_remaining_pattern_checker_stages_on_server',
    'scalar': 'passed_all_four_remaining_original_scalar_codec_owners_on_server',
}[args.kind]
assert state['status'] == terminal, 'The actual runner must finish successfully first.'
assert state['execution_host'] == 'root@162.43.92.154'
assert state['source_commit'] == plan['source_commit'] == manifest['source_commit']
assert subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip() == manifest['source_commit']
assert not subprocess.check_output(['/usr/bin/git', '-C', str(repo), 'status', '--porcelain'], text=True).strip()
for path, sha in manifest['hashes'].items():
    assert h(repo / path) == sha, path
for path, sha in read(control / 'control-hashes.json').items():
    assert h(control / path) == sha, path
assert state['binaries'] == plan['validated_checker_binaries']
for binary in state['binaries'].values():
    assert h(Path(binary['path'])) == binary['sha256'], binary['path']

files = {'status.json': h(reports / 'status.json')}
verified = []
if args.kind == 'pattern':
    cases = {c['case']: c for c in plan['pattern_cases']}
    assert len(cases) == 6 and len(state['stages']) == 12
    actual = {}
    for row in state['stages']:
        key = (row['case'], row['backend'])
        assert key not in actual and row['case'] in cases
        assert row['backend'] in ['rust', 'go']
        assert row['stage'] == row['case'] + '-' + row['backend']
        assert row['exit_code'] == 0 and row['verdict'] == 'accepted'
        assert row['source_commit'] == manifest['source_commit']
        assert row['checker_source_commit'] == plan['validated_checker_source_commit']
        assert row['execution_host'] == state['execution_host']
        input_path = control / 'pattern-inputs' / cases[row['case']]['file']
        assert h(input_path) == row['input_sha256'] == cases[row['case']]['input_sha256']
        stdout, stderr = reports / (row['stage'] + '.json'), reports / (row['stage'] + '.stderr.txt')
        assert h(stdout) == row['report_sha256'] and h(stderr) == row['stderr_sha256']
        report = read(stdout)
        assert report['verdict'] == 'accepted'
        assert report['hashes']['certificate'] == hashlib.sha256(b'MPK-MODULE-CERT-0.1\0' + input_path.read_bytes()).hexdigest()
        assert report.get('axiom_count', 0) == 0
        if row['backend'] == 'rust':
            assert all(v == 0 for v in report['axiom_report']['summary'].values())
            assert not report['axiom_report']['entries']
            assert not report['axiom_report']['declaration_dependencies']
        actual[key] = report
        files[stdout.name], files[stderr.name] = h(stdout), h(stderr)
    assert set(actual) == {(case, backend) for case in cases for backend in ['rust', 'go']}
    for case in cases:
        for key in ['module', 'declaration_count', 'hashes']:
            assert actual[(case, 'rust')][key] == actual[(case, 'go')][key], (case, key)
        verified.append({'case': case, 'input_sha256': cases[case]['input_sha256'], 'both_checkers_accepted': True})
else:
    jobs = plan['scalar_owner_jobs']
    assert len(state['stages']) == len(jobs) == 4
    for row, (label, selector, variable) in zip(state['stages'], jobs):
        assert row['stage'] == row['destination'] == label and row['selector'] == selector
        assert row['exit_code'] == 0 and row['test_counts'] == ['1']
        assert row['execution_host'] == state['execution_host']
        log = reports / (label + '.log.txt')
        assert h(log) == row['log_sha256']
        assert re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text()) == ['1']
        generated = reports / 'goldens' / label
        actual_files = {p.relative_to(generated).as_posix(): h(p) for p in sorted(generated.rglob('*')) if p.is_file()}
        assert actual_files and actual_files == row['generated_files']
        for path, sha in actual_files.items():
            source = generated / path
            current = repo / 'develop/migrations/csharp-03/ordinary-foundation' / label / path
            assert h(current) == sha and source.read_bytes() == current.read_bytes(), (label, path)
            files['goldens/' + label + '/' + path] = sha
        files[log.name] = h(log)
        verified.append({'owner': label, 'selector': selector, 'original_complete_test_passed': True, 'exact_current_output_files': actual_files})

receipt = {
    'status': 'passed_independent_server_' + args.kind + '_terminal_results_review',
    'kind': args.kind, 'execution_host': state['execution_host'],
    'source_commit': manifest['source_commit'],
    'source_manifest_raw_sha256': h(control / 'server-source-manifest.json'),
    'verified_source_files': len(manifest['hashes']),
    'runner_status_raw_sha256': h(reports / 'status.json'),
    'checker_binaries': state['binaries'],
    'checker_source_commit': plan['validated_checker_source_commit'],
    'verified_stages': len(state['stages']), 'verified': verified, 'report_files': files,
    'historical_interrupted_statuses_modified': False,
    'historical_results_attributed_to_new_source_or_binaries': False,
    'local_tests_executed': 0, 'application_proof_ids_pending': 987,
    'full_t01_gate': 'deferred_to_final_W10_on_server',
    'full_t06_gate': 'deferred_to_W12_on_server',
}
out = Path('/root/mpk-w09-server-e89c2a0f-' + args.kind + '-audit.json')
assert not out.exists(), 'Preserve any earlier audit receipt.'
out.write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
print(json.dumps({k: receipt[k] for k in ['status', 'execution_host', 'verified_stages', 'verified_source_files']}, sort_keys=True))
