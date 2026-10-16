"""Raw-model tuples and separate actual-Go slice construction/index outcomes."""

import json

from byte_differential import less_word, equal_word
from order_differential import adder
from run_proof_pilot import json_bytes, sha, timed
from slice_differential import cell, ZERO

CAPACITIES = (0, 1, 8, 32, 127, 128, 129)
ARTIFACTS = ("access-input.json", "access-output.json")
SCOPE = "raw computed descriptor validity and full64 address arithmetic versus independent bit loops, with separate actual-Go construction/index outcomes; invalid model metadata is not an incoming Go slice or a captured-source theorem"
WITNESS_SCOPE = "explicit raw-model or out-of-profile Go witnesses to false mutations; rejected representation metadata and source construction/index failures remain distinct"
MASK = (1 << 64) - 1
FAMILIES = ("address-safety", "physical-bounds", "live-region", "full-value-coverage", "finite-read", "valid-copy", "valid-full", "nil-unindexable", "empty-no-access")


def padding(n):
    return 1 << max(n - 1, 0).bit_length()


def row(b, n, o, length, cap, index, *, nil=False, region=False, dead=False):
    live = [True, True]
    if dead:
        live[int(region)] = False
    return {"profile": b, "storage": str(n), "offset": str(o), "length": str(length),
            "capacity": str(cap), "index": str(index), "nil": nil, "region": region, "live": live}


