#!/usr/bin/env python3
"""Replay limits/descriptor bindings and21 Python/Rust/Go complete hash preimages."""
import argparse
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from foundation_limits_protocol import ROOT,HERE,RECORD,RULES,OP_TABLE,package,schema_draft
from foundation_limits_vectors import model,cases,apply_row,hash_vectors,linked_contexts
from run_context_protocol import inputs as prior_inputs
from run_structural_limits import timed_input
from run_proof_pilot import DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from run_identity_dag import TARGET
from check_scope_report import strict_json,require,exact

SCOPE="three private closed limits/definition-recipe-inventory/descriptor roots;64 counters,56 logical operations,eight families,two joint core interfaces,21 complete cross-language hash preimages; not complete ordinary foundation/source/proof/activation"
NEW_SOURCES=("foundation_limits_protocol.py","foundation_limits_vectors.py","run_foundation_limits.py","test_foundation_limits.py","check_foundation_limits_metadata.py")
PENDING=schema_draft()["remaining"]


def documents():
    return [(name+".json",value) for name,value in package().items()]+[("schemas.json",schema_draft()),("vectors.json",cases()),("hash-vectors.json",hash_vectors()),("linked-contexts.json",linked_contexts())]


def inputs():
    rows={r["path"]:r["sha256"] for r in prior_inputs()}
    paths=[HERE/name for name in NEW_SOURCES]+[RULES,OP_TABLE]+[RECORD/name for name,_ in documents()]
    # Pin the actual evidence/core source closure named by the definition inventory.
    defs=package()["definitions"]
    paths += [ROOT/m["path"] for m in defs["evidence_receipts"]+sum([x["sources"] for x in defs["core_interfaces"]],[])]
    for path in paths:rows[path.relative_to(ROOT).as_posix()]=sha(path.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def observations():
    p=model();out=[]
    for row in cases():
        try:apply_row(row,p);accepted=True
        except (ValueError,TypeError):accepted=False
        require(accepted is row["expected_accepted"],"unexpected packaging outcome: "+row["id"])
        out.append({"id":row["id"],"kind":row["kind"],"result":"configuration_candidate" if accepted else "configuration_rejected"})
    p.validate_package(p.compiled)  # Also check registered filesystem paths, not only supplied bytes.
    return out


def expected_hashes():return [{"id":v["id"],"accepted":True,"canonical_hex":v["canonical_hex"],"digest":v["digest"]} for v in hash_vectors()]


def check_retained():
    report=strict_json((RECORD/"observations.json").read_bytes())
    require(report["scope"]==SCOPE and report["pending"]==PENDING and report["inputs"]==inputs(),"foundation packaging scope/source pins")
    for name,value in documents():require((RECORD/name).read_bytes()==json_bytes(value),"canonical packaging fixture: "+name)
    observed=observations();require(exact(report["cases"],observed) and report["outcomes"]==dict(Counter(x["result"] for x in observed)),"complete expected packaging cases")
    for language in ("rust","go"):
        raw=(RECORD/(language+"-hashes.json")).read_bytes()
        require(exact(strict_json(raw),expected_hashes()),"complete Python/Rust/Go hash preimages")
        require(len(report["native_runs"][language])==2 and all(r["exit_code"]==0 and r["stdout_sha256"]==sha(raw) for r in report["native_runs"][language]),"two exact hash observer runs")
    for key in ("complete_ordinary_foundation","source_proofs_checked","production_core_changed","profile_activated","migration_completed"):
        require(report[key] is False,"configuration/proof boundary: "+key)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);group=parser.add_mutually_exclusive_group(required=True);group.add_argument("--write",action="store_true");group.add_argument("--check",action="store_true");args=parser.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in documents():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/"host-toolchain.json").read_bytes()))
    build=subprocess.run(["cargo","build","--release","--offline","--locked","--manifest-path",str(HERE/"identity-hashes/Cargo.toml")],cwd=ROOT,env={**os.environ,"CARGO_TARGET_DIR":str(TARGET)},capture_output=True,timeout=300)
    require(build.returncode==0 and b"warning:" not in build.stderr,"retained Rust canonicalizer build: "+build.stderr.decode())
    rust=TARGET/"release/go-practical-identity-hashes";observed=observations();native={};runs={}
    with tempfile.TemporaryDirectory(prefix="mpk-foundation-hashes-",dir="/private/tmp") as temporary:
        go=Path(temporary)/"hashes"
        env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
        build=subprocess.run([str(DEFAULT_GOROOT/"bin/go"),"build","-trimpath","-buildvcs=false","-o",str(go),str(HERE/"identity_hashes.go")],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(build.returncode==0 and not build.stderr,"retained Go canonicalizer build: "+build.stderr.decode())
        binaries={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())};payload=json_bytes([{k:v[k] for k in ("id","domain","raw_json_hex")} for v in hash_vectors()])
        for language,exe in (("rust",rust),("go",go)):
            output=[timed_input([exe],payload,env) for _ in range(2)]
            require(output[0][0]==output[1][0] and exact(strict_json(output[0][0]),expected_hashes()),"three-observer foundation hashes: "+language)
            native[language]=output[0][0];runs[language]=[r for _,r in output]
        require(before==inputs() and binaries=={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())},"source/binary stability")
    report={"schema":"mpk.go_practical.foundation_limits_probe.v0","scope":SCOPE,"pending":PENDING,"inputs":before,"host_toolchain_inventory":inventory,"native_binaries":binaries,"native_runs":runs,"cases":observed,"outcomes":dict(Counter(x["result"] for x in observed)),"hash_vectors":len(hash_vectors()),"complete_ordinary_foundation":False,"source_proofs_checked":False,"production_core_changed":False,"profile_activated":False,"migration_completed":False,"direct_child_parallelism":1}
    if args.write:
        for language,raw in native.items():(RECORD/(language+"-hashes.json")).write_bytes(raw)
        (RECORD/"observations.json").write_bytes(json_bytes(report))
    old=check_retained();require(old["native_binaries"]==binaries and all((RECORD/(k+"-hashes.json")).read_bytes()==v for k,v in native.items()),"fresh native hash replay")
    print(f"PASS: three packaging roots,64 counters/56 logical operations/eight families/two core interfaces,{len(observed)} configuration vectors,21 Python/Rust/Go hash preimages; complete ordinary/source foundation remains due",flush=True)


if __name__=="__main__":main()
