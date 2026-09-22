# Binding reconstruction audit: stop classification retracted

`CSHARP-03-T06-W09` remains In progress. The 2026-09-22 noninjectivity
observation is correct, but its classification as a T01 freeze contradiction
was mistaken. `remapped-boundary-sequence` was added in W09 commit
`09a14bdde6b14408e4f25e55cb787dd989577d7e`. Neither the private freeze nor the
published specifications contain it. The original projection review explicitly
leaves inverse definitions and universal proofs pending. A definition test or
candidate VC is not an accepted verified compilation.

The executable probe constructs two public `ProjectionCases.Root` values that
differ at stored member `Extra`, confirms their complete storage differs and
their semantic projections agree. For `P(x) = P(x')` and distinct admitted
source observations, a unary function `R` cannot satisfy both source round
trips. Final proof acceptance must therefore reject this candidate.

This does not invalidate the frozen obligation or justify a new blanket rule
rejecting all unmapped members: a public invariant may determine an unmapped
field. The normative paragraph introduced with the erroneous stop judgment is
removed. T01, its hashes and valid component evidence remain unchanged.

The binary rebuild correctly preserves an explicit source completion. W09 must
connect unary candidates to the original public-domain, both round-trip and
stored-member obligations. Choosing a completion is not a proof of those
conditions. The negative witness remains a regression requirement. No W09
completion or W10 unblock is claimed; the T06 whole gate stays at T06-W12.
