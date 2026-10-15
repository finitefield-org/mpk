# Source-shaped order/subtraction and addition guards

This private W03 experiment proves arithmetic for the continuing step of the
design's `Quote` loop. The [producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/order.rs)
and [path layer](../../../../probes/go-practical/certificates/src/word/balanced/signed/order/path.rs)
define actual modular subtraction, signed comparison and full-word addition.
They do not complete a Go frontend/source binding or a whole-function proof.
The [measurement report](observations.json) records actual independent
Rust/Go checker outcomes; empirical Go observations have a separate scope.

## Operation and carry model

For a full word of width `w`, `not_w` complements every bit. It is distinct
from the earlier pilot's low-bit mutation. Define subtraction and comparison
by ordinary Bool-cube operations:

```text
sub(l,a)       = sum_w(l, not_w(a), incoming=true)
unsignedLE(a,b)= carry_w(b, not_w(a), incoming=true)
signedLE(a,b)  = if sign(a) == sign(b) then unsignedLE(a,b) else sign(a)
```

The subtraction result wraps at the actual word width. Checked complement
projections/sign binding, adder projections and carry/sign binding link the
same values throughout the proof. No arithmetic success oracle is assumed.

Four interacting carry chains are evaluated at each bit:

```text
D = l + not(a) + d        // remaining amount
E = D + not(x) + e        // remaining amount minus next item
R = a + x + r            // actual accumulator update
T = l + not(R) + t        // upper-bound comparison after update
```

The invariant `d + e + r = t + 1` has six Boolean carry states. Its local
preservation theorem splits seven local flags: three operand bits and four
incoming carries. It never enumerates complete operand words. Checked
half-word projection lemmas compose preservation through all bits. With
initial carries `(true,true,false,true)`, final `d=e=true` implies final
`r=false,t=true`: the update does not wrap and its unsigned result is within
the limit.

## Signed continuing-path claims

The source-shaped assumptions are exactly these modeled loop-state and
continuing-branch conditions:

```text
sign(limit) = false
sign(accumulator) = false
signedLE(accumulator, limit) = true
sign(item) = false
signedLE(item, sub(limit, accumulator)) = true
```

The proof derives nonnegative remaining amount, actual subtraction/comparison
carries, unsigned no-wrap, a nonnegative updated accumulator, its signed
upper bound, and the actual sign-stage incoming/outgoing carry agreement.
The last predicate is the premise consumed by the previously checked
[signed prefix theorem](../signed-prefix-sums/README.md). It is derived
here from these comparison conditions rather than supplied as an unrelated
success flag. Connecting each Go branch and loop invariant to the model
remains a frontend/VC/loop obligation; these assumptions are not silently
added to the application entry contract.

A further genuine `Std.Nat` theorem uses the derived no-wrap result with
the checked word addition theorem. For every initial natural `z`, it proves

```text
V(succ, accumulator)(V(succ, item)(z))
  = V(succ, sum_w(accumulator,item,false))(z)
```

All three full operands remain universally quantified. The sum is exact
and unbounded in the mathematical domain. Generic checked pointwise
composition/identity lemmas avoid unfolding enormous unary values or
assuming function eta/extensionality. No axioms or checker changes are used.

## Matrix, observations and construction findings

The matrix has 40 candidates. Carry composition covers widths
`1/2/4/8/16/32/64/128`; path and natural-addition claims cover actual Go
signed widths `8/16/32/64`. Width128 is an internal arithmetic probe, not
Go source integer admission. Every group has two fresh identical generations,
with the same canonical bytes checked by Rust and Go. Sixteen claims pass
with zero axioms and matching hashes; 24 deliberate premise/operand
substitutions fail at core type checking. No fuel outcome is counted as a
semantic negative.

The 64-bit path positive has 300 declarations, 20,459 terms and 233,086 bytes;
its exact-natural counterpart has 20,387 terms and 232,314 bytes. Each
quantifies all 192 operand bits, and the latter quantifies its initial
natural. Per-group construction, per-checker wall/CPU time and peak resident
bytes, source/binary identities and pinned toolchain inventory are in the
report. Measurements are Darwin/arm64 observations, not frozen application
budgets or native Linux release evidence.

The [pinned Go observer](../../../../probes/go-practical/order_reference.go)
executes actual typed `int8/int16/int32/int64` subtraction, addition and
comparisons on 3,428 deterministic cases. Each width has 729 boundary/sign
triples and 128 seeded full-range triples. The
[input bytes](go-reference-input.json) and [actual output bytes](go-reference-output.json)
are compared with an independent bit-loop model. Inputs outside the
continuing path remain in the observations, including negative values and
wrapping arithmetic. The observer is tooling, not admitted profile source.
This empirical comparison does not substitute for a universal theorem or
prove that a production frontend binds Go operators to these definitions.

Two untrusted construction findings were fixed before retained measurement:
dependent Boolean facts must be rewritten before their carry subexpressions,
and case identifiers must include the family to prevent a negative path
file from being overwritten by a natural-mode case. The runner now rejects
duplicate IDs and any mismatch between the theorem index and actual file
set. Neither correction changes the intended claims or their premises.

## Validation and remaining scope

`run_order_probes.py --check` rebuilds and replays both proof checkers and
the Go observations. `check_order_report.py` checks retained integrity only.
Adversarial tests cover lost full operands/initial naturals, weakened path
premises, theorem/byte/hash/source crossing, missing checkers, axioms,
resource misclassification, empirical-proof confusion and omitted negative
Go observations.

Targeted checks cover the new arithmetic/path producer, source-shaped
conditions, actual typed Go observation tool and evidence helpers. Earlier
private proof families are remeasured because they share the changed
producer; their certificate bytes must stay unchanged. Production identities
are checked by the W01 audit instead of unrelated production-suite reruns.
The whole T01 gate remains deferred to W05.

W03 remains In progress. Complete source/model/operation bindings, actual
`Quote` loop/prefix/return composition, collection/descriptor/control and
byte-operation law closure, and aggregate boundary/resource freeze stay
open. W04 has not started; no practical public profile is active.
