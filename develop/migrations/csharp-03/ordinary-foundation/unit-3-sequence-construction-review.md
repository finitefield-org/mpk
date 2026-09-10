# W09 unit 3 sequence/construction review checkpoint

This is a direct review of the two new operation generators, their exports,
source captures, tests and pinned artifacts. It is not a review/completion
receipt for all uncommitted domain/default work, internal unit 3 or W09.
Current results and artifact/log hashes are in
`unit-3-sequence-construction-progress.json`.

The review checked reconstruction of every expanded concrete instance,
source/foundation identity, exact frozen operation signatures/equations/error
precedence, read/update address groups, full-word index and length checks,
normal-body versus gate separation, optional total comparison, initialization
prefixes, publication projection, importer equality and resource limits.
Neither generator grants authority to ownership/default flags or claims an
application proof. Constructor ownership is an explicit unresolved source-state
obligation; its absence from the storage carrier is intentional. Input/public
domains and source-state proof integration remain mandatory later work.

Resolved test/review findings:

- The construction corpus initially deduplicated solely by element type ID.
  A readonly product in two distinct source contexts can have the same
  declaration ID. The test now keys by original VIR hash and type ID and
  executes both contexts; all five pass.
- A high-index raw-write mutation was not observable for a Boolean slot already
  holding true when the replacement was also true. Replacements now complement
  the original bits, making an aliased write observable in every tested shape.
  The strengthened five-context test passes.
- The first sequence value-pair corpus used different lengths or identical
  values. Supplemental scalar/product tests use equal-length values differing
  at a late element and at the first element with an opposing later difference,
  in both directions. They pass through the generated shared-fold operations.
- The initial construction test had an unparenthesized Rust cast before `<`;
  compilation rejected it and parentheses fixed the test expression. Test-only
  cloned slices and export ordering were corrected for lint/format checks.

The construction full-bitmap test passes in 443.44 seconds without restart:
16,384 initialized cells, missing last cell, odd 16,383 prefix, rejection of
length 16,385 with a full physical bitmap, and the final 4096 publication slot.
The shared fold's clamp alone cannot satisfy the invalid-length assertion.

Both pinned source replays, source/gate/hash mutations, high-index sequence
tests, five-context construction semantics, equal-length sequence order,
targeted clippy, scoped formatting and all five inventory checks pass. Large
output padding is sampled as specified in the test code and README; these
observations are not universal proofs.

Both unchanged checkers accept all 17 pinned byte strings with zero axioms and
reject all 17 hash-corrupted variants. That run finished successfully in 773.304
seconds; source generation and semantic observation results remain distinct.

No additional actionable implementation finding was identified in this scope.
The original-source sequence semantic pass completed in 1868.22 seconds with
457 indexed reads, small semantic pairs, the full 4096 capacity and IEEE NaN.
Every emitted certificate and its full metadata match the pinned dual-checked
corpus exactly. Supplemental order/high-bit tests are separate evidence.

The previously started recursive-domain boundary tests remain pending. Their
live process must be continued, not replaced because
of elapsed observation time. Until the required evidence is complete, no full
unit/W09 zero-finding completion receipt or commit is issued. The bounded-sequence
operations and construction storage bodies/predicates have passed their scoped
tests and review; source
state/domain integration and application proof assembly are still outstanding. The
original maps/sets/outcomes and units 4-8 scope is unchanged; the full T06 gate
remains deferred to W12.
