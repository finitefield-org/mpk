"""Source-shaped private overlay vectors; expected outcomes are design claims."""
import copy
import hashlib
from overlay_contracts import SOURCE_SCHEMA


def e(op, *args, **fields):
    return {"op": op, **({"args": list(args)} if args else {}), **fields}


def nat(n): return e("nat.literal", value=str(n))
def integer(n): return e("int.literal", value=str(n))
def truth(): return e("bool.literal", value=True)
def inp(n): return e("input", position=n)
def res(n): return e("result", position=n)
def local(n): return e("local", name=n)
def word(t, n): return e("word.literal", type=t, value=str(n))
def field(v, n): return e("field", value=v, field=n)
def observe(v): return e("int.from_signed", v)
def sum_lines(upper):
    line = e("old", value=e("element", inp(1), e("bound", name="j")))
    return e("int.sum", name="j", lower=nat(0), upper=upper, body=observe(field(line, "Amount")))


def base(function="Quote", receiver=None):
    return {"schema": SOURCE_SCHEMA, "source": "policy.go", "function": {"name": function, **({"receiver": receiver} if receiver else {})},
            "termination": "total", "requires": [], "ensures": [truth()], "error_inputs": [], "loops": []}


def selector(loop):
    return {k: loop[k] for k in ("kind", "start_byte", "end_byte", "ordinal")}


def vectors(facts):
    q = next(f for f in facts["functions"] if f["name"] == "Quote")
    s = next(f for f in facts["functions"] if f["name"] == "Scopes")
    cases = []
    def add(identity, overlay, expected="normalized_unproved", universe=("fixture/policy.RejectCode",), error=None):
        cases.append({"id": identity, "overlay": copy.deepcopy(overlay), "universe": list(universe), "expected": expected, "error_contains": error})
    quote = base()
    quote["requires"] = [e("le", e("length", inp(0)), nat(256))]
    total = observe(local("total"))
    quote["loops"] = [{"selector": selector(q["loops"][0]), "invariants": [
        e("bool.and", e("le", integer(0), total), e("le", total, observe(inp(2)))),
        e("eq", total, sum_lines(e("range.index"))),
    ]}]
    quote["ensures"] = [e("if", e("is_nil", res(1)),
        e("eq", observe(field(res(0), "Total")), sum_lines(e("length", inp(1)))),
        e("bool.and", e("error.is", value=res(1), case="fixture/policy.RejectAlias"), e("eq", field(res(0), "Total"), word("fixture/policy.Amount", 0))))]
    add("quote-prefix-sum-and-results", quote)
    partial = base("Scopes"); partial["termination"] = "partial"
    partial["error_inputs"] = [{"input": 4, "path": ["elements"], "cases": []}]
    add("missing-invariants-partial", partial)
    total_contract = copy.deepcopy(partial); total_contract["termination"] = "total"
    add("missing-for-variants-pending", total_contract, "normalized_pending_variant")
    explicit = copy.deepcopy(total_contract)
    explicit["loops"] = [{"selector": selector(l), "invariants": [e("eq", local("total"), local("total"))],
                          "variant": nat(0)} for l in s["loops"]]
    add("exact-header-shadowing", explicit)
    method = base("Error", "fixture/policy.RejectAlias")
    method["ensures"] = [e("eq", e("length", res(0)), nat(14))]
    add("value-receiver-alias", method)
    errors = base("InputErrors")
    errors["error_inputs"] = [{"input": 0, "path": ["field:1"], "cases": ["fixture/policy.RejectAlias"]},
                              {"input": 1, "path": ["elements"], "cases": []}]
    add("nested-and-tail-error-entry-sets", errors)
    for identity, mutate, msg in (
        ("missing-mode", lambda c: c.pop("termination"), "closed object fields"),
        ("old-schema", lambda c: c.update(schema="mpk.go.contract.v0"), "source overlay schema"),
        ("empty-ensures", lambda c: c.update(ensures=[]), "nonempty Boolean"),
        ("extra-root-field", lambda c: c.update(adapter="trusted"), "closed object fields"),
        ("external-write-annotation", lambda c: c.update(modifies=["lines"]), "read-only"),
        ("wrong-source-object", lambda c: c.update(source="else.go"), "source/language"),
        ("stale-loop-span", lambda c: c["loops"][0]["selector"].update(start_byte=q["loops"][0]["start_byte"]+1), "physical source LoopRef"),
        ("duplicate-loop", lambda c: c["loops"].append(copy.deepcopy(c["loops"][0])), "duplicate loop"),
        ("empty-explicit-invariant", lambda c: c["loops"][0].update(invariants=[]), "nonempty Boolean"),
        ("word-variant", lambda c: c["loops"][0].update(variant=word("int", 0)), "Nat-valued variant"),
        ("result-in-invariant", lambda c: c["loops"][0].update(invariants=[e("is_nil", res(1))]), "only at same invocation return"),
        ("range-declared-variable-before-first-assignment", lambda c: c["loops"][0].update(invariants=[e("eq", local("line"), local("line"))]), "live lexical header binding"),
        ("local-in-postcondition", lambda c: c.update(ensures=[e("eq", local("total"), word("fixture/policy.Amount", 0))]), "local unavailable"),
        ("source-call-in-overlay", lambda c: c.update(ensures=[e("call", function="arbitrary")]), "closed practical v1"),
        ("result-under-old", lambda c: c.update(ensures=[e("is_nil", e("old", value=res(1)))]), "only at same invocation return"),
        ("unbounded-quantifier", lambda c: c.update(ensures=[e("forall", name="j", body=truth())]), "closed object fields"),
    ):
        value = copy.deepcopy(quote); mutate(value); add(identity, value, "configuration_rejection", error=msg)
    missing = copy.deepcopy(errors); missing["error_inputs"].pop()
    add("unconstrained-nested-error-input", missing, "configuration_rejection", error="all nested error-bearing")
    add("nonplain-global-universe-for-error-cells", partial, "configuration_rejection", universe=("fixture/policy.RejectCode", "fixture/policy.SliceError"), error="plain backing cell")
    return cases


def capture_id(case, source_sha):
    # Private identity DAG demonstration. Raw source overlay precedes the
    # derived contract; no digest of a normalized contract enters its capture.
    import json
    payload = json.dumps({"source": source_sha, "raw_overlay": case["overlay"], "universe": case["universe"]}, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(b"mpk.private.overlay-capture-probe.v0\0" + payload).hexdigest()
