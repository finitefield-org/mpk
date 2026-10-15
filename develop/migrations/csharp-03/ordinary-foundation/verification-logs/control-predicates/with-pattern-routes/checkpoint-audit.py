"""Join terminal test and checker receipts without claiming application proofs."""
import argparse
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys


def read(path):
    return json.loads(path.read_bytes())


def digest(path):
    return sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    server = root / 'server-linux'
    audit = read(server / 'checkout-audit-execution.json')
    assert audit['exit_code'] == 0
    assert audit['log_sha256'] == digest(server / 'checkout-audit.log')
    assert audit['source_script_sha256'] == digest(server / 'checkout-audit.py')
    checkout = read(server / 'checkout-verification.json')
    assert json.loads((server / 'checkout-audit.log').read_text().splitlines()[0]) == checkout
    assert checkout['status'] == 'passed_independent_checkout_source_log_binary_audit'
    assert checkout['clean_checkout'] is True
    assert checkout['commit'] == 'a9d2e6fe92b6279521798bcdbb12bf5add6dd898'
    assert checkout['git_blob_files_verified'] == 732
    assert checkout['source_hashes_verified'] == 388
    assert checkout['fixture_hashes_verified'] == 348
    assert checkout['manifest_sha256'] == digest(server / 'source-manifest.json')
    assert checkout['manifest_sha256'] == digest(root / 'source-manifest.json')
    assert checkout['runner_sha256'] == digest(root / 'tool-sources/run-targeted.py')
    assert checkout['supervisor_status_sha256'] == digest(server / 'status.json')
    state = read(server / 'status.json')
    assert checkout['test_binaries_verified'] == state['test_binaries']
    assert checkout['actual_stage_exit_codes'] == {s['stage']: s['exit_code'] for s in state['stages']}
    launch = read(server / 'launch.json')
    for key in ('commit', 'clean_checkout', 'git_blob_files_verified', 'manifest_sha256', 'runner_sha256'):
        assert launch[key] == checkout[key]
    transfer = read(server / 'transfer-execution.json')
    assert transfer['exit_code'] == 0 and transfer['stderr'] == ''
    assert all((server / name).is_file() for name in transfer['files'])

    checks = root / 'checks'
    result = read(checks / 'verification.json')
    assert result['status'] == 'passed'
    assert result['stage_count'] == 72
    assert result['newly_executed_stages'] == 28
    assert result['retained_stage_count'] == 44
    receipt = read(checks / 'independent-audit-execution.json')
    assert receipt['exit_code'] == 0
    assert receipt['log_sha256'] == digest(checks / 'independent-audit.log')
    print('Verified copied Linux checkout, source, log, binary and dual-checker receipts.', flush=True)
    subprocess.run([sys.executable, str(root / 'tool-sources/audit.py'), '--repo', str(args.repo)], check=True)
    checkpoint = read(root / 'verification.json')
    assert all(s['status'] == 'passed_targeted_tests_lint_format' for s in checkpoint['hosts'].values())
    assert checkpoint['checker_terminal_stages'] == 72 and checkpoint['checker_audit_complete'] is True
    assert checkpoint['coverage']['defined_conditions'] == 102
    assert checkpoint['coverage']['pending_conditions'] == 0
    assert checkpoint['application_proofs_pending'] == 987
    assert checkpoint['unit_5_complete'] is False and checkpoint['w09_complete'] is False
    print('Checkpoint passed; application proofs and the final T06 gate remain pending.', flush=True)


if __name__ == '__main__':
    main()
