# Implementation Roadmap

This file records the dependency-ordered completed Go implementation baseline.
Do not begin a later phase by expanding the trusted base to compensate for an
incomplete earlier phase.

The forward VIR/Rust migration sequence is owned by
`../docs/05_rust_frontend_design.md` and
`../docs/05_rust_frontend_design-todo.md`:

1. VIR-00 froze all replacement specifications, vectors, ownership, and
   governance amendments.
2. VIR-01 implemented the shared VIR/VC/checking foundations.
3. GO-VIR-02 implemented all new Go consumers and completed the atomic cutover.
4. RUST-03 through RUST-07-T05 completed the pinned Rust frontend, semantics,
   verification, hardening, and release without widening the proof trust
   boundary.
5. The serial post-Rust program below now owns subsequent production phases.

Post-Rust source-language expansion is governed by
`../docs/06_multilanguage_frontend_design.md` and
`../docs/06_multilanguage_frontend_design-todo.md`. It is one continuation of
the same flow, not a parallel track. The flow completed `RUST-07-T05`,
`MLANG-00`, `MLANG-01`, the C# scalar release, and Java through T10, including
its native x86-64 Linux receipt. `CSHARP-03-T01` has published the normative but
inactive practical C# package and `CSHARP-03-T02-W01` is next; Dart,
TypeScript, and Python follow the complete practical-profile release in that
order. A governance or inactive specification record may prepare a later
phase, but no production profile, selection branch, release tuple, bundle, or
frontend starts before its entry gate.

The Phase 0 through Phase 12 entries below are a historical record of the
replaced Go-only baseline, not a parallel active path. Certificate v0,
source-free checker inputs, and axiom categories remain unchanged.

## Phase 0: Governance and spec freeze

**Objective:** make the trust boundary and MVP scope explicit before coding.

Deliverables:

- `CORE_V0.md` frozen for implementation.
- `CERT_V0.md` frozen for implementation.
- `GIR_V0.md` frozen for implementation.
- Axiom policy document.
- Unsafe-code policy.
- `GO_SUBSET_V0.md` frozen for implementation.
- `AI_API_V0.md` frozen for implementation.
- Deterministic hash-domain registry.

Exit gate:

- All contributors can explain what is trusted and what is not.
- No parser, AI, tactic, solver, or frontend component is inside the acceptance boundary.

## Phase 1: Core data model and type checker skeleton

**Objective:** implement the smallest usable dependent core.

Deliverables:

- `mpk-core` Rust crate.
- Level and term arenas.
- De Bruijn representation.
- Substitution/lifting.
- Weak-head normalization.
- Type inference/checking for `Sort`, `Var`, `Const`, `App`, `Lam`, `Pi`, `Let`.
- Deterministic structured error enum.

Exit gate:

- Unit tests cover all core term constructors.
- Ill-typed terms reject deterministically.
- No source-level syntax is required for core tests.

## Phase 2: Definitional equality and declarations

**Objective:** make theorem and definition checking possible.

Deliverables:

- Reducible and opaque definitions.
- Opaque theorem declarations.
- Axiom declarations and axiom dependency collection.
- Deterministic, fuel-limited defeq.
- Basic builtin constants for equality and Bool.
- MVP inductive declarations, conservative positivity checks, generated constructors/recursors, and iota reduction for generated recursors.

Exit gate:

- Theorems check with proof bodies and export as opaque constants.
- Fuel exhaustion rejects.
- Opaque bodies are not unfolded by downstream defeq.

## Phase 3: Canonical certificate format

**Objective:** make source-free artifacts possible.

Deliverables:

- `mpk-cert` Rust crate.
- Canonical encoder/decoder.
- Re-encode byte-equality check.
- Certificate hash.
- Export hash.
- Axiom report hash.
- Import table validation.
- Minimal fixture certificates.

Exit gate:

- A valid certificate round-trips byte-identically.
- Non-canonical encodings reject.
- Hashes recompute exactly.

## Phase 4: Fast Rust kernel verifier

**Objective:** provide the primary production checker.

Deliverables:

