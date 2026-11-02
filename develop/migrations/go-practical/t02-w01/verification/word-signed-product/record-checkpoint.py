"""Bind the actual current signed-product observations; rerun no checks."""
from pathlib import Path
import collections,datetime,hashlib,json

root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
execution_path=root/'final/rust-tests/result.json';execution=read(execution_path)
assert execution['status']=='passed' and execution['exit_code']==0
stdout=(root/'final/rust-tests/stdout.log').read_text();assert '4 passed; 0 failed' in stdout and 'matched named prerequisites=2387' in stdout
for source in execution['sources']:
 assert seal(source['path'])==source
 copy=seal(root/'final/rust-tests/inputs'/source['path']);assert (copy['sha256'],copy['bytes'])==(source['sha256'],source['bytes'])
certificate=root/'final/rust-tests/certificates/word-signed-product-foundation.mpcert';report=read(certificate.with_suffix('.rust.json'))
assert report['verdict']=='accepted' and report['declaration_count']==2612 and report['axiom_count']==0 and certificate.stat().st_size==1032832
assert certificate.read_bytes()==(root/'actual-signed-product/rust-construction/certificates/word-signed-product-foundation.mpcert').read_bytes()
assert certificate.read_bytes()==(root/'final/rust-tests/model/certificate.mpcert').read_bytes()
paired_path=root/'paired-final/result.json';paired=read(paired_path);assert paired['status']=='passed' and len(paired['cases'])==33
expected={'WrongZeroSum','IgnoredMultiplierPositive','ZeroPower','WrongBalance','IgnoredUnsignedWitness','IgnoredModulusPositive','FalseReflection'}
expected|={f'{name}-{w}'for w in [8,16,32,64]for name in ['WrongPositiveCorrection','WrongNegativeCorrection','AlwaysTrueGuard','FakeTruePremise','FalseComplete','FalseIntegerEquality']}
observed=set();kinds=collections.Counter()
for case in paired['cases']:
 r=case['rust_observation']
 if r['verdict']=='rejected':
  d=json.loads(r['error_detail']);assert d['code']=='CORE_TYPE_MISMATCH';observed.add(case['name']);kinds[d['details']['kind']]+=1
assert expected==observed and sum(kinds.values())==31
comparison_path=root/'native-comparison-final/result.json';comparison=read(comparison_path)
assert comparison['status']=='passed' and comparison['actual_stages']==1440 and comparison['exact_signed_products']==21 and comparison['overflowed_signed_products']==27
metrics_path=root/'final/rust-tests/model/metrics.json';metrics=read(metrics_path);assert metrics['Nat_witness_correction_guard_or_theorem_bodies_executed'] is False
checks={}
for name,path in [('strict_clippy',root/'final/clippy/result.json'),('Rust_format',root/'final/format/result.json')]:
 check=read(path);assert check['status']=='passed'
 for source in check['sources']:assert seal(source['path'])==source
 checks[name]={'record':seal(path),'status':check['status'],'wall_seconds':check['wall_seconds']}
