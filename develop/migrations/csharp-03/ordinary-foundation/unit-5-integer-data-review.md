# W09 integer data relations review (component in progress)

The component reconstructs W03 data definitions and operations from validated
VIR, requires exact frozen integer/Boolean signatures, and compares every
physical result bit. Integer/char representations have no ignored padding;
Boolean equality is expressed by existing Boolean elimination. Existing scalar
bodies and their error order remain unchanged. Zero and signed min/-1 failures
are disjoint, so their frozen ordered predicates serve as individual failures.

The original W03 success, failure and prefix terms are lowered directly, with
full SSA subject order, source function/node IDs, exception metadata and normal
successor IDs retained. Every other data definition stays explicitly pending.
These definitions do not prove a native CFG, source call or application VC.

Review fixed name construction: raw data/SSA IDs can contain a numeric-start
name component, which canonical certificate decoding correctly rejected.
Core names now hash complete typed identities under an H-prefixed component;
original identities remain available verbatim in metadata. Initial failed
source/candidate runs are retained in the progress receipt. A native source
regeneration is required before concluding this component passed.

The result-bit unit test passed 1,068 cases. Its first fixture incorrectly used
unpromoted i8/i16 arithmetic; it now follows the frozen promotion rules and uses
conversions to exercise small integer and char result widths. Source use-point,
mutations, inventory and identical-byte checker verification remain pending.


The first native runtime assertion exposed the W03 free-index convention:
subjects[i] differs from the contract compiler's de Bruijn index below subject
lambdas. Direct reuse had reversed same-typed operands/results and rejected
mixed integer/Boolean signatures. The adapter now maps each free index to
subjects.len()-1-index, validates the original type and rejects unsupported
local-binder forms. The existing native add-result and mixed-type cases are the
regressions for this finding; reruns are pending. Structural equality appearing
in the Boolean source stays explicitly pending rather than being relabeled as
an integer primitive. Its pending family is now checked in the fixture.


Final component verification passed. Eight original use points passed 80
result/guard cases in 350.67 seconds; all three runtime certificate/metadata
pairs equal the already accepted checker pins. Source metadata and byte
mutations reject. All three same-byte Rust/Go checker cases passed with zero
axioms and matching reports/hashes, and corruptions reject. The result-comparison
and point-binding helpers were then shared with structural data; regenerated
integer certificates/metadata stayed exactly identical (6.31 seconds), so the
unchanged arithmetic runtime tests were not repeated. Inventory and scoped
lint/format checks pass. No actionable finding remains in this integer-data
component. Original W09 units 3-8 remain open; no native CFG, source invocation
or application proof is supplied by these definitions.
