"""Preserve the tested scoped construction checkpoint and its targeted replay."""
from pathlib import Path
from hashlib import sha256
from datetime import datetime, timezone
import ast, json, re, shutil
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
root=Path('/private/tmp/mpk-w09-scoped-construction')
parent=repo/'develop/migrations/csharp-03/ordinary-foundation'
base=parent/'verification-logs/scoped-construction-proofs'
prior=parent/'verification-logs/construction-storage-types/tool-sources'
assert not base.exists()
base.mkdir(); (base/'tool-sources').mkdir(); (base/'local').mkdir()
manifest=json.loads((root/'source-manifest.json').read_bytes())
assert (len(manifest['source_hashes']),len(manifest['fixture_hashes']))==(749,173)
for group in ('source_hashes','fixture_hashes'):
 for name,digest in manifest[group].items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
assert manifest['fixture_hashes']==json.loads((prior.parent/'source-manifest.json').read_bytes())['fixture_hashes']
def catalog(folder):return {p.name:sha256(p.read_bytes()).hexdigest() for p in folder.iterdir() if p.is_file()}
expected=catalog(root/'full-1/certificates');legacy=catalog(root/'legacy-1/certificates')
assert len(expected)==70 and len(legacy)==92
assert legacy==catalog(parent/'verification-logs/concrete-allocation-proofs/attempt-5/certificates')
metadata=[json.loads(p.read_bytes()) for p in (root/'full-1/certificates').glob('*-program.json')]
assert len(metadata)==45 and sum(len(m['candidates']) for m in metadata)==12
assert sum(len(m['pending_source_operation_ids']) for m in metadata)==0
assert sum(len(m['generic_pending_operations']) for m in metadata)==25
assert sum(len(m['pending_proof_ids']) for m in metadata)==987
(base/'source-manifest.json').write_bytes((root/'source-manifest.json').read_bytes())
(base/'expected-certificates.json').write_text(json.dumps(expected,indent=2,sort_keys=True)+'\n')
for source,destination in [('full-1/certificates','local/certificates'),('legacy-1/certificates','local/legacy-certificates')]:shutil.copytree(root/source,base/destination)
selection='Test all 45 original contexts and 12 scoped fill/freeze source points with exact original W06 sequents, checked flow/receiver dependencies, strict imports, unchanged certificate prefixes and an actual-kernel typed ownership mutation. Include the legacy 442-proof allocation route and all 45 original operation pins because shared serialized operation/proof structures and domain-symbol lowering changed. Run crate Clippy and crate/support format. Generic ownership, original is_binding refinements, native execution and all 987 application proof IDs remain pending. The full T06 gate is deferred to T06-W12.'
testbase=['cargo','test','-p','mpk-vc','--test','csharp_practical_vc'];tail=['--','--nocapture','--test-threads=1']
jobs=[('scoped-construction-proofs','full-1','csharp_03_t06_w09_scoped_construction_operation_proofs_original_source',10112),('original-allocation-proofs','legacy-1','csharp_03_t06_w09_concrete_allocation_proofs_original_source',4836),('original-operation-pins','pins-1','csharp_03_t06_w09_concrete_operations_original_source_certificates',4836)]
stages=[];binaries={}
for label,logname,test,session in jobs:
 data=(root/(logname+'.log')).read_bytes();text=data.decode();assert 'test result: ok. 1 passed; 0 failed;' in text
 for binary in re.findall(r'Running [^\n]*\(([^\n]+)\)',text):binaries[binary]=sha256(Path(binary).read_bytes()).hexdigest()
 log=base/'local'/(label+'.log.txt');log.write_bytes(data)
 stages.append(dict(stage=label,command=testbase+[test]+tail,exit_code=0,terminal_exec_session=session,log_sha256=sha256(data).hexdigest()))
commands=[('clippy','clippy-1',['cargo','clippy','-p','mpk-vc','--lib','--tests','--','-D','warnings']),('format','format-1',['cargo','fmt','-p','mpk-vc','--','--check']),('support-format','support-format-1',['rustfmt','--edition','2021','--check','crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_operation_tests.rs','crates/mpk-vc/tests/support/csharp_practical_ordinary_scoped_construction_tests.rs'])]
for label,logname,command in commands:
 data=(root/(logname+'.log')).read_bytes();(base/'local'/(label+'.log.txt')).write_bytes(data)
 stages.append(dict(stage=label,command=command,exit_code=0,terminal_exec_session=4836,log_sha256=sha256(data).hexdigest()))
