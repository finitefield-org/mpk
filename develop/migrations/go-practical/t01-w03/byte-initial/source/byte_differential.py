"""Original-Go byte observations versus an independent bit-loop observer model."""

import json
import random

from run_proof_pilot import json_bytes, sha, timed

CAPACITIES = (0, 1, 32, 256)
ARTIFACTS = ("byte-input.json", "byte-output.json")
SCOPE = "original Go bounded string equality/order/index and tail/input observations versus independent bit-loop model; empirical only, no source-bound application proof"


def inputs():
    cases, seen = [], set()
    rng = random.Random(0x4D504B42595445)
    for capacity in CAPACITIES:
        examples = [b"", b"\x00", b"a", b"a\x00", b"a\x00x", b"\x80", b"\xff", b"\xc3\xa9", b"\xe3\x81\x82", b"\xc3("]
        examples += [b"a" * capacity, b"\x00" * capacity, b"\xff" * capacity, bytes(rng.randrange(256) for _ in range(capacity))]
        examples = list(dict.fromkeys(x for x in examples if len(x) <= capacity))
        for a in examples:
            for b in examples:
                backing_a = bytes(a) + bytes(rng.randrange(256) for _ in range(max(capacity, 1) - len(a)))
                backing_b = bytes(b) + bytes(rng.randrange(256) for _ in range(max(capacity, 1) - len(b)))
                for index in dict.fromkeys((-1, 0, len(a) - 1, len(a), len(a) + 1, -(1 << 63), (1 << 63) - 1)):
                    row = {"capacity": capacity, "left": backing_a.hex(), "right": backing_b.hex(), "left_length": len(a), "right_length": len(b), "index": str(index)}
                    key = tuple(row.items())
                    if key not in seen:
                        seen.add(key)
                        cases.append(row)
    return cases


def less_word(width, a, b):
    for bit in range(width - 1, -1, -1):
        x, y = (a >> bit) & 1, (b >> bit) & 1
        if x != y:
            return x == 0
    return False


def equal_word(width, a, b):
    return all(((a >> bit) & 1) == ((b >> bit) & 1) for bit in range(width))


def model(row):
    a, b = bytes.fromhex(row["left"]), bytes.fromhex(row["right"])
    n, m = row["left_length"], row["right_length"]
    index = int(row["index"])
    equal = equal_word(64, n, m)
    less, decided = False, False
    for i in range(max(row["capacity"], 1)):
        active_a, active_b = less_word(64, i, n), less_word(64, i, m)
        x, y = a[i] if active_a else 0, b[i] if active_b else 0
        equal = equal and equal_word(8, x, y)
        if not decided:
            if active_a and active_b:
                if not equal_word(8, x, y):
                    less, decided = less_word(8, x, y), True
            elif active_a != active_b:
                less, decided = active_b, True
    defined = index >= 0 and less_word(64, index, n)
    raw_equal, raw_less, raw_decided = True, False, False
    for x, y in zip(a, b):
        raw_equal = raw_equal and equal_word(8, x, y)
        if not raw_decided and not equal_word(8, x, y):
            raw_less, raw_decided = less_word(8, x, y), True
    return {"equal": equal, "less": less, "masked_equal": equal, "masked_less": less,
            "raw_equal": raw_equal, "raw_less": raw_less, "index_defined": defined, "index_byte": a[index] if defined else -1,
            "left_length": n, "right_length": m, "input_unchanged": True}


def compare(cases, outputs):
    if cases != inputs() or len(outputs) != len(cases):
        raise ValueError("byte observation input coverage")
    counts = {"equal": 0, "less": 0, "defined_index": 0, "undefined_index": 0, "equal_with_different_tail": 0, "padding_changes_raw_order": 0}
    for row, actual in zip(cases, outputs):
        expected = model(row)
        if not isinstance(actual, dict) or set(actual) != set(expected) or any(type(actual[k]) is not type(v) for k, v in expected.items()):
            raise ValueError("byte observation field/type mismatch")
        if actual != expected:
            raise ValueError("Go/bit-loop byte mismatch")
        counts["equal"] += actual["equal"]
        counts["less"] += actual["less"]
        counts["defined_index" if actual["index_defined"] else "undefined_index"] += 1
        counts["equal_with_different_tail"] += actual["equal"] and not actual["raw_equal"]
        counts["padding_changes_raw_order"] += actual["less"] != actual["raw_less"]
    if not all(counts.values()):
        raise ValueError("byte observations lost boundary/tail cases")
    return counts


def run(executable, directory):
    cases = inputs()
    path = directory / ARTIFACTS[0]
    path.write_bytes(json_bytes(cases))
    measured = timed([executable, path])
    if measured["exit_code"]:
        raise RuntimeError("actual Go byte execution failed")
    raw = measured.pop("stdout").encode()
    (directory / ARTIFACTS[1]).write_bytes(raw)
    outcomes = compare(cases, json.loads(raw))
    return {"scope": SCOPE, "cases": len(cases), "outcomes": outcomes, "input_sha256": sha(path.read_bytes()),
            "output_sha256": sha(raw), "go_binary_sha256": sha(executable.read_bytes()), "go_measurement": measured}


def check(report, directory):
    raw_inputs, raw_outputs = ((directory / filename).read_bytes() for filename in ARTIFACTS)
    if report["scope"] != SCOPE or report["input_sha256"] != sha(raw_inputs) or report["output_sha256"] != sha(raw_outputs):
        raise ValueError("byte observation scope/byte identity")
    cases, outputs = json.loads(raw_inputs), json.loads(raw_outputs)
    outcomes = compare(cases, outputs)
    if report["cases"] != len(cases) or report["outcomes"] != outcomes or raw_inputs != json_bytes(inputs()):
        raise ValueError("byte observation outcome coverage")
    return len(cases)
