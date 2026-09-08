# W09 unit 2 decimal conversion/rounding component: direct review

Scope: 33 decimal conversion, unary and rounding definitions; original-source
linkage; exact product storage; finite step composition; fixtures; and the test
observer's shared arrays and call-by-need evaluation. This is not completion of
unit 2 or W09.

The review checked the unit-1 product address ordering and prepended child
padding independently of the circuit encoder. All outputs initialize the full
512-address cube to zero, then write only sign, scale and coefficient fields.
All nine integer/char input widths and signed-minimum magnitudes are preserved.
Unary plus/negate retain scale and representation-sensitive signed zero.

For valid decimal scale 0-28, the initializer chooses the smaller target scale
and the exact reduction count. Each of 28 ordinary steps divides by ten, retains
the last discarded digit, and ORs earlier discarded nonzero digits into a sticky
bit. Count zero is an identity. Final rounding is applied once; cases such as
2.51 rounded to zero digits with nearest-even distinguish this from repeated intermediate rounding.
Signed floor/ceiling and unsigned conversion of negative fractional zero follow
the T03 oracle. Range/overflow failures remain ordered and have a zero result.
Helpers expose their concrete cube signatures, while public definitions retain
source signatures. Invalid decimal carrier domains remain later work.

A source fixture combining conversion and equality correctly failed because
comparison is not yet implemented. The positive conversion case was replaced
with the dedicated original capture; the combined source remains an explicit
rejection test. Missing decimal definitions cannot silently disappear. Imported
metadata, signatures, ordered checks and exact certificate bytes are rebuilt.

The original actual-core test observer retained growing closure environments
and used about 7 GB while still running. Resource-heavy observation attempts were
interrupted, not classified as deterministic proof failures. The final test-only
observer shares Boolean-array storage and memoizes suspended core arguments/let
values within their captured environments. It forces only demanded values and
the selected Bool-recursion branch. Thus unused arithmetic is not evaluated,
while every observed result still comes from the actual core term. There is no
operation-name shortcut or host arithmetic result. Memo tables remain per closure
or suspended term, never shared across different captured environments. Production
terms and both checkers are unchanged. Existing cube, integer, temporal, calendar,
floating and conversion core cases exercise the changed observer. Final results
are recorded in `unit-2-decimal-verification.json`.

Direct review findings for this component: 0. Seven certificates are accepted
with zero axioms by both unchanged checkers and hash mutations reject. All 33
metrics and pinned bytes reproduce, and consumer inventory baselines are unchanged.
Decimal arithmetic/comparisons, UTF-16 strings, domains, whole-foundation expansion,
application proofs and units 3-8 remain outstanding. The full gate stays at W12.
