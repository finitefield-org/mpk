#!/usr/bin/env python3
"""Build/replay private ordinary order/strong-induction and type-case evidence."""
import argparse
import json
import os
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

import termination_foundation_build as build
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, json_bytes, sha, observe, timed, verify_host_toolchain, semantic_expectation

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/nat-termination"
TARGET=Path("/private/tmp/mpk-go-practical-termination-target")
MODULE="Probe.GoPractical.NatTermination"
SCOPE="private additive fixed Bool.recType plus retained Sort0 Nat.induct; ordinary arbitrary-Nat order and general Sort0 strong-induction proofs; no production checker/profile/source-application or completed variant termination claim"
PENDING=["W04 exact additive core/foundation/overlay/wire/limits/migration freeze and review",
         "ordinary complete loop variant/range/word bridge and source-model composition in T02–T06",
         "W05 cumulative T01 gate; T07 installed activation"]
FAMILIES=("less-zero","less-zero-successor","less-successor-cancel","less-irreflexive",
          "strong-induction","type-select-zero","type-select-successor","false-elimination")
VARIANTS=("canonical-false","canonical-true","legacy-without-type-cases","nongenerated","wrong-interface",
          "wrong-false-name","wrong-true-type","nongenerated-true","extra-constructor-before","extra-constructor-after",
          "higher-universe-family","higher-result-universe","recursor-levels-partial","recursor-levels-neutral",
          "family-levels","constructor-levels","prior-opaque-levels","prior-theorem-levels","prior-family-levels","unused-term-levels")
ACCEPTED_VARIANTS={"canonical-false","canonical-true","legacy-without-type-cases","unused-term-levels"}


def inputs():
    rows={r["path"]:r["sha256"] for r in build.sources()}
    for name in ("run_termination_probes.py","check_termination_report.py","test_termination_report.py",
                 "run_proof_pilot.py","run_module_probes.py","host-toolchain.json"):
        p=HERE/name;rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def classify(case, observations, interface=False):
    if set(observations)!={"rust","go"}:raise ValueError("termination independent checkers")
    if case["expected"]=="accepted":
        semantic_expectation(case,observations,declarations=case["declarations"],
                             module=case.get("module",MODULE))
        return "accepted_zero_axiom"
    r,g=observations["rust"],observations["go"]
    if r["exit_code"]!=1 or g["exit_code"]!=1 or r["report"].get("error_code")!="KERNEL_CORE_CHECK" or g["report"].get("error_kind")!="core_check":
        raise ValueError("termination rejection boundary")
    code=json.loads(r["report"]["error_detail"])["code"]
    if code!=("CORE_INVALID_DECLARATION" if interface else "CORE_TYPE_MISMATCH"):
        raise ValueError("termination fuel/other failure is not expected rejection")
    if not interface and (not g["report"]["error_detail"].startswith("term ") or " inferred " not in g["report"]["error_detail"]):
        raise ValueError("termination Go type rejection")
    if interface and not any(t in g["report"]["error_detail"] for t in ("Bool type selection","universe argument")):
        raise ValueError("termination Go interface rejection")
    return "malformed_interface_rejection" if interface else "core_type_mismatch"


