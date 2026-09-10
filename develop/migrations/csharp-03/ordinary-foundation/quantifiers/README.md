# Ordinary bounded contract quantifiers (partial W09)

The user clarified the missing endpoint rules: ranges are half-open
`lower <= i < upper`; reversed ranges violate definedness; statically known
expansion counts exceeding the existing limit reject generation. This is an
explicit task clarification, not an inferred change to a frozen artifact.

The contract compiler defines `bounded_forall` and `bounded_exists` for i32,
u32, i64 and u64 through the existing ordinary integer circuits and balanced
ordered fold. The bound checks ordering in the original signed/unsigned type
and the complete unsigned distance against 16,384 before body traversal. A
64-bit distance is checked before projecting the low count word. Offsets are
added in the original width, including signed intervals crossing zero. Empty
ranges give true for forall and false for exists. Invalid ranges have a zero
traversal count and false definedness.

Direct literal ranges exceeding 16,384 reject before network emission.
Symbolic ranges carry the same bound as a W03 definedness condition; an
oversized symbolic interval cannot become an accepted truncated traversal.
Every emitted concrete state transformer still counts toward the shared
inclusive static network budget. Reusing a fold does not reset that budget.

Both quantifiers use universal body definedness through the canonical W03
`ContractForall` symbol. Finding an existential witness does not hide a partial
read or failed arithmetic check elsewhere in the range. Nested and lexical
binders are compiled through the original typed contract terms. Source public
consumers retain their requirement to discharge partial-clause definedness.

Verification is scoped to new quantifier circuits, original-source contract
attachments, their compiler consumers, same-byte checker acceptance, and the
affected inventory/lint/format checks. Results and pending work are recorded in
`../unit-4-quantifier-progress.json`. Runtime observations and helper certificate
acceptance do not prove an application VC. Units 3–8 and W09 remain open; no
component-only commit is made. The whole T gate remains deferred to T06-W12.

The component checks passed: 116 core range/value cases, one full 16,384-element
traversal, poison-body checks for empty/invalid ranges, 320 actual offset-selector
observations, and 160 original-source clause observations. Six source contexts
also cover static oversized rejection and all three public-consumer gates.
All six certificates in `certificates.json` pass both unchanged checkers on
identical bytes with zero axioms and matching reports; actual hash corruption
rejects. Existing compiler consumer fixtures, inventory and affected lint/format
checks pass. The final component review has no actionable findings.
