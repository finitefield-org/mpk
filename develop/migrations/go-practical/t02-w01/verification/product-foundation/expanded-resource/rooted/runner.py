"""Born-ignored complete original resource diagnostic, selected lint/format/listing."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,time


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode,attempt=sys.argv[1:3];assert attempt.replace("-","").isalnum();assert mode in {"diagnostic","listing","clippy","format"}
out=root/attempt/mode;out.mkdir(exist_ok=False,parents=True)
producer=Path("crates/mpk-vc/src/go_practical_ordinary")
sources=[Path("crates/mpk-vc/src/go_practical_ordinary.rs"),*sorted(producer.rglob("*.rs")),Path("crates/mpk-vc/tests/go_practical_product_expanded_resource_diagnostic.rs"),Path("crates/mpk-vc/tests/support/go_product_expanded_resource.rs"),Path("crates/mpk-vc/tests/support/go_product_alias_codec.rs")]
records=[seal(p) for p in sources];snapshot=root/attempt/"inputs"
for source in records:
    target=snapshot/source["path"]
    if target.exists():
        raw=target.read_bytes();assert (hashlib.sha256(raw).hexdigest(),len(raw))==(source["sha256"],source["bytes"])
    else:target.parent.mkdir(exist_ok=True,parents=True);target.write_bytes(Path(source["path"]).read_bytes())
runner_copy=root/attempt/"runner.py"
if runner_copy.exists():assert runner_copy.read_bytes()==Path(__file__).read_bytes()
else:runner_copy.write_bytes(Path(__file__).read_bytes())
variables={"CARGO_NET_OFFLINE":"true","CARGO_PROFILE_TEST_OPT_LEVEL":"1","CARGO_PROFILE_TEST_DEBUG_ASSERTIONS":"true","CARGO_PROFILE_TEST_OVERFLOW_CHECKS":"true","CARGO_INCREMENTAL":"0","RUST_TEST_THREADS":"1","MPK_GO_PRODUCT_RESOURCE_DIAGNOSTIC_OUTPUT":str((out/"certificates").resolve())}
env=os.environ.copy();env.update(variables)
for name in ["RUST_MIN_STACK","MPK_CORE_MAX_STEPS","MPK_CORE_MAX_SECONDS"]:env.pop(name,None)
target="go_practical_product_expanded_resource_diagnostic"
if mode in {"diagnostic","listing"}:
    command=["cargo","test","--offline","--locked","-p","mpk-vc","--test",target,"--","--ignored","--nocapture","--test-threads=1"]
    if mode=="listing":command.append("--list")
elif mode=="clippy":command=["cargo","clippy","--offline","--locked","-p","mpk-vc","--test",target,"--","-D","warnings"]
else:command=["rustfmt","--check","--edition","2021","--config","skip_children=true",*[str(p) for p in sources if p.name in {"go_practical_product_expanded_resource_diagnostic.rs","go_product_expanded_resource.rs"}]]
record={"status":"running","started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"command":command,"environment":variables,"sources":records,"source_snapshot":str(snapshot),"runner":seal(__file__),"selection_reason":"Actual expanded Rust consumer stopped at unchanged term cap after arity59. Born-ignored new independent diagnostic copy keeps its complete original algorithm and all65 arities/93600 leaf obligations, removes only the before-encoding assertion to retain full over-cap candidates and sends them to unchanged production verifiers for actual resource classification. All60 completed original in-cap inputs must match entire original checked bytes and are not reverified. No captured running Rust/Go checker source is changed. No cap/fuel/axiom/import change or semantic/source/context/application proof promotion. CSharp/full gate not run."}
p=out/"result.json";p.write_text(json.dumps(record,indent=2)+"\n");start=time.monotonic()
with (out/"stdout.log").open("wb") as stdout,(out/"stderr.log").open("wb") as stderr:
    code=subprocess.run(command,env=env,stdout=stdout,stderr=stderr,check=False).returncode
changed=[s for s in records if seal(s["path"])!=s]
record.update(status="passed" if code==0 and not changed else "failed",exit_code=code,changed_inputs_during_execution=changed,wall_seconds=time.monotonic()-start,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),stdout=seal(out/"stdout.log"),stderr=seal(out/"stderr.log"))
p.write_text(json.dumps(record,indent=2)+"\n");print(json.dumps({k:record[k] for k in ["status","exit_code","wall_seconds"]}));raise SystemExit(0 if record["status"]=="passed" else 1)
