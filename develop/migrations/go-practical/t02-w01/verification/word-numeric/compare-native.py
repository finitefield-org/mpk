"""Compare actual Boolean circuit execution, native Go and exact Python integers."""
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
for name in ['original_source', 'owned_source', 'original_queries', 'owned_queries']:
    assert seal(native[name]['path']) == native[name]
for command in native['commands']:
    assert command['exit_code'] == 0
    for name in ['stdout', 'stderr']:
        assert seal(command[name]['path']) == command[name]
queries = json.loads(Path(native['original_queries']['path']).read_text())
observations = json.loads((root / 'native/run1.stdout.log').read_text())
data = root / attempt / 'rust-tests/model'
model = json.loads((data / 'model.json').read_text())
metrics = json.loads((data / 'metrics.json').read_text())
assert (data / 'certificate.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/word-numeric-foundation.mpcert').read_bytes()
assert metrics['query_sha256'] == native['original_queries']['sha256']
assert metrics['Nat_denotation_terms_executed'] is False
assert len(queries) == len(observations) == len(model) == 288
oracle, derived = [], []
safe = 0
for query, actual, observed in zip(queries, model, observations, strict=True):
    assert query['id'] == actual['id'] == observed['id']
    assert all(observed[name] == value for name, value in actual.items()), (query, actual, observed)
    width = query['width']
    x, y = (int(query[name], 16) for name in ['x_hex', 'y_hex'])
    power = 1 << width
    half = power // 2
    incoming = int(query['incoming'])
    initial = int(query['initial_nat'])
    assert initial >= 0 and 0 <= x < power and 0 <= y < power
    total = x + y + incoming
    result = total % power
    carry = total >= power
    sign_incoming = (x % half + y % half + incoming) >= half
    sign = lambda value: power if value & half else 0
    def describe(bits, carry, into, guard):
        return {
            'id': query['id'], 'word_hex': format(bits, '0' + str(width // 4) + 'x'),
            'carry': carry, 'sign_incoming': into, 'no_overflow': guard,
            'go_signed_result': str(bits - sign(bits)),
            'unsigned_left': str(initial + x + y + incoming),
            'unsigned_right': str(initial + bits + power * int(carry)),
            'signed_cross_left': str(initial + x + y + incoming + sign(bits)),
            'signed_cross_right': str(initial + bits + sign(x) + sign(y)),
            'integer_sum': str(x - sign(x) + y - sign(y) + incoming),
        }
    expected = describe(result, carry, sign_incoming, carry == sign_incoming)
    actual_derived = describe(int(actual['word_hex'], 16), actual['carry'], actual['sign_incoming'], actual['no_overflow'])
    assert expected == actual_derived == observed, (query, expected, actual_derived, observed)
    assert expected['unsigned_left'] == expected['unsigned_right']
    signed_valid = expected['signed_cross_left'] == expected['signed_cross_right']
    assert signed_valid == expected['no_overflow']
    assert signed_valid == (-half <= int(expected['integer_sum']) < half)
    safe += int(signed_valid)
    oracle.append(expected)
    derived.append(actual_derived)
assert safe == 210
write(out / 'independent-integer-oracle.json', oracle)
write(out / 'circuit-derived-integer-observations.json', derived)
record = {
    'schema': 'mpk.go_practical.word_numeric_native_comparison.v1', 'status': 'passed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'native_execution': seal(native_path), 'actual_certificate': seal(data / 'certificate.mpcert'),
    'actual_decoded_certificate_Boolean_model': seal(data / 'model.json'),
    'model_metrics': seal(data / 'metrics.json'),
    'independent_integer_oracle': seal(out / 'independent-integer-oracle.json'),
    'circuit_derived_integer_observations': seal(out / 'circuit-derived-integer-observations.json'),
    'rows': 288, 'signed_no_overflow': 210, 'signed_overflow': 78,
    'Nat_denotation_terms_executed': False,
    'scope': 'The actual ordinary Boolean circuits/guards execute from the current decoded certificate. Python derives the displayed exact scalar sums from those bits, carries and original decimal initial naturals; these are finite mathematical observations, not execution of2^64 unary successors. They agree with independent Python arithmetic and two identical fresh pinned native Go/math-bits/math-big byte streams. Universal Nat/Int laws are checked separately in both cores; no Go source/program, immutable application replay or Linux release claim.',
    'CSharp_tests_executed': False, 'public_profile_activated': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
write(out / 'result.json', record)
print(json.dumps({'status': 'passed', 'rows': 288, 'signed_safe': 210, 'overflow': 78}))
