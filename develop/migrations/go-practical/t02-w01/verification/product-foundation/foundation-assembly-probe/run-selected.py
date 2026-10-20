"""Scoped full-original ordinary assembly feasibility diagnostic and exact source controls."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode,attempt=sys.argv[1:3];assert mode in {"tests","clippy","format","listing"} and attempt.replace("-","").isalnum()
out=root/attempt/mode;out.mkdir(exist_ok=False,parents=True)
sources=[Path("crates/mpk-vc/src/go_practical_ordinary.rs"),*sorted(Path("crates/mpk-vc/src/go_practical_ordinary").rglob("*.rs")),Path("crates/mpk-vc/tests/go_practical_ordinary_assembly_probe.rs"),Path("crates/mpk-vc/tests/support/go_ordinary_assembly_probe.rs"),Path("crates/mpk-vc/tests/support/go_word_division_dependencies.rs"),*[Path("proofs/std")/p/("std-"+p+".hex")for p in ["bool","eq","nat"]],root.parent/"explicit-result-types/construction/certificates/product-foundation.mpcert",root.parent.parent/"word-shift-count-bounds/final/rust-tests/certificates/word-shift-counts.mpcert"]
records=[seal(p)for p in sources];archive=root/attempt/"inputs"
for source in records:
    target=archive/source["path"]
    if target.exists():
        stored=seal(target);assert(stored["sha256"],stored["bytes"])==(source["sha256"],source["bytes"])
    else:target.parent.mkdir(exist_ok=True,parents=True);target.write_bytes(Path(source["path"]).read_bytes())
runner=root/attempt/"runner.py"
if runner.exists():assert runner.read_bytes()==Path(__file__).read_bytes()
else:runner.write_bytes(Path(__file__).read_bytes())
variables={"CARGO_NET_OFFLINE":"true","CARGO_PROFILE_TEST_OPT_LEVEL":"1","CARGO_PROFILE_TEST_DEBUG_ASSERTIONS":"true","CARGO_PROFILE_TEST_OVERFLOW_CHECKS":"true","CARGO_INCREMENTAL":"0","RUST_TEST_THREADS":"1","MPK_GO_ORDINARY_ASSEMBLY_PROBE_OUTPUT":str((out/"certificates").resolve())}
env=os.environ.copy();env.update(variables)
for name in ["RUST_MIN_STACK","MPK_CORE_MAX_STEPS","MPK_CORE_MAX_SECONDS"]:env.pop(name,None)
target="go_practical_ordinary_assembly_probe"
if mode in {"tests","listing"}:
    command=["cargo","test","--offline","--locked","-p","mpk-vc","--test",target,"--","--ignored","--nocapture","--test-threads=1"]
    if mode=="listing":command.append("--list")
elif mode=="clippy":command=["cargo","clippy","--offline","--locked","-p","mpk-vc","--lib","--test",target,"--","-D","warnings"]
else:command=["rustfmt","--check","--edition","2021","--config","skip_children=true","crates/mpk-vc/tests/go_practical_ordinary_assembly_probe.rs","crates/mpk-vc/tests/support/go_ordinary_assembly_probe.rs"]
record={"status":"running","started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"command":command,"environment":variables,"sources":records,"source_snapshot":str(archive),"runner":seal(__file__),"selection_reason":"A new independent born-ignored assembly probe retains ALL4053 original full64 shift declarations and ALL43 new product declarations, with every original type/body/proof/level/generated-family identity. Exact4096decl boundary and actual complete-byte/zero-axiom both-kernel proof feasibility, not complete56-family/source/context/program authority. Every source node retained modulo exact structural sharing, no beta/delta/substitution/pruning/import/axiom/fuel/cap workaround. A genuinely valid alternate same-name theorem proof passes kernel but exact assembly rejects its identity; a false UnitZeroValid inside full4096closure must fail actual core typechecking. No original running Go source is edited; original23component/12249name library and all56/all169/T02-T11 remain required. Selected new probe/lint/format/listing only, no CSharp/agents/Actions/whole gate.","CSharp_tests_executed":False,"T02_W01_complete":False,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
p=out/"result.json";p.write_text(json.dumps(record,indent=2)+"\n");start=time.monotonic()
with(out/"stdout.log").open("wb")as stdout,(out/"stderr.log").open("wb")as stderr:code=subprocess.run(command,env=env,stdout=stdout,stderr=stderr,check=False).returncode
changed=[s for s in records if seal(s["path"])!=s]
record.update(status="passed" if code==0 and not changed else "failed",exit_code=code,changed_inputs_during_execution=changed,wall_seconds=time.monotonic()-start,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),stdout=seal(out/"stdout.log"),stderr=seal(out/"stderr.log"))
p.write_text(json.dumps(record,indent=2)+"\n");print(json.dumps({k:record[k]for k in ["status","exit_code","wall_seconds"]}));raise SystemExit(0 if record["status"]=="passed" else 1)