- `mpk-kernel` crate.
- Checker driver over canonical certificates.
- `core-bootstrap` proof-node checker.
- `mvp-structural` proof-node checker for let, rewrite, equality recursor, constructor, and recursor nodes.
- Term/type/proof-node caches.
- Import environment.
- Structured JSON diagnostics.
- CLI `mpk check`.

Exit gate:

- All fixture certificates check or reject deterministically.
- Kernel does not read source files, tactics, source maps, or AI traces.

## Phase 5: Independent Go reference checker

**Objective:** reduce single-implementation risk.

Deliverables:

- `go-tools/mpk-checker-ref`.
- Independent decoder.
- Independent hash recomputation.
- Independent type checker for MVP core.
- Independent axiom-report verifier.

Exit gate:

- Rust and Go checkers agree on all fixtures.
- The Go checker has no runtime dependency on Rust kernel code.

## Phase 6: Foundational standard library

**Objective:** build the small proof foundation required for Go VCs.

Deliverables:

- `Std.Logic`.
- `Std.Bool`.
- `Std.Eq`.
- `Std.Nat` minimal.
- `Std.Int` interface.
- `Std.BitVec` interface and ground evaluator.
- `Std.Array.Fixed` interface.

Exit gate:

- Every exported theorem is certificate-checked.
- Axiom reports are clean and intentional.
- No unproved convenience theorem is admitted without report.

## Phase 7: Pre-cutover Go frontend to VIR baseline

**Objective:** convert a safe Go subset into a verification IR.

Deliverables:

- `go2vir` command.
- Go package loading.
- SSA extraction.
- Supported-feature detector.
- Rejected-feature report.
- VIR emitter.
- Source manifest.

Exit gate:

- Unsupported features fail closed.
- Pure examples lower to stable VIR.
- VIR hash is deterministic.

## Phase 8: Pre-cutover VIR semantics and VC generator baseline

**Objective:** generate theorem obligations from Go subset programs.

Deliverables:

- VIR type model in MPK core.
- Go integer/bitvector semantics layer.
- Straight-line symbolic execution.
- If/else VC generation.
- Loop invariant obligations and optional total-correctness variant obligations.
- Runtime-safety obligations for division, shift, and indexing.

Exit gate:

- `Max64`, `Abs64`, and `Clamp64` produce expected VCs.
- Generated VCs are stable and hashable.

## Phase 9: AI proof API and repair loop

**Objective:** make proof candidate generation efficient for AI agents.

Deliverables:

- ID-based term/proof construction API.
- Batch candidate checking.
- Structured rejection diagnostics.
- Candidate cache.
- Minimal non-theory proof search heuristics.
- JSONL import/export.

Exit gate:

- An AI or script can generate and repair proof-node DAGs without human syntax.
- Failed proof candidates are rejected cheaply and locally.

## Phase 10: Theory certificate checkers

**Objective:** avoid expanding all arithmetic and bitvector proofs into huge proof terms.

Deliverables:

- Bool normalization certificate checker.
- BitVec ground-normalization checker.
- Linear arithmetic certificate checker.
- Fixed-array read/write certificate checker.
- Theory-backed API strategy hook.
- Theory-certificate fuzz tests.

Exit gate:

- Solver yes/no answers are never trusted.
- Only checkable theory certificates are accepted.

## Phase 11: CI, package verification, and release artifacts

**Objective:** make verification reproducible.

Deliverables:

- `check-fast.sh`.
- `check-reference.sh`.
- `check-all.sh`.
- Package manifest and lock format.
- Hash-pinned import store.
- Release verification report.

Exit gate:

- Clean checkout can rebuild and verify artifacts deterministically.
- Rust and Go checker verdicts are both included in release evidence.

## Phase 12: Pre-cutover Go alpha corpus and performance-hardening baseline

**Objective:** demonstrate the system on realistic small Go verification tasks.

Deliverables:

- 100-function Go MVP corpus.
- 1,000+ VC corpus.
- 10,000 failed-candidate benchmark.
- Performance profile.
- Memory profile.
- Regression dashboard.

Exit gate:

- Full corpus verifies reproducibly.
- Failed candidates reject quickly.
- No trusted-boundary regressions.
