# W09 unit 4: reconstruction observations and projected-result agreement

The previous proof-comparison choice is superseded; see
../unit-4-binding-observation-review.md and its progress record. Proof-level
Binding.Equal and Binding.ObserveEqual preserve admitted observations, including
exact float bits. The C# foundation .equal operation remains IEEE equality.

The generator places the exact forward projections and their transitive ordinary
definitions in one builder with source observations and semantic equality. It
lowers W06 Binding.ObserveEqual, Binding.Equal and Binding.MemberEqual symbols
with concrete source/semantic argument types. Every stored member is included,
including inactive payloads and unmapped fields. Each projected-result agreement
is the ordinary function ObserveEqual(Project(source_result), semantic_result).

These functions are the predicates used to state reconstruction and commutation
obligations. They do not supply reconstruction witnesses, run a native method,
establish source/target domains or prove a sequent. Unresolved W06 definition
names remain an exact independently reconstructed inventory. Only genuinely
provided projection/identity-reconstruction/predicate symbols are removed from
that inventory. A later complete theorem compiler must resolve all required
names and prove all goals; this component cannot produce an accepted application
certificate by omitting those obligations.

The ordinary program binds source IR, foundation and complete binding-VC hashes,
plus exact projection descriptors and predicate signatures. Import regenerates
and compares every metadata and certificate byte. Ordinary word helpers are
shared with the projection emitter rather than redefined. Boolean equality is
available even if a source signature does not otherwise need a Bool carrier.

Tests use the 44 original source/fact contexts from the projection corpus and one new
actual Make source with an operation map.
They compare every projected global's complete declaration/term/level dependency
closure with the standalone projection program and all source-observation roots
with the standalone observation program, validate exact W06 symbol and
stored-member coverage, retain unresolved reconstruction symbols, and reject
changed metadata, signatures/inventories, bytes and source contexts.

Four source contexts independently test 126 ordinary comparisons: float-valued
ordered entries, result wrappers and the remapped three-state wrapper. NaN
proof observations remain reflexive while native IEEE equality is false; signed
zero and distinct NaN payloads remain distinguishable in proof comparisons.
Changing an unmapped Extra field preserves forward result agreement, changes
full source observation, and changes precisely that member's reconstruction
comparison. Source values are validated and semantic equality uses the existing
value model as an independent oracle. These are observations, not proofs.

Actual command results, pending jobs and corpus metrics are maintained in
../unit-4-binding-observation-progress.json. This remains a component of unit 4;
unit 3's outstanding deep checks/review and original units 4-8/W09 acceptance
remain open. check-fast.sh is deferred to T06-W12.
