# Actual ordinary balanced integer foundation

This W01 component extends the freshly compiled Nat foundation with mathematical
integers. The fresh zero-axiom certificate has709 declarations,11,994 terms and
160,569 bytes. Its complete certificate hash is
`20336bfcfe4ad0e03c81bf38f35d2af86b5cf669496064387d70d58ae1a152e5`.
An integer representative is a full Bool -> Nat function: its false
projection is p, its true projection is n, and its observed integer is p-n.
Neither projection has a machine-word bound. Representatives may be redundant:
5-3 and2-0 observe the same integer. `Equivalent x y` is exactly ordinary Nat
equality `Positive x + Negative y = Positive y + Negative x`. It is never core
equality of the functions, and no function extensionality or canonical-normal-form
assumption enters the foundation.

The [integer recipes](../../../../../../crates/mpk-vc/src/go_practical_ordinary/integer.rs)
define13 ordinary members: Make, Positive, Negative, Zero, FromNat, Negate, Add,
Sub, Mul, Equivalent, Equal, Less and LessEqual. Fourteen checked projection laws
expose their actual computation over arbitrary values. Further checked ordinary
theorems establish the equivalence relation, additive inverse and ring laws,
representative-independent arithmetic and all three Boolean observations,
Boolean equality reflection/completeness, representation shifts and injective
natural embedding. All premises remain in the checked theorem and checkpoint
types. There are42 named library laws and566 checked Nat-equality checkpoints,
in addition to13 definitions and the unchanged88-declaration Nat basis.
Integers are mathematical contract values; Go fixed-width word/source
arithmetic and conversions still need their separate bridges.

The [polynomial proof constructor](../../../../../../crates/mpk-vc/src/go_practical_ordinary/polynomial.rs)
constructs explicit applications of already checked Nat associativity,
commutativity, distributivity, zero and unit laws. A deterministic syntax order
only selects a sequence of these applications. Named ordinary checkpoints keep
the complete local variables and actual equality premises. A matching host
normal form supplies no fact: both cores must typecheck every emitted proof.
A failed search or existing byte/term/declaration generation limit supplies no
partial component or application success. There is no new trusted evaluator,
kernel primitive, opcode, axiom, conversion, eta or fuel rule.

The exact importer regenerates all complete bytes. Independent acceptance of an
honestly different zero-axiom component is insufficient for exact import. The
complete preceding Nat component retains its original certificate hash. The
private named syntax adds only Sort0 construction for the semantic relation;
it still accepts compiled-in recipes and requires lexical variable binding.
No prototype certificate, source/context graph or evidence receipt is promoted.

The [selected execution](final/rust-tests/result.json),
[paired current-checker observations](paired-final/result.json),
[strict Clippy](final/clippy/result.json), [format](final/format/result.json)
and [verification record](verification.json) contain the actual outcomes.
All four isolated Rust roots pass; both current checkers agree on all15 same-byte
candidates, comprising four zero-axiom acceptances with identical full hashes and
eleven core type-mismatch rejections. Strict Clippy and format pass.
The isolated tests exercise complete ordinary proofs, deterministic exact import,
unchanged Nat bytes/hash, three redundant positive/negative/zero representations,
and fully reencoded controls. Those alter Make/projection/equivalence, positive-only
equality, incomplete multiplication and premise-dependent arithmetic/observations.
A separate attempted function equality between5-3 and2-0 must reject even though
their semantic integer equality holds. Generic core type mismatches, construction
failures, configuration rejection and source counterexamples remain distinct.

The [first execution](initial/rust-tests/result.json) retains every exact input,
raw diagnostic and emitted candidate. Its natural-embedding injection proof
incorrectly relies on automatic conversion of computed zero projections inside
Nat addition. The corrected proof explicitly composes checked projection rules,
Nat additive identity, congruence and transitivity. Both kernels remain unchanged.
Initial negative-test passes are not counted as rejection evidence because that
initial baseline itself failed. Only controls checked after the corrected full
baseline passes establish the final negative matrix.

All255 frozen design inputs remain unchanged, as recorded by the
[preservation observation](frozen-design-preservation.json). The changed-input
review scans every original byte with the unchanged scanner and historical seal
policy; original failed outputs and raw formatter EOF observations remain exact.
Affected integration tests and lint/format are local. Unchanged core/Go/legacy
matrices are retained without repetition. No C# tests execute. The cumulative
T02 gate stays deferred to W04: `check-fast.sh --skip-csharp`.

Half-open logical range folds, word/carrier bridges, complete source/storage/
operation/lifetime/program laws and the full56-family foundation/descriptor/
private context/registry remain due. W01 stays In progress, and all169 application
theorem demands remain unelaborated. No public practical profile, source-program
proof or native Linux acceptance is claimed.
