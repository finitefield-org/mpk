# Std.Array.Fixed Axiom Review

Scope: STD-007 (`proofs/std/array/std-array-fixed.hex`).

Review evidence:

- Command: `cargo run -q -p mpk-cli -- axiom-report proofs/std/array/std-array-fixed.hex`
- Certificate hash: `b0fe4c18c7747d9d1e5b921676a6e67964813e8f16d0b0d69c2f8100072ca66d`
- Export hash: `01893431153c2b40e1b1bf4326d01103d8f8edf88db87c9b8edc57df8c086ff1`
- Axiom report hash: `37491db0d85fb74cfbe401027ca3333a53e5f8b4df517aecad56c20a1e9a3222`
- Expected release profile: `core-mvp` only after these concrete identities are
  approved by name and hash; not permitted by `zero-axiom`.
- Owner: Stdlib lead owns the interface; Theory lead owns the TH-005 checked
  array read/write replacement path.
- Deterministic test fixture: `proofs/std/array/read-write-fixture.hex`, which
  is zero-axiom and included in `./scripts/checker-agreement.sh`.

Summary:

| Category | Count |
|---|---:|
| CoreAxiom | 3 |
| BuiltinTheoryAxiom | 0 |
| GoSemanticsAxiom | 0 |
| ExternalAxiom | 0 |
| Total | 3 |

Stable hook families:

| Family | Names |
|---|---|
| Array shape | `Length`, `Array`, `Index` |
| Index safety | `InBounds` |
| Operations | `length`, `read`, `write` |

Review decision:

- Accepted for STD-007 as an explicit interface certificate, not as a
  release-approved axiom set.
- `Length`, `Array`, `Index`, and `InBounds` are inductive declarations, not
  axioms.
- All observed axioms are `CoreAxiom`; no `BuiltinTheoryAxiom`,
  `GoSemanticsAxiom`, or `ExternalAxiom` is present.
- These three axiom identities are release blockers until a profile explicitly
  approves them by name and hash.
- TH-005 must replace solver yes/no trust with checked array read/write
  certificates against the stable `read` and `write` hooks.
- If any reviewed type hash or declaration hash changes, that changed identity
  must be reviewed as a new axiom.
