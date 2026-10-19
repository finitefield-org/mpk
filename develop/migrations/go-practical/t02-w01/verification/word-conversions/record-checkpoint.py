"""Bind complete actual conversion proofs to their scoped local observations."""
from pathlib import Path
import collections
import datetime
import hashlib
import json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def current_sources(path,production_only=False):
 r=read(path);assert r['status']=='passed' and r['exit_code']==0 and r['changed_inputs_during_execution']==[]
 for source in r['sources']:
  s=seal(path.parent/'inputs'/source['path']);assert(s['sha256'],s['bytes'])==(source['sha256'],source['bytes'])
  if not production_only or source['path'].startswith('crates/mpk-vc/src/'):assert seal(source['path'])==source
 return r
execution_path=root/'final/rust-tests/result.json';execution=current_sources(execution_path)
stdout=(execution_path.parent/'stdout.log').read_text();assert '4 passed; 0 failed' in stdout and 'matched retained named conversion prerequisites=888' in stdout
certificate=execution_path.parent/'certificates/word-conversions.mpcert';report=read(certificate.with_suffix('.rust.json'))
assert report['verdict']=='accepted' and report['declaration_count']==1882 and report['axiom_count']==0
assert report['hashes']['certificate']=='fae13fa0cae53f8e7c851cf87d34fe5b3f12284b17c5a1ba5a9fdaee3c1267bf'
assert report['hashes']['export']=='ec264515374e18bada1d6b39f289c3e47246ea9c5a3230004b5115fc77da217c'
assert seal(certificate)['sha256']=='d8453a91807c0add99575b7b76a18cb9bf3e65d78895b5c8d4a5f75c1dcc09a1' and certificate.stat().st_size==483095
assert certificate.read_bytes()==(root/'actual-count-transport/rust-construction/certificates/word-conversions.mpcert').read_bytes()
assert certificate.read_bytes()==(execution_path.parent/'model/certificate.mpcert').read_bytes()
construction_path=root/'actual-count-transport/rust-construction/result.json';current_sources(construction_path,True)
candidate_path=root/'actual-count-transport/go-candidate/result.json';candidate=read(candidate_path)
assert candidate['exit_code']==0 and candidate['Rust_observation']['verdict']==candidate['Go_observation']['verdict']=='accepted'
assert candidate['Rust_observation']['hashes']==candidate['Go_observation']['hashes']==report['hashes']
paired_path=root/'paired-final/result.json';paired=read(paired_path);assert paired['status']=='passed' and len(paired['cases'])==172
expected={'FalseFill6','FalseMask36','FalseRatio63','FalseTruncate63','FakeNarrowDivision','FakeNarrowRadix','FakeWideExtension','FakeWideRadix','FalseScalar','FakeScalarBalance'}
for w in [8,16,32,64]:
 for t in [8,16,32,64]:
  for sg in [False,True]:
   op=('Signed' if sg else 'Unsigned')+'To'+str(t)
   labels=['WrongActualConversion','FalseUnsignedConservation','FalseSignedRefinement','FalseUnsignedRefinement']
   if w<=t:labels.append('FalseSignedExact' if sg else 'FalseUnsignedExact')
   expected.update(f'{label}-W{w}-{op}' for label in labels)
 expected.update(f'{label}-W{w}'for label in ['FalseSignedCountGuard','FalseSignedCountValue','FakeSignedCountPremise'])
assert len(expected)==170
observed=set();positives=set();kinds=collections.Counter()
for row in paired['cases']:
 rust,go=row['rust_observation'],row['go_observation'];assert rust['verdict']==go['verdict']
 if rust['verdict']=='accepted':
  assert rust['axiom_count']==go.get('axiom_count',0)==0 and rust['hashes']==go['hashes'];positives.add(row['name'])
 else:
  detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH';kinds[detail['details']['kind']]+=1;observed.add(row['name'])
