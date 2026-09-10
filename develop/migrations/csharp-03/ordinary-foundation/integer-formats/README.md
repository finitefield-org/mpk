# Canonical integer formatting — partial W09 unit 4

This component defines canonical base-10 output for all eight signed/unsigned
8/16/32/64-bit integer codecs, `duration_ticks` and `unix_milliseconds`.
Each ordinary function takes its concrete scalar Boolean cube and returns
non-null C19 text. Zero emits `0`; negative values emit one leading minus;
there are no redundant leading zeros, plus signs, separators or locale inputs.

The shared division helper consumes the 64 magnitude bits from most to least
significant. Its four-bit remainder is in 0..9; the next five-bit partial
dividend is in 0..19, so one subtraction of ten yields the next quotient bit
and remainder. Twenty concrete quotient/remainder stages cover the full u64
range. Signed magnitudes are obtained at their original bit width before
unsigned extension. This retains the magnitude of every signed minimum without
requesting a representable positive signed result.

Quotient/remainder stages and the reversed digit storage are let-bound outside
the output selectors, sharing arithmetic state across output observations.
The highest nonzero quotient determines the digit count, with one digit for
zero. Length includes the sign when present. Output selection reverses the
stored digits, inserts the minus sign and clears every inactive character cell
and header padding bit. No CLR formatter or host-computed numeric answer becomes
a definition. The maximum valid output length is twenty code units.

Generation examines 65 actual-source contexts: the earlier 64-context corpus
plus an existing captured `TimeSpan` addition source needed for duration.
54 contexts pin 86 formatter occurrences across all ten codec IDs. The largest
program has 10,693 terms and 130 declarations. Canonical import regenerates the
exact source/foundation metadata and bytes and rejects metadata, hash and
cross-context substitutions.

The independent `BoundaryCodec` model supplies expected output using each
captured context's validated root/closed set. Tests observe the actual ordinary
length and every active UTF-16 bit, parse the observed text with the independent
model, and compare the original scalar. They cover signed minima, maxima,
unsigned maxima, zero, negative values and powers of ten with their neighbours.
Header padding is checked, while large inactive storage is sampled at the first
inactive cell, capacity boundaries and high index bits. These are finite tests,
not an ordinary parser or universal round-trip proof.

Generation/semantic, pinned replay, lint, inventory and the 54-case same-byte
dual-checker harness are tracked independently in
`../unit-4-integer-format-progress.json`. All 54 same-byte checker cases passed
in 387.289 seconds with zero axioms and rejected hash corruptions. Fixed replay
of all 65 contexts passed in 14.27 seconds, and latest lint, inventory and
format checks passed. The source semantic matrix then passed all 392
observations in 4407.63 seconds. Scoped definition review found no additional
actionable issue. Integer parser implementation and its ongoing verification
are recorded separately in `../integer-parsers/`. Registered result linkage,
decimal and calendar/time codecs, whole-document JSON relations, source/native
semantics and all remaining W09 proof work remain required. No internal unit or W09 completion is claimed;
the repository gate remains deferred to T06-W12.
