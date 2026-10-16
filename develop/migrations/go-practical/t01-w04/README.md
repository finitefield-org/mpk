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

1. Freeze typed physical-region counts, storage bounds and their entry/allocation
   enforcement; assign the ordinary zero-access normalization/frame proofs.
2. Freeze capture/module/source/count budgets and the real snapshot/compiler
   enforcement, separately from measured proof budgets.
3. Specify loop-contract defaults, variant obligations and pending-proof states
   so inferred invariants or missing strategies cannot become assumptions.
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

Selected checks cover the new private native observer and evidence validator:
49 original-Go cases reproduce across two executions and a fresh-build replay;
35 evidence/adversarial checks pass. Strict JSON, exact source/toolchain/artifact
identity, existing-anchor hashes, unique rule/vector IDs, planned executor/test
ownership, local links, formatting and whitespace are also checked.
The [metadata checker](../../../probes/go-practical/check_scope_metadata.py)
validates this draft boundary; the
[verification record](scope-observations/verification/verification.json)
retains the selected commands, outputs and deferred gate.
Unchanged production/kernel suites need no repeat for these private draft
changes. W04 remains open until every listed freeze deliverable is exact and
reviewed. W05 owns inactive specification/vector publication,
source-model feasibility review and the complete `./scripts/check-fast.sh`
T01 gate. No workflow or GitHub Actions is introduced or used.
