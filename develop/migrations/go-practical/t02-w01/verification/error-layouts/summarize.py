"""Audit terminal targeted evidence; never turn a partial matrix into success."""
from pathlib import Path
import collections
import datetime
import hashlib
import json

ROOT=Path(__file__).resolve().parent


def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


def same(row):
    assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']},row['path']


def execution(epoch,passed=True):
    path=ROOT/epoch/'result.json'
    d=json.loads(path.read_bytes())
    assert d['status']==('passed' if passed else 'failed')
    assert d['whole_source_inputs_unchanged'] or d['formatter_mutation_expected']
    for key in ['stdout','stderr','source_receipt','actual_driver_prelaunch']:
        same(d[key])
    return path,d


current={}
for name in ['corrected-controls','scoped-lint','scoped-format','replay']:
    p,d=execution(name)
    current[name]=seal(p)
inline_path,inline=execution('inline')
original_path,original=execution('final',passed=False)
original_out=(ROOT/'final/stdout.log').read_text()
assert 'all_balanced_layout_sizes_zero_through_seventy_keep_canonical_shapes ...' in original_out
assert 'exact_layout_ownership_and_unchanged_artifact_resource_boundaries ...' in original_out
assert '2 passed; 1 failed' in original_out
assert 'reject_wrong_layout_nil_field_order_or_incomplete_adapters' in original_out
bad=ROOT/'final/certificates/empty-carrier-extra-slot.mpcert'
bad_report=bad.with_suffix('.rust.json')
assert json.loads(bad_report.read_bytes())['verdict']=='accepted'
client_bare=ROOT/'carrier-client/certificates/unreferenced-carrier-is-only-math.mpcert'
assert bad.read_bytes()==client_bare.read_bytes()
assert bad_report.read_bytes()==client_bare.with_suffix('.rust.json').read_bytes()
replay=ROOT/'replay/literal-current-original-map.json'
replayed=json.loads(replay.read_bytes())
assert len(replayed)==71 and {r['case_count'] for r in replayed}==set(range(71))
for row in replayed:
    assert row['complete_original_and_current_bytes_literal_equal']
    path=ROOT/f"final/certificates/matrix-N{row['case_count']}.mpcert"
    raw=path.read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
    assert hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()==row['certificate_hash']

matrix_path=ROOT/'incremental-go/result.json'
matrix=json.loads(matrix_path.read_bytes())
assert matrix['status']=='layout_matrix_passed' and matrix['distinct_full_streams']==71
assert {r['case_count'] for r in matrix['cases']}==set(range(71))
for row in matrix['literal_matrix_correspondence']:
    assert row['whole_original_bytes_and_reports_equal']
    for key in ['matrix_stream','matrix_report','original_checked_stream']:
        same(row[key])
    assert Path(row['matrix_stream']['path']).read_bytes()==Path(row['original_checked_stream']['path']).read_bytes()

pairs=[]
for epoch in ['incremental-go','carrier-client-paired','controls-paired','ownership-paired']:
    path=ROOT/epoch/'result.json'
    d=json.loads(path.read_bytes())
    assert d['status'] in {'passed','layout_matrix_passed'}
    same(d['actual_driver_prelaunch'])
    same(d['go_binary'])
    for source in d['go_production_sources']:
        same(source)
    pairs.append((path,d))
streams={}
mismatches=collections.Counter()
for _,pair in pairs:
    for row in pair['cases']:
        for key in ['certificate','rust_report','stdout','stderr']:
            same(row[key])
        raw=Path(row['certificate']['path']).read_bytes()
        digest=hashlib.sha256(raw).hexdigest()
        assert digest not in streams,'do not count repeated identical bytes as new verification'
        rust,go=row['rust_observation'],row['go_observation']
        assert rust['verdict']==go['verdict']
        if rust['verdict']=='accepted':
            assert row['exit_code']==0 and rust['axiom_count']==0 and not rust['axiom_report']['entries']
            assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
            assert go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
        else:
            assert row['exit_code']!=0 and rust['error_code']=='KERNEL_CORE_CHECK' and go['error_kind']=='core_check'
            detail=json.loads(rust['error_detail'])
            assert detail['code']=='CORE_TYPE_MISMATCH'
            mismatches[detail['details']['kind']]+=1
        streams[digest]=row
assert len(streams)==85
assert sum(r['rust_observation']['verdict']=='accepted' for r in streams.values())==74
assert sum(mismatches.values())==11

controls=ROOT/'controls/certificates'
repaired=ROOT/'corrected-controls/certificates'
assert {p.name for p in controls.glob('*.mpcert')}=={p.name for p in repaired.glob('*.mpcert')}
assert len(list(controls.glob('*.mpcert')))==10
for path in controls.iterdir():
    if path.suffix in {'.mpcert','.json'}:
        assert path.read_bytes()==(repaired/path.name).read_bytes(),path.name
for n in [0,3,70]:
    path=ROOT/f'inline/certificates/layout-N{n}.mpcert'
    canonical=ROOT/f'final/certificates/matrix-N{n}.mpcert'
    assert path.read_bytes()==canonical.read_bytes()
    assert path.with_suffix('.rust.json').read_bytes()==canonical.with_suffix('.rust.json').read_bytes()

