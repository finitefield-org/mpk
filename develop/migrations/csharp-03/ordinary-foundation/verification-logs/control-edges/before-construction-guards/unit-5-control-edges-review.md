# Source guard and edge transport review

The generator independently reconstructs W03 data and W04 control programs.
Every function and edge retains its complete original metadata. Only supported
binder-free guard constants are lowered; unresolved constants leave the guard
and transport pending. Integer definitions reuse the existing emitter, whose
three predecessor pins remain byte-identical after extraction.

Argument order was checked against original guard bindings, then each slot's
source assignedness/value and target assignedness/value. Closing lambdas reverses
indices once. A true guard requires matching flags and complete physical equality
for assigned payloads. False guards and inactive payloads impose no fabricated
state. Entry and terminal edges do not emit this transport relation.

Review found that the first version used a target node-entry snapshot, while
W04 post-edge goals reference the target with the incoming edge ID. In addition
to missing those goal identities, this could alias a loop result with a previous
iteration's entry. The intermediate target-incoming to target-entry variant was
also rejected before promotion. The final relation transports source exit to the
edge-specific target, preserving the same edge ID on both ends. No target entry
snapshot occurs. Node-entry merging is explicitly pending in `state_rule`.
Superseded artifacts and checker receipts are archived under
`verification-logs/control-edges/before-incoming-point-fix/`; their earlier
kernel acceptance is not evidence that their source bindings were correct.

The final nine-source suite includes actual variable decrease goals and compares
95 distinct original W04 state bindings, including assignedness on decrease
entry/backedges. It exercises 11,718 observations with integer boundaries,
exception ordering, disabled guards, flag changes, high physical bits and
inactive storage. Metadata and certificate mutations reject import, including
removing the snapshot rule or replacing the target edge ID with null.

The final runtime suite passed in 93.40 seconds; all nine identical certificate
byte sequences subsequently passed Go/Rust checking, zero axioms, report
agreement and hash-corruption rejection. Promoted files match those verified
candidate hashes exactly. Relevant clippy, format and five inventory tests pass.
Inventory adds one Std consumer (135 to 136; total 4958 to 4959). Unchanged slot
and integer semantics were not rerun without a relevant change.

No remaining defect was found within the supported guard/transport component.
Ten index_update guards still need construction length/index checks and exact
source-point ownership dependencies. Native execution, source-exit production,
node frames, node-entry merging, exceptional entry and application proofs remain
open. Neither the runtime observations nor the checked open definitions discharge
these obligations. Unit 5 and W09 are incomplete; no component-only commit is
made. The full gate remains T06-W12.
