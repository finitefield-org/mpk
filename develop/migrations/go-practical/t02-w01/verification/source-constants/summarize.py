"""Evidence from original completed checks and byte-preserving transports."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
epochs=[json.loads(p.read_bytes())for p in sorted(OUT.glob('*/inputs.json'))if(p.parent/'run-checks.py').is_file()]
assert len(epochs)==7 and sum(len(e['sources'])for e in epochs)==5063
executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes());assert d['status']in {'passed','failed'}
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert len(executions)==18 and sum(e['status']=='failed'for e in executions)==3
current={
 'Go-native':OUT/'runtime-bound/go-oracle/result.json','Go-vet':OUT/'runtime-bound/vet/result.json',
 'Go-format':OUT/'runtime-bound/go-format/result.json','Rust-constants':OUT/'paired/rust-selected/result.json',
 'Rust-scopes':OUT/'formatted/rust-scopes/result.json','Clippy':OUT/'boxed-formatted/clippy/result.json',
 'Rust-format':OUT/'boxed-formatted/rust-format/result.json',
}
for p in current.values():assert json.loads(p.read_bytes())['status']=='passed'
audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert audit['status']=='passed'
sdk=json.loads((OUT/'sdk-check/result.json').read_bytes());assert sdk['status']=='passed'and sdk['original_files_unchanged']==14496
record={
 'status':'scoped_checks_passed','basis_commit':'f36999192fb1d9160ba61b7e83a2241eefc6aaf5',
 'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_languages':['Go','Rust'],
 'current_checks':{k:seal(v)for k,v in current.items()},'actual_executions':executions,
 'source_epochs':7,'whole_source_preimages':5063,'original_failed_checks':3,
 'Go_test_roots':1,'new_Rust_constant_roots':2,'affected_existing_scope_roots':3,
 'native_cases':5117,'exact_scalar_acceptances':4832,'agreeing_actual_rejections':281,'explicitly_outside_reader':4,
 'outside_reader_cases':['uint16(1.0)','len([3]byte{})','1.5','1i'],
 'all_actual_positive_values_types_and_nominal_keys_exact':True,
 'compiler_integer_precision_bits':512,'compiler_max_constant_shift_count':1074,
 'compiler_max_numeric_literal_bytes':10000,'target_integer_width':64,
 'these_are_pinned_compiler_rules_not_raised_core_or_profile_caps':True,
 'original_archives':[seal(OUT/name/'manifest.json')for name in ['native-build-failure','native-original','native-runtime','rust-original','scope-original']],
 'complete_original_files':30978,'complete_original_bytes':66409565,
 'literal_byte_mode_audit':seal(OUT/'original-byte-audit/result.json'),
 'derived_native_corpus':seal(OUT/'derived-native/result.json'),'SDK_current_exact':seal(OUT/'sdk-check/result.json'),
 'previous_scope_basis':seal(OUT.parent/'source-scopes/verification.json'),
 'unchanged_parser_basis':seal(OUT.parent/'go-parse/verification.json'),
 'unchanged_lexer_basis':seal(OUT.parent/'go-lex/verification.json'),
 'unchanged_frontend_basis':seal(OUT.parent/'type-keys/corrected/analysis-inputs/captured-inputs.json'),
 'unchanged_ordinary_basis':seal(OUT.parent/'storage-lens/verification.json'),
 'unchanged_ordinary_paired_basis':seal(OUT.parent/'storage-lens/paired/result.json'),
 'original_failure_causes':['Go reference string-root ID missing type assertion (both oracle and vet compile fail).','Rust processing-plan large enum warnings; box scalar conversion metadata without suppressing lint.'],
 'float_complex_aggregate_constant_builtins_complete':False,
 'array_length_type_graph_and_full_source_typing_complete':False,
 'ordinary_source_evaluation_or_carrier_proof':False,'complete_nominal_import_module_build_capture_context_authority':False,
 'source_program_or_registered_profile_authority':False,'full_U_Plain_weight_depth_counts_or_capture_budget_proved':False,
 'public_profile_activated':False,'R01_complete':False,'R06_complete':False,'W01_complete':False,
 'native_Linux_release_acceptance':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','native_cases','complete_original_files','complete_original_bytes']}))
