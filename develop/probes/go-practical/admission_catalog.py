"""Generate a closed draft catalog against the pinned compiler's AST source."""
import hashlib
import json
import re
from pathlib import Path
from run_proof_pilot import DEFAULT_GOROOT

ROOT = Path(__file__).resolve().parents[3]
RECORD = ROOT / "develop/migrations/go-practical/t01-w04/source-admission"
REJECT = set("BadDecl BadExpr BadStmt ChanType CommClause DeferStmt FuncLit GoStmt IndexListExpr InterfaceType LabeledStmt MapType SelectStmt SendStmt StarExpr TypeAssertExpr TypeSwitchStmt".split())
STRUCTURE = set("Comment CommentGroup Field FieldList File ImportSpec Package".split())
CONDITIONS = {
    "ArrayType": "fixed array or slice with complete Root/Plain/type/length validation; inferred literal extent only in its Go context",
    "AssignStmt": "Go assignability, short declaration/shadowing; each location/operand/call once, two-phase stores and compound operation checks",
    "BasicLit": "compiler-resolved exact constants; runtime materialization only in admitted carriers",
    "BinaryExpr": "closed Bool/word/string operations and nil-only slice/error comparisons; actual guards and full-width operands",
    "BlockStmt": "original lexical scope and exact activation/lifetime; structured sequential execution",
    "BranchStmt": "unlabeled break/continue with exact structured target; goto/fallthrough reject",
    "CallExpr": "direct captured acyclic source call/method, exact checked registered adapter, whitelisted builtin or GP-OP-03 conversion; one joint invocation",
    "CaseClause": "expression switch case/default only, source-order selection",
    "CompositeLit": "acyclic admitted array/slice/struct layout; keyed or positional Go initialization, omitted-field zeroes and checked freshness",
    "DeclStmt": "local var/const/type declarations of admitted types; compiler scopes and no hidden global effects",
    "Ellipsis": "inferred array literal extent; no variadic function declarations/arbitrary spread calls; append spread is CallExpr metadata",
    "EmptyStmt": "no operation; preserve control/source identity",
    "ExprStmt": "admitted call with all result/effect/dependency members, including arity zero",
    "ForStmt": "exact init/test/body/binding/post/abrupt paths; LC01–LC12 and partial/total mode",
    "FuncDecl": "captured nongeneric nonvariadic source body; value receiver, exact Root signature, results arity 0..16",
    "FuncType": "declared direct function/method signature only; function-valued fields/parameters/results/locals reject",
    "GenDecl": "referenced types/constants and local declarations; package var initialization never becomes a theorem assumption",
    "Ident": "compiler object/alias/nominal/constant/lexical identity; mutable package state and function values reject",
    "IfStmt": "Go init/scope/short circuit; all branches in closure unless accepted unreachability evidence exists",
    "IncDecStmt": "word target evaluated once, wrapping operation, permissions/source frame checked",
    "IndexExpr": "array/slice/string full-index checks; generic instantiation and map indexing reject",
    "KeyValueExpr": "Go composite field/index key, exact assignability and constant extent",
    "ParenExpr": "preserve operand context/type/evaluation",
    "RangeStmt": "array/slice only, captured expression and effective-version bindings; pointer-array/string/integer/map/channel/function range reject",
    "ReturnStmt": "explicit/naked/forwarding return, same-invocation ordered results/post-state, enclosing posts skipped",
    "SelectorExpr": "direct nonpromoted field/value-method or exact captured package object; no pointer/interface dispatch or ambient state",
    "SliceExpr": "array/slice two/three-index view or string two-index substring; exact bounds/capacity/alias/source checks",
    "StructType": "acyclic direct-field Root aggregate and Go tags; embedded fields and nonplain backing cells reject",
    "SwitchStmt": "expression/tagless switch, one tag evaluation, ordered cases/default, correct break target",
    "TypeSpec": "captured nongeneric nominal or alias identity; complete expanded Root/Plain/error graph",
    "UnaryExpr": "Go + - ! ^ only, preserved type/width/evaluation; address-taking and receive reject",
    "ValueSpec": "exact constants and admitted local value storage; referenced mutable package state rejects",
}
BUILTINS = "append cap clear close complex copy delete imag len make max min new panic print println real recover".split()
BUILTIN_CONDITIONS = {
    "append": "Go-compatible values/spread/string-byte special form; fresh positive writes and proved no-reallocation/domain bound",
    "cap": "array/slice only, constant-evaluation exception and complete descriptor validity",
    "copy": "identical slice elements or byte/string special form; snapshot overlap and fresh positive writes",
    "len": "array/slice/byte-string only with constant-evaluation exception",
    "make": "slice only; executed integer checks, bounds, initialized fresh storage and allocation/frame proofs",
}


def generate():
    source = (DEFAULT_GOROOT / "src/go/ast/ast.go").read_bytes()
    names = sorted(set(re.findall(r"func \(\w+ \*(\w+)\) Pos\(", source.decode())))
    if set(names) != REJECT | STRUCTURE | set(CONDITIONS): raise ValueError("complete pinned AST disposition")
    rows = []
    for name in names:
        disposition = "reject_in_verification" if name in REJECT else "capture_only_structure" if name in STRUCTURE else "conditional_candidate"
        condition = "outside v1 selected verification closure" if name in REJECT else "captured original structure; does not authorize body execution" if name in STRUCTURE else CONDITIONS[name]
        component = "closure" if name in REJECT | STRUCTURE | {"CallExpr", "FuncDecl", "FuncType", "Ident", "GenDecl"} else "types" if name in {"ArrayType", "StructType", "TypeSpec", "ValueSpec", "BasicLit", "Ellipsis"} else "lower"
        owner = "practical_" + component
        rows.append({"id": "GA-AST-"+name, "ast_kind": name, "disposition": disposition, "condition": condition,
                     "primary_production_executor": "go-tools/go2vir/"+owner+".go", "primary_test_owner": "go-tools/go2vir/"+owner+"_test.go", "independent_validator": "crates/mpk-vc/src/go_practical_artifacts.rs"})
    builtins = [{"name": n, "disposition": "conditional_candidate" if n in BUILTIN_CONDITIONS else "reject_in_verification", "condition": BUILTIN_CONDITIONS.get(n, "outside v1 builtin whitelist"),
                 "primary_production_executor": "go-tools/go2vir/practical_closure.go", "primary_test_owner": "go-tools/go2vir/practical_closure_test.go"} for n in BUILTINS]
    return {"schema": "mpk.go_practical.source_admission_catalog.v0", "status": "private closed draft; not a published freeze or production admission", "semantic_profile": "mpk.go.practical.v1",
            "effective_languages": ["go1.22", "go1.23", "go1.24", "go1.25"], "target": "linux/amd64", "target_integer_bits": 64, "pinned_compiler": "go1.25.0",
            "pinned_ast_source": {"toolchain_relative_path": "src/go/ast/ast.go", "sha256": hashlib.sha256(source).hexdigest(), "extraction": "all concrete pointer receiver Pos methods in pinned go/ast/ast.go"},
            "ast_nodes": rows, "builtins": builtins,
            "pending": ["complete source/type/constant/effect predicates and source/VC/ordinary proof composition", "all numeric/structural/capture/resource limits and exact successor schemas/hashes/migrations"]}


if __name__ == "__main__":
    RECORD.mkdir(exist_ok=True)
    (RECORD / "catalog.json").write_text(json.dumps(generate(), indent=2, sort_keys=True, ensure_ascii=False)+"\n")
