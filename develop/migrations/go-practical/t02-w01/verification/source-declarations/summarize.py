"""Summarize actual saved check/output receipts, without recreating evidence."""
from pathlib import Path
import datetime, hashlib, json, re

OUT = Path(__file__).resolve().parent


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


executions = []
for path in sorted(OUT.glob('*/*/result.json')):
    d = json.loads(path.read_bytes())
    assert d['status'] == 'passed' and d['exit_code'] == 0
    for key in ['stdout', 'stderr']:
        assert seal(d[key]['path']) == d[key]
    executions.append({'result': seal(path), 'status': d['status'], 'command': d['command']})
assert len(executions) == 4
epochs = [json.loads(p.read_bytes()) for p in sorted(OUT.glob('*/inputs.json'))]
assert len(epochs) == 2 and sum(len(e['sources']) for e in epochs) == 1488
current = {mode: seal(OUT/'formatted'/mode/'result.json')
           for mode in ['rust-selected', 'clippy', 'rust-format']}
stdout = (OUT/'formatted/rust-selected/stdout.log').read_text()
roots = re.findall(r'^test (\w+) \.\.\. ok$', stdout, re.M)
assert len(roots) == 8 and '8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in stdout
rejections = dict(re.findall(r'^([^ :]+): actual component rejection (\w+)$', stdout, re.M))
assert len(rejections) == 38
new = ['forged-name','inside-name','short-type-end','identifier-only','including-type-keyword',
       'wrong-package-clause','alias-marked-definition','definition-marked-alias','generic-definition',
       'generic-alias','value-declaration-as-type','malformed-unselected-body',
       'nonutf8-unselected-bytes','second-package-clause','unselected-file-wrong-package',
       'unselected-file-malformed-body']
assert all(rejections[n] == 'Source' for n in new)
record = {
    'status': 'scoped_checks_passed', 'basis_commit': '900c08645d0d54666d260d2f2914f680c8e09050',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'source_languages': ['Go','Rust'], 'current_checks': current, 'actual_check_executions': executions,
    'source_epochs': len(epochs), 'whole_source_preimages': sum(len(e['sources']) for e in epochs),
    'affected_Rust_roots': roots, 'actual_Go_component_streams': 5,
    'actual_native_bound_declarations_per_stream': [5,2,1,1,0],
    'constructed_syntax_membership_acceptances': 6, 'new_rehashed_source_rejections': new,
    'all_printed_actual_component_rejections': rejections,
    'all_complete_emitted_original_files': 236, 'all_complete_emitted_original_bytes': 842726,
    'original_retention': seal(OUT/'original-retention/result.json'),
    'complete_original_output_manifest': seal(OUT/'rust-originals/manifest.json'),
    'retained_native_component_basis': seal(OUT.parent/'type-keys/verification.json'),
    'unchanged_parser_basis': seal(OUT.parent/'go-parse/verification.json'),
    'unchanged_lexer_basis': seal(OUT.parent/'go-lex/verification.json'),
    'unchanged_frontend_basis': seal(OUT.parent/'type-keys/corrected/analysis-inputs/captured-inputs.json'),
    'unchanged_ordinary_basis': seal(OUT.parent/'storage-lens/verification.json'),
    'unchanged_ordinary_paired_basis': seal(OUT.parent/'storage-lens/paired/result.json'),
    'unchanged_frontend_inputs': 189, 'unchanged_frozen_design_inputs': 255,
    'exact_original_declaration_syntax_membership': True,
    'type_expression_to_candidate_graph_denotation_proved': False,
    'source_typing_or_registered_context_or_program_authority': False,
    'full_global_U_Plain_weight_depth_counts_or_capture_budget_proved': False,
    'ordinary_source_carrier_or_storage_lifetime_state_program_correspondence': False,
    'public_profile_activated': False, 'R01_complete': False, 'R06_complete': False, 'W01_complete': False,
    'native_Linux_release_acceptance': False,
    'original_input_review_failure': {
        'evidence':'initial-review-failure/result.json',
        'cause':'Report link checked before report creation; actual driver, output and prospective report retained. Report is initialized before link checks.',
        'not_product_importer_failure':True,
    },
    'full_T_gate': 'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
}
(OUT/'verification.json').write_text(json.dumps(record, indent=2)+'\n')
print(json.dumps({'status': record['status'], 'roots':len(roots), 'new_source_rejections':len(new)}))