assert len(binaries)==1
state=dict(status='passed_targeted_tests_lint_format',recorded_at=datetime.now(timezone.utc).isoformat(),stages=stages,unique_tests_passed=3,test_binaries=binaries,selection_reason=selection,source_files_verified=749,fixture_files_verified=173,source_contexts=45,scoped_candidates=12,source_operations_pending=0,observed_operation_kinds=['fill','freeze'],generic_operations_pending=25,application_proofs_pending=987,original_is_binding_refinement_proofs_pending=7,complete_application_assembly_pending=True,full_t_gate='deferred to T06-W12',exported_certificate_hashes={'certificates/'+n:h for n,h in expected.items()},legacy_certificate_hashes=legacy,legacy_certificate_bytes_unchanged=True,build_environment=dict(CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',CARGO_PROFILE_TEST_OPT_LEVEL='2',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true'))
(base/'local/status.json').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n')
(base/'local/source-manifest.json').write_bytes((root/'source-manifest.json').read_bytes())
(base/'local/probe-1.log.txt').write_bytes((root/'probe-1.log').read_bytes())
(base/'local/build-attempts.json').write_text(json.dumps(dict(initial_compile_result='exit 101; accidental field initializer in tuple destructuring repaired before passing library check',initial_compile_terminal_session=91802,library_check_terminal_session=71724,library_check_exit_code=0,probe_terminal_session=90458,probe_exit_code=0,probe_filter='bool-construction',probe_is_not_final_source_verification=True,final_source_verification='all 45 contexts; session 10112 exit 0'),indent=2,sort_keys=True)+'\n')
runner=(prior/'run-targeted.py').read_text()
start=runner.index('"""');end=runner.index('"""',start+3)
runner='"""'+selection+'\n"""'+runner[end+3:]
old="""            ('storage-type-proofs', test + ['csharp_03_t06_w09_construction_type_proofs_original_source'] + tail, 1),
            ('private-storage-semantics', test + ['csharp_03_t06_w09_construction_storage_domains_original_semantics'] + tail, 1),
            ('original-type-proofs', test + ['csharp_03_t06_w09_concrete_type_proofs_original_source'] + tail, 1),
            ('original-type-pins', test + ['csharp_03_t06_w09_concrete_types_original_source_certificates'] + tail, 1),"""
new="""            ('scoped-construction-proofs', test + ['csharp_03_t06_w09_scoped_construction_operation_proofs_original_source'] + tail, 1),
            ('original-allocation-proofs', test + ['csharp_03_t06_w09_concrete_allocation_proofs_original_source'] + tail, 1),
            ('original-operation-pins', test + ['csharp_03_t06_w09_concrete_operations_original_source_certificates'] + tail, 1),"""
