# Std.Nat

`std-nat.hex` is the canonical MPK certificate for the foundational natural
number interface.

It exports:

- `Std.Nat`: natural number family.
- `Std.Nat.zero`: generated zero constructor interface.
- `Std.Nat.succ`: generated successor constructor interface.
- `Std.Nat.rec`: generated natural number recursor interface.

`normalization-fixture.hex` is a self-contained fixture that duplicates the Nat
interfaces and a minimal local equality interface because certificate import
resolution is not implemented yet. It checks `Eq.refl` proofs whose expected
types force definitional normalization of representative `Nat.rec` zero,
successor, and nested-successor expressions.

Both certificates are zero-axiom. Verify them with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/nat/std-nat.hex
cargo run -q -p mpk-cli -- check proofs/std/nat/normalization-fixture.hex
./scripts/checker-agreement.sh
```
