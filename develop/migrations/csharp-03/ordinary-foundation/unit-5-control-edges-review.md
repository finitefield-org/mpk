# Source guard and edge transport review

Current entry-merge review: `verification-logs/control-edges/node-entry/review.md`.
The following checkpoints are chronological; their former pending items are
superseded only where the later checkpoint explicitly implements them.

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

The native phi extension adds 62 guarded incoming relations for 100 selected
values. Exact validated predecessors, native types, simultaneous copies and
edge-specific snapshots are retained; source-slot conversion is not inferred.
All seventeen certificates pass both unchanged checkers, report agreement, zero
axioms and hash rejection. Reviewed inputs pass 38,059 observations in 124.72
seconds of test execution, with prior guard/slot definitions preserved exactly.
Clippy and formatting pass. See verification-logs/control-edges/phis/review.md
for the strengthened preservation/mutation checks and the six unresolved
logical-slot/native-representation transfers. Node-entry merges, private memory
projections, native execution and application proofs remain incomplete.

Native memory bindings now retain the original allocation identity while
resolving one store and three source loads against exact ownership states.
The three loads select newer native storage IDs. The changed index_update
certificate passes both checkers and hash rejection; 4,559 observations include
100 new private-storage equality/corruption cases. All sixteen other contexts
regenerate unchanged. The current total is 38,159 observations, with every
previous guard/slot/phi definition retained. Source public-slot projection,
nullable presence and general memory effects still require implementation. See
verification-logs/control-edges/memory-bindings/review.md for precise scope.


Nullable edge transport now retains the complete Option storage representation
for the `type` source's 53 joins. Its 2,758 observations replace 2,016 previous
observations; the current edge total is 38,901. Sixteen other edge programs are
byte-identical, and the changed certificate passes both checkers and hash
rejection. See `verification-logs/control-edges/nullable-slots/verification.json`
and its review. Nullable slot-to-edge transport is implemented; mapping W04
logical observations, node-entry merging, non-transfer memory effects and
complete application proofs remain pending. The changed fixture contains zero
W04 goal bindings. Unit 5 and W09 remain incomplete.


The captured SwitchExpressionException throw now has an ordinary local guard
and slot transport bound to its exact source/native anchor, frozen tag-8
literal, exception check and target. All existing definitions remain an exact
prefix, and the sixteen other edge contexts are unchanged. The changed source
passes 2,807 observations and same-byte Go/Rust checking with hash rejection;
the current corpus has 704 guards, 667 transports and 38,950 observations.
See `verification-logs/control-edges/builtin-throw/verification.json` and its
review. All guards in these seventeen sources are defined; node reachability,
handler semantics, W04 observation mapping and complete application proofs
remain open. Unit 5 and W09 remain incomplete, with the full gate at T06-W12.


Node-entry relations now cover all 617 native blocks across the seventeen
original control sources. Each relation requires exactly one enabled incoming
edge and preserves its assigned slot/phi snapshot at a fresh entry; other
predecessor states remain independent. The 13,576 new observations pass,
including nonzero values, missing/double selection and physical corruption.
All fourteen distinct certificate byte sequences (seventeen source contexts)
pass both checkers and hash rejection. The previous 38,950 edge observations
retain exact declaration prefixes, giving 52,526 scoped observations. See
`verification-logs/control-edges/node-entry/verification.json` and its review.
Source-exit production, non-transfer memory effects, W04 observation mapping
and full native/application proofs remain open. Unit 5 and W09 are incomplete.

## Successful receiver memory effect checkpoint

The original `index_update` rewrite now relates exact source receiver snapshots
to native before/after allocation states, with 13 new runtime observations.
Both checkers accept the changed bytes and reject hash corruption. Existing
edge/entry declarations and sixteen other programs are preserved; all nine
slot pins replay unchanged. See
[scoped review](verification-logs/control-edges/memory-effects/review.md).
The `count_fill` entry arity issue and remaining frames/exceptions/application
proofs are explicitly open. Unit 5 and W09 remain incomplete.

