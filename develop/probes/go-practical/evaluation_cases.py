"""Independent expected facts for original-Go evaluation, not a frontend."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
RECORD = ROOT / "develop/migrations/go-practical/t01-w04/evaluation-observations"
VECTORS = RECORD.parent / "evaluation-vectors.json"
SCOPE = ("private original-Go evaluation/control observations versus independent expected facts; "
         "no practical-profile admission, source-model theorem, loop termination proof or Linux release")
PENDING = [
    "W04 complete limits/overlay/wire/migration freeze and source-model review",
    "T02–T06 production evaluation/lvalue/call/loop bindings and ordinary proofs",
    "W05 complete T01 gate; T07 public activation",
]
FIXED = {
    "parallel-index": ({"i": 1, "s": [2, 5, 7]}, []),
    "duplicate-target": ({"s": [2]}, []),
    "slice-rebind-target": ({"s": [3, 4], "old": [9, 2]}, []),
    "array-rebind-target": ({"a": [9, 4], "b": [3, 4]}, []),
    "array-view-assignment": ({"a": [3, 4], "b": [8, 4], "view": [3, 4]}, []),
    "struct-rebind-target": ({"v": 9}, []),
    "struct-slice-rebind-target": ({"v": 3, "s": [3, 4], "old": [9, 2]}, []),
    "cell-field-merge": ({"a": 2, "b": 1}, []),
    "joint-call-once": ({"a": 7, "b": 9}, [1]),
    "discarded-result": ({"a": 7}, [1]),
    "forwarded-return": ({"a": 7, "b": 9}, [1]),
    "compound-once": ({"s": [17]}, [1, 2]),
    "assignment-calls-before-store-panic": ({"s": [7, 2], "panicked": True}, [1, 2]),
    "rhs-panic-before-writes": ({"s": [1, 2], "panicked": True}, [2, 1]),
    "switch-case-order": ({"v": 7}, [1, 2, 3]),
    "switch-default-first": ({"v": 7}, [1, 2]),
    "range-array-copy": ({"seen": [1, 2, 3], "a": [1, 20, 3]}, []),
    "range-array-descriptor-copy": ({"seen": [7, 7], "rebound": 9}, []),
    "range-slice-capture": ({"seen": [1, 20, 3], "s": [1]}, []),
    "range-slice-nil": ({"iterations": 0}, []),
    "range-assignment-target": ({"i": 0, "s": [3, 5, 3]}, []),
    "range-array-call-once": ({"sum": 3}, [1]),
    "range-blank-second-variable": ({"sum": 3}, [1]),
    "range-no-element-read": ({"sum": 3}, []),
    "array-size-call": ({"length": 3, "capacity": 3}, [1, 1]),
    "early-return-skips-post": ({"value": 7, "posts": 0}, []),
    "named-return-zero": ({"value": 0, "ok": False}, []),
    "byte-string-copy": ({"count": 3, "bytes": [0, 128, 120]}, []),
    "byte-string-append": ({"bytes": [7, 0, 128, 120], "capacity": 4}, []),
    "integer-defined-edges": (
        {"div": -(1 << 63), "rem": 0, "left": 0, "unsigned_right": 0, "signed_right": -1}, []),
    "aggregate-error-equality": ({"panicked": True}, []),
}
LOOPS = ("switch-break-target", "switch-continue-target", "nested-loop-break")


def inputs():
    rows = [{"id": identity, "n": 0, "flag": False} for identity in FIXED]
    rows += [{"id": "short-circuit", "n": 0, "flag": flag} for flag in (False, True)]
    rows += [{"id": identity, "n": n, "flag": False} for identity in LOOPS
             for n in (-1, 0, 1, 2, 5, 128, 129)]
    return rows


def expected(row):
    identity = row["id"]
    if identity in FIXED:
        value, trace = FIXED[identity]
        return {**value, "trace": trace}
    if identity == "short-circuit":
        return {"a": row["flag"], "b": True, "trace": [1] if row["flag"] else [2]}
    n = max(0, row["n"])
    if identity == "switch-break-target":
        return {"total": 100*n + 10*((n+1)//2) + n//2, "iterations": n, "trace": []}
    if identity == "switch-continue-target":
        return {"total": n//2, "posts": n, "trace": []}
    if identity == "nested-loop-break":
        return {"outer": n, "inner": n, "trace": []}
    raise ValueError("unknown expected evaluation case")


def vectors():
    return {
        "schema": "mpk.go_practical.w04_evaluation_vectors_draft.v0",
        "status": "native semantic observations; not profile admission or proof",
        "source": "develop/probes/go-practical/evaluation_reference.go",
        "scope": SCOPE,
        "observer_only": [
            "global trace counters and function literals used to expose evaluation",
            "recovery used to observe panic order; verified code still excludes recovery",
            "JSON result maps and observation aggregation outside verified functions",
        ],
        "cases": [{"input": row, "expected": expected(row)} for row in inputs()],
    }
