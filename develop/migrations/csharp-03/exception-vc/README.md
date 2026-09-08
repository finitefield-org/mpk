# T06-W05: exceptional-control VC handoff

`ExceptionVcProgram` generates pending ordinary obligations from independently
validated VIR and original source handlers. Its domain-separated digest and
sequent IDs are bound into the existing VC wire. Exact reconstruction rejects
changed regions, ordering, state, omitted goals, completion rules and runtime
identity fields; serialization alone cannot create a validated program.

The handoff retains each native node, SSA phi, invocation, check, abrupt value,
closed exception payload, ownership snapshot and unwind plan. Every normal or
exceptional edge has a sequent (the synthetic function-entry edge binds inputs).
Abrupt throw records with outgoing exception edges describe the originating
completion; they do not create a second uncaught exit. Uncaught obligations
occur on actual terminal throws or exception edges into the native exit.
Built-in/callee failure values use `<edge-id>.exception`, constrained by their
operation/check relation; explicit throws use the original abrupt SSA value.
Postcondition slots refer to the completed edge, while `old` slots remain at
method entry. Edge ownership records source entry and destination entry; native
snapshots and cleanup actions retain the intervening transformation.

Search predicates select the first candidate whose closed type matches and
whose filter completes normally with true. Earlier candidates must be rejected.
A throwing filter is false and preserves the original pending exception; local
catches inside a filter still run. Original transfer candidates and finally
entries retain two-pass search-before-unwind ordering, including cross-frame
calls and outer filters. The finite finally table preserves every incoming
normal/return/break/continue/throw value and target on normal cleanup; a throwing
cleanup replaces the completion and restarts search. Outward return/break/
continue from cleanup have no admitted rule.

Normal postconditions are emitted once, reusing an existing W04 return sequent
when present. Exceptional cases select the first applicable exact type and path,
with separate definedness/assignment goals, conditional ensures and coverage.
An explicit empty throws set produces false coverage; a missing contract leaves
summary inference pending. Source-defined exception checks require an admitted
closed definition and an exactly matching failure type. W05 groups depend on
applicable W03/W04 groups; no proof status is accepted from captured metadata.

`goldens.json` covers the 46 admitted original exception/handler captures in
`../control-emission/source-cases.json`. `requests.json` and `responses.json`
contain three additional real Roslyn captures for valid, false and empty
exceptional postconditions, produced with the existing Linux
`--test-control-emission-requests` harness. Tests vary filter outcomes, compare
finally rules with the source completion algebra, and reject handoff mutations.
Existing canonical CLR/two-pass trace and malformed native-region tests are
also run locally. Current construction golden VC hashes change because the
wire now commits W05 obligations; construction/data/control handoff hashes and
frozen producer/vector bytes remain unchanged.

These are pending VCs and expansion recipes, not semantic/kernel proof receipts.
W09 owns ordinary expansion, proof construction and dual checking. The full
`./scripts/check-fast.sh` gate is deferred to the final T06-W12.