path=root/'final/go-format/result.json';check=read(path);assert check['status']=='passed' and seal(check['source']['path'])==check['source'];checks['new_native_Go_format']={'record':seal(path),'status':'passed','seconds':check['seconds']}
phases={
 'natural-cancellation/rust-construction/result.json':'Fresh closed arithmetic/Int prerequisites, genuine unbounded Nat zero-sum and positive multiplication cancellation, positive powers and all four positive moduli accept. All2387 named prerequisites and prior full byte hashes match. Later signed-product laws not yet attached.',
 'scalar-correction/rust-construction/result.json':'Generic signed correction balance, both exact-cross directions and Bool reflection agreement accept. All statements quantify over arbitrary Nat components; only local reflection flags use Bool cases.',
 'actual-signed-product/rust-construction/result.json':'All-width actual signed result, actual computed correction, exact guard reflection/completeness and agreement with balanced-Int equality accept. Exactly final certificate bytes; diagnostic unoptimized profile changes no core guard.',
}
assert all(read(root/p)['status']=='passed'for p in phases)
record={'schema':'mpk.go_practical.actual_signed_product_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'entry_commit':'9b4b8a0c1a8d71526d0957ded64883f7686d818f',
 'production_sources':[s for s in execution['sources']if s['path'].startswith('crates/mpk-vc/src/')],
 'test_model_and_query_sources':[s for s in execution['sources']if not s['path'].startswith(('crates/mpk-vc/src/','proofs/std/'))],
 'standard_raw_inputs':[s for s in execution['sources']if s['path'].startswith('proofs/std/')],
 'fresh_component':{'certificate':seal(certificate),'hashes':report['hashes'],'declarations':2612,'terms':80631,'bytes':1032832,'axioms':0,'retained_named_prerequisites':2387,'new_signed_product_declarations':225,'widths':[8,16,32,64],
  'frozen_candidate_bounds':{'bytes':2097152,'declarations':4096,'terms':131072},'all_bounds_unchanged_and_met':True,
  'actual_operand_interpretation':'S(word)=Int.Make(U(word),modulus*Bit(actual highest bit)), using the exact established signed projection declarations.',
  'actual_correction':'Let q be the existing ordinary Nat MulDiscard_width computed from every real fixed-shift/product carry, and sx/sy/sr the actual operand/result sign bits. C_positive=q+sr+modulus*sx*sy; C_negative=Ux*sy+Uy*sx; C=Int.Make(C_positive,C_negative). No caller quotient or truth flag supplies authority.',
  'unconditional_law':'Int.Equivalent(Int.Mul(S(x),S(y)),Int.Add(S(actual Mul(x,y)),Int.Mul(Int.FromNat(modulus),C))). Overflowing source multiplication is still a normal wrapped result.',
  'exact_result_guard':'Actual ordinary NoOverflow(x,y)=Nat.Equal(C_positive,C_negative). It implies actual mathematical Int product equals actual signed machine result, and that Int equivalence implies the guard is true. A further ordinary Bool theorem equates it to actual balanced-Int.Equal(product,result).',
  'positivity_and_cancellation':'Genuine arbitrary-Nat nested induction proves cancellation by any positive multiplier. Successor/zero contradiction is derived through checked Nat equality and ordinary Bool false-elimination. PowerTwo positivity uses genuine unbounded induction; exact old weight bridges prove every actual modulus positive. The reverse guard direction consumes this actual proof.',
  'construction':'Explicit checked positive/negative Int projection laws expose Nat cross-sums before generic checked semiring/cancellation. Only local reflection Bool flags are enumerated; full word operands are universal. No function extensionality, arithmetic opcode, new core/axiom/fuel or source adapter.',
  'exact_import':'Complete deterministic regeneration with owning bytes; altered/oversized/honestly different otherwise valid components reject. This component is not a complete descriptor/private registry/context or installed profile.'},
 'prerequisite_preservation':{'named_types_bodies_proofs_levels_metadata_match':2387,'arithmetic_reference':seal(root.parent/'word-arithmetic/verification.json'),'complete_arithmetic_raw_sha256':'2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c','unsigned_division_reference':seal(root.parent/'word-division/verification.json'),'complete_unsigned_division_raw_sha256':'64ef60d8a3517bede84945e89af06ab155ec04cc4346f1f828b20f8bbda4f462','unchanged_prior_suites_repeated':False,'scope':'Generate the established full arithmetic prefix before ordinary dependency selection, retaining exact internal checkpoint identities. Compare every retained named declaration independently; only regenerate/hash the unchanged complete prior arithmetic/unsigned division consumers, without rerunning their proof/model/native suites.'},
 'Rust_execution':seal(execution_path),'passed_affected_roots':4,'paired_current_checkers':seal(paired_path),'same_byte_cases':33,'accepted_zero_axiom_exact_hashes':2,'rejected_core_type_mismatch':31,'core_mismatch_subtypes':dict(kinds),'negative_case_names':sorted(expected),
 'initial_observations':[{'record':seal(root/p),'scope':scope}for p,scope in phases.items()],
 'same_final_byte_Go_construction_acceptance':seal(root/'actual-signed-product/go-candidate/result.json'),'checks':checks,
 'actual_finite_observations':{'comparison':seal(comparison_path),'native_execution':seal(root/'native/result.json'),'native_source':seal('develop/probes/go-practical/signed_product_observer.go'),'query_origins':seal(root/'query-origins.json'),'metrics':seal(metrics_path),'selected_original_queries':48,'actual_stages':1440,'actual_true_carries':107,'exact_signed_products':21,'overflowed_signed_products':27,'minimum_times_minus_one_cases':4,'AST_steps':metrics['total_AST_steps'],'maximum_query_AST_steps':metrics['maximum_query_AST_steps'],'two_native_outputs_byte_identical':True,'prepared_image_files':5577,'actual_Boolean_terms_executed':True,'Nat_witness_correction_guard_or_theorem_bodies_executed':False,'scope':comparison['scope']},
 'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json',
 'CSharp_tests_executed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'authoritative_immutable_application_replay_claimed':False,'native_Linux_gate_claimed':False,'kernel_axiom_fuel_or_artifact_bounds_changed':False,
 'remaining_work':'Actual signed negation/magnitude and signed division/remainder, variable/signed/right-shift numeric/source laws, remaining full source-typed carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All later T02-T11 gates remain.',
 'full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
(root/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'certificate':seal(certificate),'paired_cases':33,'core_negatives':31,'exact_products':21,'overflowed_products':27}))
