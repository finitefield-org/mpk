# Actual word bounds, unsigned order, equality and divisor guards

This W01 component connects actual8/16/32/64-bit Boolean word circuits to
genuine unbounded Nat values, order and equality. The fresh ordinary certificate
has3437 declarations,71,143 terms and985,945 bytes, zero axioms and certificate
hash `1e4f59ae52bb6e2d774ede00d039d833488ded5274e3104ed0e3e7f751983b45`.
All2579 preceding numeric declarations and55,105 term nodes remain exact, as
do its complete743,954 bytes. The new root also regenerates the preceding
[arithmetic component](../word-arithmetic/README.md) and checks its complete
original byte hash. All255 frozen design inputs and all candidate artifact
bounds remain unchanged. No prototype seed, graph, receipt or context supplies
foundation authority.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/order.rs)
uses a disjoint ordinary namespace. Its checked Nat polynomial checkpoints do
not collide with the existing arithmetic component. The new owning-byte API
regenerates the complete component on import and rejects changed, oversized,
other retained and honestly different valid components.

The [complement and bound proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/order/bounds.rs)
use direct pointwise complements and a balanced low/high block decomposition.
Word projection/definition equalities lift explicitly through Value before
scalar arithmetic. Local Bool cases establish the one-bit base, and ordinary
Nat algebra composes the seven depths. The actual Word.Not result supplies the
strict gap; callers do not provide a bound witness:

```text
U(x) + U(actual Not(x)) + 1 = modulus
Nat.Less(U(x), modulus) = true
```

The [subtraction and order proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/order/compare.rs)
combine that complement with the unchanged actual full-adder conservation.
They use the actual Sub result and actual final carry of x+Not(y)+1:

```text
U(x) + modulus
  = (U(actual Sub(x,y)) + U(y)) + modulus*Bit(actual carry)
actual UnsignedLess(x,y) = Nat.Less(U(x),U(y))
actual UnsignedLessEqual(x,y) = Nat.LessEqual(U(x),U(y))
```

The generic order lemma requires the strict result bound and conservation
equation. Each word theorem supplies both from actual named checked laws.
Bool elimination concerns the outgoing carry only; no full word is enumerated.
Nat cancellation and translated order handle its two cases.

The [equality proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/order/equality.rs)
derive a local opposite-carry identity using four Bool flags. Balanced236-block
proofs then connect the conjunction of opposite actual subtraction carries to
the actual bitwise Word.Equal. A separate checked fold correspondence for each
block connects its named definition to the literal equality tree. A genuine
[double Nat induction](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/order/natural.rs)
proves that neither value being less than the other is exactly Nat.Equal.
Consequently actual Word.Equal and Word.NotEqual reflect genuine Nat equality
and its negation for arbitrary full words.

The [zero and definedness proofs](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/order/zero.rs)
establish that actual Word.Zero has value0 and actual IsZero equals
Nat.Equal(value,0). Genuine Nat induction connects nonzero equality to positive
order. Actual DivideDefined therefore equals Nat.Less(0,value), and its actual
equality-to-true premise implies positive value. This supplies a proved source
division guard, not a caller-supplied true flag or division/remainder result law.

The [current scoped execution](corrected/rust-tests/result.json) passes all four
affected roots. The [same-byte matrix](paired-corrected/result.json) checks31
complete streams with both unchanged independent production cores. Three
accept with zero axioms and identical certificate/export/axiom hashes. All28
wrong complement/order/carry/bounds/subtraction/equality/zero/guard controls
reject as actual core type mismatches:24 conclusion mismatches and four
lambda-domain mismatches. Every width includes a substituted unrelated true
guard. No format, fuel, resource or diagnostic-prefix outcome counts as a
semantic negative. Current strict [Clippy](corrected/clippy/result.json) and
[Rust format](corrected/format/result.json) pass. The unchanged Go observer does
not require a new formatting check.

The [actual Boolean-term model](../../../../../../crates/mpk-vc/tests/support/go_word_order_observations.rs)
selects all272 relevant original frozen rows, preserving every original input
field and full hex word. It executes actual Not/Sub, both subtraction carries,
Equal/NotEqual/comparisons/IsZero/DivideDefined from the same decoded certificate.
Only Bool constructors/recursors and assigned input-word bits are primitives;
there is no arithmetic model callback. It observes52 zero divisors and85 borrows
in19,642,573 total term/application steps, with218,351 at the largest query.
Scalar bounds, conservation and order meanings use independent host integers.
The model does not execute enormous unary Nat values, order definitions or
theorem bodies; universal genuine Nat proofs check separately in both cores.

A [fresh native build](native/result.json) executes the unchanged original Go
word observer under an isolated, exact5577-file prepared compiler image with
empty cache and complete before/after checks. Both native outputs match byte
for byte. All80 word and192 Boolean results agree with the actual Boolean DAG
and an independent Python integer [oracle](native-comparison-corrected/result.json).
The original observer does not expose internal carry/zero/divisor guards;
those additional actual bits agree with independent host meanings. The
[origins](query-origins.json) bind the complete original1640-row input and all272
selected full rows. Finite native comparisons supply neither source/program
proof nor authoritative immutable application replay or Linux release evidence.

The original [bounds/order smoke](initial/rust-construction/result.json) passes
before equality/zero guards are attached. The first complete
[candidate](equality/rust-construction/result.json) fits all frozen bounds but
exhausts conversion fuel in [both cores](equality/go-candidate/result.json).
The [prefix diagnosis](equality-first-declaration/rust-diagnostic/result.json)
locates W64.CarryPairEquality. Explicit small block folds repair the large
implicit root conversion; [all-width construction](equality-explicit-blocks/rust-construction/result.json)
and [Go inspection](equality-explicit-blocks/go-candidate/result.json) accept
exactly the final bytes. No checker rule, axiom, fuel or artifact bound changes.
Prefix checks are diagnostic in-memory observations, not complete canonical
certificate acceptances. The exact [temporary helper](diagnostics/declaration-prefix-harness.rs)
is archived and removed from active test discovery.

The first [final harness compilation](final/rust-tests/result.json) reports
borrowed temporary arrays in the new observation helper; no test root executes.
Named argument storage fixes that helper, and the full affected selection then
passes. Exact original sources and raw diagnostics remain. A preserved
unconnected control draft is source context only, with no execution claim.
Strict changed-input review permits only five exact hash/length-bound original
libtest EOF observations, with no source/JSON/directory or blanket whitespace
exclusion. The [verification record](verification.json) binds all these scopes.

Actual division/remainder/modulo laws, variable/signed/right shifts, signed Int
multiplication and complete source-typed operation correspondence remain due.
Full representations/storage/lifetimes/program foundation, bounded complete
assembly,56-family descriptor/private registry/context and authoritative original
application source/export/selected-closure replay also remain due. All169 Go
application demands remain unelaborated. W01 stays In progress, and no public
profile activates. C# tests are skipped; the cumulative T02 gate stays deferred
to W04: `check-fast.sh --skip-csharp`.