def inputs():
    cases = []
    for b in CAPACITIES:
        for o in dict.fromkeys((0, min(1, b), b // 2, b)):
            for cap in dict.fromkeys((0, min(1, b - o), b - o)):
                for length in dict.fromkeys((0, min(1, cap), cap)):
                    for index in dict.fromkeys((-(1 << 63), -1, 0, 1, length - 1, length, cap - 1, cap, (1 << 63) - 1)):
                        for region in (False, True):
                            cases.append(row(b, b, o, length, cap, index, region=region))
        cases.extend([row(b, 0, 0, 0, 0, 0, nil=True), row(b, 1, 0, 1, 1, 0),
                      row(b, 1, 0, 2, 1, 1), row(b, 1, 0, 1, 1, 0, dead=True),
                      row(b, padding(b) + 1, 0, padding(b) + 1, padding(b) + 1, padding(b)),
                      row(b, b + 1, 0, b + 1, b + 1, b), row(b, 0, 0, 1, 0, 0, nil=True)])
        for bad in (-(1 << 63), -1, (1 << 63) - 1):
            cases.extend([row(b, b, bad, 0, 0, 0), row(b, b, 0, bad, 1, 0),
                          row(b, b, 0, 0, bad, 0), row(b, b, bad, 1, 1, 1)])
    return list({json.dumps(r, sort_keys=True): r for r in cases}.values())


def signed(n):
    return n - (1 << 64) if n & (1 << 63) else n


def model(r):
    n, o, length, cap, index = (int(r[k]) for k in ("storage", "offset", "length", "capacity", "index"))
    ou, lu, ku, iu, nu = (v & MASK for v in (o, length, cap, index, n))
    extent, ec, _ = adder(64, ou, ku, False)
    absolute, ac, _ = adder(64, ou, iu, False)
    selected = r["live"][int(r["region"])]

    def valid(l, k):
        if r["nil"]:
            return equal_word(64, l & MASK, 0) and equal_word(64, k & MASK, 0)
        return selected and all(v >= 0 for v in (o, l, k, n)) and not less_word(64, k & MASK, l & MASK) and not ec and not less_word(64, nu, extent)

    high, _, _ = adder(64, ou, lu, False)
    constructible = length == 0 and cap == 0 if r["nil"] else 0 <= o <= signed(high) <= signed(extent) <= n
    defined = constructible and not r["nil"] and index >= 0 and index < length
    reconstructed = absolute & (padding(r["profile"]) - 1)
    seed = 0x6a09e667f3bcc909 if r["region"] else 0
    return {"valid": valid(length, cap), "selected_live": selected, "guard": not r["nil"] and index >= 0 and index < length,
            "extent": str(extent), "extent_carry": ec, "absolute": str(absolute), "absolute_carry": ac,
            "physical_bound": less_word(64, absolute, nu), "physical_nonnegative": signed(absolute) >= 0,
            "storage_bound": not less_word(64, r["profile"], nu), "reconstructed": str(reconstructed),
            "covered": equal_word(64, absolute, reconstructed), "copy_valid": valid(length, cap),
            "full_valid": valid(cap, cap), "nil_guard": False, "constructed": constructible,
            "construction_panicked": not r["nil"] and not constructible,
            "read_defined": defined, "read_panicked": constructible and not defined,
            "actual_value": cell(seed, absolute) if defined else "", "modeled_value": cell(seed, reconstructed)}


def compare(cases, outputs):
    if cases != inputs() or len(cases) != len(outputs):
        raise ValueError("access observation input coverage")
    counts = {k: 0 for k in ("proof_domain_reads", "source_reads", "construction_rejections", "index_panics",
                             "invalid_metadata_with_local_guard", "dead_binding_constructible",
                             "out_of_profile_reads", "padding_not_admission", "uncovered_source_reads")}
    for r, actual in zip(cases, outputs):
        expected = model(r)
        if not isinstance(actual, dict) or set(actual) != set(expected) or any(type(actual[k]) is not type(v) for k, v in expected.items()):
            raise ValueError("access observation field/type mismatch")
        if actual != expected:
            raise ValueError("Go/bit-loop access mismatch")
        if actual["valid"]:
            if not actual["copy_valid"] or not actual["full_valid"]:
                raise ValueError("valid descriptor preservation failed")
            if actual["guard"] and not all(actual[k] for k in ("selected_live", "physical_bound", "physical_nonnegative", "constructed", "read_defined")):
                raise ValueError("valid local guard failed actual physical safety")
            if actual["guard"] and actual["absolute_carry"]:
                raise ValueError("valid address wrapped")
        domain = actual["valid"] and actual["guard"] and actual["storage_bound"]
        if domain and (not actual["covered"] or actual["actual_value"] != actual["modeled_value"]):
            raise ValueError("bounded full-value/read coverage failed")
        if r["profile"] == 0 and actual["valid"] and actual["storage_bound"] and actual["guard"]:
            raise ValueError("zero storage admitted a valid access")
        counts["proof_domain_reads"] += domain
        counts["source_reads"] += actual["read_defined"]
        counts["construction_rejections"] += not actual["constructed"]
        counts["index_panics"] += actual["read_panicked"]
        counts["invalid_metadata_with_local_guard"] += not actual["valid"] and actual["guard"]
        counts["dead_binding_constructible"] += not actual["selected_live"] and actual["constructed"]
        counts["out_of_profile_reads"] += not actual["storage_bound"] and actual["read_defined"]
        counts["padding_not_admission"] += int(r["storage"]) > r["profile"] and int(r["storage"]) <= padding(r["profile"]) and actual["read_defined"]
        counts["uncovered_source_reads"] += actual["read_defined"] and not actual["covered"]
    if not all(counts.values()):
        raise ValueError("lost access boundary/nonvacuity observations")
    return counts


def negative_witnesses(cases, outputs):
    result = []
    for b in CAPACITIES:
        for family in FAMILIES:
            data = family in FAMILIES[:5]
            if not b and data or b and family == "empty-no-access":
                continue
            domain = "bounded_source"
            if family == "physical-bounds":
                wanted, domain = row(b, 1, 0, 2, 1, 1), "raw_invalid_representation"
            elif family == "live-region":
                wanted, domain = row(b, 1, 0, 1, 1, 0, dead=True), "raw_invalid_binding"
            elif family == "full-value-coverage":
                p = padding(b)
                wanted, domain = row(b, p + 1, 0, p + 1, p + 1, p), "out_of_profile_source"
            elif family == "empty-no-access":
                wanted, domain = row(b, 1, 0, 1, 1, 0), "out_of_profile_source"
            elif family == "nil-unindexable":
                wanted = row(b, 0, 0, 0, 0, 0, nil=True)
            else:
                wanted = row(b, min(b, 1), 0, min(b, 1), min(b, 1), 0)
            i = cases.index(wanted)
            a = outputs[i]
            if family == "address-safety":
                left, right = not a["absolute_carry"] and a["physical_nonnegative"], False
            elif family in ("physical-bounds", "live-region"):
                left, right = a["physical_bound" if family == "physical-bounds" else "selected_live"], True
            elif family == "full-value-coverage":
                left, right = a["absolute"], a["reconstructed"]
            elif family == "finite-read":
                left, right = a["modeled_value"], ZERO
            elif family in ("valid-copy", "valid-full"):
                left, right = a["copy_valid" if family == "valid-copy" else "full_valid"], False
            elif family == "nil-unindexable":
                left, right = a["nil_guard"], True
            else:
                left, right = a["guard"], False
            if left == right:
                raise ValueError("no access false-claim witness")
            label = "erased-validity" if family in ("physical-bounds", "live-region") else "erased-storage-bound" if family in ("full-value-coverage", "empty-no-access") else "zeroed-read" if family == "finite-read" else "altered-result"
            result.append({"case_id": f"slice-access-{b}-{family}-{label}", "input_position": i,
                           "domain": domain, "left": left, "right": right})
    return result


def run(executable, directory):
    cases = inputs()
    path = directory / ARTIFACTS[0]
    path.write_bytes(json_bytes(cases))
    measured = timed([executable, path])
    if measured["exit_code"]:
        raise RuntimeError("actual Go access observer failed")
    raw = measured.pop("stdout").encode()
    (directory / ARTIFACTS[1]).write_bytes(raw)
    outputs = json.loads(raw)
    return {"scope": SCOPE, "cases": len(cases), "outcomes": compare(cases, outputs),
            "witness_scope": WITNESS_SCOPE, "false_claim_witnesses": negative_witnesses(cases, outputs),
            "input_sha256": sha(path.read_bytes()), "output_sha256": sha(raw),
            "go_binary_sha256": sha(executable.read_bytes()), "go_measurement": measured}


def check(report, directory):
    a, b = ((directory / filename).read_bytes() for filename in ARTIFACTS)
    if report["scope"] != SCOPE or report["input_sha256"] != sha(a) or report["output_sha256"] != sha(b):
        raise ValueError("access observation scope/byte identity")
    cases, outputs = json.loads(a), json.loads(b)
    if a != json_bytes(inputs()) or report["cases"] != len(cases) or report["outcomes"] != compare(cases, outputs):
        raise ValueError("access observation outcome coverage")
    if report["witness_scope"] != WITNESS_SCOPE or json_bytes(report["false_claim_witnesses"]) != json_bytes(negative_witnesses(cases, outputs)):
        raise ValueError("missing, mistyped or misclassified access false-claim witnesses")
    return len(cases)
