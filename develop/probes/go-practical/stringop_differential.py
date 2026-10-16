"""Original-Go substring/concat against full64 arithmetic and byte-loop models."""
import json
import random

from byte_differential import less_word
from order_differential import adder
from run_proof_pilot import json_bytes, sha, timed

CAPACITIES = (0, 1, 31, 32, 33, 255, 256, 257)
ARTIFACTS = ('stringop-input.json', 'stringop-output.json')
MASK = (1 << 64) - 1
SCOPE = 'actual Go substring/concat, independent full64 bit arithmetic and arbitrary-byte loops; empirical observations, no production source/effect binding or process OOM theorem'
WITNESS_SCOPE = 'bounded-source, source-substring-panic and source-concat-exceeds-profile-without-panic witnesses remain distinct; checker resource failures never establish false claims'

def depth(n): return max(n - 1, 0).bit_length()
def padding(n): return 1 << depth(n)
def families(n):
    result = ['substring-nonnegative', 'substring-length', 'concat-carry', 'concat-length', 'concat-nonnegative']
    if n:
        result += ['substring-index', 'substring-coverage', 'substring-read', 'substring-canonical', 'concat-left-index', 'concat-right-index', 'concat-left-coverage', 'concat-right-coverage', 'concat-left-read', 'concat-right-read', 'concat-left-canonical', 'concat-right-canonical']
    return result

def label(n, f):
    if f == 'substring-length': return 'erased-range-guard'
    if f == 'concat-length' and n: return 'erased-output-bound'
    if f.endswith('index'): return 'false-defined-index'
    if f.endswith('coverage'): return 'flipped-word'
    if f.endswith('read') or f.endswith('canonical'): return 'flipped-byte'
    return 'flipped-boolean'

def make_row(n, a=b'', b=b'', lo=0, hi=0, index=0):
    def backing(value, salt):
        size = max(padding(n), len(value))
        return (value + bytes((j * 73 + salt) & 255 for j in range(len(value), size))).hex()
    return dict(bound=n, left=backing(a, 191), right=backing(b, 239), left_length=len(a), right_length=len(b), lower=str(lo), upper=str(hi), index=str(index))

