"""Summarize only terminal affected checks and literal original evidence."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
BASE='59310cba83d5a20a292103153622dadec623d7fc'
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
epochs=[json.loads(p.read_bytes())for p in OUT.glob('*/inputs.json')if(p.parent/'run-checks.py').exists()]
executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes())
 if'source_receipt'not in d:continue
 assert d['status']in {'passed','failed'}
 for k in ['stdout','stderr']:assert seal(d[k]['path'])==d[k]
 executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert(len(epochs),sum(len(e['sources'])for e in epochs),len(executions),sum(e['status']=='failed'for e in executions))==(12,10229,16,3)
checks={'Rust-selection':OUT/'inventory/rust-values/result.json','Rust-Clippy':OUT/'fixed/clippy/result.json','Rust-format':OUT/'fixed/rust-format/result.json','Go-native':OUT/'final-native/go-oracle/result.json','Go-format':OUT/'fixed/go-format/result.json','Go-vet':OUT/'current/vet/result.json'}
for p in checks.values():assert json.loads(p.read_bytes())['status']=='passed'
audit=json.loads((OUT/'complete-byte-audit/result.json').read_bytes());assert audit['status']=='passed'
record={
 'status':'scoped_checks_passed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'source_languages':['Go','Rust'],'current_checks':{k:seal(p)for k,p in checks.items()},'actual_executions':executions,
 'source_epochs':12,'whole_source_preimages':10229,'original_failed_scoped_checks':3,
 'initial_failures':[
  'Two malformed directive placements in the original new fixture were rejected by actual original offline Go export compilation. Original source bytes and compiler diagnostics remain; only valid placement examples feed final positive/negative selected-source comparisons.',
  'The existing ErrorOnly test incorrectly checked every Error declaration after a new unrelated Error method was added. It now checks the actual non-type-check-only dependency; unused methods retain original type-check-only roles.',
  'The first independent Rust comparison used normalized comment length rather than full physical CRLF token end. The full original end is now retained. The initial assertion ran before writing both case27 actual rows; only the114 actually retained files and real diagnostic are claimed. Future results are saved before assertions.'
 ],
 'input_inventory_followup':'The prior33-case corpus was already immutable Git input at BASE, but missing as an explicit row in the first two Rust inventories. A final prelaunch inventory adds the complete exact Git-blob reference and reruns all four selected roots. Both174-file passing Rust output maps compare byte for byte; prior manifests/drivers remain unchanged.',
 'new_native_original_cases':51,'current_exact_graphs':24,'current_negative_rejections':27,'baseline_original_cases':33,'baseline_exact_graphs':15,'baseline_negative_rejections':18,
 'total_original_cases':84,'exact_positive_graphs':39,'independent_negative_rejections':45,'exact_directive_spans_and_paths':11,
 'physical_comment_bytes_include_CR':True,'line_directives_never_redirect_source_identity':True,'signature_only_Error_dependencies_checked':True,'unselected_functions_stay_type_check_only':True,
 'positive_call_edges':1032,'positive_dependency_edges':2142,'positive_declaration_rows':25995,'affected_Rust_roots':4,
 'original_helpers':512,'simultaneous_roots':70,'parameter_slots':70,
 'selection_reason':'Only source-selection logic and its comparison tests change. Check all affected new and original source-selection cases, changed-file formats, selected-package/test Clippy and Go vet. Existing lexer/parser/scopes/type/body/core/ordinary engines and Go production are byte-identical; no unrelated semantic tests or intermediate full/equivalent gate.',
 'original_archives':[seal(p)for p in sorted(OUT.glob('*/manifest.json'))],
 'complete_original_files':audit['actual_original_files_compared_literally'],'complete_original_bytes':audit['actual_original_bytes_compared_literally'],
 'literal_byte_mode_audit':seal(OUT/'complete-byte-audit/result.json'),'pre_inventory_literal_audit':seal(OUT/'original-byte-audit/result.json'),
 'passing_Rust_replays_byte_exact':True,'SDK_current_exact':seal(OUT/'sdk-check/result.json'),'pinned_parser_source':audit['pinned_go_parser_source'],
 'previous_source_selection_basis':seal(OUT.parent/'source-selection/verification.json'),'unchanged_frontend_basis':seal(OUT.parent/'source-selection/frontend-original/manifest.json'),
 'unchanged_ordinary_paired_basis':seal(OUT.parent/'storage-lens/paired/result.json'),'unchanged_frontend_inputs':190,'new_frontend_rebuild_required':False,
 'registered_capture_context_or_module_build_authority':False,'complete_v1_Root_Plain_closed_error_and_conversion_admission':False,
 'contract_dependent_verification_closure_complete':False,'source_runtime_execution_or_state_program_proof':False,'all56_foundation_and_all169_requirements_complete':False,
 'public_profile_activated':False,'R01_complete':False,'R06_complete':False,'W01_complete':False,'native_Linux_release_acceptance':False,
 'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_epochs','whole_source_preimages','complete_original_files','complete_original_bytes','total_original_cases','exact_directive_spans_and_paths']}))
