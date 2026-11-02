# Complete source-step observer repair

The [exact original executable](../control-source-steps-diagnosis/result.json)
fails after6,339.122 seconds at the original line129 component-count assertion:
actual3, expected2. Its full eighteen-context invocation stops during the first
`count_fill` case. The complete raw stdout/stderr and all original inputs remain
preserved; no passing original result is claimed.

The [failure analysis](failure-analysis.json) checks all eighteen current golden
contexts. The obsolete observer omits three existing components: memory effects
in `count_fill` and `index_update`, and the source value constructor in `type`.
All42 original input members and all three relevant production sources match
the original source320 Git bytes. No producer, fixture, interpreter, kernel or
checker change is needed.

The first repair retained every original component-count summand and extended the exact
component list with these relations. It checks each memory effect's source
entry/exit snapshot and SSA argument bindings, and each constructor's public VIR
function parameter/result signature, anchor, definition identity and operand/
result SSA indices. The [source audit](source-preservation-audit-corrected.json)
reverses only these additions and recovers the entire original source file byte
for byte, including all112 original assertion invocations and every runtime
case, mutation, bit position, strict import and golden comparison. Seven
assertion invocations are added.

The initial [build](build/result.json) fails with E0624 because the test initially
used two crate-private APIs. Its [exact source and drivers](initial-private-api-attempt/input-snapshot.json)
are preserved. The corrected test reads the public constructor VIR function
signature; production visibility is unchanged. The [corrected build](build-corrected/result.json)
passes in34.270 seconds with test optimization1, debug assertions and overflow
checks. [Format](format/result.json) and [target Clippy](clippy/result.json) pass
in3.246 and4.638 seconds. The prior executable that passed both input-echo owners
is [preserved separately](prior-passing-binary-snapshot.json).

The [first complete normal owner](normal/result.json) fails after5,476.090
seconds in the added memory-binding observer: its expected SSA node IDs still
describe use sites rather than original VIR definition points. Its exact source,
binary and raw logs remain preserved. The [SSA follow-up](ssa-origin-followup/README.md)
restores those origins independently, shares the same component observer with
the existing metadata owners, and preserves the full original runtime entry.
Both metadata consumers pass, including all eighteen contexts/582 complete
source steps. The complete eighteen-context runtime retry is running with every
original probe and bound; no passing runtime result is yet claimed.

The [verification record](verification.json) distinguishes terminal build/static
checks from the running full runtime owner. Final changed-input review and the
entire committed repaired T01 gate remain required. W05/T01 is open, T02 is
unstarted, all987 C# application IDs and169 Go demands remain pending, and no
profile is activated.
