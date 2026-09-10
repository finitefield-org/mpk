# Numeric capacity evidence after the typed cell correction

This is a regression scope analysis, not a proof of W09 or an assertion that
historical bytes are current bytes. The original Validation and array capacity
runs have passed. Neither was restarted solely
because the typed cell upper bound changed.

The new certificate comparison decodes each current and `previous-cell-count`
array/sum certificate, validates both structures, compares every declaration
name and type, and compares every direct definition body. It also requires full
transitive equality to fail, so it cannot mistake these candidates for identical
programs. The 2,092 array and 2,295 sum declarations retain their names/types.
Exactly six generated circuit blocks differ in each pair: `Child` blocks
4/8/14/15, `Syntax` block 3 and `FinishPacket` block 3. The test pins those exact
names, including the circuit emitter's hex-encoded operation namespace.
All other definition bodies, including lexer, scalar parsing, sequence stepping,
role capacity, active storage and sum arms, remain structurally identical.

The corresponding generator changes are the comparisons against the cumulative
typed cell maximum in `json_grammar::header_valid` and `emit_child`: 16,384 becomes
the authoritative 65,536. Both continue requiring positive counts and valid
cursors/reserved bits. The already completed corrected grammar tests exercise
complete headers, cumulative boundaries, invalid inputs and packet masking.

The array documents contain only Bool values and request lengths 4,095/4,096/4,097.
Even the attempted over-capacity case has at most 4,098 semantic cells. Validation
documents contain i32/Bool payloads, with error lengths 0/1/256/257 and at most
259 attempted cells. They contain no semantic String value; textual sum tags
have no semantic cells. All intermediate counts fit both old and new cell bounds.
The role capacity and depth rejection paths are unchanged. Thus the upper-bound
expansion does not change the behavior tested by these specific numeric runs.
This inference is limited to their inputs and selected storage observations;
it is not a universal equivalence theorem and does not cover String consumers.

The original Validation run passed ten document cases in 3,989.47 seconds.
Its retained log is `verification-logs/json-cell-count/` and its historical status
is recorded in `unit-4-json-sum-roles-progress.json`. The original array run
passed on 2026-09-11 after 44,333.77 seconds. All three documents completed:
4,095/4,096 elements accepted and 4,097 rejected, with complete header/length
and selected storage observations. The retained log and recorded historical pin
are in `unit-4-json-sequences-progress.json`. PID 68689 was absent after the
terminal success log. Current and archived array hashes still match the exact
six-definition comparison. The current array checker PASS and its retained log
hash were separately revalidated; the changed array/sum checker queue has already
completed. Historical runtime/checker results do not become current-byte runs.
The 45 affected String-containing document cases were rerun on corrected bytes
and passed, as recorded in `unit-4-json-cell-count-progress.json`.

Test selection for this follow-up: run the exact numeric certificate difference
test and an existing transitive-definition comparison consumer because its shared
comparator was factored into direct-body/type modes. Run affected integration lint
and scoped formatting. Preserve completed unaffected runtime results; wait for
existing live jobs. Do not run `check-fast.sh` or a split whole-repository gate.
The T-wide gate remains at T06-W12. Units 3-8 and W09 remain incomplete.