_,affected=execution('affected')
assert '4 passed; 0 failed' in (ROOT/'affected/stdout.log').read_text()
old=ROOT.parent/'error-values/final/certificates'
fresh=ROOT/'affected/certificates/error-values'
assert {p.name for p in old.iterdir()}=={p.name for p in fresh.iterdir()}
for path in old.iterdir():
    assert path.read_bytes()==(fresh/path.name).read_bytes(),path.name
frozen_path=ROOT.parent/'product-relations/frozen-design-preservation.json'
frozen=json.loads(frozen_path.read_bytes())
assert len(frozen['inputs'])==255
for row in frozen['inputs']:
    same(row)

epochs=[]
for path in sorted(ROOT.glob('*/inputs.json')):
    source=json.loads(path.read_bytes())
    result_path=path.parent/'result.json'
    d=json.loads(result_path.read_bytes())
    assert d['status'] in {'passed','failed'},'all original handles must be terminal'
    assert d['source_receipt']==seal(path) and source['actual_driver_prelaunch']==seal(path.parent/'driver.py')
    assert d['whole_source_inputs_unchanged'] or d['formatter_mutation_expected']
    same(d['stdout']);same(d['stderr'])
    epochs.append({'result':seal(result_path),'status':d['status'],'command':d['command'],'preimages':len(source['sources'])})
failed=[r for r in epochs if r['status']=='failed']
assert {Path(r['result']['path']).parent.name for r in failed}=={'final','inline-lint'}
production=[Path('crates/mpk-vc/src/go_practical_ordinary.rs'),Path('crates/mpk-vc/src/go_practical_ordinary/error_layout.rs')]
inline_inputs=json.loads(Path(inline['source_receipt']['path']).read_bytes())['sources']
for path in production:
    row=next(r for r in inline_inputs if r['path']==str(path))
    same(row)

verification={
    'schema':'mpk.go_practical.error_layout_checkpoint.v1','status':'scoped_checks_passed',
    'basis_commit':'d4ac004547a575e5f4e81ec48385dc50f93ed435','language_scope':['Go','Rust'],
    'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'case_counts_checked':list(range(71)),'current_layouts_checked_in_both_cores':71,
    'distinct_original_streams_checked_in_both_cores':85,'zero_axiom_mathematical_acceptances':74,
    'genuine_core_type_mismatches':11,'mismatch_subtypes':dict(mismatches),
    'current_checks':current,'current_endpoint_construction':seal(inline_path),
    'literal_current_original_replay':seal(replay),'paired_executions':[seal(p) for p,_ in pairs],
    'original_expected_control_failure':seal(original_path),
    'unused_carrier_replacement_is_valid_math_but_fails_exact_ownership':True,
    'wrong_carrier_actual_typed_client_genuinely_type_rejects':True,
    'original_matrix_and_ownership_roots_passed_before_bad_probe':True,
    'corrected_control_outputs_reports_literal_equal_to_independent_checked_originals':True,
    'current_production_sources_unchanged_from_successful_inline_epoch':[seal(p) for p in production],
    'full_previous_error_view_outputs_literal_equal':True,'affected_old_Rust_roots':4,
    'unchanged_frozen_inputs':{'manifest':seal(frozen_path),'count':255},
    'source_epochs':len(epochs),'source_preimages':sum(r['preimages'] for r in epochs),
    'actual_executions':epochs,'original_failed_epochs':failed,
    'retained_hash_observer_error':seal(ROOT/'replay-hash-observer-first/result.json'),
    'N70_current':{'declarations':2568,'terms':122677,'bytes':1113774,'imports':0,'axioms':0},
    'selection_reason':'Exact frozen balanced carrier/nil and complete payload/Word64 observations for all sizes0..70. Current0/3/70 retain every old2050 named identity and complete original error-view hash. Preserve the original expensive matrix and failed unused-type expectation, repair only the output-helper style and control root, and compare all71 fresh complete artifacts literally before reusing their already-completed checks. Ten original definition corruptions and actual named-type use provide genuine core rejections; unused replacement is only mathematical acceptance with exact-ownership rejection. No unchanged word/frontend or unrelated suite is repeated.',
    'limits':{'declarations':4096,'terms':131072,'bytes':2097152,'conversion_fuel':1024},
    'kernel_axiom_import_or_budget_changes':False,'Go_production_frontend_changed':False,
    'agents_used':False,'GitHub_Actions_used':False,'all_actual_validation_handles_terminal':True,
    'complete_R01_R06_W01':False,'public_profile_activated':False,
    'full_T02_gate':'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed':False,
    'remaining':'Actual source/capture/module roles and closed-U/tag binding, source canonical nil/inactive zeros and payload validity, receiver/dispatch/precondition/result/storage/frame/lifetime/loop/state/runtime/program proofs; complete56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings still require final-gate resolution. No source/profile/program/release authority is established by the mathematical adapters.'}
(ROOT/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
print(json.dumps({k:verification[k] for k in ['status','current_layouts_checked_in_both_cores','distinct_original_streams_checked_in_both_cores','zero_axiom_mathematical_acceptances','genuine_core_type_mismatches','source_epochs','source_preimages']}))
