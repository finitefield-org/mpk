from pathlib import Path
from datetime import datetime,timezone
import collections,hashlib,json,shutil
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs');root=Path('/private/tmp/mpk-w09-identity-proofs');parent=repo/'develop/migrations/csharp-03/ordinary-foundation';base=parent/'verification-logs/identity-proofs';assert not base.exists()
base.mkdir();(base/'tool-sources').mkdir()
for name in ('source-manifest.json',):shutil.copyfile(root/name,base/name)
shutil.copyfile(root/'local/expected-certificates.json',base/'expected-certificates.json')
for folder in ('local','checks'):
 shutil.copytree(root/folder,base/folder)
shutil.copytree(root/'probe',base/'probe')
p=base/'probe/test.log';p.rename(p.with_name('test.log.txt'))
for name in ('run-targeted.py','audit-local.py','check.py','audit-dual.py','launch-linux.py','preserve.py'):shutil.copyfile(root/name,base/'tool-sources'/name)
metadata=[(p.stem,json.loads(p.read_bytes())) for p in (base/'local/certificates').glob('*.json')]
counts=collections.Counter();contexts={}
for id,m in metadata:
 ids=m['remaining_binding_sequent_ids'];counts.update(x.split('.')[1] for x in ids);contexts[id]=dict(remaining_sequent_ids=ids,count=len(ids))
assert sum(counts.values())==456
(base/'remaining-binding-vcs.json').write_text(json.dumps(dict(schema='mpk.csharp.remaining_binding_vcs.v1',source_contexts=45,supplied_binding_sequents=531,remaining_binding_sequents=456,original_application_proofs_pending=987,counts_by_kind=dict(sorted(counts.items())),contexts=contexts,application_scope_pending=True),indent=2,sort_keys=True)+'\n')
verification=dict(status='passed_scoped_local_and_same_byte_dual_checker_verification',recorded_at=datetime.now(timezone.utc).isoformat(),source_contexts=45,source_files_verified=753,fixture_files_verified=263,identity_proofs=2,supplied_binding_sequents=531,remaining_binding_sequents=456,original_application_proofs_pending=987,generic_operations_pending=25,original_foundation_programs_unchanged=45,unchanged_foundation_certificates=44,fresh_checker_stages=6,local_receipt='local/status.json',local_audit='local/final-audit.json',dual_checker_receipt='checks/verification.json',dual_checker_audit='checks/final-audit.json',remaining_vcs='remaining-binding-vcs.json',linux_verification='pending exact public-source replay',application_scope_pending=True,w09_status='In progress',w10_w12_status='Blocked',full_t_gate='deferred to T06-W12',selection_reason=json.loads((root/'source-manifest.json').read_bytes())['selection_reason'])
(base/'verification.json').write_text(json.dumps(verification,indent=2,sort_keys=True)+'\n')
review='''# Original identity projection proofs

The partial T06-W09 assembly now supplies the complete original identity projection
sequent family: Bool and f32 in the float-make-commutation context. It retains the
universal source receiver, exact PublicDomain premise and both original project
and reconstruct equality goals. Binding.Equal is proof-level agreement over the
complete value carrier; the C# float .equal operation retains its separate NaN
semantics. No source bits, assumptions, kernel rules or axioms are added.

The new opt-in API resumes the complete checked foundation proof context, emits
its original source-bound projection definitions and appends ordinary Std.Eq and
Std.Logic.And proof terms. Both original operands remain in each theorem type;
conversion is checked by the unchanged kernels. Every original declaration and
term stays at its existing index. The earlier foundation API, metadata and all
45 certificates remain byte-identical. The new API returns the same certificate
in the 44 contexts without identity sequents. A cumulative transformer budget
includes the original foundation and all appended projection work.

The ordered supplied partition now contains 531 original W06 sequents and 456
remaining binding sequents. All 987 original application proof IDs and 25 generic
construction/currency operations stay pending. Source use, reconstruction,
invariants, native bodies and complete application assembly remain open. W09
stays In progress; W10-W12 stay Blocked; the full T06 gate is deferred to W12.

## Verification and review

The targeted source test covers all 45 original contexts. It independently
compares the complete identity family and ordered partitions against the original
full VC, compares all old foundation bytes/metadata to published fixtures, checks
exact theorem receiver/premise/conjunction/equality operands, and verifies strict
metadata/certificate imports. A well-typed constant-false Bool projection passes
before the identity proofs and fails CoreCheck with them. This proves that the
new theorem requires the complete original projection; a definition alone does
not establish identity.

The actual Rust kernel passes the new context with zero axioms and empty proof-
node/theory-certificate tables. Crate Clippy and crate/changed-support formatting
pass. Both unchanged Go/Rust checkers pass six fresh stages: two positive checks,
two hash rejection checks and two typed false-projection CoreCheck rejections.
Their positive hashes/counts agree. The 44 exact unchanged certificates reuse
the previously audited same-byte dual-checker reports; no unaffected standalone
exporter tests or full T06 gate were repeated. The independent local audit checks
753 source files, 263 fixture dependencies, all four logs, the actual test binary
and all 47 exports. The extra 90 fixtures are the old foundation bytes/metadata
consumed for preservation assertions; original source fixtures are unchanged.

An initial test-only missing hex loader caused compilation failure before any
test ran; the repaired complete run is final. The failed compile log remains in
probe/. Published-source Linux verification remains pending. See
verification-logs/identity-proofs/verification.json.
'''
(parent/'unit-7-identity-proofs-review.md').write_text(review)
p=repo/'develop/docs/08_csharp_practical_subset_design-todo.md';s=p.read_text();anchor='Owns: translation of every already-expanded concrete foundation definition in\n';assert s.count(anchor)==1
addition='''The original identity-projection proof family is now integrated on the unchanged
foundation context: Bool and f32 retain their original source receiver, domain
premise and both project/reconstruct equality goals. Across the same 45 source
contexts, 531 component sequent proofs are supplied and 456 binding sequents
remain. All original 529 foundation components and their API exports stay
unchanged; 44 contexts reuse their exact certificate bytes. The new context
passes both unchanged checkers with zero axioms, hash corruption rejection and
typed false-projection CoreCheck rejection. The source test, Clippy, format and
independent local/dual audits pass. Published-source Linux verification remains
pending. All 987 original application proof IDs, 25 generic operations, native
bodies and complete application assembly remain pending; W09 stays In progress
and the full T06 gate is deferred to W12. See
`ordinary-foundation/unit-7-identity-proofs-review.md`.

'''
p.write_text(s.replace(anchor,addition+anchor))
p=parent/'w09-completion-audit.json';d=json.loads(p.read_bytes());unit=next(u for u in d['units'] if u['unit']==7)
unit['original_identity_projection_checkpoint']=dict(status='passed_scoped_local_and_same_byte_dual_checker_verification',evidence='verification-logs/identity-proofs/verification.json',review='unit-7-identity-proofs-review.md',source_contexts=45,identity_proofs=2,supplied_binding_sequents=531,remaining_binding_sequents=456,original_application_proofs_pending=987,scope='All original identity projection sequents retain receiver/domain/complete equality operands on the unchanged foundation proof prefix; 44 unchanged contexts reuse exact prior checker evidence. Exact published-source Linux verification remains pending. This does not complete W09.')
p.write_text(json.dumps(d,indent=2)+'\n');print(json.dumps(verification))
