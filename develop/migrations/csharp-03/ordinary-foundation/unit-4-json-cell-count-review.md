# W09 typed JSON cell-count correction: open review finding

The authoritative value contract is `TOTAL_VALUE_CELLS_MAX = 65_536` and
`validate_value_inner(String) = 1 + utf16.len()` in
`crates/mpk-vc/src/csharp_practical_vir_model.rs`. Boundary input accumulates
those complete typed counts. Its separate raw JSON limit is4*65,536 nodes.
The16,384 bound is the number of UTF-16 units in a string, not the total typed
cell bound. Quoted decimal/date/etc values remain one semantic scalar cell.

The initial ordinary JSON grammar incorrectly capped cumulative typed cells
at16,384, while its string adapter always returned one cell. Earlier tests and
notes repeated those wrong expectations. This could reject legal large compound
values or admit excessive string-containing values. Checker acceptance of those
old certificates proves their typing, not the required cell semantics. The
affected component reviews are reopened; W09 has never been marked complete.

The correction uses the authoritative total constant in both header validity
and child joins. The string adapter reads all32 bits of its stored UTF-16 length,
enforces the16,384-unit bound and returns1+length cells. The unchanged wrapper
rules still exclude only anonymous map entries, Transition events containers
and sum tag metadata. Source products and ordinary sequences retain their cells.

Targeted tests cover120 complete string headers and all stored length bits,
including high/oversized words;99 complete container-join packets;242 grammar
headers at the corrected inclusive total/cursor/depth limits;and complete invalid
packet masking. String-containing Transition, Option and Map/Set expectations
are corrected, including different key lengths and supplementary Unicode.
Their actual original-document execution remains required on the new programs.

All affected typed JSON fixture generators are being replayed into a separate
temporary output tree. Unchanged lexical, scalar codec and source capture
families are not rerun. Completed old vectors will be archived before verified
new bytes are pinned. Only changed canonical bytes require repeated checker
agreement. Old capacity runs are allowed to reach their terminal result; they
are historical observations of the old candidate until their applicable scope
is reconciled with the correction. No running job is restarted on timeout.

This is an actionable semantic finding within W09. It must close before the
affected components or unit4 can be accepted. Full boundary/native/replay
relations, actual propositions/proofs, assembly/mutations and predecessor audit
remain required. No check-fast or equivalent whole gate is run in W09.

The10 affected source replay tests passed (687.69s). All27 source vectors were
regenerated and hash-verified;22 changed and5 stayed byte-identical. Old vectors
are retained in each family's previous-cell-count directory. All45 affected
original-document cases passed with corrected UTF-16 accounting. The new
string-cell helper and changed all-scalar candidate also passed both checkers.
The22 changed source vectors are still being checked; component review remains
open until those terminal results and pending capacity scope are reconciled.

The numeric capacity scope is now documented in `unit-4-json-cell-count-capacity-scope.md`. All 4,387 array/sum declaration names and types match; exactly six grammar blocks differ in each candidate pair. The targeted difference test and existing shared-comparator consumer passed, with lint/format clean. The initial comparison failure was a test namespace assumption (the emitter hex-encodes operation names); it was corrected to an exact six-name expectation. Historical Validation runtime and Map/Set checker jobs have completed and their logs are retained without treating them as current-byte acceptance. The changed-only runner has passed the two changed json-values vectors and continues with json-products; the full semantic finding remains open.
