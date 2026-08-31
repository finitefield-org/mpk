"""Private T05 fixtures and independent report checks for source/contract admission."""

import copy
import hashlib
import json
import re


SUBSET_PREFIXES = ("local.", "control.", "literal.", "call.", "dispatch.", "conversion.", "heap.",
                   "types.", "operations.", "async.", "declaration.", "identifier.", "method.")
SUBSET_IDS = tuple("""local.uninitialized local.parameter_assignment local.repeated_names local.multiple_declarators
local.final local.var local.assignment_expression local.increment local.compound_assignment control.loop
control.empty_statement control.switch control.throw control.try control.assert control.synchronized literal.hex
literal.octal literal.binary literal.separator literal.unary_plus literal.char literal.minimum_parentheses
literal.positive_overflow call.recursion call.library call.checked_overflow_library call.floor_library
dispatch.instance conversion.boxing conversion.boolean_truthiness heap.array heap.null types.float types.decimal
types.unbounded operations.unbounded operations.unbounded_shift conversion.range_checked async.future types.byte
types.short types.char operations.mixed_binary operations.long_shift_count conversion.mixed_ternary declaration.class
declaration.field declaration.inheritance declaration.missing_public declaration.overload declaration.unrelated_method
declaration.annotation declaration.import declaration.generic declaration.throws identifier.dollar identifier.unicode
identifier.contextual identifier.ignored_control method.parameter_slots""".split())
CONTRACT_IDS = tuple("""contract.missing contract.unused contract.duplicate contract.duplicate_key contract.requires_result
contract.unknown_parameter contract.local contract.empty_ensures contract.division contract.shift contract.conversion
contract.unsigned contract.negative_zero contract.profile""".split())
PRECEDENCE = ("subset_before_contract", "excluded_class_before_accepted_tree_compare", "excluded_var_before_accepted_tree_compare")
LIMITS = ("method_closure", "parameter_slots", "contract_clauses", "contract_nodes_per_method", "contract_nodes_per_closure", "contract_depth")
SOURCE = "src/vector/Case.java"
METHOD = "vector.Case::f(int)->int"


def fixture_ownership(vector, build):
    build.require(tuple(row["id"] for row in vector["rejected_cases"] if row["id"].startswith(SUBSET_PREFIXES)) == SUBSET_IDS,
                  "JAVA_SUBSET_TEST_OWNERS")
    build.require(tuple(row["id"] for row in vector["rejected_cases"] if row["id"].startswith("contract.")) == CONTRACT_IDS,
                  "JAVA_CONTRACT_TEST_OWNERS")
    build.require(len(vector["semantic_rows"]) == 34 and len(vector["conversion_rules"]) == 35)
    build.require(set(LIMITS) <= {row["id"] for row in vector["limit_cases"]})
    coverage = {row for case in vector["accepted_cases"] for row in case["rows"]}
    coverage.update(row for case in vector["rejected_cases"] if case["id"] in SUBSET_IDS + CONTRACT_IDS for row in case["rows"])
    build.require(coverage == {row["row"] for row in vector["semantic_rows"]}, "JAVA_SUBSET_TEST_MATRIX_OWNERS")


def contract(method=METHOD, ensures=None):
    return dict(schema="mpk.java.contract.v0", semantic_profile="mpk.java.scalar.v0", method=method, requires=[],
                ensures=ensures if ensures is not None else [{"bool": True}], modifies=[], abrupt_completion="forbidden", termination="total")


def source(body="return x;", params="int x", result="int", extra=""):
    return f"package vector;\npublic interface Case {{ public static {result} f({params}) {{ {body} }} {extra} }}\n"


def integer(value, kind="i32"):
    return {"int": {"decimal": value, "type": kind}}


def op(name, *args):
    return {"op": name, "args": list(args)}


def node_tree(count):
    if count == 1:
        return {"bool": True}
    if count == 2:
        return op("not", {"bool": False})
    children = min(64, count - 1)
    width, extra = divmod(count - 1, children)
    return op("and", *(node_tree(width + (index < extra)) for index in range(children)))


