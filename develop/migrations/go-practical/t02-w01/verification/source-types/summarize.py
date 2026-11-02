"""Record scoped results from original completed executions."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
epochs=[json.loads(p.read_bytes())for p in sorted(OUT.glob('*/inputs.json'))if(p.parent/'run-checks.py').is_file()]
assert len(epochs)==7 and sum(len(e['sources'])for e in epochs)==5288
executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 if p.parent.name not in {'go-format-write','rust-format-write','go-oracle','vet','go-format','rust-format','rust-importer','rust-selected','clippy'}:continue
 d=json.loads(p.read_bytes());assert d['status']in {'passed','failed'}
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert len(executions)==17 and sum(e['status']=='failed'for e in executions)==3
current={
 'Go-native':OUT/'formatted/go-oracle/result.json','Go-vet':OUT/'formatted/vet/result.json','Go-format':OUT/'formatted/go-format/result.json',
 'Rust-source-types':OUT/'context-formatted/rust-selected/result.json','Rust-importer':OUT/'typed/rust-importer/result.json',
 'Clippy':OUT/'context-formatted/clippy/result.json','Rust-format':OUT/'context-formatted/rust-format/result.json',
}
for p in current.values():assert json.loads(p.read_bytes())['status']=='passed'
audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert audit['status']=='passed'
sdk=json.loads((OUT/'sdk-check/result.json').read_bytes());assert sdk['status']=='passed'and sdk['original_files_unchanged']==14496
record={
 'status':'scoped_checks_passed','basis_commit':'75eb029ed8b073c2e5597d8d478927bbcd47d6a8','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],
 'current_checks':{k:seal(v)for k,v in current.items()},'actual_executions':executions,'source_epochs':7,'whole_source_preimages':5288,'original_failed_checks':3,
 'Go_test_roots':1,'new_Rust_source_type_roots':4,'affected_existing_importer_roots':8,'actual_native_streams':7,'retained_prior_native_streams':5,'paired_streams':12,'fully_rehashed_body_controls':7,'context_mismatch_controls':2,
 'actual_original_source_type_denotation_domain':['canonical scalar builtins','nominal declarations','transparent source aliases','bounded arrays with scalar integer constant lengths','slices','ordered bounded structs with grouped and embedded alias fields and exact raw tags'],
 'semantic_roots_and_underlying_erase_intermediate_nominal_spelling':True,'source_declaration_and_alias_origins_retained_separately':True,
 'observed_type_streams_may_omit_unused_import_package_metadata':True,'additional_scope_context_requires_exact_owned_source_objects_bytes_and_package_identities':True,
 'original_archives':[seal(OUT/name/'manifest.json')for name in ['native-original','importer-build-failure','importer-original','rust-partial','rust-original']],
 'complete_original_files':11339,'complete_original_bytes':88718325,'literal_byte_mode_audit':seal(OUT/'original-byte-audit/result.json'),'derived_native_components':seal(OUT/'derived-native/result.json'),'SDK_current_exact':seal(OUT/'sdk-check/result.json'),
 'previous_scalar_basis':seal(OUT.parent/'source-constants/verification.json'),'previous_scope_basis':seal(OUT.parent/'source-scopes/verification.json'),
 'unchanged_parser_basis':seal(OUT.parent/'go-parse/verification.json'),'unchanged_lexer_basis':seal(OUT.parent/'go-lex/verification.json'),'unchanged_frontend_basis':seal(OUT.parent/'type-keys/corrected/analysis-inputs/captured-inputs.json'),
 'unchanged_ordinary_basis':seal(OUT.parent/'storage-lens/verification.json'),'unchanged_ordinary_paired_basis':seal(OUT.parent/'storage-lens/paired/result.json'),
 'original_failure_causes':['Rust underlying map inference needs explicit owned String value type (original Clippy and importer compile failures).','Paired partial: unused import package absent in observed type metadata; require actual larger owned scope context. Fully rehashed embedded-marker control collided with an existing equivalent structural key; deduplicate by the new identity.'],
 'full_source_typechecking_including_float_or_aggregate_constant_lengths_complete':False,'anonymous_candidate_roots_certified_without_source_locator':False,'ordinary_source_evaluation_or_carrier_proof':False,'complete_nominal_import_module_build_capture_context_authority':False,'source_program_or_registered_profile_authority':False,'full_U_Plain_weight_depth_counts_or_capture_budget_proved':False,'public_profile_activated':False,'R01_complete':False,'R06_complete':False,'W01_complete':False,'native_Linux_release_acceptance':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','paired_streams','complete_original_files','complete_original_bytes']}))
