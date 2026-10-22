"""Audit terminal scoped evidence; private source recipes confer no program authority."""
from pathlib import Path
import datetime
import hashlib
import importlib.util
import json

ROOT=Path(__file__).resolve().parent


def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


def same(row):
    assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']},row['path']


streams={};pair_runs=[];go_preimages=0
for epoch in ['paired','controls-paired','ownership-paired']:
    path=ROOT/epoch/'result.json';d=json.loads(path.read_bytes());assert d['status']=='passed' and d['whole_inputs_unchanged']
    same(d['actual_driver_prelaunch']);same(d['complete_prelaunch_inputs']);same(d['go_binary'])
    for row in d['go_production_sources']:same(row)
    inputs=json.loads(Path(d['complete_prelaunch_inputs']['path']).read_bytes());go_preimages+=len(inputs)
    for row in inputs:
        same(row);same(row['complete_prelaunch_snapshot']);original=Path(row['path']);copy=Path(row['complete_prelaunch_snapshot']['path'])
        assert original.read_bytes()==copy.read_bytes()
        assert original.stat().st_mode&0o777==copy.stat().st_mode&0o777==row['mode']
    for row in d['cases']:
        for k in ['certificate','rust_report','stdout','stderr']:same(row[k])
        digest=row['certificate']['sha256'];assert digest not in streams
        rust,go=row['rust_observation'],row['go_observation'];assert rust['verdict']==go['verdict']
        if rust['verdict']=='accepted':
            assert row['exit_code']==0 and rust['axiom_count']==0 and not rust['axiom_report']['entries']
            assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
        else:
            assert row['exit_code']!=0 and rust['error_code']=='KERNEL_CORE_CHECK' and go['error_kind']=='core_check'
            detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH' and detail['details']['kind']=='check_type_mismatch'
        streams[digest]=row
    pair_runs.append(seal(path))
assert len(streams)==15
assert sum(r['rust_observation']['verdict']=='accepted' for r in streams.values())==7
assert sum(r['rust_observation']['verdict']=='rejected' for r in streams.values())==8
base=ROOT/'first/certificates/all-root-shapes.mpcert';repeat=ROOT/'controls/certificates/current-source-baseline.mpcert'
assert base.read_bytes()==repeat.read_bytes() and base.with_suffix('.rust.json').read_bytes()==repeat.with_suffix('.rust.json').read_bytes()

checks={}
for epoch in ['first','first-large','controls','ownership','replay','final-lint','final-format','oracle-complete-inputs','oracle-large','oracle-ownership']:
    p=ROOT/epoch/'result.json';d=json.loads(p.read_bytes());assert d['status']=='passed' and d['whole_source_inputs_unchanged'];checks[epoch]=seal(p)
    for k in ['source_receipt','actual_driver_prelaunch','stdout','stderr']:same(d[k])
source_map=ROOT/'replay/source-literal-map.json';generic_map=ROOT/'replay/generic-literal-map.json'
a=json.loads(source_map.read_bytes());b=json.loads(generic_map.read_bytes());assert len(a)==5 and len(b)==71
assert all(r['complete_current_original_bytes_literal_equal'] for r in a)
assert all(r['complete_original_and_current_bytes_literal_equal'] for r in b)
for row in a:
    epoch='first-large' if row['name']=='source-N70' else 'first';p=ROOT/epoch/f"certificates/{row['name']}.mpcert";raw=p.read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
for row in b:
    p=ROOT.parent/f"error-layouts/final/certificates/matrix-N{row['case_count']}.mpcert";raw=p.read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']

oracles=[ROOT/name/'structural-comparison.json' for name in ['oracle-complete-inputs','oracle-large']]
observed=[]
for p in oracles:
    d=json.loads(p.read_bytes());assert d['status']=='frozen_structural_comparison_passed';observed.extend(d['cases'])
assert len(observed)==5 and sum(len(r['types']) for r in observed)==115
for row in observed:
    same({'path':row['certificate'],'bytes':Path(row['certificate']).stat().st_size,'sha256':row['sha256']})
    assert row['all_source_rows_and_case_payloads_retained'] and all(r['complete_carrier_zero_and_recipe_match'] for r in row['types'])
owned=ROOT/'oracle-ownership/structural-comparison.json';d=json.loads(owned.read_bytes());assert d['status']=='passed' and d['frozen_source_zero_structural_mismatch_detected']

epochs=[]
for path in sorted(ROOT.glob('*/inputs.json')):
    inputs=json.loads(path.read_bytes());d=json.loads(path.with_name('result.json').read_bytes());assert d['status']=='passed'
    assert d['whole_source_inputs_unchanged'] or d['formatter_mutation_expected']
    same(d['source_receipt']);same(d['actual_driver_prelaunch']);same(d['stdout']);same(d['stderr'])
    epochs.append({'result':seal(path.with_name('result.json')),'inputs':len(inputs['sources']),'formatter_mutation_expected':d['formatter_mutation_expected']})
