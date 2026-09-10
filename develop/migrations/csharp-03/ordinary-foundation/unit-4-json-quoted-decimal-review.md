# Quoted decimal JSON token component

This implements one part of W09 unit 4. It does not complete typed JSON
schemas, source/native/transition relations, application propositions/proofs,
or W09 acceptance.

The wrapper calls the existing canonical string frame with the original C24
document, absolute u32 start and ending. It binds that frame once, parses the
decoded C19 UTF-16 text with the exact decimal codec, and requires frame
validity, the codec success tag, and an ending in 0..3. Thus a failed frame,
noncanonical decimal, precision/range error or field-name colon cannot produce
a valid value. The delimiter is checked without consumption.

The C10 output contains valid/whole flags, all 512 bits of the C9 decimal
carrier, the absolute u32 end and zero padding. Invalid output is entirely
zero. The sign, scale and all 96 coefficient bits retain their original
carrier positions inside that payload. One finish circuit is shared by all
146 normalized/fixed-scale/rounding configurations; the original configuration
metadata and parser names remain explicit.

Decimal emission now separates its unchanged core emission from configuration
declarations and reuses complete existing declarations. A unit test compares
the resulting standalone bytes against the prior dual-checked certificate and
checks that re-emission neither changes metadata nor increments transformer
counts. The JSON/collection/decimal integration test also reuses the exact
already emitted configurations at an unchanged cumulative count of 15,453.

Direct source review checked de Bruijn indices under the frame let, closed-sum
success and payload projection, complete C9 payload copying, C10 padding,
full-width ending validation, metadata linkage and cumulative budgets. No
actionable source findings remain in this component. Runtime, actual-source
and final certificate verification status is tracked separately in
`unit-4-json-quoted-decimal-progress.json`; source review is not evidence that
pending checks passed.

Tests are restricted to new quoted decimal composition, its affected source
and structural consumers, and changed certificate vectors. Existing decimal
semantics/checker results remain applicable because standalone bytes are
identical. Complete dependency comparison protects older JSON token behavior;
unrelated semantic matrices are not rerun. W09's full gate remains deferred
to T06-W12, with no component-only commit or push.

All 146 closed-setting runtime samples passed in 12839.31s; the original
process handle returned exit 0. The complete log is retained in the progress
receipt. This closes the pending setting matrix without rerunning it.
