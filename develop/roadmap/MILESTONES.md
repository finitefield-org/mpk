# Milestones and Acceptance Criteria

## Current language scope and milestone ownership

MPK supports [Go and Rust only](../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
M0 through M10 below record the completed Go implementation baseline.
`VIR-00`, `VIR-01`, `GO-VIR-02` and `RUST-03` through `RUST-07-T05` completed
the shared Go/VIR/Rust migration and scoped Rust release. Certificate v0 and
source-free checker acceptance remain unchanged.

Current expansion follows [the practical Go work plan](../docs/09_go_practical_subset_design-todo.md):

```text
GO-PRAC-T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08 -> T09 -> T10 -> T11
```

Continue its accepted feature priority and serial T/W order. T01 is complete;
T02-W01 remains in progress and the practical Go profile is inactive. Other
languages have no implementation, resumption task or release prerequisite.
Historical migration records apply only to their original identities.

## M0: Project charter accepted

Acceptance criteria:

- Trust boundary approved.
- MVP Go subset approved.
- Kernel taboo list approved.
- Axiom policy approved.
- Repository skeleton created.

## M1: Core checker prototype

Acceptance criteria:

- Core terms can be constructed without surface syntax.
- Type inference works for basic dependent functions.
- Definitional equality handles beta/delta/zeta under deterministic fuel.
- Generated recursor iota reduction works only for accepted MVP inductives.
- Opaque theorem bodies do not unfold downstream.

## M2: Certificate round trip

Acceptance criteria:

- `.mpcert` canonical encoder and decoder exist.
- Re-encoding validates byte identity.
- Certificate, export, and axiom-report hashes are deterministic.
- Minimal theorem fixture checks source-free.

## M3: Fast kernel usable

Acceptance criteria:

- CLI can check a certificate.
- CLI emits JSON verdict and structured errors.
- Bootstrap and structural proof-node fixtures pass under their active profiles.
- Import hash validation works.
- Axiom report recomputes.

## M4: Independent reference checker usable

Acceptance criteria:

- Go checker can decode and check MVP certificates.
- Go checker shares no kernel implementation code with Rust checker.
- Rust and Go verdicts agree on positive and negative fixtures.

## M5: MVP standard library

Acceptance criteria:

- Bool, equality, minimal Nat, Int interface, BitVec interface, and fixed-array interface are available.
- Standard library certificates verify under both checkers.
- Axiom report is explicit and reviewable.

## M6: Pre-cutover Go-to-VIR frontend baseline

Acceptance criteria:

- `go2vir` lowers allowed pure Go functions.
- Unsupported language features reject with exact reasons.
- VIR output is canonical and hashable.
- Source manifest records Go version and frontend hash.

## M7: Pre-cutover VIR VC generation baseline

Acceptance criteria:

- VCs generated for straight-line functions.
- VCs generated for branches.
- Runtime-safety VCs generated for division, shift, and indexing.
- Loop invariant VCs generated for explicitly annotated loops.
- Variant/decreases VCs generated only when total correctness is claimed.

## M8: AI proof API

Acceptance criteria:

- Candidate proof DAGs can be submitted via JSONL/API.
- Batch candidate checking works.
- Diagnostics identify failed node, expected type, actual type, and likely repair operators.
- Candidate generation remains outside trusted base.

## M9: Theory certificates

Acceptance criteria:

- BitVec ground proofs can be checked without huge proof terms.
- Linear arithmetic certificates check independently.
- Array read/write certificates check independently.
- Malformed theory certificates reject deterministically.

## M10: Pre-cutover Go alpha verification demo baseline

Acceptance criteria:

- At least 100 small Go functions verify.
- At least 1,000 VCs are generated and checked.
- At least 10,000 invalid AI candidates are rejected deterministically.
- Both checkers agree on all alpha artifacts.
- Release report includes hashes and axiom reports.
