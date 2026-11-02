"""Actual Go index guards/panics and finite-bitmask observations, empirical only."""

import json

from byte_differential import equal_word, less_word
from run_proof_pilot import json_bytes, sha, timed

CAPACITIES = (0, 1, 32, 256)
ARTIFACTS = ("index-input.json", "index-output.json")
SCOPE = "actual Go signed index guards, recovered index panics and finite-mask value observations versus independent bit loops; empirical only, no captured-source application proof"
WITNESS_SCOPE = "explicit actual-Go/bit-loop witnesses to deliberately false mutations; checker type failures alone do not establish falsity"


def inputs():
    cases = []
    for capacity in CAPACITIES:
        for length in dict.fromkeys((0, 1, max(capacity - 1, 0), capacity, capacity + 1)):
            for index in dict.fromkeys((-(1 << 63), -1, 0, 1, length - 1, length, length + 1, capacity - 1, capacity, capacity + 1, (1 << 63) - 1)):
                cases.append({"capacity": capacity, "length": length, "index": str(index)})
    return cases


def model(row):
    n, capacity, index = row["length"], row["capacity"], int(row["index"])
    raw = index & ((1 << 64) - 1)
    negative = bool(raw & (1 << 63))
    unsigned_less = less_word(64, raw, n)
    signed_less = negative if negative != bool(n & (1 << 63)) else unsigned_less
    source_guard = not negative and signed_less
    low = raw & (capacity - 1) if capacity else 0
    return {"bound": not less_word(64, capacity, n), "source_guard": source_guard,
            "mask_guard": not negative and unsigned_less, "covered": capacity > 0 and equal_word(64, raw, low),
            "actual_defined": source_guard, "actual_byte": (index * 31 + 17) & 255 if source_guard else -1,
            "unsigned_index": str(raw), "low_index": str(low)}


def compare(cases, outputs):
    if cases != inputs() or len(cases) != len(outputs):
        raise ValueError("index observation input coverage")
    counts = {"bounded_safe": 0, "bounded_unsafe": 0, "unbounded_safe_uncovered": 0,
              "negative_panics": 0, "large_positive_panics": 0, "bounded_empty_no_index": 0}
    for row, actual in zip(cases, outputs):
        expected = model(row)
        if not isinstance(actual, dict) or set(actual) != set(expected) or any(type(actual[k]) is not type(v) for k, v in expected.items()):
            raise ValueError("index observation field/type mismatch")
        if actual != expected:
            raise ValueError("Go/bit-loop index mismatch")
        if actual["bound"]:
            if actual["source_guard"] != actual["mask_guard"]:
                raise ValueError("bounded signed guard differs")
            if actual["source_guard"] and not actual["covered"]:
                raise ValueError("bounded safe index was truncated")
        counts["bounded_safe"] += actual["bound"] and actual["actual_defined"]
        counts["bounded_unsafe"] += actual["bound"] and not actual["actual_defined"]
        counts["unbounded_safe_uncovered"] += not actual["bound"] and actual["actual_defined"] and not actual["covered"]
        counts["negative_panics"] += int(row["index"]) < 0 and not actual["actual_defined"]
        counts["large_positive_panics"] += int(row["index"]) == (1 << 63) - 1 and not actual["actual_defined"]
        counts["bounded_empty_no_index"] += row["capacity"] == 0 and actual["bound"] and not actual["actual_defined"]
    if not all(counts.values()):
        raise ValueError("index observations lost boundary/nonvacuity cases")
    return counts


def negative_witnesses(cases, outputs):
    """Full values, including a real successful read beyond each private bound."""
    result = []
    for capacity in CAPACITIES:
        for family, mutation, length, index in (
                ("length-sign", "altered-result", 0, 0),
                ("guard-equivalence", "altered-result", 0, 0),
                *(([("index-coverage", "erased-bound", capacity + 1, capacity),
                    ("index-coverage", "erased-guard", 0, -1)]) if capacity else
                  [("empty-no-index", "erased-bound", 1, 0)])):
            wanted = {"capacity": capacity, "length": length, "index": str(index)}
            position = cases.index(wanted)
            actual = outputs[position]
            if family == "length-sign":
                left, right = bool(length & (1 << 63)), True
                assert actual["bound"]
            elif family == "guard-equivalence":
                left, right = actual["source_guard"], not actual["mask_guard"]
                assert actual["bound"]
            elif family == "empty-no-index":
                left, right = actual["source_guard"], False
                assert actual["actual_defined"] and not actual["bound"]
            else:
                left, right = actual["unsigned_index"], actual["low_index"]
                assert (not actual["bound"] and actual["source_guard"] and actual["actual_defined"]
                        if mutation == "erased-bound" else actual["bound"] and not actual["source_guard"])
            if left == right:
                raise ValueError("deliberate mutation has no false-claim witness")
            result.append({"case_id": f"index-bound-{capacity}-{family}-{mutation}",
                           "input_position": position, "left": left, "right": right})
    return result


def run(executable, directory):
    cases = inputs()
    path = directory / ARTIFACTS[0]
    path.write_bytes(json_bytes(cases))
    measured = timed([executable, path])
    if measured["exit_code"]:
        raise RuntimeError("actual Go index observer failed")
    raw = measured.pop("stdout").encode()
    (directory / ARTIFACTS[1]).write_bytes(raw)
    outputs = json.loads(raw)
    return {"scope": SCOPE, "cases": len(cases), "outcomes": compare(cases, outputs),
            "negative_witness_scope": WITNESS_SCOPE, "negative_claim_witnesses": negative_witnesses(cases, outputs),
            "input_sha256": sha(path.read_bytes()), "output_sha256": sha(raw),
            "go_binary_sha256": sha(executable.read_bytes()), "go_measurement": measured}


def check(report, directory):
    a, b = ((directory / filename).read_bytes() for filename in ARTIFACTS)
    if report["scope"] != SCOPE or report["input_sha256"] != sha(a) or report["output_sha256"] != sha(b):
        raise ValueError("index observation scope/byte identity")
    cases, outputs = json.loads(a), json.loads(b)
    if report["cases"] != len(cases) or report["outcomes"] != compare(cases, outputs) or a != json_bytes(inputs()):
        raise ValueError("index observation outcome coverage")
    if report["negative_witness_scope"] != WITNESS_SCOPE or report["negative_claim_witnesses"] != negative_witnesses(cases, outputs):
        raise ValueError("missing or misleading false-claim witnesses")
    return len(cases)
