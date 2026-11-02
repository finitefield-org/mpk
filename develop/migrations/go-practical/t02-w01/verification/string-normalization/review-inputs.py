"""Review complete changed inputs, original failures and retained core outcomes."""
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
BASE = "a874633cef63dcfa10965d0b702789937969f98c"
REPORT = OUT/"input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates/mpk-core", "crates/mpk-cert", "crates/mpk-kernel", "go-tools", "release")
assert not Path(".github/workflows").exists()
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n",1)[1].rsplit("\nPY",1)[0].rsplit("raise SystemExit(main())",1)[0]
module = types.ModuleType("mpk_string_normalization_input_review")
module.__file__ = str(ROOT/scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner),str(ROOT),"strict"]
exec(compile(code,str(scanner),"exec"),module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest,matchers)
assert (len(matchers),len(module.SCOPES),len(manifest["sealed_records"])) == (127,4,39)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git("show",BASE+":"+name)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
findings, inputs, links, decoded = [], [], [], []
cache = {}
rules = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name,raw):
    key = (hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m["path"]) for m in rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name,raw.decode(errors="ignore"),matchers)]
    findings.extend({**finding,"path":name} for finding in cache[key])


selected = git("diff","--name-only","-z","HEAD","--") + git("ls-files","--others","--exclude-standard","-z")
names = sorted(set(selected.decode().split("\0")) - {"",str(REPORT.relative_to(ROOT))})
for name in names:
    path = ROOT/name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name,raw))
    if name not in configured:
        scan(name,raw)
    if path.name.startswith("raw-") and path.suffix == ".json":
        row = json.loads(raw)
        for field in ["all256_raw_bytes","raw","normalized","twice","expected"]:
            if field in row:
                payload = bytes(row[field])
                assert len(payload) == 256
                scan(name+"#"+field,payload)
                decoded.append(seal(name+"#"+field,payload))
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)",raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:",target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#",1)[0])
            links.append({"document":name,"target":target,"exists":(path.parent/clean).resolve().exists()})

pair = json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 65
assert pair["zero_axiom_acceptances"] == 2 and pair["genuine_core_type_mismatches"] == 63
assert seal(pair["go_binary"]["path"]) == pair["go_binary"]
assert all(seal(row["path"]) == row for row in pair["go_production_sources"])
assert seal(pair["actual_driver_prelaunch"]["path"]) == pair["actual_driver_prelaunch"]
for row in pair["cases"]:
    for field in ["input","Rust_report","stdout","stderr"]:
        assert seal(row[field]["path"]) == row[field]
    rust = json.loads(Path(row["Rust_report"]["path"]).read_bytes())
    go = json.loads(Path(row["stdout"]["path"]).read_bytes())
    assert rust == row["Rust"] and go == row["Go"]
    if row["expected_acceptance"]:
        assert rust["verdict"] == go["verdict"] == "accepted" and rust["hashes"] == go["hashes"]
        assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
        assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        assert json.loads(rust["error_detail"])["code"] == "CORE_TYPE_MISMATCH" and go["error_kind"] == "core_check"
prefixes = json.loads((OUT/"epochs/typed-controls/selected/candidates/definition-controls.prefix.json").read_bytes())
assert len(prefixes) == 7 and all(row["actual_definition_prefix_typechecks"] and not row["full_certificate_acceptance_claimed"] for row in prefixes)

archives = [json.loads(path.read_bytes()) for path in sorted((OUT/"epochs").glob("*/inputs.json"))]
assert len(archives) == 5
oids = sorted({row["original_git_blob"] for archive in archives for row in archive["sources"] if "original_git_blob" in row})
raw = subprocess.check_output(["/usr/bin/git","cat-file","--batch"],input=("\n".join(oids)+"\n").encode())
originals,cursor = {},0
for oid in oids:
    end = raw.index(b"\n",cursor)
    got,kind,size = raw[cursor:end].decode().split()
    assert got == oid and kind == "blob"
    size = int(size)
    originals[oid] = raw[end+1:end+1+size]
    cursor = end+size+2
assert cursor == len(raw)
preimages = 0
for archive in archives:
    for row in archive["sources"]:
        if "original_git_blob" in row:
            raw = originals[row["original_git_blob"]]
            assert row["original_commit"] == BASE
        else:
            raw = Path(row["complete_changed_input"]).read_bytes()
        assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
        preimages += 1
