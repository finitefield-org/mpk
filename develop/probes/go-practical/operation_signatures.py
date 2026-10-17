"""Concrete signatures for all56 Go operation families; no execution/source proof."""
import copy
import json
from pathlib import Path
from contract_scope import Scope, need, exact
from contract_wire import canonical, digest
from closed_carriers import sealed, Specializer
from run_proof_pilot import ROOT, HERE, DEFAULT_GOROOT, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w04/operation-signatures"
TABLE = RECORD.parent / "operation-table.json"
LIMITS = RECORD.parent / "structural-limits/limits.json"
CAPS = {r["id"]: r["maximum"] for r in json.loads(LIMITS.read_bytes())["counters"]}
SYNTAX_MAX = CAPS["capture.typed_syntax_nodes_across_included_packages"]
SCHEMA = "mpk.go.operation_definition.v1"
DOMAIN = "MPK-GO-OPERATION-DEFINITION-1.0"
ROOT_FIELDS = ("schema", "semantic_context", "capture_context_id", "source_scope_sha256",
               "function_decl_id", "signature_id", "type_definitions_sha256", "error_universe_id",
               "operation_table_sha256", "op", "parameters", "inputs", "outputs", "regions",
               "obligations", "binding_requirements", "operation_hash")
COMMON = ("source_binding", "input_validity", "evaluation", "safety_domain_permission",
          "value_effect_output_validity", "frame_joint_invocation")
PARAMETERS = {
    1: ("type",), 2: ("type", "value"), 3: ("from", "to"),
    4: ("type", "operator"), 5: ("type", "operator"), 6: ("type", "operator"),
    7: ("type", "count_type", "operator"), 8: ("type", "operator"),
    9: ("type", "operator"), 10: ("type", "initializers"), 11: ("type", "field"),
    12: ("type",), 13: ("type", "index_type"), 14: ("type", "operator"),
    15: ("condition_type",), 16: ("type", "case_arities", "default_index"),
    17: ("types",), 18: ("callee", "index"), 19: ("callee",), 20: (),
    21: ("targets", "types"), 22: ("type", "rhs_type", "operator"),
    23: (), 24: ("type",), 25: (), 26: (), 27: ("type", "bytes_hex"),
    28: ("type", "operator"), 29: ("type",), 30: ("type", "index_type"),
    31: ("type", "lo_type", "hi_type"), 32: ("type",), 33: (), 34: (),
    35: ("type",), 36: ("type",), 37: ("type", "size"), 38: ("type",),
    39: ("type", "index_type"), 40: ("type", "lo_type", "hi_type"),
    41: ("type", "lo_type", "hi_type", "max_type"),
    42: ("type", "slice_type", "lo_type", "hi_type", "max_type"),
    43: ("type", "len_type", "cap_type"), 44: ("type", "initializers"),
    45: ("type", "index_type"), 46: ("destination", "source"),
    47: ("type",), 48: ("type", "count"), 49: ("destination", "source"),
    50: ("type",), 51: ("loop", "condition_type", "has_test"),
    52: ("loop", "type", "index", "value", "evaluation"), 53: ("loop", "type", "index", "value"),
    54: ("enclosing",), 55: ("enclosing",), 56: ("type", "size"),
}
UNARY = {"plus", "negate", "complement"}
ARITH = {"add", "subtract", "multiply", "and", "or", "xor", "and_not"}
COMPARE = {"equal", "not_equal", "less", "less_equal", "greater", "greater_equal"}
COMPOUND = ARITH | {"divide", "remainder", "left", "right"}


def uint(x, limit=(1 << 32)-1):
    need(type(x) is int and 0 <= x <= limit, "bounded canonical unsigned integer")
    return x


def source(t): return {"kind": "source", "source_type_id": t}
def location(t): return {"kind": "location", "source_type_id": t}
def joint(ts): return {"kind": "joint", "result_types": list(ts)}
STATE = {"kind": "state"}


