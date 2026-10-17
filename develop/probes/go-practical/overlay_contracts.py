"""Private closed v1 overlay normalizer/type probe; no production admission.

Compiler facts are untrusted inputs. This probe tests design consistency; it
does not prove the facts, source/model correspondence or expression semantics.
"""
import copy
import re
from dataclasses import dataclass

BOOL, NAT, INT = "$Bool", "$Nat", "$Int"
SOURCE_SCHEMA = "mpk.go_practical.source_overlay_draft.v0"
NORMAL_SCHEMA = "mpk.go_practical.normalized_overlay_draft.v0"
LANGUAGES = {"go1.22", "go1.23", "go1.24", "go1.25"}


class OverlayError(ValueError):
    pass


class ProbeResourceLimit(RuntimeError):
    pass


def need(condition, reason):
    if not condition:
        raise OverlayError(reason)


def shape(value, required, optional=()):
    need(type(value) is dict and set(required) <= set(value) <= set(required) | set(optional), "closed object fields")


def natural(value):
    need(type(value) is int and value >= 0, "nonnegative JSON integer")
    return value


def decimal(value, signed=False):
    pattern = r"(?:0|[1-9][0-9]*|-[1-9][0-9]*)" if signed else r"(?:0|[1-9][0-9]*)"
    need(type(value) is str and re.fullmatch(pattern, value), "canonical decimal literal")
    # Operational construction guard only, never a bound on Nat semantics.
    if len(value) > 4096:
        raise ProbeResourceLimit("private literal construction ceiling; not semantic rejection")
    return int(value)


class Types:
    def __init__(self, facts, universe):
        self.rows = {r["id"]: r for r in facts["types"]}
        need(len(self.rows) == len(facts["types"]), "unique source types")
        self.aliases = facts.get("aliases", {})
        self.universe = tuple(sorted(self.resolve(t) for t in universe))
        need(len(self.universe) == len(set(self.universe)), "unique closed error cases")
        for t in self.universe:
            r = self.rows[t]
            need(r["kind"] == "named" and r.get("implements_error") is True
                 and self.base(t)["kind"] in {"bool", "word", "struct"}, "application nonpointer concrete error case")
            self.root(t)

    def resolve(self, t):
        need(type(t) is str, "source type reference")
        t = self.aliases.get(t, t)
        need(t in self.rows, "known captured source type")
        return t

    def base(self, t):
        t = self.resolve(t)
        visited = set()
        while self.rows[t]["kind"] == "named":
            need(t not in visited, "acyclic named type graph")
            visited.add(t)
            t = self.resolve(self.rows[t]["underlying"])
        return self.rows[t]

    def root(self, t, plain=False, trail=()):
        t = self.resolve(t)
        need(t not in trail, "acyclic expanded root graph")
        r = self.rows[t]
        trail = trail + (t,)
        k = r["kind"]
        if k == "named":
            return self.root(r["underlying"], plain, trail)
        if k in {"bool", "word", "string"}:
            return
        if k == "struct":
            for f in r.get("fields", []):
                self.root(f["type"], plain, trail)
            return
        if k == "array":
            natural(r.get("length", 0))
            self.root(r["element"], plain, trail)
            return
        if k == "slice":
            need(not plain, "plain backing cell cannot contain mutable reference")
            self.root(r["element"], True, trail)
            return
        if k == "error":
            for case in self.universe:
                self.root(case, plain, trail)
            return
        raise OverlayError("source type outside practical v1 root domain")

    def comparable(self, t, trail=()):
        t = self.resolve(t)
        need(t not in trail, "acyclic equality graph")
        r = self.rows[t]
        k = r["kind"]
        if k == "named":
            return self.comparable(r["underlying"], trail + (t,))
        if k in {"bool", "word", "string"}:
            return True
        if k == "struct":
            return all(self.comparable(f["type"], trail + (t,)) for f in r.get("fields", []))
        if k == "array":
            return self.comparable(r["element"], trail + (t,))
        return False

    def errors(self, t, path=()):
        r = self.base(t)
        if r["kind"] == "error":
            return [path]
        if r["kind"] == "struct":
            return [p for f in r.get("fields", []) for p in self.errors(f["type"], path + (f["key"],))]
        if r["kind"] in {"array", "slice"}:
            return self.errors(r["element"], path + ("elements",))
        return []


