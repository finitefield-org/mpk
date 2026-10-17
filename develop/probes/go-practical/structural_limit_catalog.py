"""Exact numeric candidate inventory, preserving predecessor limit meanings."""
import re
from pathlib import Path
from run_proof_pilot import ROOT, sha
from structural_counters import VALUE_UNITS, CURRENT_UNITS, ALL_UNITS, DESCRIPTOR_UNITS, LOCATION_UNITS


def table(path, section, prefix, category, outcome, producer, test):
    raw=(ROOT/path).read_bytes(); text=raw.decode().split(section,1)[1]
    text=text.split("\n## ",1)[0];rows=[]
    for key, cell in re.findall(r"^\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*$",text,re.M):
        match=re.match(r"([0-9][0-9,]*)",cell.strip())
        if not match: continue
        name=key.strip().strip('`"')
        slug=re.sub(r"[^a-z0-9]+","_",name.lower()).strip("_")
        rows.append({"id":prefix+"."+slug,"name":name,"maximum":int(match[1].replace(",","")),"category":category,"above":outcome,
                     "counting":"unchanged predecessor counting rule in "+path+" "+section,
                     "origin":{"path":path,"sha256":sha(raw),"section":section},"planned_producer":producer,"planned_test":test})
    return rows


def generate():
    rows=table("develop/specs/GO_VIR_PROFILE_V0.md","## 13. Deterministic limits","capture","source_capture","capture_rejected","go-tools/go2vir/practical_capture.go","go-tools/go2vir/practical_capture_test.go")
    syntax=next(r for r in rows if r["id"]=="capture.typed_syntax_nodes_across_included_packages")
    syntax["origin_maximum"]=syntax["maximum"];syntax["maximum"]=1<<22
    syntax["counting"]="all materialized loaded go/ast nodes, including standard-library syntax; each tree once; new practical candidate only, predecessor remains1,000,000"
    syntax["rationale"]="pinned unused-HTTP original-package load has1,417,549 nodes (179 packages/1,025 syntax files); power-of-two ceiling above twice that measurement is4,194,304"
    # These are distinct measurements with their predecessor definitions;
    # repeated numeric ceilings do not merge independent counters.
    rows+=table("develop/specs/VIR_V0.md","## 12. Shared deterministic limits","shared","structural_artifact","model_rejected_limit","crates/mpk-vc/src/go_practical_artifacts.rs","crates/mpk-vc/tests/go_practical_artifacts.rs")
    rows+=table("develop/specs/SOURCE_MANIFEST_V0.md","## 11. Limits","manifest","capture_manifest","model_rejected_limit","go-tools/go2vir/practical_capture.go","go-tools/go2vir/practical_capture_test.go")
    for row in rows:
        if row["id"] == "shared.aggregate_type_nesting":
            row["counting"]="successor candidate: array/slice element, struct field and closed-error payload edges add one; named/alias edges add zero; predecessor array/struct meaning retained"
    semantic={"string_bytes":256,"slice_capacity":128,"positive_owners_per_exact_root_type":2,"joint_results":16,"source_word_bits":64}
    for name,maximum in semantic.items():
        rows.append({"id":"semantic."+name,"name":name,"maximum":maximum,"category":"semantic_domain","above":"domain_obligation_unproved","counting":"registered-profile candidate dimension; all typing/validity/source and domain members remain independently required","planned_producer":"crates/mpk-vc/src/go_practical_vc.rs","planned_test":"crates/mpk-vc/tests/go_practical_vc.rs"})
    for name,maximum in {"value_units":VALUE_UNITS,"current_store_units":CURRENT_UNITS,"past_store_units":CURRENT_UNITS,"all_store_units":ALL_UNITS}.items():
        rows.append({"id":"weighted."+name,"name":name,"maximum":maximum,"category":"carrier_accounting","above":"domain_obligation_unproved","counting":"complete fixed abstract value/owner/role/entry/result/target/past-store accounting; no live set supplied; saturated value above-limit count is a lower bound","planned_producer":"crates/mpk-vc/src/go_practical_storage.rs","planned_test":"crates/mpk-vc/tests/go_practical_storage.rs"})
    next(r for r in rows if r["id"]=="weighted.value_units")["counting"]="each source root/inline/observation value and each materialized Plain heap cell; backing array collections use store totals, not the per-value ceiling"
    proof={"certificate_bytes":1<<21,"certificate_terms":1<<17,"certificate_declarations":4096,"construction_wall_ns":60_000_000_000,"checker_wall_ns_each":60_000_000_000,"peak_rss_bytes_each_child":1<<30}
    for name,maximum in proof.items():
        rows.append({"id":"proof."+name,"name":name,"maximum":maximum,"category":"proof_resources","above":"proof_pending_resource","counting":"one bounded direct child/certificate; wall clock in monotonic nanoseconds and RSS in bytes; no false-claim inference","planned_producer":"crates/mpk-api/src/go_practical_policy.rs","planned_test":"crates/mpk-api/tests/go_practical_policy.rs"})
    ids=[r["id"] for r in rows]
    if len(ids)!=len(set(ids)):raise ValueError("independent unique counter IDs")
    return {"schema":"mpk.go_practical.structural_limits_draft.v0","status":"private candidate numeric/accounting inventory; no normative or runtime limit changed","counters":rows,
            "weights":{"bool_or_full_word":1,"byte_string":257,"slice_descriptor":DESCRIPTOR_UNITS,"typed_location":LOCATION_UNITS,"array":"1+n*W(element)","struct":"1+sum(W(field))","closed_error":"1+sum(W(each global-U payload))","named_alias":"W(resolved underlying)","owner_header":1,"scope_or_packet_header":1,"snapshot_observation_header":1},
            "weighted_rationale":"per-value2Mi units includes the complete 16-result product of [256]string (1,052,689 units); current-store8Mi and all-current/past32Mi give separate 4x composition ceilings. Units are abstract carrier slots, not Go bytes, RSS, proof cost or a support guarantee.",
            "pending":["complete counter/source/frame/finite-ID/normalization proofs, source capture generalized module/namespace enforcement","exact successor schema/hash identities and reviewed migration; cumulative T01 gate W05","production resource supervision and both-checker complete application proofs/releases"]}
