# W09 string contract connections

Six independently captured source contexts cover nullable and non-null basic,
construction and ordinal strings: 21 unary/binary operation kinds and 67
contract attachments. Candidate replay preserves the complete existing operation
definition closures and typed linkage; helper/metadata mutations reject.

Large literal selection now uses a balanced address tree, fixing the Rust
checker stack overflow found at 16,384 UTF-16 units. All 32,768 targeted address,
padding and inactive-bit observations pass. The final six candidates total
576,246 bytes (maximum 137,380 bytes). All 67 source observations, including
18 undefined cases, are covered by current or identical-byte retained results.
Both unchanged checkers accept all six identical-byte candidates with zero
axioms and matching reports/hashes; actual hash corruptions reject. Only the
two changed construction contexts were rerun after the balanced-selector fix.
See `../unit-4-string-clauses-progress.json` for logs, exact reuse and prior failures.

These are helper definitions and observations, not application/native-body
proofs or an original W09 unit completion receipt. Larger native arities remain
outside unary/binary contract tags. Units 3-8 remain open; the full gate is
reserved for T06-W12.