@dataclass(frozen=True)
class Context:
    phase: str
    inputs: tuple
    results: tuple
    bindings: tuple = ()
    loop_kind: str = ""
    entry: bool = False
    variables: tuple = ()


UNARY = {
    "bool.not": (BOOL, BOOL), "nat.to_int": (NAT, INT), "int.neg": (INT, INT),
}
BINARY = {
    **{op: (BOOL, BOOL) for op in ("bool.and", "bool.or", "bool.implies")},
    **{op: (NAT, NAT) for op in ("nat.add", "nat.mul", "nat.sub")},
    **{op: (INT, INT) for op in ("int.add", "int.sub", "int.mul")},
}
WORD_UNARY = {"word.neg", "word.not"}
WORD_BINARY = {"word.add", "word.sub", "word.mul", "word.and", "word.or", "word.xor", "word.andnot", "word.div", "word.rem", "word.shl", "word.shr"}
COMPARISONS = {"eq", "ne", "lt", "le", "gt", "ge"}
QUANTIFIERS = {"forall", "exists", "nat.sum", "int.sum"}
OTHER_OPERATORS = {"bool.literal", "nat.literal", "int.literal", "word.literal", "bytes.literal", "source.zero",
                   "input", "result", "local", "bound", "range.index", "range.length", "old", "if", "word.convert",
                   "bool.value", "nat.from_unsigned", "nat.from_signed", "int.from_unsigned", "int.from_signed", "field",
                   "length", "capacity", "is_nil", "byte_at", "element", "bytes.prefix", "bytes.suffix", "bytes.concat",
                   "bytes.slice", "slice.view2", "slice.view3", "error.is", "error.payload", "sequence.equal"}
OPERATORS = set(UNARY) | set(BINARY) | WORD_UNARY | WORD_BINARY | COMPARISONS | QUANTIFIERS | OTHER_OPERATORS


