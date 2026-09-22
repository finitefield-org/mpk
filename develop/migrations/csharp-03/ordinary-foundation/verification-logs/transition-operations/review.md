# T06-W09 unit 6: transition foundation operations (validation in progress)

The standalone emitter reconstructs each reachable concrete transition from
validated original VIR. It checks the frozen three type arguments, exactly one
bounded-sequence event dependency, field order, complete expanded operation
signatures/equations and ordered make failure. Uninvoked instances are included.
Unknown shapes or signatures fail rather than becoming uninterpreted constants.

Make and the three getters use the existing ordinary product storage operations.
Semantic equality and eligible comparison use the existing structural relation
emitter, in state/events/response order. Event order and duplicates remain intact.
The only make failure is `event_bound`, evaluated at the events argument (index
1), with an unsigned full-u32 length comparison against inclusive 4096. Input
representation/public domains and source invariants are separate obligations.
A false event-bound failure alone does not establish all constructor preconditions.

Generation/import tests replay the structural source corpus and two additional
captured scalar-map/compound transition contexts, checking all reachable
transition instances. Metadata and certificate mutations reject. Semantic tests
use independently encoded monomorphic values and the frozen structural oracle,
with isolated field changes, ordered and duplicate events, and raw boundary
lengths including every high u32 length bit. Product/getter observations retain
all set bits and every selector bit/padding boundary in the sampled values.
The equality/ordering checks apply to the whole sampled values.

All generated terms stay in the existing ordinary certificate format. No
axiom, proof primitive, checker change or host-computed semantic result is
introduced. All three standalone certificates pass both unchanged checkers with zero axioms
and hash corruptions rejected. The original-source generation/import/mutation
and semantic tests pass: three instances and 3,652 observations. The three integrated certificates also pass both checkers. Verified standalone
and integrated fixtures are now published; 43 existing structural certificates
are byte-identical, and one gains the six transition operations (462 -> 468
operation occurrences across 44 contexts).

This component does not implement admission, source purity/totality proofs,
state preservation, time/version behavior, ordered history append, retained-key
lookup, replay, or complete application theorem assembly. The integrated structural generator now appends these foundation operations
after existing profiles. Its operation entries are no longer deferred; exact
standalone/integrated metadata and 291 transitive definitions agree. The prior
term-table prefix and declaration kinds/resolved names are preserved. Canonical
name-table indices shift when a new name is sorted in; the initial test was
corrected to compare names rather than those indices. No production fix was
needed. Seven public/boundary profiles and 21 collection/sequence source outputs
remain byte-identical. Targeted lint, format and inventory checks pass. Unit 6 and W09 remain incomplete; the whole T06 gate stays
with W12. No partial-unit commit is issued.
