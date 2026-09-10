# Decimal contract adapter direct review

The adapter validates exact nominal argument/result signatures, ordered checks
and predicate counts. Existing decimal result, success and ordered exception
definitions are preserved. Arithmetic contract checks project the independent
FIT-state zero/overflow flags before exception precedence. Rounding and
conversions have at most one check and reuse that independent predicate. The
source compiler registers these predicates with the corresponding ContractFails
name. The quantifier cache still selects unchanged check-free integer helpers.

All 45 frozen decimal signatures, their existing metadata, 14 certificate pins,
eight overlap observations and 135 signature/check corruptions passed. Four
affected quantifier tests and 22 integer/fixed-codec/floating source certificate
pins passed. The decimal helper-name comparison admits only internal shared
DecimalArithmetic/DecimalSteps names and recursively compares both declarations,
including types, values, levels and dependencies. Stable public names remain
exact. A mutated helper below the add root rejects. The existing strict floating
closure/runtime test passed after this test-comparator refactor (60.34 seconds).

There are 44 native-syntax operation contexts within nine grouped source inputs.
Internal value_equality remains covered by the 45-operation adapter check; the
non-admitted x.Equals(y) API was removed from the public-source fixture. Char
literals use a single UTF-16 code unit, not a JSON integer. Source v5 completed
add overflow and comparisons before the old char fixture failed; only those
completed, unchanged byte contexts may be reused. No overall pass is attributed
to v5. Source v4 was intentionally stopped after a test mutation-selection fix.

All nine final candidates passed (55.54s), covering 44 aliases/dependency
closures and 49 attachments. The final v7 run passed nine selected runtime
conditions/four failures in 1365.79 seconds. The completed v5 add overflow case
is byte/metadata-identical, yielding ten combined conditions/five failures.
All nine same-byte Rust/Go cases passed (883.971s), with zero axioms, report
agreement and hash corruption rejection. Terminal results, runtime/candidate
bytes, metadata, hashes/sizes, capture receipt and retained logs were reconciled.
Final direct component review found no actionable findings.

The test-only observer optimization is a separate ongoing experiment; these
observations use the previously verified evaluator. Native method bodies and
application VC proofs remain outstanding. Original units 3-8 and W09 stay open;
the full gate is deferred to T06-W12, and no component commit/push is made.
