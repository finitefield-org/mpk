# Scoped review: lifted nullable native data adapter

This is a unit 5 component review, not W09 acceptance. The implementation and
original-source tests are present; the previously pending long partitions now
pass as recorded below. No internal-unit completion or commit boundary is claimed.

## Source and signature reconstruction

The generator starts from validated original VIR and freshly generated W03 data
VCs. It selects only the `NullableOutcome` lifted family, validates each complete
lifted signature through the existing domain rules, and checks underlying scalar
argument/result/check signatures and failure arity. The option carrier is
reconstructed from the current closed layouts; Some/None arm IDs, tags and
payload shape are checked. All original SSA operation records and roles are
retained. Non-lifted definitions stay in an explicit pending list. Import is
exact regeneration, not acceptance based on caller metadata or digest alone.

## Semantics and laziness

All-present operands select the existing ordinary scalar result. Otherwise
arithmetic returns canonical None. Boolean AND/OR separately detect a present
false/true operand that determines the result even when the other is absent.
Scalar getters return zero for inactive payloads, giving the correct determined
Bool while the option tag records presence. Equal/not-equal distinguish both
absent from one absent; other comparisons return false when either is absent.
Every independent scalar failure is guarded by all-present before evaluation,
so a null operand does not cause a divide-by-zero or overflow from inactive bits.
The shared clause lowering retains ordered failure prefixes and guards.

The result predicate binds the computed carrier once and compares all physical
bits through the existing storage comparator. No host result, source observation
or new axiom becomes an ordinary definition. Native results in the tests come
from the independent `OutcomeModel` oracle only. Whole native bodies and their
universal proofs remain separate required work.

## Earlier verification checkpoint (historical)

The unchanged offline source harness captured seven sources twice with identical
bytes. Initial Docker execution lacked permission for its inner `unshare` sandbox;
adding container-only SYS_ADMIN resolved that environment failure while retaining
network isolation, a read-only repository and the 4 GiB limit. The initial f64
source used an unsupported numeric conversion; typed double literals removed that
unrelated conversion while preserving all six lifted comparison use points. No
source acceptance rule was changed.

All 47 definition occurrences and original SSA points passed exact regeneration
and import mutation checks. Runtime coverage is 35 distinct signatures and 1,998
correct/wrong-result guard observations. Each selected context retains all operand
cases, including None, signed extrema, floating NaNs and signed zeros. Wrong
results flip the final physical bit, checking padding as well as active results.
The production comparator still compares every physical address.

The aggregated checked-integer runtime hit the diagnostic cap; operation selection
was added without reducing cases. No-match selection explicitly fails. Checked
multiply/divide, remainder and decimal equality hit individual diagnostic caps;
similar remaining expensive operations are delegated without repeating diagnostics.
Unchecked multiplication completed in 47.63 seconds after stopping its dispatcher;
its log records a full PASS and 72 observations. The child had already exited when
the stop was attempted; wrapper exit status was not captured. This is not recorded
as a cancellation or a failure.

Bool/f32/f64 same-byte dual-checker suites passed, including changed-hash rejection.
Decimal and i32-unchecked checker suites timed out; full acceptance/rejection suites
remain unproven for those and the larger integer contexts. Clippy, format and five
inventory tests passed. The previous 129-path Std. consumer fingerprint was checked
before adding exactly `ordinary_scalar_relations` (earlier count optimization) and
`ordinary_lifted_data`, yielding 131 paths and aggregate count 4,954.

The precise receipts, source hashes and remaining commands are recorded in
`unit-5-lifted-data-progress.json`. No unrun test is called passed; earlier component
checks do not substitute for full unit 5 or W09 acceptance. The W09 completion audit
continues to require all eight approved units, including control/ownership,
transitions, exact theorem/proof assembly and the predecessor/mutation corpus.

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
