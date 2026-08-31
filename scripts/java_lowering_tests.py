"""T06 fixtures and independent artifact/CFG/scalar evaluation checks. No release activation."""

import copy
import hashlib
import json
import re


REJECTIONS = tuple("lowering." + name for name in (
    "div_missing", "div_extra", "overflow", "shift_unmasked", "shift_wrong_mask", "shift_unlinked", "unsigned_escape"))
PRECEDENCE = ("contract_before_lowering", "map_failure_prevents_partial_output")
LIMITS = ("instructions_per_method", "instructions_per_closure", "cfg_blocks_per_method", "cfg_blocks_per_closure",
          "frontend_stdout", "frontend_stderr", "vir_canonical_bytes", "source_map_canonical_bytes", "source_manifest_canonical_bytes")
SOURCE = "src/vector/Case.java"


def contract(method):
    return dict(schema="mpk.java.contract.v0", semantic_profile="mpk.java.scalar.v0", method=method, requires=[],
                ensures=[{"bool": True}], modifies=[], abrupt_completion="forbidden", termination="total")


def fixtures(destination, build):
    vector = build.load_json(build.ROOT / build.VECTOR)
    build.require(tuple(row["id"] for row in vector["rejected_cases"] if row["id"].startswith("lowering.")) == REJECTIONS)
    build.require(len(vector["accepted_cases"]) == 49 and len(vector["operation_mappings"]) == 27 and len(vector["cfg_patterns"]) == 6)
    build.require(len(vector["source_map_cases"]) == 7 and set(LIMITS) <= {row["id"] for row in vector["limit_cases"]})
    records = []

    def add(case_id, sources, methods, contracts, group, **metadata):
        selection = dict(schema="mpk.selection.java_methods.v0", value=dict(compilation="lowering-vector",
                         sources=sorted(sources), contracts=[f"contracts/c{n:03}.json" for n in range(len(contracts))], methods=sorted(methods)))
        files = {path: text.encode("utf-8") for path, text in sources.items()}
        files.update({path: build.canonical(value) + b"\n" for path, value in zip(selection["value"]["contracts"], contracts)})
        folder = destination / "lowering" / str(len(records))
        folder.mkdir(parents=True)
        (folder / "selection.json").write_bytes(build.canonical(selection))
        for name, data in files.items():
            path = folder / "snapshot" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        records.append(dict(id=case_id, group=group, folder=folder.name, selection=selection, files=files, **metadata))

    def source_case(case_id, body, params="int x", result="int", extra="", group="extra", **metadata):
        text = f"package vector;\npublic interface Case {{\n public static {result} f({params}) {{ {body} }}\n {extra}\n}}\n"
        methods = []
        for return_type, name, parameters in re.findall(r"public static (boolean|int|long) ([A-Za-z_]+)\(([^)]*)\)", text):
            types = [parameter.strip().split()[0] for parameter in parameters.split(",") if parameter.strip()]
            methods.append(f"vector.Case::{name}({','.join(types)})->{return_type}")
        add(case_id, {SOURCE: text}, [methods[0]], [contract(method) for method in methods], group, **metadata)

    for row in vector["accepted_cases"]:
        add("accepted/" + row["id"], row["sources"], row["methods"], row["contracts"], "accepted", vector=row)
    for index, row in enumerate(vector["operation_mappings"]):
        op = row["source"]
        scalar = "long" if "bv64" in row["operand_rule"] else "int"
        params, result, extra = f"{scalar} x, {scalar} y", scalar, ""
        if op in ("!", "&&", "||"):
            params, result = "boolean x, boolean y", "boolean"
        if op in ("==", "!=", "<", "<=", ">", ">="): result = "boolean"
        if op in ("<<", ">>", ">>>"): params = f"{scalar} x, int y"
        expression = f"x {op} y"
        if op in ("!", "~", "unary -"): expression = ("-" if op == "unary -" else op) + "x"
        if op == "?:": params, expression = "boolean c, int x, int y", "c ? x : y"
        if op == "direct_static_call":
            expression, extra = "g(x)", "public static int g(int z) { return z; }"
        source_case(f"operation/{index:02}", "return " + expression + ";", params, result, extra, "operation", vector=row)
    for row in vector["cfg_patterns"]:
        kind = row["id"]
        params = {"identity": "int x", "widen_return": "int x", "ternary": "boolean c, int a, int b",
                  "early_return": "boolean c, int a, int b", "short_circuit_and": "boolean a, boolean b",
                  "nested_call_arguments": "int x"}[kind]
        result = "long" if kind == "widen_return" else "boolean" if kind == "short_circuit_and" else "int"
        extra = ("public static int left(int x) { return x + 1; } public static int right(int x) { return x - 1; } "
                 "public static int add(int a, int b) { return a + b; }") if kind == "nested_call_arguments" else ""
        source_case("cfg/" + kind, row["source_body"], params, result, extra, "cfg", vector=row)

    source_case("extra/live-prefix", "return (x + 1) + (c ? x / 2 : x % 3);", "boolean c, int x",
                evaluations=[dict(arguments=[True, 8], result=13), dict(arguments=[False, 8], result=11)])
    source_case("extra/live-joins", "return (a ? x : y) + (b ? y : x);", "boolean a, boolean b, int x, int y",
                evaluations=[dict(arguments=[True, True, 3, 7], result=10), dict(arguments=[False, True, 3, 7], result=14),
                             dict(arguments=[True, False, 3, 7], result=6), dict(arguments=[False, False, 3, 7], result=10)])
    source_case("extra/live-call-arguments", "return add(left(x), c ? right(x) : left(x), d ? left(x) : right(x));",
                "boolean c, boolean d, int x", extra="public static int left(int x) { return x + 1; } "
                "public static int right(int x) { return x - 1; } public static int add(int a, int b, int c) { return a + b + c; }",
                evaluations=[dict(arguments=[True, False, 5], result=14, calls=["left", "right", "right", "add"]),
                             dict(arguments=[False, True, 5], result=18, calls=["left", "left", "left", "add"])])
    source_case("extra/live-shift", "return (x + 1) >>> (c ? y / 2 : y % 3);", "boolean c, int x, int y",
                evaluations=[dict(arguments=[True, -2, 4], result=1073741823), dict(arguments=[False, -2, 4], result=2147483647)])
    source_case("extra/live-boolean", "return (x / 2 == 3) == (c && x / 4 > 0);", "boolean c, int x", "boolean",
                evaluations=[dict(arguments=[True, 6], result=True), dict(arguments=[False, 6], result=False)])
    source_case("extra/local-order", "if (c) { int a = x + 1; return a; } else { int b = x - 1; return b; }", "boolean c, int x",
                evaluations=[dict(arguments=[True, 8], result=9), dict(arguments=[False, 8], result=7)])
    source_case("extra/nested-if", "int y = x; if (c) { if (d) { return y; } y = 2; } else { y = 3; } return y;",
                "boolean c, boolean d, int x", evaluations=[dict(arguments=[True, True, 7], result=7),
                dict(arguments=[True, False, 7], result=2), dict(arguments=[False, True, 7], result=3)])
    source_case("extra/map-unicode", "// あ😀\n\tint y = x; y = (y + 1) >>> k; return y;", "int x, int k",
                evaluations=[dict(arguments=[-2, 1], result=2147483647)])
    source_case("extra/negative-zero", "int y = - 0; return y + -(1);", "", evaluations=[dict(arguments=[], result=-1)])
    source_case("extra/eager-checks", "return (x / a) / (x % b);", "int x, int a, int b",
                evaluations=[dict(arguments=[8, 0, 0], trap=True, check_ops=["bv_sdiv"]),
                             dict(arguments=[8, 2, 0], trap=True, check_ops=["bv_sdiv", "bv_srem"]),
                             dict(arguments=[8, 2, 3], result=2, check_ops=["bv_sdiv", "bv_srem", "bv_sdiv"])])
    source_case("extra/short-circuit-checks", "return c || (x / a > 0 && x % b > 0);", "boolean c, int x, int a, int b", "boolean",
                evaluations=[dict(arguments=[True, 8, 0, 0], result=True, check_ops=[]),
                             dict(arguments=[False, -8, 2, 0], result=False, check_ops=["bv_sdiv"]),
                             dict(arguments=[False, 8, 2, 3], result=True, check_ops=["bv_sdiv", "bv_srem"])])
    source_case("extra/constant-no-fold", "return 1 + 2 * 3;", "", evaluations=[dict(arguments=[], result=7)])
    source_case("extra/identity-casts", "boolean b = (boolean)c; int a = (int)x; long z = (long)y; return b ? z : (long)a;",
                "boolean c, int x, long y", "long", evaluations=[dict(arguments=[True, 3, 8], result=8), dict(arguments=[False, -3, 8], result=-3)])
    source_case("link/changed-source", "return x + 1;", group="link")
    baseline = next(row for row in vector["accepted_cases"] if row["id"] == "int.identity")
    add("link/changed-sidecar-bytes", baseline["sources"], baseline["methods"], baseline["contracts"], "link")
    changed = records[-1]
    path = changed["selection"]["value"]["contracts"][0]
    changed["files"][path] += b" \n"
    (destination / "lowering" / changed["folder"] / "snapshot" / path).write_bytes(changed["files"][path])
    source_case("limit/block-method", "int y = x; " + "if (c) { y = x; } else { y = x; } " * 342 + "return y;",
                "boolean c, int x", group="limit", code="JAVA_LIMIT_CFG_BLOCKS_PER_METHOD", phase="lowering")
    bad = contract("vector.Case::f(int,int)->int"); bad["ensures"] = []
    add("precedence/contract_before_lowering", {SOURCE: "package vector; public interface Case { public static int f(int x, int y) { return x >>> y; } }\n"},
        [bad["method"]], [bad], "precedence", code="JAVA_CONTRACT_SHAPE", phase="subset")

    (destination / "lowering-cases.tsv").write_text("".join(
        f"{row['id']}\t{row['folder']}\t{row['group']}\t{row.get('code', 'ir-lowered')}\t{row.get('phase', 'emission')}\n" for row in records))
    maps = destination / "maps"; maps.mkdir()
    for row in vector["source_map_cases"]: (maps / (row["id"] + ".txt")).write_text(row["source"], encoding="utf-8")
    (destination / "source-maps.tsv").write_text("".join(
        f"{row['id']}\t{row['utf16_start']}\t{row['utf16_end']}\t{row['expected_code'] or 'accept'}\n" for row in vector["source_map_cases"]))
    return records


