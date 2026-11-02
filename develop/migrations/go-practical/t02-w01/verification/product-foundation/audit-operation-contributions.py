"""Keep every original operation and source/effect obligation visible beside products."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
source=Path("develop/migrations/go-practical/t01-w04/foundation-limits/definitions.json")
original=json.loads(source.read_text())
operations=[op for family in original["operation_families"] for op in family["operations"]]
assert len(operations)==56 and {op["id"] for op in operations}=={f"GP-OP-{i:02}" for i in range(1,57)}
common=original["common_required_members"];assert len(common)==6
basis=root/"explicit-result-types/construction/certificates/product-foundation.mpcert"
report=json.loads(basis.with_suffix(".rust.json").read_text())
assert report["verdict"]=="accepted" and report["axiom_count"]==0 and report["declaration_count"]==78
contributions={
    "GP-OP-01":["UnitZero","UnitZeroValid","UnitValidUnique","ValidMake"],
    "GP-OP-10":["Make","LeftOfMake","RightOfMake","ValidMake"],
    "GP-OP-11":["Left","Right","PointwiseLeft","PointwiseRight"],
    "GP-OP-12":["SetLeft","SetRight","LeftOfSetLeft","RightOfSetLeft","LeftOfSetRight","RightOfSetRight","OverwriteLeft","OverwriteRight","SetLeftValid","SetRightValid"],
    "GP-OP-17":["UnitZero","UnitZeroValid","Make","LeftOfMake","RightOfMake","ValidMake"],
    "GP-OP-18":["Left","Right","Reconstruct","PointwiseOfProjections"],
    "GP-OP-21":["SetLeft","SetRight","OverwriteLeft","OverwriteRight","CommuteUpdates","SetLeftValid","SetRightValid"],
}
rows=[]
for op in sorted(operations,key=lambda op:op["id"]):
    declared={key:{"path":op[key],"exists":Path(op[key]).is_file()} for key in ["primary_production_executor","primary_test_owner","vc_owner","independent_validator"]}
    rows.append({"original_complete_operation_record":op,"named_production_artifacts":declared,"ordinary_product_value_prerequisites":["Mpk.Go.Practical.Product."+name for name in contributions.get(op["id"],[])],"source_bound_operation_completion":"unproven","all_original_common_proof_obligations_still_required":common,"scope":"Listed mathematical prerequisites do not supply source/type/context/nominal-field correspondence, actual evaluation/definedness/effects, same-invocation/post-state or frame authority for this source operation."})
record={"status":"all56_original_operations_preserved_source_completion_unproven","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"complete_frozen_recipe_inventory":seal(source),"ordinary_product_component":seal(basis),"ordinary_product_full_Rust_report":seal(basis.with_suffix(".rust.json")),"ordinary_product_logical_hashes":report["hashes"],"original_operation_count":56,"original_common_obligations_per_operation":6,"operations":rows,"fully_source_verified_operations_claimed":0,"remaining_application_demands":169,"public_profile_activated":False,"T02_W01_complete":False,"scope":"Product contribution/remaining-source audit, not a replacement for the complete numerical or full56-family library audit. All original operation records/signatures/evaluation/runtime/domain/permission/value-effect/frame clauses remain complete. File existence does not prove implementation; mathematical product laws do not promote the original169 application requests. Full bounded foundation/source/descriptor/private context/registry, retained policies and original immutable package/export replay remain required."}
p=root/"operation-contributions.json";assert not p.exists();p.write_text(json.dumps(record,ensure_ascii=False,indent=2)+"\n")
print(json.dumps({"original_operations":56,"common_obligations_each":6,"product_prerequisite_operations":len(contributions),"fully_source_verified_claimed":0}))