assert preimages == 1842
for path in (OUT/"epochs").glob("*/inputs.json"):
    assert seal(path.parent/"run-checks.py")["sha256"] == json.loads(path.read_bytes())["runner_sha256"]

executions = []
expected_failures = {"initial/foundation":"CORE_NOT_A_FUNCTION","contents-function/foundation":"CORE_FUEL_EXHAUSTED","typed-controls/selected":"finite_observer_resource_ceiling"}
for path in sorted((OUT/"epochs").glob("*/*/result.json")):
    row = json.loads(path.read_bytes())
    assert row["status"] in {"passed","failed"} and row["whole_source_inputs_unchanged"]
    key = str(path.parent.relative_to(OUT/"epochs"))
    assert (row["status"] == "failed") == (key in expected_failures)
    if key in expected_failures:
        assert row["exit_code"] == 101
        if key != "typed-controls/selected":
            report = json.loads((path.parent/"candidates/normalization-foundation.rust.json").read_bytes())
            assert json.loads(report["error_detail"])["code"] == expected_failures[key]
    else:
        assert row["exit_code"] == 0
    executions.append({"result":seal(path),"stdout":seal(path.parent/"stdout.log"),"stderr":seal(path.parent/"stderr.log")})
assert len(executions) == 9

current = json.loads((OUT/"epochs/trimmed-observer/inputs.json").read_bytes())
assert all(seal(row["path"])["sha256"] == row["sha256"] for row in current["sources"])
selected = json.loads((OUT/"epochs/typed-controls/selected/result.json").read_bytes())
delta = [{"path":row["path"],"original_sha256":row["sha256"],"current_sha256":seal(row["path"])["sha256"]}
         for row in selected["source_inputs"] if seal(row["path"])["sha256"] != row["sha256"]]
assert delta == pair["observer_only_source_delta"] and len(delta) == 1
assert delta[0]["path"] == "crates/mpk-vc/tests/support/go_string_normalization_eval.rs"
earlier = json.loads((OUT/"epochs/size-congruence/inputs.json").read_bytes())
for name in ["crates/mpk-vc/src/go_practical_ordinary.rs","crates/mpk-vc/src/go_practical_ordinary/string.rs","crates/mpk-vc/src/go_practical_ordinary/string/normalization.rs"]:
    assert seal(name)["sha256"] == next(row["sha256"] for row in earlier["sources"] if row["path"] == name)
stdout = (OUT/"epochs/typed-controls/selected/stdout.log").read_text()
assert all("test "+name+" ... ok" in stdout for name in pair["Rust_complete_core_test_roots_passed"])
assert "test result: FAILED. 4 passed; 1 failed;" in stdout
for mode in ["observations","lint","format"]:
    assert json.loads((OUT/"epochs/trimmed-observer"/mode/"result.json").read_bytes())["status"] == "passed"

old = OUT/"epochs/typed-controls/selected/candidates"
new = OUT/"epochs/trimmed-observer/observations/candidates"
assert len(list(old.glob("raw-*.input.json"))) == 31 and len(list(old.glob("raw-*.observation.json"))) == 30
assert len(list(new.glob("raw-*.input.json"))) == len(list(new.glob("raw-*.observation.json"))) == 54
lengths = [0,1,2,3,7,8,15,16,31,32,63,64,127,128,129,254,255,256]
steps, observed_cases = 0,set()
for path in new.glob("raw-*.input.json"):
    _,n,p = path.name.split(".",1)[0].split("-")
    n,p = int(n),int(p)
    assert n in lengths and p in {0,1,2}
    expected_raw = [255 if p == 0 else i if p == 1 else (i*37+129)%256 for i in range(256)]
    source = json.loads(path.read_bytes())
    assert source["full_length_u64"] == n and source["all256_raw_bytes"] == expected_raw
    assert not source["source_or_nominal_authority"]
    output = json.loads(path.with_name(path.name.replace(".input.json",".observation.json")).read_bytes())
    expected = expected_raw[:n] + [0]*(256-n)
    assert output["lengths"] == [n,n,n] and output["raw"] == expected_raw
    assert output["normalized"] == output["twice"] == output["expected"] == expected
    assert not output["arithmetic_or_string_definition_callbacks"] and not output["theorem_bodies_executed"]
    assert not output["universal_core_proof_claimed_from_finite_model"]
    steps += output["actual_AST_steps"]
    observed_cases.add((n,p))
