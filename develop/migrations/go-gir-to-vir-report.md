# Go GIR-to-VIR semantic migration report

> Derived by `scripts/compare-go-gir-vir.py`; do not edit by hand.
> This is a development-only audit artifact owned for archival or deletion by GO-VIR-02-T12.

## Result

**equivalent_with_reviewed_changes** with 0 unexplained differences.

| Measure | Count |
|---|---:|
| Baseline leaves covered | 463 / 463 |
| Reviewed dispositions | 62 |
| Historical accepted/rejected rules | 23 / 21 |
| Accepted/rejected source cases | 108 / 16 |
| Canonical function identities | 105 |
| Runtime checks | 3 |
| Obligation kinds | 7 |
| Checker anchors | 2 |
| Checked fixture files | 66 |

## Explicitly allowed changes

| Change | Old | New | Reviewed disposition |
|---|---|---|---|
| `schema` | mpk.gir.v0 and GIR-era envelopes | mpk.vir.v0 and generic frontend envelopes | reviewed breaking replacement; no compatibility importer |
| `identifier` | GIR function and theorem strings | canonical import-path declaration IDs and stable VC member IDs | reviewed identity replacement; semantic ownership preserved |
| `declaration_group` | one theorem declaration per GIR obligation | contract and panic-free declarations containing ordered VC members | reviewed grouping change; every member remains assigned exactly once |
| `foundation_name` | Std.Go.Base.* | Std.Program.Base.* | reviewed zero-axiom checked-foundation rename |
| `artifact_bytes_and_hashes` | GIR, VC v0, skeleton, policy, and evidence bytes/hashes | regenerated VIR, VC v1, grouped skeleton, policy, and evidence bytes/hashes | audit anchors only; byte and hash equality is not required |

## Frontend contract

| Baseline pointer | Old | New | Disposition |
|---|---|---|---|
| `/frontend_contract/cli_schema` | mpk.go2gir.cli.v0 | mpk.frontend.cli.v0 | breaking replacement |
| `/frontend_contract/success_status` | gir-lowered | ir-lowered | breaking replacement |
| `/frontend_contract/gir_emit_schema` | mpk.gir.emit.v0 | mpk.vir.v0 nested in the generic frontend envelope | breaking replacement |
| `/frontend_contract/source_manifest_schema` | mpk.go.source_manifest.v0 | mpk.source_manifest.v0 | breaking replacement with preserved input traceability |
| `/frontend_contract/canonical_binary` | MPK_GIR_V0 framing and hash | MPK-VIR-0.1 canonical JSON hash and generic JCS envelope | no byte or hash equality |
| `/frontend_contract/fail_closed` | true | true | preserved |

## Obligation kinds

| Old | New | Preserved intent |
|---|---|---|
| `precondition` | `callee_precondition` | call site proves each ordered callee requirement |
| `postcondition` | `postcondition` | each reachable return proves each ordered ensure |
| `runtime_safety` | `operation_safety` | path assumptions prove each exact VIR safety check |
| `loop_invariant_initial` | `loop_initialization` | preheader establishes each invariant |
| `loop_invariant_preservation` | `loop_preservation` | each backedge preserves each invariant |
| `loop_exit` | `loop_exit` | invariant plus false header condition establishes ensures |
| `decreases` | `loop_decreases` | signed nonnegativity where required and strict decrease per backedge |

## Source inventory

| Corpus | Accepted | Rejected | Semantic disposition |
|---|---:|---:|---|
| Go alpha | 100 | 0 | Complete 100-function inventory and operation intent preserved |
| Go basic | 3 | 4 | Outcomes and deterministic rejection classes preserved |
| Payment policies | 5 | 4 | Eight postcondition intents, classifications, and clause/branch order preserved |
| Focused frontend cases | 0 | 8 | Reviewed diagnostic corrections only |

## Required runtime checks

| Operation | Old predicate | New check/member kind | Predicate components |
|---|---|---|---:|
| signed division | `divisor != 0` | `divisor_nonzero` / `operation_safety` | 1 |
| shift with signed count | `count >= 0` | `shift_count_nonnegative` / `operation_safety` | 1 |
| fixed-array read with signed index | `0 <= index && index < length` | `index_in_bounds` / `operation_safety` | 2 |

## Contracts, loops, and property intent

- Normalized contract rules: 5 reviewed rules.
- Partial loop members: `loop_initialization, loop_initialization, loop_preservation, loop_preservation, loop_exit`.
- Total-correctness loop members: 7 including two `loop_decreases` members.
- Payment-policy postcondition intents per policy: 8; clause and branch order preserved.

## Checker anchors

| Anchor | Source-free | Reference | Hash agreement | Status |
|---|---|---|---|---|
| `release_report` | accepted | accepted | true | `preserved` |
| `payment_reserve_theory_evidence` | pending | pending | pending | `audit_only_pending_GO-VIR-02-T11` |

## Coverage

All 463 baseline leaves have exactly one coverage assignment. The findings list is empty.
