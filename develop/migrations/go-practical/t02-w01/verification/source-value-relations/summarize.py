"""Assemble terminal evidence without treating historical failed epochs as passes."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent

def seal(p):
    p=Path(p);raw=p.read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
def result(name):
    p=ROOT/name/'result.json';d=json.loads(p.read_bytes());assert d['status']=='passed',name;return p,d
normal_names=['small-fixed','heterogeneous-final','final-large']
normal=[]
for name in normal_names:
    p,d=result(name);assert d['whole_source_inputs_unchanged'];normal.append(seal(p))
metadata=[json.loads(p.read_bytes()) for name in normal_names for p in (ROOT/name/'certificates').glob('*.sources.json')]
assert len(metadata)==83 and all(m['all_prior_source_operations_specific_identities_unchanged'] for m in metadata)
for unit,count in [('nil-only',1),('entry',4),('heterogeneous-payloads',7),('seventy-cases',71)]:
    rows=[m for m in metadata if m['unit']==unit];assert len(rows)==count
    assert {m['boxed_source_type_id'] for m in rows}=={None,*[r[0] for r in rows[0]['case_tags']]}
paired=[];cases=[]
for name in ['paired-small','paired-final-small','paired-heterogeneous','paired-large','paired-controls']:
    p,d=result(name);assert d['whole_inputs_unchanged'];paired.append(seal(p));cases+=d['cases']
assert len(cases)==97
assert len({r['certificate']['sha256'] for r in cases})==97
accepted=[r for r in cases if r['rust_observation']['verdict']=='accepted'];rejected=[r for r in cases if r['rust_observation']['verdict']=='rejected']
assert len(accepted)==89 and len(rejected)==8
for r in accepted:assert r['go_observation']['verdict']=='accepted' and r['rust_observation']['hashes']==r['go_observation']['hashes']
for r in rejected:assert r['go_observation']['verdict']=='rejected' and json.loads(r['rust_observation']['error_detail'])['code']=='CORE_TYPE_MISMATCH'
oracles=[];observations=[]
for name in ['oracle-final-small','oracle-heterogeneous','oracle-large']:
    p,d=result(name);assert d['whole_source_inputs_unchanged'];oracles.append(seal(p));observations+=json.loads((p.parent/'structural-comparison.json').read_bytes())['streams']
assert len(observations)==83
_,control=result('oracle-control');assert control['whole_source_inputs_unchanged']
checks=[]
for name in ['controls-small-fixed','format-name-fixed','lint-fixed']:
    p,d=result(name);assert d['whole_source_inputs_unchanged'];checks.append(seal(p))
epochs=[]
for p in sorted(ROOT.glob('*/inputs.json')):
    inputs=json.loads(p.read_bytes());d=json.loads(p.with_name('result.json').read_bytes());assert d['status'] in ['passed','failed']
    if not d['formatter_mutation_expected']:assert d['whole_source_inputs_unchanged']
    epochs.append({'result':seal(p.with_name('result.json')),'status':d['status'],'source_preimages':len(inputs['sources'])})
prelaunch=[]
for p in sorted(ROOT.glob('*/prelaunch-inputs.json')):
    d=json.loads(p.with_name('result.json').read_bytes());assert d['status']=='passed' and d['whole_inputs_unchanged']
    prelaunch.append({'receipt':seal(p),'input_rows':len(json.loads(p.read_bytes())),'kind':'Go paired checker' if 'go_binary' in d else 'Rust diagnostic only'})
application=json.loads((ROOT/'source-application/receipt.json').read_bytes());assert application['status']=='applied_exact_isolated_tested_source_bytes'
binding=json.loads((ROOT/'helper-name-adjustment/result.json').read_bytes());assert binding['status']=='private_helper_identifier_only_and_final_source_binding' and binding['generated_expression_strings_unchanged']
for row in binding['sources']:
    assert seal(row['path'])['sha256']==row['sha256']
_,replay=result('producer-replay');assert replay['whole_source_inputs_unchanged']
replay_rows=json.loads((ROOT/'producer-replay/certificates/producer-replay.json').read_bytes())['streams'];assert len(replay_rows)==4
assert all(row['fresh_unary_fix_producer_bytes_equal_original'] for row in replay_rows)
choice=json.loads((ROOT/'adapter-choice-audit.json').read_bytes());assert choice['status']=='only_nil_only_error_root_changes'
retired=json.loads((ROOT/'retired-plan-preservation.json').read_bytes())
for row in retired['sources']:
    assert seal(row['retained_path'])['sha256']==row['sha256'] and Path(row['retained_path']).stat().st_mode&0o777==row['mode']
summary={'schema':'mpk.go_practical.source_value_relations.v1','status':'checked_complete_closed_encoding_relation_component_only','basis_commit':'7ead1ea45e602338b52a0d9ddf6d30db92782432','normal_executions':normal,'isolated_tested_source_application':seal(ROOT/'source-application/receipt.json'),'private_helper_identifier_only_final_source_binding':seal(ROOT/'helper-name-adjustment/result.json'),'complete_boundary_byte_replay':seal(ROOT/'producer-replay/result.json'),'fresh_boundary_byte_streams':4,'closed_adapter_choice_audit':seal(ROOT/'adapter-choice-audit.json'),'full_source_type_adapter_choices':100,'only_changed_shape':'nil-only error, a unary product represented by its vector','retired_active_multilanguage_plan_preservation':seal(ROOT/'retired-plan-preservation.json'),'scope_document_originals':seal(ROOT/'scope-document-originals.json'),'paired_executions':paired,'independent_executions':oracles,'independent_wrong_source_identity_control':seal(ROOT/'oracle-control/result.json'),'affected_checks':checks,'normal_final_streams':83,'additional_original_pilot_streams':5,'initial_failed_control_suite_preserved_with_two_passed_roots':seal(ROOT/'controls/result.json'),'only_failed_first_control_root_reexecuted':True,'all_distinct_paired_streams':97,'zero_axiom_mathematics_acceptances':89,'genuine_type_rejections':8,'coherent_wrong_source_view_is_math_only':1,'independent_counts':{'closed_definitions':sum(r['closed_relation_definitions'] for r in observations),'closed_type_and_field_law_statements':sum(r['closed_relation_statements'] for r in observations),'whole_error_row_statements':sum(r['whole_row_statements'] for r in observations),'actual_all64_tag_bit_observations':sum(r['actual_tag_bit_observations'] for r in observations),'actual_nil_tag_bit_observations':sum(r['actual_nil_tag_bit_observations'] for r in observations),'actual_encoded_scalar_leaf_changes_and_identical_zero_observations':sum(r['decoded_leaf_changes_observed'] for r in observations)},'maxima':{k:max(m[k] for m in metadata) for k in ['declarations','terms','bytes']},'native_caps':{'declarations':4096,'terms':131072,'bytes':2097152,'fuel':1024},'source_epochs':epochs,'source_preimages':sum(e['source_preimages'] for e in epochs),'native_prelaunch_receipts':prelaunch,'Go_prelaunch_rows':sum(r['input_rows'] for r in prelaunch if r['kind']=='Go paired checker'),'all_prior_source_specific_identities_unchanged':True,'source_specificity':'Closed source-derived identity views and all encoded scalar observations; mathematical relations are not Go comparison/source validity/runtime/context authority.','source_only_assembly':'All same-U source-specific carrier/zero/boxing/nil/copy declarations retained; unchanged old call-site dispatch/receiver/representation producers remain in their own complete libraries. No source U, case, root, helper or parameter admission cap.','Go_Rust_only':True,'funext_or_axioms_added':False,'resource_limits_changed':False,'original_live_execution_restarted_after_observation_timeout':False,'not_application_verification_closure':True,'public_profile_activated':False,'R01_complete':False,'R06_complete':False,'full_T_gate':'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh','native_Linux_release_acceptance':False,'remaining':'Full source/input/active validity, receiver-once/nil safety/callee/precondition/results/effects/frame, canonical capture/module/U/tag/target/context, full56/private-registry/nativeLinux/all169/T02-T11 remain due. Prior historical-scope findings and nontext binary semantic handling remain for the final strict gate.'}
(ROOT/'verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:summary[k] for k in ['status','normal_final_streams','all_distinct_paired_streams','independent_counts','maxima','source_preimages','Go_prelaunch_rows']}))
