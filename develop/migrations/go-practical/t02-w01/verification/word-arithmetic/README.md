# Actual fixed shifts and multiplication in unbounded Nat

This W01 component extends the exact preceding numeric basis with ordinary
unbounded PowerTwo, actual discarded-bit/fixed-left-shift laws, exact bit-prefix
expansion and complete actual shift/add multiplication conservation. The
certificate has3828 declarations,101,398 terms and1,358,764 bytes, zero axioms,
and certificate hash
`ead6495bce8b5be3697d9a759432909a2d0c066e93e768b8a3997c518f14741d`.
All2579 preceding numeric declarations and55,105 term nodes remain exact;
regenerating that complete [component](../word-numeric/README.md) retains its
original743,954 bytes. Every255 frozen design input remains unchanged. No
prototype seed/certificate, graph, receipt or context is foundation authority.

The [fresh arithmetic producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic.rs)
defines PowerTwo on genuine unbounded Std.Nat by ordinary recursion. It proves
the exponent-addition law for arbitrary natural exponents and connects powers
at word widths to the existing shared squared weights. Explicit checked Nat
numeral-addition congruence handles computed exponents before recursion; no
2^64 unary literal is materialized.

The [balanced push proof](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic/push.rs)
tracks the actual dropped highest bit when shifting and inserting a low bit.
Low/high block laws compose through checked Nat semiring identities. Explicit
word projection equalities lift through Value before Nat arithmetic, and Top
equalities lift through Bit. This avoids implicit conversion of large scalar
denotations. The result concerns the existing actual PushLow circuit:

```text
U(PushLow(x,b)) + modulus * Bit(highest(x)) = 2*U(x) + Bit(b)
```

The [fixed-shift producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic/shift.rs)
builds a genuine Nat ShiftDiscard_i from actual highest bits of previous Left
results. For every arbitrary full8/16/32/64-bit word and every fixed index
0 <= i < width, it proves:

```text
U(actual Left_i(x)) + modulus * ShiftDiscard_i(x) = PowerTwo(i) * U(x)
```

Left0 versus arbitrary x uses pointwise equality. Later shifts share a checked
[literal-bit value congruence](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic/bits.rs):
all addressed bit equalities imply equality of balanced Nat values. A single
generic Bool address-elimination proof per width is reused. Each actual shift's
closed bit projections checks independently; this assumes neither function
extensionality nor a larger conversion budget. Zero values, actual Select with
a zero branch and actual Add with its real outgoing carry have separate laws.
Selection first proves word equality and lifts through Value, including32/64
bits. These fixed shifts are the ones used by actual multiplication; general
variable-count/signed/right-shift numeric correctness remains separate work.

The [digit-prefix proof](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic/digits.rs)
defines prefixes from the actual multiplier bits. A balanced236-block proof
connects the complete prefix to the same balanced Value, using checked exponent
splitting and Nat arithmetic. The
[product proof](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/arithmetic/product.rs)
then follows every actual Mul0..Mul_width definition. Its Nat MulDiscard witness
adds selected shift-discard amounts and actual full-adder carries. A checked
generic step composes the preceding product, shift, selection and addition
laws. For arbitrary operands and an arbitrary unbounded initial Nat:

```text
initial + U(x)*U(y)
  = initial + (U(actual Mul(x,y)) + modulus * MulDiscard_width(x,y))
```

The witness is computed by the ordinary definitions from the actual words and
circuits; it is no caller-provided quotient or true flag. All120 multiplication
steps, the four initial states and final/arbitrary-initial equations are checked.
This is exact unsigned
product conservation, not an assumed arithmetic evaluator, a signed Int
multiplication theorem or Go source/program correctness.

The [original final execution](final/rust-tests/result.json) passes the complete
component/import/retained-basis root, three generic controls and the actual
finite observation root. One deliberately wrong64-bit witness obtained by
dropping its final addend exhausts Rust conversion fuel; that original
candidate/report/input remains and supplies no semantic negative. The
[reviewed control-only change](review/control-source-change.json) instead sets
that witness to the distinctly wrong constant0 at all four widths. The
[focused corrective run](corrected/rust-controls/result.json) checks those four
changed cases and the one previously unreached64-bit shift-discard case. The
other eighteen actual typed rejections and all original positive/production/
model bytes are retained without unrelated-root repetition.

