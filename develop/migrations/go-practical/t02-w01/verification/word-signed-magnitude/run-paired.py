"""Both unchanged production checkers inspect the same complete order bytes."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import time

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt = sys.argv[1]
assert attempt.replace('-', '').isalnum()
resume = len(sys.argv) == 3 and sys.argv[2] == 'corrected'
assert len(sys.argv) == 2 or resume
out = root / ('paired-' + attempt + ('-corrected' if resume else ''))
out.mkdir(exist_ok=False)

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

execution_path = root / attempt / 'rust-tests/result.json'
execution = json.loads(execution_path.read_text())
assert execution['status'] == 'passed' and execution['exit_code'] == 0
assert '4 passed; 0 failed' in (root / attempt / 'rust-tests/stdout.log').read_text()
for source in execution['sources']:
    assert seal(source['path']) == source
    snapshot = seal(root / attempt / 'rust-tests/inputs' / source['path'])
    assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
previous_path = root.parent / 'core-interfaces/paired/result.json'
previous = json.loads(previous_path.read_text())
assert previous['status'] == 'passed'
binary = previous['go_binary']
assert seal(binary['path']) == binary
for source in previous['go_production_sources']:
    assert seal(source['path']) == source
positive = {'word-signed-magnitude-foundation', 'honest-different-signed-magnitude-component'}
certificates = sorted((root / attempt / 'rust-tests/certificates').glob('*.mpcert'))
assert [seal(p) for p in certificates] == execution['certificates']
assert len(certificates) == 45
assert (root / 'explicit-magnitude-choices/rust-construction/certificates/word-signed-magnitude-foundation.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/word-signed-magnitude-foundation.mpcert').read_bytes()
prior_rows = {}
prior_path = root / ('paired-' + attempt) / 'result.json'
if resume:
    prior = json.loads(prior_path.read_text())
    assert prior['status'] == 'failed' and prior['failure_type'] == 'AssertionError'
    assert prior['failure_detail'].startswith("('honest-different-signed-magnitude-component',")
    assert prior['Rust_execution'] == seal(execution_path) and prior['go_binary'] == binary
    assert prior['unchanged_go_production_sources'] == previous['go_production_sources']
    prior_rows = {row['name']: row for row in prior['cases']}
    assert len(prior_rows) == len(prior['cases']) == 43
record = {
    'schema': 'mpk.go_practical.paired_word_signed_magnitude.v1', 'status': 'running',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'Rust_execution': seal(execution_path), 'go_binary': binary,
    'unchanged_go_build_evidence': seal(previous_path),
    'unchanged_go_production_sources': previous['go_production_sources'], 'cases': [],
    'selection_reason': 'Same actual all-width negation/magnitude bytes in both unchanged production cores. Computed signed-negation correction and both exact-result directions, actual zero/division-guard preservation, sign-to-nonzero and balanced-Int magnitude correspondence. Two zero-axiom positives plus43 altered generic bound/carry/sign/negative-balance and all-width source/zero/guard/correspondence controls. Count only actual core type mismatches as semantic negatives. Only this affected Go root; no CSharp or unrelated prior suite.',
    'CSharp_tests_executed': False, 'T02_W01_complete': False,
    'public_profile_activated': False, 'source_program_proof_claimed': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
if resume:
    record['configuration_fix'] = {'original_failed_record': seal(prior_path),
        'original_runner': seal(root / ('paired-' + attempt) / 'original-runner.py'),
        'reason': 'The honest-different positive retained the preceding component label. Original43 completed negative checks are revalidated against exact current bytes/reports/logs/binary/sources and their original semantic outcomes. Only the two previously unexecuted positive cases run now.',
        'reused_complete_case_count': 43, 'new_checker_invocations': 2}
result = out / 'result.json'

def save():
    result.write_text(json.dumps(record, indent=2) + '\n')

save()
try:
    for certificate in certificates:
        name = certificate.stem
        accepted = name in positive
        rust_path = certificate.with_suffix('.rust.json')
        rust = json.loads(rust_path.read_text())
        assert rust['verdict'] == ('accepted' if accepted else 'rejected'), (name, rust)
        if name in prior_rows:
            row = prior_rows[name]
            assert not accepted
            assert row['certificate'] == seal(certificate) and row['rust_report'] == seal(rust_path)
            assert row['rust_observation'] == rust and row['same_original_bytes'] is True
            assert row['command'] == [binary['path'], 'verify', str(certificate)]
            for key in ['stdout', 'stderr']:
                assert seal(row[key]['path']) == row[key]
            stderr_bytes = Path(row['stderr']['path']).read_bytes()
            go = json.loads(Path(row['stdout']['path']).read_bytes())
            assert go == row['go_observation']
            exit_code = row['exit_code']
        else:
            command = [binary['path'], 'verify', str(certificate)]
            start = time.monotonic()
            run = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 timeout=60, check=False)
            stdout = out / (name + '.stdout.log')
            stderr = out / (name + '.stderr.log')
            stdout.write_bytes(run.stdout)
            stderr.write_bytes(run.stderr)
            go = json.loads(run.stdout)
            row = {'name': name, 'certificate': seal(certificate), 'rust_report': seal(rust_path),
                   'command': command, 'exit_code': run.returncode, 'seconds': time.monotonic() - start,
                   'stdout': seal(stdout), 'stderr': seal(stderr),
                   'rust_observation': rust, 'go_observation': go, 'same_original_bytes': True}
            stderr_bytes = run.stderr
            exit_code = run.returncode
        record['cases'].append(row)
        save()
        assert not stderr_bytes and exit_code == (0 if accepted else 1) and go['verdict'] == rust['verdict'], row
        if accepted:
            assert rust['axiom_count'] == go.get('axiom_count', 0) == 0, row
            assert rust['hashes'] == go['hashes'] and rust['declaration_count'] == go['declaration_count'], row
        else:
            detail = json.loads(rust['error_detail'])
            assert rust['error_code'] == 'KERNEL_CORE_CHECK' and detail['code'] == 'CORE_TYPE_MISMATCH', row
            kinds = {'sort', 'var', 'const', 'app', 'lam', 'pi', 'let'}
            assert go['error_kind'] == 'core_check', row
            if detail['details']['kind'] == 'check_type_mismatch':
                inferred, expected = (detail['details'][k] for k in ['inferred_kind', 'expected_kind'])
                assert inferred in kinds and expected in kinds, row
                assert re.fullmatch(r'term [0-9]+ inferred ' + re.escape(inferred) + ' but expected ' + re.escape(expected), go['error_detail']), row
            else:
                assert detail['details']['kind'] == 'lambda_domain_mismatch', row
                assert detail['details']['actual_kind'] in kinds and detail['details']['expected_kind'] in kinds, row
                assert go['error_detail'] == 'lambda domain mismatch', row
    assert sum(row['name'] in positive for row in record['cases']) == 2
    record.update(status='passed', accepted_zero_axiom=2, rejected_core_type_mismatch=43)
except BaseException as error:
    record.update(status='failed', failure_type=type(error).__name__, failure_detail=str(error))
    save()
    raise
save()
print(json.dumps({'status': 'passed', 'same_byte_cases': 45,
                  'accepted_zero_axiom': 2, 'rejected_core_type_mismatch': 43}))
