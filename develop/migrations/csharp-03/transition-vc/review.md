# T06-W08 direct review

Scope: transition/source-snapshot VC generation, strict reconstruction APIs,
VC ownership/dependencies/resources, original-source tests and task records.
Review was performed directly without delegation.

## Findings resolved

- Input invariant definedness cannot be hidden behind the invariant's truth.
  Admission now explicitly defines caller requirements including definedness,
  public domains, method preconditions and explicit time. Retained-key
  uniqueness belongs to admission rather than asserting that every State has
  unique history. Output invariants remain separate actual-return goals.
- Fixed error selection by string name confused valid disabled-mode business
  errors with idempotency rules. Selection now uses validated mode and ordinal.
  Two new real Roslyn captures exercise `idempotency_conflict` and
  `history_capacity` business codes; error path names have a distinct prefix.
- Business predicate definedness must hold only under its reached error prefix.
  The ordered partition preserves replay before every later check, followed by
  version, capacity, exhaustion and business priority. Independent Boolean
  valuations test the generated priority and single-path partition.
- Equality cannot be delegated to a digest or only the mapped semantic fields.
  The helper goal compares actual source execution with complete Command and
  Context equality and a separate canonical-field equivalence goal. DAG members
  are checked against independently attached snapshots, including inactive
  storage. The omitted-Context source body yields finite CLR counterexamples.
- Error State preservation concerns post-Apply input State, not an inactive
  success payload. Success/replay goals use the actual projected return;
  replay requires unchanged State, zero events and the full stored Response.
  Actual CLR observations expose broken invariant/version/event/response goals.
- Substituting free subjects must preserve local lambda/let binders. Explicit
  capture-avoidance tests cover shifted free values and unchanged local indices.
  The exact W03 definedness routine is shared without regenerating all data VCs.
- Separately serialized path guards need resource accounting, and pairwise
  partitions must be bounded before quadratic cloning. Node/name/binder counts
  include every serialized term; strict import rejects omitted paths/sequents,
  source functions, contracts, definitions and snapshot members. Integration
  checks bind all W08 sequents/digest and predecessor dependencies in the VC wire.
- Older ledger statements assigned proof discharge to W08. They now distinguish
  W08 VC generation from W09 ordinary expansion, proof construction and checkers.

## Final review

No findings. Rechecked exact source/contract lineage, actual return projections,
input admission, clause scope/definedness, priority and branch identity,
complete snapshot storage, strict reconstruction, resource reservations and
changed-file scope. No producer/vector, certificate schema or checker acceptance
rule changes are introduced. W09 retains proof/checker ownership and becomes
ready; the full T06 gate remains deferred to W12.
