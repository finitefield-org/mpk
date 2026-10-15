"""Assemble immutable evidence; this script invokes no tests or checkers.

Only completed predecessor results and terminal server results with an actual
independent server audit may be assembled. Never replace the interrupted status.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

parser = argparse.ArgumentParser()
parser.add_argument('--kind', choices=['pattern', 'scalar'], required=True)
args = parser.parse_args()
b = Path(__file__).parent
migration = b / 'server-only-test-migration'
remote = b / 'server-results-e89c2a0f'
out = b / ('completed-' + args.kind + '-result-provenance')
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())
audit_path = remote / (args.kind + '-audit.json')
audit = read(audit_path)
assert audit['status'] == 'passed_independent_server_' + args.kind + '_terminal_results_review'
assert audit['execution_host'] == 'root@162.43.92.154'
assert audit['local_tests_executed'] == 0
assert audit['source_manifest_raw_sha256'] == h(migration / 'server-source-manifest.json')
server_folder = remote / (args.kind + '-reports')
server_status = read(server_folder / 'status.json')
assert h(server_folder / 'status.json') == audit['runner_status_raw_sha256']
for path, sha in audit['report_files'].items():
    assert h(server_folder / path) == sha, path
assert server_status['source_commit'] == audit['source_commit']
assert server_status['binaries'] == audit['checker_binaries']
assert server_status['application_proof_ids_pending'] == 987

copies = []
def retain(src, relative, expected=None):
    sha = h(src)
    if expected is not None:
        assert sha == expected, str(src)
    copies.append((src, Path(relative), sha))

stages = []
retain(audit_path, 'server/audit.json')
retain(server_folder / 'status.json', 'server/status.json')
retain(migration / 'server-source-manifest.json', 'server/source-manifest.json')
retain(migration / 'control-hashes.json', 'server/control-hashes.json')
retain(migration / 'remaining-server-test-plan.json', 'server/plan.json')
retain(migration / 'run-server-remaining-tests.py', 'server/run-server-remaining-tests.py')
retain(b / 'audit-server-remaining-results.py', 'server/audit-server-remaining-results.py')
retain(migration / 'receipt.json', 'historical/local-interruption-receipt.json')

if args.kind == 'pattern':
    historical_path = migration / 'control-pattern-consumer-dual-checkers-interrupted-status.json'
    assert h(historical_path) == 'e950aae7a9533ba459ca6597abf0f0fbaa98b3773ec8cfcc75f8d70664ec26fc'
    historical = read(historical_path)
    assert historical['status'] == 'running' and len(historical['stages']) == 30
    prior_review = read(b / 'wrapper-completed-230-checker-final-second-review.json')
    prior = prior_review['results']['control-pattern-consumer-dual-checkers']
    assert prior['completed_stages_verified'] == 30 and prior['complete_report_pairs_independently_verified'] == 15
    assert len(server_status['stages']) == audit['verified_stages'] == 12
    retain(b / 'wrapper-completed-230-checker-final-second-review.json', 'historical/completed-thirty-stage-independent-review.json')
    retain(b / 'control-pattern-consumer-dual-checkers/plan.json', 'historical/plan.json', '2a3dcc4ece0b9994531bb832f3232e5eba031051b0fc4c6b655e1d666e42d9bb')
    expected = {row['label'] for row in read(b / 'control-pattern-consumer-dual-checkers/plan.json')['cases']}
    for origin, status, folder in [('historical', historical, b / 'control-pattern-consumer-dual-checkers'), ('server', server_status, server_folder)]:
        for row in status['stages']:
            assert row['exit_code'] == 0 and row['verdict'] == 'accepted'
            assert row['stage'] == row['case'] + '-' + row['backend']
            stdout = Path(origin) / 'reports' / (row['stage'] + '.json')
            stderr = Path(origin) / 'reports' / (row['stage'] + '.stderr.txt')
            retain(folder / stdout.name, stdout, row['report_sha256'])
            retain(folder / stderr.name, stderr, row['stderr_sha256'])
            source = folder / (row['case'] + '.mpcert') if origin == 'historical' else migration / 'pattern-inputs' / (row['case'] + '.mpcert')
            input_file = Path(origin) / 'inputs' / source.name
            retain(source, input_file, row['input_sha256'])
            stages.append({**row, 'provenance': origin, 'execution_host': status.get('execution_host', 'historical_local_execution_before_server_only_instruction'),
                           'binary': status['binaries'][row['backend']], 'report_file': stdout.as_posix(), 'stderr_file': stderr.as_posix(), 'input_file': input_file.as_posix()})
    keys = {(row['case'], row['backend']) for row in stages}
    assert len(keys) == len(stages) == 42
    assert keys == {(case, backend) for case in expected for backend in ['rust', 'go']}
    terminal = 'assembled_twenty_one_actual_checker_pairs_with_explicit_historical_and_server_provenance'
else:
    historical_path = migration / 'ordinary-regeneration-4b-interrupted-status.json'
    assert h(historical_path) == '38e5343d106f12afba0716d886ce05d0d0a7125704dee0bbb59f5a4a2c09b153'
    historical = read(historical_path)
    assert historical['status'] == 'running' and len(historical['stages']) == 6
    prior = read(b / 'six-semantic-owner-checkpoint-second-review.json')
    assert prior['status'] == 'passed_independent_six_completed_owner_checkpoint_review'
    assert prior['actual_complete_original_owner_logs'] == 6 and prior['actual_current_output_files_verified'] == 140
    assert len(server_status['stages']) == audit['verified_stages'] == 4
    retain(b / 'six-semantic-owner-checkpoint-second-review.json', 'historical/completed-six-owner-independent-review.json')
    retain(b / 'ordinary-source-manifest-4b.json', 'historical/source-manifest.json')
    retain(b / 'run-ordinary-regeneration-4b.py', 'historical/run-ordinary-regeneration-4b.py')
    for origin, status, folder in [('historical', historical, b / 'ordinary-regeneration-4b'), ('server', server_status, server_folder)]:
        for row in status['stages']:
            assert row['exit_code'] == 0 and row['test_counts'] == ['1']
            log = Path(origin) / 'logs' / (row['stage'] + '.log.txt')
            retain(folder / log.name, log, row['log_sha256'])
            goldens = Path(origin) / 'goldens' / row['destination']
            for path, sha in row['generated_files'].items():
                retain(folder / 'goldens' / row['destination'] / path, goldens / path, sha)
            stages.append({**row, 'provenance': origin, 'execution_host': status.get('execution_host', 'historical_local_execution_before_server_only_instruction'),
                           'log_file': log.as_posix(), 'goldens_directory': goldens.as_posix()})
    expected = ['integer-parsers', 'integer-formats', 'hex-codecs', 'calendar-codecs', 'decimal-parsers', 'decimal-formats', 'decimal-fixed-formats', 'structural-storage', 'scalar-domains', 'scalar-domain-ranges']
    assert [row['stage'] for row in stages] == expected
    terminal = 'assembled_ten_actual_original_owners_with_explicit_historical_and_server_provenance'

# A reviewed joined scalar report retains the exact original runner snapshots.
if args.kind == 'scalar' and 'actual_original_runner_status_files' in audit:
    originals = audit['actual_original_runner_status_files']
    assert originals == ['original-runner-statuses/serial-status.json', 'original-runner-statuses/independent-status.json']
    for path in originals:
        retain(server_folder / path, Path('server') / path, audit['report_files'][path])

retain(historical_path, 'historical/interrupted-status.json')
retain(Path(__file__), 'assemble-server-result-provenance.py')
out.mkdir(exist_ok=False)
for src, relative, sha in copies:
    destination = out / relative
    if destination.exists():
        assert h(destination) == sha
        continue
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, destination)
    assert h(destination) == sha
receipt = {'status': terminal, 'kind': args.kind, 'stages': stages,
           'historical_stages': len(historical['stages']), 'server_stages': len(server_status['stages']),
           'historical_status_raw_sha256': h(historical_path), 'server_audit_raw_sha256': h(audit_path),
           'historical_execution_preserved': 'All original outputs, interrupted status and source/binary provenance remain unchanged. Only actually completed predecessor stages are retained.',
           'server_execution': 'All new execution and its independent terminal review occurred on root@162.43.92.154. No local test or checker is invoked by evidence assembly.',
           'application_proof_ids_pending': 987, 'full_t01_gate': 'deferred_to_final_W10_on_server', 'full_t06_gate': 'deferred_to_W12_on_server'}
(out / 'receipt.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
files = {p.relative_to(out).as_posix(): h(p) for p in sorted(out.rglob('*')) if p.is_file()}
(out / 'file-manifest.json').write_text(json.dumps({'schema': 'mpk.evidence_file_manifest.v1', 'files': files}, sort_keys=True, indent=2) + '\n')
print('Assembled', args.kind, 'evidence with', len(stages), 'actual completed stages; no tests executed.')
