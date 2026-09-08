# W09 unit 3 relation checkpoint review

Latest direct review: no findings. The relation component passed its scoped
verification; unit 3 and W09 remain incomplete. See
`unit-3-relations-verification.json` for the exact commands, logs and limits.

## Reviewed scope

- Reachable storable carriers are reconstructed from validated original VIR.
  Internal sequence-construction state is excluded explicitly. Exact metadata
  and certificate bytes are regenerated on import; context/foundation/hash or
  definition substitutions cannot serve as authority.
- Equality reads semantic fields, active payloads and ordered sequence prefixes.
  Canonical ordering uses signed scalar semantics, numeric decimal semantics,
  field/tag/element order, full lengths and currency-before-amount for money.
  IEEE values and closed exceptions, including enclosing types, have no public
  canonical comparison. Domain/public-invariant obligations remain separate.
- The common fold is reused rather than changing its frozen concrete transformer
  construction. Original-source tests extract the actual predicate index word
  and check upper index bits through the 4,096/16,384 boundaries. Existing full
  fold tests remain the evidence for executing the inclusive fold capacity.
- The test-only evaluator was moved without semantic changes (verified against
  the committed implementation after formatting and the `eval` visibility
  adjustment). Existing lazy-argument and storage regressions pass.
- The existing certificate structure guard rejects imports, proof-node/theory
  tables, axioms/theory primitives and nonzero recomputed axiom inventories.
  All 22 pinned certificates passed the unchanged same-byte dual-checker harness,
  including matched report hashes and rejection of each changed hash.
- Scope stays within unit 3. These are foundation definitions and observations,
  not proofs of application VCs, recursive domains, defaults or public clauses.

## Findings resolved during implementation

1. Raw captured source members contain names/order, not normalized stored-member
   IDs. The sample generator now pairs the preserved declaration order with the
   carrier fields and checks the lengths; the original source suite passed.
2. Initial samples did not distinguish money's currency-first order from amount
   order. The new values make those orders disagree. Extra cases cover later
   source fields, equal-length differing elements, nullable NaN and zero signs,
   decimal scale/sign representations, user-exception payloads and ignored empty
   storage. Identical complete input pairs are deduplicated without merging
   distinct representations.
3. Source observations on the default test stack aborted in the Money case.
   The suite uses the existing decimal tests' 64 MiB stack. The initial source
   matrix then passed 408 core pairs in 804.30 seconds. The stronger matrix
   passed 979 value-pair/storage observations in 1,859.76 seconds. Its exact
   metadata and bytes match the pinned, dual-checked certificate inventory.
4. Checking only short sequences would miss the new index-word expression's
   higher selector bits. The fast source certificate test now observes that
   generated expression directly at every power-of-two boundary and the last
   permitted index. It also validates every sample before expensive core work.
5. Three manual divisibility expressions failed lint and were corrected. Moving
   the observer adds exactly one namespace consumer. Removing it reproduces the
   previous fingerprint; the linked inventory hash and aggregate count were
   updated, preserving the historical cache entries. All five inventory tests
   and the latest scoped lint/format checks pass.

6. The broader matrix compared equal currencies only on identical Money
   values. A dedicated actual-source test now checks same-currency
   numeric equivalence with different decimal representations and same-currency
   ordering with different amounts. Both passed in 223.50 seconds. Two needless
   Box replacements in its sample construction were changed to assignments into
   the existing boxes; the resulting test code passes lint.

## Checkpoint result

The component is ready to commit/push. Source definitions, both checkers,
metadata/context/hash mutations, scalar boundaries, high index bits, source
observations, Money tie-breaking, storage regressions, inventory and scoped
lint/format checks pass. No test was restarted because an observation expired;
the original strengthened source run and Money run reached terminal success.
Recursive domains/defaults, collection mutations, the remaining foundation
operations and W09 units 4-8 remain open. The full T06 `check-fast.sh` gate stays
deferred to W12. This is not a W09 completion receipt or an application proof.
