# GO-PRAC T01-W04 freeze preparation

Status: In progress, not frozen or published. Entry is W03 completion commit
`7e5f94bda47968aeec938098d5e2a324370a6218`. Its requirement-by-requirement
[completion audit](../t01-w03/completion-audit.json) pins the measured foundation.
All v1 priorities1–4 remain required. Maps/pointers and the second release
remain T08–T11 work; the complete eleven-stage objective is unchanged.

## Current draft artifacts

- [Freeze worklist](freeze-worklist.json): 24 requirement groups with intended
  meaning, negative boundaries, a primary future production executor and test
  owner. These rows are not complete conformance vectors or exact wire schemas.
- [Surface inventory](surface-inventory.json): 17 implementation components,
  their current code anchors and 143 discovered implementation/test files.
  Existing files are byte-pinned. Planned Go-specific files are explicitly
  unimplemented. Discovery still needs indirect-consumer tracing and a reviewed
  migration/retention disposition for every file.
- [Candidate limits](candidate-limits.json): W03-derived semantic and proof
  budget inputs. They change no runtime limit. Structural/capture counters,
  source-domain rules and exact enforcement remain to be frozen.
- [Root/error/empty-slice decisions](type-error-zero-decisions.md): 26 draft
  semantic rules distinguishing root values from backing cells, finite error
  dispatch from boxing, and zero-access descriptors from positive-capacity
  empty views. Production normalization and source/operation binding proofs
  remain due.
- [Source vectors](source-vectors.json) and
  [native observations](scope-observations/observations.json): 49 intact Go
  snippets, exact selected type/call facts and runtime observations. Go1.25.0
  accepts 43 snippets, reports five original type errors and one parse error.
  The future profile's 22 accepted-model, two pending, one configuration
  rejection and 24 source-rejection expectations are draft outcomes, not
  observations from an implemented practical frontend.
- [Operation table](operation-table.json) and
  [evaluation/contract/loop decisions](operation-loop-decisions.md):
  56 logical operation families and 25 draft rules with planned producer,
  independent validator, VC and test ownership. Operation IDs are design
  references; exact wire tags/schemas remain pending.
- [Evaluation vectors](evaluation-vectors.json) and
  [native receipt](evaluation-observations/observations.json): 54 original-Go
  cases over 35 forms, including two-phase lvalues, single-call results,
  switch targets, array/slice range, byte/string copy/append and integer edges.
  Native observations do not establish loop termination or profile admission.
- [Private order/strong-induction evidence](nat-termination/README.md):
  16 ordinary claims, 20 isolated Bool type-selection interface cases and
  three retained regression certificates checked by both private verifiers.
  General Sort0 strong induction is measured; complete source-variant
  termination and the additive core-interface freeze remain open.
- [Private variant-termination evidence](variant-termination/README.md):
  ten ordinary candidates and three retained regression certificates. The
  arbitrary-Nat rank+1 stop theorem requires preservation/decrease only on
  continuing backedges; an exiting rank-zero instance is checked. Complete
  Go source totality and exact foundation/overlay identities remain open.
- [Closed typed overlay draft](overlay-contracts/README.md): 70 source and 70
  normalized expression heads, 24 source-overlay vectors and intact native
  lexical/type observations. Old/current/result/finite-sum and error entry
  semantics are explicit. The rank theorem uses the LC02 pre-condition header;
  production schema/hash/importer/definedness/proof review remains open.
- [Canonical storage-owner/typed-view draft](storage-views/README.md):
  15 native/model observations and ten ordinary coupling candidates in the
  unchanged production checkers. Generic Sort0 cells retain parent/child
  updates, sibling frames and distinct owners. The slot candidate counts
  positive root owners; complete path/exposure/lifetime/live-counter/source
  proofs and exact reference schemas remain required.
- [Derived lifetime/slot-reuse draft](storage-lifetimes/README.md): 12 native/model
  observations, 24 targeted tests and five ordinary candidates in unchanged
  production checkers (two accepted, three type mismatches). Roots include
  active entry values, pending operands/results/targets and error payloads;
  old/post stores remain independent across reuse. Source root completeness,
  safe retirement/translation and concrete bounded-state proofs remain due.
