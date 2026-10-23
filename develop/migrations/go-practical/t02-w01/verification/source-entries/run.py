"""Complete original inputs and actual affected entry/prologue checks."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
BASIS="c2bc547a414becc2d26d920cf8fd89076e3c20f4"
SDK=Path("/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go")
CHANGED=["crates/mpk-vc/src/go_practical_ordinary.rs","crates/mpk-vc/src/go_practical_ordinary/source_entry.rs",
         "crates/mpk-vc/src/go_practical_type_keys.rs","crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs",
         "crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/entry_layouts.rs",
         "crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/execution.rs",
         "crates/mpk-vc/tests/go_practical_source_entries.rs","crates/mpk-vc/tests/support/go_source_entry_inputs.rs"]


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"bytes":len(raw),"sha256":hashlib.sha256(raw).hexdigest(),"mode":path.stat().st_mode&0o777}


def main():
    epoch,mode=sys.argv[1:]
    assert epoch.replace("-","").isalnum()
    out=ROOT/epoch;out.mkdir(exist_ok=False)
    files={REPO/"Cargo.toml",REPO/"Cargo.lock",Path(__file__).resolve()}
    files.update(ROOT.glob("*.py"));files.update(p for p in (ROOT/"project").rglob("*") if p.is_file())
    for name in ["crates","proofs/std","fixtures","develop/specs/vectors","release/bundles","go-tools/mpk-checker-ref"]:
        files.update(p for p in (REPO/name).rglob("*") if p.is_file())
    for name in [".cargo","rust-toolchain","rust-toolchain.toml"]:
        p=REPO/name
        if p.is_dir():files.update(q for q in p.rglob("*") if q.is_file())
        elif p.is_file():files.add(p)
    for name in ["source-inputs/corpus.json","source-invocations/corpus.json","source-execution/corpus.json","source-execution/vectors.json","source-execution/native-values.json","source-execution/program.go"]:
        files.add(ROOT.parent/name)
    if (ROOT/"native/observations.json").exists():files.add(ROOT/"native/observations.json")
    if mode in ["pair","inspect"]:
        files.update(p for directory in ROOT.glob("rust*/certificates") for p in directory.rglob("*") if p.is_file())
        files.add(ROOT.parent/"execution-histories/reference-build/mpk-checker-ref")
        files.update([ROOT.parent/"source-invocations/structural-oracle.py",ROOT.parent/"source-error-layouts/structural-oracle.py",REPO/"develop/probes/go-practical/carrier_terms.py"])
        files.update((ROOT.parent/"source-invocations/generation-linked/certificates").glob("*.mpcert"))
        files.update([ROOT/"row-review.json",ROOT/"row-inspection/result.json"])
    sources=[seal(p) for p in sorted(files)]
    with tarfile.open(out/"inputs.tar.gz","x:gz",dereference=True) as archive:
        for p in sorted(files):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
    receipt=out/"inputs.json";receipt.write_text(json.dumps({"basis_commit":BASIS,"sources":sources,"complete_archive":seal(out/"inputs.tar.gz")},indent=2)+"\n")
    cargo=["cargo","test","--offline","--locked","-p","mpk-vc"]
    commands={
        "format-write":["rustfmt","--edition=2021","--config","skip_children=true",*CHANGED],
        "format":["rustfmt","--edition=2021","--check","--config","skip_children=true",*CHANGED],
        "go-format-write":[str(SDK/"bin/gofmt"),"-w",str(ROOT/"project/main.go")],
        "go-format":[str(SDK/"bin/gofmt"),"-l",str(ROOT/"project/main.go")],
        "rust":[*cargo,"--test","go_practical_source_entries","--","--nocapture","--test-threads=1"],
        "build":[*cargo,"--test","go_practical_source_entries","--no-run"],
        "matrix":[*cargo,"--test","go_practical_source_execution","--","--nocapture","--test-threads=1"],
        "clippy":["cargo","clippy","--offline","--locked","-p","mpk-vc","--lib","--test","go_practical_source_entries","--test","go_practical_source_execution","--","-D","warnings"],
        "native":[str(SDK/"bin/go"),"build","-trimpath","-o",str(out/"original-app"),"."],
        "pair":[sys.executable,"-B",str(ROOT/"check-pair.py")],
        "inspect":[sys.executable,"-B",str(ROOT/"inspect-parts.py")],
    }
    changes={"CARGO_NET_OFFLINE":"true","CARGO_PROFILE_TEST_OPT_LEVEL":"1","CARGO_PROFILE_TEST_DEBUG_ASSERTIONS":"true","CARGO_PROFILE_TEST_OVERFLOW_CHECKS":"true","CARGO_INCREMENTAL":"0","RUST_TEST_THREADS":"1"}
    if mode=="rust":changes["MPK_GO_SOURCE_ENTRY_OUTPUT"]=str(out/"certificates")
    if mode=="matrix":changes["MPK_GO_SOURCE_EXECUTION_OUTPUT"]=str(out/"observations")
    if mode=="native" or mode.startswith("go-"):
        changes.update({"GOROOT":str(SDK),"GOTOOLCHAIN":"local","GOENV":"off","GOWORK":"off","GOFLAGS":"-mod=readonly","GOPROXY":"off","GOSUMDB":"off","CGO_ENABLED":"0","GOMAXPROCS":"1","GOCACHE":"/private/tmp/mpk-go-source-entry-178-cache"})
    env=os.environ.copy();env.update(changes)
    for key in ["RUST_MIN_STACK","MPK_CORE_MAX_STEPS","MPK_CORE_MAX_SECONDS"]:env.pop(key,None)
    result={"status":"running","command":commands[mode],"environment":changes,"inputs":seal(receipt),"driver":seal(__file__),"started_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "tools":{name:seal(shutil.which(name)) for name in ["cargo","rustc","rustfmt"]},
            "selection_reason":"Source-owned complete entry layouts now drive the actual original-source interpreter prologue and ordinary initialization of all real entry variables. Complete original zero/70-case units retain65 arguments,16 results and every selected/transitive body and inactive local. A separate entire original Go project covers blank/unnamed arguments/results, zero aggregates/error and lexical shadowing; unselected channel/callback/map/main bodies remain. Recheck direct 127-case source execution and its boundary consumer; unchanged broad kernel and unrelated frontend suites are not repeated.",
            "full_gate":"Deferred to final T02-W04: ./scripts/check-fast.sh"}
    if mode=="native" or mode.startswith("go-"):
        result["go_tool"]=seal(SDK/"bin/go")
        result["go_formatter"]=seal(SDK/"bin/gofmt")
    path=out/"result.json";path.write_text(json.dumps(result,indent=2)+"\n")
    start=time.monotonic()
    with (out/"stdout.log").open("xb") as stdout,(out/"stderr.log").open("xb") as stderr:
        code=subprocess.run(commands[mode],cwd=ROOT/"project" if mode=="native" else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
    result.update(exit_code=code,stdout=seal(out/"stdout.log"),stderr=seal(out/"stderr.log"))
    if mode=="native" and code==0:
        command=[str(out/"original-app")];result["native_command"]=command;result["native_binary"]=seal(out/"original-app");path.write_text(json.dumps(result,indent=2)+"\n")
        with (out/"native.stdout.log").open("xb") as stdout,(out/"native.stderr.log").open("xb") as stderr:
            run=subprocess.run(command,cwd=ROOT/"project",env=env,stdout=stdout,stderr=stderr,check=False)
        result.update(native_exit_code=run.returncode,native_stdout=seal(out/"native.stdout.log"),native_stderr=seal(out/"native.stderr.log"))
        assert run.returncode==0
        values={}
        for line in (out/"native.stderr.log").read_text().splitlines():
            name,*tokens=line.split();values[name]=[True if t=="true" else False if t=="false" else int(t) for t in tokens]
        assert values=={"named":[41,False],"zero":[8,0,0,True],"hidden":[7,True],"shadow":[6]}
        (out/"observations.json").write_text(json.dumps({"status":"passed","values":values,"project":[seal(ROOT/"project/main.go"),seal(ROOT/"project/go.mod")],"whole_original_project_executed":True,"Linux_release_claim":False},indent=2)+"\n")
    unchanged=sources==[seal(p) for p in sorted(files)]
    formatting=mode in ["format-write","go-format-write"]
    result.update(status="passed" if code==0 and (unchanged or formatting) else "failed",seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,expected_formatter_mutation=formatting,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps({k:result[k] for k in ["status","exit_code","seconds","consumed_inputs_unchanged"]}))
    raise SystemExit(result["status"]!="passed")


if __name__=="__main__":
    try:main()
    except BaseException as error:
        if len(sys.argv)==3:
            path=ROOT/sys.argv[1]/"result.json"
            if path.exists():
                result=json.loads(path.read_bytes())
                if result["status"]=="running":
                    result.update(status="failed",driver_exception=repr(error),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
                    path.write_text(json.dumps(result,indent=2)+"\n")
        raise
