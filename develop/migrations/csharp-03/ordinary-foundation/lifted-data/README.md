# Native lifted nullable data relations (W09 unit 5 component)

Seven captured C# sources generate 47 definition occurrences and 47 original
SSA use points (46 distinct lifted signatures). The ordinary adapter preserves
W03 source definitions, subject order, normal successors, independent failures,
frozen failure priority and success/exception predicates. Import regenerates
both exact metadata and canonical certificate bytes from the original VIR.
Other data families remain explicitly pending; these predicates are not native
body, control, ownership or application proofs.

Null propagation precedes scalar failure demand. Nullable Bool implements the
full three-valued AND/OR tables; equality handles both-absent and one-absent
operands, and ordered comparisons with absent operands return false. Arithmetic
wraps an active scalar result in Some and otherwise emits canonical None.
Result equality observes the complete physical representation, including unused
storage. Existing scalar definitions provide the arithmetic and exception tests.

Candidate/import/mutation checks passed for all sources. Prior short runs
cover 35 signatures and 1,998 observations. The user completed all eleven
remaining runtime groups (552 additional observations), including every original
operand pair after the bounded deep-cache correction. This yields 46 signatures
and 2,550 observations across the scoped corpus; it does not mean all prior short
runs were repeated after that correction. All seven certificates pass same-byte
Rust/Go agreement, zero-axiom checks and hash-corruption rejection.

The formerly timing-out checked i32 multiplication passes in 163.15 seconds;
the four remaining checker contexts pass in 505.21 seconds total. The logs and
source/byte consistency checks are retained in
`../verification-logs/lifted-data/user-long-completed/verification.json`.
`run-long-checks.sh` remains a reproducible command for relevant future changes;
no checks in that script are currently pending. Do not rerun them unchanged.

For isolated diagnostics, `MPK_W09_LIFTED_DATA_OPERATION` selects an exact
signature, `MPK_W09_LIFTED_DATA_CONTEXT` a captured source, and
`MPK_W09_LIFTED_DATA_CASE` an operand-index pair. No-match selection fails.
The complete script clears the case filter to retain every original case.

All receipts, coverage limits and scoped reviews are in
`../unit-5-lifted-data-progress.json` and `../unit-5-lifted-data-review.md`.
No original internal unit or W09 is complete; the full T06 gate stays at W12.
