# Ordinary map/set operations (W09 unit 3, in progress)

The generator reconstructs every reachable ordered-map/set instance, including
uninvoked operations. Map dependencies are resolved by required concrete role
and arguments: the VIR dependency list is a sorted set and cannot be treated
as the template recipe order. The exact 4096-slot shape and complete frozen
operation signatures/equations/error lists are checked. Shared storage definitions,
including lookup types shared by maps with different key types, are emitted once.

Validate uses the existing recursive representation domain, including child
validity, zero padding, logical-cell bounds and strictly increasing keys/elements.
Count observes the full stored u32 length. Lower-bound search uses the shared
ordinary First pipeline and a total key comparison; zero is the no-hit sentinel
before conversion back to an index. Contains compares the selected key after
checking the full index against length. Lookup builds an explicit missing/found
sum and preserves found(none) for nullable stored values.

Add inserts at lower bound, moves later entries one slot, increments length and
zeros the new inactive tail/count padding. Replace changes the selected map
value while retaining its stored key representation and length. Search state is
shared outside the result's address selectors. The position and new length use
the existing C6 state so the update helper has three value binders and a C253
result remains within 256 binders. Every failure predicate is retained in frozen
order: invalid representation, duplicate key/element, then capacity for add;
invalid representation then missing key for replace. Failed-operation storage
bodies are not accepted normal results. Input/output public domains, source
correspondence and application proofs remain obligations.

Nine original-source contexts are being tested: six existing integer/string/
decimal/compound map/set captures and three new captures in
`../collection-sources/`. The additions exercise non-total float values, nullable
lookup and two maps sharing a lookup dependency. Current verification is in
`../unit-3-entry-collection-progress.json`. Nine current generated programs are
pinned; exact replay and both checkers passed with zero axioms, and all nine
hash corruptions rejected (686.922 seconds). The current maximum-update suite
also passed (4,165.23 seconds). Integer map/set domain and capacity verification
passed both 4096-slot source cases (4,341.47 seconds), using the current writer
and corrected thunk lifetime before iterative EnvRef destruction. Complete
collection semantics subsequently passed all 669 observations across ten
instances from nine sources (12,673.09 seconds), with the v4 writer and the
earlier observer before lifetime/destruction corrections. This does not claim
verification of the later observer version.

The ordinary-term tests compare search, membership, lookup, insertion, replacement
and equality/order with the independent value model. Integer domain/capacity
tests cover empty/sorted/duplicate/reversed storage, nonzero padding, high length
bits and 4096 slots. Large output padding is sampled, not proved universally.
The supplemental three sources passed 243 ordinary observations across four
instances. The completed maximum-update test covers exact decimal-cohort key
retention and the final full-capacity update implementation. The current corpus contains
75 operations; maximum certificate sizes are 46,305 terms, 558 declarations
and 8,600 counted static transformers.

This is not a unit 3/W09 completion receipt. Original units 4-8 and application
certificate assembly remain outstanding. The full T06 gate stays deferred to W12.
