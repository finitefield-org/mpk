"""Read-only exact source-delta audit of all original product consumer obligations."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
old=Path("crates/mpk-vc/tests/support/go_product_foundation_shapes.rs")
new=Path("crates/mpk-vc/tests/support/go_product_alias_shapes.rs")
original=old.read_text();shared=new.read_text();before=[seal(old),seal(new)]
old_pair='''fn rec_type(a: E, b: E, tag: E) -> E {
    app(c("Std.Bool.recType"), vec![a, b, tag])
}
fn pair(a: E, b: E) -> E {
    pi("_tag", c("Std.Bool"), rec_type(a, b, v("_tag")))
}
'''
new_pair='''fn pair(a: E, b: E) -> E {
    // This checked type definition is not applied as a neutral function here.
    // Projections use the library's explicitly typed Left/Right interfaces.
    pc("Type", vec![a, b])
}
'''
old_label='inspect(&format!("balanced-shape-{arity:02}"), &bytes, true);'
new_label='inspect(&format!("alias-balanced-shape-{arity:02}"), &bytes, true);'
assert original.count(old_pair)==original.count(old_label)==1
assert shared==original.replace(old_pair,new_pair,1).replace(old_label,new_label,1)
assert "for arity in 0..=64 {" in original and "(2080, 2080, 2080, 87360)" in original
assert [seal(old),seal(new)]==before
executions=[]
for run,path in [("expanded",root/"final/tests/result.json"),("shared",root/"alias-consumers/initial/tests/result.json")]:
    d=json.loads(path.read_text())
    for source in before:
        if source in d["sources"]:
            saved=seal(Path(d["source_snapshot"])/source["path"])
            assert (saved["sha256"],saved["bytes"])==(source["sha256"],source["bytes"])
            executions.append({"run":run,"complete_original_snapshot":saved,"observed_execution_status":d["status"]})
assert len(executions)==2
record={"status":"exactly_pair_type_syntax_and_output_label_differ","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"complete_sources":before,"literal_differences":[{"original":old_pair,"shared":new_pair},{"original":old_label,"shared":new_label}],"original_execution_source_snapshots":executions,"all65_arity_loop_and_every_constructor_projection_replace_claim_proof_binder_lowering_and_cap_check_elsewhere_byte_identical":True,"leaf_obligations":{"pack":2080,"reconstruct":2080,"updated":2080,"unchanged":87360,"total":93600},"scope":"Independent exact source-delta observation only. Both full source files and their original execution snapshots remain untouched. The whole matrix including every arbitrary independent type/value/field, actual projection/update and complete original cap check is text-identical except the two displayed literal substitutions. The sole type-name expansion is separately compared against its exact checked body in derived goal-type graphs. No source replacement/normalization, dropped field/claim or missing execution is accepted as proof/source/context/application authority; actual complete production-core outcomes remain separately required."}
p=root/"shared-consumer-source.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"arities":65,"leaf_obligations":93600,"original_source_snapshots":2}))
