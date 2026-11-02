"""Pin the reproduced failure and audit the additive source-step test repair."""

from pathlib import Path
from collections import Counter
import difflib
import hashlib
import json
import re
import shutil
import subprocess
import sys


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
base = root.parent
compiled_source = "320ecd67ab420e1ea806196271eea3271f929fbe"
corrected = len(sys.argv) == 2 and sys.argv[1] == "corrected"
assert len(sys.argv) == 1 or corrected
suffix = "-corrected" if corrected else ""


def seal(path):
    data = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


def write_new(name, value):
    path = root / name
    encoded = json.dumps(value, indent=2) + "\n"
    if corrected and path.exists():
        assert path.read_text() == encoded, str(path)
        return
    assert not path.exists(), str(path)
    path.write_text(encoded)


def git_bytes(path):
    return subprocess.check_output(["/usr/bin/git", "show", f"{compiled_source}:{path}"])


diagnosis = base / "control-source-steps-diagnosis"
original_result = json.loads((diagnosis / "result.json").read_bytes())
assert original_result["status"] == "failed" and original_result["exit_code"] == 101
for stream in ("stdout", "stderr"):
    assert seal(original_result[stream]["path"]) == original_result[stream]
error = (diagnosis / "stderr.log").read_text()
assert "csharp_practical_ordinary_control_step_tests.rs:129:13" in error
assert "left: 3\n right: 2" in error
inputs = []
for item in original_result["inputs"]:
    current = seal(item["path"])
    assert all(current[key] == item[key] for key in ("path", "sha256", "bytes"))
    assert Path(item["path"]).read_bytes() == git_bytes(item["path"])
    inputs.append(dict(current, matches_source320_git_bytes=True))
assert len(inputs) == 42
production = []
for name in (
    "csharp_practical_ordinary_control_steps.rs",
    "csharp_practical_ordinary_control_edges.rs",
    "csharp_practical_ordinary_source_value_data.rs",
):
    path = Path("crates/mpk-vc/src") / name
    assert path.read_bytes() == git_bytes(path)
    production.append(dict(seal(path), matches_source320_git_bytes=True))
assert not subprocess.check_output([
    "/usr/bin/git", "diff", "--name-only", "HEAD", "--", "crates/mpk-vc/src",
])

source = Path("crates/mpk-vc/tests/support/csharp_practical_ordinary_control_step_tests.rs")
old_path = diagnosis / "original-test-source.rs.txt"
original = old_path.read_text()
assert old_path.read_bytes() == git_bytes(source)
current = source.read_text()
restored = current.replace(
    ") -> usize {\n    use sha2::{Digest, Sha256};\n\n    let layouts",
    ") -> usize {\n    let layouts", 1,
)
assert restored != current
start = restored.index("            let effects = f\n")
end = restored.index("            assert_eq!(\n                step.components.len(),", start)
restored = restored[:start] + restored[end:]
pattern = r"            assert_eq!\(\n                step\.components\.len\(\),[\s\S]+?\n            \);"
old_count = re.findall(pattern, original)
new_count = re.findall(pattern, restored)
assert len(old_count) == len(new_count) == 1
assert re.sub(r"\s+", "", new_count[0]) == re.sub(
    r"\s+", "", old_count[0].replace("literals.len()", "literals.len()+effects.len()+constructors.len()")
)
restored = restored.replace(new_count[0], old_count[0], 1)
start = restored.index("            expected_components.extend(\n                effects\n")
end = restored.index("            expected_components.sort();", start)
restored = restored[:start] + restored[end:]
start = restored.index('                    "memory_effect" => {')
end = restored.index('                    other => panic!("unexpected step component {other}"),', start)
restored = restored[:start] + restored[end:]
assert restored == original
assertions = lambda text: len(re.findall(r"\bassert(?:_eq|_ne)?!", text))
assert assertions(current) == assertions(original) + 7
write_new(f"source-preservation-audit{suffix}.json", {
    "status": "passed", "source": seal(source), "original_source": seal(old_path),
    "original_assertion_invocations": assertions(original), "added_assertion_invocations": 7,
    "entire_original_source_recovered_byte_for_byte": True,
    "only_original_assertion_extension": "The component-count RHS retains every original summand and adds the independently identified memory effects and source constructors.",
    "preserved": ["all18 original contexts", "NativeRuntime::Steps", "all original scalar examples and literal forgery checks", "every original argument mutation and bit position", "independent SSA definition origins", "all original positive/negative conjunction evaluations", "strict import mutations", "complete JSON and certificate byte goldens", "all other test functions in this file"],
    "production_sources": production, "full_gate_passed": False,
})
diff = "".join(difflib.unified_diff(
    original.splitlines(True), current.splitlines(True),
    fromfile=str(old_path), tofile=str(source), n=0,
))
diff_path = root / f"test-source{suffix}.diff.log"
assert not diff_path.exists()
diff_path.write_text(diff)

