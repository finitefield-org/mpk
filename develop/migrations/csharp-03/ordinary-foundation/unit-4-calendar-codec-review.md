# W09 unit 4 date/time codec review (scoped review complete)

This component provides ordinary parse and format definitions for `date` and
`time`. Registered result identities, source/boundary linkage, universal codec
proofs and the remainder of the original eight-unit W09 plan remain required.

## Direct implementation review

- Date syntax is exactly four year digits, two month digits and two day digits
  with hyphens. Time syntax is exactly two hour/minute/second digits and seven
  fractional digits with colons and a decimal point. Classification uses all
  sixteen bits of each active UTF-16 character. Unlike integer/decimal codecs,
  these fixed syntaxes have no noncanonical or precision-error alternative.
- The outer parser checks the complete unsigned 32-bit input length against
  16,384. Exact spelling length and syntax precede numeric range. A wrong
  length is already a syntax error regardless of later characters, so these
  fixed-length codecs require no full-string fold. Numeric fields and their
  range checks remain lazy when length or syntax rejects.
- Numeric capture uses C8 with four character-index selectors followed by four
  character-bit selectors. Date ignores unused character slots 10..15. Numeric
  digit decoding is demanded only after complete syntax validation. Year,
  month and day ranges use the existing Gregorian month-length circuit;
  hours/minutes/seconds are bounded by 23/59/59. Seven validated fractional
  digits are inherently in 0..9,999,999.
- Date conversion reuses the existing ordinary Gregorian decomposition and
  construction circuits. Their three helper visibility changes do not change
  circuit bodies. Time conversion uses unsigned 64-bit ordinary arithmetic;
  the maximum accepted result is 863,999,999,999 ticks.
- Formatting shares one small character cube before text output selectors.
  Constant-divisor circuits emit the required zero-padded widths. All header
  padding, unused date character slots, and every character index above 15
  evaluate to zero. Formatter inputs retain their normal Date/Time domain
  preconditions; this component does not silently discharge those domains.
- Parse sums explicitly describe success(value) and error(parse_error), with
  zero header/error padding. There is no invented registered result identity.
  No host parser/formatter output is embedded as an ordinary definition or
  accepted as a universal proof.

## Verification in progress

Source tests extend the existing 65-context corpus with two existing captured
DateOnly/TimeOnly source requests from the data-stage replay. They regenerate
definitions, import exact metadata and certificate bytes, reject substitutions,
and pin all nonempty contexts. Semantic tests compare ordinary results with the
independent BoundaryCodec model: leap/common/century month boundaries, year
limits, tick and day boundaries, all active output bits, high inactive text
positions, parse(format(value)), non-ASCII units in every position, and full
unsigned input-length bounds.

Compilation passed without warnings (12.94 seconds). Source generation/import/
mutation passed across all 67 contexts (47.81 seconds), producing two date
programs and one time program. The largest has 77,061 terms and 806 declarations.
The complete existing Gregorian/Guid/DayOfWeek metrics and pinned certificate
bytes remained identical after helper visibility changes (5.28 seconds).
Targeted lint, all five inventory tests (17.75 seconds) and formatting passed.
Exact 67-context/three-certificate pinned replay passed in 48.58 seconds.
The complete semantic matrix passed all 795 cases in 901.46 seconds.
All three same-byte checker cases passed with zero axioms and hash corruption
rejection (1694.257 seconds including build waits). After reviewing these results
and the complete scoped implementation, no additional finding remains for these
definitions. Registry/source linkage and universal codec proofs remain open. No component-only commit,
unit completion, or W09 completion is claimed. `check-fast.sh` remains deferred
to T06-W12.
