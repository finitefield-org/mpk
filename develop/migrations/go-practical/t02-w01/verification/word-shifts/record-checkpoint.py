"""Seal scoped actual fixed-right and full-count prerequisite observations."""
from pathlib import Path
import collections,datetime,hashlib,json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def execution(p,current=True):
 r=read(p);assert r['status']=='passed' and r['exit_code']==0 and not r['changed_inputs_during_execution']
 for s in r['sources']:
  snapshot=seal(p.parent/'inputs'/s['path']);assert (s['sha256'],s['bytes'])==(snapshot['sha256'],snapshot['bytes'])
  if current:assert seal(s['path'])==s
 return r
p=root/'metadata-corrected/rust-tests/result.json';r=execution(p);stdout=p.parent/'stdout.log';text=stdout.read_text()
assert '1 passed; 0 failed' in text and 'matched exact2669 original named shift prerequisites' in text
cert=p.parent/'certificates/word-shifts.mpcert';report=read(cert.with_suffix('.rust.json'))
assert report['verdict']=='accepted' and report['declaration_count']==3105 and report['axiom_count']==0
assert report['hashes']['certificate']=='a3017e8969d14663af2340b4d72b5fcec21b0d0ead6d3ad05a0b5bd7c62350a2'
assert cert.stat().st_size==1100109 and cert.read_bytes()==(root/'applied-count-cases/rust-construction/certificates/word-shifts.mpcert').read_bytes()==(root/'final/rust-tests/model/certificate.mpcert').read_bytes()
execution(root/'applied-count-cases/rust-construction/result.json',False)
paired=read(root/'paired-final/result.json');assert paired['status']=='passed' and len(paired['cases'])==103
positives=set();negatives=set();kinds=collections.Counter()
for case in paired['cases']:
 a,b=case['rust_observation'],case['go_observation'];assert a['verdict']==b['verdict']
 if a['verdict']=='accepted':assert a['hashes']==b['hashes'] and a['axiom_count']==b.get('axiom_count',0)==0;positives.add(case['name'])
 else:
  detail=json.loads(a['error_detail']);assert a['error_code']=='KERNEL_CORE_CHECK' and detail['code']=='CORE_TYPE_MISMATCH';kinds[detail['details']['kind']]+=1;negatives.add(case['name'])
assert positives=={'word-shifts','honest-different-shifts'} and len(negatives)==101
expected=set()
for n in ['PowerMinusBalance','PowerMinusAdd','PowerPositive','PopConservation6','CubeValueBound6','RightCompose','CountSumZero','CountNonzeroPositive','WeightedBitZero','CountDeMorgan']:expected.add('False'+n)
for n,i in [('RightCompose',11),('RightCompose',12),('WeightedBitZero',2),('CountPairZero',4),('CountPairZero',5)]:expected.add(f'FakePremise-{n}-{i}')
for w in [8,16,32,64]:
 for j in range(w.bit_length()-1):
  for n in ['UnsignedRightConservation','SignedRightConservation','RemainderBound']:expected.add(f'False{n}-W{w}-{1<<j}')
 for n in ['FalseSignedRightTop','FalseFilledRight','WrongActualRight','WrongActualSignedRight']:expected.add(f'{n}-W{w}')
for d in range(3,7):
 for n in ['FalseCountBalance','FalseCountHighZero','WrongCountHigh','WrongCountHighValue']:expected.add(f'{n}-{d}')
assert negatives==expected
checks={}
for mode in ['clippy','format','diagnostic-list']:
 q=root/'current-final'/mode/'result.json';execution(q);checks[mode]=seal(q)
