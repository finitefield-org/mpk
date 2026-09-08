# W09 internal unit 2: decimal arithmetic review

Baseline: `06996db`. This is a partial W09 review, not a W09 completion receipt.
The twelve new signatures complete the 45 non-literal decimal operations;
UTF-16 and units 3-8 remain outstanding.

## Implementation review

- Alignment keeps both coefficients in 192 bits: the largest aligned value is
  `(2^96 - 1) * 10^28 < 2^190`. Only the smaller-scale operand is multiplied.
  The 28-step counter terminates at zero and subsequent steps are identities.
- Signed addition/subtraction uses 193-bit operands and an exact sum/difference.
  Equal magnitudes preserve the frozen left-sign rule, including signed zero.
  Comparisons treat both signed/scaled zero representations as numerically equal.
- Multiplication keeps the complete 192-bit product through 96 shift/add steps;
  scale addition needs eight bits because the maximum sum is 56.
- Division scales an aligned coefficient by `10^28` in 288 bits. The bound is
  below `2^96 * 10^56 < 2^283`. Restoring division uses a 193-bit remainder and
  192-bit divisor, consuming every numerator bit in 288 steps. The initial
  nearest-even decision compares twice the remainder in 194 bits with the divisor.
- Fitting retains the last decimal digit and sticky remainder and rounds once
  at the chosen scale. It does not feed rounded candidates into later reductions.
  At most 56 reductions plus one terminal decision suffice. Completed fit states
  preserve all fields. For division, a nonzero original fractional remainder
  participates in sticky rounding after any subsequent reduction.
- Remainder preserves dividend sign and aligned scale. One aligned operand is
  still at most 96 bits, so a valid nonzero-divisor remainder is representable.
  Divide-by-zero is the first declared failure; unsuccessful normal results are
  zeroed by the existing ordinary circuit emitter.
- Pipeline finalization uses the last phase position, validates one input,
  one repetition and the public result width, and uses the finalizer's ordered
  failure definitions. It does not infer finalization from a helper's reused name.
- Helper reuse requires exact equality of serialized gate topology, widths,
  circuit start, physical results/failures, argument/result types, operation tag
  and ordered checks. Only the internal operation identifier is excluded.
  Public metadata is rebuilt with the requested signature. Explicit transformer
  repetitions remain charged independently of shared declarations.
- The combined 45-operation certificate retains distinct public names and
  contains 173,588 terms, 2,270 declarations and 3,858 transformer occurrences.
  All counts fit the unchanged bounds. The 33 previous individual certificate
  bytes and metrics remain unchanged under exact helper sharing.
- The original-source tests require exact emitted operation sets and verify
  source/foundation/signature/check/name/byte mutation rejection. The previous
  mixed conversion/equality rejection is now positive because both operations
  are implemented. Typed literals and application proofs are not claimed here.

## Observer review and completed validation

The test-only evaluator uses an explicit continuation stack instead of native
recursive reduction. Each suspension captures its own environment, each lambda
owns its own memo table, and cube storage is shared with offset/stride selectors.
Captured environments are persistent binding lists: one binding adds one node,
and de Bruijn lookup walks from the newest binding without changing shadowing.
The memo stores two optional values, indexed by already known Boolean arguments.
Suspended arguments stay lazy, including Boolean arguments. A poison-suspension
regression verifies that an unused lambda argument and an unselected Bool branch
are not demanded; poison terms are test inputs, never certificate evidence.
The evaluator performs ordinary beta/let reduction and the existing Bool eliminator, without recognizing decimal names
or substituting circuit/oracle results. Both production checkers are unchanged.

An earlier recursive observer overflowed its native stack. An explicit-stack
attempt then consumed about 17 GB while evaluating suspended Boolean arguments;
that known process was deliberately stopped to correct memoization. Neither
attempt is a passing check or a checker rejection. A subsequent Boolean-memo
run passed the seven prior core test functions but again grew to about 10 GB
during arithmetic observation; it was stopped to replace copied environment
vectors with shared binding lists. A further attempt reached about 13 GB during addition and was stopped to replace generic Boolean memo trees
with two fixed slots. Review then found that eagerly forcing Boolean arguments
undermined lazy evaluation. That optimization was removed and a regression was
added; shared environments and compact memo slots remain.

Review of the arithmetic and helper-sharing implementation found no further
source-level issue. All eight same-byte dual-checker cases passed with zero axioms and hash-mutation
rejection. The final lazy observer passed all eight core test functions, including sixteen
combined-certificate observations, in 3628.93 seconds. No findings remain in this
component review.
The full `check-fast.sh` gate remains deferred to T06-W12.
