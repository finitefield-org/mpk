# W09 unit 4 integer parser review (component in progress)

This review covers integer/duration/instant parsing definitions and their
source regeneration, exact metadata import and independent ordinary evaluation.
It does not close unit 4, unit 3 or W09.

## Direct review

- Compared error precedence with the frozen BoundaryCodec implementation.
  Full-width InputBound wraps the syntax/canonical/range decision. All active
  UTF-16 bits participate in digit classification; negative unsigned values
  are syntax errors. Long digit sequences still undergo syntax and canonical
  checks before the length shortcut may return Range.
- Reviewed the accumulator's sticky overflow: the high bits lost in shifts,
  both add carries and all prior overflow are retained. Range compares the full
  unsigned magnitude, including u64::MAX and the signed minimum's magnitude.
- Checked de Bruijn indices through all twenty state lets, the shared decision
  and final pack. Success and error arms clear inactive payload/header bits;
  result metadata uses the established positional field id `0`.
- Reviewed domain boundaries: zero-length/lone-sign decisions are lazy; the
  full 32-bit length prevents wrapped counts from authorizing an overlong input.
  Maximum-length syntax is an actual ordinary fold, not an oracle shortcut.
- Reviewed provenance: helper generation invokes BoundaryCodec only to validate
  the static codec configuration. Host parsing occurs solely in tests. The
  metadata describes a structural result and does not invent a nominal closed
  instance or assert a source/native proof.
- Shared helpers changed visibility only. Independent replay covers all 65
  integer formatter contexts and 54 pins after those visibility changes.
  Parser replay separately requires exact current bytes and metadata.

## Verification and open obligations

The 65-source import/mutation and 456-case ordinary semantic suite passed in
249.95 seconds. Inventory passed all five tests with exactly one additional
Std namespace consumer (118 to 119, total fixture matches 4,941 to 4,942);
removing only the new parser path reproduces the previous recorded path hash.
The existing inventory formatting is unchanged. Compile/lint, pin replay,
full-capacity semantics and both-checker runs are recorded separately in
`unit-4-integer-parse-progress.json`; ongoing runs are not passes.

No actionable code issue was found in this component review beyond the earlier
Rust overlapping-borrow compile error, fixed before semantic execution. The full-capacity four-case suite subsequently passed in 588.42 seconds; the
checker corpus then finished with 53 passing cases and one warning-contaminated
protocol failure. That one case passed both acceptance and corruption checks on
targeted recheck with separated streams; all 54 pinned cases are accounted for.
The original full run remains recorded as failed. Final scoped definition review
found no additional actionable issue.
Neither finite evaluation nor zero-axiom acceptance of helper definitions proves
the universal boundary/source/operation theorems. All original implementation
units and their exit gates remain intact. No partial component commit is made;
check-fast.sh remains at T06-W12.
