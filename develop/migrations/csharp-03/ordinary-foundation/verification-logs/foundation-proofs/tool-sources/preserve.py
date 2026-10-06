from pathlib import Path
from hashlib import sha256
from datetime import datetime, timezone
import json, shutil
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
root=Path('/private/tmp/mpk-w09-foundation-proofs')
parent=repo/'develop/migrations/csharp-03/ordinary-foundation'
base=parent/'verification-logs/foundation-proofs'
assert not base.exists()
base.mkdir();(base/'tool-sources').mkdir();(base/'local').mkdir()
manifest=json.loads((root/'source-manifest.json').read_bytes())
for group in ('source_hashes','fixture_hashes'):
 for p,h in manifest[group].items():assert sha256((repo/p).read_bytes()).hexdigest()==h,p
assert (len(manifest['source_hashes']),len(manifest['fixture_hashes']))==(751,173)
audit=json.loads((root/'local/final-audit.json').read_bytes())
assert audit['status']=='passed' and audit['terminal_exec_session']==29306
for name in ('source-manifest.json','expected-certificates.json','status.json','final-audit.json'):
 shutil.copyfile(root/'local'/name,base/'local'/name)
shutil.copyfile(root/'source-manifest.json',base/'source-manifest.json')
shutil.copyfile(root/'local/expected-certificates.json',base/'expected-certificates.json')
for folder in ('certificates','type-certificates','operation-certificates'):
 shutil.copytree(root/'local'/folder,base/'local'/folder)
for stage in audit['unique_tests_passed'] and json.loads((root/'local/status.json').read_bytes())['stages']:
 shutil.copyfile(root/'local'/(stage['stage']+'.log'),base/'local'/(stage['stage']+'.log.txt'))
for attempt in ('probe-1','probe-2','attempt-1','attempt-2'):
 dest=base/'local'/attempt;dest.mkdir()
 for name in ('test.log','status.json','source-manifest.json','final-audit.json'):
  source=root/attempt/name
  if source.exists():shutil.copyfile(source,dest/(name+'.txt' if name.endswith('.log') else name))
 for source in (root/attempt).glob('*.log'):
  shutil.copyfile(source,dest/(source.name+'.txt'))
# Preserve the exact two unchanged component outputs by hash and complete logs,
# while the optional local reuse optimization keeps its binary outside Git.
previous=json.loads((root/'attempt-2/status.json').read_bytes())
archive={Path(binary).name:h for binary,h in previous['test_binaries'].items()}
(base/'local/attempt-2/retained-test-binary.json').write_text(json.dumps(dict(sha256=archive,retained_local_directory=str(root/'attempt-2/test-binaries'),not_committed=True),indent=2,sort_keys=True)+'\n')
for name in ('build-attempts.json',):shutil.copyfile(root/name,base/'local'/name)
for name in ('run-targeted.py','check.py','audit-local.py','preserve.py'):
 shutil.copyfile(root/name,base/'tool-sources'/name)
verification=dict(status='passed_local_targeted_verification',recorded_at=datetime.now(timezone.utc).isoformat(),
 source_files_verified=751,fixture_files_verified=173,source_contexts=45,supplied_type_proofs=87,
 supplied_operation_proofs=442,supplied_binding_sequents=529,remaining_binding_sequents=458,
 original_application_proofs_pending=987,generic_operations_pending=25,application_scope_pending=True,
 complete_application_assembly_pending=True,local_audit='local/final-audit.json',local_receipt='local/status.json',
 unique_tests_passed_final_source=1,unique_unchanged_component_tests_reused=2,
 exported_files_verified=91,legacy_type_exports_unchanged=92,legacy_operation_exports_unchanged=92,
 dual_checker_verification='running; terminal audit pending',linux_verification='pending exact published-source replay; previous server observation timed out during SSH banner exchange',
 w09_status='In progress',w10_w12_status='Blocked',full_t_gate='deferred to T06-W12',
 selection_reason=(root/'selection.txt').read_text().strip(),
 final_selection_reason=json.loads((root/'local/status.json').read_bytes())['reused_unchanged_component_receipt']['reason'])
