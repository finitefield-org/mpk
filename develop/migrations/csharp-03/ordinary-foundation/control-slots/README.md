# Source-bound slot entry and transfer relations

These ordinary Boolean definitions lower the W04 slot rules at their original
source anchors. Entry relates each parameter to its exact SSA value and marks
locals unassigned. A successful load requires an assigned slot and exact storage
agreement with the retained SSA result. A successful store or pattern binding
assigns its result to the target slot; every other assigned slot is framed.
Inactive slot storage remains unobservable and is not forced to a fabricated
zero/default value. Storage agreement is physical, including all carrier bits,
so it does not conflate signed zeros, decimal encodings or NaN comparisons.

Metadata retains the independently reconstructed control program, function,
source transfer, node endpoints, complete argument order, types and SSA IDs.
Imports regenerate the whole program and reject every byte or metadata mismatch.

These are open normal-execution relations. They neither assert reachability nor
prove source execution. Scoped guard and edge transport are now recorded in
`../control-edges/README.md`. Node-entry merging, non-transfer node frames,
caught-exception entry, exceptional transfers, loop sequents and application
proof assembly remain pending. `application_scope_pending` remains true. Unit 5
and W09 are incomplete; the full repository gate remains T06-W12.

The six pinned source contexts pass 1,482 runtime/linkage observations, identical-
byte Go/Rust checking, report agreement, zero-axiom checks and hash-corruption
rejection. See `../verification-logs/control-slots/verification.json` and
`../unit-5-control-slots-review.md`. Explicit execution scopes distinguish
function entry from successful transfer; no relation applies to a throwing edge.

The source-slot checkpoint now covers nine source contexts and 130 relations,
with 2048 observations. Private array source snapshots admit partial initialization
under the 4096 profile bound; complete public projections remain separate.
Nullable slot storage preserves Option presence and payload. The three array
certificates, nullable source certificate and Some storage helper passed both
unchanged checkers and hash rejection. See
`verification-logs/control-slots/public-projection/verification.json` and
`verification-logs/control-slots/nullable-representation/verification.json`.
Nullable edge transport, non-transfer memory effects and application proof
assembly remain open; this does not complete W09.


Nullable edge transport now retains the complete Option storage representation
for the `type` source's 53 joins. Its 2,758 observations replace 2,016 previous
observations; the current edge total is 38,901. Sixteen other edge programs are
byte-identical, and the changed certificate passes both checkers and hash
rejection. See `verification-logs/control-edges/nullable-slots/verification.json`
and its review. Nullable slot-to-edge transport is implemented; mapping W04
logical observations, node-entry merging, non-transfer memory effects and
complete application proofs remain pending. The changed fixture contains zero
W04 goal bindings. Unit 5 and W09 remain incomplete.
