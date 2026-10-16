"""Actual Go descriptor/sharing observations against independent bit loops."""

import json

from byte_differential import less_word
from order_differential import adder
from run_proof_pilot import json_bytes, sha, timed

CAPACITIES = (0, 1, 8, 32, 127, 128, 129)
ARTIFACTS = ("slice-input.json", "slice-output.json")
SCOPE = "actual Go slice assignment/full reslicing, shared writes, array value copies and recovered index panics versus independent cell/bit loops; empirical only, no captured-source function proof"
WITNESS_SCOPE = "explicit actual-Go/bit-loop witnesses to false slice model mutations; type failure alone does not establish falsity"
MASK = (1 << 64) - 1
REPLACEMENT = "8000000000000000:ffffffffffffffff"
ZERO = "0000000000000000:0000000000000000"


def inputs():
    cases = []
    for n in CAPACITIES:
        for offset in dict.fromkeys((0, min(1, n), n // 2, n)):
            remaining = n - offset
            for capacity in dict.fromkeys((0, min(1, remaining), remaining)):
                for length in dict.fromkeys((0, min(1, capacity), capacity // 2, capacity)):
                    for index in dict.fromkeys((-(1 << 63), -1, 0, 1, length - 1, length, length + 1, capacity - 1, capacity, (1 << 63) - 1)):
                        for seed in (0, MASK):
                            cases.append({"storage": n, "offset": offset, "length": length, "capacity": capacity,
                                          "index": str(index), "nil": False, "seed": str(seed)})
        for index in (-(1 << 63), -1, 0, (1 << 63) - 1):
            cases.append({"storage": n, "offset": 0, "length": 0, "capacity": 0,
                          "index": str(index), "nil": True, "seed": "0"})
    return cases


def cell(seed, position):
    amount = seed ^ ((position + 1) * 0x9e3779b97f4a7c15 & MASK)
    tag = ((seed ^ 0xa5a5a5a5a5a5a5a5) + position * 0x100000001) & MASK
    return f"{amount:016x}:{tag:016x}"


def model(row):
    nil, length, capacity = row["nil"], row["length"], row["capacity"]
    index, offset, seed = int(row["index"]), row["offset"], int(row["seed"])
    raw = index & MASK
    nonnegative = not bool(raw & (1 << 63))
    original = not nil and nonnegative and less_word(64, raw, length)
    extended = not nil and nonnegative and less_word(64, raw, capacity)
    physical, carry, _ = adder(64, offset, raw, False)
    if extended and (carry or physical >= row["storage"]):
        raise ValueError("empirical source descriptor escaped backing")
    before = cell(seed, physical) if original else ""
    full_before = cell(seed, physical) if extended else ""
    return {"nil": nil, "length": length, "capacity": capacity, "copy_metadata": True,
            "full_metadata": True, "nil_empty_distinct": True,
            "source_defined": original, "read_defined": original, "write_defined": original,
            "before": before, "alias_after": REPLACEMENT if original else "",
            "wrong_fresh_view": cell(seed ^ 0x6a09e667f3bcc909, physical) if original else "",
            "full_defined": extended, "full_before": full_before,
            "full_after": REPLACEMENT if original else full_before,
            "other_region_frame": True, "same_region_frame": True, "array_copy_preserved": True,
            "array_differs_after_write": original and before != REPLACEMENT}


def compare(cases, outputs):
    if cases != inputs() or len(cases) != len(outputs):
        raise ValueError("slice observation input coverage")
    counts = {k: 0 for k in ("nil", "nonnil_empty", "defined_write", "recovered_panics", "exposed_tail",
                             "alias_changes_visible", "other_region_frames", "array_copy_preserved",
                             "negative_panics", "large_positive_panics", "restricted_capacity_views")}
    for row, actual in zip(cases, outputs):
        expected = model(row)
        if not isinstance(actual, dict) or set(actual) != set(expected) or any(type(actual[k]) is not type(v) for k, v in expected.items()):
            raise ValueError("slice observation field/type mismatch")
        if actual != expected:
            raise ValueError("Go/bit-loop slice mismatch")
        counts["nil"] += actual["nil"]
        counts["nonnil_empty"] += not actual["nil"] and actual["length"] == 0
        counts["defined_write"] += actual["write_defined"]
        counts["recovered_panics"] += not actual["read_defined"]
        counts["exposed_tail"] += actual["full_defined"] and not actual["read_defined"] and actual["full_before"] != ZERO
        counts["alias_changes_visible"] += actual["read_defined"] and actual["alias_after"] != actual["before"]
        counts["other_region_frames"] += actual["write_defined"] and actual["other_region_frame"]
        counts["array_copy_preserved"] += actual["write_defined"] and actual["array_copy_preserved"] and actual["array_differs_after_write"]
        counts["negative_panics"] += int(row["index"]) < 0 and not actual["read_defined"]
        counts["large_positive_panics"] += int(row["index"]) == (1 << 63) - 1 and not actual["read_defined"]
        counts["restricted_capacity_views"] += row["offset"] + row["capacity"] < row["storage"]
    if not all(counts.values()):
        raise ValueError("lost slice boundary/nonvacuity observations")
    return counts


def negative_witnesses(cases, outputs):
    result = []
    for n in CAPACITIES:
        for family in ("descriptor-copy", "full-reslice", "nil-empty", "same-key-alias", "other-region-frame", "array-value-copy", "tail-exposure"):
            if not n and family in ("same-key-alias", "other-region-frame", "array-value-copy", "tail-exposure"):
                continue
            row = {"storage": n, "offset": 0, "length": 1 if family in ("same-key-alias", "other-region-frame", "array-value-copy") else 0,
                   "capacity": n, "index": "0", "nil": False, "seed": "0"}
            i = cases.index(row)
            actual = outputs[i]
            label = {"same-key-alias": "fresh-id-per-view", "other-region-frame": "erased-region-separation",
                     "array-value-copy": "shared-copy-region", "tail-exposure": "zeroed-tail"}.get(family, "altered-result")
            if family in ("descriptor-copy", "full-reslice", "nil-empty"):
                key = {"descriptor-copy": "copy_metadata", "full-reslice": "full_metadata", "nil-empty": "nil_empty_distinct"}[family]
                left, right = actual[key], False
            elif family == "same-key-alias":
                left, right = actual["wrong_fresh_view"], REPLACEMENT
            elif family in ("other-region-frame", "array-value-copy"):
                left, right = actual["alias_after"], actual["before"]
            else:
                left, right = actual["full_before"], ZERO
            if left == right or left == "":
                raise ValueError("no slice false-claim witness")
            result.append({"case_id": f"slice-store-{n}-{family}-{label}", "input_position": i,
                           "left": left, "right": right})
    return result


def run(executable, directory):
    cases = inputs()
    path = directory / ARTIFACTS[0]
    path.write_bytes(json_bytes(cases))
    measured = timed([executable, path])
    if measured["exit_code"]:
        raise RuntimeError("actual Go slice observer failed")
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
        raise ValueError("slice observation scope/byte identity")
    cases, outputs = json.loads(a), json.loads(b)
    if report["cases"] != len(cases) or report["outcomes"] != compare(cases, outputs) or a != json_bytes(inputs()):
        raise ValueError("slice observation outcome coverage")
    if report["negative_witness_scope"] != WITNESS_SCOPE or json_bytes(report["negative_claim_witnesses"]) != json_bytes(negative_witnesses(cases, outputs)):
        raise ValueError("missing or misleading slice false-claim witnesses")
    return len(cases)
