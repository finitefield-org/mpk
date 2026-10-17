"""Private source/closure candidate classifier; not an independent importer."""
from pathlib import Path
import hashlib
from check_scope_report import strict_json, require

ROOT = Path(__file__).resolve().parents[3]
RECORD = ROOT / "develop/migrations/go-practical/t01-w04/source-admission"
PENDING = [
    "complete captured multi-package source/type/constant identity and independent import",
    "complete global error universe, Root/Plain recursion and entry binding",
    "executed evaluation/commutation/lvalue effect, permission, safety and domain proofs",
    "all root/operation/call/result/frame/loop/termination source-model proof members",
    "numeric/structural/capture/resource and wire/hash/migration freeze",
]


def catalog():
    return strict_json((RECORD / "catalog.json").read_bytes())


def classify(source, native, root):
    c = catalog()
    require(native["schema"] == "mpk.go_practical.admission_native.v0", "native schema")
    require(native["source_sha256"] == hashlib.sha256(source.encode()).hexdigest(), "original source identity")
    require(native["language"] in c["effective_languages"] and native["target_sizes"] == "gc/amd64" and native["int_bits"] == 64, "pinned language and target")
    require(native["predeclared_builtins"] == [r["name"] for r in c["builtins"]], "closed builtin universe")
    if native["kind"] in {"source_error", "type_error"}:
        require(bool(native["errors"]), "original package failure evidence")
        return {"status": native["kind"], "diagnostics": native["errors"], "closure": [], "typecheck_only": [], "pending": []}
    require(native["kind"] == "typed_original_package" and not native["errors"], "typed original package")
    functions = {r["id"]: r for r in native["functions"]}
    require(len(functions) == len(native["functions"]), "unique physical declaration identities")
    selected = [f for f in functions.values() if f["name"] == root]
    require(len(selected) == 1 and root != "init", "one explicit non-init selected root")
    nodes = {r["ast_kind"]: r for r in c["ast_nodes"]}
    builtins = {r["name"]: r for r in c["builtins"]}
    visited = {}; diagnostics = []

    def problem(code, detail, path, start, end):
        diagnostics.append({"code": code, "detail": detail, "path": path, "start_byte": start, "end_byte": end})

    def visit(identity, path, active):
        require(identity in functions, "callee has captured physical declaration")
        f = functions[identity]; path = path + [f["name"]]
        if identity in active:
            problem("CALL_CYCLE", "acyclic initial call graph", path, f["start_byte"], f["end_byte"])
            return
        if identity in visited: return
        visited[identity] = path
        for row in f["nodes"]:
            require(row["kind"] in nodes, "unknown pinned AST node")
            require(type(row["start_byte"]) is int and f["start_byte"] <= row["start_byte"] <= row["end_byte"] <= f["end_byte"], "physical UTF-8 node span")
            rule = nodes[row["kind"]]
            if rule["disposition"] == "reject_in_verification":
                problem("SYNTAX_"+row["kind"], rule["condition"], path, row["start_byte"], row["end_byte"])
            if row["kind"] == "BranchStmt" and (row.get("op") not in {"break", "continue"} or row.get("label")):
                problem("BRANCH_KIND", "unlabeled break/continue only", path, row["start_byte"], row["end_byte"])
            if row["kind"] == "UnaryExpr" and row.get("op") not in {"+", "-", "!", "^"}:
                problem("UNARY_KIND", "no address-taking or channel receive", path, row["start_byte"], row["end_byte"])
        for row in f["issues"]:
            problem(row["code"], row["detail"], path, row["start_byte"], row["end_byte"])
        for name in f["builtins"]:
            require(name in builtins, "known predeclared builtin")
            if builtins[name]["disposition"] == "reject_in_verification":
                problem("BUILTIN_"+name, builtins[name]["condition"], path, f["start_byte"], f["end_byte"])
        for callee in dict.fromkeys(f["calls"]): visit(callee, path, active | {identity})

    visit(selected[0]["id"], [], set())
    return {"status": "rejected_candidate" if diagnostics else "candidate_unproved", "diagnostics": diagnostics,
            "closure": [{"id": i, "name": functions[i]["name"], "path": p} for i, p in visited.items()],
            "typecheck_only": [f["name"] for i, f in functions.items() if i not in visited],
            "pending": PENDING.copy()}
