"""Full changed-input/proof/old-byte review; execute outside sandbox for Git."""
from pathlib import Path
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
BASE = "a19a97c4d601460b3ac73062b429bd196b9b9333"
REPORT = OUT / "input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates/mpk-core", "crates/mpk-cert", "crates/mpk-kernel", "go-tools")
script = Path("scripts/check-no-active-gir.sh")
code = script.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_word_index_input_review")
module.__file__ = str(ROOT / script)
sys.modules[module.__name__] = module
sys.argv = [str(script), str(ROOT), "strict"]
exec(compile(code, str(script), "exec"), module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"])) == (127, 4, 39)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
selected = git("diff", "--name-only", "-z", "HEAD", "--") + git("ls-files", "--others", "--exclude-standard", "-z")
paths = sorted(set(selected.decode().split("\0")) - {"", str(REPORT.relative_to(ROOT))})
findings, links, inputs = [], [], []
cache = {}
path_rules = [m for m in matchers if m["scope"] == "exact-path"]
for name in paths:
    path = ROOT / name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name))
    if name not in configured:
        key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m["path"]) for m in path_rules))
        if key not in cache:
            cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors="ignore"), matchers)]
        findings.extend({**finding, "path": name} for finding in cache[key])
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            links.append({"document": name, "target": target, "exists": (path.parent / urllib.parse.unquote(target.split("#", 1)[0])).resolve().exists()})
pair = json.loads((OUT / "paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 346
assert seal(pair["go_binary"]["path"]) == pair["go_binary"]
assert all(seal(row["path"]) == row for row in pair["go_production_sources"])
for row in pair["cases"]:
    for key in ["input", "Rust_report", "stdout", "stderr"]:
        assert seal(row[key]["path"]) == row[key]
    rust = json.loads(Path(row["Rust_report"]["path"]).read_bytes())
    go = json.loads(Path(row["stdout"]["path"]).read_bytes())
    assert rust == row["Rust"] and go == row["Go"]
    if rust["verdict"] == "accepted":
        assert rust["hashes"] == go["hashes"] and rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
    else:
        assert json.loads(rust["error_detail"])["code"] == "CORE_TYPE_MISMATCH" and go["error_kind"] == "core_check"
for mode in ["selected", "lint", "format"]:
    path = OUT / "epochs/explicit-controls" / mode / "result.json"
    record = json.loads(path.read_bytes())
    assert record["status"] == "passed" and record["whole_source_inputs_unchanged"]
    assert all(hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest() == row["sha256"] for row in record["source_inputs"])

# Recover original source bytes through their actual complete archived inputs
# or the exact retained Git blobs. No new run supplies an old source epoch.
archives = [json.loads(p.read_bytes()) for p in sorted((OUT / "epochs").glob("*/inputs.json"))]
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
original_preimages = 0
for archive in archives:
    for row in archive["sources"]:
        if "original_git_blob" in row:
            raw = originals[row["original_git_blob"]]
            assert row["original_commit"] == BASE
        else:
            raw = Path(row["complete_changed_input"]).read_bytes()
        assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
        original_preimages += 1
for path in sorted((OUT / "epochs").glob("*/inputs.json")):
    archive = json.loads(path.read_bytes())
    assert seal(path.parent / "run-checks.py")["sha256"] == archive["runner_sha256"]
runtime = json.loads((OUT / "epochs/controls/additional-runtime-input.json").read_bytes())
assert seal(runtime["input"]["path"]) == runtime["input"]
for name in ["develop/docs/go-practical-implementation-traceability-ledger.md", "develop/migrations/go-practical/t02-w01/README.md"]:
    original = git("show", BASE+":"+name)
    assert Path(name).read_bytes().startswith(original)
name = "develop/migrations/go-practical/t02-w01/requirements.json"
original = json.loads(git("show", BASE+":"+name))
current = json.loads(Path(name).read_bytes())
row = next(row for row in current["requirements"] if row["id"] == "T02-W01-R01")
assert row.pop("preparatory_word_index_checkpoint")["status"] == "scoped_checks_passed"
assert current == original
frozen = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json").read_bytes())["inputs"]
assert len(frozen) == 255 and all(seal(row["path"]) == row for row in frozen)
go_inputs = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/original-packages/integrated/analysis-inputs/captured-inputs.json").read_bytes())["build_inputs"]
assert len(go_inputs) == 185
for row in go_inputs:
    raw = (Path("go-tools/go2vir") / row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
missing = [row for row in links if not row["exists"]]
unchanged = all(seal(row["path"]) == row for row in inputs)
result = {"status": "passed" if not findings and not missing and unchanged else "failed",
          "basis_commit": BASE, "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "inputs": inputs, "source_inputs_unchanged": unchanged, "findings": findings,
          "missing_links": missing, "links_checked": len(links), "paired_full_original_inputs_verified": 346,
          "original_epoch_source_preimages_verified": original_preimages,
          "complete_original_source_epochs": len(archives),
          "frozen_design_inputs_unchanged": 255, "Go_frontend_build_inputs_unchanged": 185,
          "original_matcher_scope_seal_counts": [127, 4, 39],
          "new_source_JSON_directory_or_blanket_exclusions": False,
          "core_and_Go_ref_implementations_unchanged": True,
          "native_release_acceptance": False, "full_T_gate_passed": False}
REPORT.write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({key: value for key, value in result.items() if key != "inputs"}))
raise SystemExit(result["status"] != "passed")