assert expected==observed and positives=={'word-conversions','honest-different-conversions'}
checks={}
for label,path in [('strict_clippy',root/'current/clippy/result.json'),('Rust_format',root/'current/format/result.json'),('ignored_diagnostic_listing',root/'current/diagnostic-list/result.json')]:
 checked=current_sources(path);checks[label]={'status':'passed','record':seal(path),'seconds':checked['wall_seconds']}
listing=(root/'current/diagnostic-list/stdout.log').read_text();assert 'identify_original_first_failed_declaration_without_changing_the_checker: test' in listing and 'prefix=' not in listing
format_path=root/'current/go-format/result.json';formatted=read(format_path);assert formatted['status']=='passed' and seal(formatted['source']['path'])==formatted['source'];checks['new_Go_format']={'status':'passed','record':seal(format_path)}
identities_path=root/'retained-identities.json';identities=read(identities_path)
assert identities['all_retained_original_named_identities']==888 and identities['new_conversion_declarations']==994 and identities['certificate']['sha256']==seal(certificate)['sha256']
comparison_path=root/'native-comparison-final/result.json';comparison=read(comparison_path);metrics_path=root/'final/rust-tests/model/metrics.json';metrics=read(metrics_path)
assert comparison['status']=='passed' and metrics['Nat_Int_masks_ratios_quotients_or_theorem_bodies_executed'] is False
for key,count in [('actual_source_conversion_results',256),('actual_count_normalizations',256),('defined_counts',192),('actual_false_signed_count_guards',64),('signed_exact_type_cases',80),('unsigned_exact_type_cases',80)]:assert comparison[key]==metrics[key]==count
native_path=root/'native/result.json';native=read(native_path);assert native['status']=='passed' and native['native_rows']==256 and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries','complete_original_prepared_image']:assert seal(native[key]['path'])==native[key]
for check in native['complete_original_image_checks']:assert check['status']=='passed' and check['files']==5577
failure_records=[]
for attempt in ['actual-conversions','shared-literal-bridge','owned-argument','original-size-diagnostic','shared-scalar-laws','explicit-ones-equation','explicit-cube-transport','explicit-definition-transport','named-major-equations']:
 path=root/attempt/'rust-construction/result.json';r=read(path);assert r['status']=='failed' and r['exit_code']==101 and r['changed_inputs_during_execution']==[]
 for source in r['sources']:
  snapshot=seal(path.parent/'inputs'/source['path']);assert(snapshot['sha256'],snapshot['bytes'])==(source['sha256'],source['bytes'])
 for key in ['stdout','stderr']:assert seal(r[key]['path'])==r[key]
 row={'attempt':attempt,'original_Rust_execution':seal(path),'semantic_negative_counted':False}
 go_path=root/attempt/'go-candidate/result.json'
 if go_path.exists():
  g=read(go_path)
  for key in ['certificate','stdout','stderr','original_Rust_report']:assert seal(g[key]['path'])==g[key]
  assert g['Rust_observation']['verdict']==g['Go_observation']['verdict']=='rejected';row['same_original_byte_Go_observation']=seal(go_path)
 failure_records.append(row)
phase=(root/'original-size-diagnostic/rust-construction/stderr.log').read_text();assert 'before W64To16 signed=false: declarations=4056 terms=53970' in phase
original=read(root/'shared-scalar-laws/rust-construction/certificates/word-conversions.rust.json');assert json.loads(original['error_detail'])['code']=='CORE_FUEL_EXHAUSTED'
diagnostics=[]
for attempt,index,name in [('first-normalization-failure',1091,'FillValue6'),('first-type-failure',1103,'MaskBalance3To3'),('first-source-type-failure',1690,'W8.UnsignedCountGuard')]:
 path=root/attempt/'rust-diagnostic/result.json';r=read(path);assert r['status']=='passed' and r['changed_inputs_during_execution']==[]
 for source in r['sources']:
  snapshot=seal(path.parent/'inputs'/source['path']);assert(snapshot['sha256'],snapshot['bytes'])==(source['sha256'],source['bytes'])
 dpath=path.parent/'diagnostic.json';d=read(dpath);assert d['first_failed_declaration_index']==index and d['first_failed_declaration_name']=='Mpk.Go.Practical.Word.Conversions.'+name
 diagnostics.append({'execution':seal(path),'diagnostic':seal(dpath),'complete_prefix_or_semantic_negative_claimed':False})
