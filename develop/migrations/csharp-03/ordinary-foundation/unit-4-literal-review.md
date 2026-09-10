# W09 unit 4 literal review checkpoint

This component lowers validated VIR literal bodies and failed-check exception
values. It does not yet connect boundary-run constants, native invocations or
application VC proofs. Unit 4 and W09 remain incomplete.

Direct inspection found and fixed two issues before acceptance:

- An unprefixed digest could start a core global-name component with a digit.
  Actual-source certificate decoding rejected it. Roots and shared parts now
  prefix their digest components with H; exact source replay checks decoding.
- Applying a closed child lambda under a deep product's selectors can exceed
  the frozen binder bound even when the carrier itself fits. Children now become
  shared ordinary definitions before placement. The depth-253 regression checks
  every padding selector, mixed field depths and repeated sharing. Depth 254
  and an undersized child slot reject.

The source capture initially used two excluded framework static-field reads.
They were replaced with admitted arithmetic expressions; all 21 final requests
passed deterministic two-run capture in the existing offline Linux image.
Their arithmetic results are not claimed as native literal bodies. Dedicated
scalar edge cases separately test raw NaN and decimal-cohort encoding.

Review checked closed-term placement, little-endian selector order, padding,
source field order, active-tag payload depth, fixed sequence capacity, signed
integer parsing through i128 (including u64 maximum), 128-bit Guid storage,
96-bit decimal coefficients, exception payload flattening and exact importer
reconstruction. Source origin bindings are retained even when values are shared.
No oracle computes the generated semantic body, and no source/body proof is
invented. Final checker, pin, lint and inventory results are tracked separately
in unit-4-literal-progress.json; no final whole-unit review is claimed here.

All 64 source pins and two supplemental edge pins passed both unchanged
checkers on identical bytes with zero axioms and rejection of every tested hash
corruption. Source/request/edge exact replay, scoped lint, format and five
inventory tests passed. No remaining actionable finding was identified in this
bounded component review. This does not close the full original unit 4 or W09.
