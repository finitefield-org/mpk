# Actual fixed right shifts and full64 count prerequisites

This T02-W01 checkpoint proves the actual unsigned/signed fixed-right source
circuits at widths8/16/32/64 for every power-of-two amount1..width/2. It also
proves a full64 count decomposition and high-bit zero reflection. The ordinary
certificate contains3,105 declarations,77,415 terms,1,100,109 bytes and zero
axioms, with logical hash
`a3017e8969d14663af2340b4d72b5fcec21b0d0ead6d3ad05a0b5bd7c62350a2`.
The original4096/131072/2097152 caps and1024 core conversion fuel remain.
All3 variable-shift conservation/saturation laws remain required. W01 is In
progress; no complete source/program/application/profile authority is claimed.

The private [original-name composer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/composition.rs)
retains every one of the1,882 previous conversion declarations and783 selected
original arithmetic/source-shift dependencies. The existing Builder seed adds
four exact original Std.Bool definitions(and/if/not/or), making2,669 retained
originals and436 new shift declarations. The original
[2665-root measurement](prerequisite-analysis/result.json) remains intact;
[complete metadata comparison](retained-identities.json) includes all four seed
entries. Every old type/body/proof/level/family identity matches without rename
or substitution. The prior conversion raw hash and the existing signed-division
composer raw hash also remain exact. Metadata comparisons and private exact
owning import checks supply no core/source/application proof authority.

The [power proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/powers.rs)
use genuine unbounded Nat induction. PowerMinusOne(n)+1=PowerTwo(n), exponent
addition and positivity hold for arbitrary Nat exponents. Small closed powers
only use small indices. No huge unary64-bit count or2^(huge count) is evaluated.

A [balanced PopHigh cube](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/pop.rs)
removes the lowest bit and inserts a supplied highest bit. A generic Nat
composition proof and explicit small cube equalities mapped through Value give
its conservation equation and source value bound. The
[fixed-right binding](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/fixed_right.rs)
doubles that actual operation. For M=2^width and i a fixed amount,

```text
U(x) + M*(2^i-1)*Bit(fill) = 2^i*U(FilledRight_i(x,fill)) + R_i(x)
R_i(x) = value of the actual lowest i source bits
R_i(x) < 2^i
```

All literal-address equalities bind to the original Word.Wwidth.Right_i and
SignedRight_i circuits. Unsigned fill=false gives U(x)=2^i*U(result)+R_i.
Signed fill=the actual source top gives the corresponding filled conservation
and preserves the source top. There are18 amounts and36 actual source bindings.
The full variable signed Int floor equation remains separate required work.

The [count proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/counts.rs)
use the exact balanced Or tree of source count bits d..63, where d=log2(width).
HighValue is the matching balanced sum of relatively weighted source bits;
Digits_d is the original low-prefix expansion. Ordinary prefix/power equations
and genuine unbounded Nat zero reflection prove

```text
U64(count) = Digits_d(count) + 2^d*HighValue_d(count)
Nat.Equal(HighValue_d(count),0) = Not(CountHigh_d(count))
```

Only local Bool selectors are split. Whole count/operand enumeration supplies
no theorem. The low-prefix bound, full count/width equivalence, computed excess
and symbolic large-count power identity still need to be proved before all3
actual variable stages and final saturation/sign fill can be composed.

The [verification record](verification.json) binds three unique affected roots
across a combined control/model run and a narrow metadata-root rerun. The
combined run completes101 semantic controls and the504-row Bool model, then
fails its separate original-count assertion2665 because it omits four Builder
seed definitions. The assertion and report-text count literals are corrected
to2669; only that affected root is rerun. Production proof bytes and every
control/model/query input stay exact. Current selected Clippy, Rust/Go format
and the ignored-diagnostic listing pass. No unaffected semantic root is repeated.
Both unchanged production cores inspect103 complete streams: two zero-axiom
positives and101 genuine core type rejections, including real source circuit,
remainder, bound, high-tree and arithmetic-premise mutations. An honest valid
extra declaration passes both cores but fails private owning import.

All504 original complete shift rows, their full64 signedness routes, actual
source stages/high guards and4,212 actual fixed-right observations agree with
typed Go and independent Python integers. Native Go produces24 real negative
count panics,480 defined results and309 defined high counts. One fresh pinned
5577-file prepared image and empty cache produce two byte-identical runs, with
before/after image checks. The unchanged Bool interpreter evaluates only actual
Boolean circuits. Nat/Int/witness/theorem bodies do not execute there. Large
counts compare actual zero/sign-fill outputs without materializing huge powers.
These finite observations supply no universal arithmetic theorem.

Original compiler borrow errors, the complete WeightedBitZero type failure and
both direct checker outcomes remain with their exact sources/candidates/raw
reports. An ignored prefix diagnostic identifies its unapplied Bool-case
function; explicit function application and Bool transports repair the recipe.
The original metadata-reader/count assertion and observer lambda-subtype
assumption failures also remain. The observer matcher is corrected to the real
lambda domain mismatch and its exact first inspected pair is reused. Original
native runner key error and early Go-report lookup fail before native/checker
execution; their sources/owned inputs/annotations remain. Tool-text transcripts
are labeled separately from raw OS reports. No original source, JSON or directory
is excluded from the changed-input review; only exact per-file hash/length-bound
libtest stdout EOF observations are allowed. All255 frozen design/specification/
vector inputs, official127-rule/four-scope scanner and39 historical seals remain.

The [implementation plan](implementation-notes.md) retains all3 full-count
variable shifts, full numeric/source/type/context/storage/lifetime/program and
bounded56-family descriptor/private registry work. All169 authoritative original
application demands remain unelaborated, and T02-T11 scope/gates stay active.
C# tests stay skipped; the cumulative T02 gate stays deferred to W04:
`./scripts/check-fast.sh --skip-csharp`. No native Linux release is claimed.
