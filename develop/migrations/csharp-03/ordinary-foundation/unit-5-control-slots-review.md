# Source slot relation review

The generator independently reconstructs W04 control VCs from validated VIR and
W03 data VCs. Each load/store/pattern-bind relation retains its complete source
transfer and checks its original anchor, entry/exit nodes and SSA result. Entry
uses the original parameter mapping. Slot order and every argument role, type,
carrier depth and source identity are included in regenerated metadata.

Reviewed de Bruijn indexing against the full argument order: the final SSA
arguments follow all slot state pairs; indices are reversed once when closing
the ordinary function. A load requires assigned input and bitwise SSA equality,
and frames the target as well as other slots. A store or pattern binding makes
only its target assigned and equal to the result. Other slots preserve their
assignedness and, when assigned, their complete stored value. Unassigned payloads
are unconstrained. The test suite changes each argument separately, checks
inactive storage, and corrupts a high physical bit.

The first implementation used recursive bitwise storage comparison and exceeded
its 300-second evaluator deadline after three source cases. The replacement
reuses the existing construction helper's exact zero-region test and recursive
comparison. This is a complete ordinary definition for arbitrary carrier values;
it does not assume sparse inputs or change the evaluator/checkers. Only the
helper's Rust visibility changes for existing consumers. The previously checked
source preimage was recovered exactly by reversing that visibility change, so
unaffected construction checks were not rerun.

A review clarification added an explicit execution_scope to every relation:
function_entry or successful_transfer. Normal stores must not be used on a
throwing edge. All six certificate byte sequences stayed identical after this
metadata addition; exact regeneration now checks the scope field too.

Six original loop/pattern sources exercise entry, load, store and pattern_bind:
while, for, short_circuit, switch, is_binding and guard_order. The current pinned
runtime/linkage suite passes 1,482 observations in 0.72 seconds. All six identical
certificate byte sequences pass Go and Rust, zero axioms, report agreement and
hash-corruption rejection. Removing source transfers, argument metadata or scope,
altering identities/application status, or corrupting bytes rejects import.
The namespace inventory adds exactly this new consumer (134 to 135).

No remaining defect was found within this scoped slot relation component.
Guard and edge-transport progress is recorded in unit-5-control-edges-review.md.
Non-transfer node frames, node-entry merging, caught-exception entry, exceptional
transfers, native invocation/control sequents and application proofs remain open.
The source's requested termination mode is not proved here. Neither this review
nor the runtime observations complete unit 5 or W09. No component-only commit is
made; the full repository gate remains T06-W12. Current results and source hashes
are in verification-logs/control-slots/verification.json.

The source-slot checkpoint now covers nine source contexts and 130 relations,
with 2048 observations. Private array source snapshots admit partial initialization
under the 4096 profile bound; complete public projections remain separate.
Nullable slot storage preserves Option presence and payload. The three array
certificates, nullable source certificate and Some storage helper passed both
unchanged checkers and hash rejection. See
`verification-logs/control-slots/public-projection/verification.json` and
`verification-logs/control-slots/nullable-representation/verification.json`.
Nullable edge transport, non-transfer memory effects and application proof
assembly remain open; this does not complete W09.


Nullable edge transport now retains the complete Option storage representation
for the `type` source's 53 joins. Its 2,758 observations replace 2,016 previous
observations; the current edge total is 38,901. Sixteen other edge programs are
byte-identical, and the changed certificate passes both checkers and hash
rejection. See `verification-logs/control-edges/nullable-slots/verification.json`
and its review. Nullable slot-to-edge transport is implemented; mapping W04
logical observations, node-entry merging, non-transfer memory effects and
complete application proofs remain pending. The changed fixture contains zero
W04 goal bindings. Unit 5 and W09 remain incomplete.
