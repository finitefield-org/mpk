"""Record the actual bounded-rank/full-string equality bridge and scoped checks."""
from pathlib import Path
import datetime
import hashlib
import json

OUT=Path(__file__).resolve().parent
BASE='86f29e185fa4911f5a03a68f9c75c37055215a7b'

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}

pair=json.loads((OUT/'paired/result.json').read_bytes())
assert pair['status']=='passed' and len(pair['cases'])==62
assert pair['zero_axiom_acceptances']==2 and pair['genuine_core_type_mismatches']==60
checks={}
for mode in ['selected','lint','format']:
    path=OUT/'epochs/direction-corrected'/mode/'result.json';row=json.loads(path.read_bytes())
    assert row['status']=='passed' and row['exit_code']==0 and row['whole_source_inputs_unchanged']
    checks[mode]={'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log')}
root=OUT/'epochs/direction-corrected/selected/candidates'
bounded=sum(json.loads(path.read_bytes())['actual_AST_steps']for path in root.glob('bounded-*.observation.json'))
domain=sum(json.loads(path.read_bytes())['actual_AST_steps']for path in root.glob('domain-*.observation.json'))
assert len(list(root.glob('bounded-*.input.json')))==75 and len(list(root.glob('bounded-*.observation.json')))==150
assert len(list(root.glob('domain-*.input.json')))==len(list(root.glob('domain-*.observation.json')))==5
assert bounded==152401067 and domain==5157035
archives=[json.loads(path.read_bytes())for path in sorted((OUT/'epochs').glob('*/inputs.json'))]
assert len(archives)==3 and sum(len(row['sources'])for row in archives)==1143
executions=[]
for path in sorted((OUT/'epochs').glob('*/*/result.json')):
    row=json.loads(path.read_bytes());assert row['status']in {'passed','failed'} and row['whole_source_inputs_unchanged']
    executions.append({'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log'),'status':row['status'],'exit_code':row['exit_code']})
assert len(executions)==7 and sum(row['status']=='failed'for row in executions)==1
record={
 'schema':'mpk.go_practical.string_bounded_order_checkpoint.v1','status':'scoped_checks_passed','entry_commit':BASE,
 'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_language_scope':['Go','Rust'],
 'generic_component':{'declarations':1729,'terms':53121,'bytes':687164,'new_definitions':0,'new_ordinary_laws':23,'all1706_prior_identities_exact':True,'old_complete_order_producer_bytes_exact':True,'input':seal(root/'bounded-order-foundation.mpcert')},
 'proof_structure':'A real upper witness on the larger full64 logical size makes the shorter length a valid slot address, using the original guarded Word.Index.D8.BoundedIndex. Equal ranks would identify end0 with the actual present-byte successor, a contradiction proved with ordinary Nat.Equal/Pred laws. Both upper witnesses therefore recover full size equality. Pointwise rank equality recovers every observable byte bit via actual unsigned value equality/ByteEqual reflection, without function extensionality. Reflection/completion connects this real content relation to existing string Equal. Total/neither-strict and both-weak equality, weak antisymmetry, trichotomy, equality exclusion and weak inclusion all use the actual operators.',
 'affected_Rust_test_roots':5,'affected_checks':checks,
 'complete_same_byte_inputs':62,'zero_axiom_acceptances':2,'genuine_core_type_mismatches':60,'independent_comparison':seal(OUT/'paired/result.json'),
 'negative_controls':{'changed_actual_conclusions':23,'changed_required_typed_witness_domains':37,'partial_certificate_acceptance_claimed':False},
 'finite_observations':{'whole_bounded_input_pairs':75,'raw_and_normalized_observations':150,'bounded_AST_steps':bounded,'full_length_word_bits':64,'all_byte_bits':8,'complete_original_arrays_and_available_partial_outputs_retained':True,'columns':['Less','LessEqual','Greater','GreaterEqual','StringEqual','RankEqual','neither_strict','both_weak','trichotomy','upper_left','upper_right'],'actual_definition_callbacks':False,'theorem_bodies_executed':False,'supplies_universal_proof':False,'native_Go_source_or_runtime_receipt':False,'observer_ceilings_unchanged':[50000000,2000000,2000000]},
 'required_bound_countermodels':{'cases':5,'full_raw_lengths':[[256,257],[257,256],[257,258],[511,512],[512,511]],'AST_steps':domain,'every_observed_rank_equal_true_and_full_string_equal_false':True,'full64_bits_and_all256_slot_bytes_retained':True,'actual_NUL_value_zero_and_present_NUL_rank_nonzero':True,'within_frozen_profile_domain':False,'admitted_source_program_counterexample':False},
 'complete_source_epochs':3,'complete_original_source_preimages':1143,'actual_executions':executions,
 'original_failure':'The added trichotomy recipe reversed an intermediate Eq chain, so construction returned Linkage before any candidate could be emitted; all five root outcomes, complete original source and driver/output bytes remain. Removing that reversal repairs the recipe; no checker/cap/observer change or reconstructed original certificate is used. The initial17-law component passed separately.',
 'selection_reason':'Select the new ordinary producer and actual consumers, all1706 prior identities/complete old order bytes, exact ownership, every new conclusion and required typed witness, plus75 whole raw/normalized cases for the actual new statements and five genuine out-of-domain countermodels proving why bounds matter. Byte/arithmetic circuits, Go/frontend/native capture/core/ref/protocol and unrelated suites remain exact and are not repeated; current target Clippy/five-file format cover all edited Rust files. All checks run locally.',
 'all_candidates_and_actual_drivers_saved_before_verification':True,'all_actual_test_checker_handles_terminal':True,
 'frozen_design_inputs_unchanged':255,'Go_frontend_build_inputs_unchanged':186,'checker_limits_or_frozen_caps_changed':False,
 'public_profile_activated':False,'source_program_or_registered_context_proof':False,'native_release_acceptance':False,
 'R01_complete':False,'R06_complete':False,'W01_complete':False,'full_T_gate_passed':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
 'remaining':'Original Go source/type/evaluation/nominal/signed-domain binding and source preservation; prefix/suffix/substring/concat/array/storage/state/program; full ordinary foundation/private descriptor/registry/context/native kernel/general namespace/selected closure/all169/T02-T11. Fixed raw data/proof carriers and observer results alone provide no source authority.'
}
(OUT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({key:record[key]for key in ['status','affected_Rust_test_roots','complete_same_byte_inputs','zero_axiom_acceptances','genuine_core_type_mismatches','complete_source_epochs','complete_original_source_preimages','R01_complete','R06_complete','W01_complete']}))
