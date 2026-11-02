"""Check retained private proof/type-case evidence and preserve its exact scope."""
import re
from check_scope_report import exact,require,sha,strict_json
from run_termination_probes import RECORD,ROOT,MODULE,SCOPE,PENDING,FAMILIES,VARIANTS,ACCEPTED_VARIANTS,inputs,classify
import termination_foundation_build as build
from run_proof_pilot import semantic_expectation


def check(report):
    require(set(report)=={"schema","scope","status","pending","inputs","mirrored_checker_inputs",
                         "checker_binaries","host_toolchain_inventory","generations","direct_child_parallelism",
                         "child_ceiling_seconds","ceiling_scope","cases","interface_cases","regressions","outcomes"},
            "termination closed report shape")
    require(report["schema"]=="mpk.go_practical.nat_termination_report.v0" and report["scope"]==SCOPE
            and report["status"]=="private strong-induction feasibility checked; W04 remains in progress","termination scope is not source/public/variant completion")
    require(report["pending"]==PENDING,"remaining termination/source/freeze work")
    require(report["inputs"]==inputs(),"termination complete source identities")
    require(report["mirrored_checker_inputs"]==[{"path":p,"sha256":sha(raw)} for p,raw in sorted(build.files().items())],"termination mirrored checker identities")
    require(set(report["checker_binaries"])=={"rust","go"} and
            all(re.fullmatch("[0-9a-f]{64}",h) for h in report["checker_binaries"].values()),
            "termination checker binary identities")
    require(exact(report["host_toolchain_inventory"],{"files":14496,"sha256":"85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc"}),"termination exact toolchain")
    require(report["direct_child_parallelism"]==1 and report["child_ceiling_seconds"]==60
            and report["ceiling_scope"]=="private measurement setting, not normative profile limits","termination private resource scope")
    require(len(report["generations"])==2 and all(r["exit_code"]==0 for r in report["generations"]),"termination deterministic construction")
    ids=[f"{f}-{suffix}" for f in FAMILIES for suffix in ("valid","altered")]
    require([r["case"]["id"] for r in report["cases"]]==ids,"termination ordinary coverage")
    require([r["case"]["id"] for r in report["interface_cases"]]==["type-interface-"+v for v in VARIANTS],"termination interface coverage")
    for index,key,interface in (("index.json","cases",False),("interfaces.json","interface_cases",True)):
        cases=strict_json((RECORD/index).read_bytes())["cases"]
        require([r["case"] for r in report[key]]==cases,"termination exact case index")
        for row in report[key]:
            c=row["case"];raw=(RECORD/(c["id"]+".mpcert")).read_bytes()
            expected=("accepted" if c["variant"] in ACCEPTED_VARIANTS else "rejected") if interface else ("rejected" if c["altered"] else "accepted")
            require(c["expected"]==expected,"termination expected boundary")
            require(row["raw_sha256"]==sha(raw) and c["bytes"]==len(raw),"termination canonical candidate bytes")
            require(row["classification"]==classify(c,row["observations"],interface),"termination observation classification")
            for r in row["observations"].values():
                require(r["wall_seconds"]>=0 and r["wall_seconds"]<60 and r["peak_rss_bytes"]>0,"termination child resource fields")
    require([r["id"] for r in report["regressions"]]==["legacy-bool","legacy-nat","retained-nat-induct"],"termination legacy coverage")
    for row in report["regressions"]:
        p=ROOT/row["path"];raw=bytes.fromhex(p.read_text()) if p.suffix==".hex" else p.read_bytes()
        require(sha(raw)==row["raw_sha256"],"termination legacy bytes")
        r=row["observations"]["rust"]["report"]
        semantic_expectation({"expected":"accepted"},row["observations"],declarations=r["declaration_count"],module=r["module"])
    require(report["outcomes"]=={"ordinary_accepted":8,"ordinary_type_mismatch":8,"interface_accepted":4,"malformed_interface":16,"legacy_regressions":3},"termination outcome counts")
    return 39


if __name__=="__main__":
    print(f"PASS: {check(strict_json((RECORD/'observations.json').read_bytes()))} retained private proof/interface/regression cases; W04 open")
