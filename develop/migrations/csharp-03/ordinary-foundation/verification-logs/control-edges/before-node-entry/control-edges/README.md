# Source guards, slot transport and native phi inputs

The generator reconstructs the original W04 control VCs and lowers supported
edge guards to ordinary Boolean definitions. Integer failure predicates retain
their source argument order and ordered failure conditions. A missing semantic
constant leaves the entire guard and its transport relation explicitly pending.

For a taken edge, transport preserves assignedness and each assigned slot's
complete physical value from the source exit to the edge-specific target
observation. Both observations retain the exact edge ID. Inactive payloads are
unconstrained; a false guard imposes no transport constraint. Function entry
uses the separate slot-entry relation, and terminal edges have no target state.

The relation does not identify the incoming state with a node-entry snapshot:
a loop backedge must not equate its result with the preceding iteration's entry.
`state_rule` explicitly retains `node_entry_merge_pending`. Execution producing
the source-exit state, node frames, entry merging and application proofs remain
separate obligations. These are open relations, not reachability proofs.

Seventeen original sources provide 703 guards and 666 transport relations. All
38,159 current observations pass. Original W04 goal-binding comparisons cover
95 identities in the first partition and 36 in the collection partition. All seventeen
certificate byte sequences, including the phi extension, pass both unchanged checkers, report agreement,
zero-axiom checks and hash-corruption rejection. Exact regeneration rejects
source, guard, transport, snapshot-rule and certificate mutations.

Construction bounds and four exact source-point ownership bindings now resolve
all ten formerly pending `index_update` guards. The ownership failure alias is
installed only while generating an edge at that function/node/receiver; it is
never interpreted as a globally false check. Its flow and receiver theorems
remain checked dependencies in the same certificate.

String length/index, public sequence index checks and Option extraction now use
their ordinary definitions at original guard bindings. The former five pending
foreach_string guards are generated. One type-pattern SwitchExpressionException
edge remains pending, and removing that pending marker is rejected. Other data
and exceptional families still need definitions and integration. Direct loop
coverage for reference adapters and combined option/sequence emission also
remains open. Unit 5 and W09 remain incomplete. Verification and review are in
`../verification-logs/control-edges/verification.json`,
`../verification-logs/control-edges/construction/review.md` and
`../verification-logs/control-edges/collections/review.md`, with the overall review
in `../unit-5-control-edges-review.md`. The full gate remains T06-W12.

Native phi relations now cover 62 incoming edges and 100 selected values. Each
relation uses the exact original edge guard and the validated target phi's
operand for that predecessor. Copies are simultaneous, preserve the native
carrier including private construction memory, and retain an incoming edge ID
on both endpoints. They do not equate a backedge with the old loop entry.

All seventeen previous guard/slot programs retain exact term/declaration
prefixes and metadata after stripping the added phi field. The current
certificates have fresh dual-checker and hash-rejection receipts. Reviewed
runtime inputs distinguish phi values and flip low/high physical bits. See
`../verification-logs/control-edges/phis/verification.json` and `phis/review.md`
in that log directory. Six retained transfers in index_update/type need
source-slot/native-representation projections; non-store source nodes cannot
all be framed as unchanged. These projections, entry merging, native execution
and full application proofs remain open.

Four source array transfers now have native memory bindings: one store and
three loads. The original source anchor remains an allocation identity; loads
refer to the current SSA storage in the exact pre-invocation ownership state,
and the store uses its exit normal state. Full private storage is compared,
including the 4096/4097 and 16384 boundaries. The public source slot projection
remains explicitly pending. The changed index_update pin passes both checkers
and 4,559 observations; sixteen other pins are byte-identical. Current evidence
is in `../verification-logs/control-edges/memory-bindings/verification.json`.


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
