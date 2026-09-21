# Deep scalar observation cache correction

Current status: the full user run subsequently passed all eleven runtime
partitions and four checker contexts; see the completion update below.
The following diagnostic narrative is retained chronologically.

The user reported a 600.29-second timeout in lifted checked i32 multiplication.
The handoff stopped at its first operation; later operations/checkers did not run.

The observer previously retained only six selector levels, but this ordinary
circuit uses a C14 state (8,908 gates). Rebuilding weak intermediate closures
therefore lost observed deep leaves. The new cache records a demanded Bool by
its complete selector path and length, up to 18 arguments, in a FIFO capped at
1,024 leaves for each originating lambda. Descendant views share that bounded
table. No closure, suspension or environment is retained by the table. Overflow
beyond either bound only causes ordinary reevaluation. Pending arguments are
never forced to obtain a cache key. Terms, certificate bytes and checker rules
are unchanged.

Direct review checked path/length collision avoidance, independent roots,
FIFO eviction, no reference cycles, no eager evaluation and unchanged semantic
expectations. Regression covers released/rebuilt C5/C14/C18 closures and 2,048
cache insertions. All 20 generic observer tests, eight aggregate/recursive tests,
six source-domain tests and the complete nullable Bool table pass. Clippy and
formatting pass. Exact commands, receipts and source hashes are in
`verification-logs/lifted-data/deep-cache/verification.json`.

Three original checked multiplication pairs pass individually: 0*0 (7.54s),
-1*-1 (9.09s), and int.MinValue*-1 overflow (7.19s). These include correct/wrong
physical result checks and original ordered failure/guard predicates. The
measured overflow process peaked at 43,188,680 bytes physical footprint and
68,403,200 bytes RSS; this is not a whole-suite memory measurement.

A whole-operation diagnostic still exceeded 45 seconds after the correction.
The complete 36-pair operation remains unverified and user-owned, along with
all other pending long checks. Case progress now prints the last started pair;
`MPK_W09_LIFTED_DATA_CASE` supports isolated diagnostics and rejects an empty
selection via the existing observation assertion. The long-check script unsets
that filter, preserving every original case. No tests or carrier bounds were
removed. W09 remains incomplete and the T-wide gate stays at W12.

## Completed user long checks

All eleven pending runtime partitions and four pending Rust/Go checker contexts
passed. Logs were copied from `/tmp/mpk-w09-lifted-long` to
`verification-logs/lifted-data/user-long-completed/`; `verification.json` records
hashes, all operand-pair sets, observation counts and source consistency checks.
No tests were rerun to import these results.

Checked i32 multiplication now completes all 36 pairs (72 correct/wrong-result
observations) in 163.15 seconds, following the previous 600.29-second timeout.
The i64 divide test passes in 516.56 seconds, and all six decimal comparisons
pass in 399.92–495.10 seconds. All four remaining checker contexts pass in
505.21 seconds total, including acceptance of the same bytes with zero axioms
and rejection of corrupted hashes. No full-run memory measurement was recorded.

Together with the earlier short-run evidence, the scoped corpus covers 46
signatures and 2,550 result/guard observations; all seven certificate contexts
are checked. The earlier 35 signatures were not all rerun after the observer
change. These results close this component's pending test list, not the whole
unit 5 or W09. Do not rerun passed partitions without a relevant change.
