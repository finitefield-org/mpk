# Unit 5 symbolic ownership equation component review

Reviewed the generator against the original source operations, existing symbolic
ownership validator, captured trace structure and frozen certificate limits.
This review closes the equation component only; it does not close unit 5/W09.

- State identity is allocation origin plus a full 32-bit SSA version token.
  Unused physical slots are zero. Shared word equality compares every bit.
- Allocate requires an absent origin; write replaces the existing receiver;
  read/complete requires possession; freeze and discard consume the token.
  Source actors match the enclosing function. No host-derived successful Bool
  replaces these state relations.
- Every source CFG edge is covered exactly once, including delayed backedges.
  Exceptional edges start from the invocation input. Cleanup clears exact
  allocation origins, its incoming union is checked, and phi rewriting is
  followed by equality to the target entry state. Entry and terminal conditions
  are explicit ordinary predicates.
- Point failure terms only test possession in the supplied state. They cannot
  independently establish reachability, uniqueness, or application safety;
  `pending_flow_proofs` keeps their required proof dependency open.
- Exact regeneration binds source/foundation identity, equations, token tables,
  candidate witnesses and canonical bytes. Missing equations/proof requirements,
  modified bytes and checker hash corruptions reject. Concrete untraced ownership
  functions are separately pending.
- The added alias test exposed a test assumption, not a generator defect: two
  allocations need not overlap in lifetime. The corrected test requires an
  identity transition with multiple live tokens in the dedicated eight-live
  source, and retains the separate-lifetime source. Production bytes were
  unchanged; the already passing dual-checker results remain applicable.

Initial equation checkpoint: 14 source fixtures, 7,457 equations, 14,999 observations,
14 same-byte dual-checker cases, lint, format and consumer inventory. See
`verification-logs/ownership-equations/verification.json`. Broader unchanged
scalar/construction evaluation is not rerun. The full gate remains T06-W12.
No component-only commit, W09 completion receipt or downstream unblock is made.


The subsequent encoding review retains all 32 token bits and every flow
conjunct. Shared source-state definitions and reduced selector trees do not
change allocation/SSA identities. Each WordEqual prefix adds exactly the next
bit comparison; the final balanced conjunction retains source equation order.
The additional physical-state checks raise the independent runtime observation
count to 19,831. Exact regeneration and all 14 same-byte dual-checker cases pass
for that prefix checkpoint; see
`verification-logs/ownership-proofs/prefix-equation-verification.json`.
This does not transfer acceptance to the separate proof candidates.

The current eight-origin comparison uses a balanced tree of disjoint ranges;
its leaves cover bits 0 through 31 exactly once. The other thirteen equation
certificates retain their accepted bytes. All fourteen pins regenerate exactly;
the changed eight-origin case passes 2,505 runtime observations, both unchanged
checkers and hash-corruption rejection. Current total coverage remains 19,831
observations. See `verification-logs/ownership-proofs/current-equation-verification.json`.
