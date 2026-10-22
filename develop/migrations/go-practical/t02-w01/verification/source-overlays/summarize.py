"""Actual terminal original-source bindings; no full-T or typed-contract claim."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
current={'Rust-overlays':('complete','rust-overlays'),'Rust-selection':('affected','rust-selection'),'Rust-dispatch':('affected','rust-dispatch'),'Go-overlays':('prepared-native','go-overlays'),'Go-closure':('native-closure','go-closure'),'frontend-reproduction':('frontend','frontend'),'clippy':('checks','clippy'),'rust-format':('checks','rust-format'),'go-format':('checks','go-format'),'vet':('checks','vet')};checks={}
for name,(epoch,mode)in current.items():
 p=OUT/epoch/mode/'result.json';d=json.loads(p.read_bytes());assert d['status']=='passed'and d['whole_source_inputs_unchanged'];checks[name]=seal(p)
executions=[];epochs=[]
for p in sorted(OUT.glob('*/inputs.json')):
 if(p.parent/'run-checks.py').is_file():epochs.append(json.loads(p.read_bytes()))
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes())
 if'source_receipt'not in d:continue
 assert d['status']in {'passed','failed'};executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert sum(r['status']=='failed'for r in executions)==4
for name in ['original-byte-audit','sdk-check']:assert json.loads((OUT/name/'result.json').read_bytes())['status']=='passed'
old=json.loads((OUT.parent/'error-dispatch/verification.json').read_bytes());audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes())
record={'status':'scoped_checks_passed','basis_commit':'abf965a8a750d75828daa1f066eba0dafb7b4ae6','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],'current_checks':checks,'actual_executions':executions,'source_epochs':len(epochs),'source_preimages':sum(len(e['sources'])for e in epochs),'preserved_failed_executions':4,'failure_scope':'Two native fixture preparation errors (symlinked temp-root spelling, then writing after read-only namespace binding), one Clippy match-style diagnostic, and a syntax-witness coverage expectation that omitted nested bound. All original outputs/preimages are retained; raw fixtures and expected semantic facts are not weakened.','new_Rust_roots':5,'affected_old_Rust_roots':7,'original_source_sidecar_pairs':13,'closed_source_expression_heads':70,'input_slots':70,'old_native_source_closure_cases':51,'previous_dispatch_basis':seal(OUT.parent/'error-dispatch/verification.json'),'literal_byte_mode_audit':seal(OUT/'original-byte-audit/result.json'),'SDK_audit':seal(OUT/'sdk-check/result.json'),'original_source_packet_conversion':seal(OUT/'derived/result.json'),'unchanged_ordinary_paired_basis':old['unchanged_ordinary_paired_basis'],'frontend_inputs':190,'fresh_empty_cache_frontend_builds':2,'frontend_binary_bytes':audit['frontend_binary_bytes'],'frontend_binary_sha256':audit['frontend_binary_sha256'],'expression_types_and_logical_header_namespaces_validated':False,'whole_module_overlay_set_or_normalized_contract':False,'application_package_role_capture_authority':False,'complete_raw_capture_or_context_binding':False,'dynamic_entry_callee_loop_store_frame_or_runtime_program_proof':False,'public_profile_activated':False,'R01_R06_W01_completed':False,'full_T_gate_passed':False,'full_T_gate':'deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh'};(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_epochs','source_preimages','preserved_failed_executions','frontend_binary_sha256']}))
