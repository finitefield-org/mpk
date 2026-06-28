# Std.Logic

`std-logic.hex` is the canonical MPK certificate for the foundational logic
interfaces used by later VC certificates.

It exports:

- `Std.Logic.Imp`: reducible implication interface, definitionally equal to
  `P -> Q`.
- `Std.Logic.And`: conjunction family.
- `Std.Logic.And.intro`: generated constructor interface.
- `Std.Logic.And.rec`: generated recursor interface.
- `Std.Logic.Or`: disjunction family.
- `Std.Logic.Or.inl`: generated left constructor interface.
- `Std.Logic.Or.inr`: generated right constructor interface.
- `Std.Logic.Or.rec`: generated recursor interface.

The certificate is zero-axiom. Verify it with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/logic/std-logic.hex
./scripts/checker-agreement.sh
```
