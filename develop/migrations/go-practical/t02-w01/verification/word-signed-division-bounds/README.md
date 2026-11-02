# Actual signed division ranges and the minimum/-one exception

This T02-W01 component proves the actual quotient/remainder ranges, zero
remainder correction and exact minimum-dividend/negative-unit quotient
correction for all 8/16/32/64-bit words. The complete ordinary certificate has
4,093 declarations, 99,499 terms, 1,465,556 bytes and zero axioms, with logical
certificate hash
`f847c610dd931b6cb2a6b7b99a56da0419938d819b04e1bf94577e40aaf7f047`.
All frozen 4096/131072/2097152 limits remain unchanged. W01 is In progress;
the complete source/program/foundation/application scope remains due.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/bounds.rs)
retains the entire [signed-refinement component](../word-signed-refinement/README.md).
Every one of its 3,897 original named types/bodies/proofs/levels/family identities
and its complete raw generator bytes remain exact. Eight additional original
half-positivity/magnitude-balance proofs come from the unchanged full signed-bound
input; 188 declarations are new. The [identity observation](retained-identities.json)
checks the extra metadata after the same explicit prerequisite renaming. It is
diagnostic evidence; both unchanged cores check every actual proof term.

Let a=U(actual AbsBits(x)), d=U(actual AbsBits(y)), q and r be the actual unsigned
quotient/remainder of those words, H half the modulus and M the modulus. The real
source divisor guard transfers through AbsBits and supplies d>0. The established
ordinary proofs give a=d*q+r, r<d and actual gaps gx/gy with a+gx=H, d+gy=H.
The [new natural proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/bounds/natural.rs)
use the real positive-divisor decomposition d=1+(d-1) to prove

```text
QGap(x,y) = (d-1)*q + r + gx
q + QGap(x,y) = H
q <= H
r < H
```

Strict upper bounds follow from actual strict gaps and additive balances. All
arithmetic is genuine unbounded Nat arithmetic. No huge unary machine literal,
caller-provided quotient, arithmetic oracle or operand enumeration supplies a fact.

For any actual word z with U(z)+gap=H, the
[sign proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/bounds/signs.rs)
show that its sign equals Nat.Equal(U(z),H), and the actual Neg(z) sign equals
actual DivideDefined(z). The zero branch uses the actual negation sum and Nat
zero-sum consequences. In the nonzero branch, a false negation sign would force
U(z)+U(Neg(z))<M, contradicting the exact negation conservation equation. Only
local Bool cases and checked strict-bound/cancellation laws are used.

Consequently the prior arbitrary-word selection correction is

```text
SelectionCorrection(z,flag)
  ≈ Int.FromNat(Bit(And(Not(flag), Nat.Equal(U(z),H))))
```

Actual remainder magnitude is strictly below H, so its correction is equivalent
to zero for every dividend sign. The actual signed remainder therefore equals
its mathematical signed magnitude remainder. The prior source refinement and
computed correction remain actual ordinary proof terms throughout this derivation.

The [negative-unit proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/bounds/exception.rs)
define NegativeUnit(y)=And(actual sign(y),Nat.Equal(UAbs(y),1)). Both directions
show that this predicate is true exactly when the actual signed input is
equivalent to mathematical -1. Nat induction proves r<1 implies r=0.

If the actual magnitude quotient equals H, its computed gap is zero. Nat
zero-sum consequences and positive multiplier cancellation force d=1 and a=H.
The actual half/minimum/sign correspondences then force the minimum dividend;
a positive mathematical quotient forces the divisor sign negative. Conversely,
a minimum dividend and negative-unit divisor force q=H and r=0. Thus, under
the actual source divisor guard,

```text
And(Not(actual Xor(sign(x),sign(y))), Nat.Equal(q,H))
  = MinimumMinusOne(x,y)
  = And(Nat.Equal(U(x),H), NegativeUnit(y))
```

The actual quotient correction CQ is equivalent to Int.FromNat(Bit of that
exact source predicate), while CR is equivalent to zero. This classifies the
previous [computed source corrections](../word-signed-refinement/README.md)
universally. The source reconstruction becomes

