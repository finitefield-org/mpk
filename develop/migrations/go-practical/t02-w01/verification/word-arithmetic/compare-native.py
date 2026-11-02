"""Compare every actual circuit stage with native Go and exact Python arithmetic."""
from pathlib import Path
import datetime
import hashlib
import json
import sys

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt = sys.argv[1]
assert attempt.replace('-', '').isalnum()
out = root / ('native-comparison-' + attempt)
out.mkdir(exist_ok=False)
def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

native_path = root / 'native/result.json'
native = json.loads(native_path.read_text())
assert native['status'] == 'passed' and native['two_native_outputs_byte_identical']
for key in ['original_source', 'owned_source', 'original_queries', 'owned_queries']:
    assert seal(native[key]['path']) == native[key]
for command in native['commands']:
    for key in ['stdout', 'stderr']:
        assert seal(command[key]['path']) == command[key]
    assert command['exit_code'] == 0
queries = json.loads(Path(native['original_queries']['path']).read_text())
observations = json.loads((root / 'native/run1.stdout.log').read_text())
data = root / attempt / 'rust-tests/model'
model = json.loads((data / 'model.json').read_text())
metrics = json.loads((data / 'metrics.json').read_text())
assert (data / 'certificate.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/word-arithmetic-foundation.mpcert').read_bytes()
assert len(queries) == len(observations) == len(model) == 48
assert model == observations
assert metrics['Nat_witness_declarations_executed'] is False
origins = json.loads((root / 'query-origins.json').read_text())
assert seal(origins['original_input']['path']) == origins['original_input']
assert metrics['original_frozen_query_sha256'] == origins['original_input']['sha256']
assert [q['id'] for q in queries] == origins['original_IDs']
oracle, stage_count, carries = [], 0, 0
for query, actual in zip(queries, model, strict=True):
    assert query['id'] == actual['id']
    width = query['width']
    x, y = (int(query[k], 16) for k in ['x_hex', 'y_hex'])
    modulus = 1 << width
    mask = modulus - 1
    assert 0 <= x < modulus and 0 <= y < modulus
    acc, discard, stages = 0, 0, []
    for i in range(width):
        exact_shift = x << i
        left, dropped = exact_shift % modulus, exact_shift // modulus
        flag = (y >> i) & 1
        selected = left if flag else 0
        carry = acc + selected >= modulus
        acc = (acc + selected) % modulus
        discard += flag * dropped + int(carry)
        assert x * (y & ((1 << (i + 1)) - 1)) == acc + modulus * discard
        stages.append({'index': i, 'left_hex': format(left, '0' + str(width // 4) + 'x'),
                       'shift_discard': str(dropped), 'carry': carry,
                       'acc_hex': format(acc, '0' + str(width // 4) + 'x')})
        stage_count += 1
        carries += int(carry)
    product = x * y
    bits = product & mask
    signed = bits - (modulus if bits >= modulus // 2 else 0)
    expected = {'id': query['id'], 'word_hex': format(bits, '0' + str(width // 4) + 'x'),
                'go_signed_result': str(signed), 'unsigned_product': str(product),
                'discarded_quotient': str(product // modulus), 'stages': stages}
    assert discard == product // modulus and expected == actual, (query, expected, actual)
    oracle.append(expected)
assert stage_count == 1440 and carries == metrics['observed_true_carries'] > 0
write(out / 'independent-integer-oracle.json', oracle)
record = {
    'schema': 'mpk.go_practical.word_arithmetic_native_comparison.v1', 'status': 'passed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'native_execution': seal(native_path), 'query_origins': seal(root / 'query-origins.json'),
    'actual_certificate': seal(data / 'certificate.mpcert'),
    'actual_Boolean_certificate_model': seal(data / 'model.json'),
    'model_metrics': seal(data / 'metrics.json'),
    'independent_integer_oracle': seal(out / 'independent-integer-oracle.json'),
    'selected_original_queries': 48, 'actual_fixed_shift_and_product_stages': stage_count,
    'observed_true_carries': carries, 'Nat_witness_declarations_executed': False,
    'scope': 'All actual Left_i/Mul_i/carry circuit bits execute from the same decoded current certificate with Boolean-only primitives. Host arithmetic derives scalar witness recurrence values. These agree with independent Python exact integers and two identical fresh native typed Go multiplication/shifts/addition/math-bits/math-big outputs. Universal Nat witnesses are checked separately in both cores; no arithmetic model callback, whole Go source/program, authoritative immutable application replay or Linux release proof.',
    'CSharp_tests_executed': False, 'public_profile_activated': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
write(out / 'result.json', record)
print(json.dumps({'status': 'passed', 'queries': 48, 'stages': stage_count, 'actual_carries': carries}))
