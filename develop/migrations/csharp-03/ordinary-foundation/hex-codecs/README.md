# Ordinary hexadecimal codecs — partial W09 unit 4

These definitions implement the frozen `binary32`, `binary64`, `guid.n` and
`guid.d` parse/format relations from source-design section 10.2. They accept
the existing non-null C19 UTF-16 text carrier and return the concrete ordinary
result layout: tag 0 with the exact scalar bits, or tag 1 with `parse_error`.
This structural result description has no invented closed-instance ID. Later
VC assembly must link it to the exact registered concrete result instance.

Both float codecs preserve every input bit: no numeric conversion, NaN
normalization, CLR call or host-computed arithmetic result supplies a definition.
GUID encoding follows the frozen 128-bit hexadecimal value, with hyphens at
positions 8, 13, 18 and 23 in D form. It does not use CLR mixed-endian Guid byte
storage. Character classification examines all 16 UTF-16 bits. Parsing checks
the complete unsigned 32-bit length and enforces this error precedence:
input length above 16,384, then wrong length/punctuation/non-hex syntax, then
uppercase noncanonical spelling. The failed result contains only its error arm.
No partially decoded value is exposed by the parser result.

Formatting emits exactly 8, 16, 32 or 36 lowercase ASCII code units. The C19
header's unused bits and all inactive character cells are zero. These fixed
lengths are below the frozen text output bound. Input representation/public
domains, source/native commutation and universal round-trip theorems are still
independent obligations; finite observations do not discharge them.

The generator examines the concrete carriers reconstructed from validated VIR
and emits both relevant GUID codecs or the relevant float codec for each
reachable scalar type. Unknown metadata/schema/hash/context changes reject on
canonical regeneration. It does not accept arbitrary codec configuration, source
paths, an ambient formatter or a caller-supplied implementation.

64 existing actual-source contexts were examined. Eleven contain these scalar
types and pin 14 codec occurrences. The largest program has 3,119 terms and
93 declarations. The independent parser/formatter oracle is the earlier frozen
`BoundaryCodec` model and uses the captured context's validated root/closed set.
Cases include signed zeros, infinities, subnormal/normal boundaries, NaN payloads,
asymmetric GUID fields, every character position's invalid Unicode/punctuation,
uppercase combined with syntax errors, 16,384/16,385 input lengths and high
32-bit length headers. Every parse result bit is observed. Formatter checks
cover all active character bits, header padding, every capacity index's low
character bit and all bits in the final inactive cell.

The source test, pinned replay and
`TestCheckerAgreementWithRustCLIHexCodecs` have separate actual run records in
`../unit-4-hex-codec-progress.json`. The checker test requires the
`checkeragreement` build tag and checks all eleven identical byte sequences,
zero axioms and hash-corruption rejection. A running test is not counted as a
pass. The source suite passed in 785.35 seconds with 39 format/parse comparisons
and 680 parse/error cases. All eleven same-byte checker cases passed in 183.927
seconds. A separate 64-context pinned regeneration passed in 7.40 seconds after
the shared core helpers' sibling visibility changed; every certificate and
metadata record remained identical. Scoped lint, inventory and format passed.

Integer, decimal, calendar/time, duration/instant and whole-document JSON
relations remain required, as do all other incomplete W09 units and proof gates.
The repository-wide gate stays deferred to T06-W12.