assert steps == 28685384 and observed_cases == {(n,p) for n in lengths for p in range(3)}
for path in old.glob("raw-*.input.json"):
    assert path.read_bytes() == (new/path.name).read_bytes()
for path in old.glob("raw-*.observation.json"):
    a,b = json.loads(path.read_bytes()),json.loads((new/path.name).read_bytes())
    a.pop("actual_AST_steps");b.pop("actual_AST_steps")
    assert a == b
assert (old/"raw-63-0.input.json").exists() and not (old/"raw-63-0.observation.json").exists()
old_source = next(row for row in json.loads((OUT/"epochs/typed-controls/inputs.json").read_bytes())["sources"] if row["path"] == delta[0]["path"])
for path in [Path(old_source["complete_changed_input"]),Path(delta[0]["path"])]:
    source = path.read_text()
    assert all(text in source for text in ["self.steps <= 50_000_000","self.values.len() <= 2_000_000","self.environments.len() <= 2_000_000"])

frozen = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json").read_bytes())["inputs"]
assert len(frozen) == 255 and all(seal(row["path"]) == row for row in frozen)
go_inputs = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json").read_bytes())["build_inputs"]
assert len(go_inputs) == 186
for row in go_inputs:
    raw = (Path("go-tools/go2vir")/row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
for name in ["develop/docs/go-practical-implementation-traceability-ledger.md","develop/migrations/go-practical/t02-w01/README.md"]:
    assert Path(name).read_bytes().startswith(git("show",BASE+":"+name))
original = json.loads(git("show",BASE+":develop/migrations/go-practical/t02-w01/requirements.json"))
current = json.loads(Path("develop/migrations/go-practical/t02-w01/requirements.json").read_bytes())
row = next(row for row in current["requirements"] if row["id"] == "T02-W01-R01")
assert row.pop("preparatory_string_normalization_checkpoint")["status"] == "scoped_checks_passed"
assert current == original
whitespace = json.loads((OUT/"git-whitespace-review.json").read_bytes())
assert whitespace["status"] == "passed_with_original_output_retained"
assert whitespace["all_other_staged_files_pass_whitespace_check"] and not whitespace["original_Cargo_output_bytes_modified"]
assert not whitespace["obsolete_IR_matcher_scope_seal_or_exclusion_changes"]
assert len(whitespace["exact_raw_output_warnings"]) == 5
assert all(seal(row["path"]) == row for row in whitespace["exact_raw_output_warnings"])
missing = [row for row in links if not row["exists"]]
unchanged = all(seal(row["path"]) == row for row in inputs)
record = {"status":"passed" if not findings and not missing and unchanged else "failed",
          "basis_commit":BASE,"observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "inputs":inputs,"source_inputs_unchanged":unchanged,"findings":findings,"missing_links":missing,"links_checked":len(links),
          "paired_complete_original_inputs":65,"zero_axiom_acceptances":2,"genuine_core_type_mismatches":63,
          "complete_original_source_epochs":5,"complete_epoch_preimages_verified":preimages,
          "complete_actual_executions":executions,"retained_original_failed_executions":expected_failures,
          "four_core_roots_retained_without_repeat":pair["Rust_complete_core_test_roots_passed"],"observer_only_source_delta":delta,
          "corrected_complete_observations":54,"retained_original_inputs":31,"retained_original_available_outputs":30,
          "complete_decoded_data_payloads":decoded,"frozen_inputs_unchanged":255,"current_Go_frontend_inputs_unchanged":186,
          "matcher_scope_seal_counts":[127,4,39],"new_source_JSON_directory_or_blanket_exclusions":False,
          "original_Cargo_stdout_EOF_blank_line_warnings_retained":5,
          "core_and_Go_ref_implementations_unchanged":True,"native_release_acceptance":False,"full_T_gate_passed":False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode())
record["findings"] = findings
record["status"] = "passed" if not findings and not missing and unchanged else "failed"
REPORT.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:v for k,v in record.items() if k not in {"inputs","complete_actual_executions","complete_decoded_data_payloads"}}))
raise SystemExit(record["status"] != "passed")
