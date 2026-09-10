# W09 unit 4 fixed decimal formatting review (in progress)

This component covers all five fixed-decimal rounding modes and all 29 valid
scales. It retains the remaining codec parsing, exact registered linkage,
universal source/codec proofs and original W09 scope.

## Direct review

- Reviewed reuse of ordinary decimal.round.Mode.2. Every exported configuration
  fixes a target in 0..28; no exceptional rounding result is assumed for an
  arbitrary target. The existing 96-bit rounder carries the most significant
  discarded digit, sticky tail and retained parity for all specified modes.
- Scale reduction can increase the divided coefficient by at most one; this
  cannot overflow 96 bits because division already reduced it by at least ten.
  Scale extension changes text only, avoiding coefficient overflow even for
  the maximum decimal followed by 28 display zeros.
- Checked the distinction between rounded actual scale and configured display
  scale. Position arithmetic uses actual scale for coefficient digits and emits
  literal zero only beyond the last real fractional digit. The point is present
  exactly when display scale is positive; negative numeric zero suppresses sign.
- Counted all lets and cube positions: after rounded value, 29 pairs, digit cube
  and count, indices are rounded=31/target=32/count=0. After layout and text
  selectors, layout=19 and digits=21. All field widths stay monomorphic and the
  caller-facing functions take one concrete decimal argument.
- Checked output limits and padding. Full length is at most 59 for a valid
  decimal/configuration; the 14-bit character index guards every inactive cell.
  Before-point high quotient digits provide leading integer zero, while a
  wrapped reverse index beyond the fractional digits is masked by ASCII-zero
  selection. Header and inactive bits are cleared.
- The shared digit-helper extraction preserves normalized certificates exactly,
  demonstrated by all 65-context/six-pin replay checks (12.72 seconds). Production
  static BoundaryCodec construction validates configuration only; expected
  numeric text is supplied by the independent oracle exclusively in tests.

## Verification

Compilation and 65-source exact import/mutation generation completed. Six source
programs pin 870 configuration occurrences (145 each); maxima are 34,535 terms
and 518 declarations. Initial lint and all five inventory tests passed.
All six same-byte checker cases passed with zero axioms and hash corruptions
rejected (286.362 seconds); final pin replay passed (11.20 seconds) and latest
lint passed. The ongoing semantic matrix is tracked separately in
unit-4-decimal-fixed-format-progress.json and is not counted as a pass. The semantic matrix covers every mode/scale and additional
rounding ties, parity, sticky tails, sign, maximum values and zero/cohort cases.

No additional actionable code issue was found in this direct review. Final
component review remains open until actual semantic and checker verification
finish. Checker verification has passed. These helper definitions/certificates do not prove the complete boundary
or source relations. No component-only commit or unit/W09 completion is issued;
check-fast.sh remains at T06-W12.