The [final same-byte matrix](paired-final/result.json) checks26 complete streams
in both unchanged independent cores. Three accept with zero axioms and matching
certificate/export/axiom hashes. All23 final wrong power/push/left/shift-premise/
discarded-value/omitted-contribution controls reject as core type mismatches:
22 conclusion mismatches and one lambda-domain mismatch. Every width includes
an omitted product contribution and omitted shift contribution. No format,
fuel, resource or diagnostic-prefix outcome is counted as a semantic rejection.
Exact arithmetic import regenerates all bytes and rejects altered, oversized
and honestly different valid components. Current scoped
[Clippy](corrected/clippy/result.json), [Rust format](corrected/format/result.json)
and [Go observer format](final/go-format/result.json) pass.

The [actual Boolean-term observation](../../../../../../crates/mpk-vc/tests/support/go_word_arithmetic_observations.rs)
selects all48 original frozen Mul rows and evaluates all1440 actual Left_i,
Mul_i and carry stages from the same decoded certificate. It uses only Bool
constructors/recursors and assigned input-word bits, with no arithmetic model
callback. It observes107 true carries,181,613,956 total term/application steps
and13,336,376 for the largest query. Scalar witness recurrence values are
derived in host arithmetic from these actual bits/carries; Nat witness
declarations are not executed as enormous unary values. Their universal
meaning is checked separately by both cores.

The [new independent native observer](../../../../../probes/go-practical/arithmetic_observer.go)
executes typed Go unsigned/signed multiplication, every native fixed shift and
shift/add carry, math/bits and exact math/big products/quotients. A
[fresh empty-cache build](native/result.json) uses an isolated exact5577-file
prepared image, fully checked before and after. Two native outputs match byte
for byte. Every root/stage,107 carries and exact quotient agrees with the
actual Boolean DAG and an independent Python integer
[oracle](native-comparison-final/result.json). The original1640-query input is
unchanged; [origins](query-origins.json) bind the48 selected original IDs/full
hex words. These are finite comparisons, not source, immutable application
replay or native Linux release evidence.

All initial findings remain with complete original sources and raw diagnostics.
The [first candidate](initial/rust-tests/result.json) fails both cores at
PowerTwoWeight1 because a computed exponent needed explicit congruence. The
[diagnostic helper's first run](initial/rust-diagnostic/result.json) uses the
wrong package-relative path; its corrected-path run locates the original
failure. A later helper compilation moves its input path and needs a borrow;
that diagnostic/source also remains. Diagnostic in-memory declaration-prefix
checks are never full canonical-certificate acceptances. The exact final
[temporary harness](diagnostics/declaration-prefix-harness.rs) is archived and
removed from active test discovery.

The [complete draft](product/rust-tests/result.json) reaches a construction
limit before emitting a candidate. Its [construction diagnostic](envelope/rust-construction/result.json)
finds3736 declarations and131,072 terms at W64.DigitsBlock0At35. Sharing fixed
vector correspondence brings the [next candidate](shared-vectors/rust-construction/result.json)
inside every unchanged artifact bound, but implicit large scalar projection
conversion exhausts fuel at PushConservation4. Explicit projection lifting
advances the [next run](explicit-projections/rust-construction/result.json) to
W32.SelectZeroValue; explicit word selection lifting advances the
[following run](explicit-selection/rust-construction/result.json) through32-bit
multiplication to W64.LeftStep1. Its full-function conversion still exhausts
fuel in both cores. Shared literal-bit value congruence makes the
[all-width smoke](literal-values/rust-construction/result.json) pass, with an
independent Go acceptance of exactly those bytes. Final bytes are identical.
No checker reduction, axiom, fuel, construction bound or frozen input changes.

The [verification record](verification.json) distinguishes original failures,
diagnostic prefixes, current actual checked streams and scoped corrective
coverage. Strict changed-input review retains original raw stdout EOF bytes
under exact hash/length observations only, without source/JSON/directory or
blanket whitespace exemptions. The full T02 gate stays deferred to W04:
`check-fast.sh --skip-csharp`; C# tests are skipped.

Word bounds/order/modulo/division/remainder and variable/signed/right-shift
numeric laws, signed Int multiplication and full source-typed operation
correspondence remain due. Full carriers/strings/products/errors/slices/storage/
lifetimes/definedness/program assembly and the56-family descriptor/private
registry/context remain due. All169 Go application demands remain unelaborated.
W01 stays In progress; no public profile, whole Go source/program proof,
authoritative immutable application replay or Linux release gate is claimed.
