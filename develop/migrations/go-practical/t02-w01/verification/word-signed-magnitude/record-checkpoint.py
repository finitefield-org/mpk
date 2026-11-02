"""Bind completed current observations, preserving every original failed attempt."""
from pathlib import Path
import collections,datetime,hashlib,json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def current_sources(r):
 for s in r['sources']:assert seal(s['path'])==s
execution_path=root/'final/rust-tests/result.json';execution=read(execution_path)
assert execution['status']=='passed' and execution['exit_code']==0 and not execution['changed_inputs_during_execution']
current_sources(execution)
for s in execution['sources']:
 snap=seal(root/'final/rust-tests/inputs'/s['path']);assert (snap['sha256'],snap['bytes'])==(s['sha256'],s['bytes'])
stdout=(root/'final/rust-tests/stdout.log').read_text();assert '4 passed; 0 failed' in stdout and 'matched named prerequisites=1795' in stdout
certificate=root/'final/rust-tests/certificates/word-signed-magnitude-foundation.mpcert';report=read(certificate.with_suffix('.rust.json'))
assert report['verdict']=='accepted' and report['declaration_count']==2163 and report['axiom_count']==0
assert seal(certificate)['sha256']=='5f3d83784114bfb471f9a8c305628c15c04e3f3b1398a566d4721c754912c976' and certificate.stat().st_size==666154
assert certificate.read_bytes()==(root/'explicit-magnitude-choices/rust-construction/certificates/word-signed-magnitude-foundation.mpcert').read_bytes()==(root/'final/rust-tests/model/certificate.mpcert').read_bytes()
paired_path=root/'paired-final-corrected/result.json';paired=read(paired_path);assert paired['status']=='passed' and len(paired['cases'])==45
expected={'WrongNonzero','IgnoredComplementBound','IgnoredCarryBound','IgnoredCarryEquation','IgnoredComplementEquation','FalseSignNonzero','IgnoredNegativeBalance'}
expected|={f'{name}-{w}'for w in [8,16,32,64]for name in ['WrongNegation','WrongMagnitude','WrongZeroSign','FalseSignDefined','FalseMagnitudeZero','AlwaysTrueGuard','FakeTruePremise','FalseComplete','FalseIntegerEquality']}
observed=set();kinds=collections.Counter();accepts=0
for row in paired['cases']:
 r=row['rust_observation'];g=row['go_observation'];assert r['verdict']==g['verdict']
 if r['verdict']=='rejected':
  d=json.loads(r['error_detail']);assert d['code']=='CORE_TYPE_MISMATCH';observed.add(row['name']);kinds[d['details']['kind']]+=1
 else:
  assert r['hashes']==g['hashes'] and r['axiom_count']==g.get('axiom_count',0)==0;accepts+=1
assert observed==expected and accepts==2 and sum(kinds.values())==43
comparison_path=root/'native-comparison-final/result.json';comparison=read(comparison_path);assert comparison['status']=='passed' and comparison['actual_stages']==960 and comparison['exact_negations']==28 and comparison['minimum_negation_wrap_cases']==4
metrics_path=root/'final/rust-tests/model/metrics.json';metrics=read(metrics_path);assert metrics['Nat_correction_guard_or_theorem_bodies_executed'] is False
assert metrics['total_AST_steps']==2266488 and metrics['maximum_query_AST_steps']==214296
checks={}
for name,path in [('strict_clippy',root/'final/clippy/result.json'),('Rust_format',root/'final/format/result.json')]:
 r=read(path);assert r['status']=='passed' and not r['changed_inputs_during_execution'];current_sources(r);checks[name]={'record':seal(path),'status':'passed','seconds':r['wall_seconds']}
r=read(root/'final/go-format/result.json');assert r['status']=='passed' and seal(r['source']['path'])==r['source'];checks['new_Go_format']={'record':seal(root/'final/go-format/result.json'),'status':'passed','seconds':r['seconds']}
failed=[]
for attempt,scope in [('initial','Rust ownership/borrow compilation failure; no generated certificate or proof outcome.'),('owned-bounds','Original complete candidate rejected by both current cores; direct prefix diagnosis identifies SumCarryComplement. Explicit checked Bool-not to Bit conversion repairs it.'),('explicit-complement-flag','Original next complete candidate rejected by both current cores; direct prefix diagnosis identifies MagnitudeCross. Explicit small selector and Bit equalities repair it.')]:
 path=root/attempt/'rust-construction/result.json';r=read(path);assert r['status']=='failed' and r['exit_code']==101
 failed.append({'record':seal(path),'scope':scope,'semantic_negative_counted':False})
 if attempt!='initial':
  go=read(root/attempt/'go-candidate/result.json');assert go['Rust_observation']['verdict']==go['Go_observation']['verdict']=='rejected'
  failed[-1]['Go_original_observation']=seal(root/attempt/'go-candidate/result.json')
