# Actual word addition and unbounded numeric conservation

This W01 component gives the actual8/16/32/64-bit word circuits balanced
unbounded Nat values and conditional mathematical Int addition laws. The
complete zero-axiom certificate has2579 declarations,55,105 terms and743,954
bytes, with certificate hash
`4cbb139e28e35e6b62d69f412081135480b83a172f16bce1a68ca0c7f8640597`.
All1892 preceding word declarations and31,104 term nodes are retained; the
complete regenerated [word component](../word-foundation/README.md) keeps its
exact original433,285 bytes and three hashes. All255 frozen design inputs
remain unchanged. No old prototype certificate, graph, context or receipt is
imported as foundation authority.

The [fresh ordinary producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric.rs)
defines a Bool's Nat value as0/1. At depth0 the shared weight is2; each next
weight squares its predecessor. A word's value is its low half's value plus
the preceding weight times its high half's value. Depths3/4/5/6 therefore give
the actual full8/16/32/64-bit values and2^width moduli. All arithmetic is the
preceding genuine Std.Nat library. Shared ordinary definitions avoid a giant
unary literal. The fresh scalar recipe differs from the frozen probe's
function-composition recipe; it proves the same mathematical conservation
equations with current checked Nat operations, without promoting that seed.

Pointwise value congruence is checked structurally at all seven depths. It
needs equality of every addressed bit, rather than equality of word functions.
The proof assumes neither function extensionality nor eta. A local full-adder
law uses Bool elimination only on its three flags. A checked general Nat
semiring combination then composes low/high interval laws. Each of236 aligned
binary blocks is tied to the actual AddCarry result and actual ripple prefix
carries. The240 sum-bit/carry-step projection laws expose exactly the existing
circuit definitions. Entire operand values are universally quantified and
never enumerated.

For arbitrary words, incoming Bool and unbounded initial Nat, the resulting
ordinary theorem states the exact scalar equation:

```text
initial + (unsigned(x) + unsigned(y) + incoming)
  = initial + (unsigned(AddCarry(x,y,incoming)) + 2^width * final_carry)
```

The [signed producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed.rs)
uses the highest bit of those same operands and result. Under equality of the
actual sign-stage incoming carry and actual final carry, it proves:

```text
initial + (unsigned(x) + unsigned(y) + incoming + 2^width * sign(result))
  = initial + (unsigned(result) + 2^width * (sign(x) + sign(y)))
```

SignAddition derives the required local sign equation by genuine Nat
cancellation from the full-adder law. NoOverflow is the Boolean equality of
the actual two carries; a checked reflection theorem converts its true result
to that equality premise. The Signed interpretation is the existing balanced
Int.Make(unsigned, modulus * sign) representative. Explicit positive/negative
projection laws lift the checked cross sum to Int.Equivalent of the actual
signed operand sum and actual AddCarry result. Int equality remains semantic
cross-sum equivalence. Go's word addition keeps its modular result when this
extra mathematical no-overflow theorem is unavailable.

Every rewrite is an ordinary checked Nat law or Eq.congr/trans/symm proof.
Polynomial checkpoints have a separate compiled numeric namespace; the old
integer checkpoint bytes remain exact. Reusing untrusted Boolean definition
recipes supplies no fact: both current cores check all emitted terms against
the actual definitions. No kernel, reduction, axiom, conversion fuel or
construction bound changes. Exact numeric import regenerates all bytes and
rejects altered bytes, oversized inputs and honestly different valid components.

The [four scoped Rust roots](final/rust-tests/result.json) pass. The
[current-checker matrix](paired-final-reviewed/result.json) checks the same26
complete byte streams in both unchanged independent checkers. Three accept
with zero axioms and identical certificate/export/axiom hashes. All23 wrong
Bit/Weight/Value, actual sum/carry projection, ignored-overflow-premise and
dropped-final-carry controls reject as core type mismatches. Four carry
omission controls include both32- and64-bit words; none is counted through a
resource or canonical-format failure. Fifteen negatives are conclusion type
mismatches and eight are lambda-domain type mismatches. Strict
[Clippy](final/clippy/result.json) and [changed-file format](final/format/result.json)
pass. C# tests and unrelated older matrices are skipped.

Both original [unsigned](initial/rust-tests/result.json) and
[signed](signed/rust-tests/result.json) smoke checks pass. The first comparison
[consumer execution](paired-final/result.json) stops after assuming every core
type failure has the single conclusion-mismatch subtype. Its actual Rust/Go
observations both reject the original SignedAddition-16 bytes with a lambda
domain mismatch. The [focused reproduction](initial/paired-consumer/reproduction.json)
preserves that original assertion failure and raw diagnostics. The repaired
comparison accepts only the two precise current error forms, verifies all26
unchanged candidates and changes no producer, proof, checker or Rust test input.

The [actual decoded-certificate Boolean interpreter](../../../../../../crates/mpk-vc/tests/support/go_word_numeric_observations.rs)
executes all288 unchanged frozen numeric queries: actual AddCarry values,
actual carries and the actual NoOverflow definition. It executes19,103,859
term/application steps, with203,856 for the largest query. Its only value
primitives remain Bool constructors/recursors and assigned input-word bits.
The [native run](native/result.json) compiles the original independent Go
observer using a fresh empty cache and an isolated pinned5577-file prepared
image, checked completely before and after. Two native byte streams match.
Every actual circuit result/guard agrees with independent exact Python
arithmetic and native typed Go/math-bits/math-big
[observations](native-comparison-final/result.json), including initial Nat
10^50+37. The210 no-overflow cases satisfy the signed cross sum;78 overflow
cases do not. Python derives the displayed scalar sums from the original
inputs and actual circuit results. This finite comparison does not execute
2^64 unary successors or replace the separately checked universal Nat/Int laws.

The [verification record](verification.json) retains original inputs, commands,
outcomes, times and selection reasons. The unchanged strict scanner checks all
changed bytes with its39 exact historical seals and fixtures. Only three exact
hash/length-bound original libtest stdout EOF observations are recorded, with
no source/JSON/directory or blanket whitespace exception. The cumulative T02
gate stays deferred to W04: `check-fast.sh --skip-csharp`.

General numeric correctness for multiplication/division/shifts, source-typed
operation/nominal binding, carriers/strings/slices/storage/lifetimes/definedness,
program assembly and complete56-family descriptor/private registry/context
remain due. All169 Go application theorem demands remain unelaborated. W01
stays In progress; no public profile, Go source/program proof, authoritative
immutable application replay or native Linux release gate is claimed.
