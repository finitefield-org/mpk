# Conditional signed prefix composition

This private T01-W03 experiment extends the
[unsigned prefix model](../prefix-sums/README.md) to two's-complement balanced
differences. Its [producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/fold/signed_prefix.rs)
and [actual observations](observations.json) establish conditional arithmetic
ingredients. They do not prove a Go application establishes the condition,
complete W03, freeze a profile limit, or enable a frontend.

## Full domain and actual guard

Every 64-bit accumulator and backing payload is universally quantified,
including negative two's-complement patterns and unused padding. The
actual modular adder/fold is unchanged. Define the negative-place transformer

```text
N(f, x) = maybe(P(f), sign64(x))
```

where `P(Std.Nat.succ)` adds `2^64` and `sign64` reads the actual highest
payload bit. The signed model is the difference between the unsigned value
and its negative place. `Ns(n,f,xs)` composes `N` for exactly the processed
input elements; it is the identity for an empty prefix.

At every update, the premise requires equality of the actual incoming and
outgoing carries at the sign stage of that same 64-bit adder. This is the
previously checked signed no-overflow predicate. A guard cube computes
those predicates from each real intermediate accumulator, with inactive
slots true. Checked projections transport a parent's observations into the
same child states used by the fold. It is not an arbitrary success flag,
nor a restriction of payloads to nonnegative values.

## Cross-sum proof

Write `F` for the actual prefix fold, `M` for the exact unsigned mathematical
sum transformer, and `O` for its actual unsigned-carry corrections. Under
all actual sign-stage guards, the new balance theorem proves

```text
compose(O(n,f,a,xs), N(f,F(n,a,xs)))
  = compose(N(f,a), Ns(n,f,xs))
```

At one element it instantiates the checked full-word sign/carry balance.
At a parent it composes the high child's balance at the **actual low-child
result**, then the low child's balance. The intermediate sign contributions
match directly; no cancellation or new logical rule is assumed.

The already checked unconditional unsigned law gives
`compose(V(f,a), M) = compose(V(f,F), O)`. Congruence, associativity and the
new balance theorem yield the signed cross-sum equality

```text
compose(compose(V(f,a), M(n,f,xs)), N(f,F(n,a,xs)))
  = compose(V(f,F(n,a,xs)), compose(N(f,a), Ns(n,f,xs)))
```

Specializing `f` to `Std.Nat.succ` and observing at every initial natural
establishes a genuine `Std.Nat` equality of the cross sums. Equivalently,
the balanced two's-complement value of the final accumulator equals the
initial balanced value plus the processed signed values, under those actual
guards. The internal exact domain is unbounded; source words remain 64-bit.
Function equality and pointwise observation use existing checked rules,
without function extensionality, integer axioms or operand enumeration.

## Measurement and failure classes

The matrix has counts `0/1/3/8/32/127/128/129` with four candidates each.
Each complete group has two byte-identical fresh generations, and both
independent checkers receive the same canonical bytes. Nine claims pass with
zero axioms and identical logical hashes. These include the empty-prefix
control with a vacuous guard. Fifteen altered claims fail at core type
checking: dropping the initial value, or replacing nonempty actual guards
by unrelated true equalities. Eight dropped-result-sign candidates exhaust
conversion fuel; they are resource findings, not semantic-negative evidence.

The 128-element positive contains 537 declarations, 17,736 terms and
214,747 bytes. Its 8,192 storage bits, 64 accumulator bits and initial natural
are universally quantified. Count129 retains the full 256-cell/16,384-bit
backing store and probes the proposed boundary; it admits no new Go capacity.
The report records individual construction/checker wall and CPU time,
peak resident bytes, pinned toolchain inventory and exact source/binary
identities on Darwin/arm64. No native Linux release or frozen application
resource budget is claimed.

## Validation and remaining work

`run_signed_prefix_probes.py --check` rebuilds and replays both checkers;
`check_signed_prefix_report.py` checks retained-evidence integrity. Its
adversarial tests reject weakened guards, reduced payloads, missing padding,
incorrect empty-negative labels, absent checker evidence, axioms, hash/source
crossing and resource outcomes relabeled as semantic negatives.

The signed and unsigned runners share the same measurement implementation
with separate commands, schemas, case identities and theorem validators.
Earlier private proof families are remeasured because they share the changed
producer and arithmetic sources; their certificates must remain byte-identical.
Production identity is checked with W01's audit; unrelated production tests
are not rerun. The whole T01 gate remains deferred to W05.

W03 remains In progress. Establishing these guards from the actual Go
`Quote` comparisons requires source-bound order/subtraction and nonnegative
invariant proofs. Complete operation/representation/collection/control laws
and aggregate boundary/resource closure also remain due. A conditional
arithmetic theorem cannot substitute for that application proof. W04 has
not started.
