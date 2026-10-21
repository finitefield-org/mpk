"""Review complete lexicographic-order sources, certificates and actual finite data observations."""
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
BASE = "8814646e76c9dd75d2666e4610e8c091ea44fc8b"
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
module = types.ModuleType("mpk_string_order_input_review")
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
    if path.name.startswith(("byte-","string-","operation-")) and path.suffix == ".json":
        row = json.loads(raw)
        for field in ["all256_left_bytes","all256_right_bytes","all_right_bytes"]:
            if field in row:
                payload = bytes(row[field]);assert len(payload) == 256
                scan(name+"#"+field,payload);decoded.append(seal(name+"#"+field,payload))
        if "left_byte" in row:
            payload = bytes([row["left_byte"]]);scan(name+"#left_byte",payload);decoded.append(seal(name+"#left_byte",payload))
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)",raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:",target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#",1)[0])
            links.append({"document":name,"target":target,"exists":(path.parent/clean).resolve().exists()})

pair=json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"]=="passed" and len(pair["cases"])==339
assert pair["zero_axiom_acceptances"]==2 and pair["genuine_core_type_mismatches"]==337
assert seal(pair["go_binary"]["path"])==pair["go_binary"]
assert all(seal(row["path"])==row for row in pair["go_production_sources"])
assert seal(pair["actual_driver_prelaunch"]["path"])==pair["actual_driver_prelaunch"]
for row in pair["cases"]:
    for field in ["input","Rust_report","stdout","stderr"]:assert seal(row[field]["path"])==row[field]
    rust=json.loads(Path(row["Rust_report"]["path"]).read_bytes());go=json.loads(Path(row["stdout"]["path"]).read_bytes())
    assert rust==row["Rust"] and go==row["Go"]
    if row["expected_acceptance"]:
        assert rust["verdict"]==go["verdict"]=="accepted" and rust["hashes"]==go["hashes"]
        assert rust["axiom_count"]==0 and not rust["axiom_report"]["entries"]
        assert go["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:assert json.loads(rust["error_detail"])["code"]=="CORE_TYPE_MISMATCH" and go["error_kind"]=="core_check"
failed=json.loads((OUT/"failed-paired/result.json").read_bytes())
assert failed["status"]=="passed" and len(failed["cases"])==2
for row in failed["cases"]:
    for field in ["input","Rust_report","stdout","stderr"]:assert seal(row[field]["path"])==row[field]
    assert row["Rust"]["verdict"]==row["Go"]["verdict"]=="rejected"
archives=[json.loads(path.read_bytes())for path in sorted((OUT/"epochs").glob("*/inputs.json"))]
assert len(archives)==10
oids=sorted({row["original_git_blob"]for archive in archives for row in archive["sources"]if "original_git_blob"in row})
raw=subprocess.check_output(["/usr/bin/git","cat-file","--batch"],input=("\n".join(oids)+"\n").encode())
originals,cursor={},0
for oid in oids:
    end=raw.index(b"\n",cursor);got,kind,size=raw[cursor:end].decode().split();assert got==oid and kind=="blob";size=int(size)
    originals[oid]=raw[end+1:end+1+size];cursor=end+size+2
assert cursor==len(raw)
preimages=0
for archive in archives:
    for row in archive["sources"]:
        if "original_git_blob"in row:raw=originals[row["original_git_blob"]];assert row["original_commit"]==BASE
        else:raw=Path(row["complete_changed_input"]).read_bytes()
        assert len(raw)==row["bytes"]and hashlib.sha256(raw).hexdigest()==row["sha256"];preimages+=1
for path in (OUT/"epochs").glob("*/inputs.json"):
    assert seal(path.parent/"run-checks.py")["sha256"]==json.loads(path.read_bytes())["runner_sha256"]
executions=[]
for path in sorted((OUT/"epochs").glob("*/*/result.json")):
    row=json.loads(path.read_bytes());assert row["status"]in {"passed","failed"}and row["exit_code"]is not None and row["whole_source_inputs_unchanged"]
    assert row["source_inputs"]==json.loads((path.parent.parent/"inputs.json").read_bytes())["sources"]
    executions.append({"result":seal(path),"stdout":seal(path.parent/"stdout.log"),"stderr":seal(path.parent/"stderr.log"),"status":row["status"],"command":row["command"]})
assert sum(row["status"]=="failed"for row in executions)==5
current=json.loads((OUT/"epochs/lint-corrected/inputs.json").read_bytes())
assert all(seal(row["path"])["sha256"]==row["sha256"]for row in current["sources"])
previous=json.loads((OUT/"epochs/semantic-controls/inputs.json").read_bytes())
prior_by_name={row["path"]:row for row in previous["sources"]}
now_by_name={row["path"]:row for row in current["sources"]}
assert set(now_by_name)-set(prior_by_name)=={"develop/migrations/go-practical/t02-w01/verification/string-order/epochs/semantic-controls/selected/candidates/order-foundation.mpcert"}
changed=[name for name in prior_by_name if prior_by_name[name]["sha256"]!=now_by_name[name]["sha256"]]
assert changed==["crates/mpk-vc/src/go_practical_ordinary/string/order.rs"]
before=Path(prior_by_name[changed[0]]["complete_changed_input"]).read_text()
expected=before.replace("    fn lifted_prefix(\n        &self,\n","    fn lifted_prefix(\n").replace("self.lifted_prefix(","Self::lifted_prefix(")
assert Path(changed[0]).read_text()==expected
fixtures=OUT/"epochs/semantic-controls/selected/candidates"
assert (fixtures/"order-foundation.mpcert").read_bytes()==(OUT/"epochs/lint-corrected/foundation/candidates/order-foundation.mpcert").read_bytes()
prefixes=json.loads((fixtures/"definition-controls.prefix.json").read_bytes())
assert len(prefixes)==25 and all(row["actual_definition_prefix_typechecks"]and not row["full_certificate_acceptance_claimed"]for row in prefixes)
byte_steps=0
assert len(list(fixtures.glob("byte-*.input.json")))==len(list(fixtures.glob("byte-*.observation.json")))==256
for left in range(256):
    path=fixtures/f"byte-{left:03}.input.json";source=json.loads(path.read_bytes())
    assert source["left_byte"]==left and source["all_right_bytes"]==list(range(256))and not source["source_or_nominal_authority"]
    output=json.loads(path.with_name(path.name.replace(".input.json",".observation.json")).read_bytes())
    expected=[[left<right,left<=right,left>right,left>=right]for right in range(256)]
    assert output["actual"]==output["expected"]==expected and not output["definition_callbacks"]and not output["theorem_bodies_executed"]
    byte_steps+=output["actual_AST_steps"]
assert byte_steps==113660694
string_steps=0
assert len(list(fixtures.glob("string-*.input.json")))==75 and len(list(fixtures.glob("string-*.observation.json")))==150
for path in fixtures.glob("string-*.input.json"):
    source=json.loads(path.read_bytes());n,m=source["lengths"];a,b=bytes(source["all256_left_bytes"]),bytes(source["all256_right_bytes"])
    assert len(a)==len(b)==256 and 0<=n<=256 and 0<=m<=256 and not source["source_or_nominal_authority"]
    x,y=a[:n],b[:m];expected=[x<y,x<=y,x>y,x>=y,y<x,y<=x,y>x,y>=x]
    assert source["expected_both_directions"]==expected
    for mode in ["raw","normalized"]:
        stem=path.name.removesuffix(".input.json")+"-"+mode
        output=json.loads((fixtures/(stem+".observation.json")).read_bytes())
        assert output["actual"]==output["expected"]==expected and output["complete_carrier_observed"]
        assert not output["definition_callbacks"]and not output["theorem_bodies_executed"];string_steps+=output["actual_AST_steps"]
        for count in range(1,9):
            partial=json.loads((fixtures/f"{stem}-{count:02}.partial.json").read_bytes())
            assert partial["available_results"]==expected[:count] and partial["actual_AST_steps"]<=output["actual_AST_steps"]
assert string_steps==147304477
assert len(list(fixtures.glob("operation-*.input.json")))==len(list(fixtures.glob("operation-*.counterexample.json")))==25
for path in fixtures.glob("operation-*.input.json"):
    source=json.loads(path.read_bytes());n,m=source["lengths"];a,b=bytes(source["all256_left_bytes"]),bytes(source["all256_right_bytes"])
    assert len(a)==len(b)==256 and 0<=n<=256 and 0<=m<=256 and not any(a[n:])and not any(b[m:])
    assert source["canonical_padding"] and not source["source_or_nominal_authority"]
    x,y=a[:n],b[:m];expected={"Less":x<y,"LessEqual":x<=y,"Greater":x>y,"GreaterEqual":x>=y}[source["operation"]]
    output=json.loads(path.with_name(path.name.replace(".input.json",".counterexample.json")).read_bytes())
    assert output["actual_definition_counterexample"] and output["actual"]!=output["expected"]==source["expected"]==expected
    assert not output["rejected_theorem_bodies_executed"]
original_diagnostic=OUT/"epochs/original-boolean-diagnosis/diagnostic/candidates/original-total-algebra.observation.json"
rows=json.loads(original_diagnostic.read_bytes());assert len(rows)==16
for index,row in enumerate(rows):assert row["case"]==index and row["actual_endpoints"]==row["expected_endpoints"]==[index==0,index==0] and not row["theorem_bodies_executed"]
frozen=json.loads(Path("develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json").read_bytes())["inputs"]
assert len(frozen)==255 and all(seal(row["path"])==row for row in frozen)
go_inputs=json.loads(Path("develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json").read_bytes())["build_inputs"]
assert len(go_inputs)==186
for row in go_inputs:
    raw=(Path("go-tools/go2vir")/row["path"]).read_bytes();assert len(raw)==row["bytes"]and hashlib.sha256(raw).hexdigest()==row["raw_sha256"]
for name in ["develop/docs/go-practical-implementation-traceability-ledger.md","develop/migrations/go-practical/t02-w01/README.md"]:assert Path(name).read_bytes().startswith(git("show",BASE+":"+name))
original=json.loads(git("show",BASE+":develop/migrations/go-practical/t02-w01/requirements.json"));latest=json.loads(Path("develop/migrations/go-practical/t02-w01/requirements.json").read_bytes())
row=next(row for row in latest["requirements"]if row["id"]=="T02-W01-R01")
assert row.pop("preparatory_string_order_checkpoint")["status"]=="scoped_checks_passed" and latest==original
if (OUT/"git-whitespace-review.json").exists():
    whitespace=json.loads((OUT/"git-whitespace-review.json").read_bytes())
    assert whitespace["status"]=="passed_with_original_output_retained" and whitespace["all_other_staged_files_pass_whitespace_check"]
    assert not whitespace["original_Cargo_output_bytes_modified"] and not whitespace["obsolete_IR_matcher_scope_seal_or_exclusion_changes"]
    for warning in whitespace["exact_raw_output_warnings"]:
        assert seal(warning["path"])=={key:warning[key]for key in ["path","sha256","bytes"]}
missing=[row for row in links if not row["exists"]];unchanged=all(seal(row["path"])==row for row in inputs)
record={"status":"passed"if not findings and not missing and unchanged else"failed","basis_commit":BASE,"observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"inputs":inputs,"source_inputs_unchanged":unchanged,"findings":findings,"missing_links":missing,"links_checked":len(links),"paired_complete_original_inputs":339,"zero_axiom_acceptances":2,"genuine_core_type_mismatches":337,"original_failed_candidates_independently_rejected":2,"complete_original_source_epochs":len(archives),"complete_epoch_preimages_verified":preimages,"complete_actual_executions":executions,"complete_byte_pairs_observed":65536,"byte_AST_steps":byte_steps,"whole_string_cases_observed":75,"raw_and_normalized_string_results":150,"string_AST_steps":string_steps,"actual_typed_definition_counterexamples":25,"source_or_native_Go_proof_from_model_claimed":False,"complete_decoded_data_payloads":decoded,"frozen_inputs_unchanged":255,"current_Go_frontend_inputs_unchanged":186,"matcher_scope_seal_counts":[127,4,39],"new_source_JSON_directory_or_blanket_exclusions":False,"core_and_Go_ref_implementations_unchanged":True,"native_release_acceptance":False,"full_T_gate_passed":False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record["findings"]=findings;record["status"]="passed"if not findings and not missing and unchanged else"failed"
REPORT.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:v for k,v in record.items()if k not in {"inputs","complete_actual_executions","complete_decoded_data_payloads"}}))
raise SystemExit(record["status"]!="passed")