late=ROOT/'oracle-first/late-originals.json';gap=json.loads(late.read_bytes());assert len(gap['sources'])==2
for row in gap['sources']:
    original=row['original_source_row'];p=Path(row['complete_after_terminal_snapshot']);raw=p.read_bytes()
    assert len(raw)==original['bytes'] and hashlib.sha256(raw).hexdigest()==original['sha256'] and p.stat().st_mode&0o777==original['mode']
assert gap['status']=='original_unmodified_source_bytes_preserved_after_terminal'

production=['crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-vc/src/go_practical_ordinary/error_layout.rs','crates/mpk-vc/src/go_practical_ordinary/source_error_layout.rs']
for epoch in ['first-large','replay','final-lint']:
    rows=json.loads((ROOT/epoch/'inputs.json').read_bytes())['sources']
    for p in production:same(next(r for r in rows if r['path']==p))
frozen_path=ROOT.parent/'product-relations/frozen-design-preservation.json';frozen=json.loads(frozen_path.read_bytes());assert len(frozen['inputs'])==255
for row in frozen['inputs']:same(row)

spec=importlib.util.spec_from_file_location('source_recipe_section_stats',ROOT/'structural-oracle.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
p=ROOT/'first-large/certificates/source-N70.mpcert';raw=p.read_bytes();r=m.Sections(raw);r.at=7;r.string();r.string();r.string();assert r.number()==0;r.vec(r.string);r.vec(r.level);terms=len(r.vec(r.term));report=json.loads(p.with_suffix('.rust.json').read_bytes())
verification={'schema':'mpk.go_practical.source_error_layout_checkpoint.v1','status':'scoped_checks_passed','basis_commit':'52bf6f32fa544d69f551e83b72b6e5fa1cea46dd','language_scope':['Go','Rust'],'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_summary_driver_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'distinct_full_streams_checked_in_both_cores':15,'zero_axiom_mathematical_acceptances':7,'genuine_core_type_mismatches':8,'paired_executions':pair_runs,'Go_complete_prelaunch_inputs':go_preimages,'normal_original_streams':5,'independent_frozen_source_type_recipes':115,'structural_comparisons':[seal(p) for p in oracles],'coherent_wrong_zero_is_math_but_fails_exact_source_import_and_oracle':seal(owned),'current_source_literal_replay':seal(source_map),'old_generic71_literal_replay':seal(generic_map),'source_epochs':len(epochs),'original_input_rows':sum(e['inputs'] for e in epochs),'actual_source_executions':epochs,'initial_observer_copy_gap':{'rows':2,'initial_result_retained':seal(ROOT/'oracle-first/result.json'),'complete_after_terminal_archive':seal(late),'archive_not_claimed_prelaunch':True,'corrected_complete_prelaunch_rerun':checks['oracle-complete-inputs']},'current_checks':checks,'production_sources':[seal(p) for p in production],'N70':{'cases':70,'source_type_recipes':73,'declarations':report['declaration_count'],'terms':terms,'bytes':len(raw),'imports':0,'axioms':0},'frozen_design_inputs':{'manifest':seal(frozen_path),'count':255},'selection_reason':'New consumer of complete checked source value domain, all actual U cases and all checked source type dependencies. Verify source-derived zero/nil recipes, independently frozen structure, every retained element/field, exact source ownership and actual word/string/nil/empty/array-padding corruptions. Replay complete current5/source and old71/generic bytes to reuse accepted unchanged streams, without repeating expensive native checks or unrelated suites. Go sources/frontend unchanged; no new Go build required.','all_actual_validation_handles_terminal':True,'limits_unchanged':{'declarations':4096,'terms':131072,'bytes':2097152,'conversion_fuel':1024},'no_new_case_root_helper_parameter_cap':True,'Go_frontend_changed':False,'kernel_or_axiom_changes':False,'complete_R01_R06_W01':False,'public_profile_activated':False,'full_T02_gate':'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh','strict_whole_repository_gate_passed':False,'remaining':'Arbitrary-input validity and source-Go zero-evaluation/canonical-zero soundness, inactive-case validity, canonical capture/module-role/U/tag/boxing bindings, entry/receiver/dispatch/precondition/result/store/frame/lifetime/loop/runtime/program proofs; full56-family foundation/private registry/context, canonical scope/contract wire, native Linux/all169 application demands/T02-T11. Earlier historical-scope scanner findings still require final-gate resolution. These private source-key recipes do not supply frozen context or program authority.'}
(ROOT/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
print(json.dumps({k:verification[k] for k in ['status','distinct_full_streams_checked_in_both_cores','zero_axiom_mathematical_acceptances','genuine_core_type_mismatches','source_epochs','original_input_rows','Go_complete_prelaunch_inputs','N70']}))
