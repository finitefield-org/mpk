# Ordinary unsigned-word value model

This is a private W03 feasibility model, not a frozen Go profile or a claim
that W03 is complete. Signed interpretation, source/model binding, aggregate
sum composition and the profile-wide resource freeze remain required.
Measurements are owned by `run_word_pilot.py` and its retained observations;
the equations below alone are not acceptance evidence.

## Carrier and denotation

Let `F = Std.Nat -> Std.Nat`. All following definitions are ordinary closed
certificate definitions, parameterized by functions and Boolean digits.

```text
identity(n)       = n
compose(f, g)(n)  = f(g(n))
square(f)        = compose(f, f)
maybe(f, b)      = if b then f else identity
power_0(f)       = f
power_(k+1)(f)   = power_k(square(f))
value_0(f, [])   = identity
value_(k+1)(f, b::tail)
                 = compose(maybe(f, b), value_k(square(f), tail))
```

Digits are least-significant first. Instantiating `f = Std.Nat.succ` makes
`power_k(f)` act by `2^k` successor applications. Applying
`value_k(f, bits)` to an arbitrary initial natural gives the binary digit
denotation plus that initial value. No input payload is enumerated. This
interpretation uses the actual zero-axiom `Std.Nat` constructors, rather than
an unconnected Boolean assertion that a carry is correct.

Representing these as functions lets the certificate retain shared named
definitions without expanding `2^64` unary constructors. Function equality
is checked directly; no unchecked extensionality principle is introduced.
Pointwise natural-number equality follows by checked `Std.Eq.congr` with
the context `fun operation => operation(initial)`.
These higher-order functions are internal mathematical representations;
they do not admit Go callbacks or indirect calls into the source profile.

## Exact addition statement

The producer defines the ordinary ripple equations at each digit:

```text
sum_i       = x_i xor y_i xor carry_i
carry_(i+1) = (x_i and y_i) or (carry_i and (x_i xor y_i))
```

For all root functions `f`, all complete operands and every incoming carry,
the width-`k` functional theorem states:

```text
compose(value_k(f, x), compose(value_k(f, y), maybe(f, incoming)))
  = compose(value_k(f, sum), maybe(power_k(f), final_carry))
```

The final `Claim` specializes this equality at `Std.Nat.succ` and applies it
to every initial natural. It preserves the final carried place, so it is
exact unsigned value conservation, including unsigned overflow. One mutation
drops that final place while retaining the proof candidate. The initial
32-bit measurement reaches conversion fuel in both checkers; it does not
count as a core type-mismatch negative. The source, canonical bytes and raw
result are retained in [the initial observations](word-initial/observations.json).

A separate mutation flips the low result bit while retaining the correct
carried place. The complete pilot measures both mutations and preserves
their separate classifications. A parser error, fuel failure or timeout is
not counted as false-claim evidence. The intended low-bit negatives still
require actual both-checker observations before the measurement can close.

## Proof dependency and growth

The local full-adder functional lemma splits only its three Boolean flags.
All width-dependent theorems use earlier checked lemmas:

1. Ordinary function composition is associative. A function commutes with
   its square, and commutation is preserved by products, squares and the
   Boolean `maybe` selector.
2. `value_k(f, bits)` commutes with any function that commutes with `f`.
   Its step uses the preceding width's theorem at `square(f)`.
3. The addition step unfolds only the current value definition, exchanges
   the high-part transformers with the low-digit selectors, applies the
   local full adder, and uses the preceding addition theorem on the high
   digits. Explicit congruence and transitivity connect the steps.
4. The pointwise natural claim follows from the resulting function equality.

Each width contributes two definitions and four theorems. The self-contained
positive/negative candidate has `39 + 6*k` declarations, including the Bool,
Eq and Nat foundations and its final claim. Input quantification has `2*k`
operand bits plus an incoming carry and initial natural. Width induction is
performed by the untrusted producer constructing checked theorem
applications, not by assuming induction or enumerating operand bit patterns.

The existing checker rules, Certificate v0 transport and four axiom
categories remain unchanged. Every declaration is independently checked by
the Rust and Go implementations. The builder, source hashes, reported term
counts and this explanation cannot substitute for that verification.

## Remaining bridge work

This statement is still insufficient for a practical Go application theorem.
The next layers must establish the two's-complement signed interpretation,
the precise no-overflow relation, source operation/result binding, and
bounded prefix sums using the widened exact domain. They must also measure
the actual aggregate proof shapes and boundary workloads. No frozen bound
may be inferred just because a short-width candidate succeeds.

The next signed construction can reuse the exact unsigned result without
changing the source width. For a width-`k` two's-complement payload `x`, its
signed denotation is the balanced difference
`unsigned(x) - 2^k * sign(x)`. A candidate ordinary representation stores the
positive and negative natural denotations separately; integer equality is a
checked equality of cross sums. This is a representation choice to validate,
not an assumption that the arithmetic library is already present.

For an addition result `r` and final unsigned carry `c`, the unsigned theorem
gives `unsigned(x) + unsigned(y) = unsigned(r) + 2^k*c`. The remaining signed
step must prove, under the exact signed-no-overflow predicate,
`c + sign(r) = sign(x) + sign(y)`, then compose that equality with the unsigned
theorem. The sign-stage relation is a three-input Boolean/full-adder lemma;
it must be instantiated with the actual incoming and outgoing sign-bit
carries. It cannot be accepted as an unrelated flag assumption. Overflowing
Go addition retains its modular result and must not acquire a mathematical
integer-addition claim.

For `Quote`, the processed-prefix sum additionally needs a widened exact
domain large enough for the full bounded input, a checked relation from each
64-bit amount into that domain, per-step sum preservation, and the actual
guard/order/subtraction relations that rule out overflow on successful
paths. Natural denotation, signed equality, ordering and sum accumulation
are separate obligations. No one of those constructions closes the others.
