# Original calendar and structural archive diagnostics

`result.json` and its raw logs reproduce the source320 calendar failure in the
unchanged original integration binary. `related-structural-archive-result.json`
separately reproduces the old structural test's missing retired counter global.
The original binary is retained at the exact local path/hash in
`original-inputs.json`. The active cumulative diagnostic remains separate.

`definition-diff.json` compares direct declarations; it does not prove semantic
equivalence. `token-closure-diff.json` restricts that inventory to the actual
177 token roots: exactly one retained counter body changes,26 helpers retire,
and every other2,298 retained body and every retained type agrees. The unrelated
calendar-format differences are outside this token closure.

The four structural direct differences span later reviewed domain/count and
aggregate changes. `retained-candidates-result.json` retains all candidate
search outcomes, including rejected comparisons. The unchanged aggregate audit
then passes all four actual pre-count-refresh pairs in
`retained-aggregate-result.json`. Their exact members and recording commits are
in the [sealed epoch manifest](../../../retained-csharp/structural-boundary-epochs/manifest.json).
The failed lookup of the older09a14 commit remains a setup finding; the actual
successor files were recorded by fe9dc4fc.

The eight original compiler inputs and original test-source copies are retained
losslessly as `raw_utf8` in `compiler-source-snapshots.json`, with their original
recorded paths, byte lengths and hashes. Packaging verifies every decoded byte
against its original file before removing the temporary loose copy. Original
command paths in older receipts remain unchanged. Restore those exact strings
to their recorded filenames before replaying the diagnostic compilers; the
packaging itself is not a new execution or checker result.

The [repair verification](../json-calendar-repair/verification.json) owns actual
rebuilt test/lint results. No existing certificate, source capture, production
emitter, checker or limit changes. No Go/C# application proof is discharged and
neither profile is activated. A passing complete repaired T01 gate remains due.
