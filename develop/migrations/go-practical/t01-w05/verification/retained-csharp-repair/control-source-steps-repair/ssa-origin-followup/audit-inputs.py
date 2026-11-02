"""Audit SSA definition origins without weakening the complete runtime owner."""

from pathlib import Path
import difflib
import hashlib
import json
import re
import subprocess


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
previous = root.parent
diagnosis = previous.parent / "control-source-steps-diagnosis"
source320 = "320ecd67ab420e1ea806196271eea3271f929fbe"


def seal(path):
    data = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


def write_new(name, value):
    path = root / name
    assert not path.exists(), str(path)
    path.write_text(json.dumps(value, indent=2) + "\n")


def git_bytes(revision, path):
    return subprocess.check_output(["/usr/bin/git", "show", f"{revision}:{path}"])


normal_path = previous / "normal/result.json"
normal = json.loads(normal_path.read_bytes())
assert normal["status"] == "failed" and normal["exit_code"] == 101
for stream in ("stdout", "stderr"):
    assert seal(normal[stream]["path"]) == normal[stream]
snapshot = json.loads((root / "previous-attempt-snapshot.json").read_bytes())
assert snapshot["normal_result"] == seal(normal_path)
assert seal(snapshot["executable_snapshot"]["path"]) == snapshot["executable_snapshot"]
for item in snapshot["source_members"]:
    saved = seal(item["snapshot_path"])
    assert all(saved[key] == item[key] for key in ("sha256", "bytes"))

support = Path("crates/mpk-vc/tests/support")
step_path = support / "csharp_practical_ordinary_control_step_tests.rs"
old_step_path = diagnosis / "original-test-source.rs.txt"
original = old_step_path.read_text()
assert old_step_path.read_bytes() == git_bytes(source320, step_path)
current = step_path.read_text()
start = current.index("pub(super) fn run_steps(")
inner = current.index("fn inspect_steps(", start)
prefix = current[start:inner]
assert prefix.count("inspect_steps(p, vir, cert, true)") == 1
assert prefix.count("inspect_steps(p, vir, cert, false)") == 1
assert current.count("inspect_steps(") == 3
assert current.count("if !runtime {") == 2
assert "    runtime: bool,\n" in current[inner:current.index(") -> usize {", inner)]
old_start = original.index("pub(super) fn run_steps(")
old_body = original.index(") -> usize {", old_start) + len(") -> usize {")
body = current.index(") -> usize {", inner) + len(") -> usize {")
restored = current[:start] + original[old_start:old_body] + current[body:]
guards = [
    "            if !runtime {\n                observations += 1;\n                continue;\n            }\n",
    "    if !runtime {\n        assert!(complete > 0);\n        return observations;\n    }\n",
]
for guard in guards:
    assert restored.count(guard) == 1
    restored = restored.replace(guard, "", 1)
restored = restored.replace(
    ") -> usize {\n    use sha2::{Digest, Sha256};\n\n    let layouts",
    ") -> usize {\n    let layouts", 1,
)
start = restored.index("            let effects = f\n")
end = restored.index("            assert_eq!(\n                step.components.len(),", start)
restored = restored[:start] + restored[end:]
pattern = r"            assert_eq!\(\n                step\.components\.len\(\),[\s\S]+?\n            \);"
old_count, = re.findall(pattern, original)
new_count, = re.findall(pattern, restored)
assert re.sub(r"\s+", "", new_count) == re.sub(r"\s+", "", old_count.replace("literals.len()", "literals.len()+effects.len()+constructors.len()"))
restored = restored.replace(new_count, old_count, 1)
start = restored.index("            expected_components.extend(\n                effects\n")
end = restored.index("            expected_components.sort();", start)
restored = restored[:start] + restored[end:]
start = restored.index('                    "memory_effect" => {')
end = restored.index('                    other => panic!("unexpected step component {other}"),', start)
restored = restored[:start] + restored[end:]
assert restored == original
assertions = lambda text: len(re.findall(r"\bassert(?:_eq|_ne)?!", text))
assert assertions(current) == assertions(original) + 9

edge_path = support / "csharp_practical_ordinary_control_edge_tests.rs"
old_edge_path = root / "previous-attempt/csharp_practical_ordinary_control_edge_tests.rs.txt"
old_edge = old_edge_path.read_text()
assert old_edge_path.read_bytes() == git_bytes("HEAD", edge_path)
edge = edge_path.read_text()
start = edge.index("            let component_count = steps::audit_step_components(")
end = edge.index('            eprintln!("control source step metadata {id}:', start)
added = edge[start:end]
assert re.sub(r"\s+", "", added) == re.sub(r"\s+", "", '''            let component_count = steps::audit_step_components(&p, vir, &c);
            eprintln!(
                "control source step component metadata {id}: {component_count} complete local components"
            );
''')
assert edge[:start] + edge[end:] == old_edge
assert assertions(edge) == assertions(old_edge)
for name, old, new in (("step", original, current), ("harness", old_edge, edge)):
    (root / f"{name}-source.diff.log").write_text("".join(difflib.unified_diff(old.splitlines(True), new.splitlines(True), fromfile=f"original-{name}", tofile=f"current-{name}", n=0)))

