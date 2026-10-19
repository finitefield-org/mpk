"""Bind the current signed half-range observations without rerunning checks."""
from pathlib import Path
import collections,datetime,hashlib,json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def current_sources(record):
 assert not record['changed_inputs_during_execution']
 for s in record['sources']:assert seal(s['path'])==s
execution_path=root/'final/rust-tests/result.json';execution=read(execution_path)
assert execution['status']=='passed'and execution['exit_code']==0
current_sources(execution)
for s in execution['sources']:
 snap=seal(root/'final/rust-tests/inputs'/s['path']);assert(snap['sha256'],snap['bytes'])==(s['sha256'],s['bytes'])
stdout=(root/'final/rust-tests/stdout.log').read_text();assert'4 passed; 0 failed'in stdout and'matched named prerequisites=2163'in stdout
certificate=root/'final/rust-tests/certificates/word-signed-bounds-foundation.mpcert';report=read(certificate.with_suffix('.rust.json'))
assert report['verdict']=='accepted'and report['declaration_count']==2683 and report['axiom_count']==0
assert seal(certificate)['sha256']=='8895b359ea61393aaf241c82e26366498f06cd12a0645e5ba37e3cbe14c41a31'and certificate.stat().st_size==818605
assert certificate.read_bytes()==(root/'actual-half-range-and-sign/rust-construction/certificates/word-signed-bounds-foundation.mpcert').read_bytes()==(root/'final/rust-tests/model/certificate.mpcert').read_bytes()
paired_path=root/'paired-final/result.json';paired=read(paired_path);assert paired['status']=='passed'and len(paired['cases'])==60
expected={'FalseGapOrder','IgnoredThresholdBound','FalseThresholdEquality','IgnoredValueSplit','IgnoredComplementEquation','IgnoredHalfDouble','IgnoredNegativeBalance','WrongScalarBalance'}
expected|={f'{name}-{d}'for d in range(7)for name in ['ZeroHalf','FalseTop']}
expected|={f'{name}-{w}'for w in [8,16,32,64]for name in ['WrongGap','FalseMagnitudeBound','FakeSignPremise','IgnoredHalfInput','FalseMinimumReflect','FalseMinimumComplete','FalseMinimumEquality','FalseMagnitudeSign','FalseUnsignedThreshold']}
observed=set();kinds=collections.Counter();accepted=0
for row in paired['cases']:
 r=row['rust_observation'];g=row['go_observation'];assert r['verdict']==g['verdict']
 if r['verdict']=='rejected':
  d=json.loads(r['error_detail']);assert d['code']=='CORE_TYPE_MISMATCH';observed.add(row['name']);kinds[d['details']['kind']]+=1
 else:
  assert r['axiom_count']==g.get('axiom_count',0)==0 and r['hashes']==g['hashes'];accepted+=1
assert observed==expected and accepted==2 and sum(kinds.values())==58
comparison_path=root/'native-comparison-final/result.json';comparison=read(comparison_path);assert comparison['status']=='passed'
metrics_path=root/'final/rust-tests/model/metrics.json';metrics=read(metrics_path)
assert metrics['Nat_half_low_complement_gap_guard_or_theorem_bodies_executed']is False
assert metrics['actual_word_observations']==metrics['actual_Top_definition_applications']==96 and metrics['actual_word_bit_observations']==2880
assert metrics['minimum_magnitude_cases']==metrics['zero_cases']==4
checks={}
for name,path in [('strict_clippy',root/'current/clippy/result.json'),('Rust_format',root/'current/format/result.json')]:
 r=read(path);assert r['status']=='passed';current_sources(r);checks[name]={'record':seal(path),'status':'passed','seconds':r['wall_seconds']}
r=read(root/'current/go-format/result.json');assert r['status']=='passed'and seal(r['source']['path'])==r['source'];checks['new_Go_format']={'record':seal(root/'current/go-format/result.json'),'status':'passed','seconds':r['seconds']}
initial_path=root/'actual-half-range/rust-construction/result.json';initial=read(initial_path);assert initial['status']=='failed'and initial['exit_code']==101 and initial['certificates']==[]
construction_path=root/'actual-half-range-and-sign/rust-construction/result.json';construction=read(construction_path);assert construction['status']=='passed'and not construction['changed_inputs_during_execution']
for s in construction['sources']:
 if s['path'].startswith('crates/mpk-vc/src/'):assert seal(s['path'])==s
