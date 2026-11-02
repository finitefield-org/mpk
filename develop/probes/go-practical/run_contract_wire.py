#!/usr/bin/env python3
"""Replay24 source contracts,70 typed expression components and three hash observers."""
import argparse
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from contract_scope import ROOT,HERE,RECORD,Scope,SCOPE_DOMAIN,BINDING_DOMAIN,validate_scope
from contract_wire import normalize_contract,validate_contract,context,WireExpressions
from contract_wire_vectors import source_cases,typed_witnesses
from contract_catalog import generate
from overlay_contracts import ProbeResourceLimit
from identity_wire import canonical,digest
from identity_catalog import GO
from run_closed_carriers import inputs as prior_inputs
from run_identity_dag import TARGET
from run_structural_limits import timed_input
from run_proof_pilot import DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from check_scope_report import strict_json,require,exact

SCOPE="closed source/scope/normalized Go contract roots,70 typed expression heads with nominal ids/de Bruijn binders/full snapshot namespaces,24 source candidates; no independent Go import,source/definedness proof or activation"
NEW_SOURCES=("contract_scope.py","contract_wire.py","contract_catalog.py","contract_wire_vectors.py","run_contract_wire.py","test_contract_wire.py","check_contract_wire_metadata.py")
PENDING=generate()["remaining"]


def observed():
    outcomes=[];accepted=[]
    for row in source_cases():
        try:
            s=Scope(row["raw"],row["universe"],"contracts/"+row["id"]+".json");c=normalize_contract(s);validate_scope(s,s.value);validate_contract(s,c)
            result="normalized_pending_variant" if c["pending_candidates"] else "normalized_unproved"
            accepted.append({"id":row["id"],"raw_bytes_hex":s.raw_bytes.hex(),"scope":s.value,"contract":c})
        except ProbeResourceLimit:raise
        except ValueError:result="configuration_rejection"
        require(result==row["expected"],"unexpected complete contract wire outcome: "+row["id"])
        outcomes.append({"id":row["id"],"classification":result})
    s,rows=typed_witnesses();return outcomes,accepted,{"scope":s.value,"raw_bytes_hex":s.raw_bytes.hex(),"witnesses":rows}


def documents():
    outcomes,accepted,witnesses=observed()
    return [("catalog.json",generate()),("source-vectors.json",source_cases()),("normalized-contracts.json",accepted),("expression-witnesses.json",witnesses),("outcomes.json",outcomes)]


def hash_vectors():
    _,accepted,witnesses=observed();rows=[]
    def add(name,domain,value,omit=None):
        body={k:v for k,v in value.items() if k!=omit};raw=canonical(body)
        rows.append({"id":name,"domain":domain,"raw_json_hex":raw.hex(),"canonical_hex":raw.hex(),"digest":digest(domain,body),"expected_accepted":True})
    for record in accepted:
        add(record["id"]+"-scope",SCOPE_DOMAIN,record["scope"],"content_sha256")
        add(record["id"]+"-contract",GO["normalized_contract"][1],record["contract"],"contract_hash")
    for n,row in enumerate(witnesses["scope"]["bindings"]):add("binding-"+str(n),BINDING_DOMAIN,row,"id")
    for row in witnesses["witnesses"]:add("component-"+row["op"],"MPK-GO-IDENTITY-PILOT-1.0",row["wire"])
    return rows


def inputs():
    rows={r["path"]:r["sha256"] for r in prior_inputs()}
    for path in [HERE/n for n in NEW_SOURCES]+[RECORD/name for name,_ in documents()]:rows[path.relative_to(ROOT).as_posix()]=sha(path.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def expected_hashes():return [{"id":r["id"],"accepted":True,"canonical_hex":r["canonical_hex"],"digest":r["digest"]} for r in hash_vectors()]


def check_retained():
    r=strict_json((RECORD/"observations.json").read_bytes());require(r["scope"]==SCOPE and r["pending"]==PENDING and r["inputs"]==inputs(),"contract wire scope/source pins")
    for name,value in documents():require((RECORD/name).read_bytes()==json_bytes(value),"complete canonical contract data: "+name)
    outcomes,_,witnesses=observed();require(r["outcomes"]==dict(Counter(x["classification"] for x in outcomes)) and len(witnesses["witnesses"])==70,"typed contract outcome coverage")
    for lang in ("rust","go"):
        raw=(RECORD/(lang+"-hashes.json")).read_bytes();require(exact(strict_json(raw),expected_hashes()),"three canonical/preimage/digest observers")
        require(len(r["native_runs"][lang])==2 and all(x["exit_code"]==0 and x["stdout_sha256"]==sha(raw) for x in r["native_runs"][lang]),"two exact native hash outputs")
    for field in ("independent_source_import","source_proofs_checked","definedness_proofs_checked","profile_activated"):require(r[field] is False,"typed wire evidence boundary: "+field)
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument("--write",action="store_true");g.add_argument("--check",action="store_true");args=p.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in documents():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/"host-toolchain.json").read_bytes()))
    build=subprocess.run(["cargo","build","--release","--offline","--locked","--manifest-path",str(HERE/"identity-hashes/Cargo.toml")],cwd=ROOT,env={**os.environ,"CARGO_TARGET_DIR":str(TARGET)},capture_output=True,timeout=300)
    require(build.returncode==0 and b"warning:" not in build.stderr,"retained Rust hash observer build: "+build.stderr.decode())
    rust=TARGET/"release/go-practical-identity-hashes";native={};runs={};vectors=hash_vectors()
    with tempfile.TemporaryDirectory(prefix="mpk-contract-wire-",dir="/private/tmp") as temporary:
        go=Path(temporary)/"hashes";env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
        build=subprocess.run([str(DEFAULT_GOROOT/"bin/go"),"build","-trimpath","-buildvcs=false","-o",str(go),str(HERE/"identity_hashes.go")],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(build.returncode==0 and not build.stderr,"retained Go hash observer build: "+build.stderr.decode())
        binaries={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())};payload=json_bytes([{k:r[k] for k in ("id","domain","raw_json_hex")} for r in vectors])
        for lang,exe in (("rust",rust),("go",go)):
            output=[timed_input([exe],payload,env) for _ in range(2)];require(output[0][0]==output[1][0] and exact(strict_json(output[0][0]),expected_hashes()),"fresh complete contract canonical/hash agreement: "+lang)
            native[lang]=output[0][0];runs[lang]=[x for _,x in output]
        require(before==inputs() and binaries=={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())},"contract source/binary stability")
    outcomes,_,_=observed();report={"schema":"mpk.go_practical.contract_wire_probe.v0","scope":SCOPE,"pending":PENDING,"inputs":before,"host_toolchain_inventory":inventory,"native_binaries":binaries,"native_runs":runs,"hash_vectors":len(vectors),"outcomes":dict(Counter(r["classification"] for r in outcomes)),"independent_source_import":False,"source_proofs_checked":False,"definedness_proofs_checked":False,"profile_activated":False,"direct_child_parallelism":1}
    if args.write:
        for lang,raw in native.items():(RECORD/(lang+"-hashes.json")).write_bytes(raw)
        (RECORD/"observations.json").write_bytes(json_bytes(report))
    old=check_retained();require(old["native_binaries"]==binaries and all((RECORD/(k+"-hashes.json")).read_bytes()==v for k,v in native.items()),"fresh contract replay")
    print(f"PASS:24 source cases (5 unproved/1 pending/18 rejected),70 typed expression heads,{len(vectors)} Python/Rust/Go hash inputs; no source/definedness/activation proof",flush=True)


if __name__=="__main__":main()
