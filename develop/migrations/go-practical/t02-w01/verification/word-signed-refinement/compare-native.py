"""Exact actual Bool observations, typed native Go and independent integers."""
from pathlib import Path
import collections
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
assert native['native_rows'] == 160
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
assert queries == [q for q in read(origins['original_input']['path'])
                   if q['operation'] in ['Neg', 'SignedDiv', 'SignedRem']]
assert [q['id'] for q in queries] == origins['original_IDs']
data = root / attempt / 'rust-tests/model'
model = read(data / 'model.json')
metrics = read(data / 'metrics.json')
assert metrics['original_frozen_query_sha256'] == origins['original_input']['sha256']
assert metrics['Nat_Int_correction_or_theorem_bodies_executed'] is False
assert (data / 'certificate.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/word-signed-refinement.mpcert').read_bytes()
assert len(queries) == 128 and len(model) == len(observations) == 160 and model == observations
by_key = {(row['id'], row['flag']): row for row in model}
assert len(by_key) == 160

oracle = []
generic_corrections = collections.Counter()
source_corrections = collections.Counter()
source_faults = source_minima = 0
for query in queries:
    width = query['width']
    modulus, half = 1 << width, 1 << (width - 1)
    x, y = (int(query[key], 16) for key in ['x_hex', 'y_hex'])
    assert 0 <= x < modulus and 0 <= y < modulus
    signed = lambda value: value - modulus if value >= half else value
    hexadecimal = lambda value: format(value, '0' + str(width // 4) + 'x')
    generic = query['operation'] == 'Neg'
    flags = [False, True] if generic else [
        (signed(x) < 0) != (signed(y) < 0) if query['operation'] == 'SignedDiv' else signed(x) < 0]
    defined = generic or y != 0
    for flag in flags:
        expected = {'kind': 'selection' if generic else 'arithmetic', 'id': query['id'],
                    'operation': query['operation'], 'flag': flag,
                    'defined': defined, 'panicked': not defined}
        values = dict.fromkeys(['amount_hex', 'negated_amount_hex', 'result_hex',
                               'mathematical_result', 'go_signed_result',
                               'correction_positive', 'correction_negative', 'correction'])
        if defined:
            if generic:
                amount = x
            elif query['operation'] == 'SignedDiv':
                amount = abs(signed(x)) // abs(signed(y))
            else:
                quotient = abs(signed(x)) // abs(signed(y))
                if (signed(x) < 0) != (signed(y) < 0):
                    quotient = -quotient
                remainder = signed(x) - signed(y) * quotient
                assert abs(remainder) < abs(signed(y))
                amount = abs(remainder)
            negated = (-amount) % modulus
            mathematical = -amount if flag else amount
            result = mathematical % modulus
            positive = int(negated >= half) if flag else int(amount >= half)
            negative = int(flag and amount != 0)
            correction = positive - negative
            assert mathematical == signed(result) + modulus * correction
            values.update(amount_hex=hexadecimal(amount), negated_amount_hex=hexadecimal(negated),
                          result_hex=hexadecimal(result), mathematical_result=str(mathematical),
                          go_signed_result=str(signed(result)), correction_positive=positive,
                          correction_negative=negative, correction=correction)
            if generic:
                generic_corrections[correction] += 1
            else:
                source_corrections[correction] += 1
                exceptional = query['operation'] == 'SignedDiv' and signed(x) == -half and signed(y) == -1
                assert (correction == 1) == exceptional
                assert correction in [0, 1]
                source_minima += int(signed(x) == -half and signed(y) == -1)
        else:
            source_faults += 1
        expected.update(values)
        assert expected == by_key[(query['id'], flag)], (query, expected, by_key[(query['id'], flag)])
        oracle.append(expected)
assert oracle == model
assert source_faults == 8 and source_minima == 8
assert source_corrections == {0: 84, 1: 4}
assert set(generic_corrections) == {-1, 0, 1} and sum(generic_corrections.values()) == 64
write(out / 'independent-integer-oracle.json', oracle)
record = {
    'schema': 'mpk.go_practical.actual_signed_refinement_native_comparison.v1', 'status': 'passed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'native_execution': seal(native_path), 'query_origins': seal(root / 'query-origins.json'),
    'certificate': seal(data / 'certificate.mpcert'), 'actual_Bool_model': seal(data / 'model.json'),
    'model_metrics': seal(data / 'metrics.json'), 'independent_integer_oracle': seal(out / 'independent-integer-oracle.json'),
    'original_queries': 128, 'observations': 160, 'generic_sign_choices': 64,
    'source_operator_queries': 96, 'defined_source_results': 88, 'source_zero_divisor_faults': 8,
    'minimum_minus_one_source_cases': 8, 'generic_corrections': dict(generic_corrections),
    'source_corrections': dict(source_corrections), 'prepared_image_files': 5577,
    'two_fresh_native_outputs_identical': True, 'Nat_Int_correction_or_theorem_bodies_executed': False,
    'scope': 'Actual source words/sign-selection/magnitude quotient and remainder/negated amounts/Boolean guards agree with independent typed Go and integers. Host derives small correction parts from actual decoded flags; native math/big and Python arithmetic validate exact correction equations. The minimum/-one and zero correction classification observed here is finite; its universal proof remains required, as do source/application/foundation release gates.',
    'CSharp_tests_executed': False, 'T02_W01_complete': False, 'public_profile_activated': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
write(out / 'result.json', record)
print(json.dumps({key: record[key] for key in ['status', 'original_queries', 'observations',
                                             'generic_corrections', 'source_corrections']}))
