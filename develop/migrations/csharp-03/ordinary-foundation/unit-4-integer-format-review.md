# Integer formatting review — partial W09

Reviewed against the frozen integer/duration/instant codec grammar and existing
ordinary scalar representation. The production division helper uses Boolean
circuits and a bounded five-bit remainder transition, not a host division
answer. Helper signatures use their actual ordinary cube types; no native
operation or named application method is inferred from them.

- Original-width two's-complement magnitude followed by unsigned extension
  retains signed minima. Extending a signed minimum and negating in the wrong
  signed width would give incorrect high digits; minimum/maximum cases are
  explicit in the independent source-context oracle matrix.
- Twenty stages cover u64 decimal output. The remainder transition uses the
  high-to-low input bit order and the quotient carry from subtraction by ten.
  The last nonzero quotient sets the digit count; zero still has one digit.
- Shared arithmetic is bound outside text selectors. Final digit selection
  reverses remainder order, includes the sign only at index zero, and guards
  inactive output with the complete index/length comparison. Header padding is
  zero and output length stays below the text bound.
- The importer reconstructs all applicable codecs from validated VIR scalar
  carriers and fixed codec/type configuration; callers cannot substitute a
  formatter, metadata, source context or certificate bytes.
- The first Rust check found a temporary borrowed helper name; a stable local
  name now survives the call. The first source suite generated the existing
  corpus but found only nine codec IDs because duration was absent. Added the
  previously captured real `TimeSpan` addition source, preserving the required
  ten-codec coverage rather than accepting the incomplete corpus.
- Large output storage is sampled; exact active text and header padding are
  observed. Parsing that actual text with the earlier independent model checks
  loss/canonical spelling but does not replace the separately implemented ordinary
  integer parser or either pending universal round-trip theorem.

Actual terminal results and specific live handles are recorded in
`unit-4-integer-format-progress.json`. The source semantic matrix completed with
392 passing observations in 4407.63 seconds. Final scoped definition review
found no additional actionable issue. Both unchanged checkers have now passed all
54 identical pinned byte sequences with zero axioms and rejected their hash
corruptions; pinned replay, latest lint, inventory and format also passed.
Integer parser verification is recorded separately. The unchanged W09 plan
keeps registered codec linking, universal round-trip proofs, other codecs,
source/native, transition and proof-assembly obligations. No component-only
commit, internal-unit or W09 completion is claimed.