assert old in runner;runner=runner.replace(old,new)
runner=runner.replace("env.pop('MPK_W09_CONCRETE_TYPES_OUT', None)\n        env['MPK_W09_CONCRETE_TYPE_PROOFS_OUT'] = str(args.reports / 'certificates')","env.pop('MPK_W09_CONCRETE_TYPES_OUT', None)\n        env.pop('MPK_W09_CONCRETE_TYPE_PROOFS_OUT', None)\n        env.pop('MPK_W09_SCOPED_SOURCE_FILTER', None)")
old="env['MPK_W09_CONCRETE_TYPE_PROOFS_OUT'] = str(args.reports / ('certificates' if label == 'storage-type-proofs' else 'legacy-certificates'))"
new="env['MPK_W09_SCOPED_CONSTRUCTION_PROOFS_OUT'] = str(args.reports / 'certificates')\n            env['MPK_W09_CONCRETE_OPERATION_PROOFS_OUT'] = str(args.reports / 'legacy-certificates')"
assert old in runner;runner=runner.replace(old,new)
runner=runner.replace('csharp_practical_ordinary_concrete_type_tests.rs','csharp_practical_ordinary_concrete_operation_tests.rs').replace('csharp_practical_ordinary_construction_type_tests.rs','csharp_practical_ordinary_scoped_construction_tests.rs')
runner=runner.replace("glob('*.json')","glob('*-program.json')")
runner=runner.replace("supplied = sum(len(json.loads(p.read_bytes())['proofs']) for p in emitted)","supplied = sum(len(json.loads(p.read_bytes())['candidates']) for p in emitted)")
runner=runner.replace("pending_types = sum(len(json.loads(p.read_bytes())['pending_type_instances']) for p in emitted)","pending_source = sum(len(json.loads(p.read_bytes())['pending_source_operation_ids']) for p in emitted)\n        generic_pending = sum(len(json.loads(p.read_bytes())['generic_pending_operations']) for p in emitted)")
runner=runner.replace("assert (supplied, pending_types, pending_ids) == (87, 0, 987)","assert (supplied, pending_source, generic_pending, pending_ids) == (12, 0, 25, 987)")
runner=runner.replace('len(actual_catalog) == 92','len(actual_catalog) == 70').replace("len(state['legacy_certificate_hashes']) == 91","len(state['legacy_certificate_hashes']) == 92")
runner=runner.replace("state.update(supplied_concrete_type_proofs=supplied, concrete_type_instances_pending=pending_types, private_storage_domains=6, source_ownership_pending=True)","state.update(scoped_candidates=supplied, source_operations_pending=pending_source, generic_operations_pending=generic_pending, observed_operation_kinds=['fill','freeze'], source_ownership_pending=True)")
runner=runner.replace('unique_tests_passed=4','unique_tests_passed=3')
ast.parse(runner);(base/'tool-sources/run-targeted.py').write_text(runner)
for name in ['launch-linux.py','launch-wrapper.py','final-linux-fetch.py']:
 script=(prior/name).read_text().replace('mpk-w09-construction-storage-types','mpk-w09-scoped-construction').replace('verification-logs/construction-storage-types','verification-logs/scoped-construction-proofs')
 if name=='final-linux-fetch.py':
  script=script.replace("unique_tests_passed']==4","unique_tests_passed']==3").replace('unique_tests_passed=4','unique_tests_passed=3')
  script=script.replace("['storage-type-proofs','private-storage-semantics','original-type-proofs','original-type-pins','clippy','format','support-format']","['scoped-construction-proofs','original-allocation-proofs','original-operation-pins','clippy','format','support-format']")
  script=script.replace("assert state['supplied_concrete_type_proofs']==87\nassert state['concrete_type_instances_pending']==0 and state['private_storage_domains']==6 and state['source_ownership_pending'] is True and state['application_proofs_pending']==987", "assert state['scoped_candidates']==12 and state['source_operations_pending']==0\nassert state['generic_operations_pending']==25 and state['source_ownership_pending'] is True and state['application_proofs_pending']==987")
  script=script.replace('len(catalog)==92','len(catalog)==70').replace('concrete-type-proofs/attempt-3/certificates','concrete-allocation-proofs/attempt-5/certificates').replace('len(legacy)==91','len(legacy)==92')
  script=script.replace("glob('*.json')","glob('*-program.json')").replace("assert sum(len(m['proofs']) for m in metadata)==87\nassert sum(not m['proofs'] for m in metadata)==10\nassert sum(len(m['pending_type_instances']) for m in metadata)==0", "assert sum(len(m['candidates']) for m in metadata)==12\nassert sum(len(m['pending_source_operation_ids']) for m in metadata)==0\nassert sum(len(m['generic_pending_operations']) for m in metadata)==25")
  script=script.replace("domains=[d for m in metadata for d in m.get('construction_storage_domains',[])]\nassert len(domains)==6 and all(d['private_storage_only'] and d['ownership_pending'] for d in domains)","domains=[d for m in metadata for c in m['candidates'] for d in c['program'].get('construction_storage_domains',[])]\nassert domains and all(d['private_storage_only'] and d['ownership_pending'] for d in domains)")
  script=script.replace("source_contexts=45,supplied_concrete_type_proofs=87,\n           contexts_with_empty_type_group=sum(not m['proofs'] for m in metadata),concrete_type_instances_pending=0,private_storage_domains=6,source_ownership_pending=True,", "source_contexts=45,scoped_candidates=12,source_operations_pending=0,\n           contexts_with_scoped_candidates=sum(bool(m['candidates']) for m in metadata),generic_operations_pending=25,source_ownership_pending=True,")
 ast.parse(script);(base/'tool-sources'/name).write_text(script)
for name in ['check.py','prepare-checkpoint.py']:
 shutil.copyfile(root/name,base/'tool-sources'/name)
