# Std.BitVec Axiom Review

Scope: STD-006 (`proofs/std/bitvec/std-bitvec.hex`).

Review evidence:

- Command: `cargo run -q -p mpk-cli -- axiom-report proofs/std/bitvec/std-bitvec.hex`
- Certificate hash: `c00a4930bf34d0c56b2341804dc919bbef0fd2aeb50c45a9a3c2db88b803afb2`
- Export hash: `6bc79207c7348c9656f15a02e88eed2837bb78ea2ffbabeb907e6299de46cd70`
- Axiom report hash: `a4bcc1d5793236709d0d9f853b7d214cef2d7d296188c68bbce20d5ae8a439e6`
- Expected release profile: `core-mvp` only after these concrete identities are
  approved by name and hash; not permitted by `zero-axiom`.
- Owner: Stdlib lead owns the interface; Theory lead owns the TH-002 checked
  bitvector-normalization replacement path.
- Deterministic test fixture: `proofs/std/bitvec/ground-eval-fixture.hex`, which
  is zero-axiom and included in `./scripts/checker-agreement.sh`.

Summary:

| Category | Count |
|---|---:|
| CoreAxiom | 68 |
| BuiltinTheoryAxiom | 0 |
| GoSemanticsAxiom | 0 |
| ExternalAxiom | 0 |
| Total | 68 |

Stable hook families:

| Family | Names |
|---|---|
| Carriers | `BV8`, `BV16`, `BV32`, `BV64` |
| Literal constants | `zero`, `one` |
| Unary operations | `not`, `neg` |
| Binary operations | `and`, `or`, `xor`, `add`, `sub`, `mul`, `shl`, `lshr`, `ashr` |
| Unsigned relations | `ult`, `ule`; flipped definitions `ugt`, `uge` |
| Signed relations | `slt`, `sle`; flipped definitions `sgt`, `sge` |

Review decision:

- Accepted for STD-006 as an explicit interface certificate, not as a
  release-approved axiom set.
- The `BV8`, `BV16`, `BV32`, and `BV64` carrier declarations are inductive
  declarations, not axioms.
- The flipped comparison forms `ugt`, `uge`, `sgt`, and `sge` are reducible
  definitions, not new axioms.
- All observed axioms are `CoreAxiom`; no `BuiltinTheoryAxiom`,
  `GoSemanticsAxiom`, or `ExternalAxiom` is present.
- These 68 axiom identities are release blockers until a profile explicitly
  approves them by name and hash.
- TH-002 must replace solver yes/no trust with checked bitvector-normalization
  certificates against the stable operation and relation hooks.
- If any reviewed type hash or declaration hash changes, that changed identity
  must be reviewed as a new axiom.