class Signatures:
    def __init__(self, scope):
        scope.assert_integrity()
        # Retain original bytes, not a mutable caller fact map, as the authority.
        self.raw_bytes = scope.raw_bytes
        self.universe = tuple(scope.types.universe)
        self.overlay_path = scope.overlay_path
        self.scope = Scope(json.loads(self.raw_bytes), self.universe, self.overlay_path, self.raw_bytes)
        self.defs = self.scope.value["type_definitions"]
        self.specializer = Specializer(self.defs, self.scope.context)
        self.rows = self.specializer.rows
        self.nodes = self.specializer.nodes
        self.ids = self.scope.ids
        self.table = json.loads(TABLE.read_bytes())
        self.operations = {r["id"]: r for r in self.table["operations"]}
        self.callees = {n["declaration_id"]: n for n in self.scope.graph
                        if n["schema"] == "mpk.go.signature.v1"}
        self.fn_results = [self.id(t["type"]) for t in self.scope.function["results"]]
        for binding in self.scope.function["inputs"] + self.scope.function["results"]:
            self.root(self.id(binding["type"]))

    def id(self, name): return self.ids[self.scope.types.resolve(name)]

    def root(self, t, kinds=None):
        need(type(t) is str and t in self.rows, "captured concrete nominal type")
        self.specializer.source(t)
        r = self.rows[t]
        while r["kind"] == "named": r = self.rows[r["underlying"]]
        if kinds is not None: need(r["kind"] in kinds, "operation source kind")
        return r

    def integer(self, t): return self.root(t, {"word"})
    def value(self, t): self.root(t); return source(t)

    def underlying(self, t):
        self.root(t)
        while self.rows[t]["kind"] == "named": t = self.rows[t]["underlying"]
        return t

    def identical(self, a, b):
        # Go identity ignores struct tags for explicit conversions, not names.
        if a == b: return True
        x, y = self.nodes[a], self.nodes[b]
        if x.get("kind") != y.get("kind") or x.get("kind") in {"nominal", "builtin"}: return False
        if x["kind"] in {"array", "slice"}:
            return x.get("length") == y.get("length") and self.identical(x["element"], y["element"])
        if x["kind"] == "struct":
            return len(x["fields"]) == len(y["fields"]) and all(
                f["name"] == g["name"] and f["unexported_package_id"] == g["unexported_package_id"]
                and self.identical(f["type_id"], g["type_id"]) for f, g in zip(x["fields"], y["fields"]))
        return False

    def assignable(self, a, b):
        # Assignment uses tag-sensitive source identity, unlike conversion.
        self.root(a); self.root(b)
        if a == b: return True
        aa, bb = self.underlying(a), self.underlying(b)
        return aa == bb and (self.nodes[a]["kind"] not in {"nominal", "builtin"}
                             or self.nodes[b]["kind"] not in {"nominal", "builtin"})

    def callee(self, decl):
        need(type(decl) is str and decl in self.callees, "exact captured static callee")
        sig = self.callees[decl]
        args = ([] if sig["receiver"] is None else [sig["receiver"]]) + sig["params"]
        return args, sig["results"], {"function_decl_id": decl, "signature_id": sig["id"]}

    def adapter(self, name):
        # Exact private Go1.25.0 source binding; a source/value proof is still due.
        files = []
        for path in ("src/strings/strings.go", "src/internal/stringslite/strings.go"):
            raw = (DEFAULT_GOROOT / path).read_bytes()
            need(("func " + name + "(").encode() in raw, "exact pinned stdlib source declaration")
            files.append({"path": path, "raw_sha256": sha(raw), "bytes": len(raw)})
        return {"symbol": "strings." + name, "compiler_release": "go1.25.0",
                "compiler_inventory_sha256": sha((HERE / "host-toolchain.json").read_bytes()), "sources": files}

    def derive(self, op, p):
        need(type(op) is str and op in self.operations, "closed56 operation tags")
        n = int(op[-2:]); exact(p, PARAMETERS[n])
        p = copy.deepcopy(p); ins = []; outs = []; regions = []; bindings = []
        for key in ("operator", "size", "evaluation"):
            if key in p: need(type(p[key]) is str, "scalar operation subtag")
        B, I, S, E = (self.id(k) for k in ("bool", "int", "string", "error"))
        V = self.value
        def types(key, limit=16):
            need(type(p[key]) is list and len(p[key]) <= limit, "ordered source value arity")
            return [V(t) for t in p[key]]
        def region(role, inputs, outputs): regions.append({"role": role, "inputs": inputs, "outputs": outputs})
        control = {"kind": "control", "result_types": self.fn_results}
        if n in {1, 23, 35}: outs = [V(E if n == 23 else p["type"])]
        if n == 35: self.root(p["type"], {"slice"})
        if n == 2:
            r = self.root(p["type"], {"bool", "word", "string"}); val = p["value"]
            if r["kind"] == "bool": need(type(val) is bool, "exact Bool constant")
            elif r["kind"] == "word":
                need(type(val) is str and __import__("re").fullmatch(r"0|-?[1-9][0-9]*", val) is not None and len(val) <= 21, "canonical bounded source integer constant")
                value = int(val); width = r["width"]; low = -(1 << (width-1)) if r["signed"] else 0; high = (1 << (width-(1 if r["signed"] else 0)))-1
                need(low <= value <= high, "Go source representability, no constant wrapping")
            else: self.bytes_value(val)
            outs = [V(p["type"])]
        if n == 3:
            a, b = self.root(p["from"]), self.root(p["to"])
            allowed = a["kind"] == b["kind"] == "word" or a["kind"] == b["kind"] and a["kind"] in {"bool", "string"}
            allowed |= a["kind"] == b["kind"] and a["kind"] in {"array", "struct", "slice"} and self.identical(self.underlying(p["from"]), self.underlying(p["to"]))
            allowed |= p["from"] == p["to"] == E
            need(allowed, "closed explicit Go conversion relation")
            ins, outs = [V(p["from"])], [V(p["to"])]
        if n in {4, 5, 6, 7, 22}:
            self.integer(p["type"])
            allowed = UNARY if n == 4 else ARITH if n == 5 else {"divide", "remainder"} if n == 6 else {"left", "right"} if n == 7 else COMPOUND
            need(type(p["operator"]) is str and p["operator"] in allowed, "exact integer operator subtag")
            ins, outs = [V(p["type"])], [V(p["type"])]
            if n in {5, 6}: ins.append(V(p["type"]))
            if n == 7: self.integer(p["count_type"]); ins.append(V(p["count_type"]))
            if n == 22:
                self.integer(p["rhs_type"])
                need(p["operator"] in {"left", "right"} or self.assignable(p["rhs_type"], p["type"]), "compound RHS preserves nominal assignment")
                ins, outs = [location(p["type"]), V(p["rhs_type"]), STATE], [STATE]
        if n == 8:
            self.root(p["type"], {"bool"}); need(p["operator"] in {"not", "and", "or"}, "closed Boolean operator")
            ins, outs = [V(p["type"])], [V(p["type"])]
            if p["operator"] != "not": region("guarded_rhs", [STATE], [V(p["type"]), STATE]); ins.append(STATE); outs.append(STATE)
        if n in {9, 14, 28}:
            r = self.root(p["type"], {"bool", "word"} if n == 9 else {"array", "struct"} if n == 14 else {"string"})
            allowed = {"equal", "not_equal"} if n == 14 or r["kind"] == "bool" else COMPARE
            need(p["operator"] in allowed, "exact source comparison subtag")
            if n == 14: need(self.comparable(p["type"]), "complete aggregate comparability excludes error/slice")
            ins, outs = [V(p["type"]), V(p["type"])], [V(B)]
        if n in {10, 44}:
            r = self.root(p["type"], {"array", "struct"} if n == 10 else {"slice"})
            indexes = p["initializers"]; need(type(indexes) is list and len(indexes) <= 256, "closed initializer indices")
            need(all(type(x) is int for x in indexes) and len(set(indexes)) == len(indexes), "no duplicate or noninteger initializer")
            limit = len(r["fields"]) if r["kind"] == "struct" else r["length"] if r["kind"] == "array" else 128
            for index in indexes:
                uint(index, limit-1); ins.append(V(r["fields"][index]["type"] if r["kind"] == "struct" else r["element"]))
            outs = [V(p["type"])]
            if n == 44: ins.append(STATE); outs.append(STATE)
        if n == 11:
            r = self.root(p["type"], {"struct"}); field = uint(p["field"], len(r["fields"])-1)
            node = self.nodes[self.underlying(p["type"])]; need(node["fields"][field]["name"] != "_", "blank fields are not observable")
            ins, outs = [V(p["type"])], [V(r["fields"][field]["type"])]
        if n == 12: ins, outs = [location(p["type"]), V(p["type"]), STATE], [STATE]
        if n == 13:
            r = self.root(p["type"], {"array"}); self.integer(p["index_type"])
            ins, outs = [V(p["type"]), V(p["index_type"])], [V(r["element"])]
        if n == 15:
            self.root(p["condition_type"], {"bool"}); ins, outs = [V(p["condition_type"]), STATE], [control]
            for role in ("then", "else"): region(role, [STATE], [control])
        if n == 16:
            self.root(p["type"], {"bool", "word", "string", "array", "struct"})
            need(self.comparable(p["type"]), "switch equality-admitted aggregate tag; nil-only error/slice tags lower through closed nil tests")
            arities = p["case_arities"]
            need(type(arities) is list and len(arities) <= SYNTAX_MAX and all(type(x) is int and 1 <= x <= SYNTAX_MAX for x in arities) and sum(arities) <= SYNTAX_MAX, "ordered case-expression groups within existing complete-source syntax budget")
            count = len(arities)
            need(p["default_index"] is None or type(p["default_index"]) is int and 0 <= p["default_index"] <= count, "required nullable default position")
            ins, outs = [V(p["type"]), STATE], [control]
            for i, arity in enumerate(arities):
                for j in range(arity): region("case_test:"+str(i)+":"+str(j), [STATE], [V(p["type"]), STATE])
                region("case_body:"+str(i), [STATE], [control])
            if p["default_index"] is not None: region("default", [STATE], [control])
        if n == 17: ins, outs = types("types"), [joint(p["types"])]
        if n in {18, 19}:
            args, results, binding = self.callee(p["callee"]); bindings = [binding]
            if n == 18:
                index = uint(p["index"], len(results)-1); ins, outs = [joint(results), STATE], [V(results[index])]
            else: ins, outs = [V(t) for t in args]+[STATE], [joint(results), STATE]
        if n == 20: ins, outs = [joint(self.fn_results), STATE], [control]
        if n == 21:
            values = types("types", SYNTAX_MAX); targets = p["targets"]; need(type(targets) is list and len(targets) == len(values), "parallel assignment arity")
            for target, value in zip(targets, p["types"]):
                if target is not None: self.root(target); need(self.assignable(value, target), "Go assignment source identity"); ins.append(location(target))
            ins += [{"kind":"values", "types":p["types"]}, STATE]; outs = [STATE]
        if n in {24, 25, 26}:
            if n == 24:
                need(p["type"] in self.specializer.universe["cases"], "exact same global error universe"); ins, outs = [V(p["type"])], [V(E)]
            elif n == 25: ins, outs = [V(E)], [V(B)]
            else:
                ins, outs = [V(E), STATE], [joint([S]), STATE]
                signatures = {e["type_id"]: e["signature_id"] for e in self.defs["error_implementers"]}
                bindings = [{"case_type_id": t, "signature_id": signatures[t], "function_decl_id": self.nodes[signatures[t]]["declaration_id"]} for t in self.specializer.universe["cases"]]
        if n in {27, 29, 30, 31, 32, 33, 34}:
            t = S if n in {33, 34} else p["type"]; self.root(t, {"string"})
            ins, outs = [V(t)], [V(t)]
            if n == 27: self.bytes_value(p["bytes_hex"]); ins = []
            if n == 29: outs = [V(I)]
            if n == 30: self.integer(p["index_type"]); ins.append(V(p["index_type"])); outs = [V(self.id("uint8"))]
            if n == 31:
                for key in ("lo_type", "hi_type"): self.integer(p[key]); ins.append(V(p[key]))
            if n in {32, 33, 34}: ins.append(V(t))
            if n in {33, 34}: outs = [V(B)]; bindings = [self.adapter("HasPrefix" if n == 33 else "HasSuffix")]
        if n in {36, 37, 38, 39, 40, 41, 43, 45, 47, 48, 50}:
            r = self.root(p["type"], {"slice"}); t = p["type"]; ins, outs = [V(t)], [V(t)]
            if n == 37: need(p["size"] in {"length", "capacity"}, "closed size query"); outs = [V(I)]
            if n == 38: outs = [V(B)]
            if n in {39, 45}:
                self.integer(p["index_type"]); ins.append(V(p["index_type"]))
                if n == 39: outs = [V(r["element"])]
                else: ins.append(V(r["element"])); outs = []
            if n in {40, 41}:
                for key in ("lo_type", "hi_type") + (("max_type",) if n == 41 else ()):
                    self.integer(p[key]); ins.append(V(p[key]))
            if n == 43:
                ins = []
                for key in ("len_type", "cap_type"): self.integer(p[key]); ins.append(V(p[key]))
            if n in {47, 50}:
                need(r["element"] == self.id("uint8"), "destination assignable to []byte, aliases retain uint8 identity")
                ins.append(V(S)); outs = [V(I)] if n == 47 else [V(t)]
            if n == 48: ins += [V(r["element"])] * uint(p["count"], SYNTAX_MAX)
            if n in {39, 43, 45, 47, 48, 50}: ins.append(STATE)
            if n in {43, 45, 47, 48, 50}: outs.append(STATE)
        if n == 42:
            a = self.root(p["type"], {"array"}); s = self.root(p["slice_type"], {"slice"})
            need(a["element"] == s["element"], "array view exact element identity"); need(self.specializer.shapes.inspect(a["element"])[1], "Plain array view element")
            ins, outs = [location(p["type"])], [V(p["slice_type"]), STATE]
            for key in ("lo_type", "hi_type"): self.integer(p[key]); ins.append(V(p[key]))
            if p["max_type"] is not None: self.integer(p["max_type"]); ins.append(V(p["max_type"]))
            ins.append(STATE)
        if n in {46, 49}:
            d, s = (self.root(p[k], {"slice"}) for k in ("destination", "source"))
            need(d["element"] == s["element"], "copy/spread exact same source element type")
            ins = [V(p["destination"]), V(p["source"]), STATE]; outs = [V(I if n == 46 else p["destination"]), STATE]
        if n in {51, 52, 53}:
            need(type(p["loop"]) is int and p["loop"] in self.scope.loop_ids, "exact captured loop selector")
            loop = next(l for l in self.scope.function["loops"] if l["ordinal"] == p["loop"])
            need(loop["kind"] == ("for" if n == 51 else "range"), "source loop kind")
            bindings = [{"loop_ref_id": self.scope.loop_ids[p["loop"]]}]; ins, outs = [STATE], [control]
            if n == 51:
                self.root(p["condition_type"], {"bool"}); need(type(p["has_test"]) is bool, "required test-presence Bool")
                region("init", [STATE], [STATE])
                if p["has_test"]: region("test", [STATE], [V(p["condition_type"]), STATE])
                else: need(p["condition_type"] == B, "absent for test is canonical true")
                region("body", [STATE], [control]); region("post", [STATE], [STATE])
            else:
                r = self.root(p["type"], {"array"} if n == 52 else {"slice"})
                need(type(p["index"]) is bool and type(p["value"]) is bool, "effective nonblank range variables")
                evaluate = True
                if n == 52:
                    need(p["evaluation"] in {"evaluate", "constant_length"}, "closed array range evaluation mode")
                    evaluate = p["evaluation"] == "evaluate"
                    need(evaluate or not p["value"], "second nonblank range value requires array evaluation")
                ins = ([V(p["type"])] if evaluate else []) + [STATE]; args = [STATE]
                if p["index"]: args.append(V(I))
                if p["value"]: args.append(V(r["element"]))
                region("body", args, [control])
        if n in {54, 55}:
            need(type(p["enclosing"]) is list and p["enclosing"], "outer-to-inner lexical control stack")
            seen = set(); candidates = []; previous = None
            for target in p["enclosing"]:
                exact(target, {"kind", "source_object_id", "start_byte", "end_byte", "ordinal"})
                need(type(target["kind"]) is str and target["kind"] in {"loop", "switch"} and target["source_object_id"] == self.scope.source_id, "closed same-source control target")
                start, end, ordinal = (uint(target[k]) for k in ("start_byte", "end_byte", "ordinal"))
                need(start < end <= len(self.scope.source), "physical control span")
                need(self.scope.function["start_byte"] <= start < end <= self.scope.function["end_byte"], "same-function control scope")
                if previous is not None: need(previous[0] <= start and end <= previous[1] and (start, end) != previous, "outer-to-inner containment")
                previous = start, end
                if target["kind"] == "loop":
                    loop = next((l for l in self.scope.function["loops"] if l["ordinal"] == ordinal), None)
                    need(loop is not None and (start, end) == (loop["start_byte"], loop["end_byte"]), "exact source loop target")
                token = canonical(target); need(token not in seen, "no duplicate enclosing targets"); seen.add(token)
                if n == 54 or target["kind"] == "loop": candidates.append(target)
            need(candidates, "continue needs an enclosing loop")
            bindings = [{"target": candidates[-1]}]; ins, outs = [STATE], [control]
        if n == 56:
            self.root(p["type"], {"array"}); need(p["size"] in {"length", "capacity"}, "array size kind")
            ins, outs = [V(p["type"]), STATE], [V(I), STATE]
        row = self.operations[op]
        obligations = [{"category": "ordinary", "ordinal": i, "predicate": name}
                       for i, name in enumerate(COMMON)]
        members = [row]
        if n == 22:
            component = 6 if p["operator"] in {"divide", "remainder"} else 7 if p["operator"] in {"left", "right"} else 5
            members += [self.operations["GP-OP-"+str(component).zfill(2)], self.operations["GP-OP-12"]]
        for category, key in (("runtime_safety", "runtime_safety"), ("profile_domain", "profile_domain"), ("permission", "permission_obligations")):
            predicates = [text for member in members for text in member[key]]
            obligations += [{"category": category, "ordinal": i, "predicate": text} for i, text in enumerate(predicates)]
        return sealed(SCHEMA, DOMAIN, {"semantic_context": self.scope.context, "capture_context_id": self.scope.capture,
            "source_scope_sha256": self.scope.value["content_sha256"], "function_decl_id": self.scope.declaration,
            "signature_id": self.scope.signature, "type_definitions_sha256": self.defs["content_sha256"],
            "error_universe_id": self.defs["error_universe_id"], "operation_table_sha256": sha(TABLE.read_bytes()),
            "op": op, "parameters": p, "inputs": ins, "outputs": outs, "regions": regions,
            "obligations": obligations, "binding_requirements": bindings}, "operation_hash")

    @staticmethod
    def bytes_value(value):
        need(type(value) is str and len(value) <= 512 and len(value) % 2 == 0
             and all(c in "0123456789abcdef" for c in value), "complete lowercase byte string within256-byte literal domain")

    def validate(self, record):
        return self.validate_all([record])

    def validate_all(self, records):
        need(all(self.classify(records)), "complete operation signature batch reconstruction")
        return True

    def classify(self, records):
        # Caller-mutable prototype maps are never a consumer authority.
        fresh = Signatures(Scope(json.loads(self.raw_bytes), self.universe, self.overlay_path, self.raw_bytes))
        outcomes = []
        for record in records:
            try:
                exact(record, ROOT_FIELDS)
                expected = fresh.derive(record["op"], record["parameters"])
                need(canonical(record) == canonical(expected), "complete operation/signature/context/source/member reconstruction")
                outcomes.append(True)
            except ValueError: outcomes.append(False)
        return outcomes

    def comparable(self, t):
        r = self.root(t)
        if r["kind"] in {"bool", "word", "string"}: return True
        if r["kind"] == "array": return self.comparable(r["element"])
        if r["kind"] == "struct": return all(self.comparable(f["type"]) for f in r["fields"])
        return False
