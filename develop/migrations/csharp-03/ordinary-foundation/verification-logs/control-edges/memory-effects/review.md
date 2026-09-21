# Successful source array update — scoped review

This change appends one open relation for the original `index_update` source.
It connects the receiver slot's before/after bounded source snapshots to the
existing native rewrite relation and exact allocation/ownership states. Both
source snapshots must be assigned. Ordered failure predicates must all be false;
the ownership predicate is the existing source-bound scoped definition.
The native array result and the source expression's assigned-element result
remain distinct typed values.

## Correction found during verification

The first probe assumed a source update anchor had the same native entry and
exit. Actual lowering spans the rewrite invocation and its normal successor.
That assumption left the effect pending. The generator now locates the exact
invocation through the anchor's artifact node IDs and binds the after-snapshot
to its normal successor. The passing runtime regression independently verifies
both points, the source operation and its element result. The failed probe is
retained in the ownership-proofs logs; it is not acceptance evidence.

## Scope and review

- Source receiver identity is reconstructed from the original update's load
  input and exact slot transfer. A reachable intervening store/pattern binding
  to that slot leaves the effect pending, because alias identity is not yet
  modeled. Source evaluation paths stop at the update.
- Allocation identity is checked against the exact ownership `.invoke` and
  `.normal` live-memory states. Native subject types and the source result
  must match. There is no unknown-check fallback.
- Runtime inputs independently encode arrays `[7, 11]`, `[13, 11]` and
  `[7, 13]`. Thirteen observations include both valid index writes, false
  assignedness, stale/mismatched snapshots and native memory, incomplete
  initialization, negative/end indices and an incorrect assigned value.
- Import reconstructs the expected metadata and certificate, rejecting
  changed source/result/slot/allocation/state/ownership/snapshot/scope fields
  and omission of the memory-effect record.
- Six historical preservation tests cover all prior declarations and relevant
  metadata. Sixteen complete edge artifacts remain byte-identical. All nine
  dependent slot pins replay unchanged because effects append only in the
  public edge generator. Snapshot and source-slot runtime checks pass.
- Fresh dual-checker acceptance and hash-corruption rejection for the changed
  bytes are prerequisites for promotion; the verification receipt records
  their actual results. Four export changes reverse to the prior source hashes.

The relation constrains one receiver snapshot on successful native rewriting.
It does not frame other source slots, resolve aliases, execute exceptional
transfers or establish a complete body/application theorem. Open relational
arguments and finite observations cannot discharge those obligations.

## Remaining integration finding

The existing `count_fill` source has a node with eleven incoming edges. Its
flat node-entry relation needs at least 501 arguments before native phis,
exceeding the current 256-argument bound. This was established from original
slot/edge metadata, not from a runtime/checker verdict. See
`node-entry-fan-in-follow-up.json`. Entry checks must be factored without
weakening selected-edge or snapshot conditions before extending fill-loop
execution coverage. No fill-loop memory-effect acceptance is claimed here.
Unit 5 and W09 remain incomplete; the whole gate remains at T06-W12.