def typed_hash(build, domain, value):
    return hashlib.sha256(domain.encode("ascii") + b"\0" + build.canonical(value)).hexdigest()


def symbolic(blocks):
    def kind(type_):
        return "boolean" if type_["kind"] == "bool" else "int" if type_["width"] == 32 else "long"
    result = []
    for block in copy.deepcopy(blocks):
        for binding in block["parameters"]: binding["type"] = kind(binding["type"])
        for instruction in block["instructions"]:
            instruction["type"] = kind(instruction["type"])
            instruction.pop("contract_hash", None)
            if instruction["kind"] == "Const":
                value = instruction["value"]
                instruction["value"] = value.get("bool", value.get("int", {}).get("value"))
            else:
                for key in ("value", "lhs", "rhs"):
                    if key in instruction: instruction[key] = instruction[key]["var"]
                if "args" in instruction: instruction["args"] = [arg["var"] for arg in instruction["args"]]
        end = block["terminator"]
        if "cond" in end:
            end["condition"] = end.pop("cond")["var"]
            if not end["else_args"]: end.pop("else_args")
            if not end["then_args"]: end.pop("then_args")
        for key in ("values", "args"):
            if key in end: end[key] = [arg["var"] for arg in end[key]]
        result.append(block)
    return result


def evaluate(functions, selected, arguments):
    """Independent mathematical Bool/BV interpreter; no javac execution or producer helper."""
    functions = {function["id"]: function for function in functions}
    calls, checks = [], []

    class Trap(Exception):
        pass

    def wrap(value, type_):
        if type_["kind"] == "bool":
            assert isinstance(value, bool)
            return value
        width = type_["width"]
        bits = int(value) % (1 << width)
        return bits - (1 << width) if type_["signed"] and bits >= 1 << (width - 1) else bits

    def invoke(id_, args):
        function = functions[id_]
        globals_ = {binding["id"]: wrap(value, binding["type"]) for binding, value in zip(function["params"], args)}
        blocks = {block["label"]: block for block in function["blocks"]}
        block, block_args = blocks["bb0"], []
        seen = set()
        while True:
            assert block["label"] not in seen
            seen.add(block["label"])
            values = dict(globals_)
            values.update({binding["id"]: value for binding, value in zip(block["parameters"], block_args)})
            def atom(value):
                if "var" in value: return values[value["var"]]
                if "bool" in value: return value["bool"]
                return int(value["int"]["value"])
            for instruction in block["instructions"]:
                kind, type_ = instruction["kind"], instruction["type"]
                if kind in ("Const", "Copy", "Convert"): value = atom(instruction["value"])
                elif kind == "CallStatic":
                    calls.append(instruction["function"].split("::")[1].split("(")[0])
                    value = invoke(instruction["function"], [atom(arg) for arg in instruction["args"]])
                elif kind == "UnaryOp":
                    operand = atom(instruction["value"])
                    value = {"not": lambda: not operand, "bv_neg": lambda: -operand, "bv_not": lambda: ~operand}[instruction["op"]]()
                else:
                    left, right = atom(instruction["lhs"]), atom(instruction["rhs"])
                    op = instruction["op"]
                    if op in ("bv_sdiv", "bv_srem"):
                        checks.append(op)
                        if right == 0: raise Trap()
                        quotient = (abs(left) // abs(right)) * (-1 if (left < 0) != (right < 0) else 1)
                        value = quotient if op == "bv_sdiv" else left - quotient * right
                    else:
                        value = {"bv_add": lambda: left + right, "bv_sub": lambda: left - right, "bv_mul": lambda: left * right,
                                 "bv_and": lambda: left & right, "bv_or": lambda: left | right, "bv_xor": lambda: left ^ right,
                                 "bv_shl": lambda: left << right, "bv_ashr": lambda: left >> right, "bv_lshr": lambda: left >> right,
                                 "eq": lambda: left == right, "not_eq": lambda: left != right, "signed_lt": lambda: left < right,
                                 "signed_le": lambda: left <= right, "signed_gt": lambda: left > right, "signed_ge": lambda: left >= right}[op]()
                value = wrap(value, type_)
                values[instruction["id"]] = value
                if kind == "Copy": globals_[instruction["target"]] = values[instruction["target"]] = value
            end = block["terminator"]
            if end["kind"] == "Return": return atom(end["values"][0])
            if end["kind"] == "Jump": label, args = end["label"], end["args"]
            else:
                prefix = "then" if atom(end["cond"]) else "else"
                label, args = end[prefix + "_label"], end[prefix + "_args"]
            block_args = [atom(arg) for arg in args]
            block = blocks[label]
    try:
        result = invoke(selected, arguments)
        return dict(result=result, calls=calls, check_ops=checks)
    except Trap:
        return dict(trap=True, calls=calls, check_ops=checks)


def validate_report(report, fixtures_, build):
    vector = build.load_json(build.ROOT / build.VECTOR)
    build.require(report["schema"] == "mpk.java.lowering_tests.v0" and report["assertions"] >= 100)
    build.require([row["id"] for row in report["cases"]] == [row["id"] for row in fixtures_])
    build.require(report["counter_boundaries"] == list(LIMITS))
    evaluations = 0
    for actual, expected in zip(report["cases"], fixtures_):
        envelope = json.loads(actual["envelope"])
        build.require(actual["envelope"].encode("utf-8") == build.canonical(envelope) + b"\n", "JAVA_LOWERING_TEST_CANONICAL")
        build.require(envelope["semantic_context"] == vector["semantic_context_fixture"] and envelope["selection"] == expected["selection"])
        # Exact bytes, including sidecar formatting, go to the independent Rust importer.
        actual["captured_inputs"] = [dict(path=path, kind="source" if path in expected["selection"]["value"]["sources"] else "contract",
                                          text=data.decode("utf-8")) for path, data in sorted(expected["files"].items())]
        if "code" in expected:
            build.require(actual["code"] == expected["code"] and actual["phase"] == expected["phase"])
            build.require(set(envelope) == {"schema", "status", "phase", "semantic_context", "selection", "diagnostics", "rejected_features"})
            continue
        build.require(actual["exit"] == 0 and envelope["status"] == "ir-lowered")
        build.require(actual["repeat_sha256"] == build.sha256(actual["envelope"].encode("utf-8")), "JAVA_LOWERING_TEST_DETERMINISM")
        vir, source_map, manifest = envelope["ir"]["value"], envelope["source_map"], envelope["source_manifest"]
        for artifact, field, domain in ((vir, "vir_hash", "MPK-VIR-1.0"), (source_map, "source_map_hash", "MPK-SOURCE-MAP-1.0"),
                                        (manifest, "source_manifest_hash", "MPK-SOURCE-MANIFEST-1.0")):
            payload = {key: value for key, value in artifact.items() if key != field}
            build.require(artifact[field] == typed_hash(build, domain, payload), "JAVA_LOWERING_TEST_HASH")
        build.require(envelope["ir"]["sha256"] == vir["vir_hash"] == source_map["source_ir_hash"] == manifest["vir_hash"])
        build.require(source_map["source_map_hash"] == manifest["source_map_hash"] and "vc_hash" not in manifest)
        build.require(manifest["selection"] == expected["selection"]
                      and manifest["frontend"]["binary_sha256"] == report["candidate_inventory"]["frontend_files"][0]["sha256"])
        build.require(manifest["toolchain"]["distribution_sha256"] == vector["toolchain_inputs"]["archive"]["sha256"])
        candidate = build.load_json(build.ROOT / "release/build-inputs/java/bundle-candidate.json")
        components = [{key: value for key, value in row.items() if key in ("kind", "name", "release", "content_sha256", "binary_sha256")}
                      for row in candidate["toolchain_bundles"][0]["components"]]
        build.require(manifest["toolchain"]["components"] == components and manifest["toolchain"]["bundle_id"] == "test.java.toolchain")
        build.require(manifest["frontend"]["name"] == "java2vir" and manifest["frontend"]["version"] == "0.1.0"
                      and manifest["frontend"]["bundle_id"] == "test.java.frontend" and manifest["frontend"]["subordinate_binaries"] == [])
        build.require(manifest["release_registry"] == dict(schema="mpk.release.bundle_registry.v1", id="mpk.release.registry.v1", registry_sha256="0" * 64))
        wanted_inputs = [dict(normalized_path=path, kind="source" if path in expected["selection"]["value"]["sources"] else "contract",
                             size_bytes=len(data), sha256=build.sha256(data)) for path, data in sorted(expected["files"].items())]
        build.require(manifest["inputs"] == wanted_inputs and manifest["input_set_hash"] == typed_hash(build, "MPK-INPUT-SET-0.1", wanted_inputs))
        functions = vir["units"][0]["functions"]
        selected = expected["selection"]["value"]["methods"][0]
        function = next(function for function in functions if function["id"] == selected)
        operations = []
        required = []
        for block in function["blocks"]:
            for instruction in block["instructions"]:
                operations.append(instruction.get("op", instruction["kind"]))
                checks = [check["kind"] for check in instruction["safety_checks"]]
                build.require(checks == (["divisor_nonzero"] if instruction.get("op") in ("bv_sdiv", "bv_srem") else []))
                required.extend(checks)
            operations.append(block["terminator"]["kind"])
            if block["parameters"]: operations.append("block_parameter")
        row = expected.get("vector", {})
        if expected["group"] == "accepted":
            remaining = iter(operations)
            build.require(all(any(op == wanted for op in remaining) for wanted in row["expected_profile_operations"]),
                          "JAVA_LOWERING_TEST_OPERATION_PROJECTION")
            build.require(required == row["expected_required_checks"])
        elif expected["group"] == "operation":
            projected = [op.split("(")[0] for op in row["lowering"]]
            remaining = iter(operations)
            build.require(all(any(op == wanted for op in remaining) for wanted in projected))
            build.require(required == row["required_checks"])
        elif expected["group"] == "cfg":
            build.require(symbolic(function["blocks"]) == row["blocks"], "JAVA_LOWERING_TEST_CFG_GOLDEN")
        for evaluation in row.get("evaluation_cases", []) + expected.get("evaluations", []):
            method = evaluation.get("method", selected)
            value = evaluate(functions, method, evaluation["arguments"])
            for key, wanted in evaluation.items():
                if key in ("arguments", "method"): continue
                if key == "result" and not isinstance(wanted, bool): wanted = int(wanted)
                build.require(value.get(key) == wanted, "JAVA_LOWERING_TEST_EVALUATION")
            evaluations += 1
        validate_origins(source_map, functions, expected, build)
    expected_failures = {row["id"]: row for row in vector["rejected_cases"] if row["id"] in REJECTIONS}
    failures = {row["id"]: row for row in report["mutations"]}
    for case_id, row in expected_failures.items():
        actual = failures[case_id]
        build.require(all(actual[key] == row["expected_" + key] for key in ("status", "phase", "code")))
    for actual, row in zip(report["maps"], vector["source_map_cases"]):
        build.require(actual["id"] == row["id"])
        if row["expected_status"] == "accept": build.require(actual["range"] == row["expected_utf8_range"])
        else: build.require(actual["code"] == row["expected_code"])
    build.require(len(report["maps"]) == 7)
    for failure in report["mutations"]:
        envelope = json.loads(failure["envelope"])
        build.require(set(envelope) == {"schema", "status", "phase", "semantic_context", "selection", "diagnostics", "rejected_features"})
        build.require(failure["published_bytes"] == 0)
    report["evaluation_count"] = evaluations


def validate_origins(source_map, functions, fixture, build):
    wanted = []
    for function in sorted(functions, key=lambda function: function["id"]):
        wanted.append(dict(kind="function", unit_id=function["unit_id"], function_id=function["id"]))
        for block in function["blocks"]:
            for instruction in block["instructions"]:
                wanted.append(dict(kind="instruction", unit_id=function["unit_id"], function_id=function["id"], block=block["label"], instruction=instruction["id"]))
        for block in function["blocks"]:
            wanted.append(dict(kind="terminator", unit_id=function["unit_id"], function_id=function["id"], block=block["label"]))
    build.require([entry["reference"] for entry in source_map["entries"]] == wanted, "JAVA_SOURCE_MAP_TEST_COVERAGE")
    for entry in source_map["entries"]:
        origin = entry["origin"]
        build.require(set(origin) == {"kind", "input_kind", "normalized_path", "start", "end"} and origin["kind"] == origin["input_kind"] == "source")
        data = fixture["files"][origin["normalized_path"]]
        first, last = origin["start"], origin["end"]
        build.require(0 <= first < last <= len(data))
        data[:first].decode("utf-8"); spelling = data[first:last].decode("utf-8")
        if entry["reference"]["kind"] == "function": build.require(spelling.startswith("public static ") and spelling.endswith("}"))