class Expressions:
    def __init__(self, types):
        self.types = types
        self.nodes = 0

    def check(self, e, ctx, depth=0):
        self.nodes += 1
        if depth > 128 or self.nodes > 16384:
            raise ProbeResourceLimit("private AST construction ceiling; not semantic rejection")
        need(type(e) is dict and type(e.get("op")) is str, "expression object/operator")
        op = e["op"]
        source = self.types
        rec = lambda child, context=ctx: self.check(child, context, depth + 1)
        norm = copy.deepcopy(e)

        def args(count):
            shape(e, ("op", "args"))
            need(type(e["args"]) is list and len(e["args"]) == count, "exact operator arity")
            checked = [rec(x) for x in e["args"]]
            norm["args"] = [v[0] for v in checked]
            return [v[1] for v in checked]

        def kind(t, allowed):
            need(type(t) is str and t not in {BOOL, NAT, INT} and source.base(t)["kind"] in allowed, "source operand domain")
            source.root(t)

        if op == "bool.literal":
            shape(e, ("op", "value")); need(type(e["value"]) is bool, "Boolean literal")
            return norm, BOOL
        if op in {"nat.literal", "int.literal"}:
            shape(e, ("op", "value")); decimal(e["value"], op == "int.literal")
            return norm, NAT if op == "nat.literal" else INT
        if op in {"word.literal", "bytes.literal", "source.zero"}:
            shape(e, ("op", "type") if op == "source.zero" else ("op", "type", "value"))
            t = source.resolve(e["type"]); source.root(t); norm["type"] = t
            if op == "word.literal":
                kind(t, {"word"}); r = source.base(t); n = decimal(e["value"], r.get("signed", False)); w = r["width"]
                lo, hi = (-(1 << (w-1)), (1 << (w-1))-1) if r.get("signed", False) else (0, (1 << w)-1)
                need(lo <= n <= hi, "word literal representability")
            if op == "bytes.literal":
                kind(t, {"string"}); need(type(e["value"]) is str and re.fullmatch(r"(?:[0-9a-f]{2})*", e["value"]), "canonical raw byte literal")
                need(len(e["value"]) <= 512, "candidate string carrier bound")
            return norm, t
        if op in {"input", "result"}:
            shape(e, ("op", "position")); n = natural(e["position"])
            if op == "result":
                need(ctx.phase == "ensures" and not ctx.entry, "result exists only at same invocation return")
            slots = ctx.inputs if op == "input" else ctx.results
            need(n < len(slots), "positional slot bounds"); t = slots[n]["type"]; source.root(t)
            return norm, t
        if op == "local":
            shape(e, ("op", "name")); need(ctx.phase == "header" and not ctx.entry, "local unavailable in this phase")
            matches = [b for b in ctx.bindings if b["name"] == e["name"]]
            need(len(matches) == 1, "exact live lexical header binding")
            b = matches[0]; source.root(b["type"])
            return {"op": "binding", "id": b["id"]}, b["type"]
        if op == "bound":
            shape(e, ("op", "name")); variables = dict(ctx.variables)
            need(type(e["name"]) is str and e["name"] in variables, "lexical logical binder")
            return norm, variables[e["name"]]
        if op in {"range.index", "range.length"}:
            shape(e, ("op",)); need(ctx.phase == "header" and ctx.loop_kind == "range" and not ctx.entry, "range observation only at own current header")
            return norm, NAT
        if op == "old":
            shape(e, ("op", "value")); need(not ctx.entry, "nested entry snapshot")
            n, t = rec(e["value"], Context(ctx.phase, ctx.inputs, ctx.results, ctx.bindings, ctx.loop_kind, True, ctx.variables))
            return {"op": op, "value": n}, t
        if op == "if":
            ts = args(3); need(ts[0] == BOOL and ts[1] == ts[2], "typed conditional branches")
            return norm, ts[1]
        if op in UNARY:
            ts = args(1); need(ts[0] == UNARY[op][0], "unary logical domain")
            return norm, UNARY[op][1]
        if op in BINARY:
            ts = args(2); need(ts == [BINARY[op][0]]*2, "binary logical domain")
            return norm, BINARY[op][1]
        if op in COMPARISONS:
            ts = args(2); need(ts[0] == ts[1], "exact nominal/comparison type")
            t = ts[0]
            if op in {"eq", "ne"}:
                need(t in {BOOL, NAT, INT} or (type(t) is str and source.comparable(t)), "closed equality domain; error/slice equality excluded")
            elif t not in {NAT, INT}:
                kind(t, {"word", "string"})
            return norm, BOOL
        if op in WORD_UNARY | WORD_BINARY:
            ts = args(1 if op in WORD_UNARY else 2)
            for t in ts: kind(t, {"word"})
            if len(ts) == 2 and op not in {"word.shl", "word.shr"}:
                need(ts[0] == ts[1], "exact nominal word operands")
            return norm, ts[0]
        if op == "word.convert":
            shape(e, ("op", "type", "value")); n, t = rec(e["value"]); kind(t, {"word"})
            out = source.resolve(e["type"]); kind(out, {"word"})
            return {"op": op, "type": out, "value": n}, out
        if op in {"bool.value", "nat.from_unsigned", "nat.from_signed", "int.from_unsigned", "int.from_signed"}:
            ts = args(1); kind(ts[0], {"bool"} if op == "bool.value" else {"word"})
            if op != "bool.value":
                need(source.base(ts[0]).get("signed", False) == op.endswith("from_signed"), "signed/unsigned bridge domain")
            return norm, BOOL if op == "bool.value" else NAT if op.startswith("nat.") else INT
        if op == "field":
            shape(e, ("op", "value", "field")); n, t = rec(e["value"]); kind(t, {"struct"})
            need(type(e["field"]) is str and e["field"] != "_", "exact nonpromoted field")
            matches = [f for f in source.base(t).get("fields", []) if f["name"] == e["field"]]
            need(len(matches) == 1, "exact nonpromoted field")
            f = matches[0]
            return {"op": op, "value": n, "field": f["key"], "owner_type": t}, f["type"]
        if op in {"length", "capacity", "is_nil", "byte_at", "element"}:
            ts = args(2 if op in {"byte_at", "element"} else 1)
            kind(ts[0], {"array", "slice", "string"} if op == "length" else {"array", "slice"} if op in {"capacity", "element"} else {"slice", "error"} if op == "is_nil" else {"string"})
            if len(ts) == 2: need(ts[1] == NAT, "mathematical index type")
            out = BOOL if op == "is_nil" else source.base(ts[0])["element"] if op == "element" else source.resolve("uint8") if op == "byte_at" else NAT
            return norm, out
        if op in {"bytes.prefix", "bytes.suffix", "bytes.concat", "bytes.slice"}:
            ts = args(3 if op == "bytes.slice" else 2); kind(ts[0], {"string"})
            if op == "bytes.slice": need(ts[1:] == [NAT, NAT], "substring indices")
            else: need(ts[0] == ts[1], "exact string nominal type")
            return norm, BOOL if op in {"bytes.prefix", "bytes.suffix"} else ts[0]
        if op in {"slice.view2", "slice.view3"}:
            ts = args(3 if op == "slice.view2" else 4); kind(ts[0], {"slice"})
            need(ts[1:] == [NAT]*(len(ts)-1), "reslice mathematical indices")
            return norm, ts[0]
        if op in {"error.is", "error.payload"}:
            shape(e, ("op", "value", "case")); n, t = rec(e["value"]); kind(t, {"error"})
            case = source.resolve(e["case"]); need(case in source.universe, "case in closed error universe")
            return {"op": op, "value": n, "case": case}, BOOL if op == "error.is" else case
        if op == "sequence.equal":
            ts = args(2)
            for t in ts: kind(t, {"array", "slice"})
            a, b = (source.base(t)["element"] for t in ts)
            need(a == b and source.comparable(a), "exact comparable sequence element type")
            return norm, BOOL
        if op in QUANTIFIERS:
            shape(e, ("op", "name", "lower", "upper", "body"))
            name = e["name"]; need(type(name) is str and re.fullmatch(r"[a-z][a-z0-9_]{0,63}", name) and name not in dict(ctx.variables), "fresh logical binder")
            lo, lot = rec(e["lower"]); hi, hit = rec(e["upper"]); need(lot == hit == NAT, "finite Nat interval")
            inner = Context(ctx.phase, ctx.inputs, ctx.results, ctx.bindings, ctx.loop_kind, ctx.entry, ctx.variables + ((name, NAT),))
            body, bodyt = rec(e["body"], inner)
            out = NAT if op == "nat.sum" else INT if op == "int.sum" else BOOL
            need(bodyt == out, "bounded binder body type")
            return {"op": op, "name": name, "lower": lo, "upper": hi, "body": body}, out
        raise OverlayError("operator outside closed practical v1 contract vocabulary")


