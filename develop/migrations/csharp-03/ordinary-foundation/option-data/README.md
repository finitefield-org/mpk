# Option results at original SSA use points

The option adapter connects native nullable `none`, `some`, `has_value`,
`value` and `value_or` foundation invocations to their frozen ordinary bodies.
It preserves the W03 input/result subjects, normal successor, exception
successors and ordered checks. `.Value` on `none` retains its invalid-operation
edge; its implementation's unused zero result cannot discharge a normal edge.

The twelve source contexts reuse captured data-stage replay and lifted-data
requests/responses. They contain sixteen invoked definitions and twenty-two
SSA use points. No new compiler response was fabricated. The result relation
compares physical bits, including NaN payloads, signed zero and carrier padding.
The other data families remain explicit pending entries. Outcome helpers for
uninvoked option members are shared, while per-SSA relations cover actual uses.

All twelve certificates pass both checkers on identical bytes with zero axioms,
and both reject hash-corrupted certificates. All twelve runtime contexts now
pass with 177 result observations. The user completed nested-string value-or
in 235.32 test seconds (235.39 seconds wall); its original log and hash-bound
manifest/results are retained under
`../verification-logs/option-data/user-long-completed/`.

The pending list is empty; no rerun is requested. The earlier 45-second timeout
is retained as diagnostic history. No source change or speedup is claimed by
this result import. See `../verification-logs/option-data/verification.json`.

This is a unit-5 component, not a native CFG proof or W09 completion. The T06
whole gate remains deferred to W12; completed checks need no unrelated rerun.
