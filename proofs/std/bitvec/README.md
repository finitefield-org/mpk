# Std.BitVec

`std-bitvec.hex` is the canonical MPK certificate for the foundational
fixed-width bitvector interface used by future Go integer and theory-certificate
work.

It exports `BV8`, `BV16`, `BV32`, and `BV64`. Each width provides:

- Literal constants: `zero`, `one`.
- Unary operations: `not`, `neg`.
- Binary operations: `and`, `or`, `xor`, `add`, `sub`, `mul`, `shl`, `lshr`,
  `ashr`.
- Unsigned relations: `ult`, `ule`; reducible flipped forms `ugt`, `uge`.
- Signed relations: `slt`, `sle`; reducible flipped forms `sgt`, `sge`.

These names are stable hooks for fixed-width Go integer lowering: unsigned Go
integers use the unsigned relations, and signed Go integer views use the signed
relations explicitly.

`ground-eval-fixture.hex` is a self-contained zero-axiom fixture. It duplicates
local BV and equality interfaces because certificate import resolution is not
implemented yet. The checked fixture covers representative ground results:

- BV8 `1 + 1 = 2`.
- BV8 `not 0 = allOnes`.
- BV8 `allOnes and 1 = 1`.
- BV8 signed `allOnes < 0` evaluates to true.
- BV16 `1 xor 1 = 0`.
- BV32 `1 << 1 = 2`.
- BV64 unsigned `0 < 1` evaluates to true.

The fixture encodes the expected ground results as reducible declarations and
proves them by local reflexivity. It is a target for TH-002's checked
bitvector-normalization path; it does not trust solver yes/no answers.

`std-bitvec.hex` intentionally contains 68 `CoreAxiom` operation and relation
declarations. The axiom use is reviewed in `AXIOM_REVIEW.md`; these identities
remain release blockers until a release profile explicitly approves them.

Verify the certificates and axiom reports with:

```sh
cargo run -q -p mpk-cli -- check proofs/std/bitvec/std-bitvec.hex
cargo run -q -p mpk-cli -- check proofs/std/bitvec/ground-eval-fixture.hex
cargo run -q -p mpk-cli -- axiom-report proofs/std/bitvec/std-bitvec.hex
cargo run -q -p mpk-cli -- axiom-report proofs/std/bitvec/ground-eval-fixture.hex
./scripts/checker-agreement.sh
```
