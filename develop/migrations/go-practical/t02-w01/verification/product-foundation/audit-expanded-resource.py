"""Observe full original expanded construction and unchanged producer-budget refusal."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
execution=root/"expanded-resource/collected/diagnostic/result.json";d=json.loads(execution.read_text())
assert d["status"]=="passed" and not d["changed_inputs_during_execution"]
for source in d["sources"]:assert seal(source["path"])==source
original_run=root/"final/tests/result.json";old=json.loads(original_run.read_text())
assert old["status"]=="failed" and old["exit_code"]==101 and not old["changed_inputs_during_execution"]
for key in ["stdout","stderr"]:
    assert seal(old[key]["path"])==old[key] and seal(d[key]["path"])==d[key]
assert "3 passed; 1 failed" in Path(old["stdout"]["path"]).read_text()
assert "c.declarations.len() <= 4096 && c.term_table.len() <= 131_072" in Path(old["stderr"]["path"]).read_text()
original=Path("crates/mpk-vc/tests/support/go_product_foundation_shapes.rs")
helper=Path("crates/mpk-vc/tests/support/go_product_expanded_resource.rs")
s=original.read_text();t=helper.read_text()
old_header="//! Independent ordinary-term consumer: every heterogeneous arity and field frame."
new_header="//! Born diagnostic copy: retain every original expanded candidate, including over-cap bytes.\n//! The only computation change removes the pre-encoding guard to observe full inputs;\n//! Out-of-budget streams supply no Go producer or kernel proof authority."
old_guard="        assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131_072);"
new_guard="        // Preserve the entire original over-cap candidate for resource diagnosis."
assert s.count(old_header)==s.count(old_guard)==1
assert t==s.replace(old_header,new_header,1).replace(old_guard,new_guard,1)
rows=[]
for arity in range(65):
    path=execution.parent/f"certificates/balanced-shape-{arity:02}.mpcert"
    metadata=path.with_suffix(".diagnostic.json");m=json.loads(metadata.read_text());candidate=seal(path)
    assert (candidate["sha256"],candidate["bytes"])==(m["raw_sha256"],m["bytes"])
    assert not m["kernel_executed"] and m["declared_axiom_count"]==0
    assert m["declarations"]==80+arity and m["bytes"]<=2097152
    assert m["outside_frozen_Go_producer_term_budget"]==(m["terms"]>131072)==(arity>=60)
    checked=None
    if arity<60:
        previous=root/f"final/tests/certificates/balanced-shape-{arity:02}.mpcert"
        assert previous.read_bytes()==path.read_bytes() and m["original_complete_checked_bytes_equal"]
        report=previous.with_suffix(".rust.json");r=json.loads(report.read_text())
        assert r["verdict"]=="accepted" and r["axiom_count"]==0 and not r["axiom_report"]["entries"]
        checked={"original_complete_checked_bytes":seal(previous),"actual_original_Rust_report":seal(report)}
    else:assert not m["original_complete_checked_bytes_equal"]
    rows.append({"arity":arity,"complete_candidate":candidate,"construction_metadata":seal(metadata),"terms":m["terms"],"declarations":m["declarations"],"outside_frozen_Go_producer_term_budget":arity>=60,"prior_actual_check":checked})
failed=[]
for path in [root/"expanded-resource/diagnostic/result.json",root/"expanded-resource/rooted/diagnostic/result.json"]:
    e=json.loads(path.read_text());assert e["status"]=="failed" and not e["changed_inputs_during_execution"]
    for key in ["stdout","stderr"]:assert seal(e[key]["path"])==e[key]
    for source in e["sources"]:
        saved=seal(Path(e["source_snapshot"])/source["path"])
        assert (saved["sha256"],saved["bytes"])==(source["sha256"],source["bytes"])
    failed.append(seal(path))
interrupted=json.loads((root/"expanded-resource/interruption.json").read_text())
assert interrupted["status"]=="interrupted_erroneous_diagnostic_before_any_over_cap_kernel_call"
assert interrupted["original_partial_inputs"]<60
record={"status":"all65_complete_original_constructions_retained_fixed_budget_preserved","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"actual_original_failed_consumer":seal(original_run),"actual_complete_collection_execution":seal(execution),"complete_original_source":seal(original),"complete_diagnostic_source":seal(helper),"only_source_differences":[{"original":old_header,"diagnostic":new_header},{"original":old_guard,"diagnostic":new_guard}],"complete_candidates":65,"in_budget_whole_original_checked_inputs_equal":60,"out_of_budget_construction_only_inputs":5,"first_out_of_budget_arity":60,"first_out_of_budget_terms":rows[60]["terms"],"all64_fields_expanded_terms":rows[64]["terms"],"original_producer_guard_unchanged":True,"kernel_executions_for_collected_bytes":0,"failed_observers":failed,"corrective_interruption":seal(root/"expanded-resource/interruption.json"),"rows":rows,"scope":"Full original expanded algorithm/fields/types/values/claims/proofs retained. The only diagnostic computation difference removes the before-encoding assertion to own complete raw candidates. All0..59 full bytes equal original actual checked inputs;60..64 remain outside the unchanged131072-term Go producer cap, without a claimed kernel rejection/acceptance, semantic negative or admitted Go/source/context/program/application proof. Frozen4096 declarations/2MiB/1024fuel and actual producer bound are not widened. Both original observer failures, corrective interruption and all complete original sources/raw outputs/partial inputs remain. Complete shared65-arity checked inputs and same-byte Go validation are separate requirements."}
p=root/"expanded-resource/verification.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:record[k] for k in ["status","complete_candidates","first_out_of_budget_arity","first_out_of_budget_terms","all64_fields_expanded_terms"]}))
