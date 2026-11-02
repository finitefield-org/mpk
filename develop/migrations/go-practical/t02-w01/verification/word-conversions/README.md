# Actual integer conversions and full64 shift-count normalization

This T02-W01 component proves all 32 actual signed/unsigned source conversions
among 8/16/32/64-bit words, both target integer interpretations and actual signed
count normalization. Its ordinary certificate contains 1,882 declarations,
31,495 terms, 483,095 bytes and zero axioms, with logical certificate hash
`fae13fa0cae53f8e7c851cf87d34fe5b3f12284b17c5a1ba5a9fdaee3c1267bf`.
All frozen 4096/131072/2097152 limits remain unchanged. W01 is In progress;
variable shifts and the complete source/program/foundation/application scope
remain required.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/conversions.rs)
generates the original complete word/base numeric definitions and the unchanged
order proof recipes, then retains fixed complete dependencies. It reuses the
original literal-address value congruence proof through a private wrapper;
existing producers retain their complete raw bytes. Every one of 888 retained
named types/bodies/proofs/levels/family identities matches the original order/
arithmetic reference without any renaming or substitution. The 994 additional
declarations are new. The [metadata observation](retained-identities.json) is
diagnostic only; both unchanged production cores inspect the actual proof terms.

The [structural cube laws](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/conversions/cubes.rs)
work over arbitrary input bits. Fill(d,b) has value Ones(d)*Bit(b), with
Ones(d)+1=Modulus(d). For widening, the computed mask satisfies

```text
Msource + Mask = Mtarget
U(Extend(x,fill)) = U(x) + Mask*Bit(fill)
```

For narrowing, Low is the actual lower source cube and Q is computed from the
actual higher source cube values. The computed radix ratio satisfies

```text
Mtarget * Ratio = Msource
U(x) = U(Low(x)) + Mtarget*Q(x)
```

The [actual conversion bindings](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/conversions/family.rs)
prove that every original source conversion has the same literal bits as that
structural extension/truncation, through the established ordinary value
congruence theorem. The actual result is strictly below its target modulus.
No quotient, fill truth flag or result value is supplied by a caller. Only
fixed address selectors are eliminated; input operands remain arbitrary.

Shared [NarrowCross/WidenCross and ScalarCorrection proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/conversions/scalar.rs)
use genuine unbounded Nat/Int algebra. For each actual conversion and each
unsigned/signed target interpretation,

```text
source desired value ≈ target actual value + Mtarget * Int.Make(CP,CN)
```

For narrowing, CP=Q+target-sign (zero for an unsigned target interpretation),
CN=Ratio*source-sign (zero for an unsigned source). For widening, CP is the
target sign or zero and CN is the source sign or zero. Both interpretations
have explicit actual-result refinement theorems. Signed source widening and
unsigned source widening preserve their respective mathematical values exactly,
including equal source/target widths. Wrapping conversions remain ordinary Go
operations; an exactness theorem does not redefine their domain.

Actual SignedTo64 counts have the source top bit as their highest bit, so the
actual SignedCountDefined guard is Not(source top). Under that real guard,
the entire unsigned64 count value equals the original unsigned source value.
Unsigned To64 always preserves the entire value. For unsigned8/16/32 inputs,
its zero extension also satisfies the signed64 guard. Unsigned64 high-bit counts
remain defined through the unsigned route. Full source count-type attachment
and variable/signed/right-shift numeric correctness remain separate work.

The [verification record](verification.json) binds four affected roots, exact
owning import, unchanged old generator bytes, strict selected Clippy/Rust/Go
format and an ignored-diagnostic listing. Both unchanged cores inspect the
[same 172 complete streams](paired-final/result.json): two zero-axiom positives
with matching hashes and 170 actual core type rejections. Ten shared controls
change fill/mask/ratio/truncation/scalar laws or real arithmetic premises. All
32 actual conversion definitions, all 32 unsigned conservation laws and all
64 target interpretation refinements are independently changed. Twenty exact
widening claims and twelve count laws/premises are also changed. An honestly
different valid component passes both cores but rejects exact owning import.
No format, fuel or resource outcome counts as a semantic negative.

The [Bool-only observation](../../../../../../crates/mpk-vc/tests/support/go_word_conversions_observations.rs)
retains all 256 original complete conversion rows and full words. Every actual
source/result word, source/result sign, signed/unsigned To64 and actual signed
count guard executes through unchanged Bool primitives and original interpreter
ceilings. Host derives numerical quotient/mask/correction counterparts from
those observations; Nat/Int/quotient/ratio/mask/theorem bodies do not execute.
The separately checked ordinary certificate supplies the universal laws.

The [fresh native observation](native/result.json) uses actual typed Go signed/
unsigned casts, real negative-count shift expressions and math/big conservation
checks. Each original conversion input x is additionally used as a count with
the source signedness indicated by its conversion operation; no original query
field is replaced. All 256 actual conversions and count normalizations agree
with the Bool model and [independent Python integers](native-comparison-final/result.json).
Python uses arbitrary-precision modular casts, integer quotient and exact signed
differences for corrections. There are 192 defined counts and 64 actual recovered
negative-count panics, plus 80 signed and 80 unsigned exact type cases. Exactness
flags describe theorem applicability by types; they are not an iff classifier
of finite-input exactness. Variable shift result circuits do not execute in
this model. Two native runs are byte identical, with an empty build cache and
before/after checks of the exact 5577-file prepared Go image. No Linux release
acceptance is claimed.

The initial copied native query path failed before image creation or native
work. The [original runner, copied source and diagnostic](native-input-path/annotation.json)
remain. Changing only that path to root/queries.json permits the single fresh
build and two observations. Preserved tool-response text is not claimed raw
OS stderr. No native build or observations are repeated.

The original per-family polynomial expansion reached the frozen declaration
limit while entering W64To16 with 4,056 declarations. Original sources/raw
phase logs remain. Shared scalar laws retain every operation and reduce the
complete component to 1,882 declarations. Original FillValue6 candidates then
exhaust normalization fuel in both cores; an additional cancellation equation
alone does not fix this and its failed candidate remains. Mapping a small cube
equality through Value resolves that expansion. Subsequent direct diagnostics
identify MaskBalance3To3 and W8.UnsignedCountGuard type mismatches. Explicit
mask/ratio/low/quotient equalities through Nat operators and actual top-bit
equalities through Not resolve the computed-recursor-major issues. Original
compiler/ownership/partial-edit failures, candidates, raw checker outputs and
all direct prefix diagnostics remain. The diagnostic helper is ignored from
its creation; current listing does not repeat it. No prefix acceptance gives
complete certificate authority, and none of these original intended-positive
failures counts among the 170 semantic controls.

The [passing one-root construction](actual-count-transport/rust-construction/result.json)
and [initial independent Go check](actual-count-transport/go-candidate/result.json)
accept the exact final bytes. Expanded controls/model are added only after
the actual construction handle terminates. All captured Rust inputs stay fixed
through their actual terminal handles. All 255 frozen design inputs and the
official scanner rules/scopes/seals remain exact; only exact per-file original
libtest EOF observations are retained.

Variable/signed/right-shift numeric/source laws, full carrier/state/program/
bounded56-family foundation/descriptor/private registry/context and authoritative
original application source/export/selected-closure replay remain due. All 169
application demands stay unelaborated. C# tests stay skipped; the cumulative T02
gate stays W04: `check-fast.sh --skip-csharp`. No complete source-program proof,
immutable application replay or practical-profile activation is claimed.