def run(command, *, env=None, cwd=ROOT):
    p=subprocess.run(list(map(str,command)),cwd=cwd,env=env,capture_output=True,timeout=300)
    if p.returncode:
        raise RuntimeError(p.stdout.decode()+p.stderr.decode())
    return p.stdout,p.stderr


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write",action="store_true");group.add_argument("--check",action="store_true")
    args=parser.parse_args()
    if (platform.system(),platform.machine())!=("Darwin","arm64"):raise RuntimeError("private resource/toolchain receipt requires Darwin arm64")
    before=inputs()
    inventory=verify_host_toolchain(DEFAULT_GOROOT,json.loads((HERE/"host-toolchain.json").read_text()))
    with tempfile.TemporaryDirectory(prefix="mpk-nat-termination-",dir="/private/tmp") as temporary:
        temp=Path(temporary);workspace=temp/"workspace";receipt=build.prepare(workspace)
        env={**os.environ,"CARGO_TARGET_DIR":str(TARGET)}
        out,err=run(["cargo","build","--release","--offline","--locked","--manifest-path",workspace/"Cargo.toml","-p","go-practical-termination-probe"],env=env)
        if b"warning:" in err:raise RuntimeError("private release compiler warning")
        rust=TARGET/"release/go-practical-termination-probe";go=temp/"checker"
        go_env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off",
                "GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"",
                "GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
        run([DEFAULT_GOROOT/"bin/go","build","-trimpath","-buildvcs=false","-o",go,"./cmd/mpk-checker-ref"],
            cwd=workspace/"go-tools/mpk-checker-ref",env=go_env)
        binaries={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())}
        first,second=temp/"first",temp/"second"
        generations=[timed([rust,"termination-generate",ROOT,p]) for p in (first,second)]
        names=sorted(p.name for p in first.iterdir())
        if any(r["exit_code"] for r in generations) or names!=sorted(p.name for p in second.iterdir()) or any((first/n).read_bytes()!=(second/n).read_bytes() for n in names):
            raise RuntimeError("termination generation differs")
        all_rows=[]
        for index,interface in (("index.json",False),("interfaces.json",True)):
            rows=[]
            for c in json.loads((first/index).read_text())["cases"]:
                raw=(first/(c["id"]+".mpcert")).read_bytes()
                observations={k:observe(exe,first/(c["id"]+".mpcert")) for k,exe in (("rust",rust),("go",go))}
                kind=classify(c,observations,interface)
                rows.append({"case":c,"raw_sha256":sha(raw),"classification":kind,"observations":observations})
                print(c["id"]+": "+kind,flush=True)
            all_rows.append(rows)
        regressions=[]
        paths=[("legacy-bool",ROOT/"fixtures/core-bool-cases/right-identity.hex",True),
               ("legacy-nat",ROOT/"proofs/std/nat/std-nat.hex",True),
               ("retained-nat-induct",ROOT/"develop/migrations/go-practical/t01-w03/control-induction/control-invariant-entry-valid.mpcert",False)]
        for identity,path,is_hex in paths:
            raw=bytes.fromhex(path.read_text()) if is_hex else path.read_bytes()
            p=temp/(identity+".mpcert");p.write_bytes(raw)
            observations={k:observe(exe,p) for k,exe in (("rust",rust),("go",go))}
            r=observations["rust"]["report"]
            semantic_expectation({"expected":"accepted"},observations,declarations=r["declaration_count"],module=r["module"])
            regressions.append({"id":identity,"path":path.relative_to(ROOT).as_posix(),"raw_sha256":sha(raw),"observations":observations})
        if before!=inputs() or binaries!={"rust":sha(rust.read_bytes()),"go":sha(go.read_bytes())}:raise RuntimeError("termination source/binary changed")
        report={"schema":"mpk.go_practical.nat_termination_report.v0","scope":SCOPE,"status":"private strong-induction feasibility checked; W04 remains in progress","pending":PENDING,
                "inputs":before,"mirrored_checker_inputs":receipt["generated"],"checker_binaries":binaries,
                "host_toolchain_inventory":inventory,"generations":generations,"direct_child_parallelism":1,
                "child_ceiling_seconds":60,"ceiling_scope":"private measurement setting, not normative profile limits",
                "cases":all_rows[0],"interface_cases":all_rows[1],"regressions":regressions,
                "outcomes":{"ordinary_accepted":8,"ordinary_type_mismatch":8,"interface_accepted":4,"malformed_interface":16,"legacy_regressions":3}}
        if args.write:
            RECORD.mkdir(exist_ok=True)
            for name in names:shutil.copyfile(first/name,RECORD/name)
            (RECORD/"observations.json").write_bytes(json_bytes(report))
        from check_termination_report import check
        retained=json.loads((RECORD/"observations.json").read_text());check(retained)
        if retained["checker_binaries"]!=binaries or any((RECORD/n).read_bytes()!=(first/n).read_bytes() for n in names):
            raise RuntimeError("termination fresh-build replay identity")
        print("PASS: 16 ordinary claims, 20 isolated interface cases and 3 legacy regressions; both private checkers; no public/source/variant claim",flush=True)


if __name__=="__main__":main()
