#!/usr/bin/env python3
"""Rebuild unchanged private checkers and check full ordinary carrier expansions."""
import argparse
import json
import os
import subprocess
import tempfile
from pathlib import Path
from closed_carriers import ROOT,HERE,RECORD,SEED,MODULE,candidate,schemas,FACTS,SOURCE,validate_expansion
from carrier_cases import CASES,plan,plain_error_candidate
from carrier_build import TARGET,prepare
from run_foundation_limits import inputs as prior_inputs
from run_proof_pilot import DEFAULT_GOROOT,json_bytes,sha,timed,observe,semantic_expectation,verify_host_toolchain
from identity_wire import canonical
from check_scope_report import strict_json,require,exact

NEW_SOURCES=("carrier_terms.py","closed_carriers.py","carrier_cases.py","carrier_build.py","run_closed_carriers.py","test_closed_carriers.py","check_carrier_metadata.py")
SCOPE="39 captured/fixture source types,42/43 concrete specializations,complete ordinary carrier/zero definitions and four arbitrary-value heterogeneous projection laws; unchanged joint private checkers; not source,validity,finite-id or full operation foundation"
PENDING=schemas()["remaining"]


def documents():
    f,i,e=candidate();t,r,pi,pe=plain_error_candidate()
    return [("fixture.json",f),("instances.json",i),("expansion.json",e),("plain-error-types.json",t),("plain-error-roots.json",r),("plain-error-instances.json",pi),("plain-error-expansion.json",pe),("schemas.json",schemas()),("cases.json",CASES)]


def encoded(name,value):return canonical(value)+b"\n" if "expansion" in name else json_bytes(value)


def inputs():
    rows={r["path"]:r["sha256"] for r in prior_inputs()}
    paths=[HERE/n for n in NEW_SOURCES]+[p for p in (HERE/"closed-carriers").rglob("*") if p.is_file()]+[SEED,FACTS,SOURCE]+[RECORD/n for n,_ in documents()]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def classify(case,obs):
    if case["expected"]=="accepted":
        semantic_expectation(case,obs,declarations=case["declarations"],module=MODULE);return "accepted_zero_axiom"
    r,g=obs["rust"],obs["go"]
    require(r["exit_code"]==g["exit_code"]==1 and r["report"].get("error_code")=="KERNEL_CORE_CHECK" and g["report"].get("error_kind")=="core_check","ordinary core rejection, not timeout/resource")
    require(json.loads(r["report"]["error_detail"])["code"]=="CORE_TYPE_MISMATCH" and g["report"]["error_detail"].startswith("term ") and " inferred " in g["report"]["error_detail"],"paired ordinary type mismatch")
    return "ordinary_core_type_mismatch"


def check_retained():
    report=strict_json((RECORD/"observations.json").read_bytes());require(report["scope"]==SCOPE and report["pending"]==PENDING and report["inputs"]==inputs(),"carrier scope/source pins")
    for name,value in documents():require((RECORD/name).read_bytes()==encoded(name,value),"exact complete carrier input: "+name)
    f,i,e=candidate();validate_expansion(f["types"],i["semantic_context"],f["roots"],i,e)
    require(len(report["cases"])==8,"two full carrier sets/six alteration controls")
    for row,expected in zip(report["cases"],CASES):
        c=row["case"];require(all(c[k]==v for k,v in expected.items()),"exact expected core case")
        require(c["bytes"]<=2097152 and c["terms"]<=131072 and c["declarations"]<=4096,"measured ordinary artifacts fit preceding candidate proof envelope")
        raw=(RECORD/(c["id"]+".mpcert")).read_bytes();require(len(raw)==c["bytes"] and sha(raw)==row["raw_sha256"] and row["classification"]==classify(c,row["observations"]),"complete canonical core evidence")
    for key in ("source_proofs_checked","complete_validity_proved","finite_ids_proved","production_checker_changed","profile_activated"):
        require(report[key] is False,"typing/projection evidence boundary: "+key)
    require(report["outcomes"]=={"accepted_zero_axiom":2,"ordinary_core_type_mismatch":6,"legacy":2},"exact carrier outcome counts")
    for row in report["legacy"]:
        raw=bytes.fromhex((ROOT/row["path"]).read_text());require(sha(raw)==row["raw_sha256"],"retained legacy input")
        semantic_expectation({"expected":"accepted"},row["observations"],declarations=row["observations"]["rust"]["report"]["declaration_count"],module=row["observations"]["rust"]["report"]["module"])
    return report