- [Closed source/selected-call draft](source-admission/README.md): all 56 pinned
  AST node kinds and 18 predeclared builtins have explicit dispositions and
  planned owners. Eighty-one original packages retain 31 unproved candidates,
  46 rejections and four compiler failures. Unselected/type-only bodies stay
  separate from transitive selected calls. Complete typed/effect/capture/
  adapter predicates and independent source/proof validation remain required.
- [Numeric/carrier accounting draft](structural-limits/README.md):64 candidate
  counters,192 isolated boundaries,85 original native layout sources and full
  current/past-state charges. An actual unused-HTTP load has1,417,549 syntax
  nodes, requiring a practical-only4,194,304 candidate while preserving the
  predecessor's1,000,000. Exact production preflight, source/finite-ID/allocator
  proofs and successor identities remain open.
- [Identity/hash DAG draft](identity-dag/README.md):101 identity candidates/41
  domains avoid the frozen inactive C# namespace. Ten closed source-identity
  schemas yield a26-node/seven-root graph,44 three-observer JSON/hash vectors
  and143 candidate migration routes. Full shared/Go roots/unions, independent
  source/registry validity and complete reviewed migrations remain required.

## Shared-family decision

The installed semantic registry is schema v1 revision3, with Go fixed, Rust
checked, C# scalar and Java scalar. Their accepted source semantics and proof
results must be preserved. Its section11 requires a new registry and affected
artifact schema when the common operation vocabulary changes; adding another
v1 revision alone is insufficient for the practical result/effect model.

The inactive C# practical v2 family is already frozen. Its context foundation
reference is specifically bound to `mpk.csharp.practical.foundation.v1`; it is
not an open Go extension point. W04 therefore prepares a distinct Go-owned
shared v3 family, using later identities where C# has already reserved a v3
name. The exact root/nested/union schemas and hash preimages are still pending.
No frozen v1/v2 parser or specification is widened in place.

T02 owns the private migration implementation and all affected active producers
and consumers. T07 owns atomic installed cutover after local release gates.
There will be one installed shared-artifact family. The inactive C# profile
is not activated as a dependency; its later resumption must reconcile the Go
shared changes as already required by the schedule amendment.

## Selected semantic decisions and remaining freeze

The full design's source scope is retained. The
[selected rules](type-error-zero-decisions.md) now specify:

1. Root value aggregates can carry slice descriptors; backing cells cannot
   contain nested mutable references. All root paths preserve sharing.
2. Boxing requires the Error signature without calling its body. A closed
   Error invocation includes every finite target body and its safety/effect
   obligations. Concrete zero and NilInterface remain distinct.
3. A capacity-zero non-nil descriptor uses a valid NoAccess representative.
   Positive-capacity empty descriptors preserve full tails and aliases.
   Canonicalization grants no separation or write permission.

The complete source and operation tables still need review. Other open work is:

1. Review canonical owner/view, exposure and scope accounting; freeze storage bounds and their entry/allocation
   enforcement; assign the ordinary zero-access normalization/frame proofs.
2. Freeze capture/module/source/count budgets and the real snapshot/compiler
   enforcement, separately from measured proof budgets.
3. Review the closed typed overlay draft and finish the ordinary foundation.
   Root termination is now explicitly partial/total, missing invariants are
   untrusted true candidates, and missing total variants stay pending.
   General Sort0 strong induction now has private both-checker evidence with a
   proposed Bool.recType interface. Exact joint core/foundation identities,
   remaining variant/source members and budgets remain due. An ordinary
   rank+1 stop schema now has private both-checker evidence, including exits
   that require no rank decrease; complete Go source totality remains due.
4. Close every successor identity, schema, hash domain/preimage and vector,
   including old/new crossing and preservation of all four installed profiles.

These decisions are W04 work, not reasons to mark the freeze complete or to
reduce the design to an easier subset. Complete application/source binding
remains required in T02–T06. The initial budget candidates retain byte256 and
slice-storage128, fixed result arity up to16, full64 target words, and the
measured two-slot typed heap. Region/type counter semantics still need review.
The proposed 2MiB/131,072-term/4,096-declaration and 60s/1GiB child budgets
include headroom over the observed recipes; they do not guarantee every source
program fits. Final application gates must meet the frozen values.

