"""Both unchanged production checkers inspect the exact emitted complete bytes."""
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
out = root / ('paired-' + attempt)
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
positive = {'signed-division-prerequisites', 'honest-different-division-prerequisites', 'honest-different-signed-div-definition'}
certificates = sorted((root / attempt / 'rust-tests/certificates').glob('*.mpcert'))
assert [seal(p) for p in certificates] == execution['certificates']
assert len(certificates) == 33
assert {p.stem for p in certificates if json.loads(p.with_suffix('.rust.json').read_text())['verdict'] == 'accepted'} == positive
assert (root / 'joint-prerequisites/rust-construction/certificates/signed-division-prerequisites.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/signed-division-prerequisites.mpcert').read_bytes()
record = {
    'schema': 'mpk.go_practical.paired_word_signed_division.v1', 'status': 'running',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'Rust_execution': seal(execution_path), 'go_binary': binary,
    'unchanged_go_build_evidence': seal(previous_path),
    'unchanged_go_production_sources': previous['go_production_sources'], 'cases': [],
    'selection_reason': 'Full all-width unsigned division, signed magnitude/half/minimum and direct-zero proofs compose under4096 declarations with twelve exact same-type proved substitutions and every mapped proof checked. Three ordinary zero-axiom positives and30 canonical typed mutations use identical bytes in both unchanged cores. The accepted altered SignedDiv definition explicitly demonstrates that these are prerequisites without a signed division theorem; the exact importer rejects every altered component. No CSharp/unrelated suite/intermediate whole gate.',
    'CSharp_tests_executed': False, 'T02_W01_complete': False,
    'public_profile_activated': False, 'source_program_proof_claimed': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
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
        record['cases'].append(row)
        save()
        assert not run.stderr and run.returncode == (0 if accepted else 1) and go['verdict'] == rust['verdict'], row
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
    assert sum(row['name'] in positive for row in record['cases']) == 3
    record.update(status='passed', accepted_zero_axiom=3, rejected_core_type_mismatch=30)
except BaseException as error:
    record.update(status='failed', failure_type=type(error).__name__, failure_detail=str(error))
    save()
    raise
save()
print(json.dumps({'status': 'passed', 'same_byte_cases': 33,
                  'accepted_zero_axiom': 3, 'rejected_core_type_mismatch': 30}))
