# Balanced unsigned and conditional signed word bridges

Status: measured private W03 word library. W03 remains In progress. Exact
prefix-sum composition, Go operation/source bindings, collection/control laws
and the aggregate/boundary freeze remain required.

[Actual observations](observations.json) cover widths 1/2/4/8/16/32/64/128 for
both unsigned and conditional signed arithmetic, with three candidates per
width. Both fresh generations are byte-identical in all 16 groups. The same
canonical bytes are independently checked by Rust and Go for all 48 cases:

- 16 universal or explicitly conditional claims are accepted with zero axioms
  and matching export/certificate/axiom-report hashes.
- 25 incorrect result/projection or premise candidates fail at core type
  checking in both checkers.
- Seven deep offset-removal candidates exhaust conversion fuel in both
  checkers. They are failed candidates, not type-mismatch negatives or
  certified counterexamples. They remain a separate recorded outcome.

The 128-bit cases probe an internal widened mathematical domain. They do not
admit a Go 128-bit source integer or prove the source64-to-widened-sum binding.
No practical Go profile, normative shared artifact or registry is changed.

## Operand carrier and exact unsigned value

Depth `d` has width `2^d`. An operand has type `Bool -> ... -> Bool` with `d`
address arguments. It represents every complete word of that width. The
first address argument selects the lower or upper half; the all-false
address is the least-significant value bit. Two operands quantify all
`2 * 2^d` payload bits without enumerating their value patterns.

Let `F = Std.Nat -> Std.Nat` and `compose(f,g)(n) = f(g(n))`.
`maybe(f,b)` is `f` or the identity according to `b`. Define:

```text
weight_0(f) = compose(f,f)
weight_d(f) = weight_(d-1)(weight_(d-1)(f))
value_0(f,x) = maybe(f,x)
value_d(f,x) = compose(value_(d-1)(f, x(false)),
                       value_(d-1)(weight_(d-1)(f), x(true)))
```

At `f = Std.Nat.succ`, `weight_d` acts by `2^(2^d)` successor applications.
`value_d(f,x)(initial)` is the exact binary denotation plus every arbitrary
initial natural. These are internal mathematical functions, not Go callback
admission.

The actual sum and carry definitions use the one-bit full adder at depth
zero, then feed the lower half's carry into the upper half. The main
functional equality is:

```text
compose(value_d(f,x), compose(value_d(f,y), maybe(f,incoming)))
  = compose(value_d(f,sum_d(x,y,incoming)),
            maybe(weight_d(f),carry_d(x,y,incoming)))
```

The final unsigned theorem applies that checked function equality at
`Std.Nat.succ` and every initial natural. It retains the final carried place.

## Signed interpretation and its actual guard

`sign_d(x)` reads the highest payload bit. The sign-stage incoming carry is
computed by the same half-word adder, passing the actual lower-half carry at
each level. Checked projection/binding lemmas connect both the result sign
and final carry to the local full-adder equations at the sign stage.

Interpret a two's-complement word as the balanced natural difference
`unsigned(x) - 2^width * sign(x)`. The conditional signed theorem requires the
actual sign-stage incoming and outgoing carries to agree. Under that
predicate the local bit lemma proves
`outgoing + sign(result) = sign(x) + sign(y)`. The word theorem combines this
with unsigned value conservation and proves equality of the natural cross
sums for the balanced differences. It allows negative operands and results;
it does not identify unsigned and signed values.

The final signed claim fixes ordinary addition's incoming carry to zero and
quantifies its actual no-overflow equality as an explicit hypothesis. This
is an additional mathematical-addition claim under a guard. Go's overflowing
addition retains its modular result. A Go application must still discharge
that guard from its own source path and contract. The mutation replacing the
actual guard by an unrelated true equality rejects at core checking for
every measured width. A result-sign-offset mutation is also retained, with
type-mismatch and fuel outcomes distinguished.

No semantic axiom, unchecked source adapter, callee-correctness assumption,
new checker rule or function extensionality principle is introduced. The
foundations are the existing Bool, Eq and Nat declarations, checked within
each self-contained ordinary certificate.

## Checked projections and predecessor findings

The [first balanced attempt](../tree-initial/observations.json) reaches
conversion fuel at a 16-bit positive claim when whole composed definitions
are treated as one reflexive conversion. The
[next attempt](../tree-initial/projection-attempt/observations.json) separates
low/high sum projections and passes 16 bits, then reaches fuel in the 32-bit
addition proof's final carry conversion. Exact producer snapshots and raw
results are retained for both findings.

The final producer adds checked sum-projection and carry-definition lemmas,
then lifts those equalities through the value/selector contexts using
`Std.Eq.congr` and explicit transitivity. It retains the original full word
statements. It changes no conversion budget or checker implementation.

## Observed resource scope

The following local observations use Rust release optimization and the
pinned Go 1.25.0 host compiler. RSS is direct-child Darwin `wait4` usage.
The report identifies Darwin/arm64 and CPU count; unavailable model/RAM
remain unknown. Construction measurements cover a batch of all three
candidates and their shared library, not an invented per-certificate cost.

| Claim | Terms | Bytes | Rust check | Go check | Rust peak RSS | Go peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Unsigned64 | 5,634 | 65,068 | 0.126 s | 0.266 s | 11,452,416 | 24,969,216 |
| Signed64 under actual guard | 7,741 | 89,106 | 0.186 s | 0.353 s | 11,665,408 | 28,164,096 |
| Unsigned128 internal | 6,392 | 73,951 | 0.138 s | 0.311 s | 10,354,688 | 27,131,904 |
| Signed128 internal under guard | 8,751 | 100,917 | 0.182 s | 0.415 s | 18,382,848 | 37,421,056 |

Unsigned64 batch construction took 0.098 s / 8,142,848 peak resident bytes;
the earlier [linear diagnostic](../word-initial/README.md) took 69.432 s /
688,193,536 bytes and checked in 198.099/476.319 s. These are observations of
private representative proofs. They freeze no application budget and do not
replace aggregate or limit-minus-one/limit/limit-plus-one measurements.

## Reproduction and targeted checks

Use the [probe commands](../../../../probes/go-practical/README.md). The full
balanced runner rebuilds locally with downloads disabled, checks the host
compiler inventory, generates every group twice and executes both checkers.
Its `--check` also compares retained canonical bytes and theorem metadata.
The retained-report helper validates evidence integrity only.

Selected validation covers the new ordinary producer, signed premise binding,
all affected private proof modes and evidence classification. The whole T01
gate remains deferred to W05. Public source admission, Go application proofs
and native Linux release gates are not claimed by these helper results.
