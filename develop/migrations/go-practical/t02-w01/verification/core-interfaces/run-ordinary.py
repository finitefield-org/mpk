"""Check fresh actual ordinary core foundation against both current frozen-interface checkers; no CSharp test target."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,time
repo=Path.cwd();root=Path(__file__).resolve().parent.relative_to(repo.resolve());mode=sys.argv[1]
attempt=sys.argv[2] if len(sys.argv)>2 else "initial"
assert attempt in ["initial","final","reviewed"]
directory=root/attempt/mode;directory.mkdir(parents=True,exist_ok=False)
def seal(p):
 p=Path(p);b=p.read_bytes();return {"path":str(p),"sha256":hashlib.sha256(b).hexdigest(),"bytes":len(b)}
sources=["crates/mpk-vc/src/go_practical_ordinary.rs","crates/mpk-vc/src/lib.rs","crates/mpk-vc/tests/go_practical_core_foundation.rs","proofs/std/bool/std-bool.hex","proofs/std/eq/std-eq.hex","proofs/std/nat/std-nat.hex"]
record={"status":"running","started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"selection_reason":"Actual fresh ordinary core foundation includes the complete existing Boolean basis, exact standard Nat, the two frozen additive interfaces, arbitrary-Nat Identity/IdentityLaw and conditional Bool FalseElim. Check exact importer, retained Boolean hash, fifteen fully re-encoded malformed/false candidates and unused-term acceptance. Current both-checker evidence is required; no CSharp test target, intermediate full gate, complete foundation descriptor/context or source/application proof claim.","sources":[seal(p) for p in sources],"full_T02_gate":"deferred to T02-W04: check-fast.sh --skip-csharp","T02_W01_complete":False}
env=os.environ.copy();cwd=repo
goroot=Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")
binary=Path("/private/tmp/mpk-go-practical-t02-core-interfaces/checker-ref")
if mode in ["rust-tests","clippy"]:
 overrides={"CARGO_NET_OFFLINE":"true","CARGO_PROFILE_TEST_OPT_LEVEL":"1","CARGO_PROFILE_TEST_DEBUG_ASSERTIONS":"true","CARGO_PROFILE_TEST_OVERFLOW_CHECKS":"true","CARGO_INCREMENTAL":"0","RUST_TEST_THREADS":"1"}
 env.update(overrides)
 for key in ["RUST_MIN_STACK","MPK_CORE_MAX_STEPS","MPK_CORE_MAX_SECONDS"]:env.pop(key,None)
 command=["cargo","test","--offline","--locked","-p","mpk-vc","--test","go_practical_core_foundation","--","--nocapture","--test-threads=1"] if mode=="rust-tests" else ["cargo","clippy","--offline","--locked","-p","mpk-vc","--lib","--test","go_practical_core_foundation","--","-D","warnings"]
 if mode=="rust-tests":
  overrides["MPK_GO_CORE_FOUNDATION_TEST_OUTPUT"]=str((directory/"certificates").resolve());env.update(overrides)
 record["environment"]=overrides
elif mode=="go-build":
 binary.parent.mkdir(parents=True,exist_ok=True)
 overrides={"GOROOT":str(goroot),"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOSUMDB":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-t02-build-cache"};env.update(overrides);record["environment"]=overrides
 command=[str(goroot/"bin/go"),"build","-trimpath","-buildvcs=false","-o",str(binary),"./cmd/mpk-checker-ref"];cwd=repo/"go-tools/mpk-checker-ref"
 record["compiler_version"]=subprocess.check_output([str(goroot/"bin/go"),"version"],env=env,text=True).strip();assert record["compiler_version"]=="go version go1.25.0 darwin/arm64"
 record["compiler"]=seal(goroot/"bin/go")
elif mode=="format":command=["rustfmt","--check","--edition","2021","--config","skip_children=true",sources[0],sources[2]]
else:raise ValueError(mode)
if attempt=="final":record["selection_reason"]="Strengthen the actual ordinary identity definition to compute through unchanged legacy Nat.rec, and prove its universal identity with the new Nat.induct. Rerun all four affected generator/importer/negative/unused-node integration roots and their18 certificate outputs, strict Clippy and format. Other kernel/Go primitive code and passing unit matrices are unchanged and retain prior evidence. No CSharp tests, intermediate full gate or complete source/program/foundation/context claim."
record.update(command=command,cwd=str(cwd.relative_to(repo)) or ".")
result=directory/"result.json";result.write_text(json.dumps(record,indent=2)+"\n");started=time.monotonic()
with (directory/"stdout.log").open("wb") as out,(directory/"stderr.log").open("wb") as err:
 code=subprocess.run(command,cwd=cwd,env=env,stdout=out,stderr=err,check=False).returncode
record.update(status="passed" if code==0 else "failed",exit_code=code,wall_seconds=time.monotonic()-started,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),stdout=seal(directory/"stdout.log"),stderr=seal(directory/"stderr.log"))
if mode=="go-build" and code==0:record["checker_binary"]=seal(binary)
if mode=="rust-tests" and code==0:
 text=(directory/"stdout.log").read_text();assert "4 passed; 0 failed" in text and "0 filtered out" in text
 record["tests_passed"]=4;record["certificates"]=[seal(p) for p in sorted((directory/"certificates").glob("*.mpcert"))];assert len(record["certificates"])==18
result.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:record[k] for k in ["status","exit_code","wall_seconds"]}),flush=True)
raise SystemExit(0 if code==0 else 1)