(base/'verification.json').write_text(json.dumps(verification,indent=2,sort_keys=True)+'\n')
review='''# Integrated original foundation proofs

This partial T06-W09 checkpoint assembles all original concrete-type and currently
supplied concrete-operation proofs in one ordinary Certificate v0 context.
Across all 45 original source contexts it supplies 87 type and 442 operation
proofs, with an ordered union of 529 original W06 sequent IDs and an explicit
458-ID remaining list. All 987 original application proof IDs and all 25 generic
pending operations remain pending for complete application assembly. W09 is In
progress; W10-W12 remain Blocked and the full T06 gate is deferred to T06-W12.

The adapter resumes the entire allocation-inclusive checked operation program,
retaining every original term, declaration and proof. It independently rebuilds
the original type definitions on the same unchanged source clauses/public
domains, appends the complete private construction storage domains and supplies
the exact original universal type equations through the existing proof generator.
Both complete W06 sequent families retain their original subjects, premises,
guards and goals. A source-free registered Bool connective lowers the Boolean
type comparisons even when Bool is absent from the source carriers. Private
storage still has private_storage_only=true and ownership_pending=true. There
is no generic owner bit, currency assumption, new kernel rule or axiom.

The strict importer regenerates source/foundation/VC linkage, complete component
metadata, certificate bytes and the ordered supplied/remaining partition. Actual
term, declaration and binder limits are enforced by the existing builder; the
adapter also checks the cumulative original-plus-appended transformer count.
The supplied IDs are component candidates and do not establish native source,
binding reconstruction, pattern refinements, admission, replay or applications.

## Verification and review

All 45 source contexts pass actual Rust kernel checking, with zero axioms and
empty proof-node/theory-certificate tables. The source test independently
compares every sequent against the original full VC, preserves the old checked
operation certificate prefix, inspects the exact named theorem types and both
complete operands of every type equation, verifies strict/cross-context imports
and rejects metadata/missing-certificate mutations. A well-typed false concrete
type predicate is accepted before its type proofs and rejected at core checking
with the proofs. All 91 new exports are retained.

The two affected standalone component tests pass and preserve all 92 original
storage-type and all 92 allocation-operation exports byte for byte. The final
review added only cumulative transformer validation and its source assertion;
the final assembly test, Clippy and format were repeated, with the unchanged
component test/binary/manifest receipts retained explicitly. The final exports
are byte-identical to the preceding complete targeted run. The independent
terminal local audit checks 751 source and 173 fixture hashes, all logs and the
actual test binary. Full T06 validation remains deferred.

Initial top-level re-export omissions were repaired before a successful
Bool-construction probe. The first full run exposed an unnecessary source Bool
observer prerequisite in binding-vc-instant; the registered Bool connective
removed that requirement and the full original corpus passed. The cumulative
budget finding was fixed before final-source verification. These earlier
attempts are retained separately and do not count as final verification.

Same-byte Go/Rust checking is running. Exact published-source Linux replay and
terminal auditing are pending. See verification-logs/foundation-proofs/.
'''
(parent/'unit-7-foundation-proofs-review.md').write_text(review)
p=repo/'develop/docs/08_csharp_practical_subset_design-todo.md';s=p.read_text()
anchor='Owns: translation of every already-expanded concrete foundation definition in\n'
assert s.count(anchor)==1
addition='''The integrated original foundation proof adapter now places all 87 concrete-type
and 442 currently supplied operation proofs in one ordinary context across all
45 source contexts. Its ordered supplied list covers 529 original W06 sequents;
458 binding sequents remain without supplied component proofs, and all 987
application proof IDs remain pending. Full original theorem types, operation
proof prefixes, source clauses and public/private domains are preserved. The
actual Rust kernel, strict imports, a typed false-domain mutation, cumulative
limits, Clippy and format pass. All 91 new exports and both 92-file unchanged
component export sets are audited against 751 source and 173 fixture hashes.
Same-byte dual-checker and published-source Linux terminal receipts remain
pending. Generic ownership/currency, native bodies, original refinements and
complete application assembly remain open. W09 stays In progress, W10-W12 stay
Blocked and the full T06 gate is deferred to W12. See
`ordinary-foundation/unit-7-foundation-proofs-review.md`.

'''
p.write_text(s.replace(anchor,addition+anchor))
p=parent/'w09-completion-audit.json';v=json.loads(p.read_bytes())
unit=next(u for u in v['units'] if u['unit']==7)
unit['integrated_original_foundation_checkpoint']=dict(status='passed_local_targeted_checks',evidence='verification-logs/foundation-proofs/verification.json',review='unit-7-foundation-proofs-review.md',source_contexts=45,supplied_type_proofs=87,supplied_operation_proofs=442,supplied_binding_sequents=529,remaining_binding_sequents=458,original_application_proofs_pending=987,scope='A single ordinary context preserves the full checked operation prefix and exact original universal type equations. Generic pending operations and every application proof ID remain open. Dual-checker and published-source Linux terminal receipts remain pending.')
p.write_text(json.dumps(v,indent=2)+'\n')
print(json.dumps(dict(status='preserved',source_files=751,fixture_files=173,supplied_component_proofs=529,remaining_binding_sequents=458,application_proofs_pending=987)))
