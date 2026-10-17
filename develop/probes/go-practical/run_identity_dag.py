#!/usr/bin/env python3
"""Replay source identity DAG and three untrusted JSON/hash observers."""
import argparse
import os
import subprocess
import tempfile
from pathlib import Path
from identity_catalog import generate,RECORD,CSHARP_VECTOR
from identity_vectors import fixture,hash_vectors
from identity_migration import proposal
from identity_wire import validate_graph
from run_structural_limits import inputs as predecessor_inputs,timed_input
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from check_scope_report import strict_json,require,exact

TARGET=Path("/private/tmp/mpk-go-practical-identity-target")
SCOPE="private101 candidate identities/41 domains,10 closed source-identity records/26-node DAG,44 Python/Rust/Go JSON/hash vectors and143 per-file candidate migration routes; no source/import/proof/schema activation"
NEW_SOURCES=("identity_catalog.py","identity_wire.py","identity_vectors.py","identity_migration.py","identity_hashes.go","test_identity_wire.py","run_identity_dag.py","check_identity_metadata.py")
PENDING=generate()["pending"]


def inputs():
    rows={r["path"]:r["sha256"] for r in predecessor_inputs()}
    paths=[HERE/n for n in NEW_SOURCES]+[p for p in (HERE/"identity-hashes").rglob("*") if p.is_file()]
    paths += [CSHARP_VECTOR,ROOT/"crates/mpk-vc/src/canonical_json.rs",ROOT/"develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md",ROOT/"develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md"]
    paths += [RECORD/n for n in ("catalog.json","fixture.json","vectors.json","migration-proposal.json")]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def expected_hashes():
    return [{"id":r["id"],"accepted":r["expected_accepted"],**({"canonical_hex":r["canonical_hex"],"digest":r["digest"]} if r["expected_accepted"] else {})} for r in hash_vectors()]


def check_retained():
    r=strict_json((RECORD/"observations.json").read_bytes())
    require(r["scope"]==SCOPE and r["pending"]==PENDING and r["inputs"]==inputs(),"identity scope/source pins")
    for name,content in (("catalog.json",generate()),("fixture.json",fixture()),("vectors.json",hash_vectors()),("migration-proposal.json",proposal())):
        require((RECORD/name).read_bytes()==json_bytes(content),"canonical private identity inputs: "+name)
    f=fixture();require(exact(r["identity_graph"],validate_graph(f["nodes"],f["roots"])),"complete typed identity graph")
    require(r["production_source_import"] is False and r["source_proofs_checked"] is False and r["migration_completed"] is False and r["profile_activated"] is False,"identity evidence is not source/profile completion")
    for language in ("rust","go"):
        raw=(RECORD/(language+"-hashes.json")).read_bytes()
        require(exact(strict_json(raw),expected_hashes()),"three observer canonical/digest/verdict agreement")
        require(len(r["native_runs"][language])==2 and all(x["exit_code"]==0 and x["stdout_sha256"]==sha(raw) for x in r["native_runs"][language]),"two exact native outputs")
    require(r["outcomes"]=={"identity_nodes":26,"hash_vectors":44,"canonical_accepted":34,"canonical_rejected":10,"candidate_migration_paths":143},"exact identity/vector counts")
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument("--write",action="store_true");g.add_argument("--check",action="store_true");args=p.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,content in (("catalog.json",generate()),("fixture.json",fixture()),("vectors.json",hash_vectors()),("migration-proposal.json",proposal())):(RECORD/name).write_bytes(json_bytes(content))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/"host-toolchain.json").read_bytes()))
    build=subprocess.run(["cargo","build","--release","--offline","--locked","--manifest-path",str(HERE/"identity-hashes/Cargo.toml")],cwd=ROOT,env={**os.environ,"CARGO_TARGET_DIR":str(TARGET)},capture_output=True,timeout=300)
    require(build.returncode==0 and b"warning:" not in build.stderr,"Rust hash observer build: "+build.stderr.decode())
    rust=TARGET/"release/go-practical-identity-hashes"
    with tempfile.TemporaryDirectory(prefix="mpk-identity-",dir="/private/tmp") as temporary:
        go=Path(temporary)/"hashes"
        env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
        build=subprocess.run([str(DEFAULT_GOROOT/"bin/go"),"build","-trimpath","-buildvcs=false","-o",str(go),str(HERE/"identity_hashes.go")],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(build.returncode==0 and not build.stderr,"Go hash observer build: "+build.stderr.decode())
        binaries={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())}
        payload=json_bytes([{k:r[k] for k in ("id","domain","raw_json_hex")} for r in hash_vectors()]);native={};runs={}
        for language,exe in (("rust",rust),("go",go)):
            observed=[timed_input([exe],payload,env) for _ in range(2)]
            require(observed[0][0]==observed[1][0] and exact(strict_json(observed[0][0]),expected_hashes()),"independent canonical/hash observer: "+language)
            native[language]=observed[0][0];runs[language]=[r for _,r in observed]
        require(before==inputs() and binaries=={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())},"identity source/binaries stable")
        f=fixture();report={"schema":"mpk.go_practical.identity_dag_probe.v0","scope":SCOPE,"pending":PENDING,"inputs":before,"host_toolchain_inventory":inventory,"native_binaries":binaries,"native_runs":runs,"identity_graph":validate_graph(f["nodes"],f["roots"]),"outcomes":{"identity_nodes":26,"hash_vectors":44,"canonical_accepted":34,"canonical_rejected":10,"candidate_migration_paths":143},"production_source_import":False,"source_proofs_checked":False,"migration_completed":False,"profile_activated":False,"direct_child_parallelism":1}
        if args.write:
            for language,raw in native.items():(RECORD/(language+"-hashes.json")).write_bytes(raw)
            (RECORD/"observations.json").write_bytes(json_bytes(report))
        old=check_retained();require(old["native_binaries"]==binaries and all((RECORD/(k+"-hashes.json")).read_bytes()==v for k,v in native.items()),"fresh identity/hash replay")
        print("PASS: 101 unreserved identity candidates/41 domains,10 closed record schemas,26-node DAG,44 cross-language hash vectors (34 valid/10 rejected),143 migration proposals; no source/schema/profile completion",flush=True)


if __name__=="__main__":main()
