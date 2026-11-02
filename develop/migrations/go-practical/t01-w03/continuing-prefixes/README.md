# Continuing-prefix bounds and zero-entry exact sums

This private W03 experiment composes the
[source-shaped step law](../order-guards/README.md) through the real
[left-to-right prefix fold](../prefix-sums/README.md). Its
[producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation.rs)
derives intermediate bounds and carries from actual item checks. It does
not assume that each intermediate state already satisfies the invariant.
The [retained report](observations.json) distinguishes proof evidence from
empirical original-Go observations. W03 and the public profile remain open.

## Actual checks and induction

For each active item, the check is

```text
not sign64(item)
  and signedLE(item, wrapping64(limit - actual_accumulator))
```

These are the successful sides of the amount checks in `Quote`. A check
cube records them at the same actual low-prefix accumulator used by the
fold. Unused backing cells have true checks and remain universally
quantified. Checked low/high projections transport the parent observations
to the children; no item value is replaced or enumerated.

The generic theorem starts with only a nonnegative limit, a nonnegative
initial accumulator within that limit, and the actual item checks for the
processed prefix. At one item it extracts the two branch facts and applies
the checked step law. At a parent it first proves the low child's result
bounds, then supplies those **derived** facts as the high child's initial
invariant. Thus it proves

- a nonnegative final accumulator within the limit;
- every actual update's unsigned carry is false;
- exact equality of the initial value plus the mathematical processed-prefix
  sum with the actual modular fold result, observed in genuine `Std.Nat`.

The carry proof is a quantified equality over the actual carry cube. It
supplies the previously checked generic prefix-sum theorem; it is not an
unrelated no-overflow promise. All 64-bit operand patterns remain in the
quantified domain. Negative or over-limit items simply fail the explicit
continuing-path condition; their early-return behavior is not proved by
these conditional prefix theorems.

## Discharging the source zero initializer

A separate entry construction uses an actual all-zero 64-bit word. Checked
zero projection/sign lemmas, the all-ones-plus-incoming-carry fact and
signed comparison establish `0 <= limit` from the limit's nonnegative sign.
The natural-value transformer of the zero word is proved to be the identity.
Checked pointwise composition/identity equalities avoid function eta or
extensionality assumptions and avoid materializing `2^64` unary values.

The zero-entry claims therefore require only a nonnegative limit and the
actual item checks. They quantify the complete backing store and initial
natural, with no arbitrary starting accumulator or per-step invariant/carry
assumption. They prove the actual zero-start fold's bounds and exact sum.
This connects the model's `total := 0` to prefix arithmetic; it does not yet
bind a production Go frontend or prove the whole function's returns/errors.

## Matrix and resources

Four families cover bounded arbitrary entry and actual zero entry, with
state and exact-natural conclusions. Counts are
`0/1/3/8/32/127/128/129`, including partial trees and boundary probes. There
are 72 candidates: 36 zero-axiom acceptances with matching Rust/Go hashes
and 36 deliberate premise/initial-value substitutions rejected at core
type checking. Empty-prefix replacements are correctly accepted as vacuous
controls. Every group has two identical fresh generations.

At count128, all 8,192 backing bits and 64 limit bits are quantified. The
bounded-entry families also quantify 64 accumulator bits. The zero-entry
natural claim has 685 declarations, 30,399 terms and 374,906 bytes. At
count129, all 16,384 bits in a 256-cell padded carrier remain quantified;
this is an internal boundary probe, not a newly admitted Go slice capacity.
Empty prefixes use one ignored carrier cell, not a claim about Go capacity.

The report records actual per-group construction and independent checker
wall/CPU time, peak resident bytes and exact source/binary identities on
Darwin/arm64. These are feasibility measurements, not frozen application
budgets or native Linux release evidence. Both checkers, their default fuel,
the source integer width and all four axiom categories remain unchanged.

## Original-Go observations and original finding

The [ordinary Go fixture](../../../../probes/go-practical/quote_reference.go)
contains the design's nominal types, closed error values, zero initializer,
range loop and early returns. The unselected observer executes that function
with 769 deterministic cases across the same counts, nil/non-nil empty
slices, empty/nonempty customer IDs, negative amounts/limits and extreme
64-bit values. Exact [inputs](quote-input.json) and
[actual outputs](quote-output.json) are retained. An independent bit-loop
model matches the 169 successes, 456 `BadInput` and 144 `LimitExceeded`
observations, including zero decisions on failure and unchanged input/nil
status. These are empirical observations, not source-free application proofs
or formal error/frame claims.

The [original positive zero-complement carry failure](../continuation-initial/README.md)
is preserved with exact source snapshots and raw checker results. Explicit
congruence transports the named zero's equality through complementation;
the correction adds no premise, axiom or checker rule.

## Targeted verification and remaining work

`run_continuation_probes.py --check` rebuilds and replays both checkers and
the original-Go observations. The retained-report checker validates integrity
only. Its adversarial tests reject oracle-like invariant premises, lost zero
entry or payloads/padding, invalid empty-negative labels, file shadowing,
axioms, missing/crossed checker evidence, resource misclassification and
empirical observations promoted to universal proofs.

The changed shared runner also checks exact file/index coverage and disables
Go VCS stamping with `-buildvcs=false`, so Go builds do not perform automatic
Git status operations. Original historical helper bytes are archived under
their original source pins. Earlier private proof families are remeasured
because they share the changed producer/measurement helpers; their canonical
certificate bytes must remain unchanged. W01's audit checks production
identity. Unrelated production suites are not rerun, and the complete T01
gate remains deferred to W05.

Complete source/operator binding, actual loop/return/closed-error and
descriptor composition, remaining byte/collection/control laws and aggregate
boundary/resource closure remain due. W03 stays In progress, W04 has not
started, and no practical Go profile is active.