```text
S(x) ≈ S(y) * (S(actual SignedDiv(x,y)) + M * Bit(MinimumMinusOne(x,y)))
       + S(actual SignedRem(x,y))
```

QuotientNoOverflow is the separate optional exactness guard Not(MinimumMinusOne).
It yields exact quotient and ordinary signed reconstruction; it is not a source
definedness requirement. Source DivideDefined continues to reject only zero
divisors. Minimum/-one remains a normal source operation: the actual signed
quotient's unsigned value is H, and the actual signed remainder is equivalent
to zero. Both conclusions have explicit source result theorems.

The [verification record](verification.json) binds four affected roots, exact
owning import and unchanged prior bytes, strict selected lint/Rust/Go format,
and the [same-byte matrix](paired-final/result.json). Both unchanged production
cores inspect all 70 complete streams: two zero-axiom acceptances with matching
hashes and 68 actual core type rejections. Twelve generic controls change gap/
sign/correction/unit laws or their real premises. Fourteen per width change
actual ranges, negative-unit/minimum predicates, exactness/source guards or
minimum source results. No format/fuel/resource result counts as a semantic
negative; an honestly different valid component also rejects exact import.

The [Bool-only model](../../../../../../crates/mpk-vc/tests/support/go_word_division_bounds_observations.rs)
retains all 96 original full signed-operation rows, with 88 defined pairs and
eight actual false divisor guards. Both source quotient/remainder results run,
giving 176 actual source results. Actual Abs/unsigned magnitude/Neg/Top/guard
circuits execute under unchanged interpreter ceilings. Host derives numerical
gap/half/predicate/correction counterparts from those full word observations;
Nat/Int/guard/theorem bodies do not execute in the finite model. The universal
classification is supplied by the separately checked certificate.

The [native observation](native/result.json) and
[independent integer comparison](native-comparison-final/result.json) agree on
all 96 rows, including eight minimum/-one rows with normal outputs. There are
80 exact source reconstructions and eight that need the classified correction;
all 88 corrected equations hold. Native Go uses actual typed source operators,
unsigned division and math/big product gaps. Python uses signed truncation,
correction by exact signed difference divided by M, and the independent gap
H-minus-quotient. Original [query fields and full words](query-origins.json)
remain unchanged; unaffected arbitrary sign-selection rows are not repeated.

The original native build and both executions completed with byte-identical
96-row outputs, but a copied unique-ID expected count128 stopped the wrapper
before its final metadata write. The [original runner](native/original-runner.py),
[observed tool diagnostic](native/observed-wrapper-failure.txt), owned inputs
and all raw native logs remain. The [metadata recovery](recover-native-record.py)
changes only that expected count, validates all outputs and fully rechecks the
exact 5577-file prepared image after recovery. Earlier validation/zero command
statuses are qualified by the exact runner's reached control flow; lost original
timings stay null. No native build or observations are repeated, and no timing
is invented. The diagnostic text is retained tool output, rather than claimed
raw OS stderr. No native Linux release gate is claimed.

The record writer's first count-only source comparison searched a spaced token
that the original runner did not contain. Its [original source and tool diagnostic](checkpoint-recording/annotation.json)
remain; matching the exact unspaced tokens fixes only that metadata comparison.
No proof, model or native observations are repeated for this recording repair.

The [one-root construction](actual-minimum-classification/rust-construction/result.json)
and [independent Go check](actual-minimum-classification/go-candidate/result.json)
accept the exact final certificate bytes. All captured Rust inputs remain fixed
until their terminal handles. All 255 frozen design inputs and official scanner
rules/seals remain unchanged, with only exact per-file raw libtest EOF observations.

Remaining variable/signed/right-shift numeric/source laws, full carrier/state/
program/bounded 56-family foundation/descriptor/private registry/context and
authoritative original application source/export/selected-closure replay remain
required. All 169 application demands stay unelaborated. C# tests stay skipped;
the cumulative T02 gate stays deferred to W04: `check-fast.sh --skip-csharp`.
No complete source-program proof, immutable application replay or practical-profile
activation is claimed.
