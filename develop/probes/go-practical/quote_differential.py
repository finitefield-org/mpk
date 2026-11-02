"""Empirical original-Go Quote observations; no source-to-certificate claim."""

import json
import random

from order_differential import adder
from run_proof_pilot import json_bytes, sha, timed

SCOPE = "original Go Quote success/failure/frame observations versus independent bit-loop model; empirical only, no source-bound or whole-function proof"
COUNTS = (0, 1, 3, 8, 32, 127, 128, 129)
ARTIFACTS = ("quote-input.json", "quote-output.json")
MAXIMUM = (1 << 63) - 1
MASK = (1 << 64) - 1
SIGN = 1 << 63


def inputs():
    result, seen = [], set()
    random_values = random.Random(0x4D504B51554F5445)

    def add(limit, values, empty=False, nil=False):
        key = (limit, tuple(values), empty, nil)
        if key not in seen:
            seen.add(key)
            result.append({"limit": str(limit), "amounts": list(map(str, values)), "customer_empty": empty, "nil_lines": nil})

    for count in COUNTS:
        for limit in (-1, 0, 1, 2, MAXIMUM - 1, MAXIMUM):
            patterns = [[0] * count, [1] * count, [i % 2 for i in range(count)]]
            if count:
                patterns.extend([[MAXIMUM] + [0] * (count - 1), [-1] + [0] * (count - 1), [0] * (count - 1) + [-1], [limit] + [1] * (count - 1)])
            for values in patterns:
                for empty in (False, True):
                    add(limit, values, empty)
                    if count == 0:
                        add(limit, values, empty, True)
        for _ in range(6):
            values = [random_values.randint(0, 10) for _ in range(count)]
            add(sum(values), values)
            add(max(sum(values) - 1, 0), values)
            add(MAXIMUM, values)
            if count:
                negative = list(values)
                negative[-1] = -1
                add(MAXIMUM, negative)
            values = [random_values.randint(-(1 << 63), MAXIMUM) for _ in range(count)]
            add(random_values.randint(0, MAXIMUM), values)
    return result


def signed_le(a, b):
    if bool(a & SIGN) != bool(b & SIGN):
        return bool(a & SIGN)
    return adder(64, b, MASK ^ a, True)[1]


def model(row):
    limit = int(row["limit"]) & MASK
    total, code = 0, 0
    if row["customer_empty"] or limit & SIGN:
        code = 1
    else:
        for item in row["amounts"]:
            item = int(item) & MASK
            if item & SIGN:
                code = 1
                break
            remaining = adder(64, limit, MASK ^ total, True)[0]
            if not signed_le(item, remaining):
                code = 2
                break
            total = adder(64, total, item, False)[0]
    if code:
        total = 0
    signed_total = total - (1 << 64) if total & SIGN else total
    return {"total": str(signed_total), "code": code, "length": len(row["amounts"]), "input_unchanged": True, "nil_preserved": True}


def compare(cases, observations):
    if cases != inputs() or len(cases) != len(observations):
        raise ValueError("Quote observation input coverage")
    outcomes = {"success": 0, "bad_input": 0, "limit_exceeded": 0}
    for row, actual in zip(cases, observations):
        expected = model(row)
        if not isinstance(actual, dict) or set(actual) != set(expected) or any(type(actual[k]) is not type(v) for k, v in expected.items()):
            raise ValueError("Quote observation field/type mismatch")
        if actual != expected:
            raise ValueError("Go/bit-loop Quote mismatch")
        outcomes[("success", "bad_input", "limit_exceeded")[actual["code"]]] += 1
        if actual["code"] == 0:
            if int(actual["total"]) != sum(map(int, row["amounts"])) or not 0 <= int(actual["total"]) <= int(row["limit"]):
                raise ValueError("Go Quote success exact sum/bound failed")
    if not all(outcomes.values()):
        raise ValueError("Quote observations lost result branches")
    return outcomes


def run(executable, directory):
    cases = inputs()
    path = directory / ARTIFACTS[0]
    path.write_bytes(json_bytes(cases))
    measurement = timed([executable, path])
    if measurement["exit_code"]:
        raise RuntimeError("actual Go Quote execution failed")
    output = measurement.pop("stdout").encode()
    (directory / ARTIFACTS[1]).write_bytes(output)
    outcomes = compare(cases, json.loads(output))
    return {"scope": SCOPE, "cases": len(cases), "outcomes": outcomes, "input_sha256": sha(path.read_bytes()),
            "output_sha256": sha(output), "go_binary_sha256": sha(executable.read_bytes()), "go_measurement": measurement}


def check(report, record):
    input_bytes, output_bytes = ((record / name).read_bytes() for name in ARTIFACTS)
    if report["scope"] != SCOPE or report["input_sha256"] != sha(input_bytes) or report["output_sha256"] != sha(output_bytes):
        raise ValueError("Quote observation scope/byte identity")
    cases, observations = json.loads(input_bytes), json.loads(output_bytes)
    outcomes = compare(cases, observations)
    if report["cases"] != len(cases) or report["outcomes"] != outcomes or input_bytes != json_bytes(inputs()):
        raise ValueError("Quote observation outcome coverage")
    return len(cases)
