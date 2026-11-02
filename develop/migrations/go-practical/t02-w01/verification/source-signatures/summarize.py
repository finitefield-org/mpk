"""Summarize the actual terminal targeted executions and original-byte audit."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
epochs=[json.loads(p.read_bytes())for p in OUT.glob('*/inputs.json')if(p.parent/'run-checks.py').exists()];executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes())
 if'command'not in d:continue
 assert d['status']in {'passed','failed'}
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert len(epochs)==11 and len(executions)==24 and sum(r['status']=='failed'for r in executions)==3
checks={
 'Go-native':OUT/'binders-formatted/go-oracle/result.json','Go-vet':OUT/'binders-formatted/vet/result.json','Go-format':OUT/'binders-formatted/go-format/result.json',
 'Rust-signatures':OUT/'binders-paired/rust-selected/result.json','Rust-source-types':OUT/'formatted/rust-importer/result.json',
 'Clippy':OUT/'binders-formatted/clippy/result.json','Rust-format':OUT/'binders-formatted/rust-format/result.json',
}
for p in checks.values():assert json.loads(p.read_bytes())['status']=='passed'
audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert audit['status']=='passed'
record={
 'status':'scoped_checks_passed','basis_commit':'323fa842a02ef1d1160a06cd02fccb92fc2eb89d','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],
 'current_checks':{k:seal(p)for k,p in checks.items()},'actual_executions':executions,'source_epochs':len(epochs),'whole_source_preimages':sum(len(e['sources'])for e in epochs),'original_failed_checks':3,
 'Go_native_roots':1,'Rust_signature_roots':5,'affected_existing_source_type_roots':4,'actual_function_observations':31,'accepted_function_signatures':27,'explicit_valid_Go_outside_signature_domain':4,'actual_native_variable_definitions':259,'declared_bindings':250,'inferred_bindings_require_expression_typing':5,'explicit_valid_Go_outside_value_type_domain':3,'type_only_parameter_names':1,
 'original_archives':[seal(OUT/name/'manifest.json')for name in ['native-span-failure','native-original','native-binders','rust-partial','types-original','rust-original']],
 'complete_original_files':audit['actual_original_files_compared_literally'],'complete_original_bytes':audit['actual_original_bytes_compared_literally'],'literal_byte_mode_audit':seal(OUT/'original-byte-audit/result.json'),'derived_native_components':[seal(OUT/name/'result.json')for name in ['derived-native','derived-native-binders']],'SDK_current_exact':seal(OUT/'sdk-check/result.json'),
 'previous_source_type_basis':seal(OUT.parent/'source-types/verification.json'),'previous_scalar_basis':seal(OUT.parent/'source-constants/verification.json'),'previous_scope_basis':seal(OUT.parent/'source-scopes/verification.json'),'unchanged_parser_basis':seal(OUT.parent/'go-parse/verification.json'),'unchanged_lexer_basis':seal(OUT.parent/'go-lex/verification.json'),'unchanged_frontend_basis':seal(OUT.parent/'type-keys/corrected/analysis-inputs/captured-inputs.json'),'unchanged_ordinary_basis':seal(OUT.parent/'storage-lens/verification.json'),'unchanged_ordinary_paired_basis':seal(OUT.parent/'storage-lens/paired/result.json'),
 'original_failures':['Native physical expression test helper needed original _ = anchor spacing; full original capture and failed logs retained.','Rust format check after map-entry edit; only line wrapping changed.','Partial variable comparison initially conflated a bare function-type slot name with a lexical value binding. Native observer now retains and explicitly marks the original type-only name; no missing earlier result reconstructed.'],
 'whole_body_initializer_or_call_state_typechecking':False,'inferred_binding_or_runtime_expression_typing_complete':False,'ordinary_source_evaluation_or_carrier_proof':False,'complete_nominal_import_module_build_capture_context_authority':False,'source_program_or_registered_profile_authority':False,'full_U_Plain_weight_depth_counts_or_capture_budget_proved':False,'public_profile_activated':False,'R01_complete':False,'R06_complete':False,'W01_complete':False,'native_Linux_release_acceptance':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({key:record[key]for key in ['status','source_epochs','whole_source_preimages','complete_original_files','complete_original_bytes']}))
