# MPK: Machine Proof Kernel

**Subtitle:** An AI-oriented, certificate-first theorem prover and program-verification kernel, initially targeting a restricted Go subset.

This package extends the prior MPK design with an implementation roadmap, task backlog, milestone gates, draft specifications, examples, and project-start templates.

## Package contents

```text
.
├── README.md
├── docs/
│   ├── CHARTER.md
│   ├── 00_executive_summary.md
│   ├── 01_design_added_implementation_roadmap.md
│   ├── 02_architecture.md
│   ├── 03_trust_boundary.md
│   └── 04_references.md
├── specs/
│   ├── CORE_V0.md
│   ├── CERT_V0.md
│   ├── TRUST_BOUNDARY_V0.md
│   ├── AXIOM_POLICY_V0.md
│   ├── UNSAFE_POLICY_V0.md
│   ├── GIR_V0.md
│   ├── GO_SUBSET_V0_DRAFT.md
│   └── AI_API_V0_DRAFT.md
├── roadmap/
│   ├── ROADMAP.md
│   ├── MILESTONES.md
│   └── RELEASE_GATES.md
├── tasks/
│   ├── TASK_BACKLOG.md
│   ├── task_backlog.csv
│   ├── github_issues_seed.jsonl
│   ├── definition_of_done.md
│   └── risk_register.md
├── examples/
│   ├── max64_example.md
│   ├── sample_contract.json
│   └── sample_go_source.go
├── templates/
│   ├── certificate_manifest.json
│   └── module_manifest.yaml
```

## Core decision

MPK should not be a human-first proof assistant. It should be a **machine-proof container and checker toolchain**:

```text
Go source
  -> Go frontend / SSA / GIR
  -> VC generator
  -> AI / solver / tactic candidate generation
  -> canonical .mpcert
  -> Rust fast kernel + independent source-free reference checker
```

The only proof evidence is the canonical certificate and checker verdicts. Go source, frontend output, AI traces, solver answers, tactic traces, theorem indexes, diagnostics, and CI metadata are useful engineering data, but they are not proof evidence.

## Primary implementation language choices

- Fast kernel: Rust.
- Independent reference checker: Go.
- First source-language verification frontend: Go subset, via an untrusted `go2gir` pipeline.
- Certificate format: canonical binary `.mpcert`, with deterministic hashes and axiom report.

## How to use this package

Start with:

1. `docs/CHARTER.md`
2. `docs/00_executive_summary.md`
3. `roadmap/ROADMAP.md`
4. `tasks/TASK_BACKLOG.md`
5. `specs/TRUST_BOUNDARY_V0.md`
6. `specs/AXIOM_POLICY_V0.md`
7. `specs/UNSAFE_POLICY_V0.md`
8. `specs/CORE_V0.md`
9. `specs/CERT_V0.md`

Then seed project-management issues from `tasks/github_issues_seed.jsonl` or import `tasks/task_backlog.csv` into a spreadsheet or tracker.

## Reference posture

This plan is inspired by NPA's certificate-first trust boundary and source-free checker design, but it is a new system design aimed at Go program verification and AI-generated proof-candidate checking.

Prepared: 2026-06-26