## Count/fill control checkpoint

Factoring the 501-argument exit into twelve conjunctive components permits the
original count/fill source to emit within the binder limit. All 148 edges, 129
node-entry records and its receiver rewrite have scoped runtime coverage:
20,598 new observations, including every selector combination. Both checkers
accept the same bytes and reject hash corruption. Seventeen previous programs
remain byte-identical. The corpus now contains eighteen contexts. See
[review and applicability correction](verification-logs/control-edges/factored-entry/review.md).
Native execution, frames, invariants and application proofs remain open.

## Source-local frame checkpoint

All eighteen original sources now retain 703 source-frame records, including
580 ordinary local-preservation relations. Exact native anchors, original
transfers and nullable storage overrides are preserved. Assigned locals retain
their complete physical values; unassigned payloads remain irrelevant. Store
and pattern-bind targets are constrained by their separate transfer relations.
Ten unreachable nodes remain explicit, and initialization, exception execution
and the two array-alias frames remain separate.

The 10,608 new observations pass, raising the scoped total to 83,745. Fifteen
distinct certificate byte sequences pass fresh same-byte Go/Rust acceptance,
report agreement, zero-axiom checks and hash-corruption rejection. Seven
historical preservation tests retain prior definitions and metadata, and all
nine private slot pins replay unchanged. Targeted lint and formatting pass.
See [verification](verification-logs/control-edges/source-frames/verification.json)
and its scoped review for selection reasons and limits. Native execution,
transfer definedness, alias/exception state, invariants and application proof
composition remain open. Unit 5 and W09 remain incomplete; the full gate stays
deferred to T06-W12.

## Native normal-operation checkpoint

The eighteen-source W04 loop/pattern subset now binds 98 normal-operation
relations to original invocation inputs and normal-successor results. Each
relation requires the original success guard and result relation together.
Source-scoped ownership aliases apply only at their exact operation point.
Eight Option/compound operations, one non-data invocation and functions outside
this W04 subset still require integration.

The targeted runtime passes 592 observations in 356.44 seconds; six additional
failed-operation/default-result cases pass 24 observations in 20.39 seconds.
All fifteen distinct byte sequences pass both checkers and hash rejection.
Eight historical preservation tests retain earlier declarations and metadata;
the nine private slot pins and all source-frame observations pass. Targeted lint
and format pass. The scoped total is 84,361 observations. See
[verification](verification-logs/control-edges/native-operations/verification.json).
The first cumulative 300-second probe is retained as a budget failure, not a
semantic verdict. Native execution witnesses, source-state composition,
exceptions, invariants and application proofs remain open. No unit/W09
completion or component-only commit is claimed.

The native Option/compound extension resolves the eight pending W03 operations
in the current eighteen-source W04 subset. All 106 now have normal relations;
one non-data native invocation remains pending. Builder-owned relation/storage
caches preserve shared structural helpers. The 81 additional runtime observations,
nine declaration/metadata preservation tests, seven affected dependency tests,
source-frame replay and three fresh same-byte dual-checker cases pass. Fourteen
unchanged certificate contexts retain prior checker evidence by exact bytes.
See `verification-logs/control-edges/native-options/review.md` and its verification
receipt. The current scoped total is 84,442 observations. This does not establish
complete source execution or application proofs; units 3–8 and W09 remain open.

Source-slot integration now places 18 function-entry and 207 successful source
transfer relations in the same certificate as native operations and control
edges. All 6,342 new observations pass, complete standalone dependency bodies
and metadata agree, and ten historical preservation tests plus nine standalone
slot pins pass. All fifteen distinct current certificates passed both checkers
and hash-corruption rejection. The scoped observation total is 90,784. See
`verification-logs/control-edges/integrated-slots/review.md`. Complete execution,
exceptional flow, alias frames and native/application proofs remain open.

The user explicitly requested commit and push of the current work in progress.
This publication is a checkpoint of the accumulated W09 changes; it does not
close an internal unit or mark W09 complete. The whole gate remains deferred
to T06-W12.
