# MPK: Machine Proof Kernel

**Subtitle:** An AI-oriented, certificate-first theorem prover and
program-verification kernel with language-neutral verification interfaces.

This package extends the prior MPK design with an implementation roadmap, task
backlog, milestone gates, frozen specifications, conformance vectors,
examples, and project-start templates.

## Core decision

MPK is a machine-proof container and checker toolchain:

```text
source + contracts
  -> untrusted registered frontend / compiler / VIR + source metadata
  -> untrusted VC generator
  -> untrusted AI / solver / tactic candidate generation
  -> canonical .mpcert
  -> Rust fast kernel + independent source-free Go reference checker
```

Only canonical certificate bytes, checked theory certificates, and checker
results derived from them can justify proof acceptance. Registry bytes,
toolchains, compilers, frontends, source, contracts, VIR, source maps, source
manifests and their internal claims, VCs, policy output, AI output, solver
answers, tactic traces, theorem indexes, diagnostics, and CI metadata are
untrusted helper artifacts.

Certificate v0 encoding, checker acceptance inputs, hash domains, and the four
axiom categories remain unchanged by the VIR/Rust migration.

## Specification authority during migration

The Go cutover completed in `GO-VIR-02-T12`. `VIR_V0.md`,
`GO_VIR_PROFILE_V0.md`, `VC_V1.md`, `POLICY_V1.md`, `AI_EXPLAIN_V1.md`, and
`AI_API_V1.md` now define the sole active helper path. The former Go IR, Go
subset, and API v0 specifications are historical records, not compatibility
contracts.

`PROGRAM_CERTIFICATE_ALPHA_V0.md` defines the active self-contained
dual-checker assembly profile for program certificates without changing the
Certificate v0 binary schema.

The completed implementation sequence and migration record are
`docs/05_rust_frontend_design.md` and
`docs/05_rust_frontend_design-todo.md`. The older numbered roadmap records the
completed baseline; it does not override that migration dependency graph.

## Future source-language expansion

The scoped Rust v0 program completed at `RUST-07-T05`. MPK plans to add C#,
Java, Dart, TypeScript, and Python only through the queued, strictly serial
follow-on flow:

1. run `MLANG-00` semantic comparison and compiler-API feasibility work;
2. run `MLANG-01` successor-contract and C# specification freeze; and
3. implement and release C#, Java, Dart, TypeScript, and Python, one language
   at a time in that order.

`MLANG-00-T01` through `MLANG-00-T03`, `MLANG-01-T01` through
`MLANG-01-T03`, and `CSHARP-02-T01` through `CSHARP-02-T20` are complete. The
four non-normative semantic, feasibility, and implementation-audit records are
`docs/mlang-00-semantic-comparison-matrix.md`,
`docs/mlang-00-compiler-integration-feasibility.md`,
`docs/mlang-00-go-rust-shared-boundary-audit.md`, and
`docs/mlang-01-go-rust-csharp-gap-audit.md`. The active successor mechanism is
`specs/SEMANTIC_PROFILE_REGISTRY_V1.md`; the active C# package is
`specs/CSHARP_PROFILE_V0.md` with its two owned vector sets.
`docs/csharp-02-implementation-traceability-ledger.md` records the completed
20-task implementation and atomic release.

The active release installs semantic-registry revision 2 and the sole
`mpk.release.bundle_registry.v1` registry beside `bin/mpk`. It admits exactly
the registered Go, Rust, and C# tuples, uses successor VIR, frontend,
source-map, source-manifest, VC, policy/evidence, AI, and API identities, and
compiles all nine C# profile contracts. C# runs only through the shared
descriptor-relative installed-tree runner under the frozen .NET launcher and
Linux isolation contract; Go and Rust use that same successor release path.
Predecessor and crossed identities reject, the executable staging tree is
removed, and only reviewed migration reports remain under
`develop/migrations/archive/`.

Certificate v0 encoding, both source-free checker inputs, and the four axiom
categories are unchanged. Frontends, compilers, registries, helper schemas,
policy/evidence documents, AI output, and API state remain untrusted. The
atomic installed-release owner is
`crates/mpk-cli/tests/successor_atomic_cutover.rs`, and the offline two-pass
release gate is `scripts/check-csharp-frontend.sh`. `JAVA-03-T01` completed
the inactive [Java profile](specs/JAVA_PROFILE_V0.md), exact Java/revision-3
vectors, pinned JDK/native inventory and disposable public-API/JVM probes.
The [design](docs/07_java_frontend_design.md) and
[implementation ledger](docs/java-03-implementation-traceability-ledger.md)
record the ten serial tasks. `JAVA-03-T02` completed the
[offline build candidate](../java-tools/README.md), with exact project/class/JAR
inventories and two isolated builds. `JAVA-03-T03` completed the inactive
profile and artifact validators; `JAVA-03-T04` completed internal capture,
the public compiler adapter and bounded diagnostics. `JAVA-03-T05` completed
source subset admission, inert initialization, call closure and typed sidecars.
T06 completed private CFG/lowering and deterministic artifact emission; T07's
registered candidate bundles and JVM runner are next. The Java build and
frontend tests use Linux amd64 CPU emulation, not the complete native Linux
release gate. Installed execution remains pending; no
Java or later-language frontend is active and registry revision 2 stays
installed.

