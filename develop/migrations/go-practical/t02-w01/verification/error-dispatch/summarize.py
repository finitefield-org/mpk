"""Summarize actual terminal source checks; whole T and runtime proof stay open."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
current={'Rust-dispatch':('cached','rust-dispatch'),'Rust-selection':('cached','rust-selection'),'Rust-domains':('cached','rust-domains'),'Rust-boxings':('cached','rust-boxings'),'Rust-methods':('affected','rust-methods'),'Rust-bodies':('affected','rust-bodies'),'Go-dispatch':('native-corrected','go-dispatch'),'Go-closure':('native-closure','go-closure'),'frontend-reproduction':('frontend','frontend'),'clippy':('cached','clippy'),'rust-format':('cached','rust-format'),'go-format':('checks','go-format'),'vet':('checks','vet')};checks={}
for name,(epoch,mode)in current.items():
 p=OUT/epoch/mode/'result.json';d=json.loads(p.read_bytes());assert d['status']=='passed'and d['whole_source_inputs_unchanged'];checks[name]=seal(p)
executions=[];epochs=[]
for p in sorted(OUT.glob('*/inputs.json')):
 if(p.parent/'run-checks.py').is_file():epochs.append(json.loads(p.read_bytes()))
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes())
 if'source_receipt'not in d:continue
 assert d['status']in {'passed','failed'};executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert sum(r['status']=='failed'for r in executions)==3
for name in ['original-byte-audit','sdk-check']:assert json.loads((OUT/name/'result.json').read_bytes())['status']=='passed'
old=json.loads((OUT.parent/'source-domains/verification.json').read_bytes());audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes())
record={'status':'scoped_checks_passed','basis_commit':'34b79b08aa7b8b2e3d0029279aa2d241f8593de1','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],'current_checks':checks,'actual_executions':executions,'source_epochs':len(epochs),'source_preimages':sum(len(e['sources'])for e in epochs),'preserved_failed_executions':3,'failure_scope':'Initial Rust move-after-borrow error, native comparison of the universe error as a captured nominal declaration, and the deliberately interrupted sampled partial suite for a concrete repeated-work performance repair. The unchanged full fixture passes after the repair.','new_Rust_roots':3,'affected_old_Rust_roots':18,'original_native_cases':23,'positive_exact_closure_comparisons':19,'negative_target_rejections':4,'explicit_case_control':70,'successive_case_growth_rounds':70,'old_native_source_closure_cases':51,'old_method_exact_comparison_promotions':3,'previous_domain_basis':seal(OUT.parent/'source-domains/verification.json'),'literal_byte_mode_audit':seal(OUT/'original-byte-audit/result.json'),'SDK_audit':seal(OUT/'sdk-check/result.json'),'original_source_packet_conversion':seal(OUT/'derived/result.json'),'actual_profile_observation':seal(OUT/'performance-baseline/observation.json'),'unchanged_ordinary_paired_basis':old['unchanged_ordinary_paired_basis'],'frontend_inputs':190,'fresh_empty_cache_frontend_builds':2,'frontend_binary_bytes':audit['frontend_binary_bytes'],'frontend_binary_sha256':audit['frontend_binary_sha256'],'application_package_role_capture_authority':False,'complete_raw_overlay_or_context_binding':False,'dynamic_receiver_nil_callee_contract_or_runtime_program_proof':False,'public_profile_activated':False,'R01_R06_W01_completed':False,'full_T_gate_passed':False,'full_T_gate':'deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh'};(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_epochs','source_preimages','preserved_failed_executions','frontend_binary_sha256']}))
