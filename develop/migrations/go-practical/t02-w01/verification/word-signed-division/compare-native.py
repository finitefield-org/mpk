"""Compare both Bool-only models with native Go and independent integers."""
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

def read(path):
    return json.loads(Path(path).read_text())

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

native_path = root / 'native/result.json'
native = read(native_path)
assert native['status'] == 'passed' and native['two_native_outputs_byte_identical']
assert native['schema'] == 'mpk.go_practical.fresh_signed_division_prerequisite_native_observation.v1'
assert seal(native['annotation_correction']['original_record']['path']) == native['annotation_correction']['original_record']
original_record = read(native['annotation_correction']['original_record']['path'])
unchanged_record = {key: value for key, value in native.items() if key != 'annotation_correction'}
unchanged_record['schema'] = original_record['schema']
assert unchanged_record == original_record
for key in ['original_source', 'owned_source', 'original_queries', 'owned_queries']:
    assert seal(native[key]['path']) == native[key]
for command in native['commands']:
    assert command['exit_code'] == 0
    for key in ['stdout', 'stderr']:
        assert seal(command[key]['path']) == command[key]
queries = read(native['original_queries']['path'])
observations = read(root / 'native/run1.stdout.log')
origins = read(root / 'query-origins.json')
assert seal(origins['original_input']['path']) == origins['original_input']
original = read(origins['original_input']['path'])
assert queries == [q for q in original if q['operation'] in ['Neg', 'SignedDiv', 'SignedRem']]
assert [q['id'] for q in queries] == origins['original_IDs']

components = []
model_by_id = {}
for component, certificate_name, expected_count in [
    (root.parent / 'word-zero', 'word-zero-foundation', 32),
    (root, 'signed-division-prerequisites', 96),
]:
    data = component / attempt / 'rust-tests/model'
    model = read(data / 'model.json')
    metrics = read(data / 'metrics.json')
    assert len(model) == expected_count
    assert metrics['original_frozen_query_sha256'] == origins['original_input']['sha256']
    assert (data / 'certificate.mpcert').read_bytes() == (component / attempt / 'rust-tests/certificates' / (certificate_name + '.mpcert')).read_bytes()
    for row in model:
        assert row['id'] not in model_by_id
        model_by_id[row['id']] = row
    components.append({'certificate': seal(data / 'certificate.mpcert'),
                       'model': seal(data / 'model.json'), 'metrics': seal(data / 'metrics.json')})
assert len(queries) == len(observations) == len(model_by_id) == 128
model = [model_by_id[q['id']] for q in queries]
assert observations == model
zero_metrics = read(components[0]['metrics']['path'])
division_metrics = read(components[1]['metrics']['path'])
assert zero_metrics['Nat_numeric_zero_or_theorem_bodies_executed'] is False
assert division_metrics['Nat_integer_or_theorem_bodies_executed'] is False
assert division_metrics['universal_signed_division_theorem_claimed'] is False

oracle = []
zero_rows = zero_cases = signed_rows = undefined = minimum_minus_one = 0
for query, actual in zip(queries, model, strict=True):
    width = query['width']
    modulus = 1 << width
    half = modulus // 2
    x, y = (int(query[k], 16) for k in ['x_hex', 'y_hex'])
    assert 0 <= x < modulus and 0 <= y < modulus
    signed = lambda u: u - modulus if u >= half else u
    hexadecimal = lambda u: format(u, '0' + str(width // 4) + 'x')
    # Independently inspect every bit, without the model's balanced recurrence.
    all_zero = lambda u: all((u // (1 << bit)) % 2 == 0 for bit in range(width))
    if query['operation'] == 'Neg':
        zero_rows += 1
        zero_cases += int(x == 0)
        expected = {'kind': 'zero', 'id': query['id'], 'word_hex': hexadecimal(x),
                    'is_zero': x == 0, 'divide_defined': x != 0,
                    'zero_bits': all_zero(x), 'source_sign': signed(x) < 0}
    else:
        signed_rows += 1
        defined = y != 0
        result = None
        value = None
        if defined:
            # Python // rounds down; compute truncation toward zero explicitly.
            quotient = abs(signed(x)) // abs(signed(y))
            if (signed(x) < 0) != (signed(y) < 0):
                quotient = -quotient
            remainder = signed(x) - signed(y) * quotient
            assert abs(remainder) < abs(signed(y))
            assert remainder == 0 or (remainder < 0) == (signed(x) < 0)
            unwrapped = quotient if query['operation'] == 'SignedDiv' else remainder
            bits = unwrapped % modulus
            value, result = hexadecimal(bits), str(signed(bits))
            if signed(x) == -half and signed(y) == -1:
                minimum_minus_one += 1
                assert bits == (half if query['operation'] == 'SignedDiv' else 0)
        else:
            undefined += 1
        expected = {'kind': 'arithmetic', 'id': query['id'], 'operation': query['operation'],
                    'defined': defined, 'panicked': not defined, 'value_hex': value,
                    'go_signed_result': result, 'x_zero': x == 0, 'y_zero': y == 0,
                    'x_defined': x != 0, 'y_defined': y != 0,
                    'x_zero_bits': all_zero(x), 'y_zero_bits': all_zero(y),
                    'x_sign': signed(x) < 0, 'y_sign': signed(y) < 0,
                    'divisor_magnitude_zero': abs(signed(y)) == 0}
    assert expected == actual, (query, expected, actual)
    oracle.append(expected)
assert zero_rows == 32 and zero_cases == 4 and signed_rows == 96
assert undefined == division_metrics['source_zero_divisor_faults'] == 8
assert minimum_minus_one == division_metrics['minimum_minus_one_observations'] == 8
write(out / 'independent-integer-oracle.json', oracle)
record = {
    'schema': 'mpk.go_practical.zero_and_signed_division_prerequisites_native_comparison.v1',
    'status': 'passed', 'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'native_execution': seal(native_path), 'query_origins': seal(root / 'query-origins.json'),
    'components': components, 'independent_integer_oracle': seal(out / 'independent-integer-oracle.json'),
    'original_queries': 128, 'zero_queries': zero_rows, 'zero_cases': zero_cases,
    'signed_queries': signed_rows, 'defined_signed_results': 88,
    'source_zero_divisor_faults': undefined, 'minimum_minus_one_observations': minimum_minus_one,
    'two_fresh_native_outputs_identical': True, 'prepared_image_files': 5577,
    'Nat_integer_or_theorem_bodies_executed': False,
    'scope': 'Finite observations of the actual decoded operators and direct/scoped Boolean zero predicates. Source zero-divisor guards compare to actual recovered Go faults; minimum/-one follows normal Go wrapping. Universal signed division/remainder and source/application proofs remain required.',
    'CSharp_tests_executed': False, 'T02_W01_complete': False,
    'public_profile_activated': False, 'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
write(out / 'result.json', record)
print(json.dumps({'status': 'passed', 'original_queries': 128, 'zero_queries': zero_rows,
                  'signed_queries': signed_rows, 'source_faults': undefined,
                  'minimum_minus_one': minimum_minus_one}))
