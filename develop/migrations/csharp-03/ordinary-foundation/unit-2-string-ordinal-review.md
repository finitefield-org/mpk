# W09 internal unit 2: ordinal UTF-16 comparison and search

Baseline: `ef4056b`. This is a component review, not a W09 completion receipt.

## Implementation

Eight signatures now have ordinary definitions: equality and inequality
operators, static and instance ordinal Equals, ordinal Compare, Contains,
StartsWith and EndsWith. The original closed string signature determines the
input carrier, result type and ordered checks. Source/foundation metadata and
canonical bytes are independently regenerated on import.

The generated First.D0..D14 and Any.D0..D14 helpers have concrete Boolean cube
argument/result types. Each definition references only a preceding, smaller
depth. There is no recursive global, unresolved template, type variable or new
checker rule. All branching uses the existing Bool-result eliminator; word
selection applies that eliminator pointwise. These are predicate/value folds,
not repeated state-transformer composition or a trusted host search routine.

## Direct review

- Splitting binds the most significant index bit while retaining physical
  least-significant-first order for the remaining bits and all result selectors.
  Thus the first differing character in increasing index order wins.
- A count at depth D is bounded by 2^D. The low count is clamped to half
  capacity using its two high bits; the high count clears the split bit, or
  selects half capacity when the whole range is full. High traversal is enabled
  by those same two bits. At exactly half capacity the high count is zero, so
  the enabled empty high fold is harmless. Count zero gives zero/false at the
  leaf. A local let shares the low result before selecting the high result.
  This removes per-node subtraction and arithmetic comparison while retaining
  the complete address space. Public windows/candidate counts establish the
  bounded-count premise; an empty needle bypasses its 16,385 candidate count.
- Char subtraction zero-extends both sixteen-bit code units to 32 bits before
  subtraction. The signed difference therefore covers -65,535 through 65,535,
  preserving NUL, high bits and isolated surrogates. Compare uses length
  difference only when the whole common prefix is equal.
- Static equality treats two null strings as equal and null/empty as different.
  Instance Equals fails on a null receiver. Compare orders null before present
  and returns -1/0/1 for the null cases; no null payload is needed for that result.
- A window compares the right text at index i with the left text at start+i.
  EndsWith uses the length difference as start; StartsWith uses zero. Contains
  folds over exactly receiver length - needle length + 1 candidate starts.
  An empty needle succeeds before that count is consumed, avoiding a 16,385th
  candidate address at maximum receiver length. A longer needle fails normally.
- Search checks null receiver before null argument. Failed Boolean results are
  false and failure flags are mutually exclusive. Input-domain assumptions remain
  explicit: invalid lengths/padding and construction of those domain predicates
  are subsequent W09 work, not silently certified by these definitions.
- Basic, construction and ordinal helpers initialize lazily with separate names.
  Shared cube helpers are defined only when absent. Construction's word emitter
  is reused with ordinal-specific identities. The existing basic/construction
  source certificate bytes are retained.
- Eight original source captures cover every new signature. Unlike the basic
  and construction cohorts, this cohort contains both depth-19 and depth-20
  text carriers. The shared source test now pins each cohort's actual depth set.
  The earlier Contains rejection becomes a positive source case; metadata and
  byte substitutions still reject, and unknown signatures remain fail-closed.
- The new source file is the sole new standard-namespace consumer: 104 becomes
  105, with the aggregate and historical inventory hash link updated together.
  Existing cache paths and inventory add/remove rejection behavior are retained.

## Verification

The actual-core oracle matrix passed 680 cases across both carrier modes and
all eight operations. Forty additional boundary cases cover maximum receivers,
empty needles, final UTF-16 positions, prefix differences after binary carries
and earlier-difference priority. A separate full-capacity test compares equal
16,384-unit strings through the actual core, with no host replacement of results.

Two combined eight-operation certificates have 5,735/5,738 terms and 140
declarations. Eight original-source certificates have at most 5,688 terms and 121
declarations. All ten are pinned for same-byte dual checking, zero-axiom
acceptance and hash-corruption rejection. This verifies the ordinary definitions;
it does not discharge application invariants or W09's complete foundation gate.

The final bit-split implementation passed the matrix, address boundaries and
405 Any-fold cases, including the last-index include/exclude pair. Full-capacity observation and all ten final same-byte checker cases also passed.
The final direct review has zero findings. Other scoped
results are recorded in `unit-2-string-ordinal-verification.json`. The full
`check-fast.sh` gate remains deferred to T06-W12.
