# Std.Int

`std-int.hex` is the canonical MPK certificate for the foundational
mathematical integer interface used by future VC and theory-certificate work.

It exports:

- `Std.Int`: mathematical integer carrier.
- `Std.Int.zero`: integer zero.
- `Std.Int.one`: integer one.
- `Std.Int.neg`: unary negation.
- `Std.Int.add`: binary addition.
- `Std.Int.sub`: reducible subtraction, `a + neg b`.
- `Std.Int.le`: non-strict integer order predicate.
- `Std.Int.lt`: strict integer order predicate.
- `Std.Int.ge`: reducible flipped `le`.
- `Std.Int.gt`: reducible flipped `lt`.

`Std.Int.add`, `Std.Int.neg`, `Std.Int.sub`, and `Std.Int.le` are the stable
linear-arithmetic hooks intended for TH-004's checked linarith certificates.
This interface does not trust solver yes/no answers and does not introduce a
checked linarith theorem yet.

`std-int.hex` intentionally contains seven `CoreAxiom` declarations. The axiom
use is reviewed in `AXIOM_REVIEW.md`; these identities remain release blockers
until a release profile explicitly approves them.

Verify the certificate and its axiom report with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/int/std-int.hex
cargo run -q -p mpk-cli -- axiom-report proofs/std/int/std-int.hex
./scripts/checker-agreement.sh
```
