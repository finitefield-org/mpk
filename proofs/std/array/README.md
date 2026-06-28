# Std.Array.Fixed

`std-array-fixed.hex` is the canonical MPK certificate for the foundational
fixed-array interface used by future Go array lowering and array
read/write-certificate work.

It exports:

- `Std.Array.Fixed.Length`: fixed-array length witness type.
- `Std.Array.Fixed.Array`: array family indexed by element type and length.
- `Std.Array.Fixed.Index`: index family indexed by length.
- `Std.Array.Fixed.InBounds`: index-safety predicate for a length and index.
- `Std.Array.Fixed.length`: array length hook.
- `Std.Array.Fixed.read`: bounds-checked array read hook.
- `Std.Array.Fixed.write`: bounds-checked array write hook that preserves the
  array length parameter.

`read` and `write` both take an explicit `InBounds` proof. That keeps runtime
index safety visible to VC generation and leaves read/write reasoning to
TH-005's checked array-certificate path.

`read-write-fixture.hex` is a self-contained zero-axiom fixture. It duplicates
local element, length, index, in-bounds, array, and equality interfaces because
certificate import resolution is not implemented yet. The checked fixture
covers representative ground results:

- Reading the same index after a write returns the written value.
- Reading a different index after a write returns the original value.
- Writing preserves the fixed-array length witness.

The fixture encodes the expected read/write results as reducible declarations
and proves them by local reflexivity. It does not trust solver yes/no answers.

`std-array-fixed.hex` intentionally contains three `CoreAxiom` hook
declarations. The axiom use is reviewed in `AXIOM_REVIEW.md`; these identities
remain release blockers until a release profile explicitly approves them.

Verify the certificates and axiom reports with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/array/std-array-fixed.hex
cargo run -q -p mpk-cli -- check proofs/std/array/read-write-fixture.hex
cargo run -q -p mpk-cli -- axiom-report proofs/std/array/std-array-fixed.hex
cargo run -q -p mpk-cli -- axiom-report proofs/std/array/read-write-fixture.hex
./scripts/checker-agreement.sh
```