## Verification and next boundary

The earlier root/error/zero checkpoint reproduced 49 original-Go cases across
two executions and a fresh-build replay; 35 evidence/adversarial checks passed.
Strict JSON, exact source/toolchain/artifact
identity, existing-anchor hashes, unique rule/vector IDs, planned executor/test
ownership, local links, formatting and whitespace are also checked.
Its [metadata checker](../../../probes/go-practical/check_scope_metadata.py)
records that original section30 boundary; the
[verification record](scope-observations/verification/verification.json)
retains the selected commands, outputs and deferred gate.
Unchanged production/kernel suites need no repeat for these private draft
changes. W04 remains open until every listed freeze deliverable is exact and
reviewed. W05 owns inactive specification/vector publication,
source-model feasibility review and the complete `./scripts/check-fast.sh`
T01 gate. No workflow or GitHub Actions is introduced or used.

The operation/evaluation checkpoint additionally passes 54 cases over35 forms
and 52 evidence checks. Its
[metadata checker](../../../probes/go-practical/check_operation_metadata.py)
records the section31 draft boundary; the earlier scope metadata log retains
its original section30 checkpoint.
The [new verification record](evaluation-observations/verification/verification.json)
separates native facts, metadata consistency and deferred ordinary proofs/gates.
Neither operation catalogue generation nor native replay is a source-to-certificate
or universal termination proof.

The foundation checkpoint's
[section32 metadata checker](../../../probes/go-practical/check_termination_metadata.py)
validates its proof receipt, retained failed constructions, planned core
ownership and unchanged predecessor inputs. It does not close W04 or establish
termination of a complete Go application.

The variant checkpoint's [section33 metadata checker](../../../probes/go-practical/check_variant_metadata.py)
checks the ordinary stop schema, preceding foundation evidence, diagnostics,
unchanged checker/production inputs and current draft scope. Its
[verification record](variant-termination/verification/verification.json)
retains selected replay/evidence checks; the whole T01 gate remains due in W05.

The overlay checkpoint's [section34 metadata checker](../../../probes/go-practical/check_overlay_metadata.py)
validates closed operator coverage, native grammar/scope receipts, prior proof
integrity and unchanged predecessor inputs. Its
[verification record](overlay-contracts/verification/verification.json)
keeps candidate type acceptance distinct from source/VC/proof acceptance.

The storage checkpoint's [section35 metadata checker](../../../probes/go-practical/check_storage_view_metadata.py)
checks current native/model/ordinary evidence, retained receipts and unchanged
predecessor files. Its [verification record](storage-views/verification/verification.json)
keeps pointwise coupling proofs distinct from complete source normalization.

The lifetime checkpoint's [section36 metadata checker](../../../probes/go-practical/check_lifetime_metadata.py)
checks derived-root/reuse evidence, retained owner/overlay/foundation receipts,
planned executor scope and unchanged predecessor/diagnostic files. Its
[verification record](storage-lifetimes/verification/verification.json)
keeps model reachability and ordinary masked-store facts distinct from actual
source/allocator correspondence. The cumulative T01 gate stays due in W05.

The source checkpoint's [section37 metadata checker](../../../probes/go-practical/check_admission_metadata.py)
checks closed AST/builtin ownership, all original/native/classifier identities,
retained proof receipts and unchanged predecessor inputs. Its
[verification record](source-admission/verification/verification.json)
keeps compiler facts and candidate classifications distinct from production
admission and source/model proof acceptance. W04 remains open.

The numeric checkpoint's [section38 metadata checker](../../../probes/go-practical/check_structural_metadata.py)
checks explicit units/result classes, original native/layout/HTTP counts,
planned enforcement owners and unchanged predecessor evidence. Its
[verification record](structural-limits/verification/verification.json)
keeps isolated counter and model-state measurements separate from complete
source/allocator enforcement. The full T01 gate remains deferred to W05.

