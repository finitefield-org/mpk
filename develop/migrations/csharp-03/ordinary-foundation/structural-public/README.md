# Structural foundations with public source conditions

The `ordinary_structural_public.v1` profile places existing structural,
collection, construction-storage, Money and observation definitions together
with compiled source public clauses, recursive public domains and public-default
predicates in one ordinary certificate. Its public conditions remain explicit
predicates for later VC assembly; this profile does not prove native bodies,
source initialization or application obligations.

Source-clause storage and the existing operations share one builder. All
representation-based operations are emitted before switching to the public
count profile with a fresh count cache. Both profiles share the scalar domain
roots reconstructed from the same VIR. Existing operation contracts are retained;
public counts add recursive source-clause filtering. Default predicates reference
the unchanged actual default candidates and declaration predicates.

Three original-source programs cover a contracted constructor, an ordered map,
and direct/array/Nullable source nesting. Exact metadata and all transitive
definition closures match the existing structural and standalone public-default
programs. Importers reconstruct the full profile and reject changed clauses,
domains/defaults and cross-profile substitutions. Source tests passed in 6.69
seconds. Shared-emitter regression tests retain all six pinned public-default
programs and representative representation bytes (9.10 seconds).

All three candidates were hash-verified before pinning. Maximum costs are 9,466
terms, 414 declarations and 2,118 static transformers. All three candidates passed unchanged Rust/Go checking with zero axioms and
hash-corruption rejection (25.047 seconds), with exact PASS sets and current
hashes reconciled. Representative predecessor structural/representation bytes,
scoped lint, formatting and inventory checks passed. Verification and
review are recorded in `../unit-3-structural-public-progress.json` and
`../unit-3-structural-public-review.md`. The original internal units and W09
acceptance remain incomplete; the T-wide gate stays deferred to T06-W12.
