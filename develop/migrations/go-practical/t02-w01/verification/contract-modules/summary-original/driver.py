"""Summarize terminal local checks and literal original-basis correspondence."""
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).resolve().parent
RUN = OUT/'summary-original'
RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


epochs = []
for path in sorted(OUT.glob('*/inputs.json')):
    if not (path.parent/'result.json').is_file():
        continue
    inputs = json.loads(path.read_bytes())
    if isinstance(inputs,dict) and 'sources' in inputs:
        result = json.loads((path.parent/'result.json').read_bytes())
        assert result['status'] in {'passed','failed'}
        epochs.append((path,inputs,result))
old_maps = []
basis = OUT.parent/'contract-expressions'
for suite,previous in [
    ('contract_expressions',basis/'current/observations'),
    ('source_overlays',basis/'originalinventory/observations/source_overlays'),
    ('source_scopes',basis/'originalinventory/observations/source_scopes'),
    ('source_selection',basis/'originalinventory/observations/source_selection'),
    ('error_dispatch',basis/'affected/observations/error_dispatch'),
    ('source_domains',basis/'affected/observations/source_domains'),
]:
    current = OUT/'affected/observations'/suite
    names = {p.name for p in current.iterdir()}
    assert names == {p.name for p in previous.iterdir()},suite
    for name in names:
        assert (current/name).read_bytes() == (previous/name).read_bytes(),(suite,name)
        assert (current/name).stat().st_mode & 0o777 == (previous/name).stat().st_mode & 0o777
    old_maps.append({'suite':suite,'previous_directory':str(previous),'files_compared_literally':len(names),
                     'bytes_compared_literally':sum((current/n).stat().st_size for n in names)})
sdk_path = OUT.parent/'array-equality/native-go/sdk-inputs.json'
sdk = json.loads(sdk_path.read_bytes())
total = 0
for row in sdk:
    raw = Path(row['path']).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
    if 'mode' in row:
        assert Path(row['path']).stat().st_mode & 0o777 == row['mode']
    total += len(raw)
assert (len(sdk),total) == (14496,202722007)
archives = [json.loads((OUT/name/'manifest.json').read_bytes()) for name in ['native-original','native-receivers-original']]
assert all(a['status'] == 'passed' for a in archives)
current = {name:seal(OUT/epoch/'result.json') for name,epoch in {
    'new-Rust':'current','affected-Rust':'affected','native':'native-receivers','clippy':'lint',
    'vet':'vet','rust-format':'format','go-format':'goformat',
}.items()}
assert all(json.loads(Path(r['path']).read_bytes())['status'] == 'passed' for r in current.values())
executions = [{'result':seal(p.parent/'result.json'),'status':r['status'],'command':r['command']} for p,_,r in epochs]
record = {
    'status':'scoped_checks_passed','basis_commit':'7e5748a5d060555197969a379b90d51f1eed0b1d','language_scope':['go','rust'],
    'actual_summary_driver_prelaunch':seal(RUN/'driver.py'),'source_epochs':len(epochs),
    'source_preimages':sum(len(e['sources']) for _,e,_ in epochs),'actual_executions':executions,'current_checks':current,
    'new_Rust_roots':6,'unique_affected_old_Rust_roots':31,'new_native_Go_root_executions':2,
    'original_native_cases':16,'current_native_cases':17,'current_positive_cases':12,'current_profile_rejections':5,
    'annotation_growth_cases':70,'annotation_growth_body_functions':141,'original_helpers':512,
    'helper_closure_functions':513,'exact_function_expr_nodes':1024,'exact_module_expr_nodes':8192,
    'exact_raw_sidecar_bytes':8388608,'old_output_correspondence':old_maps,
    'initial_failures':[{'epoch':p.parent.name,'result':seal(p.parent/'result.json')} for p,_,r in epochs if r['status'] == 'failed'],
    'native_original_archives':[seal(OUT/name/'manifest.json') for name in ['native-original','native-receivers-original']],
    'native_original_files_compared_literally':sum(a['actual_files_compared_literally'] for a in archives),
    'native_original_bytes_compared_literally':sum(a['actual_bytes_compared_literally'] for a in archives),
    'original_source_conversions':[seal(OUT/name/'result.json') for name in ['derived','derived-receivers']],
    'SDK_basis_manifest':seal(sdk_path),'SDK_original_files_unchanged':len(sdk),'SDK_original_bytes_unchanged':total,
    'previous_expression_basis':seal(basis/'verification.json'),
    'previous_frozen_and_ordinary_basis':seal(OUT.parent/'source-overlays/original-byte-audit/result.json'),
    'Go_production_frontend_changed':False,'frontend_rebuild_repeated':False,
    'canonical_source_scope_set_or_context_wire':False,'program_runtime_or_definedness_proof':False,
    'application_stdlib_role_authority':False,'complete_R01_R06_W01':False,'public_profile_activated':False,
    'all_actual_validation_handles_terminal':True,'full_T02_gate':'Deferred to final W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed':False,'preexisting_historical_scope_scanner_findings_resolved':False,
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','source_epochs','source_preimages','new_Rust_roots','unique_affected_old_Rust_roots','native_original_files_compared_literally','native_original_bytes_compared_literally']}))
