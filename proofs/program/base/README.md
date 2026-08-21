# Std.Program.Base

`std-program-base.hex` is the canonical, source-language-neutral type
foundation used by VIR v0 and VC v1. It is self-contained until certificate
import resolution is available and exports checked aliases for:

- `Std.Program.Base.Bool`;
- signed and unsigned 8-, 16-, 32-, and 64-bit integer views;
- fixed-array length, array, index, and in-bounds families; and
- nominal struct shape, field, field-type, and value families.

Signed and unsigned aliases at the same width reduce to the same bitvector
carrier. Signedness remains an explicit operation-level interpretation.
Struct shapes start with the fully qualified VIR declaration ID and retain
declaration-order fields, so equal-layout declarations do not merge.

`type-map-fixture.hex` is a self-contained type-map fixture. It proves by
local reflexivity that representative Bool, BV, fixed-array, and nominal
struct aliases reduce to their core carriers. Both certificates contain zero
axioms; `hashes.csv` records their canonical export, axiom-report, and complete
certificate hashes.

Verify them with:

```sh
cargo run -q -p mpk-cli -- check proofs/program/base/std-program-base.hex
cargo run -q -p mpk-cli -- check proofs/program/base/type-map-fixture.hex
cargo run -q -p mpk-cli -- axiom-report proofs/program/base/std-program-base.hex
cargo run -q -p mpk-cli -- axiom-report proofs/program/base/type-map-fixture.hex
./scripts/checker-agreement.sh
```
