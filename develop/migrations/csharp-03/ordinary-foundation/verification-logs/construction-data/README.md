# Construction-data scoped checks

The user delegated all test execution, including long runs, to the agent on
2026-09-20. No pending test is handed back to the user.

Selection: new construction SSA adapters, extracted construction emitter,
original source replay/mutations, runtime relation/check behavior, same-byte
Rust/Go checking, affected formatting/lint and consumer inventory. The previous
construction artifacts must remain byte-identical; unrelated semantic suites
are not rerun. The full gate is deferred to T06-W12.

The first candidates run lacked a complete-predicate source and failed its
coverage assertion. A genuine captured dynamic-string source was added.
The initial runtime reached its 300-second evaluator budget in the first fill
result comparison; this was a diagnostic failure, not a semantic verdict.
The first extensional-update probe passed the initializer partition in 79.76s.
A subsequent exact-zero-region optimization and wider corruption/boundary cases
are included in the final run; final receipts supersede intermediate probes.

Final checks pass: the 178-observation pre-freeze run took 199.62 seconds;
only the final freeze change was rerun, with 20 observations in 1.05 seconds.
These are 198 executions including repeats, not 198 distinct final-version cases.
`before-freeze-optimization/` preserves the exact earlier certificate inputs.
`compare-definitions.rs` and `unchanged-definitions.json` verify unchanged
common definition types/bodies modulo term/global numbering; only three freeze
result relation bodies change. This justifies reusing unaffected runtime cases.
All six final certificates pass both checkers in 18.96 seconds. There is no
pending delegated test. Ownership/native control/application proof work remains.
