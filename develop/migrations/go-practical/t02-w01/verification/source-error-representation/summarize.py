"""Audit terminal original streams, scoped checks and retained failed epochs."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import runpy

ROOT = Path(__file__).resolve().parent
virtual=runpy.run_path(str(ROOT/'virtual-originals.py'));kept=virtual['kept']


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def same(row):
    assert seal(row['path']) == {k: row[k] for k in ['path', 'bytes', 'sha256']}, row['path']


streams = {}
pair_runs = []
go_preimages = 0
for epoch in ['paired-small', 'paired-large', 'paired-heterogeneous', 'controls-paired', 'large-controls-paired']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_inputs_unchanged']
    for key in ['actual_driver_prelaunch', 'complete_prelaunch_inputs', 'go_binary']:
        same(run[key])
    for row in run['go_production_sources']:
        same(row)
    inputs = json.loads(Path(run['complete_prelaunch_inputs']['path']).read_bytes())
    go_preimages += len(inputs)
    for row in inputs:
        same(row)
        same(row['complete_prelaunch_snapshot'])
        original, copy = Path(row['path']), Path(row['complete_prelaunch_snapshot']['path'])
        assert original.read_bytes() == copy.read_bytes()
        assert original.stat().st_mode & 0o777 == copy.stat().st_mode & 0o777 == row['mode']
    if epoch=='paired-large':
        assert run['Rust_terminal_result_observed_after_pairing']
        assert len(run['per_row_prelaunch_receipts'])==71
        # The rolling receipt is metadata, never a core input. Verify every
        # exact prelaunch prefix against its independently saved row receipt.
        by_path={r['path']:i for i,r in enumerate(inputs)}
        for row in run['cases']:
            same(row['prelaunch_row_receipt'])
            pre=json.loads(Path(row['prelaunch_row_receipt']['path']).read_bytes())
            assert pre['saved_before_native_Go_invocation']
            end=max(by_path[pre[k]['path']] for k in ['certificate','rust_report','metadata'])+1
            buf=(json.dumps(inputs[:end],indent=2)+'\n').encode()
            assert len(buf)==pre['common_and_completed_rows']['bytes']
            assert hashlib.sha256(buf).hexdigest()==pre['common_and_completed_rows']['sha256']
    for row in run['cases']:
        for key in ['certificate', 'rust_report', 'stdout', 'stderr']:
            same(row[key])
        digest = row['certificate']['sha256']
        assert digest not in streams
        rust, go = row['rust_observation'], row['go_observation']
        assert rust['verdict'] == go['verdict']
        if rust['verdict'] == 'accepted':
            assert row['exit_code'] == 0 and rust['axiom_count'] == 0 and not rust['axiom_report']['entries']
            assert rust['hashes'] == go['hashes'] and rust['declaration_count'] == go['declaration_count']
        else:
            assert row['exit_code'] != 0 and rust['error_code'] == 'KERNEL_CORE_CHECK' and go['error_kind'] == 'core_check'
            detail = json.loads(rust['error_detail'])
            assert detail['code'] == 'CORE_TYPE_MISMATCH' and detail['details']['kind'] in ['check_type_mismatch','lambda_domain_mismatch']
            assert re.fullmatch(r'term [0-9]+ inferred (app|pi) but expected (app|pi)',go['error_detail'])
        streams[digest] = row
    pair_runs.append(seal(path))
assert len(streams) == 108
assert sum(r['rust_observation']['verdict'] == 'accepted' for r in streams.values()) == 100
assert sum(r['rust_observation']['verdict'] == 'rejected' for r in streams.values()) == 8

checks = {}
for epoch in ['entries', 'first-large', 'heterogeneous', 'controls', 'large-controls', 'lint', 'format-check', 'oracle-small', 'oracle-large', 'oracle-heterogeneous', 'oracle-owned', 'replay']:
    path = ROOT/epoch/'result.json'
    run = json.loads(path.read_bytes())
    assert run['status'] == 'passed' and run['whole_source_inputs_unchanged']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    checks[epoch] = seal(path)

native_checks={}
for epoch in ['native-format','native-format-corrected','native-corrected','native-format-check','derived']:
    path=ROOT/epoch/'result.json';run=json.loads(path.read_bytes())
    assert run['status']=='passed' and (run['whole_source_inputs_unchanged'] or run['formatter_mutation_expected'])
    for key in ['source_receipt','actual_driver_prelaunch','stdout','stderr']:same(run[key])
    native_checks[epoch]=seal(path)
assert json.loads((ROOT/'native-format-check/result.json').read_bytes())['stdout']['bytes']==0
native_run=json.loads((ROOT/'native-corrected/result.json').read_bytes())
native_events=[json.loads(line) for line in Path(native_run['stdout']['path']).read_text().splitlines()]
assert any(e.get('Action')=='pass' and e.get('Test')=='TestPracticalErrorRepresentationOriginalPayloads' for e in native_events)
controls=json.loads(kept(ROOT/'native-corrected/native-originals/dispatch-controls.json')[0])
assert (controls['cases'],controls['complete_error_payload_types'])==(1,6)
heterogeneous_corpus=ROOT/'corpus-heterogeneous.json'
assert len(json.loads(heterogeneous_corpus.read_bytes())['cases'])==1

replay=ROOT/'replay/certificates/receiver-row-literal-map.json'
literal_replays=json.loads(replay.read_bytes())
assert len(literal_replays)==92 and all(r['complete_current_original_bytes_literal_equal'] for r in literal_replays)
for row in literal_replays:
    original=Path(row['original']);assert original.stat().st_size==row['bytes']
large_rows=sorted((ROOT/'first-large/certificates').glob('*.sources.json'))
assert len(large_rows)==71

oracles = [ROOT/name/'structural-comparison.json' for name in ['oracle-small', 'oracle-large', 'oracle-heterogeneous']]
observed = []
for path in oracles:
    data = json.loads(path.read_bytes())
    assert data['status'] == 'frozen_error_representation_structure_passed'
    observed.extend(data['streams'])
assert len(observed) == 99 and sum(len(r['cases']) for r in observed) == 5072
assert sum(r['actual_tag_bit_observations'] for r in observed) == 324608
assert sum(r['actual_nil_tag_bit_observations'] for r in observed) == 6336
assert sum(r['complete_operation_definitions_compared'] for r in observed) == 495
assert sum(r['complete_operation_theorem_types_compared'] for r in observed) == 11530
assert sum(r['all_copy_payload_fields_preserved'] for r in observed) == 5072
assert sum(r['complete_case_row_theorem_types_compared'] for r in observed)==10053
assert sum(len(r['case_comparisons']) for r in observed)==4981
assert sum(r['selector_definitions_compared'] for r in observed)==10441
assert sum(r['selector_statements_compared'] for r in observed)==15414
assert len([r for r in observed if r['boxed_source_type_id'] is None])==8
assert all(r['receiver_predicate_definition_compared'] for r in observed)
assert sum(r['receiver_statements_compared'] for r in observed)==11324
assert all(r['representation_predicate_definition_compared'] for r in observed)
assert sum(r['representation_statements_compared'] for r in observed)==16032
assert sum(r['every_inactive_source_zero_checked'] for r in observed)==4981
for row in observed:
    raw = Path(row['certificate']).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['raw_sha256']
    assert row['full_source_U_preserved']
    assert all(c['all64_tag_bits_and_whole_active_inactive_payload_structure_equal'] for c in row['cases'])
owned = ROOT/'oracle-owned/structural-comparison.json'
ownership = json.loads(owned.read_bytes())
assert ownership['status'] == 'passed' and ownership['coherent_wrong_ordinal_is_valid_math_but_fails_exact_target_definition'] and ownership['different_accepted_original_call_site_owner_mismatch_detected'] and ownership['same_complete_U_and_method_targets_preserved_for_wrong_site_control']

epochs, failures = [], []
for path in sorted(ROOT.glob('*/inputs.json')):
    inputs = json.loads(path.read_bytes())
    run = json.loads(path.with_name('result.json').read_bytes())
    assert run['status'] in {'passed', 'failed'}
    assert run['whole_source_inputs_unchanged'] or run['formatter_mutation_expected']
    for key in ['source_receipt', 'actual_driver_prelaunch', 'stdout', 'stderr']:
        same(run[key])
    record = {'result': seal(path.with_name('result.json')), 'inputs': len(inputs['sources']), 'status': run['status'], 'formatter_mutation_expected': run['formatter_mutation_expected']}
    epochs.append(record)
    if run['status'] == 'failed':
        failures.append({**record, 'counted_as_semantic_negative': False})

production = ['crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-vc/src/go_practical_ordinary/source_error_receiver.rs','crates/mpk-vc/src/go_practical_ordinary/source_error_representation.rs']
for epoch in ['entries', 'first-large', 'heterogeneous', 'controls', 'large-controls', 'lint', 'replay']:
    rows = json.loads((ROOT/epoch/'inputs.json').read_bytes())['sources']
    for path in production:
        same(next(r for r in rows if r['path'] == path))
frozen_path = ROOT.parent/'product-relations/frozen-design-preservation.json'
frozen = json.loads(frozen_path.read_bytes())
assert len(frozen['inputs']) == 255
for row in frozen['inputs']:
    same(row)
large_stats=[]
for path in large_rows:
    metadata=json.loads(path.read_bytes());cert=path.with_name(path.name.removesuffix('.sources.json')+'.mpcert');report=json.loads(cert.with_suffix('.rust.json').read_bytes())
    assert len(metadata['case_tags'])==70 and len(metadata['types'])==73
    assert metadata['declarations']==report['declaration_count']<=4096
    assert metadata['terms']<=131072 and metadata['bytes']==cert.stat().st_size<=2097152
    large_stats.append({k:metadata[k] for k in ['boxed_source_type_id','declarations','terms','bytes']})

retention=json.loads((ROOT/'native-original-relocations.json').read_bytes())
assert retention['status']=='passed'
virtual_files=virtual['index']()
content_review_path=ROOT/'archive-content-review/result.json'
content_review=json.loads(content_review_path.read_bytes())
assert content_review['status']=='passed' and not content_review['findings']
assert (content_review['matcher_count'],content_review['scopes'],content_review['seals'],content_review['exclusions'])==(127,4,39,51)
mode_audit=json.loads((ROOT/'virtual-original-byte-mode-audit.json').read_bytes())
assert mode_audit['status']=='passed' and mode_audit['complete_changed_derived_prelaunch_rows_checked']==10986 and mode_audit['original_Git_blob_rows_deferred_to_scoped_input_review']==817
assert len(content_review['originals'])==len(virtual_files)==mode_audit['virtual_files']

assert len(virtual_files)==sum(g['files'] for g in retention['groups'])
for path,(raw,mode,archive,member) in virtual_files.items():
    group=next(g for g in retention['groups'] if g['archive']==archive)
    original=Path(group['retained_physical_directory'])/member
    assert original.read_bytes()==raw and original.stat().st_mode&0o777==mode

verification = {
    'schema': 'mpk.go_practical.source_error_representation_checkpoint.v1',
    'status': 'scoped_checks_passed',
    'basis_commit': 'cd427ec4e6d7a7b2e6a0628da1a20fed94475019',
    'language_scope': ['Go', 'Rust'],
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'actual_summary_driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'distinct_full_streams_checked_in_both_cores': len(streams),
    'zero_axiom_mathematical_acceptances': 100,
    'genuine_core_type_mismatches': 8,
    'paired_executions': pair_runs,
    'Go_complete_prelaunch_inputs': go_preimages,
    'normal_original_streams': 99,
    'independent_complete_source_box_constructors': 5072,
    'actual_tag_bit_observations': 324608,
    'structural_comparisons': [seal(p) for p in oracles],
    'wrong_target_ordinal_math_and_wrong_call_owner_fail_frozen_comparison': seal(owned),
    'prior_receiver92_literal_replay': {'map':seal(replay),'rows':92},
    'prior_complete_receiver_rows_evidence_unchanged': seal(ROOT.parent/'source-error-receiver/verification.json'),
    'actual_nil_tag_bit_observations': sum(r['actual_nil_tag_bit_observations'] for r in observed),
    'complete_operation_definitions_compared': sum(r['complete_operation_definitions_compared'] for r in observed),
    'complete_operation_theorem_statements_compared': sum(r['complete_operation_theorem_types_compared'] for r in observed),
    'every_source_copy_payload_field_compared': sum(r['all_copy_payload_fields_preserved'] for r in observed),
    'complete_source_case_row_theorem_statements_compared':sum(r['complete_case_row_theorem_types_compared'] for r in observed),
    'case_comparison_pairs':sum(len(r['case_comparisons']) for r in observed),
    'selector_definitions_compared':sum(r['selector_definitions_compared'] for r in observed),
    'selector_theorem_statements_compared':sum(r['selector_statements_compared'] for r in observed),
    'actual_decoded_selector_target_observations':len(observed),
    'receiver_predicate_definitions_compared':len(observed),
    'representation_predicate_definitions_compared':len(observed),
    'complete_representation_theorem_statements_compared':sum(r['representation_statements_compared'] for r in observed),
    'every_inactive_source_zero_checked':sum(r['every_inactive_source_zero_checked'] for r in observed),
    'complete_receiver_theorem_statements_compared':sum(r['receiver_statements_compared'] for r in observed),
    'actual_decoded_copy_target_and_nil_observations':3*len(observed),
    'nil_without_selected_target_rows':sum(r['boxed_source_type_id'] is None for r in observed),
    'source_epochs': len(epochs),
    'original_input_rows': sum(e['inputs'] for e in epochs),
    'actual_source_executions': epochs,
    'retained_failed_epochs': failures,
    'current_checks': checks,
    'original_Go_heterogeneous_source_comparison':native_checks,
    'heterogeneous_original_corpus':seal(heterogeneous_corpus),
    'heterogeneous_payload_cases':6,
    'heterogeneous_rows':7,
    'production_sources': [seal(p) for p in production],
    'N70': {'cases':70,'source_type_recipes':73,'rows':71,'concrete_target_rows':70,'nil_no_target_rows':1,'row_sizes':large_stats,'maximum_declarations':max(r['declarations'] for r in large_stats),'maximum_terms':max(r['terms'] for r in large_stats),'maximum_bytes':max(r['bytes'] for r in large_stats),'imports':0,'axioms':0},
    'frozen_design_inputs': {'manifest': seal(frozen_path), 'count': 255},
    'selection_reason':'Verify the exact closed full-case tag and every inactive source-zero equality; all construction/copy projection and selection/nil consequences retain the complete original U and previous source/receiver identities. Both complete cores check21 original small-site rows, all71 large rows and7 heterogeneous original-Go rows with bool/64bit/empty-struct/array/string/slice payloads. Independently compare the complete predicate, every statement and actual full decoded constructors/tag/nil/copy/target observations. Seven small and one large false predicate/witness/inactive/copy/target/nil claims must genuinely type reject. Coherently changed target ordinal math remains accepted but exact source-owner import and frozen body reject. Another accepted call-site substitution rejects. Replay all92 old receiver rows literally, scoped lint/format and Go native comparison only; no unchanged broad suites or intermediate full gate.',
    'packaging':'RepresentedCase is the actual same-U receiver-tag premise and equality of EVERY inactive payload to the corresponding original source zero, as a balanced ordinary product of propositions. Projections expose the exact conditions. Actual Box/Nil witnesses establish all fields, and copy transfers preserve each one through old checked copy observations. Every prior receiver/selector/source/case/constructor/zero/payload/nil/copy/comparison identity remains. Active payload validity and source-runtime/canonical context authority are deliberately separate requirements, not assumed or declared satisfied; no funext, axiom, pruning, alternate table or case/root/helper/parameter cap.',
    'all_actual_validation_handles_terminal': True,
    'limits_unchanged': {'declarations': 4096, 'terms': 131072, 'bytes': 2097152, 'conversion_fuel': 1024},
    'no_new_case_root_helper_parameter_cap': True,
    'Go_frontend_changed': False,
    'Go_original_comparison_test_added':True,
    'maximum_local_Go_artifact_verification_seconds':max(r['seconds'] for r in streams.values()),
    'local_timing_scope':'Concurrent Darwin mathematical checking only. Native Linux proof-process/isolation/resource acceptance remains unverified; elapsed times, including slow accepted rows, are retained literally.',
    'native_Linux_resource_gate_passed':False,
    'complete_original_mode_retention':seal(ROOT/'native-original-relocations.json'),
    'decoded_original_content_review':seal(content_review_path),
    'captured_changed_prelaunch_byte_mode_audit':seal(ROOT/'virtual-original-byte-mode-audit.json'),
    'virtual_original_files':len(virtual_files),
    'virtual_original_bytes':sum(len(r[0]) for r in virtual_files.values()),
    'kernel_or_axiom_changes': False,
    'complete_R01_R06_W01': False,
    'public_profile_activated': False,
    'full_T02_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'strict_whole_repository_gate_passed': False,
    'remaining': 'Original-source validity and full receiver-once/nil-safety/callee/precondition/joint-result/effect/frame execution, nil/copy/boxing/comparison execution and zero-evaluation and arbitrary-input/inactive-case validity; canonical capture/module-role/U/tag/boxing bindings; entry/receiver/dispatch/precondition/result/store/frame/lifetime/loop/runtime/program proofs; full56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings remain for the final strict gate. Private source tags and recipes confer no canonical context or program authority.'
}
(ROOT/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps({k:verification[k] for k in ['status','distinct_full_streams_checked_in_both_cores','source_epochs','original_input_rows','Go_complete_prelaunch_inputs']}))
