"""Independent complete public interfaces and computed history definitions."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("history_certificate_decoder", ROOT.parent / "source-invocations/structural-oracle.py")
decoder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(decoder)
f = decoder.f
freeze = decoder.m.frozen_ast
P = "Mpk.Go.Practical.ExecutionHistory."


def call(name, *args): return f.app(f.const(name), *args)
def own(name, *args): return call(P + name, *args)
def pair(a, b): return call("Mpk.Go.Practical.Product.Type", a, b)
def history(a): return f.pi("i", f.nat(), a)
def predicate(a): return f.pi("x", a, {"kind": "sort0"})
def relation(a): return f.pi("x", a, f.pi("y", a, {"kind": "sort0"}))
def at(h, k): return f.app(h, k)
def succ(k): return call("Std.Nat.succ", k)
def below(k, n): return f.eq(f.boolean(), call("Mpk.Go.Practical.Nat.Less", k, n), f.bit(True))
def start(a, h, initial): return f.eq(a, at(h, f.const("Std.Nat.zero")), initial)
def path(r, h, n):
    k = f.var("k")
    return f.pi("k", f.nat(), f.pi("bound", below(k, n), f.app(r, at(h, k), at(h, succ(k)))))
def safe(a, r, initial, prop):
    n, h = f.var("n"), f.var("h")
    return f.bind("pi", [("n", f.nat()), ("h", history(a)), ("start", start(a, h, initial)), ("path", path(r, h, n))], f.app(prop, at(h, n)))
def normal_post(a, r, initial, normal, post):
    n, h = f.var("n"), f.var("h")
    return f.bind("pi", [("n", f.nat()), ("h", history(a)), ("start", start(a, h, initial)), ("path", path(r, h, n)), ("normal", f.app(normal, at(h, n)))], f.app(post, at(h, n)))
def run(a, r, initial, normal, h, n): return pair(start(a, h, initial), pair(path(r, h, n), f.app(normal, at(h, n))))
def preserves(a, r, prop):
    x, y = f.var("x"), f.var("y")
    return f.bind("pi", [("x", a), ("y", a), ("edge", f.app(r, x, y)), ("valid", f.app(prop, x))], f.app(prop, y))


def inspect(cert):
    raw, module, table, term = decoder.decode(cert)
    sort = {"kind": "sort0"}
    a, b, r, s, h, n, p, initial = [f.var(x) for x in ["A", "B", "R", "S", "h", "n", "P", "initial"]]
    basic = [("A", sort), ("R", relation(a))]
    checked = []
    def check(name, args, result, value=None, law=False):
        d = table[P + name]
        assert d[1] == (2 if law else 1), name
        assert term(d[2]) == freeze(f.bind("pi", args, result)), (name, "complete public type")
        if value is not None:
            assert term(d[3]) == freeze(f.bind("lam", args, value)), (name, "complete computed definition")
        checked.append(P + name)
    check("History", [("A", sort)], sort, history(a))
    check("Path", basic + [("h", history(a)), ("n", f.nat())], sort, path(r, h, n))
    check("Starts", [("A", sort), ("h", history(a)), ("initial", a)], sort, start(a, h, initial))
    check("SafePrefixes", basic + [("initial", a), ("P", predicate(a))], sort, safe(a, r, initial, p))
    params = basic + [("initial", a), ("Normal", predicate(a)), ("Post", predicate(a))]
    post = normal_post(a, r, initial, f.var("Normal"), f.var("Post"))
    check("NormalPost", params, sort, post)
    check("PartialCorrectness", params + [("Safe", predicate(a))], sort, pair(safe(a, r, initial, f.var("Safe")), post))
    check("NormalRun", params[:-1] + [("h", history(a)), ("n", f.nat())], sort, run(a, r, initial, f.var("Normal"), h, n))
    check("Tail", [("A", sort), ("h", history(a))], history(a), f.lam("k", f.nat(), at(h, succ(f.var("k")))))
    check("Prepend", [("A", sort), ("initial", a), ("h", history(a))], history(a),
          f.lam("k", f.nat(), call("Std.Nat.induct", f.lam("position", f.nat(), a), initial,
                                    f.lam("position", f.nat(), f.lam("ignored", a, at(h, f.var("position")))), f.var("k"))))
    mapping = [("A", sort), ("B", sort), ("f", f.pi("x", a, b)), ("h", history(a))]
    check("Map", mapping, history(b), f.lam("k", f.nat(), f.app(f.var("f"), at(h, f.var("k")))))
    check("PathZero", basic + [("h", history(a))], path(r, h, f.const("Std.Nat.zero")), law=True)
    params = basic + [("h", history(a)), ("n", f.nat()), ("path", path(r, h, succ(n)))]
    check("PathHead", params, f.app(r, at(h, f.const("Std.Nat.zero")), at(h, succ(f.const("Std.Nat.zero")))), law=True)
    check("PathTail", params, path(r, own("Tail", a, h), n), law=True)
    params = params[:-1] + [("initial", a), ("first", f.app(r, initial, at(h, f.const("Std.Nat.zero")))), ("path", path(r, h, n))]
    check("PathPrepend", params, path(r, own("Prepend", a, initial, h), succ(n)), law=True)
    trace = [("h", history(a)), ("initial_valid", f.app(p, at(h, f.const("Std.Nat.zero")))), ("path", path(r, h, n))]
    check("InvariantHistory", basic + [("P", predicate(a)), ("preserve", preserves(a, r, p)), ("n", f.nat())], f.bind("pi", trace, f.app(p, at(h, n))), law=True)
    inv_params = basic + [("P", predicate(a)), ("initial", a), ("initial_valid", f.app(p, initial)), ("preserve", preserves(a, r, p))]
    check("InvariantPrefixes", inv_params, safe(a, r, initial, p), law=True)
    weaken = basic + [("initial", a), ("P", predicate(a)), ("Q", predicate(a)), ("safe", safe(a, r, initial, p)),
                      ("implies", f.pi("x", a, f.pi("proof", f.app(p, f.var("x")), f.app(f.var("Q"), f.var("x")))))]
    check("WeakenSafety", weaken, safe(a, r, initial, f.var("Q")), law=True)
    respects = f.bind("pi", [("x", a), ("y", a), ("edge", f.app(r, f.var("x"), f.var("y")))], f.app(s, f.app(f.var("f"), f.var("x")), f.app(f.var("f"), f.var("y"))))
    mapped = own("Map", a, b, f.var("f"), h)
    check("MapPath", [("A", sort), ("B", sort), ("R", relation(a)), ("S", relation(b)), ("f", f.pi("x", a, b)), ("respects", respects), ("h", history(a)), ("n", f.nat()), ("path", path(r, h, n))], path(s, mapped, n), law=True)
    check("MapStarts", mapping + [("initial", a), ("start", start(a, h, initial))], start(b, mapped, f.app(f.var("f"), initial)), law=True)
    params = basic + [("initial", a), ("Normal", predicate(a)), ("h", history(a)), ("n", f.nat()), ("run", run(a, r, initial, f.var("Normal"), h, n))]
    check("RunStarts", params, start(a, h, initial), law=True)
    check("RunPath", params, path(r, h, n), law=True)
    check("RunNormal", params, f.app(f.var("Normal"), at(h, n)), law=True)
    safety = safe(a, r, initial, f.var("Safe"))
    params = basic + [("initial", a), ("Safe", predicate(a)), ("Normal", predicate(a)), ("Post", predicate(a)), ("partial", pair(safety, post))]
    check("PartialSafety", params, safety, law=True)
    check("PartialPost", params, post, law=True)
    check("ObserveNormalRun", params + [("h", history(a)), ("n", f.nat()), ("run", run(a, r, initial, f.var("Normal"), h, n))], pair(f.app(f.var("Safe"), at(h, n)), f.app(f.var("Post"), at(h, n))), law=True)
    inv = f.var("Inv")
    params = basic + [("initial", a), ("Inv", predicate(a)), ("Safe", predicate(a)), ("Normal", predicate(a)), ("Post", predicate(a)), ("initial_valid", f.app(inv, initial)), ("preserve", preserves(a, r, inv)),
                      ("safe_implies", f.pi("x", a, f.pi("valid", f.app(inv, f.var("x")), f.app(f.var("Safe"), f.var("x"))))),
                      ("post_implies", f.pi("x", a, f.pi("valid", f.app(inv, f.var("x")), f.pi("normal", f.app(f.var("Normal"), f.var("x")), f.app(f.var("Post"), f.var("x"))))))]
    check("PartialFromInvariant", params, pair(safety, post), law=True)
    check("SafetyAtEntry", basic + [("initial", a), ("P", predicate(a)), ("safe", safe(a, r, initial, p))], f.app(p, initial), law=True)
    assert {name for name in table if name.startswith(P)} == set(checked)
    return {"path": str(cert), "sha256": hashlib.sha256(raw).hexdigest(), "module": module,
            "complete_definition_bodies": 10, "complete_theorem_types": 17, "exact_export_set": len(checked),
            "declarations": len(table), "no_observation_or_fuel_bound_in_semantics": True}


if __name__ == "__main__":
    cert = Path(sys.argv[1])
    result = inspect(cert)
    output = ROOT / "statement-review.json"
    with output.open("x") as stream:
        json.dump({"status": "passed", "driver_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), "inspection": result}, stream, indent=2)
        stream.write("\n")
    print(json.dumps(result))