verification=dict(status='passed_local_targeted_verification',recorded_at=datetime.now(timezone.utc).isoformat(),unique_tests_passed=3,source_contexts=45,scoped_candidates=12,observed_operation_kinds=['fill','freeze'],source_operations_pending=0,generic_operations_pending=25,application_proofs_pending=987,original_is_binding_refinement_proofs_pending=7,complete_application_assembly_pending=True,source_files_verified=749,fixture_files_verified=173,exported_files_verified=70,legacy_exported_files_verified=92,legacy_certificate_bytes_unchanged=True,selection_reason=selection,local_receipt='local/status.json',dual_checker_verification='running; terminal independent audit pending',linux_verification='pending exact published-source terminal receipt',full_t_gate='deferred to T06-W12',w09_status='In progress',w10_w12_status='Blocked')
(base/'verification.json').write_text(json.dumps(verification,indent=2,sort_keys=True)+'\n')
review='''# Source-scoped construction operation proofs

This partial T06-W09 checkpoint adds an opt-in proof route for original invoked
construction operations at their exact source ownership points. Across all 45
original contexts it supplies 12 source-specific candidates (fill and freeze)
and leaves no eligible invoked source operation pending. Six contexts contain
these points. Generic construction operations remain pending: the root retains
all 25 original pending operations, including uninvoked operations, and all 987
application proof IDs. W09 stays In progress and W10-W12 stay Blocked; the full
T06 gate is deferred to T06-W12.

## Scope and construction

Each candidate retains its complete original DataOperationVc and checks the
original signature, function/node/receiver identity and symbolic ownership
state. The scoped failure definition contains checked Let dependencies on the
existing whole-flow theorem and exact receiver-point theorem, and returns the
existing closed ownership witness. Its only parameter is the original receiver;
there is no carrier owner bit or caller-supplied capability.

The unchanged construction storage domain enforces complete length, initialized
cells, zero uninitialized storage, bitmap, padding and recursive child validity.
It explicitly retains private_storage_only=true and ownership_pending=true.
The adapter resolves only the ownership failure at its exact source point. It
preserves the complete original operation argument/result types, normal recipe,
ordered remaining failures and every original W06 subject, premise, guard and
goal. The operation proof includes the complete ordinary equation with all
previous supplied operation proofs. Its strict importer regenerates the exact
source metadata and certificate vector. The old terms/declarations, public
domains, source clauses and observations remain unchanged.

These source-specific candidates do not establish generic ownership, native
execution or application assembly. The 25 generic operations and seven original
is_binding refinements remain unresolved. Existing exact source points cover
fill and freeze; no read/rewrite coverage is claimed. Foundation schemas,
Certificate v0, kernels and checker acceptance rules are unchanged.

## Verification and direct review

Three targeted local tests pass: all 45 source contexts and 12 scoped candidates,
the affected legacy 442-operation allocation proof route and all 45 original
operation pins. Shared serialized operation/proof structures and domain-symbol
lowering changed, so both legacy consumers are included. Crate Clippy with
warnings denied, crate format and both changed support-file format checks pass.
The 749-source/173-fixture audit passes. All 70 new exports are retained, and
all 92 legacy exports exactly match the published allocation checkpoint.

The source test independently computes eligible invoked operations from the
original full VC, matches every source descriptor, compares complete W06
sequents and old definitions/conditions, and inspects both checked Let
dependencies and the original certificate prefix. The actual Rust kernel accepts
every candidate with zero axioms and empty proof-node/theory-certificate tables.
A well-typed always-true concrete ownership failure is accepted in the
definition-only certificate and rejected at core checking with supplied operation
proofs. Receiver metadata and incomplete certificate-vector mutations reject.

An initial tuple-destructuring compile error was repaired before the successful
library check. A passing Bool-only prototype is retained separately; the final
verification reruns every original source after the final metadata/test changes.

Same-byte Go/Rust verification is running; its terminal independent audit remains
pending. Requested Linux replay awaits an exact published-source receipt.
See verification-logs/scoped-construction-proofs/.
'''
(parent/'unit-7-scoped-construction-proofs-review.md').write_text(review)
todo=repo/'develop/docs/08_csharp_practical_subset_design-todo.md';text=todo.read_text();anchor='Owns: translation of every already-expanded concrete foundation definition in\n';assert text.count(anchor)==1
addition='''The opt-in source-scoped construction operation route now supplies 12 candidates
at exact original source ownership points across all 45 contexts. All eligible
invoked fill/freeze operations have candidates; each retains the complete original
DataOperationVc and W06 operation sequent, checked whole-flow and receiver-point
proof dependencies, the private storage domain and original ordered failures.
Three targeted tests, Clippy and both format checks pass locally. All 70 new
exports are retained, all 92 legacy allocation exports remain byte-identical and
all 45 original operation pins are unchanged. The tested audit verifies 749
source and 173 fixture hashes. Same-byte Go/Rust and exact published-source Linux
terminal receipts are pending. Generic ownership, all 25 original pending
operations, seven is_binding refinements and all 987 application proof IDs
remain pending. W09 stays In progress, W10-W12 stay Blocked and the full T06 gate
is deferred to W12. See
`ordinary-foundation/unit-7-scoped-construction-proofs-review.md`.

'''
todo.write_text(text.replace(anchor,addition+anchor))
print(json.dumps(dict(status='preserved',source_files=749,fixture_files=173,local_tests=3,source_candidates=12,exported_files=70,legacy_files_unchanged=92)))