def declarations(sources):
    """Independent oracle for the deliberately simple generated positive fixtures."""
    result = {}
    for text in sources.values():
        text = re.sub(r"/\*[\s\S]*?\*/|//[^\n]*", " ", text)
        package = re.search(r"\bpackage\s+([\w.]+)\s*;", text).group(1)
        owner = package + "." + re.search(r"\binterface\s+(\w+)", text).group(1)
        for match in re.finditer(r"\b(?:public\s+static|static\s+public)\s+(boolean|int|long)\s+(\w+)\s*\(([^()]*)\)", text):
            returns, name, arguments = match.groups()
            params = [parameter.strip().split() for parameter in arguments.split(",")] if arguments.strip() else []
            method = owner + "::" + name + "(" + ",".join(parameter[0] for parameter in params) + ")->" + returns
            first = text.index("{", match.end()) + 1
            last, depth = first, 1
            while depth:
                depth += (text[last] == "{") - (text[last] == "}")
                last += 1
            calls = re.findall(r"\b([A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*)\s*\(", text[first:last - 1])
            result[method] = dict(owner=owner, name=name, result=returns, parameters=params,
                                  calls=[call for call in calls if call not in ("if", "return", "int", "long", "boolean")])
    for item in result.values():
        names = [call if "." in call else item["owner"] + "." + call for call in item.pop("calls")]
        item["callees"] = sorted({key for key, value in result.items() if value["owner"] + "." + value["name"] in names})
    return result


