"""Both unchanged production checkers inspect the same current numeric bytes."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import time

repo = Path.cwd()
root = Path(__file__).resolve().parent.relative_to(repo.resolve())
attempt = sys.argv[1]
assert attempt.replace('-', '').isalnum()
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
assert not suffix or suffix[1:].isalnum()
out = root / ('paired-' + attempt + suffix)
out.mkdir(exist_ok=False)

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

assert attempt == 'final'
execution_path = root / 'final/rust-tests/result.json'
execution = json.loads(execution_path.read_text())
assert execution['status'] == 'failed' and execution['exit_code'] == 101
assert '3 passed; 1 failed' in (root / 'final/rust-tests/stdout.log').read_text()
changed_control = 'crates/mpk-vc/tests/support/go_word_arithmetic_controls.rs'
for source in execution['sources']:
    snapshot = seal(root / 'final/rust-tests/inputs' / source['path'])
    assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
    if source['path'] != changed_control:
        assert seal(source['path']) == source
focused_path = root / 'corrected/rust-controls/result.json'
focused = json.loads(focused_path.read_text())
assert focused['status'] == 'passed' and focused['exit_code'] == 0
assert '1 passed; 0 failed' in (root / 'corrected/rust-controls/stdout.log').read_text()
assert '3 filtered out' in (root / 'corrected/rust-controls/stdout.log').read_text()
for source in focused['sources']:
    assert seal(source['path']) == source
    snapshot = seal(root / 'corrected/rust-controls/inputs' / source['path'])
    assert (snapshot['sha256'], snapshot['bytes']) == (source['sha256'], source['bytes'])
fuel_path = root / 'final/rust-tests/certificates/WrongMulDiscard-64.rust.json'
fuel = json.loads(fuel_path.read_text())
assert fuel['error_code'] == 'KERNEL_CORE_CHECK' and json.loads(fuel['error_detail'])['code'] == 'CORE_FUEL_EXHAUSTED'
previous_path = root.parent / 'core-interfaces/paired/result.json'
previous = json.loads(previous_path.read_text())
assert previous['status'] == 'passed'
binary = previous['go_binary']
assert seal(binary['path']) == binary
for source in previous['go_production_sources']:
    assert seal(source['path']) == source
positive = {'word-arithmetic-foundation', 'retained-numeric-basis', 'honest-different-arithmetic-component'}
old_certificates = sorted((root / 'final/rust-tests/certificates').glob('*.mpcert'))
assert [seal(p) for p in old_certificates] == execution['certificates'] and len(old_certificates) == 25
retained = [p for p in old_certificates if not p.stem.startswith('WrongMulDiscard-')]
assert len(retained) == 21
corrected = sorted((root / 'corrected/rust-controls/certificates').glob('*.mpcert'))
assert [seal(p) for p in corrected] == focused['certificates'] and len(corrected) == 5
assert {p.stem for p in corrected} == {'WrongMulDiscard-' + str(w) for w in [8,16,32,64]} | {'WrongShiftDiscard-64'}
certificates = sorted(retained + corrected, key=lambda p: p.stem)
assert len(certificates) == len({p.stem for p in certificates}) == 26
assert (root / 'literal-values/rust-construction/certificates/word-arithmetic-foundation.mpcert').read_bytes() == (root / 'final/rust-tests/certificates/word-arithmetic-foundation.mpcert').read_bytes()
record = {
    'schema': 'mpk.go_practical.paired_word_arithmetic.v1', 'status': 'running',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'original_execution': seal(execution_path), 'focused_corrected_control_execution': seal(focused_path), 'original_resource_failure_not_semantic_negative': seal(fuel_path), 'go_binary': binary,
    'unchanged_go_build_evidence': seal(previous_path),
    'unchanged_go_production_sources': previous['go_production_sources'], 'cases': [],
    'selection_reason': 'The complete/import/retained-basis, three generic controls and48-query/1440-stage model roots pass. Preserve one wrong64-bit altered-witness fuel failure as resource evidence only. Correct only four witness mutations to constant zero and run those plus the one unreached64-bit shift case; retain eighteen unchanged typed rejections and all original positive/producer/model bytes. Same fresh actual fixed-left-shift/multiplication arithmetic bytes, exact retained numeric basis and honest different valid component.23 reencoded wrong unbounded Power/Push/Left, ignored shift premise, actual shift/product witness or omitted-modulus contribution controls. Compare exact three positive hashes and the precise current core type mismatch forms; no canonical, fuel or resource failure is a semantic negative.',
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
        run = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
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
    record.update(status='passed', accepted_zero_axiom=3, rejected_core_type_mismatch=23)
except BaseException as error:
    record.update(status='failed', failure_type=type(error).__name__, failure_detail=str(error))
    save()
    raise
save()
print(json.dumps({'status': 'passed', 'same_byte_cases': 26, 'accepted_zero_axiom': 3, 'rejected_core_type_mismatch': 23}))
