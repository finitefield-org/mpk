# T06-W06 direct review

Scope: new binding/foundation generator and strict importer, validated binding
cache, VC groups/resources, original-source fixtures, targeted tests and task
records. Review was performed directly without delegation.

## Findings resolved

- Default eligibility is a use restriction, not proof that an ineligible CLR
  default violates its source invariant. Eligible defaults retain both domain
  and exact-arm goals; ineligible defaults retain the pending use restriction.
- Returned application Result values must project and unwrap their success
  payload before comparison with a semantic scalar result. Explicit error
  carriers preserve semantic check precedence. Real instant/money captures and
  the cross-program constant-signature check exercise this correction.
- Identity projections admitted for primitive operands are not application
  sidecar entries. Retain their typed identity equations separately; both real
  operation captures now generate and import successfully.
- A matched source check's prefix is determined by its actual source position,
  including earlier unmatched checks. Unmatched checks are unreachable only
  under that prefix. The unit counterexample checks both assumption counts.
- A source method can implement multiple same-signature semantic operations.
  Contextual IDs and recipes now include binding, source and semantic identity;
  a synthetic second mapping checks uniqueness, and duplicate generated IDs
  are rejected before handoff.
- Equivalence must include uninvoked operations and exceptional/error behavior.
  The complete reachable operation table and mandatory AllOutcomes goals remain
  in the handoff, alongside normal result equations.
- Reconstruction includes every public stored member, including inactive arm
  payloads. Source invariants and all semantic relations remain obligations;
  shape validation or sidecar role classification cannot grant them.
- Resource reservations include W06 nodes, sequents, binders and the shared
  definition-name union. Existing native projection-plan limits remain enforced
  by validated import; generated sequents use the VC resource limits. Removed
  an unused counter instead of conflating those two units.

## Final review

No findings. Rechecked source/descriptor linkage, exact complete table import,
concrete term signatures, outcome precedence, stable IDs, owner dependencies,
resource accounting, current golden deltas and task status. Only two current
construction whole-VC hashes change; prior handoff hashes and frozen source
producer/vector bytes are preserved. The receipt distinguishes formula
countermodels and structural acceptance from semantic/kernel proof acceptance.
W09 retains ordinary expansion/proof/checker ownership, W07 alone becomes ready,
and the T06 full gate remains deferred to W12.
