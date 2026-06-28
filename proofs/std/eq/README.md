# Std.Eq

`std-eq.hex` is the canonical MPK certificate for the foundational equality
interface and MVP rewrite lemmas.

It exports:

- `Std.Eq`: dependent equality family, `(A : Sort0) -> A -> A -> Sort0`.
- `Std.Eq.refl`: generated reflexivity constructor interface.
- `Std.Eq.rec`: generated equality eliminator interface.
- `Std.Eq.symm`: checked symmetry theorem.
- `Std.Eq.trans`: checked transitivity theorem.
- `Std.Eq.congr`: checked congruence theorem for unary functions.
- `Std.Eq.rewrite`: checked motive rewrite theorem.

`rewrite-fixture.hex` is a self-contained structural proof-node fixture that
exercises the verifier's `Rewrite` node with local equality and unit
declarations. It is self-contained because certificate import resolution is not
implemented yet.

Both certificates are zero-axiom. Verify them with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/eq/std-eq.hex
cargo run -q -p mpk-cli -- check proofs/std/eq/rewrite-fixture.hex
./scripts/checker-agreement.sh
```
