"""Bind full-count actual source/signed proof terms to all scoped evidence."""
from pathlib import Path
import collections,datetime,hashlib,json,runpy
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def current(p):
 r=read(p);assert r['status']=='passed' and r['exit_code']==0 and not r['changed_inputs_during_execution']
 for s in r['sources']:
  assert seal(s['path'])==s;captured=seal(p.parent/'inputs'/s['path']);assert(captured['sha256'],captured['bytes'])==(s['sha256'],s['bytes'])
 return r
execution_path=root/'final/rust-tests/result.json';ex=current(execution_path);text=(execution_path.parent/'stdout.log').read_text();assert '2 passed; 0 failed' in text and 'all3105 previous named proof/definition/level/family identities match' in text
cert=execution_path.parent/'certificates/word-shift-counts.mpcert';report=read(cert.with_suffix('.rust.json'));assert report['verdict']=='accepted' and report['axiom_count']==0 and report['declaration_count']==4053 and cert.stat().st_size==1454895
assert report['hashes']['certificate']=='2431d715ee124e1a37b29458f045b523150989bd9f8d2ba17e680e2b3e3a4ac9' and report['hashes']['export']=='77ad63c2bf2bfa0c964873679f06faa6c515ebb40f37d6fc4c07fbdebdee330c'
assert cert.read_bytes()==(root/'actual-signed-interpretations/rust-construction/certificates/word-shift-counts.mpcert').read_bytes()
paired_path=root/'paired-final/result.json';paired=read(paired_path);assert paired['status']=='passed' and len(paired['cases'])==85
negative=set();positive=set();kinds=collections.Counter()
for r in paired['cases']:
 for k in ['certificate','rust_report','stdout','stderr']:assert seal(r[k]['path'])==r[k]
 a,b=r['rust_observation'],r['go_observation'];assert a['verdict']==b['verdict'] and r['same_original_bytes']
 if a['verdict']=='accepted':assert a['hashes']==b['hashes'] and a['axiom_count']==b.get('axiom_count',0)==0;positive.add(r['name'])
 else:
  d=json.loads(a['error_detail']);assert a['error_code']=='KERNEL_CORE_CHECK' and d['code']=='CORE_TYPE_MISMATCH';negative.add(r['name']);kinds[d['details']['kind']]+=1
assert negative==runpy.run_path(str(root/'planned-controls.py'))['names']() and positive=={'word-shift-counts','honest-different-count-shifts'}
checks={}
for mode in ['clippy','format','diagnostic-list']:
 p=root/'current'/mode/'result.json';current(p);checks[mode]=seal(p)
