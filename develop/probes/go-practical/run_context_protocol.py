#!/usr/bin/env python3
"""Replay six private root schemas with actual retained Rust policy calls."""
import argparse
import os
import subprocess
import sys
import unicodedata
from collections import Counter
from context_protocol import Protocol, registration_fixture, active_registry, schemas, ContextError
from context_vectors import cases, legacy_queries, contexts
from identity_wire import canonical, parse
from run_identity_dag import inputs as predecessor_inputs
from run_structural_limits import timed_input
from run_proof_pilot import ROOT, HERE, json_bytes, sha
from check_scope_report import strict_json, require, exact

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/context-protocol"
SCOPE="six private closed registry/entry/context/request/Go-parameters/Go-selection root schemas; retained four profiles use actual unchanged compiled Rust policy; no complete foundation/source/proof/activation"
NEW_SOURCES=("context_protocol.py","context_vectors.py","run_context_protocol.py","test_context_protocol.py","check_context_metadata.py")
PENDING=schemas()["remaining"]


def inputs():
    rows={r["path"]:r["sha256"] for r in predecessor_inputs()}
    paths=[HERE/name for name in NEW_SOURCES]+[ROOT/"Cargo.toml",ROOT/"Cargo.lock"]
    paths += [p for p in (HERE/"context-legacy").rglob("*") if p.is_file()]
    # The observer links these real crates. Pin all their sources, not merely the wrapper.
    for crate in ("mpk-vc","mpk-cert","mpk-core"):
        paths.append(ROOT/"crates"/crate/"Cargo.toml")
        paths += [p for p in (ROOT/"crates"/crate/"src").rglob("*") if p.is_file()]
    paths += [ROOT/"develop/specs/vectors/semantic-profile-registry-v3.json"]
    paths += [RECORD/name for name in ("schemas.json","registration-fixture.json","vectors.json","legacy-queries.json")]
    for path in paths:rows[path.relative_to(ROOT).as_posix()]=sha(path.read_bytes())
    return [{"path":path,"sha256":h} for path,h in sorted(rows.items())]


def policy_from_outputs(queries,outputs):
    require(type(outputs) is list and len(outputs)==len(queries),"complete actual legacy observations")
    for out in outputs:
        require(type(out) is dict and set(out)=={"context_accepted","selection_accepted"} and all(type(v) is bool for v in out.values()),"native result types")
    cache={canonical(q):out for q,out in zip(queries,outputs)}
    def policy(query):
        key=canonical(query)
        if key not in cache:raise AssertionError("unobserved legacy query; never treat absent oracle evidence as rejection/success")
        return cache[key]
    return policy


def observe(protocol,row):
    kind=row["kind"];value=row["value"]
    try:
        if kind=="registry":protocol.validate_registry(value)
        elif kind=="context":protocol.validate_context(value)
        elif kind=="request":protocol.validate_request(value)
        elif kind=="linkage":protocol.linkage(*value)
        elif kind=="wire":parse(bytes.fromhex(value));raise AssertionError("malformed wire unexpectedly parsed")
        else:raise AssertionError("unknown private vector kind")
        accepted=True
    except (ContextError,ValueError,TypeError):accepted=False
    require(accepted is row["expected_accepted"],"unexpected private protocol outcome: "+row["id"])
    return {"id":row["id"],"kind":kind,"result":kind+"_candidate" if accepted else "configuration_rejected"}


def outcomes(observed):
    return {kind:dict(sorted(Counter(r["result"] for r in observed if r["kind"]==kind).items())) for kind in sorted({r["kind"] for r in observed})}


def documents():return (("schemas.json",schemas()),("registration-fixture.json",registration_fixture()),("vectors.json",cases()),("legacy-queries.json",legacy_queries()))


def check_retained():
    report=strict_json((RECORD/"observations.json").read_bytes())
    require(report["scope"]==SCOPE and report["pending"]==PENDING and report["inputs"]==inputs(),"context protocol scope/source pins")
    for name,content in documents():require((RECORD/name).read_bytes()==json_bytes(content),"canonical private context input: "+name)
    raw=(RECORD/"legacy-observations.json").read_bytes();outputs=strict_json(raw)
    p=Protocol(policy_from_outputs(legacy_queries(),outputs));observed=[observe(p,row) for row in cases()]
    require(exact(report["cases"],observed) and exact(report["outcomes"],outcomes(observed)),"complete expected protocol outcomes")
    require(report["legacy_query_count"]==len(outputs) and len(report["native_runs"])==2 and all(r["exit_code"]==0 and r["stdout_sha256"]==sha(raw) for r in report["native_runs"]),"two exact actual legacy policy runs")
    for key in ("complete_foundation","production_source_import","source_proofs_checked","legacy_semantic_equivalence_proved","profile_activated"):
        require(report[key] is False,"private configuration evidence boundary: "+key)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);g=parser.add_mutually_exclusive_group(required=True);g.add_argument("--write",action="store_true");g.add_argument("--check",action="store_true");args=parser.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,content in documents():(RECORD/name).write_bytes(json_bytes(content))
    before=inputs();env={**os.environ,"CARGO_TARGET_DIR":str(ROOT/"target")}
    build=subprocess.run(["cargo","build","--offline","--locked","--manifest-path",str(HERE/"context-legacy/Cargo.toml")],cwd=ROOT,env=env,capture_output=True,timeout=300)
    require(build.returncode==0 and b"warning:" not in build.stderr,"actual legacy observer build: "+build.stderr.decode())
    exe=ROOT/"target/debug/go-practical-context-legacy";binary=sha(exe.read_bytes())
    versions={name:subprocess.check_output([name,"--version"],env=env).decode().strip() for name in ("cargo","rustc")}
    versions.update({"python":sys.version.split()[0],"unicode_data":unicodedata.unidata_version})
    queries=legacy_queries();payload=json_bytes({"registry":active_registry(),"queries":queries})
    runs=[timed_input([exe],payload,env) for _ in range(2)]
    require(runs[0][0]==runs[1][0],"identical actual retained policy results")
    raw=runs[0][0];p=Protocol(policy_from_outputs(queries,strict_json(raw)));observed=[observe(p,row) for row in cases()]
    require(before==inputs() and binary==sha(exe.read_bytes()),"context protocol sources/binary stable")
    report={"schema":"mpk.go_practical.context_protocol_probe.v0","scope":SCOPE,"pending":PENDING,"inputs":before,"tool_versions":versions,"native_binary_sha256":binary,"native_runs":[r for _,r in runs],"legacy_query_count":len(queries),"cases":observed,"outcomes":outcomes(observed),"complete_foundation":False,"production_source_import":False,"source_proofs_checked":False,"legacy_semantic_equivalence_proved":False,"profile_activated":False,"direct_child_parallelism":1}
    if args.write:
        (RECORD/"legacy-observations.json").write_bytes(raw)
        (RECORD/"observations.json").write_bytes(json_bytes(report))
    old=check_retained();require(old["native_binary_sha256"]==binary and old["tool_versions"]==versions and (RECORD/"legacy-observations.json").read_bytes()==raw,"fresh actual policy binary/outputs replay")
    print(f"PASS: six private closed root schemas, five registry profiles/{len(contexts())} valid target contexts, {len(observed)} protocol vectors/{len(queries)} actual retained-policy queries; no source/foundation/profile completion",flush=True)


if __name__=="__main__":main()
