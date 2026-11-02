"""Independently check retained native observations and their draft-only scope."""
import hashlib
import json
import re

from scope_cases import CASES, HERE, PENDING, RECORD, ROOT, SCOPE, VECTORS, vectors


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def exact(actual, expected):
    # JSON distinguishes booleans from integers; ordinary Python equality does not.
    return json_bytes(actual) == json_bytes(expected)


def strict_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError("duplicate JSON key")
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(ValueError("nonfinite JSON")))


def sources():
    paths = [HERE / name for name in (
        "scope_reference.go", "scope_cases.py", "run_scope_probes.py",
        "check_scope_report.py", "test_scope_report.py", "host-toolchain.json",
        "run_module_probes.py",
    )]
    paths.append(VECTORS)
    return [{"path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size,
             "sha256": sha(path.read_bytes())} for path in sorted(paths)]


def expected_runtime():
    # These facts are specified independently of the retained native output.
    # No descriptor's physical address is treated as a source observation.
    return {
        "root_copy_shares_backing": True,
        "root_descriptor_rebind_preserves_original": True,
        "root_array_copy_shares_referenced_backing": True,
        "zero_operations_preserve_all_backing": True,
        "zero_source_copy_preserves_all_backing": True,
        "positive_capacity_empty_exposes_full_tail": True,
        "positive_capacity_empty_shares_backing": True,
        "positive_append_to_zero_succeeds_in_go": True,
        "conversion_error_body_calls": 0,
        "invocation_error_body_calls": 1,
        "boxed_zero_non_nil": True,
        "counted_message": "counted",
        "typed_nil_interface_is_nil": False,
        "typed_nil_method_panics": True,
        "zero_value_error_is_nil": False,
        "nil_interface_is_nil": True,
        "slice_error_is_nil": False,
        "error_comparison_panics": True,
        "closed_messages": ["code", "flag", "slice"],
        "extreme_amount_text": "-9223372036854775808",
        "zero_slices": [
            {"kind": i, "nil": i == 0, "length": 0, "capacity": 0,
             "copy_count": 0, "append_empty_nil": i == 0, "reslice_nil": i == 0,
             "index_panicked": True, "positive_reslice_panicked": True,
             "named_append_type": "rows"}
            for i in range(4)
        ],
    }


def source_observations(observed):
    require(type(observed) is list and len(observed) == len(CASES), "original source coverage")
    for row, actual in zip(CASES, observed):
        require(set(actual) == {"id", "status", "errors", "types", "calls", "int_bits", "uint_bits"},
                "native source shape")
        require(actual["id"] == row["id"], "original source order/identity")
        expectation = row["native_expectation"]
        require(actual["status"] == expectation["status"], "original Go status")
        bits = 0 if actual["status"] == "source_parse_error" else 64
        require(type(actual["int_bits"]) is int and type(actual["uint_bits"]) is int
                and actual["int_bits"] == bits and actual["uint_bits"] == bits,
                "target-sized original integers")
        require(type(actual["errors"]) is list and all(type(e) is str for e in actual["errors"]),
                "native diagnostics")
        if expectation["error_contains"] is not None:
            require(actual["errors"] and any(expectation["error_contains"] in e for e in actual["errors"]),
                    "original error boundary")
        else:
            require(actual["errors"] == [], "unexpected native diagnostics")
        if actual["status"] != "ok":
            require(actual["types"] == [] and actual["calls"] == [], "failed original check has no typed facts")
            continue
        require(type(actual["types"]) is list, "native type facts")
        names = [fact["name"] for fact in actual["types"]]
        require(names == sorted(set(names)), "native type identity/order")
        for fact in actual["types"]:
            require(set(fact) == {"name", "alias", "underlying", "fields", "value_error", "pointer_error"}
                    and all(type(fact[key]) is bool for key in ("alias", "value_error", "pointer_error"))
                    and type(fact["underlying"]) is str
                    and type(fact["fields"]) is list and all(type(v) is str for v in fact["fields"]),
                    "native type fact shape")
        by_name = {fact["name"]: fact for fact in actual["types"]}
        for name, assertions in expectation["type_assertions"].items():
            require(name in by_name, "required original type")
            for key, value in assertions.items():
                require(by_name[name][key] == value, "original nominal/error type")
        require(type(actual["calls"]) is list, "native call facts")
        offsets = [fact["offset"] for fact in actual["calls"]]
        require(offsets == sorted(set(offsets)), "native call identity/order")
        for fact in actual["calls"]:
            require(set(fact) == {"offset", "kind", "target"}
                    and type(fact["offset"]) is int and 0 <= fact["offset"] < len(row["source"].encode())
                    and fact["kind"] in {"unknown", "conversion", "static_function", "builtin",
                                         "direct_method", "interface_invoke"}
                    and type(fact["target"]) is str, "native call fact shape")
        by_offset = {fact["offset"]: fact for fact in actual["calls"]}
        for assertion in expectation["call_assertions"]:
            # Go positions are byte offsets, not Python character indexes.
            source, text = row["source"].encode(), assertion["text"].encode()
            offset = -1
            for _ in range(assertion.get("occurrence", 0) + 1):
                offset = source.index(text, offset + 1)
            require(offset in by_offset and by_offset[offset]["kind"] == assertion["kind"],
                    "conversion/static/direct/interface call distinction: " + row["id"])
            require(by_offset[offset]["target"] == assertion["target"], "exact original call target")


