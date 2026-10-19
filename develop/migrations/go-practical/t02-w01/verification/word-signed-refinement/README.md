# Actual signed division and remainder with computed modular corrections

This T02-W01 component proves numeric relations for the actual 8/16/32/64-bit
SignedDiv/SignedRem definitions. Mathematical signed quotient/remainder values
come from the actual unsigned division of actual operand magnitudes; explicit
ordinary corrections relate those values to the signed machine outputs.
The complete certificate has 3,897 declarations, 94,497 terms and 1,390,896
bytes, with zero axioms and logical certificate hash
`6462496c5aa8708f1c03c3fa4f772c07a98287063733d91a5e9ccf027fd17127`.
All frozen 4096/131072/2097152 limits remain exact. W01 is In progress;
universal classification of the corrections is still required.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division.rs)
retains the complete [compact prerequisite basis](../word-signed-division/README.md).
All 3,483 original named types/bodies/proofs/levels/family identities match,
and the original prerequisite generator keeps its complete raw bytes. A fixed
extra-root route selects 166 genuine original Int equivalence/congruence
dependencies; 248 declarations are new. The
[additional identity observation](retained-identities.json) compares every
extra Int declaration with the exact original full word certificate. This
structural comparison is diagnostic metadata; both cores check every proof.
No receipt, caller formula, new axiom/core rule/fuel or external import authority
is introduced.

The generic ordinary representation is

```text
SignedNat(flag, n) = Int.Make(choose(flag, 0, n), choose(flag, n, 0))
```

It represents minus n when the flag is true and plus n otherwise. Int equality
is the established Nat cross-sum relation, rather than equality of its function
representatives. For any actual unsigned word q and arbitrary Bool flag, let
selected = actual Select(flag, actual Neg(q), q). The
[selection proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/selection.rs)
compute a real ordinary Int correction:

| Flag | Positive correction part | Negative correction part |
| --- | --- | --- |
| false | Bit(actual sign(q)) | 0 |
| true | Bit(actual sign(Neg(q))) | Bit(actual DivideDefined(q)) |

The checked relation is

```text
SignedNat(flag, U(q)) ≈ S(selected) + modulus * SelectionCorrection(q, flag)
```

This is unconditional over arbitrary word bits. It also covers negative
corrections when an arbitrary unsigned magnitude exceeds the signed half range.
Each branch uses explicit actual-word/projection equations, genuine Nat
polynomial identities and the established actual NegationValueSum. No full word
operand enumeration or function extensionality is used.

The [generic scalar theorem](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/scalar.rs)
proves signed reconstruction from a=b*q+r for all Nat a,b,q,r and both operand
signs. Four local sign cases use Xor for the quotient and the dividend sign for
the remainder. The [actual family](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_division/family.rs)
supplies every amount, sign and premise:

```text
aq = actual UnsignedDiv(actual AbsBits(x), actual AbsBits(y))
ar = actual UnsignedRem(actual AbsBits(x), actual AbsBits(y))
QM = SignedNat(actual Xor(sign(x), sign(y)), U(aq))
RM = SignedNat(actual sign(x), U(ar))
```

Under the actual source divisor guard, actual magnitude divisor transfer and
unsigned division prove UAbs(x)=UAbs(y)*U(aq)+U(ar) and U(ar)<UAbs(y).
Actual signed input magnitude equivalence, the generic scalar theorem and
original Int equivalence/congruence laws then prove
S(x) ≈ S(y)*QM+RM.

Exact actual SignedDiv/SignedRem definition theorems bind their outputs to those
sign selections. Computed quotient/remainder corrections CQ/CR prove
QM≈S(actual SignedDiv)+modulus*CQ and
RM≈S(actual SignedRem)+modulus*CR. Consequently the guarded actual source
reconstruction is

```text
S(x) ≈ S(y) * (S(actual SignedDiv(x,y)) + modulus * CQ(x,y))
              + (S(actual SignedRem(x,y)) + modulus * CR(x,y))
```

No caller-provided quotient or numeric truth flag supplies these source facts.
Zero divisors remain source faults; only the totalized value-component selection
relations are unconditional. The source reconstruction requires the real guard.
Universal quotient/half bounds, zero remainder correction and the precise
minimum/-one quotient-correction classification remain due.

The [current execution](final/rust-tests/result.json) binds four affected roots,
exact owning import, all original prerequisites, typed controls and actual Bool
observations. The [paired matrix](paired-final/result.json) checks 62 complete
identical streams in both unchanged production cores: two zero-axiom acceptances
with matching hashes and 60 actual core type rejections. Four generic controls
alter signed projection, scalar conservation, its balance premise or quotient
sign. Fourteen controls per width alter correction parts, refinements, actual
guards/reconstruction or source operator bodies. Replacing SignedDiv/SignedRem
with zero now rejects in every width; the prerequisite-only component correctly
accepted its earlier source-definition control. Exact import also rejects an
honestly different otherwise valid component.

The [Bool-only observation](../../../../../../crates/mpk-vc/tests/support/go_word_signed_refinement_observations.rs)
uses all 128 original full Neg/SignedDiv/SignedRem rows. Both generic flags give
64 selection observations and the 96 signed rows give 88 defined source results
plus eight actual false divisor guards. Actual source words, unsigned amounts,
negated amounts, Top/zero flags and selections execute under the unchanged Bool
interpreter ceilings. Host derives the small correction parts from those actual
flags; Nat/Int/correction/theorem bodies do not execute. Generic corrections
include -1, 0 and 1. Finite signed outputs have 84 zero corrections and four
positive-half quotient wraps; all eight minimum/-one source rows remain normal
observations. These finite classifications are not universal proof claims.

A [fresh pinned native build](native/result.json) runs the
[independent typed Go observer](../../../../../probes/go-practical/signed_refinement_observer.go)
under an exact isolated 5577-file prepared image with empty cache and complete
before/after name/byte/mode checks. Two native outputs are identical. Native
math/big quotient/remainder and exact corrections plus an
[independent integer oracle](native-comparison-final/result.json) match all 160
observations. Original query fields/full words remain exact in the
[query origins](query-origins.json). No native Linux release gate is claimed.

The [initial intended-positive rejection](actual-source-corrections/rust-construction/result.json),
[independent Go rejection](actual-source-corrections/go-candidate/result.json),
original sources/raw diagnostics and [direct prefix diagnostic](first-mismatch/rust-diagnostic/diagnostic.json)
remain preserved. The first failure is ScalarSignedDivision: a computed Xor
recursor major needs a real Bool equality transported through the Nat choice
motive. The [fixed-source construction](explicit-flag-transport/rust-construction/result.json)
and [Go inspection](explicit-flag-transport/go-candidate/result.json) accept
the exact final bytes. The repair adds Eq.congr transport to the recipe;
neither checker nor any limit changes. Captured Rust inputs remain fixed until
the terminal result. The [verification record](verification.json) binds the
check scopes, diagnostics, format/lint results and auxiliary diagnostic lifecycle.

All 169 application demands and the remaining numeric/source/carrier/state/
program/bounded 56-family foundation/descriptor/private registry/context/
authoritative original source/export replay remain required. No complete
source-program proof, immutable application replay or practical-profile
activation is claimed. C# tests stay skipped; the cumulative T02 gate stays
deferred to W04: `check-fast.sh --skip-csharp`.