No multi-language design, feasibility, specification, or implementation phase
runs in parallel with the Rust program or with another language phase.

The order and gates are defined by `docs/06_multilanguage_frontend_design.md`
and `docs/06_multilanguage_frontend_design-todo.md`. Every future language
gets a distinct semantic profile, mixed-language VIR remains forbidden, and
cross-language composition uses checked hash-pinned certificate imports.

The files under `tasks/` remain the original baseline tracker seed. Post-Rust
task IDs are dependency-ordered by the `05_*_todo.md` and `06_*_todo.md`
programs and are not duplicated into that legacy seed.

## Normative specification routes

Checker and governance foundations:

| Specification | Responsibility |
|---|---|
| `specs/CORE_V0.md` | Core term, declaration, and checking semantics |
| `specs/CERT_V0.md` | Unchanged canonical Certificate v0 encoding |
| `specs/PROGRAM_CERTIFICATE_ALPHA_V0.md` | Active self-contained dual-checker program-certificate assembly profile |
| `specs/TRUST_BOUNDARY_V0.md` | Sole proof-acceptance boundary |
| `specs/AXIOM_POLICY_V0.md` | Exactly four Certificate v0 axiom categories |
| `specs/UNSAFE_POLICY_V0.md` | Checker-facing unsafe-code policy |

Frozen replacement specifications and their complete conformance-vector
ownership are:

| Specification | Conformance vector set(s) |
|---|---|
| `specs/VIR_V0.md` | `specs/vectors/vir-v0.json`, `specs/vectors/vir-hash-v0.json` |
| `specs/VC_V1.md` | `specs/vectors/vc-v1.json`, `specs/vectors/vc-hash-v1.json`, `specs/vectors/vc-skeleton-v1.json` |
| `specs/FRONTEND_PROTOCOL_V0.md` | `specs/vectors/frontend-protocol-v0.json` |
| `specs/RELEASE_BUNDLES_V0.md` | `specs/vectors/release-bundles-v0.json` |
| `specs/SOURCE_MAP_V0.md` | `specs/vectors/source-map-v0.json` |
| `specs/SOURCE_MANIFEST_V0.md` | `specs/vectors/source-manifest-v0.json` |
| `specs/GO_VIR_PROFILE_V0.md` | `specs/vectors/go-vir-profile-v0.json` |
| `specs/RUST_SUBSET_V0.md` | `specs/vectors/rust-subset-v0.json`, `specs/vectors/rust-build-inputs-v0.json` |
| `specs/RUST_DRIVER_PROTOCOL_V0.md` | `specs/vectors/rust-driver-v0.json` |
| `specs/POLICY_V1.md` | `specs/vectors/policy-scan-v1.json`, `specs/vectors/policy-evidence-v1.json`, `specs/vectors/policy-recipes-v1.json` |
| `specs/AI_EXPLAIN_V1.md` | `specs/vectors/ai-explain-v1.json` |
| `specs/AI_API_V1.md` | `specs/vectors/ai-api-v1.json` |

The successor design below is normative and active for the Go/Rust/C# helper
release:

| Specification | Conformance vector set(s) |
|---|---|
| `specs/SEMANTIC_PROFILE_REGISTRY_V1.md` | `specs/vectors/semantic-profile-registry-v1.json` |
| `specs/CSHARP_PROFILE_V0.md` | `specs/vectors/csharp-profile-v0.json`, `specs/vectors/semantic-profile-registry-v2.json` |

`specs/vectors/manifest.json` is the closed repository index for every vector
set. Its repository-governance schema is
`mpk.spec.vector_manifest.v0`, owned by this section and
`scripts/check-spec-vectors.py`; it is not a proof artifact, adds no hash
domain, and is excluded from its own vector inventory. Each entry pins the
vector schema ID, repository path, raw-byte SHA-256, normative owning
specification, and implementation test owner. Run:

```sh
python3 scripts/check-spec-vectors.py --check
```

The command is check-only: it strictly parses the manifest and every vector,
rejects duplicate JSON names, verifies digests and ownership, and rejects
missing or unlisted vector files. It never rewrites normative bytes.