def normalize(overlay, facts, capture_context, universe):
    shape(overlay, ("schema", "source", "function", "termination", "requires", "ensures", "error_inputs", "loops"), ("modifies",))
    need(overlay["schema"] == SOURCE_SCHEMA, "new private source overlay schema")
    need(overlay["source"] == facts["path"] and facts["language"] in LANGUAGES, "exact captured source/language")
    need(type(capture_context) is str and re.fullmatch(r"[0-9a-f]{64}", capture_context), "capture context identity")
    selector = overlay["function"]
    shape(selector, ("name",), ("receiver",))
    receiver = facts.get("aliases", {}).get(selector.get("receiver"), selector.get("receiver")) if type(selector.get("receiver")) is str else None
    need("receiver" not in selector or receiver is not None, "source receiver type reference")
    functions = [f for f in facts["functions"] if f["name"] == selector["name"] and f.get("receiver") == receiver]
    need(len(functions) == 1, "unique captured function")
    f = functions[0]
    need(len(f["results"]) <= 16, "candidate joint result arity")
    need(type(overlay["termination"]) is str and overlay["termination"] in {"partial", "total"}, "explicit partial/total mode")
    need(overlay.get("modifies", []) == [], "v1 input storage is read-only; no ownership assertion")
    types = Types(facts, universe)
    for b in f["inputs"] + f["results"]: types.root(b["type"])
    expression = Expressions(types)
    inputs, results = tuple(f["inputs"]), tuple(f["results"])
    def clauses(values, ctx, nonempty=False):
        need(type(values) is list and (not nonempty or len(values) > 0), "nonempty Boolean clause list" if nonempty else "Boolean clause list")
        out = []
        for e in values:
            n, t = expression.check(e, ctx); need(t == BOOL, "clause predicate is logical Bool"); out.append(n)
        return out
    requires = clauses(overlay["requires"], Context("requires", inputs, results))
    ensures = clauses(overlay["ensures"], Context("ensures", inputs, results), True)
    expected_paths = {(i, p) for i, b in enumerate(inputs) for p in types.errors(b["type"])}
    need(type(overlay["error_inputs"]) is list, "explicit nested error entry sets")
    error_inputs, supplied = [], set()
    for row in overlay["error_inputs"]:
        shape(row, ("input", "path", "cases")); i = natural(row["input"])
        need(type(row["path"]) is list and all(type(p) is str for p in row["path"]), "typed entry selector path")
        key = (i, tuple(row["path"])); need(key in expected_paths and key not in supplied, "exact unique error-bearing entry path")
        supplied.add(key); need(type(row["cases"]) is list, "explicit closed error entry set")
        cases = sorted(types.resolve(t) for t in row["cases"])
        need(len(cases) == len(set(cases)) and set(cases) <= set(types.universe), "entry set within same closed error universe")
        error_inputs.append({"input": i, "path": row["path"], "cases": cases})
    need(supplied == expected_paths, "all nested error-bearing inputs explicitly constrained")
    error_inputs.sort(key=lambda r: (r["input"], r["path"]))
    need(type(overlay["loops"]) is list, "loop overlay list")
    supplied_loops = {}
    for row in overlay["loops"]:
        shape(row, ("selector", "invariants"), ("variant",)); sel = row["selector"]
        shape(sel, ("kind", "start_byte", "end_byte", "ordinal"))
        matches = [l for l in f["loops"] if all(l[k] == sel[k] and type(l[k]) is type(sel[k]) for k in sel)]
        need(len(matches) == 1, "exact physical source LoopRef selector")
        ordinal = matches[0]["ordinal"]; need(ordinal not in supplied_loops, "duplicate loop overlay")
        supplied_loops[ordinal] = row
    loops, pending = [], []
    for l in f["loops"]:
        row = supplied_loops.get(l["ordinal"])
        ctx = Context("header", inputs, results, tuple(l["bindings"]), l["kind"])
        invariants = clauses(row["invariants"], ctx, True) if row else [{"op": "bool.literal", "value": True}]
        variant = None
        if row and "variant" in row:
            variant, t = expression.check(row["variant"], ctx); need(t == NAT, "one Nat-valued variant")
        elif overlay["termination"] == "total":
            if l["kind"] == "range": variant = {"op": "nat.sub", "args": [{"op": "range.length"}, {"op": "range.index"}]}
            else: pending.append({"ordinal": l["ordinal"], "reason": "missing checked variant candidate"})
        ref = {"capture_context_id": capture_context, "source_object_id": facts["source_sha256"],
               "function_decl_id": f"decl:{f['start_byte']}:{f['end_byte']}", "ast_kind": l["kind"],
               "start_byte": l["start_byte"], "end_byte": l["end_byte"], "function_preorder_ordinal": l["ordinal"], "effective_language": facts["language"]}
        measure = {"kind": "candidate", "origin": "supplied" if row and "variant" in row else "generated_range", "expression": variant} if variant else {"kind": "absent"}
        loops.append({"loop_ref": ref, "invariants": invariants, "invariant_origin": "supplied" if row else "generated_candidate", "variant": measure})
    return {"schema": NORMAL_SCHEMA, "capture_context_id": capture_context, "source_object_id": facts["source_sha256"],
            "function_decl_id": f"decl:{f['start_byte']}:{f['end_byte']}", "termination": overlay["termination"],
            "requires": requires, "ensures": ensures, "error_inputs": error_inputs, "modifies": [], "loops": loops,
            "entry_storage": {"input_access": "read_only", "concurrent_external_mutation": "forbidden"},
            "pending_candidates": pending, "proof_status": "unproved design candidate; all source/safety/domain/frame/post/termination members remain required"}
