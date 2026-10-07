from pathlib import Path
import hashlib,json

def load_indexed_pattern_result_provenance(base):
 b=Path(base);folder=b/'completed-indexed-pattern-result-provenance';h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();read=lambda p:json.loads(p.read_bytes())
 assert h(folder/'receipt.json')=='b705328b04ed9676e2d3a3f7006f484573d3f5c24beab597ba033ac6e0fd5ddb'
 assert h(folder/'file-manifest.json')=='51b10bfa16b9c4a8e3fdfd3fe5a8cd12687f71256a69d926f16aac2a5c590a16'
 for path,sha in read(folder/'file-manifest.json')['files'].items():assert h(folder/path)==sha,path
 assert h(b/'indexed-pattern-server-union-audit.json')=='6a2d59d1988e7a87f5a8760253131748481722f96e30abcde645e18f19f4793a'
 audit=read(b/'indexed-pattern-server-union-audit.json')
 assert audit['status']=='passed_independent_server_union_of42_actual_case_backend_results_and_exact_two_step_source_guard'
 assert audit['execution_host']=='root@162.43.92.154' and audit['actual_checker_stages']==42 and audit['distinct_complete_report_pairs']==21
 assert audit['case_backend_union_duplicates']==0 and audit['all_original_stage_fields_preserved'] and audit['all_reports_inputs_stderr_domain_module_declarations_exports_axioms_verified']
 assert audit['original_interrupted_status_bytes_unchanged'] and audit['provenance_receipt_raw_sha256']==h(folder/'receipt.json') and audit['provenance_file_manifest_raw_sha256']==h(folder/'file-manifest.json')
 receipt=read(folder/'receipt.json');assert len(receipt['stages'])==42
 assert receipt['historical_actual_checker_stages']==30 and receipt['server_indexed_go_actual_checker_stages']==6 and receipt['server_fixed_rust_actual_checker_stages']==6
 assert receipt['actual_new_server_checker_executions']==11 and receipt['server_fixed_rust_completed_exact_report_reuse']==1
 for row in receipt['stages']:
  assert row['exit_code']==0 and row['verdict']=='accepted'
  for path,key in [(row['report_file'],'report_sha256'),(row['stderr_file'],'stderr_sha256'),(row['input_file'],'input_sha256')]:assert h(folder/path)==row[key]
 return receipt,folder
