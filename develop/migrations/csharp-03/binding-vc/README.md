# T06-W06: source binding and concrete foundation VC handoff

`BindingVcProgram` derives pending obligations from independently validated
source bindings, construction invariants and the reachable closed foundation.
The existing VC wire commits its domain-separated hash and complete sequent
list. Its standalone importer reconstructs the full program from validated
inputs and requires exact canonical bytes; sidecar classification, serialized
proof flags and caller-supplied expansions cannot discharge any obligation.

Each application representation retains exact source/member provenance,
projection/reconstruction operations, invariant, role, bounds, tag and payload
mapping. Goals cover total projection, source/semantic round trips, every
observable stored field (including inactive payloads), tag partition and
pairwise distinctness, member agreement, actual CLR default, identity
observability, canonical collection order and nonempty validation errors.
An ineligible default produces a default-use restriction; it does not assert
that the actual default violates the source invariant.

Commutation retains actual source/native function bodies and closed signatures.
Successful returned results project the source Result and extract its success
payload before comparison. Returned error carriers, exceptions and unmatched
checks retain source precedence and semantic precedence separately. Reverse
outcome obligations prevent a source-success-only implication from standing in
for equivalence. Enum rounding operands and primitive identity projections have
explicit typed recipes. Contextual sequent IDs include binding, source method
and semantic operation, allowing one method to implement multiple operations.

The foundation descriptor, roots hash, closed-set hash, dependency/provenance
IDs and expansion counters bind the exact rederived table. Concrete entries
contain no template IDs or parameter nodes. Every reachable concrete type and
operation, including uninvoked operations, gets definition-equivalence goals
and operation outcome coverage. The native expanded table and W06 table are
checked for equality. The existing importer enforces descriptor registration,
closure, order, deduplication, source linkage, generic exclusion and fixed
specialization/projection limits. W06 additionally reserves its generated nodes,
sequents, binders, bytes and shared definition names within VC resource limits.
The native projection-plan limit and generated sequent count are distinct units.

`requests.json` and `responses.json` contain 12 fresh original-source Roslyn
captures, one per supported binding role. Ordered map also needs an ordered
entry representation. `goldens.json` pins their program hashes and counts.
Tests replay two additional existing instant/money captures with real operation
mappings, returned errors and rounding. Typed-term/signature checks and finite
formula countermodels independently break projection, reconstruction and
commutation across all 12 roles; these are equation tests, not CLR equivalence
proofs. Handoff mutations remove members/goals/instances or alter provenance,
counters, descriptors and concrete types. Existing foundation/import tests cover
closed identity, ordering, closure, bounds and residual generics.

Current construction golden whole-VC hashes include W06; earlier handoff
hashes, receipt contents and the frozen producer/vector package are unchanged.
These are pending VCs and exact expansion recipes, not semantic/kernel proof
receipts. W09 owns ordinary expansion, proof construction and both checker runs.
The full `./scripts/check-fast.sh` gate is deferred to the final T06-W12.
