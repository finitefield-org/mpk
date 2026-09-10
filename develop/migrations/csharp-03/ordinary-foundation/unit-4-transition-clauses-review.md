# Transition clause projection review (partial W09)

Reviewed the three new projection recipes and their exact original-source
attachment paths. A matching physical product alone is insufficient: the
argument must resolve to the validated three-argument semantic Transition
instance. Role selection uses the validated stored field identity and checks
its precise nominal reference type, including the role-bound event sequence.

The value recipe aliases the existing product getter. Tests inspect the alias
target and compare its full ordinary dependency closure against independent
structural generation. This preserves every field bit and ordered event slot
without introducing a second projection encoding. Typed literal comparison,
scalar/compound samples and independently addressed raw product patterns cover
the six bindings and their field selection. Source attachment metadata imports
continue to require exact regeneration; changed result metadata rejects.

The only correction during this checkpoint was a test-code lint change from
an explicit modulo comparison to `is_multiple_of`. The original source tests
pass with six clauses and 999 bit observations. Existing tagged construction
and old/result attachment fixtures retain their pinned output. Inventory and
affected lint/format checks passed. Both unchanged checkers accepted the same
two byte sequences with zero axioms and matching reports, and rejected actual
hash corruption. Current candidate hashes, sizes, metadata and tested copies
were reconciled. No actionable implementation findings remain; results are in
`unit-4-transition-clauses-progress.json`.

This review covers the projection component. Actual native outcome bindings,
transition/replay propositions and application proof/certificate assembly
remain within W09 and are not discharged by these helpers.