The two files under `templates/` are language-neutral governance examples; they
do not activate a staged runtime interface. In
`templates/certificate_manifest.json`, `source_manifest_reference` is a
template-only audit reference to separately serialized certificate-stage
`mpk.source_manifest.v0` bytes. Its `path`, `lifecycle`, `expected_*`, and
`trust` members are not fields of that normative source-manifest schema and are
never embedded as a substitute payload. The post-cutover release uses the
complete VIR artifact set installed at the active paths.

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
│   ├── 04_references.md
│   ├── 05_rust_frontend_design.md
│   ├── 05_rust_frontend_design-todo.md
│   ├── rust-frontend-toolchain-upgrade.md
│   ├── 06_multilanguage_frontend_design.md
│   ├── 06_multilanguage_frontend_design-todo.md
│   ├── 07_java_frontend_design.md
│   ├── mlang-00-semantic-comparison-matrix.md
│   ├── mlang-00-compiler-integration-feasibility.md
│   ├── mlang-00-go-rust-shared-boundary-audit.md
│   ├── mlang-01-go-rust-csharp-gap-audit.md
│   └── csharp-02-implementation-traceability-ledger.md
├── specs/
│   ├── CORE_V0.md
│   ├── CERT_V0.md
│   ├── PROGRAM_CERTIFICATE_ALPHA_V0.md
│   ├── TRUST_BOUNDARY_V0.md
│   ├── AXIOM_POLICY_V0.md
│   ├── UNSAFE_POLICY_V0.md
│   ├── GIR_V0.md
│   ├── GO_SUBSET_V0.md
│   ├── AI_API_V0.md
│   ├── VIR_V0.md
│   ├── VC_V1.md
│   ├── FRONTEND_PROTOCOL_V0.md
│   ├── RELEASE_BUNDLES_V0.md
│   ├── SOURCE_MAP_V0.md
│   ├── SOURCE_MANIFEST_V0.md
│   ├── GO_VIR_PROFILE_V0.md
│   ├── RUST_SUBSET_V0.md
│   ├── RUST_DRIVER_PROTOCOL_V0.md
│   ├── POLICY_V1.md
│   ├── AI_EXPLAIN_V1.md
│   ├── AI_API_V1.md
│   ├── SEMANTIC_PROFILE_REGISTRY_V1.md
│   ├── CSHARP_PROFILE_V0.md
│   └── vectors/
│       ├── manifest.json
│       └── 21 owned vector sets listed above
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
└── templates/
    ├── certificate_manifest.json
    └── module_manifest.yaml
```

`GIR_V0.md`, `GO_SUBSET_V0.md`, and `AI_API_V0.md` in that inventory are
labeled historical records only; the active interfaces are VIR/profile/policy
v1 documents listed below them.

## Primary implementation language choices

- Fast kernel: Rust.
- Independent reference checker: Go.
- Source-language frontends: untrusted, registered producers of the shared VIR
  contract; Go migrates first, the pinned Rust frontend follows, and later
  languages are admitted serially after the Rust v0 release gate.
- Certificate format: canonical binary `.mpcert`, with deterministic hashes
  and a recomputed axiom report.

## How to use this package

Start with:

1. `docs/CHARTER.md`
2. `specs/TRUST_BOUNDARY_V0.md`
3. `specs/AXIOM_POLICY_V0.md`
4. `specs/UNSAFE_POLICY_V0.md`
5. `specs/CORE_V0.md`
6. `specs/CERT_V0.md`
7. `specs/PROGRAM_CERTIFICATE_ALPHA_V0.md`
8. `docs/05_rust_frontend_design.md`
9. `docs/05_rust_frontend_design-todo.md`
10. `docs/06_multilanguage_frontend_design.md`
11. `docs/06_multilanguage_frontend_design-todo.md`
12. `docs/mlang-00-semantic-comparison-matrix.md`
13. `docs/mlang-00-compiler-integration-feasibility.md`
14. `docs/mlang-00-go-rust-shared-boundary-audit.md`
15. `docs/mlang-01-go-rust-csharp-gap-audit.md`
16. `specs/SEMANTIC_PROFILE_REGISTRY_V1.md`
17. `specs/CSHARP_PROFILE_V0.md`
18. `docs/csharp-02-implementation-traceability-ledger.md`
19. `docs/07_java_frontend_design.md`
20. `docs/java-03-implementation-traceability-ledger.md`
21. `specs/JAVA_PROFILE_V0.md`
22. `roadmap/RELEASE_GATES.md`

The original baseline can be seeded from `tasks/github_issues_seed.jsonl` or
imported from `tasks/task_backlog.csv`. Use the two design todo documents and
the C# and Java implementation ledgers for post-Rust tracker tasks. Java's
T01 freeze, T02 offline build, T03 validators, T04 capture/compiler adapter,
T05 source subset/sidecars and T06 lowering/artifact emission are complete;
T07's private candidate/JVM runner is implemented, with native x86-64 Linux
acceptance still pending. Run that gate before accepting T07 and starting T08.

## Reference posture

This plan is inspired by NPA's certificate-first trust boundary and source-free
checker design, but it is a new system design for language-neutral program
verification and AI-generated proof-candidate checking.

Prepared: 2026-06-26. VIR governance and multi-language planning amendments:
2026-08-21; semantic-matrix and compiler-boundary feasibility updates:
2026-08-25; successor semantic-profile registry and C# package freeze:
2026-08-25; C# implementation decomposition: 2026-08-25.
Inactive pinned C# project and build closure: 2026-08-26.
Inactive successor semantic-profile registry core: 2026-08-26.
