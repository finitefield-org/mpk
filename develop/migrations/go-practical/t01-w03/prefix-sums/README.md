# Ordinary exact prefix-sum ingredients

This private T01-W03 experiment relates a real left-to-right 64-bit modular
accumulator to an unbounded `Std.Nat` sum. It does not complete W03, establish
the signed/nonnegative `Quote` invariant, discharge a Go source guard, or
admit a public profile. The [producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/fold.rs)
and [actual observations](observations.json) own the construction and evidence.

## Model and claims

Every payload is a full 64-bit Bool-addressed cube. A prefix of count `n`
uses the next power-of-two backing capacity, with every padding bit still
universally quantified. The accumulator and initial natural are also
arbitrary. Empty prefixes read no values; their one unused backing cell is
only a private carrier choice, not a statement about Go empty-slice capacity.

Let `V(f,x)` and `P(f)` be the previously checked full-word natural transformer
and its `2^64` place transformer. At `f = Std.Nat.succ`, `V(f,x)(z)` is the
unsigned value of `x` plus `z`; `P(f)(z)` adds `2^64` to `z`. Composition and
the Bool selector `maybe` are ordinary closed definitions. No unary value is
materialized and no function-extensionality axiom is used.

```text
F(0, a, xs)       = a
F(1, a, xs)       = add64(a, xs[0], incoming=false).sum
F(n, a, xs)       = F(high, F(low, a, xs.low), xs.high)

M(0, f, xs)       = identity
M(1, f, xs)       = V(f, xs[0])
M(n, f, xs)       = compose(M(low, f, xs.low), M(high, f, xs.high))

O(0, f, a, xs)    = identity
O(1, f, a, xs)    = maybe(P(f), add64(a, xs[0], false).carry)
O(n, f, a, xs)    = compose(O(low, f, a, xs.low),
                           O(high, f, F(low, a, xs.low), xs.high))
```

`F` threads the actual low-prefix result into the high-prefix execution.
The high-prefix carries in `O` use that same state. Partial trees process
exactly `n` elements; they never zero, truncate, or read unused input values.
Each node has checked fold/value, mathematical-sum and overflow-definition
equalities. Checked projection lemmas transport its actual carry observations
to each child. Composition/commutation lemmas establish the universal law
for every natural transformer `f`:

```text
compose(V(f, a), M(n, f, xs))
  = compose(V(f, F(n, a, xs)), O(n, f, a, xs))
```

Its final pointwise theorem specializes `f` to `Std.Nat.succ` and quantifies
every initial natural. Thus the exact unsigned sum includes `2^64` for each
actual wrapping update. It has no safety premise. A separate theorem proves
the equality without `O` under the premise that **each actual intermediate
unsigned carry** is false. The premise is an ordinary quantified equality
over the computed carry cube, not a callee-correctness promise.

This unsigned condition differs from signed no-overflow. Negative signed
values remain full bit patterns in the input domain, but this experiment
interprets them as unsigned values. Signed prefix interpretation, the
nonnegative `Quote` path, order/subtraction laws and actual source-to-model
bindings still require checked evidence. They cannot be inferred from the
existence of either prefix theorem.

## Matrix and evidence

The runner measures counts `0/1/3/8/32/127/128/129`, including partial trees
and the candidate slice boundary. Count129 is an internal boundary probe,
not a frozen supported size. At count128, all 8,192 storage bits and all
64 accumulator bits are quantified; at count129, the full 256-cell padded
store has 16,384 quantified bits. Payload patterns are never enumerated.

Both fresh generations are byte-identical for all 55 candidates. Both Rust
and Go accept 18 zero-axiom claims with matching export, certificate and
axiom-report hashes. These include the genuinely valid empty-prefix controls:
its carry premise is vacuous, and its overflow correction is the identity.
Twenty-three altered claims fail at core type checking. Fourteen candidates
that drop the last summand or overflow correction reach conversion fuel;
they are resource findings, not semantic negatives or certified counterexamples.

The 128-element conditional/unconditional claims each have 11,549 terms and
355 declarations, in 138,756/138,764 bytes. Exact per-child construction,
Rust/Go wall and CPU time, peak resident bytes and binary/source identities
are retained in the report. These are Darwin/arm64 development measurements,
not a frozen profile budget or native Linux release evidence.

## Original feasibility finding

The [original empty-prefix positive](../prefix-initial/observations.json)
exhausted conversion fuel in both checkers. Its exact source snapshots and
certificate are retained. The untrusted producer had tried to reduce the
whole 64-bit value transformer to establish a right-identity law.

The correction proves right identity for a known composite function and
lifts the word's checked definition through congruence/transitivity. It
does not assume `compose(f, identity) = f` for an arbitrary function by eta
conversion, change checker fuel, add an axiom, or shrink the payload domain.
The new raw theorem additionally proves that mathematical sums and actual
overflow corrections preserve commutation and composes the same checked
addition theorem across all actual intermediate states.

## Validation and remaining scope

Replay uses `run_prefix_probes.py --check`; retained-evidence integrity uses
`check_prefix_report.py`, with adversarial tests in `test_prefix_report.py`.
The integrity check is not proof replay. The runner builds the private
producer and independent Go checker with downloads disabled and verifies
the pinned Go host tree before construction. Each checker receives the
same canonical bytes. Intermediate incomplete runs retain an explicit
`attempt.json` instead of a completion claim.

Targeted verification covers the new prefix producer, actual carry/state
bindings, full payload/padding domain, altered claims, deterministic replay
and evidence helpers. Earlier private representation, local-Nat and balanced
word pilots are remeasured because they share the changed producer binary;
their certificate bytes must remain unchanged. Production frontend/checker,
specification and registry identities remain at W01, so unrelated production
suites are not rerun. The complete T01 gate remains deferred to W05.

W03 remains In progress. The signed/nonnegative prefix and guard/order bridge,
complete source/operation bindings, collection/descriptor/control-law closure
and aggregate boundary/resource freeze remain open. W04 has not started.

The subsequent [signed prefix experiment](../signed-prefix-sums/README.md)
adds conditional two's-complement cross-sum composition over this same fold.
It still leaves Go guard/order/subtraction and the nonnegative application
invariant open; it does not change this unsigned experiment's proof scope.
