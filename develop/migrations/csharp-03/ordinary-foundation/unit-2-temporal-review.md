# W09 unit 2 temporal component: direct review

Scope: the Time/Duration/Instant increment, shared circuit emitter, source-bound
import/export, test-only core interpreter, fixtures and progress records.
This is a component review, not completion of unit 2 or W09.

The direct review checked constant-divisor no-borrow handling, i64 minimum
magnitude, signed remainder and truncation, Euclidean day wrapping, 65/79-bit
intermediates before range checks, result zeroing and exclusive ordered errors.
It checked exact type/check-table reconstruction and rejection of substituted
source, foundation, operation metadata and certificate bytes. It also checked
that the integer alias and shared emitter preserve the existing seven golden
certificates, with no C# rule or acceptance change in either checker.

Test review corrected an Instant capture without an operation mapping by using
the original mapped sidecar captures. The expanded core-evaluation run exposed
runtime and default-stack limitations in the test interpreter; per-closure Bool
memoization and dedicated 32 MiB test stacks resolved them. Cache entries are
scoped to the exact captured environment. The final 12-test carrier-library run
checks old helper/integer behavior and the new temporal semantics, including
simultaneous precision and range failures in actual emitted core terms.

Final findings for this component: 0. Targeted results and fixture hashes are in
`unit-2-temporal-verification.json`. The W09 exit gate remains unmet; the full
T06 gate remains deferred to T06-W12.
