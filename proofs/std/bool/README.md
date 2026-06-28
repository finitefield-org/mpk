# Std.Bool

`std-bool.hex` is the canonical MPK certificate for the foundational Boolean
interface and Boolean-valued normalization definitions.

It exports:

- `Std.Bool`: Boolean family.
- `Std.Bool.false`: generated false constructor interface.
- `Std.Bool.true`: generated true constructor interface.
- `Std.Bool.rec`: generated Boolean recursor interface.
- `Std.Bool.if`: reducible Boolean selector, `if c then then_case else else_case`.
- `Std.Bool.not`: reducible Boolean negation.
- `Std.Bool.and`: reducible Boolean conjunction.
- `Std.Bool.or`: reducible Boolean disjunction.

`normalization-fixture.hex` is a self-contained fixture that duplicates the Bool
interfaces and a minimal local equality interface because certificate import
resolution is not implemented yet. It checks `Eq.refl` proofs whose expected
types force definitional normalization of representative ground `not`, `and`,
`or`, and `if` expressions.

Both certificates are zero-axiom. Verify them with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/bool/std-bool.hex
cargo run -q -p mpk-cli -- check proofs/std/bool/normalization-fixture.hex
./scripts/checker-agreement.sh
```