candidate_path=root/'actual-half-range-and-sign/go-candidate/result.json';assert read(candidate_path)['Go_observation']['verdict']=='accepted'
frozen=read(root/'frozen-design-preservation.json');assert frozen['status']=='passed'and len(frozen['inputs'])==255
for s in frozen['inputs']:assert seal(s['path'])==s
record={'schema':'mpk.go_practical.actual_signed_half_bounds_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entry_commit':'13ef01d82caef1398080d3f58833ca5e58c9b18a',
 'production_sources':[s for s in execution['sources']if s['path'].startswith('crates/mpk-vc/src/')],'test_model_and_original_inputs':[s for s in execution['sources']if not s['path'].startswith('crates/mpk-vc/src/')],
 'fresh_component':{'certificate':seal(certificate),'hashes':report['hashes'],'declarations':2683,'terms':58000,'bytes':818605,'axioms':0,'retained_named_prerequisites':2163,'new_signed_bounds_declarations':520,'widths':[8,16,32,64],'cube_depths':list(range(7)),'frozen_candidate_bounds':{'bytes':2097152,'terms':131072,'declarations':4096},'all_bounds_unchanged_and_met':True,
 'balanced_half_and_lower_parts':'H0=1, Low0=LowComplement0=0. At depth d>0, h=d-1: Hd=Weight_h*Hh; Lowd=Value_h(low_half)+Weight_h*Low_h(high_half); LowComplementd=Value_h(actual NotCube_h(low_half))+Weight_h*LowComplement_h(high_half). Topd applies all true addresses. All are real ordinary definitions.',
 'generic_actual_laws':'For every depth0..6 and arbitrary Bool cube: Hd+Hd=Weightd; Hd>0; Value_d=Lowd+Hd*Bit(actual Topd); (Lowd+LowComplementd)+1=Hd; Lowd<Hd. No huge unary modulus or operand enumeration.',
 'actual_magnitude_gap':'Gap(x)=choose(actual sign(x),Low_d(x),LowComplement_d(x)+1). For every actual word, U(actual AbsBits(x))+Gap(x)=Hd, hence magnitude<=Hd. Actual sign=false implies both unsigned value and magnitude strictly below Hd; sign=true gives magnitude+Low=Hd.',
 'actual_threshold_and_minimum':'Less(U(x),Hd)=Not(actual sign(x)). Half magnitude forces the actual sign true. Both exact Nat-equality directions show U(actual AbsBits(x))=Hd iff U(x)=Hd; reflected Nat.Equal Booleans agree. Under the actual proved half bound, the magnitude sign bit also equals Nat.Equal(U(x),Hd). Minimum magnitudes retain their high bit normally.',
 'construction':'Retain all2163 exact prior signed-magnitude declarations, then add disjoint ordinary range definitions/proofs. Generic semiring/additive cancellation and checked Nat order/Boolean reflection, only local sign cases, explicit actual selector/Bit/top/negative-balance equations. No caller truth flag, source adapter, new core rule/axiom/fuel or function extensionality.',
 'exact_import':'Private owning deterministic complete regeneration rejects altered/oversized/honestly different valid components. This is not a complete descriptor/private registry/context or activated source profile.'},
 'prerequisite_preservation':{'named_type_body_proof_level_family_metadata_matches':2163,'complete_signed_magnitude_raw_sha256':'5f3d83784114bfb471f9a8c305628c15c04e3f3b1398a566d4721c754912c976','reference_component':'word-signed-magnitude','unaffected_prior_proof_model_native_suites_repeated':False},
 'Rust_execution':seal(execution_path),'passed_affected_roots':4,'same_byte_paired_checkers':seal(paired_path),'same_byte_cases':60,'accepted_zero_axiom_exact_hashes':2,'rejected_core_type_mismatches':58,'core_mismatch_subtypes':dict(kinds),'negative_case_names':sorted(expected),'checks':checks,
 'initial_compilation_failure':{'record':seal(initial_path),'scope':'Exact original source snapshots/raw Rust E0382 ownership diagnostic retained. Borrowing observed.right after moving it prevents compilation; cloning the identical expression repairs ownership. No generated proof or semantic negative is claimed.'},'initial_passing_construction':seal(construction_path),'construction_scope':'A fixed-source one-root unoptimized construction checks exact import and every2163 prior declaration; all current production bytes match. Later tests add typed controls and actual Bool observations. Its full certificate equals the final bytes. No captured inputs change while pending.','same_final_byte_initial_Go_acceptance':seal(candidate_path),
 'finite_observations':{'comparison':seal(comparison_path),'native_execution':seal(root/'native/result.json'),'native_source':seal('develop/probes/go-practical/signed_bounds_observer.go'),'query_origins':seal(root/'query-origins.json'),'metrics':seal(metrics_path),'selected_original_queries':32,'actual_word_observations':96,'actual_Top_definition_applications':96,'actual_word_bit_observations':2880,'minimum_magnitude_cases':4,'zero_cases':4,'nonnegative_cases':metrics['nonnegative_cases'],'AST_steps':metrics['total_AST_steps'],'maximum_query_AST_steps':metrics['maximum_query_AST_steps'],'two_fresh_native_outputs_identical':True,'prepared_image_files':5577,'actual_Boolean_terms_executed':True,'Nat_half_low_complement_gap_guard_or_theorem_bodies_executed':False,'scope':metrics['scope']},
 'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json','CSharp_tests_executed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'authoritative_immutable_application_replay_claimed':False,'native_Linux_gate_claimed':False,'kernel_axiom_fuel_or_artifact_bounds_changed':False,
 'remaining_work':'Actual signed division/remainder with minimum/-one handling; remaining variable/signed/right-shift numeric/source laws, full source-typed carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All remaining T02-T11 gates remain.','full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
(root/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'declarations':2683,'terms':58000,'bytes':818605,'same_byte_cases':60,'semantic_negatives':58,'source_queries':32,'subtypes':dict(kinds)}))