for p in ['original-first-declaration/rust-diagnostic/result.json','complement-first-declaration/rust-diagnostic/result.json']:assert read(root/p)['status']=='passed'
assert read(root/'explicit-magnitude-choices/rust-construction/result.json')['status']=='passed'
assert read(root/'explicit-magnitude-choices/rust-construction/source-envelope-qualification.json')['complete_current_test_execution_claimed'] is False
assert read(root/'explicit-magnitude-choices/go-candidate/result.json')['Go_observation']['verdict']=='accepted'
frozen=read(root/'frozen-design-preservation.json');assert frozen['status']=='passed' and len(frozen['inputs'])==255
for s in frozen['inputs']:assert seal(s['path'])==s
record={'schema':'mpk.go_practical.actual_signed_magnitude_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entry_commit':'f74e3f8a0c2c37ba8565d0fd58a3f2b6cddcccbd',
 'production_sources':[s for s in execution['sources']if s['path'].startswith('crates/mpk-vc/src/')],'test_model_and_original_inputs':[s for s in execution['sources']if not s['path'].startswith('crates/mpk-vc/src/')],
 'fresh_component':{'certificate':seal(certificate),'hashes':report['hashes'],'declarations':2163,'terms':49380,'bytes':666154,'axioms':0,'retained_named_prerequisites':1795,'new_signed_magnitude_declarations':368,'widths':[8,16,32,64],'frozen_candidate_bounds':{'bytes':2097152,'terms':131072,'declarations':4096},'all_bounds_unchanged_and_met':True,
 'actual_unsigned_laws':'Ux+U(actual Neg x)+modulus*Bit(actual outgoing carry)=modulus; carry equals actual IsZero x. Therefore Ux+U(actual Neg x)=modulus*Bit(actual DivideDefined x). Actual Neg and actual AbsBits preserve IsZero and DivideDefined.',
 'actual_sign_magnitude':'Zero implies actual highest bit=false; highest bit=true implies actual DivideDefined=true. Int.Equivalent(S(x),Int.Make(choose(sign,0,U(actual AbsBits x)),choose(sign,U(actual AbsBits x),0))) holds without a source value exclusion, including the minimum signed word.',
 'computed_negation_correction':'C=Int.Make(Bit(actual sign x)+Bit(actual sign(Neg x)),Bit(actual DivideDefined x)). Unconditionally Int.Equivalent(Int.Negate(S(x)),Int.Add(S(actual Neg x),Int.Mul(Int.FromNat(modulus),C))).',
 'computed_exact_guard':'NegationNoOverflow(x)=Nat.Equal(C_positive,C_negative). Both reflection/completeness directions and Boolean agreement with actual Int.Equal(mathematical negation,actual signed result) are ordinary checked theorems. Positive-modulus cancellation proves the reverse implication. Minimum negation wraps normally; the exact-result guard is false.',
 'construction':'Established Numeric arithmetic-prefix and disjoint Order construction precede exact closed dependency selection. Reuse exact SignedProduct Nat cancellation/positivity/flag reflection. Generic Nat induction and local flag cases, explicit selector/Bool/Bit/Int-projection equations and checked semiring algebra; no whole-word enumeration, caller witness, machine arithmetic rule, function extensionality or core/axiom/fuel change.',
 'exact_import':'Deterministic complete owning regeneration. Altered/oversized/honestly different otherwise valid components reject; no full descriptor, registry/context, source-program or profile activation authority.'},
 'prerequisite_preservation':{'named_type_body_proof_level_family_metadata_matches':1795,'exact_prior_complete_arithmetic_raw_sha256':'2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c','exact_prior_complete_signed_product_raw_sha256':'5b0d67641b9a4c5de86a32411a29f00d6ebdd5cdb049f1b21839e79526a66eda','reference_components':['word-arithmetic','word-order','word-signed-product'],'prior_unaffected_proof_model_native_suites_repeated':False},
 'Rust_execution':seal(execution_path),'passed_affected_roots':4,'same_byte_paired_checkers':seal(paired_path),'paired_runner_configuration_fix':paired['configuration_fix'],'same_byte_cases':45,'accepted_zero_axiom_exact_hashes':2,'rejected_core_type_mismatches':43,'core_mismatch_subtypes':dict(kinds),'negative_case_names':sorted(expected),'checks':checks,
 'original_failed_attempts':failed,'prefix_diagnostic_records':[seal(root/p)for p in ['original-first-declaration/rust-diagnostic/result.json','complement-first-declaration/rust-diagnostic/result.json']],'diagnostic_scope':'Only truncated in-memory declaration checks locate the first failure. Preserve complete original failed bytes/reports/sources and both original core outcomes; no prefix result is a complete certificate acceptance.',
 'initial_passing_construction':seal(root/'explicit-magnitude-choices/rust-construction/result.json'),'initial_construction_source_qualification':seal(root/'explicit-magnitude-choices/rust-construction/source-envelope-qualification.json'),'same_final_byte_initial_Go_acceptance':seal(root/'explicit-magnitude-choices/go-candidate/result.json'),'final_execution_annotation_correction':execution['selection_annotation_correction'],
 'finite_observations':{'comparison':seal(comparison_path),'native_execution':seal(root/'native/result.json'),'native_source':seal('develop/probes/go-practical/signed_magnitude_observer.go'),'query_origins':seal(root/'query-origins.json'),'metrics':seal(metrics_path),'selected_original_queries':32,'actual_stages':960,'exact_negations':28,'minimum_negation_wrap_cases':4,'zero_cases':4,'AST_steps':2266488,'maximum_query_AST_steps':214296,'two_fresh_native_outputs_identical':True,'prepared_image_files':5577,'actual_Boolean_terms_executed':True,'Nat_correction_guard_or_theorem_bodies_executed':False,'scope':comparison['scope']},
 'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json','CSharp_tests_executed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'authoritative_immutable_application_replay_claimed':False,'native_Linux_gate_claimed':False,'kernel_axiom_fuel_or_artifact_bounds_changed':False,
 'remaining_work':'Actual magnitude half-range bounds and signed division/remainder, variable/signed/right-shift numeric/source laws, remaining full source-typed carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All later T02-T11 gates remain.','full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
(root/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'declarations':2163,'terms':49380,'bytes':666154,'same_byte_cases':45,'semantic_negatives':43,'native_queries':32,'actual_stages':960,'subtypes':dict(kinds)}))
