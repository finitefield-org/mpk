"""Summarize actual retained executions; no missing observations are recreated."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
epochs=[json.loads(p.read_bytes())for p in sorted(OUT.glob('*/inputs.json'))if(p.parent/'run-checks.py').is_file()]
assert len(epochs)==9 and sum(len(e['sources'])for e in epochs)==6510
executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes());assert d['status']in {'passed','failed'}
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'command':d['command'],'status':d['status']})
assert len(executions)==21 and sum(e['status']=='failed'for e in executions)==4
current={
 'Go-original':OUT/'dot-corrected/go-oracle/result.json',
 'Rust-original':OUT/'bound-receiver/rust-paired/result.json',
 'Rust-blockers':OUT/'bound-receiver/rust-blockers/result.json',
 'Clippy':OUT/'bound-receiver/clippy/result.json',
 'Rust-format':OUT/'bound-receiver/rust-format/result.json',
 'Go-format':OUT/'paired/go-format/result.json',
 'Go-vet':OUT/'paired/vet/result.json',
}
for p in current.values():assert json.loads(p.read_bytes())['status']=='passed'
partial=json.loads((OUT/'paired/rust-selected/result.json').read_bytes());assert partial['status']=='failed'
assert 'test rejects_duplicate_and_invalid_short_bindings ... ok' in (OUT/'paired/rust-selected/stdout.log').read_text()
native=json.loads((OUT/'native-components/uses.json').read_bytes());assert len(native['uses'])==117
audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert audit['status']=='passed'
record={
 'status':'scoped_checks_passed','basis_commit':'00275cd124d4951edfa88b57709718bfbc798df1',
 'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],
 'current_checks':{k:seal(p)for k,p in current.items()},'retained_unaffected_duplicate_builder_test':seal(OUT/'paired/rust-selected/result.json'),
 'source_epochs':len(epochs),'whole_source_preimages':6510,'actual_executions':executions,'original_failed_checks':4,
 'Go_test_roots':1,'Rust_test_roots':3,'actual_native_lexical_uses':117,'whole_original_component_source_objects':7,
 'definition_source_class_and_physical_offset_exact':True,
 'complete_original_files':audit['actual_original_files_compared_literally'],
 'complete_original_bytes':audit['actual_original_bytes_compared_literally'],
 'native_archives':[seal(OUT/name/'manifest.json')for name in ['native-count-failure','native-dot-conflict','native-original']],
 'original_Rust_outputs':seal(OUT/'rust-originals/manifest.json'),'original_byte_and_mode_audit':seal(OUT/'original-byte-audit/result.json'),
 'derived_native_inputs':seal(OUT/'derived-native/result.json'),
 'previous_source_declaration_basis':seal(OUT.parent/'source-declarations/verification.json'),
 'unchanged_parser_basis':seal(OUT.parent/'go-parse/verification.json'),
 'unchanged_lexer_basis':seal(OUT.parent/'go-lex/verification.json'),
 'unchanged_frontend_basis':seal(OUT.parent/'type-keys/corrected/analysis-inputs/captured-inputs.json'),
 'unchanged_ordinary_basis':seal(OUT.parent/'storage-lens/verification.json'),
 'unchanged_ordinary_paired_basis':seal(OUT.parent/'storage-lens/paired/result.json'),
 'original_failures':[
  'Rust helper visibility: pinned upper predicate was private; add internal forwarding helper without changing existing validation.',
  'First native test counted83 observations below its expected coverage threshold. Original component/source outputs remain; missing use map is not reconstructed. Expanded actual source tests produce117 observations.',
  'Expanded native package had a real Unicode dot-import versus package declaration conflict. Original source/diagnostic retained; distinct exported name fixes positive fixture.',
  'Initial Rust comparison reached receiver type-parameter binder use90 and returned no binding. Bind the original receiver declaration occurrence to itself without widening other identifier visibility.'
 ],
 'original_input_review_failure':{
  'evidence':'initial-review-failure/result.json',
  'cause':'Root production Go file count53 was incorrectly compared with whole frontend input closure189. Exact root-member set now comes from the same189-input closure; all inputs remain checked.',
  'not_product_scope_failure':True,
  'second_evidence':'second-review-failure/result.json',
  'second_cause':'Obsolete checkpoint metadata key in review; correct to preparatory_independent_lexical_scopes_checkpoint.',
 },
 'constant_evaluation_or_type_expression_denotation_proved':False,
 'source_typing_or_registered_context_or_program_authority':False,
 'complete_nominal_module_build_version_capture_identity_proved':False,
 'ordinary_source_carrier_storage_permission_lifetime_state_program_proved':False,
 'full_global_U_Plain_weight_depth_counts_or_capture_budget_proved':False,
 'public_profile_activated':False,'R01_complete':False,'R06_complete':False,'W01_complete':False,
 'native_Linux_release_acceptance':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','complete_original_files','complete_original_bytes','actual_native_lexical_uses']}))