The identity checkpoint's [section39 metadata checker](../../../probes/go-practical/check_identity_metadata.py)
checks complete identity preimages/DAG, reserved namespaces, native hash
agreement, candidate migration scope and preserved predecessor evidence. Its
[verification record](identity-dag/verification/verification.json)
keeps hash/identity acceptance separate from full schema/source/profile
completion. The cumulative T01 gate is still reserved for W05.

The [context protocol draft](context-protocol/README.md) closes six private
registry/entry/context/request/Go-parameter/selection roots. Existing four
profiles use actual unchanged compiled Rust parameter/selection validators,
including both Rust32/64 targets. The152 vectors and31 actual native queries
separate configuration acceptance from source, complete foundation and proof
acceptance. The [section40 metadata checker](../../../probes/go-practical/check_context_metadata.py)
checks retained identities/proofs, sources, diagnostics and draft scope.
Its [verification record](context-protocol/verification/verification.json)
records20 targeted tests and fresh native policy replay. W04 stays open;
full roots/unions, foundation/limits/source predicates and migrations remain
required. The cumulative T01 gate remains assigned to W05.

The [foundation/limits packaging draft](foundation-limits/README.md) closes
three roots with64 numeric/unit/outcome records,56 logical operations and the
exact two-rule joint core proposal. Its recipe inventory is explicitly not
a complete ordinary kernel/source foundation. Exact raw member bytes and
acyclic descriptor/registry/context linkage have102 configuration vectors,
21 three-observer hash preimages and20 targeted tests. The
[section41 metadata checker](../../../probes/go-practical/check_foundation_limits_metadata.py)
checks all earlier evidence,143 predecessor pins and both diagnostics.
The [verification record](foundation-limits/verification/verification.json)
retains selected checks and reasons. W04 stays open with full ordinary
specialization/expansion/binding, source/contract/downstream schemas and
reviewed migrations remaining. The cumulative T01 gate stays due in W05.

The [closed carrier/ordinary expansion draft](closed-carriers/README.md)
derives42/43 instances from39 source/fixture types and emits complete ordinary
carriers/zeros with46/47 new checked theorems. Both unchanged private checkers
accept the two full sets and reject six controls. Two legacy certificates
also reproduce. The [section42 metadata checker](../../../probes/go-practical/check_carrier_metadata.py)
checks exact schemas/closure/certificates, all retained evidence and source
pins. Its [verification record](closed-carriers/verification/verification.json)
records20 selected tests and full local replay. Source/validity/finite-id
and operation bindings remain unproved; W04 and the complete objective stay
open. The cumulative T01 gate remains assigned to W05.

The [closed contract wire draft](contract-wire/README.md) closes source-overlay,
source-scope and normalized-contract roots plus all70 expression arms. Exact
nominal types, lexical binding/loop IDs, de Bruijn depths and full snapshot
namespaces have20 selected tests. The24 source cases retain5 unproved,
1 pending-variant and18 rejected outcomes. Ninety-seven Python/Rust/Go hash
preimages agree; component hashes are private tests and no source/definedness
proof verdict is claimed. The [section43 metadata checker](../../../probes/go-practical/check_contract_wire_metadata.py)
checks all retained evidence, source pins,143 predecessor paths and both
diagnostics. Its [verification record](contract-wire/verification/verification.json)
records selected checks and reasons. Independent production import/proofs,
shared/downstream roots/unions and reviewed migration remain due. W04 stays
open, with the cumulative T01 gate due in W05 and all T02–T11 still active.

The [concrete operation-signature draft](operation-signatures/README.md) closes
parameters, ordered input/output and control-region interfaces, rule members
and binding requirements for all 56 families. Its 224 vectors retain 56 typed
unproved candidates and 168 altered-record rejections. Twenty-four selected
tests and 56 Python/Rust/Go preimages cover the component boundary. The
[section44 metadata checker](../../../probes/go-practical/check_operation_signature_metadata.py)
also reconstructs the preceding contract corpus and preserves source pins,
143 predecessor paths and document links. The
[verification record](operation-signatures/verification/verification.json)
distinguishes typed signatures from missing executable VIR, source/SSA/call
provenance and ordinary operation proofs. Shared/downstream roots, migrations,
W05/T01 and the complete T02–T11 objective remain required.

