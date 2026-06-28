# Std.Go.Base

`std-go-base.hex` is the canonical MPK certificate for the Go-facing semantic
base types used by future GIR and VC generation work.

It exports self-contained core carriers and reducible Go-facing aliases because
certificate import resolution is not implemented yet. The stable Go-facing
aliases are:

- `Std.Go.Base.Bool`.
- `Std.Go.Base.Int8`, `Std.Go.Base.Int16`, `Std.Go.Base.Int32`,
  `Std.Go.Base.Int64`.
- `Std.Go.Base.Uint8`, `Std.Go.Base.Uint16`, `Std.Go.Base.Uint32`,
  `Std.Go.Base.Uint64`.
- `Std.Go.Base.Array.Length`, `Std.Go.Base.Array`,
  `Std.Go.Base.Array.Index`, `Std.Go.Base.Array.InBounds`.
- `Std.Go.Base.Struct.Shape`, `Std.Go.Base.Struct.Field`,
  `Std.Go.Base.Struct.FieldType`, `Std.Go.Base.Struct.Value`.

The signed and unsigned integer aliases for a width reduce to the same
fixed-width bitvector carrier. Signedness is an explicit semantic view used by
later lowering and VC generation, not a separate payload type.

The fixed-array aliases expose length, array, index, and in-bounds predicates so
runtime index-safety obligations remain explicit. Struct aliases expose a shape,
field family, field-type family, and struct-value family for MVP structs whose
fields are accepted Go base types.

`type-map-fixture.hex` is a self-contained zero-axiom fixture. It proves by
local reflexivity that representative Go-facing aliases reduce to their core
carriers:

- Go bool reduces to the core Bool carrier.
- Go `int8` reduces to the BV8 carrier.
- Go `uint64` reduces to the BV64 carrier.
- A fixed-array instantiation reduces to the core fixed-array carrier.
- A struct-value instantiation reduces to the core struct carrier.

Both certificates are zero-axiom. Verify them with:

```sh
cargo run -q -p mpk-cli -- check proofs/go/base/std-go-base.hex
cargo run -q -p mpk-cli -- check proofs/go/base/type-map-fixture.hex
cargo run -q -p mpk-cli -- axiom-report proofs/go/base/std-go-base.hex
cargo run -q -p mpk-cli -- axiom-report proofs/go/base/type-map-fixture.hex
./scripts/checker-agreement.sh
```
