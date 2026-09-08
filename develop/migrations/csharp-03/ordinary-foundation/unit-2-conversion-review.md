# W09 unit 2 numeric-conversion component: direct review

Scope: the six frozen non-decimal numeric conversions, their integration into
floating source-bound generation/import, fixtures and targeted tests. This is
not a unit 2 or W09 completion receipt.

The review checked the exact closed signature whitelist and registry-derived
ordered overflow check. Integer-to-float lowering retains the unsigned magnitude
of signed minima and reuses the ordinary nearest-even packer. Width conversion
aligns NaN payloads and sets the quiet bit independently of discarded payload
bits, preserving sign and correctly distinguishing NaN from infinity. Finite
conversions preserve signed zero and handle target subnormal rounding once.

Checked float-to-integer conversion uses every exponent bit in saturating shifts;
it does not inherit source integer shift-count masking. The separate highest-set-
bit check prevents a discarded large magnitude from appearing in range. The
sign-dependent magnitude bound admits the signed minimum and rejects its positive
counterpart. Truncation occurs before the bound check. All nonfinite operands
fail. Actual-core tests check Success, ordered Overflow and a zero normal result
on failure, as well as normal results. Full exponent sweeps would fail if high
shift counts wrapped or if the independent magnitude guard were removed.

The oracle remains the independent T03 numeric implementation. Its observations
are used only in tests; emitted results are ordinary Boolean definitions. The
same six certificate byte sequences are accepted with zero axioms by the two
unchanged checkers, and both reject each corrupted certificate hash. Generation
and import independently reconstruct original source/foundation and signature
metadata; source and check substitutions reject. The existing floating-source
cases use the same reviewed linkage helper. Changes to unrelated temporal and
calendar test bodies were removed during review.

The scoped regeneration check preserves existing temporal/calendar/floating
fixtures and metrics, while pinning six conversion certificates. Consumer
inventory tests pass without any inventory baseline change. Direct review
findings for this component: 0. Exact verification evidence is recorded in
`unit-2-conversion-verification.json`.

Decimal operations/conversions, UTF-16 strings, input-domain predicates,
all-instance expansion, application VC proofs and units 3-8 remain outstanding.
The full T06 gate remains deferred to T06-W12.
