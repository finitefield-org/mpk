#!/usr/bin/env python3
"""Replay structural/carrier candidates, full HTTP syntax and native layouts."""
import argparse
import json
import os
import subprocess
import tempfile
import time
from pathlib import Path
from structural_counter_cases import boundary_cases,shape_observations,quote_type_weights,state_observations,source_inputs
from structural_limit_catalog import generate
from run_source_admission import inputs as predecessor_inputs
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from check_scope_report import strict_json,require,exact

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/structural-limits"
SCOPE="private exact numeric/accounting candidates and native parser/type-size/loaded-syntax measurements; no production counter/allocator/source/finite-ID proof or profile activation"
NEW_SOURCES=("structural_counters.py","structural_counter_cases.py","structural_limit_catalog.py","structural_go_metrics.go","structural_module_metrics.go","test_structural_counters.py","run_structural_limits.py","check_structural_metadata.py")
PENDING=generate()["pending"]


def inputs():
    rows={r["path"]:r["sha256"] for r in predecessor_inputs()}
    paths=[HERE/n for n in NEW_SOURCES]+[RECORD/"limits.json",RECORD/"source-vectors.json",ROOT/"develop/migrations/go-practical/t01-w04/overlay-contracts/native-facts.json",HERE/"module-cases.json",ROOT/"go-tools/go2vir/go.mod",ROOT/"go-tools/go2vir/go.sum"]
    paths += [ROOT/p for p in ("develop/specs/GO_VIR_PROFILE_V0.md","develop/specs/VIR_V0.md","develop/specs/SOURCE_MANIFEST_V0.md")]
    paths += [p for p in (ROOT/"go-tools/go2vir/vendor").rglob("*") if p.is_file()]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{"path":p,"sha256":h} for p,h in sorted(rows.items())]


def timed_input(command,payload,env):
    start=time.perf_counter()
    with tempfile.TemporaryFile() as stdin,tempfile.TemporaryFile() as stdout,tempfile.TemporaryFile() as stderr:
        stdin.write(payload);stdin.seek(0)
        process=subprocess.Popen(list(map(str,command)),stdin=stdin,stdout=stdout,stderr=stderr,env=env)
        while True:
            pid,status,usage=os.wait4(process.pid,os.WNOHANG)
            if pid:
                process.returncode=os.waitstatus_to_exitcode(status);break
            if time.perf_counter()-start>300:
                process.kill();os.wait4(process.pid,0);raise RuntimeError("private measurement process timeout, not source falsity")
            time.sleep(.002)
        stdout.seek(0);stderr.seek(0);raw=stdout.read();errors=stderr.read()
    require(process.returncode==0 and not errors,"native metric process: "+errors.decode())
    return raw,{"exit_code":process.returncode,"wall_seconds":time.perf_counter()-start,"peak_rss_bytes":usage.ru_maxrss,"resource_provider":"Darwin os.wait4 child rusage; not a process-tree RSS sum","stdout_sha256":sha(raw)}


def check_retained():
    r=strict_json((RECORD/"observations.json").read_bytes())
    require(r["scope"]==SCOPE and r["pending"]==PENDING and r["inputs"]==inputs(),"numeric/accounting input/scope identity")
    require((RECORD/"limits.json").read_bytes()==json_bytes(generate()) and (RECORD/"source-vectors.json").read_bytes()==json_bytes(source_inputs()),"closed numeric/source recipes")
    require(r["production_counter_enforcement"] is False and r["source_allocator_binding_proved"] is False and r["canonical_finite_ids_proved"] is False and r["application_executed"] is False,"private counter/source scope")
    require(exact(r["counter_boundaries"],boundary_cases()) and exact(r["shapes"],shape_observations()) and exact(r["quote_types"],quote_type_weights()) and exact(r["states"],state_observations()),"complete counter/type/state replay")
    native=(RECORD/"native-layouts.json").read_bytes();http=(RECORD/"native-http-syntax.json").read_bytes()
    require(sha(native)==r["native_layouts_sha256"] and sha(http)==r["native_http_sha256"],"native raw identities")
    facts=strict_json(native);require(len(facts)==85,"native original layout corpus")
    for c,f in zip(source_inputs(),facts):require(c["id"]==f["id"] and sha(c["source"].encode())==f["source_sha256"] and f["source_bytes"]==len(c["source"].encode()),"native original bytes")
    require([f["kind"] for f in facts[-3:]]==["typed_original_package"]*3 and all(f["budget_value_native_sizeof"]>0 for f in facts[-3:]),"Go-valid below/at/above model-weight layouts")
    h=strict_json(http)
    matrix=strict_json((HERE/"module-cases.json").read_bytes());case=next(c for c in matrix["cases"] if c["id"]=="unused-http-handler")
    app_files={**case["files"],"go.mod":"module "+matrix["module"]+"\n\ngo 1.23\n"}
    require(r["http_original_inputs"]==[{"path":n,"bytes":len(s.encode()),"sha256":sha(s.encode())} for n,s in sorted(app_files.items())],"exact original HTTP fixture identity")
    require(not h["diagnostics"] and (h["total_syntax_nodes"],h["standard_syntax_nodes"],h["application_syntax_nodes"],len(h["packages"]),sum(p["syntax_files"] for p in h["packages"]))==(1417549,1417510,39,179,1025),"complete pinned HTTP syntax measurement")
    require(r["http_counter_fit"]=={"predecessor_1000000":False,"practical_candidate_4194304":True},"explicit new candidate, predecessor preserved")
    for runs,digest in ((r["native_layout_runs"],sha(native)),(r["http_runs"],sha(http))):
        require(len(runs)==2 and all(p["exit_code"]==0 and p["stdout_sha256"]==digest for p in runs),"two native raw observations")
    return r