def fixtures(destination, build):
    vector = build.load_json(build.ROOT / build.VECTOR)
    fixture_ownership(vector, build)
    records = []
    baseline = next(row for row in vector["accepted_cases"] if row["id"] == "int.identity")
    original = baseline["sources"][SOURCE]

    def add(case_id, sources=None, methods=None, sidecars=None, code=None, phase="subset", group="extra", compilation="vector"):
        if any(row["id"] == case_id for row in records):
            raise AssertionError(case_id)
        sources = {SOURCE: original} if sources is None else dict(sources)
        methods = [METHOD] if methods is None else methods
        sidecars = {"contracts/f.json": contract()} if sidecars is None else copy.deepcopy(sidecars)
        selection = dict(schema="mpk.selection.java_methods.v0", value=dict(compilation=compilation,
                         sources=sorted(sources), contracts=sorted(sidecars), methods=sorted(methods)))
        folder = destination / "admission" / str(len(records))
        folder.mkdir(parents=True)
        (folder / "selection.json").write_bytes(build.canonical(selection))
        files = {path: text.encode("utf-8") for path, text in sources.items()}
        files.update({path: value if isinstance(value, bytes) else build.canonical(value) for path, value in sidecars.items()})
        for name, data in files.items():
            path = folder / "snapshot" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        record = dict(id=case_id, folder=folder.name, group=group, code=code, phase=phase, selection=selection,
                      sources=sources, sidecars=sidecars, files=files)
        if code is None:
            record["declarations"] = declarations(sources)
        records.append(record)

    for row in vector["accepted_cases"]:
        add("accepted/" + row["id"], row["sources"], row["methods"],
            {f"contracts/c{index:03}.json": value for index, value in enumerate(row["contracts"])}, group="accepted")
    add("contract-fixture", {"src/demo/Policy.java": vector["case_harness"]["baseline_files"]["src/demo/Policy.java"]},
        vector["selection_fixture"]["value"]["methods"], {"contracts/approved.json": vector["contract_fixture"]},
        compilation="payment-policy", group="contract")

    declaration_mutations = {
        "declaration.class": original.replace("public interface", "public class"),
        "declaration.field": source(extra="int state = 1;"),
        "declaration.inheritance": original.replace("interface Case {", "interface Case extends java.io.Serializable {"),
        "declaration.missing_public": original.replace("public static", "static"),
        "declaration.overload": source(extra="public static long f(long x) { return x; }"),
        "declaration.unrelated_method": source(extra="public static int g(int y) { return y; }"),
        "declaration.annotation": original.replace("public static", "@Deprecated public static"),
        "declaration.import": original.replace("package vector;", "package vector; import java.lang.Math;"),
        "declaration.generic": original.replace("static int", "static <T> int"),
        "declaration.throws": original.replace("f(int x)", "f(int x) throws Exception"),
        "identifier.dollar": source("int $x = x; return $x;"),
        "identifier.unicode": source("int 値 = x; return 値;"),
        "identifier.contextual": source("int record = x; return record;"),
        "identifier.ignored_control": source("int lo\x01cal = x; return lo\x01cal;"),
        "method.parameter_slots": source("return 0;", ",".join(f"long p{i}" for i in range(128))),
    }
    for row in vector["rejected_cases"]:
        if row["id"] not in SUBSET_IDS:
            continue
        text = row.get("source", declaration_mutations.get(row["id"]))
        build.require(isinstance(text, str), "JAVA_SUBSET_TEST_MUTATION")
        sidecars = {"contracts/f.json": contract()}
        if row["id"] == "declaration.unrelated_method":
            sidecars["contracts/g.json"] = contract("vector.Case::g(int)->int")
        sources = {SOURCE: text}
        if row["id"] == "declaration.inheritance":
            sources[SOURCE] = original.replace("interface Case {", "interface Case extends vector.Parent {")
            sources["src/vector/Parent.java"] = "package vector; public interface Parent {}\n"
        add(row["id"], sources, sidecars=sidecars, code=row["expected_code"], phase=row["expected_phase"], group="subset")

    for row in vector["rejected_cases"]:
        if row["id"] not in CONTRACT_IDS:
            continue
        case_id = row["id"]
        sidecar = contract()
        sidecars = {"contracts/f.json": sidecar}
        sources = {SOURCE: original}
        if case_id == "contract.missing":
            # Preserve the nonempty selection/capture invariant while omitting f's
            # sidecar: the reachable g still has its selected sidecar.
            sources[SOURCE] = source("return g(x);", extra="public static int g(int x) { return x; }")
            sidecars = {"contracts/g.json": contract("vector.Case::g(int)->int")}
        elif case_id == "contract.unused": sidecars["contracts/g.json"] = contract("vector.Case::g(int)->int")
        elif case_id == "contract.duplicate": sidecars["contracts/copy.json"] = contract()
        elif case_id == "contract.duplicate_key":
            sidecars["contracts/f.json"] = build.canonical(sidecar)[:-1] + b',"ensures":[{"bool":true}]}'
        elif case_id == "contract.requires_result": sidecar["requires"] = [op("eq", {"result": 0}, integer("0"))]
        elif case_id == "contract.unknown_parameter": sidecar["ensures"] = [op("eq", {"parameter": "absent"}, integer("0"))]
        elif case_id == "contract.local":
            sources[SOURCE] = source("int local0 = x; return local0;")
            sidecar["ensures"] = [op("eq", {"parameter": "local0"}, integer("0"))]
        elif case_id == "contract.empty_ensures": sidecar["ensures"] = []
        elif case_id in ("contract.division", "contract.shift", "contract.conversion"):
            name = {"contract.division": "bv_sdiv", "contract.shift": "bv_shl", "contract.conversion": "Convert"}[case_id]
            sidecar["ensures"] = [op(name, integer("1"), integer("1"))]
        elif case_id == "contract.unsigned": sidecar["ensures"] = [op("eq", integer("0", "u32"), integer("0", "u32"))]
        elif case_id == "contract.negative_zero": sidecar["ensures"] = [op("eq", integer("-0"), integer("0"))]
        elif case_id == "contract.profile": sidecar["semantic_profile"] = "mpk.csharp.scalar.v0"
        else: raise AssertionError(case_id)
        add(case_id, sources, sidecars=sidecars, code=row["expected_code"], group="contract")

    # Source restrictions not exhausted by the frozen negative list.
    extras = {
        "interface-not-public": (original.replace("public interface", "interface"), "DECLARATION"),
        "empty-interface": ("package vector; public interface Case {}\n", "DECLARATION"),
        "record-parent": ("package vector; public record Case(int y) { public static int f(int x) { return x; } }\n", "DECLARATION"),
        "enum-parent": ("package vector; public enum Case { ONLY; public static int f(int x) { return x; } }\n", "DECLARATION"),
        "annotation-parent": ("package vector; public @interface Case { int f(); }\n", "DECLARATION"),
        "nested-interface": (source(extra="interface Nested {}"), "DECLARATION"),
        "empty-member": (source(extra=";"), "DECLARATION"),
        "empty-member-first": (original.replace("interface Case {", "interface Case { ;"), "DECLARATION"),
        "local-array-after-name": (source("int y[] = new int[1]; return x;"), "TYPE"),
        "unused-uninitialized": (source("int y; return x;"), "CONTROL_FLOW"),
        "call-statement": (source("g(x); return x;", extra="public static int g(int x) { return x; }"), "CONTROL_FLOW"),
        "partial-target": (source("return Case.g(x);", extra="public static int g(int x) { return x; }"), "CALL"),
        "dead-recursion": (source("return false ? f(x) : x;"), "CALL"),
        "mutual-recursion": (source("return g(x);", extra="public static int g(int x) { return f(x); }"), "CALL"),
        "dead-loop": (source("if (false) { while (x > 0) { x = x - 1; } } return x;"), "CONTROL_FLOW"),
        "dead-var": (source("if (false) { var y = x; } return x;"), "TYPE"),
        "dead-hex": (source("return false ? 0x10 : x;"), "LITERAL"),
        "leading-zero": (source("return 00;"), "LITERAL"),
        "lowercase-long": (source("return 1l;", result="long"), "LITERAL"),
        "boolean-bitwise": (source("return true & false;", result="boolean"), "OPERATION"),
        "boolean-bitor": (source("return true | false;", result="boolean"), "OPERATION"),
        "boolean-bitxor": (source("return true ^ false;", result="boolean"), "OPERATION"),
        "source-path-mismatch": (original.replace("package vector;", "package elsewhere;"), "DECLARATION"),
        "top-name-control": (original.replace("interface Case", "interface Ca\x01se"), "IDENTIFIER"),
        "parameter-control": (original.replace("int x", "int x\x01"), "IDENTIFIER"),
        "read-control": (original.replace("return x", "return x\x01"), "IDENTIFIER"),
        "local-shadow-disjoint": (source("{ int y = x; } { int y = x; } return x;"), "CONTROL_FLOW"),
        "default-package": (original.replace("package vector;", ""), "DECLARATION"),
        "static-import": (original.replace("package vector;", "package vector; import static java.lang.Math.abs;").replace("return x;", "return abs(x);"), "DECLARATION"),
        "default-method": (original.replace("public static", "public default"), "DECLARATION"),
        "private-method": (original.replace("public static", "private static"), "DECLARATION"),
        "abstract-method": ("package vector; public interface Case { public int f(int x); }\n", "DECLARATION"),
        "varargs": (source("return 0;", "int... x"), "DECLARATION"),
        "array-parameter": (source("return 0;", "int[] x"), "TYPE"),
        "final-parameter": (source(params="final int x"), "DECLARATION"),
        "parameter-annotation": (source(params="@Deprecated int x"), "DECLARATION"),
        "method-contextual": (original.replace(" f(", " record("), "IDENTIFIER"),
        "local-contextual": (source("int when = x; return when;"), "IDENTIFIER"),
        "parameter-contextual": (source("return var;", "int var"), "IDENTIFIER"),
        "unnamed-local": (source("int _ = x; return x;"), "IDENTIFIER"),
    }
    for name, (text, code) in extras.items():
        add("source-extra/" + name, {SOURCE: text}, code="JAVA_SUBSET_" + code)
    add("source-extra/static-public-comments", {SOURCE: source("int /*é😀*/ y = x; y = (int)(long)y; return - /*comment*/ 0 + y;").replace("public static", "static /*modifier*/ public")})
    add("source-extra/comment-delimiters", {SOURCE: source("int y = (x); // ignored var bad = 0;\n return y;")})
    bindings_method = "vector.Case::f(int,int)->int"
    add("source-extra/symbol-bindings", {SOURCE: source("int local = second; local = first; return local - second;", "int second, int first")},
        [bindings_method], {"contracts/f.json": contract(bindings_method)})
    add("source-extra/unused-file", {SOURCE: original, "src/vector/Unused.java": "package vector; public interface Unused { public static int g() { return 0; } }\n"},
        sidecars={"contracts/f.json": contract(), "contracts/g.json": contract("vector.Unused::g()->int")}, code="JAVA_SUBSET_CALL")
    add("source-extra/namespace-lookalike", {"src/com/sunny/Case.java": original.replace("package vector;", "package com.sunny;")},
        ["com.sunny.Case::f(int)->int"], {"contracts/f.json": contract("com.sunny.Case::f(int)->int")})

    # Every frozen conversion rule is exercised against both the finite rule and
    # an actual attributed source context, including compiler-owned refusals.
    for index, row in enumerate(vector["conversion_rules"]):
        before, after, context = row["source"], row["target"], row["context"]
        params, returns, extra = f"{before} x", after, ""
        body = {"explicit_cast": f"return ({after})x;", "local_initializer": f"{after} y = x; return y;",
                "local_assignment": f"{after} y = {'false' if after == 'boolean' else '0L' if after == 'long' else '0'}; y = x; return y;",
                "return": "return x;", "call_argument": "return g(x);"}.get(context)
        if context == "call_argument": extra = f"public static {after} g({after} y) {{ return y; }}"
        if context in ("binary_operand", "conditional_arm"):
            params += f", {after} y"
            returns = "boolean" if context == "binary_operand" else "long" if "long" in (before, after) else after
            body = "return x == y;" if context == "binary_operand" else "return true ? x : y;"
        text = source(body, params, returns, extra)
        declared = declarations({SOURCE: text})
        roots = [key for key in declared if "::f(" in key]
        sidecars = {f"contracts/c{i}.json": contract(key) for i, key in enumerate(declared)}
        code, phase = None, "subset"
        if not row["accepted"]:
            if before == "long" and after == "int" and context not in ("binary_operand", "conditional_arm"):
                code, phase = "JAVA_SOURCE_DIAGNOSTIC", "typecheck"
            else: code = "JAVA_SUBSET_TYPE" if context == "binary_operand" else "JAVA_SUBSET_CONVERSION"
        add(f"conversion-rule/{index}", {SOURCE: text}, roots, sidecars, code, phase, "conversion")

    def chain(count):
        methods = [f"vector.Case::m{i:03}()->int" for i in range(count)]
        text = "package vector; public interface Case {\n" + "\n".join(
            f"public static int m{i:03}() {{ return {'0' if i == count - 1 else f'm{i + 1:03}()'}; }}" for i in range(count)) + "\n}\n"
        return text, methods

    for count in (128, 129):
        text, methods = chain(count)
        sidecars = {f"contracts/m{i:03}.json": contract(method) for i, method in enumerate(methods[:128])}
        add(f"limit/method_closure/{count}", {SOURCE: text}, methods[:1], sidecars,
            None if count == 128 else "JAVA_LIMIT_METHOD_CLOSURE", group="limit")
    for slots in (255, 256):
        params = [f"long p{i}" for i in range(127)] + (["int last"] if slots == 255 else ["long last"])
        text = source("return 0;", ",".join(params))
        method = next(iter(declarations({SOURCE: text})))
        selected = method
        sidecars = {"contracts/f.json": contract(method)}
        if slots == 256:
            # The parent's selection validator already refuses a 256-slot root.
            # Reach that declaration from a valid root to exercise T05's own
            # descriptor-unit check without bypassing the request boundary.
            selected = "vector.Case::entry()->int"
            arguments = ",".join("0L" for _ in params)
            text = source("return 0;", ",".join(params), extra=f"public static int entry() {{ return f({arguments}); }}")
            sidecars["contracts/entry.json"] = contract(selected)
        add(f"limit/parameter_slots/{slots}", {SOURCE: text}, [selected], sidecars,
            None if slots == 255 else "JAVA_LIMIT_PARAMETER_SLOTS", group="limit")
    for name, maximum in (("contract_clauses", 64), ("contract_nodes_per_method", 1024), ("contract_depth", 32)):
        for count in (maximum, maximum + 1):
            clause = {"bool": True}
            if name == "contract_depth":
                for _ in range(count - 1): clause = op("not", clause)
            ensures = [{"bool": True}] * count if name == "contract_clauses" else [node_tree(count)] if name == "contract_nodes_per_method" else [clause]
            add(f"limit/{name}/{count}", sidecars={"contracts/f.json": contract(ensures=ensures)},
                code=None if count == maximum else "JAVA_LIMIT_" + name.upper(), group="limit")
    for total in (8192, 8193):
        counts = [1024] * 8 + ([1] if total == 8193 else [])
        text, methods = chain(len(counts))
        add(f"limit/contract_nodes_per_closure/{total}", {SOURCE: text}, methods[:1],
            {f"contracts/m{i:03}.json": contract(method, [node_tree(counts[i])]) for i, method in enumerate(methods)},
            None if total == 8192 else "JAVA_LIMIT_CONTRACT_NODES_PER_CLOSURE", group="limit")
    combined = contract()
    combined["requires"] = [{"bool": True}] * 64
    add("contract-extra/combined-clauses", sidecars={"contracts/f.json": combined}, code="JAVA_LIMIT_CONTRACT_CLAUSES")

    for row in vector["precedence_cases"]:
        if row["id"] not in PRECEDENCE: continue
        mutation = {"subset_before_contract": "declaration.field", "excluded_class_before_accepted_tree_compare": "declaration.class",
                    "excluded_var_before_accepted_tree_compare": "local.var"}[row["id"]]
        text = declaration_mutations.get(mutation) or next(case["source"] for case in vector["rejected_cases"] if case["id"] == mutation)
        sidecars = {"contracts/f.json": contract()}
        if row["id"] == "subset_before_contract":
            text = source("return g(x);", extra="int state = 1; public static int g(int x) { return x; }")
            sidecars = {"contracts/g.json": contract("vector.Case::g(int)->int")}
        add("precedence/" + row["id"], {SOURCE: text}, sidecars=sidecars, code=row["expected_code"], group="precedence")

    # Closed grammar, raw Unicode/number handling, and semantic counter consumers.
    base_bytes = build.canonical(contract())
    invalid_json = {
        "bom": b"\xef\xbb\xbf" + base_bytes, "utf8": b"\xff" + base_bytes,
        "trailing": base_bytes + b"false", "comment": b"/* comment */" + base_bytes,
        "trailing-comma": base_bytes[:-1] + b",}", "null": base_bytes.replace(b'"bool":true', b'"bool":null'),
        "float": base_bytes.replace(b'"bool":true', b'"result":0.0'),
        "exponent": base_bytes.replace(b'"bool":true', b'"result":0e0'),
        "unsafe-number": base_bytes.replace(b'"bool":true', b'"result":9007199254740992'),
        "leading-zero-number": base_bytes.replace(b'"bool":true', b'"result":00'),
        "escaped-surrogate": base_bytes.replace(b'"bool":true', b'"parameter":"\\ud800"'),
        "low-surrogate": base_bytes.replace(b'"bool":true', b'"parameter":"\\udc00"'),
        "nested-duplicate": base_bytes.replace(b'"bool":true', b'"bool":true,"bool":false'),
        "escaped-duplicate": base_bytes.replace(b'"bool":true', b'"bool":true,"bo\\u006fl":false'),
        "control-string": base_bytes.replace(b'"bool":true', b'"parameter":"x\x01"'),
        "json-before-operator": base_bytes.replace(b'"bool":true', b'"op":"forbidden","args":[{"bool":true,"bool":false}]'),
    }
    for name, data in invalid_json.items(): add("json-extra/" + name, sidecars={"contracts/f.json": data}, code="JAVA_CONTRACT_JSON")
    for name, expr, suffix in (
            ("result-negative-zero", {"result": -1}, "TYPE"), ("atom-union", {"bool": True, "result": 0}, "SHAPE"),
            ("unknown-field", {"field": "x"}, "SHAPE"), ("missing-args", {"op": "not"}, "SHAPE"),
            ("empty-nary", op("and"), "SHAPE"), ("single-nary", op("or", {"bool": True}), "SHAPE"),
            ("large-nary", op("and", *([{"bool": True}] * 65)), "SHAPE"),
            ("unary-arity", op("not", {"bool": True}, {"bool": False}), "SHAPE"),
            ("binary-arity", op("eq", integer("0")), "SHAPE"),
            ("not-integer", op("not", integer("0")), "TYPE"), ("neg-bool", op("bv_neg", {"bool": False}), "TYPE"),
            ("integer-clause", integer("0"), "TYPE"), ("mixed-equality", op("eq", integer("0"), integer("0", "i64")), "TYPE"),
            ("bool-order", op("signed_lt", {"bool": False}, {"bool": True}), "TYPE"),
            ("bool-add", op("bv_add", {"bool": False}, {"bool": True}), "TYPE"),
            ("nary-integer", op("and", {"bool": True}, integer("0")), "TYPE"),
            ("unsigned-op", op("unsigned_lt", integer("0"), integer("0")), "OPERATOR"),
            ("source-op", op("+", integer("0"), integer("0")), "OPERATOR")):
        add("contract-extra/" + name, sidecars={"contracts/f.json": contract(ensures=[expr])}, code="JAVA_CONTRACT_" + suffix)
    for token in ("-0", "+1", "01", " 1", "1 ", "0x1", "1_0", "1L", "2147483648", "-2147483649"):
        add("integer-extra/" + str(len(records)), sidecars={"contracts/f.json": contract(ensures=[op("eq", integer(token), integer("0"))])}, code="JAVA_CONTRACT_TYPE")
    for token in ("9223372036854775808", "-9223372036854775809"):
        add("integer-extra/" + str(len(records)), sidecars={"contracts/f.json": contract(ensures=[op("eq", integer(token, "i64"), integer("0", "i64"))])}, code="JAVA_CONTRACT_TYPE")
    for kind, tokens in (("i32", ("0", "2147483647", "-2147483648")), ("i64", ("0", "9223372036854775807", "-9223372036854775808"))):
        add("contract-extra/integer-boundaries-" + kind, sidecars={"contracts/f.json": contract(ensures=[op("eq", integer(token, kind), integer(token, kind)) for token in tokens])})
    for name in ("schema", "semantic_profile", "method", "requires", "ensures", "modifies", "abrupt_completion", "termination"):
        value = contract(); del value[name]
        add("shape-extra/missing-" + name, sidecars={"contracts/f.json": value}, code="JAVA_CONTRACT_SHAPE")
    for name, value in (("modifies", ["x"]), ("abrupt_completion", "allowed"), ("termination", "partial"), ("method", "vector.Case::f(int)->void")):
        sidecar = contract(); sidecar[name] = value
        add("identity-extra/" + name, sidecars={"contracts/f.json": sidecar}, code="JAVA_CONTRACT_IDENTITY")
    unknown = contract(); unknown["extra"] = True
    add("shape-extra/unknown", sidecars={"contracts/f.json": unknown}, code="JAVA_CONTRACT_SHAPE")
    nested_result = contract(); nested_result["requires"] = [op("not", op("eq", {"result": 0}, integer("0")))]
    add("contract-extra/nested-requires-result", sidecars={"contracts/f.json": nested_result}, code="JAVA_CONTRACT_TYPE")
    add("contract-extra/result-minus-zero", sidecars={"contracts/f.json": base_bytes.replace(b'"bool":true', b'"result":-0')}, code="JAVA_CONTRACT_TYPE")
    add("contract-extra/spelling-independent", sidecars={"contracts/f.json": json.dumps(contract(), indent=2).replace('"schema"', '"sche\\u006da"').encode()})
    all_ops = [op("not", {"bool": False}), op("and", {"bool": True}, {"bool": False}), op("or", {"bool": False}, {"bool": True})]
    for name in ("bv_neg", "bv_not"):
        all_ops.append(op("eq", op(name, {"parameter": "x"}), integer("0")))
    for name in ("eq", "not_eq", "signed_lt", "signed_le", "signed_gt", "signed_ge", "bv_add", "bv_sub", "bv_mul", "bv_and", "bv_or", "bv_xor"):
        expr = op(name, {"parameter": "x"}, integer("1"))
        all_ops.append(op("eq", expr, integer("0")) if name.startswith("bv_") else expr)
    add("contract-extra/all-operators", sidecars={"contracts/f.json": contract(ensures=all_ops)})
    ill_typed = contract(ensures=[op("eq", integer("0", "u32"), integer("0"))])
    add("batch/json-before-type", sidecars={"contracts/a.json": ill_typed, "contracts/b.json": b"{"}, code="JAVA_CONTRACT_JSON")
    unknown_shape = contract(); unknown_shape["extra"] = True
    add("batch/shape-before-type", sidecars={"contracts/a.json": ill_typed, "contracts/b.json": unknown_shape}, code="JAVA_CONTRACT_SHAPE")
    add("batch/duplicate-before-type", sidecars={"contracts/a.json": ill_typed, "contracts/b.json": contract()}, code="JAVA_CONTRACT_DUPLICATE")
    unused = copy.deepcopy(ill_typed); unused["method"] = "vector.Case::absent(int)->int"
    add("batch/unused-before-type", sidecars={"contracts/a.json": contract(), "contracts/b.json": unused}, code="JAVA_CONTRACT_UNUSED")
    missing = copy.deepcopy(ill_typed); missing["method"] = "vector.Case::g(int)->int"
    add("batch/missing-before-type", {SOURCE: source("return g(x);", extra="public static int g(int x) { return x; }")},
        sidecars={"contracts/g.json": missing}, code="JAVA_CONTRACT_MISSING")
    call_source = {SOURCE: source("return g(x);", extra="public static int g(int x) { return x; }")}
    bad_callee = contract("vector.Case::g(int)->int", [op("unknown", integer("0"))])
    add("batch/type-source-order", call_source, sidecars={"contracts/a.json": ill_typed, "contracts/b.json": bad_callee}, code="JAVA_CONTRACT_TYPE")
    add("batch/type-reversed-order", call_source, sidecars={"contracts/a.json": bad_callee, "contracts/b.json": ill_typed}, code="JAVA_CONTRACT_OPERATOR")
    for name in ("parameter", "result", "bool", "int", "operator", "op"):
        add("contract-extra/operator-tag-" + name, sidecars={"contracts/f.json": contract(ensures=[op(name, {"bool": True})])}, code="JAVA_CONTRACT_OPERATOR")
    add("link/source-a")
    add("link/source-b", {SOURCE: source("return x + 1;")})
    (destination / "admission-cases.tsv").write_text("".join(
        "\t".join((row["id"], row["folder"], row["group"], row["code"] or "admitted", row["phase"])) + "\n" for row in records))
    (destination / "conversion-rules.tsv").write_text("".join(
        "\t".join((row["source"], row["target"], row["context"], str(row["accepted"]).lower())) + "\n"
        for row in vector["conversion_rules"]))
    return dict(vector=vector, cases=records)


