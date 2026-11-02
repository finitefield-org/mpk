"""Independent bit-loop predictions compared with actual pinned Go execution.

This is empirical compiler/model evidence, never a universal proof or a
frontend/source-to-certificate binding. Proof evidence stays with both checkers.
"""

from itertools import product
import json
import random

from run_proof_pilot import json_bytes, sha, timed

SCOPE = "actual pinned Go signed operations versus independent bit-loop model; empirical observations, not a Go frontend/application proof"


def inputs():
    result = []
    random_values = random.Random(0x4D504B474F)
    for width in (8, 16, 32, 64):
        low, high = -(1 << (width - 1)), (1 << (width - 1)) - 1
        edge = (low, low + 1, -2, -1, 0, 1, 2, high - 1, high)
        triples = list(product(edge, repeat=3))
        triples.extend(tuple(random_values.randint(low, high) for _ in range(3)) for _ in range(128))
        for l, a, x in triples:
            result.append({"width": width, "limit": str(l), "accumulator": str(a), "item": str(x)})
    return result


def adder(width, a, b, incoming):
    value = 0
    sign_incoming = None
    for bit in range(width):
        if bit == width - 1:
            sign_incoming = incoming
        x, y = bool(a & (1 << bit)), bool(b & (1 << bit))
        value |= int(x ^ y ^ incoming) << bit
        incoming = (x and y) or (incoming and (x ^ y))
    return value, incoming, sign_incoming


def model(row):
    width = row["width"]
    mask, sign_mask = (1 << width) - 1, 1 << (width - 1)
    l, a, x = (int(row[key]) & mask for key in ("limit", "accumulator", "item"))
    remaining, _, _ = adder(width, l, mask ^ a, True)
    result, out, into_sign = adder(width, a, x, False)

    def signed(value):
        return value - (1 << width) if value & sign_mask else value

    def le(left, right):
        if bool(left & sign_mask) != bool(right & sign_mask):
            return bool(left & sign_mask)
        return adder(width, right, mask ^ left, True)[1]

    observed = {"remaining": str(signed(remaining)), "result": str(signed(result)),
                "limit_nonnegative": not bool(l & sign_mask), "accumulator_nonnegative": not bool(a & sign_mask),
                "accumulator_within_limit": le(a, l), "item_nonnegative": not bool(x & sign_mask),
                "item_within_remaining": le(x, remaining), "remaining_nonnegative": not bool(remaining & sign_mask),
                "result_nonnegative": not bool(result & sign_mask), "result_within_limit": le(result, l),
                "add_guard": into_sign == out}
    observed["continues"] = all(observed[k] for k in ("limit_nonnegative", "accumulator_nonnegative",
                                                     "accumulator_within_limit", "item_nonnegative", "item_within_remaining"))
    return observed


def compare(cases, observations):
    if cases != inputs() or len(cases) != len(observations):
        raise ValueError("differential input coverage")
    continuing = 0
    for row, actual in zip(cases, observations):
        expected = model(row)
        if not isinstance(actual, dict) or set(actual) != set(expected) or any(type(actual[k]) is not type(v) for k, v in expected.items()):
            raise ValueError("Go observation field/type mismatch")
        if actual != expected:
            raise ValueError("Go/model signed operation mismatch")
        if actual["continues"]:
            continuing += 1
            if not all(actual[k] for k in ("remaining_nonnegative", "result_nonnegative", "result_within_limit", "add_guard")):
                raise ValueError("Go continuation invariant failed")
            if int(actual["result"]) != int(row["accumulator"]) + int(row["item"]):
                raise ValueError("Go continuation exact sum failed")
    if not continuing:
        raise ValueError("differential lost success paths")
    return continuing


def run(executable, directory):
    cases = inputs()
    input_path = directory / "go-reference-input.json"
    input_path.write_bytes(json_bytes(cases))
    measurement = timed([executable, input_path])
    if measurement["exit_code"]:
        raise RuntimeError("pinned Go differential execution failed")
    output = measurement.pop("stdout").encode()
    (directory / "go-reference-output.json").write_bytes(output)
    continuing = compare(cases, json.loads(output))
    return {"scope": SCOPE, "cases": len(cases), "continuation_cases": continuing,
            "input_sha256": sha(input_path.read_bytes()), "output_sha256": sha(output),
            "go_binary_sha256": sha(executable.read_bytes()), "go_measurement": measurement}


def check(report, record):
    input_bytes = (record / "go-reference-input.json").read_bytes()
    output_bytes = (record / "go-reference-output.json").read_bytes()
    if report["scope"] != SCOPE or sha(input_bytes) != report["input_sha256"] or sha(output_bytes) != report["output_sha256"]:
        raise ValueError("differential scope/byte identity")
    cases, observations = json.loads(input_bytes), json.loads(output_bytes)
    continuing = compare(cases, observations)
    if report["cases"] != len(cases) or report["continuation_cases"] != continuing or input_bytes != json_bytes(inputs()):
        raise ValueError("differential coverage counts")
    return len(cases)