def run(cmd,*,cwd=ROOT,env=None):
    r=subprocess.run(list(map(str,cmd)),cwd=cwd,env=env,capture_output=True,timeout=300)
    require(r.returncode==0 and b"warning:" not in r.stderr,"private build: "+r.stdout.decode(errors="replace")+r.stderr.decode(errors="replace"));return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument("--write",action="store_true");g.add_argument("--check",action="store_true");args=p.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in documents():(RECORD/name).write_bytes(encoded(name,value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/"host-toolchain.json").read_bytes()))
    with tempfile.TemporaryDirectory(prefix="mpk-carriers-",dir="/private/tmp") as temporary:
        temp=Path(temporary);workspace=temp/"workspace";receipt=prepare(workspace)
        run(["cargo","build","--release","--offline","--locked","--manifest-path",workspace/"Cargo.toml","-p","go-practical-closed-carriers"],env={**os.environ,"CARGO_TARGET_DIR":str(TARGET)})
        rust=TARGET/"release/go-practical-closed-carriers";go=temp/"checker"
        env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
        run([DEFAULT_GOROOT/"bin/go","build","-trimpath","-buildvcs=false","-o",go,"./cmd/mpk-checker-ref"],cwd=workspace/"go-tools/mpk-checker-ref",env=env)
        binaries={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())};rows=[];outputs={}
        for case in CASES:
            ast=plan(case);wire=temp/(case["id"]+".json");wire.write_bytes(canonical(ast)+b"\n")
            files=[temp/(case["id"]+f"-{n}.mpcert") for n in (0,1)];runs=[timed([rust,"emit",SEED,wire,file]) for file in files]
            require(all(r["exit_code"]==0 for r in runs) and files[0].read_bytes()==files[1].read_bytes(),"two exact ordinary emitter outputs")
            counts=strict_json(runs[0]["stdout"]);require(counts["added"]==[d["name"] for d in ast["declarations"]],"every declared ordinary member emitted in order")
            c={**case,**{k:counts[k] for k in ("bytes","terms","declarations")},"new_definitions":sum(d["kind"]=="definition" for d in ast["declarations"]),"new_theorems":sum(d["kind"]=="theorem" for d in ast["declarations"])}
            require(c["bytes"]<=2097152 and c["terms"]<=131072 and c["declarations"]<=4096,"candidate certificate construction budget")
            obs={k:observe(exe,files[0]) for k,exe in (("rust",rust),("go",go))};classification=classify(c,obs)
            require(all(r["peak_rss_bytes"]<=1073741824 and r["wall_seconds"]<=60 for r in runs+list(obs.values())),"private direct-child candidate resource envelope")
            raw=files[0].read_bytes();outputs[case["id"]+".mpcert"]=raw
            rows.append({"case":c,"ast_sha256":sha(wire.read_bytes()),"raw_sha256":sha(raw),"generations":runs,"observations":obs,"classification":classification})
            print(case["id"]+": "+classification,flush=True)
        legacy=[]
        for name,path in (("legacy-bool","fixtures/core-bool-cases/right-identity.hex"),("legacy-nat","proofs/std/nat/std-nat.hex")):
            raw=bytes.fromhex((ROOT/path).read_text());file=temp/(name+".mpcert");file.write_bytes(raw)
            obs={k:observe(exe,file) for k,exe in (("rust",rust),("go",go))};semantic_expectation({"expected":"accepted"},obs,declarations=obs["rust"]["report"]["declaration_count"],module=obs["rust"]["report"]["module"])
            legacy.append({"id":name,"path":path,"raw_sha256":sha(raw),"observations":obs})
        for row in receipt["generated"]:require(sha((workspace/row["path"]).read_bytes())==row["sha256"],"unchanged77-file joint checker proposal")
        require(before==inputs() and binaries=={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())},"source/binary stability")
        report={"schema":"mpk.go_practical.closed_carriers_probe.v0","scope":SCOPE,"pending":PENDING,"inputs":before,"mirrored_checker_inputs":receipt["generated"],"checker_binaries":binaries,"host_toolchain_inventory":inventory,"cases":rows,"legacy":legacy,"source_proofs_checked":False,"complete_validity_proved":False,"finite_ids_proved":False,"production_checker_changed":False,"profile_activated":False,"direct_child_parallelism":1,"child_ceiling_seconds":60,"ceiling_scope":"private measurement ceiling; not an application/release guarantee","outcomes":{"accepted_zero_axiom":2,"ordinary_core_type_mismatch":6,"legacy":2}}
        if args.write:
            for name,raw in outputs.items():(RECORD/name).write_bytes(raw)
            (RECORD/"observations.json").write_bytes(json_bytes(report))
        old=check_retained();require(old["checker_binaries"]==binaries and all((RECORD/name).read_bytes()==raw for name,raw in outputs.items()),"fresh deterministic carrier/checker replay")
    print("PASS:39 source/fixture types,42/43 specializations,2 accepted complete ordinary sets,6 core-rejected controls,2 legacy; not source/validity/finite-id completion",flush=True)


if __name__=="__main__":main()
