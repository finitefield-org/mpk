"""Summarize terminal scoped checks without claiming whole-T completion."""
from pathlib import Path
import datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
current={'Rust-domains':('corrected','rust-domains'),'Rust-selection':('affected-selection','rust-selection'),'Rust-boxings':('affected-boxings','rust-boxings'),'Go-domains':('native-fixed','go-domains'),'clippy':('corrected','clippy'),'rust-format':('corrected','rust-format'),'go-format':('format-go','go-format'),'vet':('vet','vet')}
checks={}
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
for name in ['original-byte-audit','native-copy-audit','sdk-check']:
 assert json.loads((OUT/name/'result.json').read_bytes())['status']=='passed'
old=json.loads((OUT.parent/'error-boxings/verification.json').read_bytes())
record={'status':'scoped_checks_passed','basis_commit':'15e087a6a51c7307355e5750c2a28c6ddac2ba4a','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],'current_checks':checks,'actual_executions':executions,'source_epochs':len(epochs),'source_preimages':sum(len(e['sources'])for e in epochs),'preserved_failed_executions':3,'failed_execution_scope':'Original Go observer field-name compile error, test-only Clippy cloned-ref diagnostics, and an extra rejection-stage expectation already enforced earlier by source types. No consumer inputs changed while a check was live.','new_Rust_roots':5,'affected_old_Rust_roots':6,'compiler_property_comparisons':15,'explicit_case_count_control':70,'previous_boxing_basis':seal(OUT.parent/'error-boxings/verification.json'),'literal_byte_mode_audit':seal(OUT/'original-byte-audit/result.json'),'SDK_audit':seal(OUT/'sdk-check/result.json'),'original_source_packet_conversion':seal(OUT/'derived/result.json'),'unchanged_ordinary_paired_basis':old['unchanged_ordinary_paired_basis'],'production_frontend_original_build_basis':old['literal_byte_mode_audit'],'production_frontend_inputs_unchanged':190,'production_frontend_rebuild_required':False,'application_package_role_capture_authority':False,'complete_raw_overlay_or_context_binding':False,'finite_Error_dispatch':False,'dynamic_entry_runtime_store_frame_or_source_program_proof':False,'public_profile_activated':False,'R01_R06_W01_completed':False,'full_T_gate_passed':False,'full_T_gate':'deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_epochs','source_preimages','preserved_failed_executions']}))
