# Decimal contract adapter final component review

No remaining actionable findings. This completes the decimal codec connection
component of original unit 4, not unit 4 itself or W09.

The cache validates the exact Decimal nominal ID and C9 representation, and
selects each parse/format definition by codec ID, scale and rounding. Shared
digit helpers belong to one builder and are emitted once in either normalized/
fixed order. Existing standalone order is unchanged: 12 decimal format pins
and 17 prior scalar-codec contract pins remain byte-identical. All 292 unique
configuration aliases and their complete dependency closures match the existing
standalone definitions. Exact Result shape and format output-bound conditions
are preserved.

Review corrected one test expectation: fixed parsing strips excess fractional
zeros to fit the 96-bit coefficient, so formatting MAX at scale 28 is accepted.
The corrected expectation and a genuine MAX+1 Range case are independently
checked against the reference parser when generating the requests. The old
assertion failure and historical candidate/capture corpus are retained; their
checker passes never implied that the failing source assertion was true.

Seven affected fixed-context runtime conditions passed in 1393.52 seconds,
including the corrected maximum case and genuine Range rejection. Two unchanged
normalized conditions reuse the completed normalized context from the earlier
run: its certificate and metadata are exactly identical to the current program.
That earlier overall test failed on the separate incorrect fixed expectation;
it is not reported as a passing whole test. The final run also passed all 177
attachment/292 alias checks, both helper-emission orders, exact program imports
and result-metadata mutations.

All five changed fixed candidates passed identical-byte Rust/Go checking with
zero axioms, agreeing reports and actual hash-corruption rejection (627.890s).
The normalized candidate retains its earlier checker result on identical bytes.
Final runtime outputs, generator outputs, six pinned candidates, metadata,
capture receipts/raw responses and all recorded log hashes were reconciled.
Scoped lint/format/inventory passed; unchanged standalone arithmetic matrices
were not repeated. check-fast.sh remains deferred to T06-W12. Native method
proofs, reconstruction witnesses and original units 3-8 remain open; no
component commit/push or W09 completion receipt is issued.
