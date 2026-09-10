# Normalized decimal formatting — W09 unit 4 in progress

This component converts the explicit sign/scale/96-bit-coefficient decimal
carrier to canonical normalized decimal text with ordinary core definitions.
It does not implement fixed-scale rounding or parsing; those remain required
parts of the original codec and proof scope.

A shared 96-bit division-by-ten circuit produces quotient and remainder.
Twenty-nine let-bound stages cover all 96-bit coefficients. The reversed digit
cube and digit count are shared across output selectors. Twenty-eight trailing
zero steps remove digits only while the original scale permits it, stopping
permanently at the first nonzero digit. The final scale and digit count determine
integer width and decimal-point position; explicit leading zeros are read from
the high zero quotient digits. Zero emits `0` independent of its stored sign
or scale. Nonzero negative values emit a leading minus. Input scale 0..28 and
coefficient width are the existing decimal domain preconditions; this formatter
is not a domain validator.

The input layout remains the unit-1 product: two field selectors followed by
seven child selectors, with the short sign and scale fields padded. Output is
C19 text with full 32-bit length, zero header padding and inactive characters.
For valid decimals, at most 31 code units are required. No host decimal result
or formatting string becomes an ordinary definition.

The actual-source suite regenerates from the same 65 captured contexts used by
the integer codecs. Its semantic phase uses the captured decimal-literal source
and an independent BoundaryCodec oracle. Cases cover every scale, coefficient
96-bit limits, all-removable zero tails, fractional leading zeros, negative
values, signed/scaled zero and poisoned unused input padding. It reads every
active output bit and tests output padding/high inactive index bits.

Compilation, generation across 65 contexts, six pinned same-byte dual-checker
cases (zero axioms, hash corruptions rejected), exact pinned replay, current
lint and inventory have passed. The largest program has 26,011 terms and 236
declarations. Semantic execution remains in progress; see
`../unit-4-decimal-format-progress.json` for results and the live handle.
The finite semantic matrix and component review are not yet complete. Fixed-decimal rounding, parsing, registered result
linkage, JSON, native/control/transition semantics and universal proofs remain
open under the original eight-unit W09 plan. check-fast.sh stays at T06-W12.
