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

The identity checkpoint's [current metadata checker](../../../probes/go-practical/check_identity_metadata.py)
checks complete identity preimages/DAG, reserved namespaces, native hash
agreement, candidate migration scope and preserved predecessor evidence. Its
[verification record](identity-dag/verification/verification.json)
keeps hash/identity acceptance separate from full schema/source/profile
completion. The cumulative T01 gate is still reserved for W05.
