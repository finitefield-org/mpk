# Std.Int Axiom Review

Scope: STD-005 (`proofs/std/int/std-int.hex`).

Review evidence:

- Command: `cargo run -q -p mpk-cli -- axiom-report proofs/std/int/std-int.hex`
- Certificate hash: `8ec7dd6d3c6ed1a49033f2480b91cd8f9fa1daba9922917ddd4f23bb30a8ef91`
- Export hash: `9c82d49ba30104f7eda631e0580309f63d6954a6edc88d3cbc68e8e48405da40`
- Axiom report hash: `5157801c5ff185adcc787824a57fc0cdcbe954086a13a497c99e4e0e0e8635a8`
- Expected release profile: `core-mvp` only after these concrete identities are approved by name and hash; not permitted by `zero-axiom`.
- Owner: Stdlib lead owns the interface; Theory lead owns the TH-004 replacement path.
- Deterministic test fixture: `proofs/std/int/std-int.hex`, included in `./scripts/checker-agreement.sh`.

Summary:

| Category | Count |
|---|---:|
| CoreAxiom | 7 |
| BuiltinTheoryAxiom | 0 |
| GoSemanticsAxiom | 0 |
| ExternalAxiom | 0 |
| Total | 7 |

Review decision:

- Accepted for STD-005 as an explicit interface certificate, not as a release-approved axiom set.
- All observed axioms are `CoreAxiom`; no `BuiltinTheoryAxiom`, `GoSemanticsAxiom`, or `ExternalAxiom` is present.
- `Std.Int.sub`, `Std.Int.ge`, and `Std.Int.gt` are reducible definitions, not new axioms.
- These seven axiom identities are release blockers until a profile explicitly approves them by name and hash.
- TH-004 must replace solver yes/no trust with checked linarith certificates against the stable `Std.Int.add`, `Std.Int.neg`, `Std.Int.sub`, and `Std.Int.le` hooks.
- If any reviewed type hash or declaration hash changes, that changed identity must be reviewed as a new axiom.

Reviewed axiom identities:

| Name | Type hash | Declaration hash | Reason |
|---|---|---|---|
| `Std.Int` | `c8fa0789d8eeccb6e64557ed2de1c82d7a3c79ae5f17981908545dc9d631e677` | `f0bb0cfb7da37b6d5998c8954fd84a3563756f90c4569dbe95119fb500eab4f3` | Carrier for mathematical integers; not encoded as fixed-width Go integers. |
| `Std.Int.add` | `7eef80dd4241e17ca52ce7453b86a77b5670b1f8f86baea976319713f76380dd` | `5810c886e9917a99f4cc66d026e55acb8055599350adfcdbd2cd708a73217a2a` | Stable linear-expression addition hook. |
| `Std.Int.le` | `a561dafab98c50264a26ef0155f7ab3663ee84cb91a7e200c30bda06728556e0` | `b94e0a7e28971fc6d6e2f888b972be66d9cdd9528a96fd87c46b6b33a6778b79` | Non-strict order predicate for linarith goals. |
| `Std.Int.lt` | `a561dafab98c50264a26ef0155f7ab3663ee84cb91a7e200c30bda06728556e0` | `ee342c4de7c63647afaf0ea75dd5d51c0bd36130b106fa117045f3c2689e228b` | Strict order predicate for interfaces that need strict comparisons. |
| `Std.Int.neg` | `fec4927b937dd569a2040d64b511b028092730155ef03682c89bf3e677608061` | `e6b2e3628ff6d0c449058900d13c5382b1836d9233ef8d5a2cf61a5fb5359248` | Stable linear-expression negation hook. |
| `Std.Int.one` | `6904241591a57b71c39737c46a128d278cbb90dc7602b1b4328853946f367690` | `29530c39dcf0f29527ad4e1e6baaa9f697734d2f9d23466b5e54bd998b7e5c3d` | Literal anchor for integer expressions. |
| `Std.Int.zero` | `6904241591a57b71c39737c46a128d278cbb90dc7602b1b4328853946f367690` | `567796f4e5f3965c6a7787f6bc577d67991b2333430ff5cd32e5e8daeb44cf44` | Literal anchor for integer expressions. |
