"""Bind current scoped division evidence without repeating any execution."""
from pathlib import Path
import collections
import datetime
import hashlib
import json

root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
execution_path=root/'reviewed/rust-tests/result.json';execution=read(execution_path)
assert execution['status']=='passed' and execution['exit_code']==0
assert '5 passed; 0 failed' in (root/'reviewed/rust-tests/stdout.log').read_text()
assert 'all 2198 retained prerequisites match prior named types/bodies/proofs/metadata' in (root/'reviewed/rust-tests/stdout.log').read_text()
for source in execution['sources']:
 assert seal(source['path'])==source
 copy=seal(root/'reviewed/rust-tests/inputs'/source['path']);assert (copy['sha256'],copy['bytes'])==(source['sha256'],source['bytes'])
certificate=root/'reviewed/rust-tests/certificates/word-division-foundation.mpcert';report=read(certificate.with_suffix('.rust.json'))
assert report['verdict']=='accepted' and report['declaration_count']==3588 and report['axiom_count']==0
assert certificate.stat().st_size==1131018
assert certificate.read_bytes()==(root/'reviewed/rust-tests/model/certificate.mpcert').read_bytes()
assert certificate.read_bytes()!=(root/'final/rust-tests/certificates/word-division-foundation.mpcert').read_bytes()
assert (root/'reviewed/rust-tests/model/model.json').read_bytes()==(root/'final/rust-tests/model/model.json').read_bytes()
paired_path=root/'paired-reviewed/result.json';paired=read(paired_path)
assert paired['status']=='passed' and len(paired['cases'])==24
expected={'FalseGap','IgnoredDoubleBound','IgnoredRemainderBound','IgnoredModulusBound','WrongHornerCombine','WrongStageCombine'}
expected|={f'{name}-{w}' for w in [8,16,32,64] for name in ['MissingOverflow','WrongFirstPrefix','FalseRemainderBound','FakeDivisorGuard']}
observed=set();kinds=collections.Counter()
for case in paired['cases']:
 r=case['rust_observation']
 if r['verdict']=='rejected':
  detail=json.loads(r['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH';observed.add(case['name']);kinds[detail['details']['kind']]+=1
assert expected==observed and sum(kinds.values())==22
comparison_path=root/'native-comparison-reviewed/result.json';comparison=read(comparison_path)
assert comparison['status']=='passed' and comparison['actual_division_stages']==2640
checks={}
for name,path in [('strict_clippy',root/'reviewed/clippy/result.json'),('Rust_format',root/'reviewed/format/result.json')]:
 check=read(path);assert check['status']=='passed'
 for source in check['sources']:assert seal(source['path'])==source
 checks[name]={'record':seal(path),'status':check['status'],'wall_seconds':check['wall_seconds']}
go_format_path=root/'final/go-format/result.json';go_format=read(go_format_path);assert go_format['status']=='passed' and seal(go_format['source']['path'])==go_format['source']
checks['new_native_Go_format']={'record':seal(go_format_path),'status':'passed','seconds':go_format['seconds']}
metrics_path=root/'reviewed/rust-tests/model/metrics.json';metrics=read(metrics_path)
assert metrics['actual_source_division_stages']==2640 and metrics['arbitrary_restore_queries']==34 and metrics['observed_arbitrary_overflow_bits']==10
assert metrics['Nat_prefix_denotation_order_or_theorem_bodies_executed'] is False
reproduction=root/'named-prerequisite-reproduction/rust-tests/result.json';assert read(reproduction)['status']=='failed'
assert 'Mpk.Go.Practical.Word.Numeric.CheckedStep0123' in (reproduction.parent/'stderr.log').read_text()
change={'status':'corrected_named_prerequisite_identity','original_passing_closed_component':seal(root/'final/rust-tests/result.json'),'focused_reproduction':seal(reproduction),
 'old_source':seal(root/'final/rust-tests/inputs/crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic.rs'),'current_source':seal('crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic.rs'),
 'current_five_root_execution':seal(execution_path),'named_retained_prerequisites_checked':2198,
 'old_certificate':seal(root/'final/rust-tests/certificates/word-division-foundation.mpcert'),'new_certificate':seal(certificate),
 'actual_Bool_model_outputs_byte_identical':True,'native_source_or_execution_repeated':False,
 'finding':'The first closed candidate is valid in both cores but a selected push proof uses an internal Numeric.CheckedStep name with a different type/proof from the established arithmetic component. By-name declaration comparison reproduces the mismatch at CheckedStep0123. Closed validity alone does not establish cross-component prerequisite identity.',
 'correction':'Run the exact established PowerTwo-then-Push construction prefix before closed dependency assembly. Unused power definitions/proofs are removed; all2198 retained named types, bodies, opaque proofs, levels and family metadata now match the two unchanged reference components. Same frozen artifact bounds and no altered core/axiom/fuel. Rerun all five affected roots and the exact current-byte paired matrix; retain unchanged original native source/build/output.'}
(root/'review/prerequisite-source-change.json').write_text(json.dumps(change,indent=2)+'\n')
phases={
 'closed-dependencies/rust-construction/result.json':'Initial2198-declaration closed assembly accepts and retains previous complete Order/Arithmetic generation hashes; this does not yet include division or by-name prerequisite compatibility.',
 'natural-bounds/rust-construction/result.json':'Genuine strict-gap and double-bit/divisor Nat bounds accept, with2263 declarations.',
 'actual-restore/rust-construction/result.json':'Original one-step proof supplies shifted input bound where subtraction-order requires its actual result bound; both cores reject by type mismatch. Original candidate/source/diagnostics remain; not a semantic negative.',
 'restoring-first-declaration/rust-diagnostic/result.json':'Direct in-memory prefix checks locate Restore00Bound; diagnostic prefixes are not complete canonical acceptances.',
 'actual-result-bound/rust-construction/result.json':'Correct actual Sub result ValueBound discharges all four-width arbitrary-remainder restore laws.',
 'complete-division/rust-construction/result.json':'Complete all-width draft before explicit folds has3468 declarations/77820 terms/1071600 bytes; first source remainder fold exhausts existing conversion fuel in both cores. No budget change or semantic negative.',
 'complete-first-declaration/rust-diagnostic/result.json':'Direct prefix diagnosis locates W8.StageRemainderFold0; source shift/decision/selection must use explicit congruence.',
 'explicit-stage-folds/rust-construction/result.json':'Separate source DividendStep/QuotientBit/remainder folds connect every120 stage; all-width full laws accept in both cores. By-name prerequisite compatibility is still not established here.',
 'final/rust-tests/result.json':'All four original roots,22 core type controls and complete Bool observation pass; original paired24 streams, Clippy/format/native comparison also pass. Later by-name review finds an internal helper identity collision.',
 'named-prerequisite-reproduction/rust-tests/result.json':'Focused new declaration-fingerprint test reproduces the genuine helper-name/type/proof mismatch; no kernel semantic rejection is claimed.',
}
assert not Path('crates/mpk-vc/tests/go_practical_word_division_diagnostic.rs').exists()
record={'schema':'mpk.go_practical.actual_unsigned_division_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'entry_commit':'f5bd2878b7923d8e6806e80020cc2c16580e52e7',
 'production_sources':[s for s in execution['sources'] if s['path'].startswith('crates/mpk-vc/src/')],
 'current_test_model_and_query_sources':[s for s in execution['sources'] if not s['path'].startswith(('crates/mpk-vc/src/','proofs/std/'))],
 'standard_raw_inputs':[s for s in execution['sources'] if s['path'].startswith('proofs/std/')],
 'fresh_component':{'certificate':seal(certificate),'hashes':report['hashes'],'declarations':3588,'terms':82264,'bytes':1131018,'axioms':0,'retained_prerequisites':2198,'new_division_declarations':1390,
  'widths':[8,16,32,64],'frozen_candidate_bounds':{'bytes':2097152,'declarations':4096,'terms':131072},'bounds_unchanged_and_met':True,
  'actual_source_stages':120,'balanced_MSB_prefix_blocks':236,
  'unbounded_generic_laws':'Strict Nat decomposition/gaps, double-remainder-plus-Bool bound, translated subtraction bound, full restoring-step conservation/bound including overflow and impossible high/carry case.',
  'prefix_law':'Every full word z has MSB Prefix_width(z)=balanced U(z). All120 actual source remainder stages preserve U(R_i)+U(y)*Prefix_i(raw quotient word)=Prefix_i(x), under actual DivideDefined(y)=true; all124 actual remainder bounds check.',
  'final_laws':'Actual DivideDefined(y)=true implies U(x)=U(y)*U(actual UnsignedDiv(x,y))+U(actual UnsignedRem(x,y)) and Nat.Less(U(actual UnsignedRem(x,y)),U(y))=true.',
  'actual_domain_binding':'The actual source nonzero guard implies positive Nat divisor. Actual IsZero false is derived from that same guard before selecting real quotient/remainder results; no caller quotient or unrelated truth flag supplies authority.',
  'source_fold_binding':'Every actual DividendStep_i, QuotientBit_i and Remainder_(i+1) is folded through explicit ordinary word/Bool congruence before Value, avoiding implicit nested-circuit conversion.',
  'exact_import':'Complete deterministic regeneration, owning bytes, altered/oversized/honestly different valid component rejection. No installed profile/complete private registry/context authority.'},
 'closed_ordinary_assembly':{'executor':seal('crates/mpk-vc/src/go_practical_ordinary/dependencies.rs'),'retained_named_prerequisites':2198,
  'scope':'Compiled-in roots only. Traverse every type, definition value, opaque theorem proof and constructor/recursor metadata dependency; retain complete primitive families. Remap names/globals/terms without redirecting references. Every retained proof is checked again in both cores. Independent normalized named-DAG fingerprints cover all2198 retained declarations against prior complete components, including proof-only dependencies. This is untrusted ordinary certificate assembly, not application verification closure or authority.',
  'reference_order_verification':seal(root.parent/'word-order/verification.json'),'reference_arithmetic_verification':seal(root.parent/'word-arithmetic/verification.json'),
  'preceding_complete_order_raw_sha256':'c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa','preceding_complete_arithmetic_raw_sha256':'2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c','complete_old_generation_bytes_unchanged':True,
  'source_review_correction':seal(root/'review/prerequisite-source-change.json')},
 'Rust_execution':seal(execution_path),'passed_affected_roots':5,'paired_current_checkers':seal(paired_path),'same_byte_cases':24,'accepted_zero_axiom_exact_hashes':2,'rejected_core_type_mismatch':22,'core_mismatch_subtypes':dict(kinds),'negative_case_names':sorted(expected),
 'initial_observations':[{'record':seal(root/p),'scope':scope} for p,scope in phases.items()],
 'original_typed_candidate_Go_observation':seal(root/'actual-restore/go-candidate/result.json'),'original_conversion_candidate_Go_observation':seal(root/'complete-division/go-candidate/result.json'),
 'original_explicit_fold_Go_acceptance':seal(root/'explicit-stage-folds/go-candidate/result.json'),'diagnostic_harness_exact_source':seal(root/'diagnostics/declaration-prefix-harness.rs'),'diagnostic_harness_removed_from_active_discovery':True,
 'checks':checks,'actual_finite_observations':{'comparison':seal(comparison_path),'native_execution':seal(root/'native/result.json'),'native_source':seal('develop/probes/go-practical/division_observer.go'),'query_origins':seal(root/'query-origins.json'),'model_metrics':seal(metrics_path),'original_queries':96,'actual_source_stages':2640,'runtime_zero_divisor_failures':8,'arbitrary_restore_queries':34,'actual_overflow_bits':10,'AST_steps':metrics['total_AST_steps'],'maximum_query_AST_steps':metrics['maximum_query_AST_steps'],'two_native_outputs_byte_identical':True,'complete_prepared_image_files':5577,'Bool_circuit_terms_executed':True,'Nat_prefix_denotation_order_or_theorem_bodies_executed':False,'scope':comparison['scope']},
 'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json',
 'CSharp_tests_executed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'authoritative_immutable_application_replay_claimed':False,'native_Linux_release_gate_claimed':False,'kernel_axiom_fuel_or_artifact_bounds_changed':False,
 'remaining_work':'Signed division/remainder and Int multiplication, variable/signed/right-shift numeric/source laws and remaining source-typed carriers/operations/storage/lifetimes/program foundation. Complete bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay remain due. All later T02-T11 gates remain.',
 'full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
(root/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'status':record['status'],'certificate':seal(certificate),'paired_cases':24,'core_negatives':22,'matched_prerequisites':2198,'actual_stages':2640}))
