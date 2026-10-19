# Direct zero reflection for actual Go words

This T02-W01 prerequisite proves actual zero, divisor-definedness and zero-sign
laws for all 8/16/32/64-bit words without the general word-order carry closure.
The complete ordinary certificate has 274 declarations, 10,538 terms,
108,827 bytes and zero axioms. Its logical certificate hash is
`143d6e03cb1b2e409fff953bcdf8a6cae638876b8f838adbe4bd88d9cbfcf725`.
The frozen 4096-declaration/131072-term/2097152-byte bounds stay unchanged.
W01 remains In progress.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/zero.rs)
defines a balanced ordinary Boolean function at every cube depth 0..6:

```text
ZeroBits_0(x) = Not(x)
ZeroBits_d(x) = And(ZeroBits_(d-1)(low_half(x)),
                    ZeroBits_(d-1)(high_half(x)))
```

Every actual source bit participates. Explicit leaf congruences connect the
source IsZero equality against the actual zero word to this Not-bit tree.
Ordinary Nat zero-sum and positive-multiplier cancellation prove that the tree
equals Nat.Equal of the actual unsigned value and zero. Actual positive weights
follow from the established complement equations, without giant unary literals.
Local Bool cases and Nat induction prove natural nonzero/positive reflection.

For each actual width, the checked theorems are IsZeroBits, IsZeroNatural,
DivideDefinedNatural, DefinedDivisorPositive and ZeroSign. Their source guards
are the real IsZero/DivideDefined definitions. All 209 retained original named
types, bodies, proofs, levels and primitive-family metadata independently match
the complete established signed-bound component; 65 declarations are new.
The complete old signed-bound generator retains its exact original bytes.

The [verification record](verification.json) binds four current affected roots,
strict [Clippy](current/clippy/result.json),
[Rust format](current/format/result.json) and the shared
[Go observer format](../word-signed-division/current/go-format/result.json).
The [paired matrix](paired-final/result.json) checks 43 identical complete
streams in both unchanged production cores: two ordinary zero-axiom acceptances
with matching hashes and 41 actual core type rejections. Controls alter four
generic zero/positive laws, three definitions/reflections at each of seven
depths and four source laws at each machine width. The owning exact importer
rejects altered, oversized and honestly different valid components.

The [Bool-only model](../../../../../../crates/mpk-vc/tests/support/go_word_zero_observations.rs)
uses all 32 original full Neg input rows for zero observations: 960 actual
word bits and 96 actual IsZero/DivideDefined/ZeroBits applications, including
four zeros. The source sign counterpart comes directly from the input word.
No Nat numeric or theorem body is executed as a unary machine integer.
The [shared fresh native observation](../word-signed-division/native/result.json)
uses the exact pinned 5577-file prepared compiler image and empty cache,
with complete before/after checks and two identical runs. The
[independent comparison](../word-signed-division/native-comparison-final/result.json)
also inspects every bit independently and matches all zero observations.

The initial [fixed-source construction](direct-balanced-zero/rust-construction/result.json)
and [Go inspection](direct-balanced-zero/go-candidate/result.json) emit/check
the exact final certificate bytes. After their terminal results, the main
module gained the separate signed-division composition API; the zero
generator/import and other original producer bodies remain exact. Current
verification covers that additive API. Original sources/raw diagnostics stay
preserved; captured Rust inputs do not change during execution.

These proofs supply smaller, exact same-type theorem implementations for the
[complete signed-division prerequisites](../word-signed-division/README.md).
They do not establish signed division/remainder, a source program or immutable
application replay. All 169 application demands and the full remaining
T02-T11 scope stay due. C# tests remain skipped; the cumulative T02 gate stays
deferred to W04: `check-fast.sh --skip-csharp`. No practical-profile activation
or native Linux release acceptance is claimed.
