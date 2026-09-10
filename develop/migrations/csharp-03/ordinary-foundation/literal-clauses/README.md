# Canonical source-contract literals

Rich `literal` recipes use the existing type-directed contract decoder and the
ordinary literal emitter. Raw UTF-16 code units survive canonical parameter
parsing. Values are emitted together so product/sequence parts can be shared
across distinct constants. Existing Bool/integer bodies retain their exact pins.

The exact original C# source/contract request and fresh captured facts are in
`requests.json`, `responses.json` and `capture.json`. Eighteen constants cover
Unit, lone-surrogate char/string, IEEE32/64 bit strings, decimal, Guid, date,
time, duration, instant, DayOfWeek, ParseError, two source products, a bounded
sequence and Nullable None/Some. Canonical float inputs use IEEE hex strings.
The first decimal-text test input was correctly rejected and was corrected.

The source test checks 1,414 independently encoded storage bits: all leaves for
small carriers, and nonzero/neighbor/padding/edge probes for large carriers.
It compares all source and public-default dependency closures with integrated
output and checks exact import and pinned replay. Large string/sequence value
comparisons are not exhaustively evaluated by this literal-specific test.

`certificates.json` identifies the source and integrated candidates. Current
checker results and selected test rationale are in
`../unit-4-literal-clauses-progress.json`. These are concrete literal definitions,
not proofs of source execution, invariant truth, native VCs or boundary codecs.
All remaining internal-unit/W09 completion conditions remain in force.