prior_analysis = json.loads((previous / "failure-analysis.json").read_bytes())
inputs = []
for item in prior_analysis["inputs"] + prior_analysis["production_sources"]:
    actual = seal(item["path"])
    assert all(actual[key] == item[key] for key in ("path", "sha256", "bytes"))
    assert Path(item["path"]).read_bytes() == git_bytes(source320, item["path"])
    inputs.append(dict(actual, matches_source320_git_bytes=True))
assert len(prior_analysis["inputs"]) == 42 and len(prior_analysis["production_sources"]) == 3

error = (previous / "normal/stderr.log").read_text()
assert "csharp_practical_ordinary_control_step_tests.rs:385:25" in error
operands = re.search(r"\n  left: (\[.*\])\n right: (\[.*\])\n", error)
assert operands
binding = re.compile(r'ControlBinding \{ kind: ("[^"]*"), edge_id: None, node_id: ("[^"]*"), value_id: ("[^"]*"), type_id: ("[^"]*") \}')


def decode(raw):
    assert re.fullmatch(r"\[binding(?:, binding)*\]", binding.sub("binding", raw))
    return [dict(zip(("kind", "node_id", "value_id", "type_id"), map(json.loads, fields)), edge_id=None) for fields in binding.findall(raw)]


left, right = map(decode, operands.groups())
assert len(left) == len(right) == 8
different = [i for i in range(8) if left[i] != right[i]]
assert different == [4, 5, 6, 7]
for i in different:
    assert left[i]["kind"] == right[i]["kind"] == "ssa"
    assert {key for key in left[i] if left[i][key] != right[i][key]} == {"node_id"}
golden = Path("develop/migrations/csharp-03/ordinary-foundation/control-edges/count_fill.json")
function = json.loads(golden.read_bytes())["functions"][0]
step = function["source_steps"][81]
component, = [c for c in step["components"] if c["role"] == "memory_effect"]
assert [step["arguments"][i] for i in component["argument_indices"]] == left
effect, = [e for e in function["memory_effects"] if e["source_node_id"] == step["source_node_id"]]
translation = {"ssa": "ssa", "source_before_assigned": "source_entry_assigned", "source_before_slot": "source_entry_slot", "source_after_assigned": "source_exit_assigned", "source_after_slot": "source_exit_slot"}
assert [dict(argument, kind=translation[argument["kind"]]) for argument in effect["arguments"]] == right
assert all(right[i]["node_id"] == step["native_node_ids"][0] for i in different)
write_new("binding-origin-analysis.json", {
    "status": "exact_failed_operands_decoded_and_original_inputs_preserved",
    "failed_complete_attempt": seal(normal_path), "failed_seconds": normal["wall_seconds"],
    "source_and_binary_snapshot": seal(root / "previous-attempt-snapshot.json"),
    "actual_step_bindings": left, "previous_observer_expected_bindings": right,
    "differing_indices": different, "only_differing_field": "node_id",
    "cause": "The added observer copied memory-effect SSA use sites into the expected definition-point binding. Step SSA arguments use their unique original VIR definition points. Source snapshot bindings and every value/type/kind/edge field already agree.",
    "repair": "Independently reconstruct each expected SSA origin from original VIR parameters, phis, literals, invocation normal results and handler exceptions; require uniqueness. Reuse the same complete component observer in the existing metadata owners before starting another full runtime attempt.",
    "inputs": inputs[:42], "production_sources": inputs[42:],
    "full_runtime_validation_required": True, "full_gate_passed": False,
})
write_new("source-preservation-audit.json", {
    "status": "passed", "sources": [seal(step_path), seal(edge_path)],
    "original_step_source": seal(old_step_path), "original_harness_source": seal(old_edge_path),
    "step_original_whole_file_assertions": assertions(original), "step_added_assertions": 9,
    "harness_original_whole_file_assertions": assertions(old_edge), "harness_added_assertions": 0,
    "both_entire_original_sources_recovered_byte_for_byte": True,
    "runtime_entry": "run_steps hardcodes inspect_steps(...,true); all original runtime paths, examples, mutations, bits, positive/negative checks and strict imports remain.",
    "metadata_entry": "Only the existing StepMetadata owners additionally call audit_step_components, which hardcodes false and stops before runtime evaluation. No environment switch or caller-supplied runtime flag exists.",
    "original_runtime_contexts": 18, "original_runtime_goldens": 36,
    "original_count_summands_and_original_assertion_calls_preserved": True,
    "production_source_changes": False, "original_fixture_changes": False,
    "core_resource_or_stack_limit_changes": False, "full_gate_passed": False,
})
print(json.dumps({"status": "passed", "step_original_assertions": assertions(original), "step_added_assertions": 9, "harness_original_assertions": assertions(old_edge), "source320_input_members": 42, "unchanged_production_sources": 3, "decoded_bindings": len(left), "SSA_origin_differences": different, "full_runtime_entry_hardcodes_true": True}))
