# Source-value SSA result relations

The source-value adapter lowers W03 `field_read` and `value_construct` operations
into ordinary definitions. It regenerates the operation signature from the
validated source member table and reuses the unit-3 storage constructor or
projection. A shared computed result is compared with the actual result over
every physical bit; semantic equality would be wrong for NaNs, signed zero,
decimal representations and padding. No public-domain or constructor-execution
obligation is discharged by this relation.

Seven unchanged original-source captures from `../../construction-vc/` yield
ten definition occurrences and twelve SSA use points. All seven metadata and
certificate pairs are pinned here. The enum-only context intentionally has no
source-product operation. Other data families remain explicit pending IDs.
Positive and deliberately broken constructor/default/initializer source contexts
remain distinct even when they share declaration IDs or storage functions.
A successful result-relation test does not prove a broken constructor invariant.

Runtime coverage uses three operand values per SSA point, with the correct
result, an altered first bit and an altered last physical bit. The latter also
covers string padding. Both success relation and guarded goal are evaluated,
with the original success guard. Separate core tests distinguish NaN payloads,
signed zeros and every bit of small physical carriers. Metadata, source context,
SSA successor, relation/value function and certificate substitutions reject.

Current evidence and measured execution times are in
`../unit-5-source-value-data-progress.json` and
`../verification-logs/source-value-data/verification.json`.
Both unchanged checkers accept the identical seven byte sequences with zero
axioms and matching reports/hashes; actual hash-corrupted variants reject.
This component does not close native execution/ownership/control, unit 5 or W09.
The full repository gate remains reserved for T06-W12.
