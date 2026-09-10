# Hexadecimal codec review — W09 unit 4 remains open

Reviewed ordinary implementations against section 10.2 and the frozen
`BoundaryCodec::parse`/`format` definitions. Production uses explicit ordinary
Boolean equations, character tables and concrete selectors; the earlier host
model is used only as the independent test oracle and for static codec/type
configuration validation.

- Parse all 16 character bits, including non-ASCII high bits. Checking only
  the low byte would accept U+0130 as ASCII `0`; per-position mutations include
  that counterexample, isolated surrogates and U+FFFF.
- Use the full 32-bit unsigned length. Input-bound errors precede syntax;
  syntax precedes uppercase noncanonical spelling. Wrong GUID separators and
  other non-hex characters cannot be hidden by an uppercase character elsewhere.
- Preserve hexadecimal significance across all scalar bits and use the exact
  GUID N/D punctuation positions. Test review added asymmetric GUID halves so
  swapping equal repeated halves cannot escape the oracle. Float cases retain
  sign bits, infinity, subnormal boundaries and distinct NaN bit patterns.
- Parser output selects either the complete scalar or the error payload and
  zero padding. Private digit/value helpers have no independent acceptance
  authority. Error cases observe the entire result cube, including inactive
  payload and tag padding. Scalar metadata follows the ordinary sum layout's
  positional `0` field identifier; it does not invent a nominal result ID.
- Formatter output length is fixed and non-null; all inactive cells and header
  padding clear. Review added explicit nonzero header-padding addresses.
- Generation checks the exact reconstructed scalar shape/depth and frozen codec
  configuration. Import regenerates metadata and certificate bytes and rejects
  every altered metadata field and corrupted certificate.
- The first test compile called the oracle formatter without its validated
  foundation/root/closed context. The test now retains that captured context
  for each representative codec and invokes the existing full formatter API.
  No production meaning or test expectation was weakened.

Generation, actual ordinary evaluation and both checkers are distinct evidence.
Current terminal results and live jobs are in `unit-4-hex-codec-progress.json`.
The source suite subsequently passed all 39 format/parse comparisons and 680
parse/error cases. Both unchanged checkers passed all eleven fixed programs
with zero axioms and rejected their hash-corrupted variants. A subsequent
64-context pinned replay confirmed byte/metadata preservation after making
three shared ordinary term helpers visible to the sibling integer formatter.
Direct rereview found no additional actionable issue in this bounded hex codec
component. The original source-suite executable predates only that visibility
change and the separate pinned-replay test; no semantic definition changed.

This component review cannot close unit 4 or W09: other codec families, strict
JSON document relations, native/source semantics and universal proofs remain
required by the original eight-unit plan.