bad_runner=root/'native-input-path/original-runner.py';fixed_runner=root/'run-native.py'
assert bad_runner.read_text().replace("Path('develop/migrations/go-practical/t02-w01/verification/word-word-conversions/queries.json')","root/'queries.json'")==fixed_runner.read_text()
previous=read(root.parent/'word-signed-division-bounds/frozen-design-preservation.json');assert previous['status']=='passed' and len(previous['inputs'])==255
for source in previous['inputs']:assert seal(source['path'])==source
frozen={**previous,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'All255 original frozen design/specification/vector inputs remain exact. All32 actual conversions/count-normalization laws fit unchanged4096/131072/2097152 bounds. Shared scalar proofs and explicit small cube/definition/actual Bool transports change recipes only; no checker/axiom/fuel/source policy widening.'}
write(root/'frozen-design-preservation.json',frozen)
record={'schema':'mpk.go_practical.actual_word_conversions_checkpoint.v1','status':'component_checks_passed_work_item_in_progress','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entry_commit':'3f494efa021f0f2c26983910735bb7a0bc641217',
'production_sources':[s for s in execution['sources']if s['path'].startswith('crates/mpk-vc/src/')],'current_target_test_model_and_inputs':[s for s in execution['sources']if not s['path'].startswith('crates/mpk-vc/src/')],
'fresh_component':{'certificate':seal(certificate),'hashes':report['hashes'],'declarations':1882,'terms':31495,'bytes':483095,'axioms':0,'widths':[8,16,32,64],'actual_conversion_operations':32,'retained_original_named_declarations':888,'new_conversion_declarations':994,'remaining_declaration_slots':2214,'frozen_candidate_bounds':{'declarations':4096,'terms':131072,'bytes':2097152},'all_bounds_unchanged_and_met':True,'exact_import':'Private owning deterministic complete regeneration compares every byte and rejects altered/oversized/honestly different valid components. This ownership supplies no core/source/program/profile authority.'},
'proof_scope':{
'actual_truncation':'Actual source converter literal bits match Low(source). Computed discarded quotient from real higher source cube values and exact radix ratio gives Usource=UactualResult+Mtarget*Q, with actual result<Mtarget. Signed/unsigned source circuits both bind; quotient is not caller supplied.',
'actual_extension':'Structural Fill cube has Ones*Bit(fill); Ones+1=Modulus. Computed mask obeys Msource+Mask=Mtarget. Actual unsigned extension gives Uresult=Usource, signed extension Uresult=Usource+Mask*actual sign bit. All source/target pairs are covered.',
'both_target_refinements':'Shared genuine NarrowCross/WidenCross and ScalarCorrection ordinary Nat/Int laws derive source desired unsigned/signed value Equivalent to actual target unsigned/signed interpretation plus Mtarget*computed balanced correction. Narrow correction positive=Q+target sign (or0), negative=Ratio*source sign (or0). Wide correction positive=target sign (or0), negative=source sign (or0). Exact source signed widening and unsigned widening hold including identity widths.',
'actual_count_normalization':'Actual signed To64 source count guard equals Not(source actual top); actual nonnegative guard yields U64=Usource. Unsigned To64 always preserves full source value. Unsigned8/16/32 extensions also satisfy the signed64 guard; unsigned64 high-bit counts remain source-defined via the unsigned route. Variable shift numerical correctness and full source count-type binding remain required.'},
'retained_identity_observation':seal(identities_path),'Rust_execution':seal(execution_path),'passed_affected_roots':4,'same_byte_paired_checkers':seal(paired_path),'same_byte_cases':172,'accepted_zero_axiom_exact_hashes':2,'rejected_core_type_mismatches':170,'core_mismatch_subtypes':dict(kinds),'negative_case_names':sorted(expected),'checks':checks,
'passing_fixed_source_construction':seal(construction_path),'passing_same_byte_initial_Go_inspection':seal(candidate_path),'construction_scope':'Initial one-root construction validates exact import/determinism and888 named original metadata plus complete original arithmetic/order raw pins. Production inputs and complete certificate bytes equal final four-root bytes. Test-only170controls/honest component/model expand after the actual construction handle terminates; original captured inputs remain. The standalone diagnostic is ignored from creation and unchanged.',
'captured_inputs_fixed_until_actual_terminal_handles':True,
'original_failures':failure_records,'direct_prefix_diagnostics':diagnostics,'repair_scope':'Original per-family repeated polynomial proofs hit4096 declarations during W64To16. Shared generic scalar laws retain all32 operations at1882 declarations. Original FillValue6 full candidates exhaust unchanged normalization fuel; explicit small cube equalities mapped through Value resolve it. OnesDefinition cancellation bridge alone was insufficient and its original failure remains. Actual mask/ratio/low/quotient definition equalities are transported through Nat recursor contexts. Actual count top equalities are mapped through Not before literal reduction. Initial compiler private-method/ownership and later partial edit/ownership failures remain with original snapshots/raw diagnostics. Every full intended-positive failed candidate and direct both-checker outcome stays preserved; none is counted among170 deliberate semantic controls. No checker, axiom, fuel or bound changes.',
'diagnostic_default_lifecycle':'Explicit separate helper is ignored from its creation. Current listing uses --ignored --list without prefix execution; selected Clippy covers main and helper and format covers captured sources. No auxiliary source transitions or exclusions are required.',
'finite_observations':{'comparison':seal(comparison_path),'native_execution':seal(native_path),'native_source':native['original_source'],'query_origins':seal(root/'query-origins.json'),'model_metrics':seal(metrics_path),'observations':metrics,'two_native_outputs_identical':True,'prepared_image_files':5577,'Nat_Int_masks_ratios_quotients_or_theorem_bodies_executed':False,'universal_conversion_laws_supplied_by_certificate_not_finite_model':True,'scope':'Original conversion input x is additionally used as a count with the source signedness specified by its conversion operation; all original query fields/words stay exact. Actual Bool To64/guard observations match real typed Go count-fault probes. Variable shifts and Nat/Int theorems do not execute in the finite model. Exactness flags denote type-level theorem applicability, not an iff over finite input results.'},
'native_pre_build_path_repair':{'original_runner':seal(bad_runner),'fixed_runner':seal(fixed_runner),'observed_tool_diagnostic':seal(root/'native-input-path/observed-tool-diagnostic.txt'),'annotation':seal(root/'native-input-path/annotation.json'),'scope':'A copied word-word-conversions query path failed before image creation/native work. Preserve exact original runner/copied source/tool-response text. Correct only query path to root/queries.json; subsequent single fresh build/two runs pass with full before/after prepared-image checks. Tool text is not claimed raw OS stderr. No native build/observations were repeated; not a semantic negative.'},
'frozen_design_preservation':seal(root/'frozen-design-preservation.json'),'strict_changed_input_review':'input-review/changed-inputs-review.json','CSharp_tests_executed':False,'T02_W01_complete':False,'all_application_demands_remaining':169,'public_profile_activated':False,'whole_source_program_proof_claimed':False,'authoritative_immutable_application_replay_claimed':False,'native_Linux_gate_claimed':False,'kernel_axiom_fuel_or_artifact_bounds_changed':False,
'remaining_work':'Variable/signed/right-shift numeric/source laws, full source/carrier/operation/storage/lifetime/program foundation, bounded56-family assembly/descriptor/private registry/context and authoritative original application source/export/selected-closure replay. All T02-T11 scope/gates remain.',
'full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
write(root/'verification.json',record)
print(json.dumps({'status':record['status'],'declarations':1882,'terms':31495,'bytes':483095,'same_byte_cases':172,'semantic_negatives':170,'subtypes':dict(kinds),'source_observations':256}))