The [typed function-body/structured-region draft](vir-body/README.md) adds
canonical SSA dominance/capture/state flow, exact static call-product/post
provenance, normal-only continuations and deferred location access sites.
Fifty-six component queries,three function candidates and20 graph cases have
21 targeted tests. Sixty-nine Python/Rust/Go hash inputs include actual256-
container boundaries using new untrusted observers. The
[section45 metadata checker](../../../probes/go-practical/check_vir_body_metadata.py)
retains the preceding operation corpus,143 predecessor pins and document
linkage. Its [verification record](vir-body/verification/verification.json)
distinguishes structural checking from absent complete shared VIR,source,
dynamic invocation and ordinary semantics. W04 and the full objective remain
open; the cumulative T01 gate remains assigned to W05.

The [shared VIR/contract module draft](shared-vir/README.md) closes the exact
five-profile envelopes and Go whole-capture scope/contract/body sets. Multiple
actual overlays and unannotated default-contract callees share one immutable
capture. Seven module candidates,73 vectors and22 targeted tests cover dispatch,
call closure/cycles, raw-buffer authority and cumulative module counters.
Sixty-two Python/Rust/Go preimages agree. The actual unchanged retained
importer/type validator/WP engine preserves full typed projection and serialized
VC data in five target contexts, with thirteen native positive/negative queries.
The [section46 metadata checker](../../../probes/go-practical/check_shared_vir_metadata.py)
retains preceding body evidence,143 predecessor pins and document linkage.
Its [verification record](shared-vir/verification/verification.json) distinguishes
fixture preservation and Go model typing from source/ordinary proof acceptance.
Source-map/manifest/VC/policy/API/release roots and reviewed migrations remain
due. W04, W05/T01 and all T02–T11 remain open; the full gate stays assigned to W05.

The [source-map/frontend-manifest draft](capture-artifacts/README.md) adds exact
common envelopes and Go input-inventory records with complete original byte/
capture/build/module/package/selection linkage. Eight artifact triples and187
cases retain16 linked-unproved outcomes and171 rejections. Nineteen full
Python/Rust/Go preimages agree;24 targeted tests cover nested region locators,
immutable source authority, rehashed inputs and limits. The actual unchanged
public retained validators preserve full map/manifest values in five target
contexts, including their exact independent synthetic-permission boundaries.
The [section47 metadata checker](../../../probes/go-practical/check_capture_artifact_metadata.py)
preserves prior module evidence and143 predecessor pins. Its
[verification record](capture-artifacts/verification/verification.json) keeps
the initial observer setup failure and selected final checks. The external
release-reference fixture is incomplete/uninstalled, with registered practical
producer/toolchain provenance still due. Certificate-manifest/VC, frontend
envelope, policy/API/release roots, reviewed migration and all source/ordinary
proofs remain required. W04 stays open; W05 owns the cumulative T01 gate and
all T02–T11 work remains active.

The [VC/certificate-source draft](proof-artifacts/README.md) adds common source-
bound VC/certificate references and retained skeleton regeneration through the
actual unchanged public APIs. Flat term arrays preserve768-depth handling.
Eight bundles,181 cases and25 targeted tests retain exact old VC/skeleton/
certificate-source values in five target contexts. Nineteen Go theorem-demand
families yield169 source-bound requests,with56 local fragments retaining416
member/access demands. All260 Python/Rust/Go preimages agree. Go ordinary
proposition expansion and final program-assembly/skeleton fields remain due;
Go skeleton requests stay pending. The
[current metadata checker](../../../probes/go-practical/check_proof_artifact_metadata.py)
retains prior capture/module evidence and143 predecessor pins. Its
[verification record](proof-artifacts/verification/verification.json) keeps
selected checks and the initial unsupported1024-depth observer setup failure.
Complete Go ordinary expansion,frontend-envelope/policy/API/release roots,
reviewed migrations and all source/ordinary proofs remain required. W04 stays
open,W05 owns the cumulative T01 gate and the full T02–T11 objective remains.