def normalize(expr, parameters):
    if "parameter" in expr: return {"var": "arg" + str(parameters.index(expr["parameter"]))}
    if "result" in expr or "bool" in expr: return expr
    if "int" in expr: return {"int": dict(value=expr["int"]["decimal"], signed=True, width=32 if expr["int"]["type"] == "i32" else 64)}
    args = [normalize(arg, parameters) for arg in expr["args"]]
    name = expr["op"]
    if name in ("not", "bv_neg", "bv_not"): return dict(op=name, value=args[0])
    if name in ("and", "or"): return dict(op=name, args=args)
    return dict(op=name, lhs=args[0], rhs=args[1])


def validate_report(report, metadata, build):
    vector, cases = metadata["vector"], metadata["cases"]
    build.require(report["schema"] == "mpk.java.admission_tests.v0")
    build.require(report["type_mappings"] == vector["type_mappings"])
    build.require(report["conversion_rules"] == [{key: row[key] for key in ("source", "target", "context", "accepted")} for row in vector["conversion_rules"]])
    build.require(len(report["cases"]) == len(cases) and len({row["id"] for row in report["cases"]}) == len(cases))
    definitions = {row["code"]: row for row in vector["diagnostic_registry"]}
    for expected, actual in zip(cases, report["cases"]):
        build.require(actual["id"] == expected["id"] and actual["group"] == expected["group"], "JAVA_ADMISSION_TEST_CASE")
        if expected["code"]:
            code = expected["code"]
            build.require(actual["code"] == code and actual["phase"] == expected["phase"], "JAVA_ADMISSION_TEST_REJECTION")
            build.require(actual["status"] == definitions[code]["status"] and actual["exit"] == definitions[code]["exit"])
            envelope = build.strict_json(actual["envelope"].encode(), canonical_transport=True)
            build.require(set(envelope) == {"schema", "status", "phase", "semantic_context", "selection", "diagnostics", "rejected_features"})
            build.require(envelope["semantic_context"] == vector["semantic_context_fixture"] and envelope["selection"] == expected["selection"])
            continue
        build.require(actual["status"] == "admitted", "JAVA_ADMISSION_TEST_ACCEPTANCE")
        declared = expected["declarations"]
        emitted = []
        while len(emitted) < len(declared):
            ready = sorted(key for key, value in declared.items() if key not in emitted and set(value["callees"]) <= set(emitted))
            build.require(bool(ready), "JAVA_ADMISSION_TEST_EXPECTED_CYCLE")
            emitted.append(ready[0])
        build.require([method["id"] for method in actual["methods"]] == emitted, "JAVA_ADMISSION_TEST_CALLEE_ORDER")
        for method in actual["methods"]:
            wanted = declared[method["id"]]
            build.require(method["callees"] == wanted["callees"] and method["result"] == wanted["result"])
            build.require(method["parameters"] == [dict(name=name, type=kind) for kind, name in wanted["parameters"]])
            parameters = {name: kind for kind, name in wanted["parameters"]}
            locals_by_name = {binding["name"]: binding["type"] for binding in method["locals"]}
            for binding in method["variable_bindings"]:
                raw = expected["files"][binding["path"]][binding["start"]:binding["end"]].decode("utf-8")
                build.require(raw == binding["name"], "JAVA_SUBSET_TEST_RAW_BINDING")
                scope = parameters if binding["parameter"] else locals_by_name
                build.require(scope.get(raw) == binding["type"], "JAVA_SUBSET_TEST_SYMBOL_BINDING")
            if expected["id"] == "source-extra/symbol-bindings":
                build.require([(binding["name"], binding["parameter"]) for binding in method["variable_bindings"]]
                              == [("second", True), ("local", False), ("first", True), ("local", False), ("second", True)])
        raw_sidecars = {}
        for path, value in expected["sidecars"].items():
            data = expected["files"][path]
            parsed = json.loads(data)
            raw_sidecars[parsed["method"]] = (path, data, parsed)
        build.require([row["normalized"]["function_id"] for row in actual["contracts"]] == emitted)
        for attached in actual["contracts"]:
            method = attached["normalized"]["function_id"]
            path, data, sidecar = raw_sidecars[method]
            build.require(attached["path"] == path and attached["raw_input_sha256"] == hashlib.sha256(data).hexdigest())
            build.require(attached["sidecar"] == sidecar)
            build.require(attached["sidecar_sha256"] == hashlib.sha256(b"MPK-JAVA-CONTRACT-SIDECAR-0.1\0" + build.canonical(sidecar)).hexdigest())
            names = [name for _, name in declared[method]["parameters"]]
            normalized = dict(semantic_context=vector["semantic_context_fixture"], unit_id=expected["selection"]["value"]["compilation"],
                              function_id=method, requires=[normalize(expr, names) for expr in sidecar["requires"]],
                              ensures=[normalize(expr, names) for expr in sidecar["ensures"]], modifies=[], panic="forbidden", termination="total", loops=[])
            normalized["contract_hash"] = hashlib.sha256(b"MPK-CONTRACT-1.0\0" + build.canonical(normalized)).hexdigest()
            build.require(attached["normalized"] == normalized, "JAVA_CONTRACT_TEST_NORMALIZATION")
            if expected["id"] == "contract-fixture":
                build.require(normalized == vector["normalized_contract_fixture"] and attached["sidecar_sha256"] == vector["contract_sidecar_sha256"])
        build.require(actual["selection_sha256"] == hashlib.sha256(b"MPK-JAVA-SELECTION-0.1\0" + build.canonical(expected["selection"])).hexdigest())
    build.require(report["link_failure"]["code"] == "JAVA_CONTRACT_HASH")
    build.require(report["counter_boundaries"] == list(LIMITS[2:]), "JAVA_CONTRACT_TEST_COUNTERS")
    report["owned_subset_cases"] = list(SUBSET_IDS)
    report["owned_contract_cases"] = list(CONTRACT_IDS)
    report["owned_semantic_rows"] = [row["row"] for row in vector["semantic_rows"]]
    report["owned_precedence_cases"] = list(PRECEDENCE)
