# Construction guard extension review

Requested definitions are restricted to validated construction instance IDs and
matching Foundation/SequenceOwnership operations. Existing construction bodies
provide negative-length and index-range predicates, including the physical
capacity bound of 16,384. Unknown definitions remain pending. The original guard
tree and ordered exception conditions are compiled without replacing their
failure order.

Ownership failure aliases are source scoped. The generator matches the data
operation by original function and node, then retains the exact receiver and
state binding, flow theorem and receiver theorem. A source operation's helper is
emitted once even if multiple edges need it. Its generic failure alias is added
only while lowering that edge and removed afterward, including pending guards.
Distinct nodes using the same construction operation cannot reuse another
node's interpretation. Checked flow/receiver proof dependencies remain inside
the certificate. This does not prove native execution or application scopes.

Reused construction helpers changed only from private to pub(super). Reversing
those two visibility changes reproduced their predecessor source exactly.
Construction and slot evidence records retain that preimage and were not rerun.
All eight unaffected edge sources regenerate to exactly the same certificates
and metadata in 0.90 seconds. Their 9,919 existing runtime observations and
same-byte checker acceptances remain applicable. No new Std consumer path or
checker change requires another inventory or core test run.

The changed index_update source passes 3,991 observations, including lengths and
indices at zero, final valid entry, first invalid entry, negative values and
16,384 capacity. Complete slot transport checks remain active. Four ownership
bindings are compared with original functions, nodes, receivers, states and
proof identifiers. Mutating each binding field, removing proof/definition tables
or changing certificate bytes rejects import. All ten previously pending guards
are now generated. The original 24 W04 goal binding identities remain covered.

The initial construction-only runtime passed in 53.79 seconds. A final combined
run also passed that subtest and regenerated identical bytes after restricting
definition selection to construction IDs. The suite exited 101 because its
separate missing-string regression initially selected a source without loops or
patterns, so W04 generated no control edges. Both attempts are retained as failed
suite records. The fixture was corrected to the captured foreach_string loop,
with an explicit nonempty-function assertion. That regression passes in 0.09
seconds, retaining five unsupported guards and rejecting their removal. Only
the failed fixture was changed; completed array observations were not repeated
again. See runtime-subtest.json and control-edge-string-pending-final.json.

The current 675,025-byte index_update certificate passes unchanged Go and Rust
checkers with zero axioms and matching reports. Go positive/hash checks took
167.557/156.200 seconds; Rust took 0.413/0.384 seconds. Promoted metadata and
certificate files exactly match both runtime candidates and checker input.
The former index_update checkpoint is retained under
../before-construction-guards/. Root-level receipts for other eight sources
remain unchanged; ../verification.json selects the current evidence per source.

No remaining finding was found within this extension. Other family guards,
node-entry merging, non-transfer frames, native execution, exception entry and
application proof assembly remain open. The component does not close unit 5 or
W09, and no component-only commit is made. The full gate remains T06-W12.
