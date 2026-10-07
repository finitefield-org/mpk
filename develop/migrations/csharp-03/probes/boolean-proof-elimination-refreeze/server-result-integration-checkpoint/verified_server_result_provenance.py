"""Hash-pinned evidence access for mutation guards; invokes no test/checker."""
import hashlib
import json
from pathlib import Path

def load_result_provenance(base, kind):
    folder = Path(base) / ('completed-' + kind + '-result-provenance')
    read = lambda p: json.loads(p.read_bytes())
    h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    files = read(folder / 'file-manifest.json')['files']
    for path, sha in files.items():
        assert h(folder / path) == sha, path
    receipt = read(folder / 'receipt.json')
    actual_audit = read(folder / 'server/audit.json')
    assert actual_audit['status'] == 'passed_independent_server_' + kind + '_terminal_results_review'
    assert actual_audit['execution_host'] == 'root@162.43.92.154' and actual_audit['local_tests_executed'] == 0
    assert h(folder / 'server/audit.json') == receipt['server_audit_raw_sha256']
    assert h(folder / 'historical/interrupted-status.json') == receipt['historical_status_raw_sha256']
    assert actual_audit['runner_status_raw_sha256'] == h(folder / 'server/status.json')
    historical = read(folder / 'historical/interrupted-status.json')
    server = read(folder / 'server/status.json')
    assert historical['status'] == 'running'
    assert receipt['historical_stages'] == len(historical['stages'])
    assert receipt['server_stages'] == len(server['stages']) == actual_audit['verified_stages']
    originals = [('historical', row) for row in historical['stages']] + [('server', row) for row in server['stages']]
    assert len(receipt['stages']) == len(originals)
    for combined, (origin, original) in zip(receipt['stages'], originals):
        assert combined['provenance'] == origin
        assert all(combined[key] == value for key, value in original.items()), original['stage']
        if kind == 'pattern':
            assert h(folder / combined['report_file']) == original['report_sha256']
            assert h(folder / combined['stderr_file']) == original['stderr_sha256']
            assert h(folder / combined['input_file']) == original['input_sha256']
        else:
            assert h(folder / combined['log_file']) == original['log_sha256']
            for path, sha in original['generated_files'].items():
                assert h(folder / combined['goldens_directory'] / path) == sha
    if kind == 'pattern':
        assert receipt['status'] == 'assembled_twenty_one_actual_checker_pairs_with_explicit_historical_and_server_provenance'
        assert receipt['historical_stages'] == 30 and receipt['server_stages'] == 12
        assert len(receipt['stages']) == 42
        assert all(s['exit_code'] == 0 and s['verdict'] == 'accepted' for s in receipt['stages'])
    else:
        assert receipt['status'] == 'assembled_ten_actual_original_owners_with_explicit_historical_and_server_provenance'
        assert receipt['historical_stages'] == 6 and receipt['server_stages'] == 4
        assert len(receipt['stages']) == 10
        assert all(s['exit_code'] == 0 and s['test_counts'] == ['1'] for s in receipt['stages'])
    return receipt, folder
