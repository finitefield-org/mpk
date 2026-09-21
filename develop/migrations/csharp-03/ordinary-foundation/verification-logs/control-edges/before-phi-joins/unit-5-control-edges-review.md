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

The initial nine-source suite includes actual variable decrease goals and compares
95 distinct original W04 state bindings, including assignedness on decrease
entry/backedges. It exercises 11,718 observations with integer boundaries,
exception ordering, disabled guards, flag changes, high physical bits and
inactive storage. Metadata and certificate mutations reject import, including
removing the snapshot rule or replacing the target edge ID with null.

That runtime suite passed in 93.40 seconds; all nine identical certificate
byte sequences subsequently passed Go/Rust checking, zero axioms, report
agreement and hash-corruption rejection. Promoted files match those verified
candidate hashes exactly. Relevant clippy, format and five inventory tests pass.
Inventory adds one Std consumer (135 to 136; total 4958 to 4959). Unchanged slot
and integer semantics were not rerun without a relevant change.

The construction extension subsequently resolved all ten index_update guards
with three construction definitions and four exact source-point ownership
bindings. The changed source passes 3,991 observations and both checkers. The
other eight sources regenerate to identical bytes, preserving 9,919 observations
and their same-byte checker receipts. The current scoped total is 13,910
observations, 360 guards and 341 transport relations. The separate foreach_string
negative regression retains five unsupported guards. See
verification-logs/control-edges/construction/review.md for the extension's review,
fixture correction and exact test/checker evidence.

The subsequent collection/Option extension generates the five previously pending
foreach_string guards and two Option extraction guards in the type pattern.
Seventeen contexts now cover 703 guards, 666 transports and 33,661 current
observations. All seventeen certificates have same-byte dual-checker acceptance
and hash-corruption rejection. Thirty-one existing data contexts and the nine
previous control contexts regenerate unchanged. The type pattern retains its
modeled SwitchExpressionException edge explicitly pending. See
verification-logs/control-edges/collections/review.md for exact partitions and
the remaining reference/mixed-foundation source coverage.

No remaining defect was found within the exercised guard/transport component.
Other data families still need guard definitions. Native execution, source-exit production,
node frames, node-entry merging, exceptional entry and application proofs remain
open. Neither the runtime observations nor the checked open definitions discharge
these obligations. Unit 5 and W09 are incomplete; no component-only commit is
made. The full gate remains T06-W12.