def inputs():
    rng = random.Random(0x4D504B535452)
    rows = []
    for n in CAPACITIES:
        examples = [b'', b'\x00', b'\xff', b'\x80', b'a\x00x', b'\xc3\xa9', b'\xe3\x81\x82', b'\xc3(', bytes(rng.randrange(256) for _ in range(n)), b'\xff' * n]
        examples = list(dict.fromkeys(a for a in examples if len(a) <= n))
        for k, a in enumerate(examples):
            b = examples[(k + 3) % len(examples)]
            ranges = [(0, 0), (0, len(a)), (len(a), len(a)), (len(a) // 2, len(a)), (1, len(a)), (-1, 0), (0, len(a) + 1), (1, 0), (-(1 << 63), 0), (0, (1 << 63) - 1)]
            for lo, hi in dict.fromkeys(ranges):
                for i in dict.fromkeys((-1, 0, max(hi - lo - 1, 0), hi - lo)):
                    # Keep every source index in the actual signed64 domain.
                    if -(1 << 63) <= i < (1 << 63): rows.append(make_row(n, a, b, lo, hi, i))
            for i in (-(1 << 63), (1 << 63) - 1, n, padding(n)):
                rows.append(make_row(n, a, b, 0, len(a), i))
        rows += [make_row(n), make_row(n, lo=-(n + 1)), make_row(n, b'\x00' * (n + 1), hi=n + 1, index=n)]
        if n:
            rows += [make_row(n, b'\x00', hi=1), make_row(n, b=b'\x00'), make_row(n, b'\x00' * n, b'\xff', hi=n), make_row(n, b'\xff' * n, hi=n, index=n - 1)]
    return list({json.dumps(r, sort_keys=True): r for r in rows}.values())

def unsigned_add(a, b): return adder(64, a & MASK, b & MASK, False)[:2]
def unsigned_sub(a, b): return adder(64, a & MASK, (b & MASK) ^ MASK, True)[0]
def signed(word): return word - (1 << 64) if word >> 63 else word
def signed_le(a, b):
    x, y = a & MASK, b & MASK
    return not less_word(64, y, x) if (x >> 63) == (y >> 63) else bool(x >> 63)
def defined(i, length): return i >= 0 and less_word(64, i & MASK, length)
def byte_read(value, i): return (True, value[i]) if defined(i, len(value)) else (False, -1)

def model(r):
    left, right = bytes.fromhex(r['left']), bytes.fromhex(r['right'])
    a = [left[i] for i in range(r['left_length'])]
    b = [right[i] for i in range(r['right_length'])]
    lo, hi, i = (int(r[key]) for key in ('lower', 'upper', 'index'))
    guard = lo >= 0 and signed_le(lo, hi) and signed_le(hi, len(a))
    length, mapped = unsigned_sub(hi, lo), unsigned_add(lo, i)[0]
    result = [a[j] for j in range(lo, hi)] if guard else []
    read, value = byte_read(result, i) if guard else (False, -1)
    combined = [a[j] for j in range(len(a))] + [b[j] for j in range(len(b))]
    total, carry = unsigned_add(len(a), len(b)); right_mapped = unsigned_add(len(a), i)[0]
    cl, vl = byte_read(combined, i); cr, vr = byte_read(combined, signed(right_mapped))
    return dict(substring_guard=guard, substring_done=guard, substring_panicked=not guard,
                substring_bytes=bytes(result).hex(), substring_length=str(length) if guard else '',
                substring_read=read, substring_read_panicked=guard and not read, substring_value=value,
                substring_tail_same=guard, raw_substring_length=str(length), raw_substring_mapped=str(mapped),
                substring_source_read=defined(signed(mapped), len(a)), concat_bytes=bytes(combined).hex(), concat_length=str(total),
                concat_bound=not less_word(64, r['bound'], total), concat_carry=carry, concat_tail_same=True,
                concat_left_read=cl, concat_left_value=vl, concat_right_read=cr, concat_right_value=vr, concat_right_mapped=str(right_mapped),
                left_source_read=defined(i, len(a)), right_source_read=defined(i, len(b)),
                input_bound=len(a) <= r['bound'] and len(b) <= r['bound'], input_unchanged=True, query_covered=(i & MASK) == ((i & MASK) & (padding(r['bound']) - 1)))

def compare(cases, outputs):
    if cases != inputs() or len(cases) != len(outputs): raise ValueError('string operation input coverage')
    counts = dict.fromkeys(('substring_operations', 'substring_panics', 'substring_empty', 'substring_defined_reads', 'substring_read_panics', 'substring_tail_preservation', 'concat_operations', 'concat_empty', 'concat_exceeds_bound_without_panic', 'concat_left_reads', 'concat_right_reads', 'concat_tail_preservation', 'invalid_utf8_substrings', 'nul_bytes', 'source_outside_bound_reads', 'uncovered_source_reads', 'negative_extreme_endpoints', 'positive_extreme_endpoints'), 0)
    for r, a in zip(cases, outputs):
        expected = model(r)
        if not isinstance(a, dict) or set(a) != set(expected) or any(type(a[k]) is not type(v) for k, v in expected.items()): raise ValueError('string operation observation field/type mismatch')
        if a != expected: raise ValueError('Go/bit-loop string operation mismatch')
        bound = r['bound']; lo, hi, i = (int(r[k]) for k in ('lower', 'upper', 'index'))
        if a['substring_done'] and a['input_bound']:
            if int(a['substring_length']) > bound: raise ValueError('substring length bound failed')
            if a['substring_read']:
                mapped = int(a['raw_substring_mapped'])
                if not a['substring_source_read'] or mapped >= r['left_length'] or mapped != lo + i: raise ValueError('substring source-index lifting failed')
                if a['substring_value'] != bytes.fromhex(r['left'])[mapped]: raise ValueError('substring full-byte relation failed')
        if a['input_bound'] and a['concat_bound']:
            if a['left_source_read'] and (not a['concat_left_read'] or a['concat_left_value'] != bytes.fromhex(r['left'])[i]): raise ValueError('concat left lifting/read failed')
            if a['right_source_read'] and (not a['concat_right_read'] or a['concat_right_value'] != bytes.fromhex(r['right'])[i]): raise ValueError('concat right lifting/read failed')
        counts['substring_operations'] += a['substring_done']; counts['substring_panics'] += a['substring_panicked']; counts['substring_empty'] += a['substring_done'] and a['substring_length'] == '0'
        counts['substring_defined_reads'] += a['substring_read']; counts['substring_read_panics'] += a['substring_read_panicked']; counts['substring_tail_preservation'] += a['substring_tail_same']
        counts['concat_operations'] += 1; counts['concat_empty'] += a['concat_length'] == '0'; counts['concat_exceeds_bound_without_panic'] += a['input_bound'] and not a['concat_bound']
        counts['concat_left_reads'] += a['left_source_read'] and a['concat_left_read']; counts['concat_right_reads'] += a['right_source_read'] and a['concat_right_read']; counts['concat_tail_preservation'] += a['concat_tail_same']
        try: bytes.fromhex(a['substring_bytes']).decode('utf-8')
        except UnicodeDecodeError: counts['invalid_utf8_substrings'] += 1
        counts['nul_bytes'] += 0 in bytes.fromhex(a['concat_bytes'])
        counts['source_outside_bound_reads'] += not a['input_bound'] and a['substring_read']; counts['uncovered_source_reads'] += not a['query_covered'] and a['substring_read']
        counts['negative_extreme_endpoints'] += lo == -(1 << 63); counts['positive_extreme_endpoints'] += hi == (1 << 63) - 1
    if not all(counts.values()): raise ValueError('lost string operation boundary coverage: ' + repr([k for k, v in counts.items() if not v]))
    return counts

def negative_witnesses(cases, outputs):
    result = []
    for n in CAPACITIES:
        for family in families(n):
            domain = 'bounded_source'
            if family == 'substring-length':
                r = make_row(n, lo=-(n + 1)); domain = 'source_substring_panic'; actual, claimed = False, True
            elif family == 'concat-length' and n:
                r = make_row(n, b'\x00' * n, b'\xff', hi=n); domain = 'source_concat_exceeds_profile_without_panic'; actual, claimed = False, True
            elif family.startswith('substring-') and family not in ('substring-nonnegative', 'substring-length'):
                r = make_row(n, b'\x00', hi=1)
                actual, claimed = (0, 255) if family.endswith(('read', 'canonical')) else ((0, MASK) if family.endswith('coverage') else (True, False))
            elif family.startswith('concat-left-') or family.startswith('concat-right-'):
                right = family.startswith('concat-right-'); r = make_row(n, b'' if right else b'\x00', b'\x00' if right else b'', hi=0 if right else 1)
                actual, claimed = (0, 255) if family.endswith(('read', 'canonical')) else ((0, MASK) if family.endswith('coverage') else (True, False))
            else:
                r = make_row(n); actual, claimed = (True, False) if family == 'concat-length' else (False, True)
            index = cases.index(r); a = outputs[index]; lo, hi, i = (int(r[k]) for k in ('lower', 'upper', 'index'))
            if not a['input_bound']: raise ValueError('false witness violates retained input bound')
            if domain == 'source_substring_panic':
                if not a['substring_panicked'] or int(a['raw_substring_length']) <= n: raise ValueError('erased range witness is not false')
            elif domain == 'source_concat_exceeds_profile_without_panic':
                if a['concat_bound'] or int(a['concat_length']) != n + 1: raise ValueError('erased output-bound witness is not false')
            elif family.startswith('substring-'):
                if not a['substring_done']: raise ValueError('bounded substring witness panics')
                if family not in ('substring-nonnegative',) and not a['substring_read']: raise ValueError('vacuous substring witness')
                if family.endswith(('read', 'canonical')) and a['substring_value'] != actual: raise ValueError('substring byte witness mismatch')
            elif family.startswith(('concat-left-', 'concat-right-')):
                side = 'right' if family.startswith('concat-right-') else 'left'
                if not a['concat_bound'] or not a[f'{side}_source_read'] or not a[f'concat_{side}_read']: raise ValueError('vacuous concat witness')
                if family.endswith(('read', 'canonical')) and a[f'concat_{side}_value'] != actual: raise ValueError('concat byte witness mismatch')
            if type(actual) is not type(claimed) or actual == claimed: raise ValueError('false witness type/value')
            result.append(dict(case_id=f'stringop-{n}-{family}-{label(n, family)}', observation_index=index, domain=domain, actual=actual, claimed=claimed))
    return result

def run(executable, directory):
    cases = inputs(); path = directory / ARTIFACTS[0]; path.write_bytes(json_bytes(cases)); measured = timed([executable, path])
    if measured['exit_code']: raise RuntimeError('actual Go string operation observer failed')
    raw = measured.pop('stdout').encode(); (directory / ARTIFACTS[1]).write_bytes(raw); outputs = json.loads(raw)
    counts = compare(cases, outputs)
    return dict(scope=SCOPE, cases=len(cases), outcomes=counts, input_sha256=sha(path.read_bytes()), output_sha256=sha(raw), go_binary_sha256=sha(executable.read_bytes()), go_measurement=measured, witness_scope=WITNESS_SCOPE, false_claim_witnesses=negative_witnesses(cases, outputs))

def check(report, directory):
    raw_inputs, raw_outputs = ((directory / f).read_bytes() for f in ARTIFACTS)
    if report['scope'] != SCOPE or report['witness_scope'] != WITNESS_SCOPE or report['input_sha256'] != sha(raw_inputs) or report['output_sha256'] != sha(raw_outputs): raise ValueError('string operation observation scope/byte identity')
    cases, outputs = json.loads(raw_inputs), json.loads(raw_outputs); counts = compare(cases, outputs)
    if report['cases'] != len(cases) or report['outcomes'] != counts or raw_inputs != json_bytes(inputs()) or report['false_claim_witnesses'] != negative_witnesses(cases, outputs): raise ValueError('string operation observation/witness coverage')
    return len(cases)
