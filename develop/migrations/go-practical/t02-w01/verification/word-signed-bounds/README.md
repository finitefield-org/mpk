# Actual signed magnitude half bounds and minimum correspondence

This T02-W01 component proves the actual8/16/32/64-bit signed magnitude's
half-range bound, computed gap and minimum/sign correspondence. Its fresh
zero-axiom certificate has2683 declarations,58,000 terms and818,605 bytes,
with certificate hash
`de096662754ff6155328b90849f1946b75fde7860b458219153842c51d29c004`.
The frozen4096-declaration/131072-term/2097152-byte limits, core rules, axioms,
fuel and all255 frozen design inputs remain unchanged. W01 is In progress.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds.rs)
retains the complete [signed-magnitude component](../word-signed-magnitude/README.md)
and adds520 declarations in a disjoint namespace. All2163 prior named types,
bodies, opaque proofs, levels and family metadata independently match the
original component. Its complete generator keeps the exact prior bytes;
unaffected prior proof/model/native suites are not repeated.

The [balanced definitions](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds/half.rs)
represent each half modulus without expanding a unary machine integer. At
cube depth0, H=1 and Low=LowComplement=0. For d>0 and h=d-1:

```text
H_d = Weight_h * H_h
Low_d(x) = Value_h(low_half(x)) + Weight_h * Low_h(high_half(x))
LowComplement_d(x) = Value_h(actual NotCube_h(low_half(x)))
                   + Weight_h * LowComplement_h(high_half(x))
Top_d(x) = x(true, ..., true)
```

For every depth0..6 and arbitrary actual Bool cube, the
[split proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds/split.rs)
establish H_d+H_d=Weight_d, H_d>0,
Value_d(x)=Low_d(x)+H_d*Bit(Top_d(x)),
(Low_d(x)+LowComplement_d(x))+1=H_d and Low_d(x)<H_d.
All lower bits and the actual highest bit participate in these equations.

The [actual word family](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds/family.rs)
connects Top_d to the actual machine sign. Its computed ordinary gap is

```text
MagnitudeGap(x) = choose(actual sign(x), Low_d(x), LowComplement_d(x)+1)
U(actual AbsBits(x)) + MagnitudeGap(x) = H_d
```

The gap equality proves magnitude<=H_d for every word. Actual sign=false
implies both unsigned value and magnitude are strictly below H_d; actual
sign=true gives magnitude+Low_d(x)=H_d. The
[generic scalar proof](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds/scalar.rs)
uses actual checked split/complement/double/negative-balance equations,
ordinary Nat cancellation and local sign cases. Every premise is supplied by
a checked actual word theorem; no caller-supplied numeric value or truth flag
establishes source correspondence.

The [minimum proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds/minimum.rs)
show that half magnitude forces the actual input sign true, and prove both
Nat-equality directions of
U(actual AbsBits(x))=H_d iff U(x)=H_d. Their actual reflected Nat.Equal Bool
values agree. Thus half magnitude characterizes the actual minimum signed
input word through its unsigned half-modulus value.

The [threshold bridge](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_bounds/threshold.rs)
proves Less(U(x),H_d)=Not(actual sign(x)). Under the actual proved magnitude
bound, ordinary Nat total-order equality identifies its not-below-half flag
with equality to H_d. Therefore the actual magnitude sign bit equals
Nat.Equal(U(x),H_d). The minimum magnitude retains its high bit normally;
there is no source rejection or assumption that every magnitude is signed
nonnegative. No machine arithmetic rule, new axiom, function extensionality
or enumeration of full word operands is used.

The [current four-root execution](final/rust-tests/result.json) passes exact
owning import, all2163 prior named declarations, the complete old raw-byte
consumer, typed generic/depth/width controls and actual Bool observations.
Altered/oversized/honestly different valid components reject exact import.
Current strict [Clippy](current/clippy/result.json),
[Rust format](current/format/result.json) and
[new Go format](current/go-format/result.json) pass. No C# tests execute.

The [same-byte matrix](paired-final/result.json) checks60 complete streams in
both unchanged independent production cores: two zero-axiom acceptances with
identical hashes and58 actual core type rejections,49 check-type and nine
lambda-domain mismatches. Eight controls target generic bounds/premises;
fourteen alter half/top definitions at every cube depth; nine per machine
width alter actual gap/bound/sign/minimum/threshold laws. Format/fuel/resource
outcomes supply no semantic negative.

The [Bool-only observation](../../../../../../crates/mpk-vc/tests/support/go_word_signed_bounds_observations.rs)
executes96 actual source/Neg/AbsBits words,2880 word-bit observations and96
new Top_d applications over all32 original frozen Neg inputs. Host balanced
recurrences derive scalar half/lower-part/complement/gap counterparts. The
model uses2,194,408 AST/application steps,209,227 at the largest input.
It does not execute Nat half/lower-part/gap/guard/theorem bodies as huge unary
values; their universal meaning is checked separately in both cores.
Original fields and full words remain exact in the [query origins](query-origins.json).

A [fresh pinned native build](native/result.json) compiles the
[independent Go observer](../../../../../probes/go-practical/signed_bounds_observer.go)
under an isolated5577-file prepared compiler image with empty cache and
complete before/after byte/name/mode checks. Actual typed signs, negation,
conditional magnitude and direct lower-bit masking/complement/gap operations,
plus math/big absolute values and modulus, yield two byte-identical runs.
The [independent comparison](native-comparison-final/result.json) agrees on all
32 inputs. Python uses remainders/subtraction and half minus absolute value
instead of either the model's balanced recurrence or Go's conditional gap.
There are four minimum magnitudes, four zeros and16 nonnegative inputs.
The unchanged full-adder stage suite is not repeated for these new range laws.

The [initial compiler diagnostic](actual-half-range/rust-construction/result.json)
and its exact source/raw output remain. Moving observed.right prevents the
initial Rust build; cloning the identical expression repairs ownership without
a proof outcome for that failed build. The fixed-source
[one-root construction](actual-half-range-and-sign/rust-construction/result.json)
and [Go inspection](actual-half-range-and-sign/go-candidate/result.json) accept
the exact final bytes. All captured Rust inputs remain fixed while either construction or the
expanded final verification is pending. The [verification record](verification.json)
binds these scopes and the selected-check rationale.

Actual signed division/remainder with the minimum/minus-one case, remaining
variable/signed/right-shift numeric/source laws, full source-typed carrier/
operation/storage/lifetime/program foundation, bounded56-family descriptor/
private registry/context and authoritative original source/export/selected-closure
replay remain due. All169 application demands remain unelaborated. No complete
source-program proof, immutable application replay, practical profile activation
or native Linux release acceptance is claimed. The cumulative T02 gate stays
deferred to W04: `check-fast.sh --skip-csharp`.
