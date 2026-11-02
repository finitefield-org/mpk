"""Compare actual Bool circuits, native Go and independent exact integers."""
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
laws = json.loads((data / 'laws.json').read_text())
metrics = json.loads((data / 'metrics.json').read_text())
assert (data / 'certificate.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/word-order-foundation.mpcert').read_bytes()
assert len(queries) == len(observations) == len(model) == len(laws) == 272
assert model == observations
assert metrics['Nat_denotation_order_or_theorem_bodies_executed'] is False
origins = json.loads((root / 'query-origins.json').read_text())
assert seal(origins['original_input']['path']) == origins['original_input']
assert metrics['original_frozen_query_sha256'] == origins['original_input']['sha256']
assert [q['id'] for q in queries] == origins['original_IDs']
all_original = json.loads(Path(origins['original_input']['path']).read_text())
assert queries == [q for q in all_original if q['operation'] in origins['operations']]
oracle, law_oracle, zero_divisors, borrows = [], [], 0, 0
for query, actual, actual_laws in zip(queries, model, laws, strict=True):
    assert query['id'] == actual['id'] == actual_laws['id']
    width = query['width']
    x, y = (int(query[k], 16) for k in ['x_hex', 'y_hex'])
    modulus = 1 << width
    mask = modulus - 1
    assert 0 <= x < modulus and 0 <= y < modulus
    def hexword(x):
        return format(x, '0' + str(width // 4) + 'x')
    nx, ny, sub = mask - x, mask - y, (x - y) % modulus
    carry, reverse = x >= y, y >= x
    assert x + nx + 1 == y + ny + 1 == modulus
    assert x + modulus == sub + y + modulus * int(carry)
    assert sub < modulus and ((carry and reverse) == (x == y))
    expected_laws = {'id': query['id'], 'x_complement_hex': hexword(nx),
                     'y_complement_hex': hexword(ny), 'sub_hex': hexword(sub),
                     'carry': carry, 'reverse_carry': reverse, 'carry_pair': carry and reverse,
                     'less': x < y, 'less_equal': x <= y, 'equal': x == y,
                     'not_equal': x != y, 'zero_divisor': y == 0, 'divide_defined': y != 0}
    assert expected_laws == actual_laws, (query, expected_laws, actual_laws)
    operation = query['operation']
    expected = {'id': query['id']}
    if operation in ['Not', 'Sub']:
        expected.update(kind='word', value_hex=hexword(nx if operation == 'Not' else sub))
    else:
        value = {'Equal': x == y, 'NotEqual': x != y,
                 'UnsignedLess': x < y, 'UnsignedLessEqual': x <= y}[operation]
        expected.update(kind='boolean', value=value)
    assert expected == actual, (query, expected, actual)
    zero_divisors += int(y == 0)
    borrows += int(not carry)
    oracle.append(expected)
    law_oracle.append(expected_laws)
assert zero_divisors == metrics['observed_zero_divisors'] > 0
assert borrows == metrics['observed_borrows'] > 0
assert sum(o['kind'] == 'word' for o in oracle) == 80
assert sum(o['kind'] == 'boolean' for o in oracle) == 192
write(out / 'independent-integer-oracle.json', oracle)
write(out / 'independent-law-oracle.json', law_oracle)
record = {
    'schema': 'mpk.go_practical.word_order_native_comparison.v1', 'status': 'passed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'native_execution': seal(native_path), 'query_origins': seal(root / 'query-origins.json'),
    'actual_certificate': seal(data / 'certificate.mpcert'),
    'actual_Boolean_certificate_model': seal(data / 'model.json'),
    'actual_Boolean_law_observations': seal(data / 'laws.json'),
    'model_metrics': seal(data / 'metrics.json'),
    'independent_integer_oracle': seal(out / 'independent-integer-oracle.json'),
    'independent_law_oracle': seal(out / 'independent-law-oracle.json'),
    'selected_original_queries': 272, 'word_results': 80, 'boolean_results': 192,
    'observed_zero_divisors': zero_divisors, 'observed_borrows': borrows,
    'scope': 'Actual complement/subtraction/equality/comparison results from the decoded Bool circuit agree with two identical fresh typed native Go outputs and independent Python exact integers. Additional actual forward/reverse carry, IsZero and DivideDefined bits agree with independent host integer meanings; the unchanged native observer does not expose those internal guards. No unary Nat denotation/order/theorem body is executed by the Bool-only model. Universal genuine Nat laws are checked separately by both cores. No source-program, authoritative immutable application replay or native Linux release claim.',
    'CSharp_tests_executed': False, 'public_profile_activated': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
write(out / 'result.json', record)
print(json.dumps({'status': 'passed', 'queries': 272, 'words': 80, 'booleans': 192,
                  'zero_divisors': zero_divisors, 'borrows': borrows}))