cases = [
    "count_fill", "while", "for", "short_circuit", "switch", "is_binding",
    "guard_order", "guard_throw", "total_variable", "index_update",
    "foreach_string", "foreach_string_var", "foreach_array", "foreach_array_var",
    "lookup", "governing_throw", "type", "string_property",
]
matrix = []
witnesses = []
goldens = Path("develop/migrations/csharp-03/ordinary-foundation/control-edges")
for case in cases:
    program = json.loads((goldens / f"{case}.json").read_bytes())
    roles = Counter()
    complete = 0
    for fi, function in enumerate(program["functions"]):
        nodes = {node["id"]: node for node in function["source"]["source_graph"]["nodes"]}
        for si, step in enumerate(function["source_steps"]):
            if step["pending_reasons"]:
                continue
            complete += 1
            source_node = nodes[step["source_node_id"]]
            frame = next(f for f in function["source_frames"] if f["source_node_id"] == source_node["id"])
            transfers = [r for r in function["slot_relations"] if (r.get("transfer") or {}).get("source_node_id") == source_node["id"]]
            assert len(transfers) <= 1
            operations = [o for o in function["native_operations"] if o["source"]["node_id"] in step["native_node_ids"]]
            literals = [v for v in function["native_literals"] if v["result"]["node_id"] in step["native_node_ids"]]
            effects = [e for e in function.get("memory_effects", []) if e["source_node_id"] == source_node["id"]]
            assert len(effects) == int(source_node["operation"] == "update")
            expected = [("source_frame", frame["definition"])]
            expected += [("slot_transfer", transfer["definition"]) for transfer in transfers]
            expected += [("native_normal", operation["predicates"]["normal_execution"]) for operation in operations]
            expected += [("literal_result", literal["definition"]) for literal in literals]
            old_count = len(expected)
            expected += [("memory_effect", effect["definition"]) for effect in effects]
            if source_node["operation"] == "construct":
                identity = f"source.constructor.{source_node['id']}"
                digest = hashlib.sha256(json.dumps(identity, ensure_ascii=False).encode()).hexdigest()
                expected.append(("source_constructor_normal", f"Mpk.CSharp.Ordinary.SourceValueData.Relation.H{digest}"))
            actual = [(component["role"], component["definition"]) for component in step["components"]]
            assert sorted(actual) == sorted(expected)
            roles.update(role for role, _ in actual)
            if old_count != len(actual):
                witnesses.append({
                    "case": case, "function_index": fi, "step_index": si,
                    "source_node_id": source_node["id"], "source_operation": source_node["operation"],
                    "original_expected_component_count": old_count,
                    "actual_component_count": len(actual), "repaired_expected_component_count": len(expected),
                    "missing_original_roles": [role for role, _ in expected if role in ("memory_effect", "source_constructor_normal")],
                    "complete_components": step["components"],
                })
    matrix.append({"case": case, "complete_steps": complete, "complete_component_roles": dict(roles)})
assert len(witnesses) == 3
assert [(w["case"], w["step_index"]) for w in witnesses] == [("count_fill", 81), ("index_update", 28), ("type", 15)]
assert witnesses[0]["original_expected_component_count"] == 2
assert witnesses[0]["actual_component_count"] == 3
write_new("failure-analysis.json", {
    "status": "original_failure_reproduced_and_component_expectations_audited",
    "original_result": seal(diagnosis / "result.json"),
    "original_target_exit_code": 101, "original_wall_seconds": original_result["wall_seconds"],
    "original_failed_assertion": "source-step component count, original line129, actual3/expected2",
    "cause": "The original runtime observer omits existing memory_effect and source_constructor_normal components. The production implementation and all42 original input files match source320 exactly.",
    "cases": matrix, "witnesses": witnesses, "inputs": inputs, "production_sources": production,
    "scope": "Metadata audit only; it does not replace executing all18 full original runtime contexts on repaired source.",
    "selected_runtime_validation_pending": True, "full_gate_passed": False,
})

prior_build = json.loads((base / "input-echo-followup-repair/build/result.json").read_bytes())
assert prior_build["status"] == "passed"
assert seal(prior_build["executable"]["path"]) == prior_build["executable"]
for item in prior_build["test_sources"]:
    assert seal(item["path"]) == item
destination = Path("/private/tmp/mpk-t01-w05-transition-literal-echo-repaired-test-binary")
if corrected:
    assert destination.exists()
else:
    assert not destination.exists()
    shutil.copy2(prior_build["executable"]["path"], destination)
snapshot = seal(destination)
assert snapshot["sha256"] == prior_build["executable"]["sha256"]
assert snapshot["bytes"] == prior_build["executable"]["bytes"]
write_new("prior-passing-binary-snapshot.json", {
    "status": "passed", "source": prior_build["executable"], "snapshot": snapshot,
    "prior_build": seal(base / "input-echo-followup-repair/build/result.json"),
    "purpose": "Retain the exact executable that passed both complete transition/literal owners before compiling the source-step observer repair.",
})
print(json.dumps({"status": "passed", "original_assertions_preserved": assertions(original), "added_assertions": 7, "original_input_members": len(inputs), "cases": len(matrix), "component_mismatches_explained": len(witnesses), "snapshot": snapshot}))
