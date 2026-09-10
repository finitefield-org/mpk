# Canonical integer parsing — partial W09 unit 4

The ordinary parsers implement the eight signed/unsigned integer codecs plus
`duration_ticks` and `unix_milliseconds`. Their input is bounded non-null C19
UTF-16 text; output is the structural sum of the exact scalar and parse_error.
The metadata records both arms and their field references. Linking this shape
to each registered concrete result instance and proving the source boundary
relations remain required; these helper certificates do not supply those proofs.

The full unsigned 32-bit length is checked first. More than 16,384 code units
returns InputBound before syntax or storage traversal. For bounded inputs, the
ordinary All.D14 fold checks every active full 16-bit character: an optional
leading sign followed by ASCII digits. Empty text, a lone sign and a negative
unsigned value are Syntax. Syntax precedes Noncanonical (plus, leading zeros,
negative zero), which precedes Range. A late invalid character must therefore
beat earlier redundant zeros or excessive numeric length.

After syntax and canonicality, more than twenty body digits is out of range.
Twenty shared stages form the unsigned magnitude with shift/add multiplication
by ten. A sticky carry catches both shifted-off bits and addition overflow;
width-specific limits include the extra negative magnitude for signed minima.
The value uses original-width two's-complement negation. Error payloads contain
only the ordered error code, without partially accumulated value bits. All sum
header and payload padding is zero. Arithmetic state and the decision are
let-bound outside result selectors. No host parser defines the ordinary result.

Generation examines 65 captured source contexts, pinning 54 nonempty programs
with 86 definitions across ten codec IDs. The largest program has 14,783 terms
and 267 declarations. Import regenerates exact metadata and certificate bytes,
rejecting source/foundation/hash/definition mutations and cross-context swaps.

The independent BoundaryCodec oracle is compared against every ordinary result
bit for 456 short/boundary cases, including unsigned-64 overflow, signed minima,
conflicting errors, Unicode lookalikes, lone surrogates and high length bits.
That suite passed in 249.95 seconds. A separate full-capacity test exercises
16,384 digits, 16,384 zeros, an invalid final character after zeros, and input
bound precedence. All four cases passed in 588.42 seconds, independently of
the short suite. Pinned replay and same-byte dual-checker tests are likewise
separate. Checker coverage is complete across 53 clean original cases and one
targeted recheck after correcting a stderr-warning protocol failure. The original
full run is retained as failed, not relabeled as passed. Exact source/pin replay,
lint and inventory passed. Results and their precise scopes are in
`../unit-4-integer-parse-progress.json`.

Universal round-trip proofs, registered result linkage, remaining codecs,
strict whole-document JSON and all remaining original W09 units stay open.
The full repository gate remains deferred to T06-W12.
