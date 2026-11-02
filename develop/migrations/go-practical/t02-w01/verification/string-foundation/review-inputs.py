"""Full changed/old/native/certificate review; run outside sandbox for Git."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = "22c964d225582ff6d955c4fe64ad98bbe4ce028f"
REPORT = OUT / "input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates/mpk-core", "crates/mpk-cert", "crates/mpk-kernel", "go-tools", "release")
assert not Path(".github/workflows").exists()
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_string_foundation_input_review")
module.__file__ = str(ROOT / scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(ROOT), "strict"]
exec(compile(code, str(scanner), "exec"), module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"])) == (127, 4, 39)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git("show", BASE+":"+name)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
findings, inputs, links, decoded = [], [], [], []
cache = {}
path_rules = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name, raw):
    key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m["path"]) for m in path_rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors="ignore"), matchers)]
    findings.extend({**finding, "path": name} for finding in cache[key])


def native_payload(path, raw):
    original = json.loads(raw)
    if "source_object" not in original:
        return
    values = {}
    for field in ["native_bytes", "original_source", "original_expression"]:
        values[field] = b"" if original[field] is None else base64.b64decode(original[field], validate=True)
        scan(path+"#"+field, values[field])
        decoded.append(seal(path+"#"+field, values[field]))
    obj, source = original["source_object"], values["original_source"]
    assert obj["schema"] == "mpk.go.source_object.v1" and obj["kind"] == "go_file"
    assert len(source) == obj["bytes"] and hashlib.sha256(source).hexdigest() == obj["raw_sha256"]
    canonical = json.dumps({k:v for k,v in obj.items() if k != "id"}, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0"+canonical).hexdigest() == obj["id"]
    span = original["span"]
    assert span["source_id"] == obj["id"] and source[span["start"]:span["end"]] == values["original_expression"]
    assert original["ordinary_source_evaluation_or_nominal_identity_proof"] is False and original["registered_context_or_program_proof"] is False


selected = git("diff", "--name-only", "-z", "HEAD", "--") + git("ls-files", "--others", "--exclude-standard", "-z")
paths = sorted(set(selected.decode().split("\0")) - {"", str(REPORT.relative_to(ROOT))})
for name in paths:
    path = ROOT / name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name, raw))
    if name not in configured:
        scan(name, raw)
    if name.endswith(".native.json"):
        native_payload(name, raw)
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#", 1)[0])
            links.append({"document": name, "target": target, "exists": (path.parent / clean).resolve().exists()})

pair = json.loads((OUT / "paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 81
assert pair["zero_axiom_acceptances"] == 22 and pair["genuine_core_type_mismatches"] == 59
assert seal(pair["go_binary"]["path"]) == pair["go_binary"]
assert all(seal(row["path"]) == row for row in pair["go_production_sources"])
assert seal(pair["actual_driver_prelaunch"]["path"]) == pair["actual_driver_prelaunch"]
for row in pair["cases"]:
    for key in ["input", "Rust_report", "stdout", "stderr"]:
        assert seal(row[key]["path"]) == row[key]
    rust = json.loads(Path(row["Rust_report"]["path"]).read_bytes())
    go = json.loads(Path(row["stdout"]["path"]).read_bytes())
    assert rust == row["Rust"] and go == row["Go"]
    if row["expected_acceptance"]:
        assert rust["verdict"] == go["verdict"] == "accepted" and rust["hashes"] == go["hashes"]
        assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
        assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        assert json.loads(rust["error_detail"])["code"] == "CORE_TYPE_MISMATCH" and go["error_kind"] == "core_check"
assert all(row["definition_prefix_typechecks"] and not row["full_certificate_acceptance_claimed"] for row in json.loads((OUT / "epochs/native-controls/selected/candidates/wrong-operations.prefix.json").read_bytes()))

archives = [json.loads(path.read_bytes()) for path in sorted((OUT / "epochs").glob("*/inputs.json"))]
assert len(archives) == 6
oids = sorted({row["original_git_blob"] for archive in archives for row in archive["sources"] if "original_git_blob" in row})
raw = subprocess.check_output(["/usr/bin/git", "cat-file", "--batch"], input=("\n".join(oids)+"\n").encode())
originals, cursor = {}, 0
for oid in oids:
    end = raw.index(b"\n", cursor)
    got, kind, size = raw[cursor:end].decode().split()
    assert got == oid and kind == "blob"
    size = int(size)
    originals[oid] = raw[end+1:end+1+size]
    cursor = end+size+2
assert cursor == len(raw)
preimages, old_native_preimages = 0, 0
for archive in archives:
    for row in archive["sources"]:
        if "original_git_blob" in row:
            raw = originals[row["original_git_blob"]]
            assert row["original_commit"] == BASE
        else:
            raw = Path(row["complete_changed_input"]).read_bytes()
        assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
        preimages += 1
        if Path(row["path"]).is_absolute():
            native_payload(row["path"], raw)
            old_native_preimages += 1
for path in sorted((OUT / "epochs").glob("*/inputs.json")):
    archive = json.loads(path.read_bytes())
    assert seal(path.parent / "run-checks.py")["sha256"] == archive["runner_sha256"]

executions = []
for path in sorted((OUT / "epochs").glob("*/*/result.json")):
    row = json.loads(path.read_bytes())
    assert row["status"] in {"passed", "failed"} and row["whole_source_inputs_unchanged"]
    expected_failure = path.parent.parent.name in {"initial", "explicit-zero", "zero-import"} or (path.parent.parent.name == "native-controls" and path.parent.name == "lint")
    assert row["status"] == ("failed" if expected_failure else "passed")
    assert row["exit_code"] == (101 if expected_failure else 0)
    executions.append({"result": seal(path), "stdout": seal(path.parent/"stdout.log"), "stderr": seal(path.parent/"stderr.log")})
for mode, epoch in [("selected", "native-controls"), ("lint", "lint-cleanup"), ("format", "lint-cleanup")]:
    row = json.loads((OUT/"epochs"/epoch/mode/"result.json").read_bytes())
    assert row["status"] == "passed"
selected = json.loads((OUT/"epochs/native-controls/inputs.json").read_bytes())
for name in ["crates/mpk-vc/src/go_practical_ordinary.rs", "crates/mpk-vc/src/go_practical_ordinary/string.rs"]:
    row = next(row for row in selected["sources"] if row["path"] == name)
    assert seal(name)["sha256"] == row["sha256"]
test = next(row for row in selected["sources"] if row["path"] == "crates/mpk-vc/tests/go_practical_string_foundation.rs")
old = Path(test["complete_changed_input"]).read_text()
start = old.index('                format!(\n                    "{{\\"bytes\\":257,')
end = old.index('.as_bytes(),', start)+len('.as_bytes(),')
expected = old[:start]+'                b"{\\"bytes\\":257,\\"modeled_domain_error\\":true,\\"semantic_falsity\\":false}\\n",'+old[end:]
assert Path(test["path"]).read_text() == expected

native = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/string-constants/input-review.json").read_bytes())
original_native = [row for row in native["original_evidence_files"] if Path(row["path"]).name.endswith("-native.json") and ("-complete-outputs-selected-evidence/" in row["path"] or "-complete-outputs-replay/evidence/" in row["path"])]
assert len(original_native) == 19
for row in original_native:
    assert seal(row["path"]) == row
for path in (OUT/"epochs/native-controls/selected/candidates").glob("*.native.json"):
    source = next(row for row in original_native if Path(row["path"]).stem == path.name.removeprefix("native-").removesuffix(".native.json"))
    assert path.read_bytes() == Path(source["path"]).read_bytes()
frozen = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json").read_bytes())["inputs"]
assert len(frozen) == 255 and all(seal(row["path"]) == row for row in frozen)
go_inputs = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json").read_bytes())["build_inputs"]
assert len(go_inputs) == 186
for row in go_inputs:
    raw = (Path("go-tools/go2vir") / row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
for name in ["develop/docs/go-practical-implementation-traceability-ledger.md", "develop/migrations/go-practical/t02-w01/README.md"]:
    assert Path(name).read_bytes().startswith(git("show", BASE+":"+name))
original = json.loads(git("show", BASE+":develop/migrations/go-practical/t02-w01/requirements.json"))
current = json.loads(Path("develop/migrations/go-practical/t02-w01/requirements.json").read_bytes())
row = next(row for row in current["requirements"] if row["id"] == "T02-W01-R01")
assert row.pop("preparatory_frozen_string_carrier_checkpoint")["status"] == "scoped_checks_passed"
assert current == original
missing = [row for row in links if not row["exists"]]
unchanged = all(seal(row["path"]) == row for row in inputs)
record = {"status": "passed" if not findings and not missing and unchanged else "failed",
          "basis_commit": BASE, "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "inputs": inputs, "source_inputs_unchanged": unchanged, "findings": findings,
          "missing_links": missing, "links_checked": len(links), "paired_complete_original_inputs": 81,
          "zero_axiom_acceptances": 22, "genuine_core_type_mismatches": 59,
          "complete_original_source_epochs": 6, "complete_epoch_preimages_verified": preimages,
          "original_native_input_preimages_verified": old_native_preimages,
          "complete_actual_executions": executions, "complete_decoded_payloads": decoded,
          "final_test_only_change": "Fixed native-domain record format! replaced by exact identical byte string; production recipes and all original outcomes unchanged.",
          "frozen_inputs_unchanged": 255, "current_Go_frontend_inputs_unchanged": 186,
          "matcher_scope_seal_counts": [127, 4, 39], "new_source_JSON_directory_or_blanket_exclusions": False,
          "core_and_Go_ref_implementations_unchanged": True, "native_release_acceptance": False, "full_T_gate_passed": False}
scan(str(REPORT.relative_to(ROOT)), json.dumps(record, indent=2).encode())
record["findings"] = findings
record["status"] = "passed" if not findings and not missing and unchanged else "failed"
REPORT.write_text(json.dumps(record, indent=2)+"\n")
print(json.dumps({k:v for k,v in record.items() if k not in {"inputs", "complete_actual_executions", "complete_decoded_payloads"}}))
raise SystemExit(record["status"] != "passed")
