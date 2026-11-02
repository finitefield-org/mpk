"""Complete goal-type preservation, reusing the43 exact completed comparisons."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
original_reader=root/"compare-type-sharing.py"
ns={"__file__":str(original_reader.resolve())}
exec(compile(original_reader.read_text().split("rows=[]",1)[0],str(original_reader),"exec"),ns)
Tables,ExactNodes=ns["Tables"],ns["ExactNodes"]
base_fingerprints=ns["base_fingerprints"]
old_receipt=root/"type-sharing-analysis/result.json";old=json.loads(old_receipt.read_text())
assert old["matched_arities"]==list(range(43))
expanded_run=root/"final/tests/result.json";expanded=json.loads(expanded_run.read_text())
assert expanded["status"]=="failed" and not expanded["changed_inputs_during_execution"]
shared_run=root/"alias-consumers/initial/tests/result.json";shared=json.loads(shared_run.read_text())
assert shared["status"]=="passed" and not shared["changed_inputs_during_execution"]
collected_run=root/"expanded-resource/collected/diagnostic/result.json";collected=json.loads(collected_run.read_text())
assert collected["status"]=="passed" and not collected["changed_inputs_during_execution"]
source_audit=root/"shared-consumer-source.json";audit=json.loads(source_audit.read_text())
assert audit["status"]=="exactly_pair_type_syntax_and_output_label_differ"
for original in audit["complete_sources"]:assert seal(original["path"])==original
rows=[]
for arity in range(65):
    original=root/f"expanded-resource/collected/diagnostic/certificates/balanced-shape-{arity:02}.mpcert"
    alias=root/f"alias-consumers/initial/tests/certificates/alias-balanced-shape-{arity:02}.mpcert"
    metadata=original.with_suffix(".diagnostic.json");meta=json.loads(metadata.read_text())
    assert not meta["kernel_executed"]
    records=[seal(p) for p in [original,alias]]
    assert (records[0]["sha256"],records[0]["bytes"])==(meta["raw_sha256"],meta["bytes"])
    report=alias.with_suffix(".rust.json");alias_report=json.loads(report.read_text())
    assert alias_report["verdict"]=="accepted" and alias_report["axiom_count"]==0 and not alias_report["axiom_report"]["entries"]
    original_report=None
    if arity<60:
        checked=root/f"final/tests/certificates/balanced-shape-{arity:02}.mpcert"
        assert checked.read_bytes()==original.read_bytes()
        original_report=checked.with_suffix(".rust.json")
        assert json.loads(original_report.read_text())["verdict"]=="accepted"
        assert not meta["outside_frozen_Go_producer_term_budget"]
    else:
        assert meta["outside_frozen_Go_producer_term_budget"] and not meta["original_complete_checked_bytes_equal"]
    names={f"Mpk.Go.Practical.Test.Product.Pack{arity}",f"Mpk.Go.Practical.Test.Product.Reconstruct{arity}"}
    names.update(f"Mpk.Go.Practical.Test.Product.Frame{arity}Target{i}" for i in range(arity))
    reused=None
    if arity<43:
        previous=old["pairs"][arity]
        for key in ["expanded_certificate","shared_certificate","expanded_Rust_report","shared_Rust_report"]:
            assert seal(previous[key]["path"])==previous[key]
        assert Path(previous["expanded_certificate"]["path"]).read_bytes()==original.read_bytes()
        assert Path(previous["shared_certificate"]["path"]).read_bytes()==alias.read_bytes()
        assert set(previous["original_goal_names"])==names and previous["exact_interned_node_equality_not_only_hashes"]
        reused={"receipt":seal(old_receipt),"original_complete_comparison":previous}
        terms=[previous["expanded_terms"],previous["shared_terms"]]
    else:
        tables=[Tables(r) for r in records]
        for table in tables:
            fingerprints=table.fingerprints()
            for name,value in base_fingerprints.items():assert fingerprints[name]==value,name
        arena=ExactNodes();goals=[arena.load(t) for t in tables]
        assert set(goals[0])==set(goals[1])==names and goals[0]==goals[1],arity
        terms=[len(t.terms) for t in tables]
        # Cache belongs to this exact arena; release it between full input pairs.
        arena.lift.cache_clear()
        del arena,tables,goals
    assert [seal(p) for p in [original,alias]]==records
    rows.append({"arity":arity,"complete_original_construction":records[0],"complete_shared_checked_certificate":records[1],"original_construction_metadata":seal(metadata),"shared_actual_Rust_report":seal(report),"expanded_actual_Rust_report":seal(original_report) if original_report else None,"original_goal_names":sorted(names),"identical_goal_types":len(names),"exact_interned_node_equality_not_only_hashes":True,"all_original78_basis_identities_preserved":True,"terms":terms,"original_expanded_input_within_frozen_Go_producer_budget":arity<60,"reused_exact_completed_comparison":reused})
    print(json.dumps({"arity":arity,"identical_goal_types":len(names),"terms":terms,"prior_comparison_reused":reused is not None}),flush=True)
assert sum(r["identical_goal_types"] for r in rows)==2210
out=root/"complete-type-sharing-analysis";out.mkdir(exist_ok=False)
record={"status":"all65_complete_original_goal_type_sets_match","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"exact_graph_reader":seal(original_reader),"source_delta_audit":seal(source_audit),"prior_exact_comparison":seal(old_receipt),"original_failed_expanded_execution":seal(expanded_run),"actual_successful_shared_execution":seal(shared_run),"full_diagnostic_construction_execution":seal(collected_run),"all_arities":65,"identical_named_goal_types":2210,"exact_prior_pair_comparisons_reused":43,"new_exact_pair_comparisons":22,"expanded_checked_in_budget_arities":60,"expanded_out_of_budget_construction_only_arities":5,"pairs":rows,"only_expanded_name":"Mpk.Go.Practical.Product.Type","expansion":"Exact checked lambda A:Sort0. lambda B:Sort0. Pi tag:Bool. Bool.recType A B tag, capture-avoiding de Bruijn lifting and exact interned-node equality.","scope":"Derived statement-preservation comparison only. All65 shared full inputs have actual successful Rust production-core verdicts. Complete expanded0..59 bytes equal their original actual checked inputs. Expanded60..64 are full original-algorithm constructions retained after removing only the diagnostic pre-encoding guard; they remain outside the frozen Go producer budget, have no actual kernel verdict here and supply no semantic-negative or Go admission authority. Every source/certificate/JSON remains untouched. Existing43 exact completed comparisons are reused only after whole-byte equality. No field, independent carrier/value, binder, claim, premise, proof obligation or frozen operation requirement is dropped. Actual same-byte Go kernel results, final source integrity, registry/context/source/program/application authority remain separate requirements."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"arities":65,"identical_goal_types":2210,"reused":43,"new":22}))