assert 'prefix=' not in (root/'current-final/diagnostic-list/stdout.log').read_text()
gof=read(root/'current/go-format/result.json');assert gof['status']=='passed' and seal(gof['source']['path'])==gof['source'];checks['Go-format']=seal(root/'current/go-format/result.json')
comparison=read(root/'native-comparison-final/result.json');assert comparison['status']=='passed' and comparison['defined_counts']==480 and comparison['negative_count_native_panics']==24 and comparison['defined_high_counts']==309
metrics=read(root/'final/rust-tests/model/metrics.json');assert metrics['original_shift_queries']==504 and metrics['actual_fixed_right_observations']==4212 and metrics['Nat_Int_witness_or_theorem_bodies_executed'] is False
previous=read(root.parent/'word-conversions/frozen-design-preservation.json');assert previous['status']=='passed' and len(previous['inputs'])==255
for s in previous['inputs']:assert seal(s['path'])==s
write(root/'frozen-design-preservation.json',{**previous,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'All255 original design/specification/vector inputs preserved exactly. Current fixed-right/count prerequisite certificate remains within original4096/131072/2097152 limits. Variable stage/saturation/full source/application work remains due.'})
failures=[]
for attempt in ['actual-count-balance','explicit-count-balance']:
 q=root/attempt/'rust-construction/result.json';f=read(q);assert f['status']=='failed' and f['exit_code']==101 and not f['changed_inputs_during_execution']
 for s in f['sources']:
  o=seal(q.parent/'inputs'/s['path']);assert (o['sha256'],o['bytes'])==(s['sha256'],s['bytes'])
 row={'execution':seal(q),'semantic_negative_counted':False}
 if (root/attempt/'go-candidate/result.json').exists():row['same_original_byte_Go_observation']=seal(root/attempt/'go-candidate/result.json')
 failures.append(row)
diag=root/'first-count-type/rust-diagnostic/diagnostic.json';d=read(diag);assert d['first_failed_declaration_index']==3086 and d['first_failed_declaration_name']=='Mpk.Go.Practical.Word.Shifts.WeightedBitZero'
record={'schema':'mpk.go_practical.word_shifts_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entry_commit':'7b7eec1fbb443f85cbcb317354e7fc18c24c1124','fresh_component':{'certificate':seal(cert),'hashes':report['hashes'],'declarations':3105,'terms':77415,'bytes':1100109,'axioms':0,'exact_original_prerequisites':2669,'complete_prior_conversions':1882,'new_declarations':436,'remaining_declaration_slots':991,'frozen_candidate_bounds':{'declarations':4096,'terms':131072,'bytes':2097152},'all_bounds_unchanged_and_met':True},'production_sources':[s for s in r['sources']if s['path'].startswith('crates/mpk-vc/src/')],'test_and_original_inputs':[s for s in r['sources']if not s['path'].startswith('crates/mpk-vc/src/')],'repaired_single_affected_Rust_root':seal(p),'original_combined_Rust_execution':seal(root/'final/rust-tests/result.json'),'affected_unique_roots_completed':3,'checks':checks,'paired_same_byte_checkers':seal(root/'paired-final/result.json'),'same_byte_cases':103,'accepted_zero_axiom_positives':2,'genuine_core_type_mismatches':101,'mismatch_subtypes':dict(kinds),'negative_case_names':sorted(expected),'finite_comparison':seal(root/'native-comparison-final/result.json'),'model_metrics':seal(root/'final/rust-tests/model/metrics.json'),'native_execution':seal(root/'native/result.json'),'native_original_queries':504,'actual_fixed_right_finite_observations':4212,'source_defined_counts':480,'real_negative_count_panics':24,'source_defined_large_counts':309,'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'original_Rust_failures':failures,'original_first_failure_diagnostic':seal(diag),'diagnostic_prefix_acceptance_claimed':False,'exact_original_identity_observation':seal(root/'retained-identities.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json','proof_scope':'Ordinary unbounded PowerMinusOne/exponent addition/positivity; PopHigh conservation and generic cube bounds; actual power-of-two fixed right/ signed-right source bindings at all widths with actual low-block remainders, strict bounds and source top preservation. Full64 count decomposition U64=Digits_d+2^d*HighValue with the exact balanced source high-bit Or and zero reflection. All2669 original named dependencies retain complete type/body/proof/level/family metadata with original names; all1882 conversions and old signed-division raw factory pins remain exact. Every assembled term is checked by both unchanged cores; metadata/import ownership/native observations supply no proof authority.','captured_inputs_fixed_until_actual_terminal_handles':True,'CSharp_tests_executed':False,'kernel_axiom_fuel_or_artifact_bounds_changed':False,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'native_Linux_gate_claimed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'remaining_work':'Actual low-prefix strict bound, full count high/width equivalence/excess/power binding; all3 actual variable-stage left/unsigned-right/signed-right conservation and remainders, full64 saturation/sign-fill laws and signed Int projection. Full carrier/state/program/bounded56-family descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All T02-T11/W01 scope remains active.','full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
write(root/'verification.json',record);print(json.dumps({k:record[k]for k in ['status','same_byte_cases','genuine_core_type_mismatches','mismatch_subtypes','native_original_queries']}))