assert 'prefix=' not in (root/'current/diagnostic-list/stdout.log').read_text()
identities=read(root/'retained-identities.json');assert identities['current_declarations']==4053 and [r['all_named_types_bodies_proofs_levels_families_equal']for r in identities['comparisons']]==[3105,3240,3352,3542,3685]
finite=read(root/'finite-counterparts/result.json');assert finite['status']=='passed' and finite['rows']==504 and finite['negative_native_count_panics']==24 and finite['defined_high_counts']==309 and finite['actual_stage_observations']==2586 and finite['symbolic_huge_powers']==96
assert finite['fresh_native_build_or_old_Bool_rerun'] is False and finite['new_Nat_Int_witness_or_theorem_bodies_executed'] is False
for key in ['old_native_execution','old_actual_Bool_and_Python_comparison','original_queries','counterparts']:assert seal(finite[key]['path'])==finite[key]
frozen=read(root.parent/'word-shifts/frozen-design-preservation.json');assert frozen['status']=='passed' and len(frozen['inputs'])==255
for s in frozen['inputs']:assert seal(s['path'])==s
write(root/'frozen-design-preservation.json',{**frozen,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Full actual variable shifts/signed interpretations remain within frozen4096/131072/2097152 and unchanged1024 core fuel. All255 original inputs remain exact; no source/JSON/directory exclusions or gate changes.'})
failures=[]
for attempt,mode in [('actual-right-stages','rust-construction'),('first-right-stage-type','rust-diagnostic'),('actual-full64-shifts','rust-construction')]:
 p=root/attempt/mode/'result.json';r=read(p);assert r['status']=='failed' and r['exit_code']==101 and not r['changed_inputs_during_execution']
 for s in r['sources']:
  saved=seal(p.parent/'inputs'/s['path']);assert(saved['sha256'],saved['bytes'])==(s['sha256'],s['bytes'])
 row={'original_execution':seal(p),'semantic_negative_counted':False}
 gp=root/attempt/'go-candidate/result.json'
 if gp.exists():row['same_original_byte_Go_observation']=seal(gp)
 failures.append(row)
diag=root/'original-right-stage-type/rust-diagnostic/diagnostic.json';d=read(diag);assert d['first_failed_declaration_index']==3403 and d['first_failed_declaration_name']=='Mpk.Go.Practical.Word.Shifts.W8.SignedRightStageTop1'
original=read(root/'actual-right-stages/go-candidate/result.json');assert original['Rust_observation']['verdict']==original['Go_observation']['verdict']=='rejected' and json.loads(original['Rust_observation']['error_detail'])['code']=='CORE_TYPE_MISMATCH'
record={'schema':'mpk.go_practical.actual_full64_shift_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entry_commit':'252af78ccd44cd49ac92d478115bf59e24d42c5c','fresh_component':{'certificate':seal(cert),'hashes':report['hashes'],'declarations':4053,'terms':101285,'bytes':1454895,'axioms':0,'previous_complete_shift_declarations_preserved':3105,'new_declarations':948,'remaining_declaration_slots':43,'frozen_candidate_bounds':{'declarations':4096,'terms':131072,'bytes':2097152},'normalization_fuel':1024,'all_production_limits_unchanged':True},'production_sources':[s for s in ex['sources']if s['path'].startswith('crates/mpk-vc/src/')],'test_and_original_inputs':[s for s in ex['sources']if not s['path'].startswith('crates/mpk-vc/src/')],'Rust_execution':seal(execution_path),'affected_Rust_roots':2,'same_byte_paired_cores':seal(paired_path),'same_byte_streams':85,'accepted_zero_axiom_positives':2,'genuine_core_type_mismatches':83,'mismatch_subtypes':dict(kinds),'negative_case_names':sorted(negative),'checks':checks,'prior_component_metadata_comparison':seal(root/'retained-identities.json'),'finite_counterparts':seal(root/'finite-counterparts/result.json'),'reused_exact_native_and_Bool_queries':504,'actual_stage_observations':2586,'negative_native_count_panics':24,'defined_high_counts':309,'huge_count_symbolic_counterparts':96,'fresh_native_or_old_Bool_model_rerun':False,'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'original_failures':failures,'ignored_direct_diagnostic':seal(diag),'prefix_acceptance_claimed':False,'outer_observation_wait_policy':seal(root/'observer-wait-policy/annotation.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json','proof_scope':'Actual full64 source-count branch equivalence and computed excess/power identities; original source ShiftLeft/ShiftRight/SignedShiftRight staged selection and exact final zero/sign fill. Actual computed unsigned-left discard, both filled-right remainders, strict bounds and signed output-top preservation. Actual signed modular-left correction and signed floor-right Int equivalence use old genuine SignedPositive/Negative and Int constructor projection equations, never truncating division for negative signed right operands. All arbitrary8/16/32/64-bit input words and full64 unsigned counts are covered without clipping or huge unary literals. Signed source count guards/normalizations remain existing real converter proofs; branch predicate is not an operation-domain restriction. All3105 prior proof/definition identities and the full old conversion raw generator remain exact. Every proof term is inspected by both unchanged production cores; metadata, owning imports and finite counterparts supply no proof authority.','repair_scope':'Original right stage complete candidate is rejected by both cores at W8.SignedRightStageTop1. Source Word.Select chooses whole functions; applying a selected function to a top address needs an ordinary Bool-case SelectTop theorem, not a computed-major/pointwise conversion rule. Added that real theorem without changing either core. Initial ignored helper relative path fails before prefix checking; repaired repository path and source/old3352 artifact capture. Original private saturation-entry Rust visibility error is repaired with a private parent wrapper, without term changes. All original sources/candidates/raw output stay preserved; no format/resource failure is counted as semantic negative. Outer subprocess observation wait300s is separate from unchanged production limits and makes no resource/Linux-release admission claim.','captured_inputs_fixed_until_actual_terminal_handles':True,'CSharp_tests_executed':False,'core_axiom_fuel_or_artifact_bounds_changed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'immutable_application_replay_claimed':False,'native_Linux_gate_claimed':False,'remaining_work':'Complete source/carrier/type/operation/storage/lifetime/state/program foundation; bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. Full original T02-T11/W01 scope remains required; all169 demands remain unelaborated.','full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
write(root/'verification.json',record);print(json.dumps({k:record[k]for k in ['status','same_byte_streams','genuine_core_type_mismatches','mismatch_subtypes','reused_exact_native_and_Bool_queries']}))
