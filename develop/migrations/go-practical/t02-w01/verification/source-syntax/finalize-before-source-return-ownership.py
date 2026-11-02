"""Independently bind complete original source bytes, physical origins and real checks."""
from pathlib import Path
import base64
import collections
import datetime
import hashlib
import json


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def source_identity(source, raw):
    assert set(source) == {"schema", "id", "capability", "path", "kind", "raw_sha256", "bytes"}
    assert source["schema"] == "mpk.go.source_object.v1"
    assert source["raw_sha256"] == hashlib.sha256(raw).hexdigest() and source["bytes"] == len(raw)
    preimage = {k: v for k, v in source.items() if k != "id"}
    canonical = json.dumps(preimage, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + canonical).hexdigest() == source["id"]
    return {"source": source, "original_bytes": len(raw), "independent_preimage_sha256": source["id"]}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "reflected-interfaces"
checks = []
for name in ["selected", "legacy", "vet", "format"]:
    path = current / name / "result.json"
    record = json.loads(path.read_text())
    assert record["status"] == "passed" and record["exit_code"] == 0 and record["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(record[key]["path"]) == record[key]
    for source in record["sources"]:
        assert seal(source["path"]) == source
        original = seal(Path(record["source_snapshot"]) / source["path"])
        assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
    if name == "selected":
        assert len(record["root_tests_passed"]) == 11 and len(record["subtests_passed"]) == 99
    if name == "legacy":
        assert not record["root_tests_passed"] and not record["subtests_passed"]
    checks.append({"record": seal(path), "seconds": record["wall_seconds"]})
selected = current / "selected"
catalog_path = Path("develop/migrations/go-practical/t01-w04/source-admission/source-vectors.json")
facts_path = catalog_path.parent / "native-facts.jsonl"
catalog = json.loads(catalog_path.read_text())["cases"]
facts = [json.loads(row) for row in facts_path.read_text().splitlines()]
observed_path = selected / "all81-originals.json"
observed = json.loads(observed_path.read_text())
assert len(catalog) == len(facts) == len(observed["cases"]) == 81
assert observed["parsed"] == 80 and observed["syntax_errors"] == 1 and observed["distinct_nodes"] == 2090
identities, cases, source_errors, native_nodes = [], [], 0, 0
native_kinds = collections.Counter()
for case, native, observation in zip(catalog, facts, observed["cases"], strict=True):
    assert observation["id"] == case["id"]
    raw = base64.b64decode(observation["raw"], validate=True)
    assert raw == case["source"].encode() and hashlib.sha256(raw).hexdigest() == native["source_sha256"]
    source = observation["source"]
    assert source["capability"] == "workspace/main" and source["path"] == f"cases/{case['id']}.go" and source["kind"] == "go_file"
    identities.append(source_identity(source, raw))
    native_kinds[native["kind"]] += 1
    if case["expected_status"] == "source_error":
        assert observation["syntax_error"] and not observation.get("nodes")
        source_errors += 1
        cases.append({"id": case["id"], "source_id": source["id"], "original_syntax_error_preserved": True})
        continue
    nodes = observation["nodes"]
    actual = {(n["kind"].removeprefix("*ast."), n["start"], n["end"]) for n in nodes}
    for node in nodes:
        assert 0 <= node["start"] <= node["end"] <= len(raw)
    assert len(actual) == observation["distinct_nodes"] == len(nodes)
    count = 0
    for function in native.get("functions", []):
        for node in function["nodes"]:
            assert (node["kind"], node["start_byte"], node["end_byte"]) in actual
            native_nodes += 1
            count += 1
    cases.append({"id": case["id"], "source_id": source["id"], "complete_distinct_nodes": len(actual),
                  "original_native_function_node_obligations": count, "typecheck_or_admission_claimed": False})
assert source_errors == 1 and native_nodes == 1710
assert sum(c.get("complete_distinct_nodes", 0) for c in cases) == 2090
physical_path = selected / "physical-origins.json"
physical = json.loads(physical_path.read_text())
raw = base64.b64decode(physical["raw"], validate=True)
identities.append(source_identity(physical["source"], raw))
comments = []
for node in physical["nodes"]:
    start, end = node["start"], node["end"]
    assert 0 <= start <= end <= len(raw)
    assert base64.b64decode(node["raw"], validate=True) == raw[start:end]
    if node["kind"] == "*ast.Comment":
        comments.append(raw[start:end])
assert comments == [b"// original " + "雪".encode() + b"\r", b"//line invented.go:700:9\r", b"/* block\r\n\r comment */", b"// tail\r", b"// final\r"]
assert any(n["kind"] == "*ast.BasicLit" and base64.b64decode(n["raw"]) == b"`a\r\nb\rc`" for n in physical["nodes"])
assert any(n["kind"] == "*ast.ValueSpec" and base64.b64decode(n["raw"]) == b"Raw = `a\r\nb\rc`" for n in physical["nodes"])
build_path, inputs_path = selected / "reproduced-builds.json", selected / "captured-inputs.json"
builds = json.loads(build_path.read_text())
inputs = json.loads(inputs_path.read_text())
assert builds["byte_exact_equal"] and builds["independent_empty_caches"] and len(builds["builds"]) == 2
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (171, 35, 133)
assert len(inputs["build_inputs"]) == 171 and not inputs["application_proof_claimed"]
assert not builds["native_Linux_acceptance_claimed"] and not builds["registered_practical_context"]
for source in inputs["build_inputs"]:
    assert source["capability"] == "analysis-frontend"
    identities.append(source_identity(source, (Path("go-tools/go2vir") / source["path"]).read_bytes()))
assert len(identities) == 253
for build in builds["builds"]:
    assert build["binary_sha256"] == "4d14e1e0d117488441e34a99fbf0ad6329cf7f6b7e63366f799024eac520536c"
    assert build["bytes"] == 8592898 and build["build_info"]["GoVersion"] == "go1.25.0"
failed = []
for attempt, reason in [("initial", "Original ast.Walk(File) path omitted unattached comments; original budget test expected duplicate visits without walking File.Comments."),
                        ("complete-comments", "Production includes complete comments. The new independent reflection observer mistakenly stopped at interface/pointer aliases; all original failures/raw inputs are retained before fixing interface traversal.")]:
    path = root / attempt / "regression/result.json"
    record = json.loads(path.read_text())
    assert record["status"] == "failed" and record["exit_code"] != 0 and record["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(record[key]["path"]) == record[key]
    for source in record["sources"]:
        original = seal(Path(record["source_snapshot"]) / source["path"])
        assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
    failed.append({"record": seal(path), "full_original_sources": record["source_snapshot"], "classification": reason,
                   "semantic_counterexample_claimed": False})
prior_path = root.parent / "product-relations/frozen-design-preservation.json"
frozen = json.loads(prior_path.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for source in frozen["inputs"]:
    assert seal(source["path"]) == source
frozen["prior_complete_manifest"] = seal(prior_path)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {"schema": "mpk.go_practical.source_syntax_checkpoint.v1", "status": "scoped_checks_passed",
          "entry_commit": "11946e845d64e29f242ed8bc5d3477a4ac80dd02",
          "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "checks": checks,
          "original_catalog": seal(catalog_path), "original_native_facts": seal(facts_path),
          "original_cases": cases, "all_original_source_cases": 81, "syntax_parsed": 80,
          "original_syntax_errors_preserved": 1, "original_native_kinds": dict(native_kinds),
          "original_native_function_node_obligations": native_nodes, "materialized_distinct_nodes": 2090,
          "all_original_syntax_observations": seal(observed_path), "physical_token_observations": seal(physical_path),
          "independently_reconstructed_complete_source_object_preimages": identities,
          "complete_source_preimages_checked": 253, "actual_frontend_builds": seal(build_path), "actual_frontend_inputs": seal(inputs_path),
          "original_failed_epochs": failed, "frozen_inputs": seal(frozen_path),
          "selection_reason": "New sealed original-source syntax helper, physical CR/line/Unicode provenance, complete comment/node accounting and all81 original unmodified source cases. Independent full AST reflection matches every2090 node and every1710 original native function-node obligation; complete source preimages reconstructed for81 originals, physical witness and171 frontend/vendor inputs. Snapshot read consumers and two actual fresh-cache frontend builds are affected; Go1.23 compile/link only, current vet/format. Unchanged Rust/scalar/checker/Go archive/module/native matrices retained without unrelated reruns. This parser supplies no package build selection/typecheck/export/compiler/context/application acceptance.",
          "syntax_limit": 4194304, "retained_predecessor_syntax_limit": 1000000,
          "new_production_sources": [seal("go-tools/go2vir/practical_syntax.go")],
          "all_actual_validation_handles_terminal": True, "source_bytes_rewritten_or_stubbed": False,
          "profile_or_ordinary_source_proof_authority": False, "T02_W01_complete": False, "R06_status": "pending",
          "full_T02_gate_passed": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
          "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
          "public_profile_or_Linux_release_claim": False,
          "remaining": "Original complete package build/Go file selection, source/export metadata and effective-language/compiler/module ownership, immutable authoritative original package/export/selected-closure replay remain due. Actual complete bounded56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation with all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies, all169 application demands and T02-T11 remain required; R06 pending. No new all65 relation-consumer or source/application proof claim."}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "original_sources": 81, "distinct_nodes": 2090, "original_native_node_obligations": 1710,
                  "complete_source_preimages_checked": 253, "fresh_build_inputs": 171, "all_actual_validation_handles_terminal": True}))