def main():
    parser=argparse.ArgumentParser(description=__doc__);g=parser.add_mutually_exclusive_group(required=True);g.add_argument("--write",action="store_true");g.add_argument("--check",action="store_true");args=parser.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True);(RECORD/"limits.json").write_bytes(json_bytes(generate()));(RECORD/"source-vectors.json").write_bytes(json_bytes(source_inputs()))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/"host-toolchain.json").read_bytes()))
    env={**os.environ,"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOCACHE":"/private/tmp/mpk-go-practical-checker-cache"}
    with tempfile.TemporaryDirectory(prefix="mpk-structural-",dir="/private/tmp") as temporary:
        temp=Path(temporary);layout=temp/"layouts";module=temp/"module"
        for source,exe,cwd,extra in (("structural_go_metrics.go",layout,ROOT,[]),("structural_module_metrics.go",module,ROOT/"go-tools/go2vir",["-mod=vendor"])):
            p=subprocess.run([str(DEFAULT_GOROOT/"bin/go"),"build",*extra,"-trimpath","-buildvcs=false","-o",str(exe),str(HERE/source)],cwd=cwd,env=env,capture_output=True,timeout=300)
            require(p.returncode==0 and not p.stderr,"new native metrics build: "+p.stderr.decode())
        binaries={"layout":sha(layout.read_bytes()),"module":sha(module.read_bytes())}
        native_runs=[timed_input([layout],json_bytes(source_inputs()),env) for _ in range(2)]
        require(native_runs[0][0]==native_runs[1][0],"native layout repeat")
        matrix=strict_json((HERE/"module-cases.json").read_bytes());case=next(c for c in matrix["cases"] if c["id"]=="unused-http-handler")
        app=temp/"app";app.mkdir();files={**case["files"],"go.mod":"module "+matrix["module"]+"\n\ngo 1.23\n"}
        for name,source in files.items():(app/name).write_text(source)
        app_pins=[{"path":n,"bytes":len(s.encode()),"sha256":sha(s.encode())} for n,s in sorted(files.items())]
        loader_env={"PATH":str(DEFAULT_GOROOT/"bin")+":/usr/bin:/bin","GOROOT":str(DEFAULT_GOROOT),"GOENV":"off","GOWORK":"off","GOTOOLCHAIN":"local","GOPROXY":"off","GOSUMDB":"off","GOTELEMETRY":"off","CGO_ENABLED":"0","GOFLAGS":"","GOEXPERIMENT":"","GOOS":"linux","GOARCH":"amd64","GOAMD64":"v1","GOCACHE":env["GOCACHE"],"GOPATH":str(temp/"gopath"),"GOMODCACHE":str(temp/"modules"),"GOMAXPROCS":"4"}
        http_runs=[timed_input([module],json_bytes({"root":str(app),"package":matrix["module"]}),loader_env) for _ in range(2)]
        require(http_runs[0][0]==http_runs[1][0] and all((app/n).read_bytes()==s.encode() for n,s in files.items()),"intact original HTTP source/repeat")
        native,http=native_runs[0][0],http_runs[0][0]
        report={"schema":"mpk.go_practical.structural_limits_probe.v0","scope":SCOPE,"pending":PENDING,"inputs":before,"host_toolchain_inventory":inventory,"native_binaries":binaries,"native_layouts_sha256":sha(native),"native_http_sha256":sha(http),"native_layout_runs":[r for _,r in native_runs],"http_runs":[r for _,r in http_runs],"http_original_inputs":app_pins,"counter_boundaries":boundary_cases(),"shapes":shape_observations(),"quote_types":quote_type_weights(),"states":state_observations(),"http_counter_fit":{"predecessor_1000000":False,"practical_candidate_4194304":True},"production_counter_enforcement":False,"source_allocator_binding_proved":False,"canonical_finite_ids_proved":False,"application_executed":False,"direct_child_parallelism":1}
        require(before==inputs() and binaries=={"layout":sha(layout.read_bytes()),"module":sha(module.read_bytes())},"sources/native binaries stable")
        if args.write:
            (RECORD/"native-layouts.json").write_bytes(native);(RECORD/"native-http-syntax.json").write_bytes(http);(RECORD/"observations.json").write_bytes(json_bytes(report))
        old=check_retained();require(old["native_binaries"]==binaries and (RECORD/"native-layouts.json").read_bytes()==native and (RECORD/"native-http-syntax.json").read_bytes()==http,"fresh native identity")
        print("PASS: 64 numeric candidates/192 isolated boundaries, 85 original layout sources, full HTTP179-package/1,417,549-node count, typed/current/past carrier accounting; no production/source/allocator proof",flush=True)


if __name__=="__main__":main()
