# W09 unit 4 decimal parser checkpoint review

This component implements ordinary parsing definitions for normalized decimal
and all 29 fixed scales with five rounding configurations. It does not complete
unit 4, source/registry linkage, universal parse/format proofs, or W09.

## Direct implementation review

- The outer decision reads the full unsigned 32-bit text length and rejects
  lengths greater than 16,384 before demanding syntax or numeric state. Syntax
  precedes noncanonical spelling, which precedes precision and range.
- The first nonzero word fold returns the first decimal point's index plus one.
  The all-character predicate permits a point only at that index, so subsequent
  points reject. Sign characters are permitted only at index zero. Both point
  sides must contain digits; empty input is explicitly rejected independently
  of inactive character storage. Character classification uses all 16 bits.
- Noncanonical checks cover leading plus, redundant integer zeroes, negative
  zero, and normalized trailing fractional zeroes. Fixed parsing retains a
  representable input's original scale and coefficient. Fixed rounding mode is
  validated and carried in metadata but does not change parsing behavior.
- Numeric accumulation is deliberately behind the ordered decisions. Any
  accepted spelling has at most 59 characters: sign, 29 integer digits, point,
  and at most 28 fractional digits. A 192-bit accumulator covers every possible
  accepted coefficient before reduction. Overflow remains sticky, even when
  the wrapped coefficient has zero high bits. A coefficient that has overflowed
  192 bits cannot fit 96 bits after at most 28 decimal zero removals.
- The 28 ordinary trim steps divide only an excessive coefficient with positive
  scale and zero remainder. Each step stops changing the value as soon as it
  fits 96 bits or cannot lose another fractional zero. Division uses the usual
  0..9 remainder invariant and a five-bit 0..19 partial dividend.
- The internal sum shape explicitly describes success(decimal) and
  error(parse_error); it invents no registered nominal result identity. The
  result header, decimal product padding, and error payload padding are zero.
- Reviewed selector environments for 14-bit character indices, 5-bit word
  selectors, the 60 accumulation lets, and the five parser lets. Input text,
  point, part lengths, state, mode and target are shared before result selectors.
  All generated terms retain the ordinary checker path; no host-parsed value
  supplies production coefficients or a proof.

## Verification

New tests regenerate all 65 captured contexts, check 146 configurations in
each of six decimal contexts, and reject exact metadata, certificate and
cross-context substitutions. Semantic tests compare all 1,024 result bits
against the independent BoundaryCodec parser. Cases cover syntax conflicts,
canonicality, fixed scale mismatches, precision/range precedence, 96-bit limits,
the exact 192-bit wrap, and full-length late errors. Source generation/import/mutation passed for all 65 contexts (66.94 seconds),
producing six pinned programs with 146 definitions each. Each certificate has
44,720 terms and 643 declarations. Compilation and targeted lint passed without
warnings, all five inventory tests passed (18.60 seconds), and Rust/Go formatting
checks passed. Exact 65-context/six-certificate pin replay passed in 68.17 seconds.
All six same-byte checker cases passed with zero axioms and hash mutation
rejection (333.820 seconds). All four full-input-bound cases passed in 2039.92 seconds. The configuration
and short-input semantic job passed all 446 observations in 14167.71 seconds.
With these results, direct review found no remaining issue in the decimal parser
component; universal parse/format relations and unit 4 remain incomplete.

No component-only commit is authorized by the eight-unit plan. The full
`check-fast.sh` gate remains deferred to T06-W12.