def check(report, record=RECORD):
    require(set(report) == {"schema", "work_item", "scope", "status", "pending", "inputs",
                            "observer_binary_sha256", "toolchain_inventory", "build", "artifacts",
                            "native_type_counts", "draft_outcome_counts"},
            "scope report closed shape")
    require(report["schema"] == "mpk.go_practical.w04_scope_observations.v0"
            and report["work_item"] == "GO-PRAC-T01-W04"
            and report["scope"] == SCOPE
            and report["status"] == "native observations retained; W04 freeze remains in progress",
            "native scope is not practical-profile support")
    require(report["pending"] == PENDING, "remaining source/proof/freeze work")
    require(exact(strict_json(VECTORS.read_bytes()), vectors()), "draft source vectors")
    require(exact(report["inputs"], sources()), "source identity coverage")
    require(re.fullmatch("[0-9a-f]{64}", report["observer_binary_sha256"]) is not None,
            "observer binary identity")
    require(exact(report["toolchain_inventory"], {
        "files": 14496,
        "sha256": "85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc",
    }), "exact Go toolchain distribution")
    require(exact(report["build"], {
        "release": "go1.25.0", "host": "darwin/arm64",
        "type_target": "gc/amd64", "effective_language": "go1.25.0",
        "trimpath": True, "buildvcs": False, "runtime_target_scope": "host observations, not Linux release",
    }), "native build/target boundary")
    expected_input = {"sources": [{"id": row["id"], "source": row["source"]} for row in CASES]}
    require(set(report["artifacts"]) == {"native-input.json", "native-output.json"}, "native artifact coverage")
    for name, pin in report["artifacts"].items():
        raw = (record / name).read_bytes()
        require(exact(pin, {"bytes": len(raw), "sha256": sha(raw)}), "native artifact identity")
    require(strict_json((record / "native-input.json").read_bytes()) == expected_input,
            "intact original native source input")
    native = strict_json((record / "native-output.json").read_bytes())
    require(set(native) == {"go_version", "target_sizes", "runtime", "sources"}
            and native["go_version"] == "go1.25.0" and native["target_sizes"] == "gc/amd64",
            "native runtime/type target")
    source_observations(native["sources"])
    require(json_bytes(native["runtime"]) == json_bytes(expected_runtime()),
            "complete independent runtime expectation")
    native_counts, draft_counts = {}, {}
    for row in CASES:
        k = row["native_expectation"]["status"]
        native_counts[k] = native_counts.get(k, 0) + 1
        k = row["draft_outcome"]
        draft_counts[k] = draft_counts.get(k, 0) + 1
    require(exact(report["native_type_counts"], native_counts), "native source classification counts")
    require(exact(report["draft_outcome_counts"], draft_counts), "draft outcomes are not observed admission")
    return len(CASES)


if __name__ == "__main__":
    count = check(strict_json((RECORD / "observations.json").read_bytes()))
    print(f"PASS: {count} original-Go cases and exact runtime facts; draft expectations remain unimplemented")
