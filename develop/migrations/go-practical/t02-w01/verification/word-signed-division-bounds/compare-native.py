"""Actual Bool source results, original native outputs and independent integers."""
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
assert native['status'] == 'passed' and native['native_rows'] == 96 and native['two_native_outputs_byte_identical']
recovery = native['metadata_recovery']
assert recovery['original_wrapper_exit_code'] == 1
assert recovery['native_build_or_observation_repeated'] is False
assert recovery['original_timing_fields_recreated'] is False
for key in ['original_runner', 'observed_tool_failure_text', 'fixed_runner', 'recovery_script']:
    assert seal(recovery[key]['path']) == recovery[key]
assert native['complete_original_image_checks'][0]['status'] == 'passed_by_reached_control_flow'
assert native['complete_original_image_checks'][1]['status'] == 'passed'
for key in ['original_source', 'owned_source', 'original_queries', 'owned_queries']:
    assert seal(native[key]['path']) == native[key]
for command in native['commands']:
    assert command['exit_code'] == 0 and command['seconds'] is None
    for key in ['stdout', 'stderr']:
        assert seal(command[key]['path']) == command[key]
queries = read(native['original_queries']['path'])
origins = read(root / 'query-origins.json')
assert seal(origins['original_input']['path']) == origins['original_input']
assert queries == [q for q in read(origins['original_input']['path']) if q['operation'] in ['SignedDiv', 'SignedRem']]
assert [q['id'] for q in queries] == origins['original_IDs']
data = root / attempt / 'rust-tests/model'
model, metrics = read(data / 'model.json'), read(data / 'metrics.json')
observations = read(root / 'native/run1.stdout.log')
assert len(queries) == len(model) == len(observations) == 96 and observations == model
assert metrics['Nat_gap_half_predicate_Int_or_theorem_bodies_executed'] is False
assert metrics['original_frozen_query_sha256'] == origins['original_input']['sha256']
assert (data / 'certificate.mpcert').read_bytes() == (root / attempt / 'rust-tests/certificates/word-division-bounds.mpcert').read_bytes()
oracle = []
faults = minima = exact = units = 0
for query, actual in zip(queries, model, strict=True):
    width = query['width']
    modulus, half = 1 << width, 1 << (width - 1)
    x, y = (int(query[key], 16) for key in ['x_hex', 'y_hex'])
    assert 0 <= x < modulus and 0 <= y < modulus
    signed = lambda value: value - modulus if value >= half else value
    hexadecimal = lambda value: format(value, '0' + str(width // 4) + 'x')
    defined = y != 0
    expected = {'id': query['id'], 'operation': query['operation'], 'defined': defined, 'panicked': not defined}
    values = dict.fromkeys(['abs_x_hex', 'abs_y_hex', 'q_amount_hex', 'r_amount_hex', 'quotient_hex', 'remainder_hex',
                           'signed_x', 'signed_y', 'signed_quotient', 'signed_remainder', 'half', 'modulus',
                           'input_gap_x', 'input_gap_y', 'quotient_gap', 'quotient_bounded', 'remainder_strict_half',
                           'negative_unit', 'quotient_overflow', 'minimum_minus_one', 'quotient_no_overflow',
                           'quotient_correction', 'remainder_correction', 'corrected_conservation', 'exact_conservation', 'minimum_results'])
    if not defined:
        faults += 1
    else:
        sx, sy = signed(x), signed(y)
        ax, ay = abs(sx), abs(sy)
        qa = ax // ay
        qm = -qa if (sx < 0) != (sy < 0) else qa
        rm = sx - sy * qm
        ra = abs(rm)
        sq, sr = signed(qm % modulus), signed(rm % modulus)
        assert (qm - sq) % modulus == (rm - sr) % modulus == 0
        qc, rc = (qm - sq) // modulus, (rm - sr) // modulus
        negative_unit = sy == -1
        minimum = sx == -half and negative_unit
        overflow = qc == 1
        assert qc == int(minimum) and rc == 0
        assert qa <= half and ra < ay <= half
        # Direct subtraction is independent of the model/native product gap.
        gap = half - qa
        corrected = sx == sy * (sq + modulus * qc) + sr
        is_exact = sx == sy * sq + sr
        assert corrected and is_exact == (not minimum) and overflow == minimum
        minima += int(minimum)
        exact += int(is_exact)
        units += int(negative_unit)
        values.update(abs_x_hex=hexadecimal(ax), abs_y_hex=hexadecimal(ay),
                      q_amount_hex=hexadecimal(qa), r_amount_hex=hexadecimal(ra),
                      quotient_hex=hexadecimal(qm % modulus), remainder_hex=hexadecimal(rm % modulus),
                      signed_x=str(sx), signed_y=str(sy), signed_quotient=str(sq), signed_remainder=str(sr),
                      half=str(half), modulus=str(modulus), input_gap_x=str(half - ax), input_gap_y=str(half - ay),
                      quotient_gap=str(gap), quotient_bounded=qa <= half, remainder_strict_half=ra < half,
                      negative_unit=negative_unit, quotient_overflow=overflow, minimum_minus_one=minimum,
                      quotient_no_overflow=not minimum, quotient_correction=qc, remainder_correction=rc,
                      corrected_conservation=corrected, exact_conservation=is_exact,
                      minimum_results=minimum and qm % modulus == half and sr == 0)
    expected.update(values)
    assert expected == actual, (query, expected, actual)
    oracle.append(expected)
assert faults == minima == 8 and exact == 80
assert metrics['minimum_minus_one_pairs'] == minima and metrics['exact_source_reconstructions'] == exact
assert metrics['negative_unit_divisors'] == units and metrics['actual_source_operator_results'] == 176
write(out / 'independent-integer-oracle.json', oracle)
record = {
    'schema': 'mpk.go_practical.signed_division_bounds_native_comparison.v1', 'status': 'passed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'native_execution': seal(native_path), 'query_origins': seal(root / 'query-origins.json'),
    'certificate': seal(data / 'certificate.mpcert'), 'actual_Bool_model': seal(data / 'model.json'),
    'model_metrics': seal(data / 'metrics.json'), 'independent_integer_oracle': seal(out / 'independent-integer-oracle.json'),
    'original_queries': 96, 'defined_pairs': 88, 'actual_source_operator_results': 176,
    'source_zero_divisor_faults': 8, 'minimum_minus_one_pairs': 8, 'negative_unit_divisors': units,
    'exact_source_reconstructions': 80, 'corrected_source_reconstructions': 88,
    'prepared_image_files': 5577, 'two_native_outputs_identical': True,
    'native_count_wrapper_metadata_recovery_qualified': True, 'original_native_timing_fields_recreated': False,
    'native_build_or_observation_repeated': False,
    'Nat_gap_half_predicate_Int_or_theorem_bodies_executed': False,
    'scope': 'Finite counterpart observations of both actual source operators and Bool magnitude/Neg/Top/guards. Python uses signed truncation/conservation, correction by exact signed difference divided by modulus and direct H-minus-quotient gap; native uses typed Go and math-big product gaps. All original full rows match. The ordinary certificate separately proves the universal bounds/correction/minimum classification; these finite observations do not establish it or a whole source/application proof. Native metadata recovery preserves original outputs and qualifies lost timing/precheck provenance without repeating native work.',
    'CSharp_tests_executed': False, 'T02_W01_complete': False, 'public_profile_activated': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
write(out / 'result.json', record)
print(json.dumps({key: record[key] for key in ['status', 'original_queries', 'actual_source_operator_results',
                                             'source_zero_divisor_faults', 'minimum_minus_one_pairs', 'exact_source_reconstructions']}))
