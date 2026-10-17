#!/usr/bin/env python3
"""Replay56 closed operation signatures,224 typed vectors and native hash observers."""
import argparse
import copy
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from operation_signature_vectors import model, definitions, vectors, mutate, catalogue
from operation_signatures import ROOT, HERE, RECORD, DOMAIN, DEFAULT_GOROOT, canonical, sha
from run_proof_pilot import json_bytes, verify_host_toolchain
from run_identity_dag import TARGET
from run_structural_limits import timed_input
from check_scope_report import strict_json, require, exact

NEW_SOURCES=("operation_signatures.py","operation_signature_vectors.py","test_operation_signatures.py",
             "run_operation_signatures.py","check_operation_signature_metadata.py")
PRIOR=RECORD.parent/"contract-wire/observations.json"
SCOPE="concrete type/control-region/check/binding-requirement signatures for all56 families; not actual VIR,independent source import,operation/definedness/value/effect/frame proofs or activation"


def corpus():
    m=model();ds=definitions(m);by_op={d["op"]:d for d in ds};rows=vectors()
    candidates=[copy.deepcopy(by_op[r["op"]]) if r["mutation"]=="none" else mutate(by_op[r["op"]],r["mutation"]) for r in rows]
    observed=m.classify(candidates)
    outcomes=[{"id":r["id"],"classification":"typed_unproved" if ok else "configuration_rejection"} for r,ok in zip(rows,observed)]
    require(all(o["classification"]==r["expected"] for o,r in zip(outcomes,rows)),"all224 signature vector classifications")
    return {"catalog.json":catalogue(m),"definitions.json":ds,"vectors.json":rows,"outcomes.json":outcomes}


def inputs():
    prior=strict_json(PRIOR.read_bytes());rows={r["path"]:r["sha256"] for r in prior["inputs"]}
    # Retain the preceding receipt as well as every actual source/data pin.
    for path in [PRIOR]+[HERE/n for n in NEW_SOURCES]+[RECORD/n for n in ("catalog.json","definitions.json","vectors.json","outcomes.json")]:
        rows[path.relative_to(ROOT).as_posix()]=sha(path.read_bytes())
    for path,expected in rows.items():require(sha((ROOT/path).read_bytes())==expected,"retained/current source pin: "+path)
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def hash_vectors(docs):
    return [{"id":d["op"],"domain":DOMAIN,"raw_json_hex":canonical({k:v for k,v in d.items() if k!="operation_hash"}).hex(),
             "digest":d["operation_hash"]} for d in docs["definitions.json"]]


def expected_hashes(docs):return [{"id":r["id"],"accepted":True,"canonical_hex":r["raw_json_hex"],"digest":r["digest"]} for r in hash_vectors(docs)]


def check_retained(docs=None):
    docs=corpus() if docs is None else docs;r=strict_json((RECORD/"observations.json").read_bytes())
    require(r["scope"]==SCOPE and r["inputs"]==inputs(),"complete operation scope/source pins")
    for name,value in docs.items():require((RECORD/name).read_bytes()==json_bytes(value),"complete operation catalogue/definition/vector/outcome data")
    require(r["outcomes"]==dict(Counter(x["classification"] for x in docs["outcomes.json"])) and r["hash_vectors"]==56,"complete operation counts")
    for language in ("rust","go"):
        raw=(RECORD/(language+"-hashes.json")).read_bytes()
        require(exact(strict_json(raw),expected_hashes(docs)),"operation canonical bytes and hash")
        require(len(r["native_runs"][language])==2 and all(x["exit_code"]==0 and x["stdout_sha256"]==sha(raw) for x in r["native_runs"][language]),"two native operation hash observations")
    for field in ("actual_vir_emitted","independent_source_import","operation_proofs_checked","definedness_proofs_checked","profile_activated"):
        require(r[field] is False,"operation evidence boundary")
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument("--write",action="store_true");g.add_argument("--check",action="store_true");args=p.parse_args()
    docs=corpus()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in docs.items():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/"host-toolchain.json").read_bytes()))
    build=subprocess.run(["cargo","build","--release","--offline","--locked","--manifest-path",str(HERE/"identity-hashes/Cargo.toml")],cwd=ROOT,env={**os.environ,"CARGO_TARGET_DIR":str(TARGET)},capture_output=True,timeout=300)
    require(build.returncode==0 and b"warning:" not in build.stderr,"unchanged Rust observer build")
    rust=TARGET/"release/go-practical-identity-hashes";runs={};native={};payload=json_bytes([{k:r[k] for k in ("id","domain","raw_json_hex")} for r in hash_vectors(docs)])
    with tempfile.TemporaryDirectory(prefix="mpk-operation-signatures-",dir="/private/tmp") as temporary:
        go=Path(temporary)/"hashes";env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
        build=subprocess.run([str(DEFAULT_GOROOT/"bin/go"),"build","-trimpath","-buildvcs=false","-o",str(go),str(HERE/"identity_hashes.go")],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(build.returncode==0 and not build.stderr,"unchanged Go observer build")
        binaries={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())}
        for language,exe in (("rust",rust),("go",go)):
            output=[timed_input([exe],payload,env) for _ in range(2)]
            require(output[0][0]==output[1][0] and exact(strict_json(output[0][0]),expected_hashes(docs)),"complete three-observer operation hash agreement")
            native[language]=output[0][0];runs[language]=[x for _,x in output]
        require(before==inputs() and binaries=={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())},"operation source/binary stability")
    report={"schema":"mpk.go_practical.operation_signature_probe.v0","scope":SCOPE,"pending":docs["catalog.json"]["remaining"],"inputs":before,
            "host_toolchain_inventory":inventory,"native_binaries":binaries,"native_runs":runs,"hash_vectors":56,
            "outcomes":dict(Counter(r["classification"] for r in docs["outcomes.json"])),"direct_child_parallelism":1,
            "actual_vir_emitted":False,"independent_source_import":False,"operation_proofs_checked":False,"definedness_proofs_checked":False,"profile_activated":False}
    if args.write:
        for language,raw in native.items():(RECORD/(language+"-hashes.json")).write_bytes(raw)
        (RECORD/"observations.json").write_bytes(json_bytes(report))
    old=check_retained(docs);require(old["native_binaries"]==binaries,"reproducible observer binaries")
    print("PASS:56 concrete operation signatures,224 vectors (56 typed unproved/168 rejected),56 Python/Rust/Go hashes; no VIR/source/operation proof or activation",flush=True)


if __name__=="__main__":main()
