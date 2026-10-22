"""Record terminal affected checks and exact correspondence to original bases."""
from pathlib import Path
import hashlib
import json
import runpy

OUT = Path(__file__).resolve().parent
RUN = OUT/'summary-original'
RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path':str(path), 'bytes':len(raw), 'sha256':hashlib.sha256(raw).hexdigest()}


epochs = []
for path in sorted(OUT.glob('*/inputs.json')):
    if not (path.parent/'result.json').is_file():
        continue
    value = json.loads(path.read_bytes())
    if isinstance(value, dict) and 'sources' in value:
        result = json.loads((path.parent/'result.json').read_bytes())
        assert result['status'] in {'passed','failed'}
        epochs.append((path, value, result))

comparisons = []
for suite, group, name in [
    ('source_overlays','source-overlays','rust-original'),
    ('source_selection','source-overlays','selection-original'),
    ('error_dispatch','source-overlays','dispatch-original'),
    ('source_domains','error-dispatch','domains-original'),
]:
    reader = OUT.parent/group/'read-originals.py'
    manifest, old = runpy.run_path(str(reader))['read'](name)
    current = OUT/'affected/observations'/suite
    rows = manifest['original_inputs']
    assert {p.name for p in current.iterdir()} == {Path(row['path']).name for row in rows}
    for row in rows:
        p = current/Path(row['path']).name
        assert p.read_bytes() == old[row['member']] and p.stat().st_mode & 0o777 == row['mode']
    comparisons.append({'suite':suite, 'basis':seal(OUT.parent/group/name/'manifest.json'), 'reader':seal(reader),
                        'files_compared_literally':len(rows), 'bytes_compared_literally':sum(len(v) for v in old.values())})

supplement = []
for suite in ['source_overlays','source_scopes','source_selection']:
    first = OUT/'affected/observations'/suite
    complete = OUT/'originalinventory/observations'/suite
    names = {p.name for p in first.iterdir()}
    assert names == {p.name for p in complete.iterdir()}
    for name in names:
        assert (first/name).read_bytes() == (complete/name).read_bytes()
        assert (first/name).stat().st_mode & 0o777 == (complete/name).stat().st_mode & 0o777
    supplement.append({'suite':suite, 'files_compared_literally':len(names), 'bytes_compared_literally':sum((first/n).stat().st_size for n in names)})

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
archive = json.loads((OUT/'native-original/manifest.json').read_bytes())
native = json.loads((OUT/'corpus.json').read_bytes())
current = {name:seal(OUT/epoch/'result.json') for name, epoch in {
    'new-Rust':'current', 'affected-domain-dispatch':'affected', 'affected-input-owners':'originalinventory',
    'native':'native', 'clippy':'lint', 'vet':'vet', 'rust-format':'format', 'go-format':'goformat',
}.items()}
assert all(json.loads(Path(row['path']).read_bytes())['status'] == 'passed' for row in current.values())
executions = [{'result':seal(p.parent/'result.json'), 'status':r['status'], 'command':r['command']} for p, _, r in epochs]
record = {
    'status':'scoped_checks_passed', 'basis_commit':'4b6ded3931d37aa5b9fec44244d097ab63e29f2b',
    'language_scope':['go','rust'], 'actual_summary_driver_prelaunch':seal(RUN/'driver.py'),
    'source_epochs':len(epochs), 'source_preimages':sum(len(e['sources']) for _,e,_ in epochs),
    'actual_executions':executions, 'current_checks':current,
    'new_Rust_roots':6, 'unique_affected_old_Rust_roots':20,
    'actual_native_Go_test_roots':1, 'native_original_functions':len(native['cases']),
    'native_original_headers':sum(len(r['native']['loops']) for r in native['cases']),
    'native_original_header_bindings':sum(len(l['bindings']) for r in native['cases'] for l in r['native']['loops']),
    'source_heads':70, 'source_type_scope_counterexamples':len(json.loads((OUT/'current/observations/counterexamples.json').read_bytes())),
    'initial_failures':[{'epoch':p.parent.name,'command':r['command'],'result':seal(p.parent/'result.json')} for p,_,r in epochs if r['status']=='failed'],
    'initial_affected_input_inventory_incomplete':{
        'result':seal(OUT/'affected/result.json'), 'test_exit_status':'passed',
        'missing_explicit_prelaunch_rows':['source-directives/corpus.json','source-scopes/native-components/*','t01-w04/contract-wire/catalog.json','t01-w04/contract-wire/expression-witnesses.json'],
        'original_basis_files_unchanged':True,
        'resolution':'Supplemental originalinventory run explicitly snapshots these inputs and reruns only overlay/scopes/selection owners. Domain and finite dispatch inventories were already complete; those passing tests are not repeated.',
        'passing_output_maps_equal_literally':supplement,
    },
    'old_output_correspondence':comparisons,
    'native_original_archive':seal(OUT/'native-original/manifest.json'),
    'native_original_files_compared_literally':archive['actual_files_compared_literally'],
    'native_original_bytes_compared_literally':archive['actual_bytes_compared_literally'],
    'SDK_basis_manifest':seal(sdk_path), 'SDK_original_files_unchanged':len(sdk), 'SDK_original_bytes_unchanged':total,
    'original_source_conversion':seal(OUT/'derived/result.json'),
    'previous_scoped_basis':seal(OUT.parent/'source-overlays/verification.json'),
    'previous_frozen_and_ordinary_basis':seal(OUT.parent/'source-overlays/original-byte-audit/result.json'),
    'Go_production_frontend_changed':False, 'frontend_rebuild_repeated':False,
    'partial_operation_definedness_or_runtime_proof':False,
    'canonical_scope_context_or_normalized_contract_authority':False,
    'whole_module_optional_callee_or_default_contracts':False,
    'complete_R01_R06_W01':False, 'public_profile_activated':False,
    'all_actual_validation_handles_terminal':True,
    'full_T02_gate':'Deferred to final W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed':False,
    'preexisting_historical_scope_scanner_findings_resolved':False,
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','source_epochs','source_preimages','new_Rust_roots','unique_affected_old_Rust_roots','native_original_headers','native_original_header_bindings']}))
