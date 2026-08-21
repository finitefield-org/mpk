# Release Gates

## Gate A: Trust-boundary gate

Before any release:

- No source parser is called by the checker.
- No tactic script is called by the checker.
- No AI trace is read by the checker.
- No solver yes/no answer is trusted.
- All accepted proof evidence is in `.mpcert` or checked theory certificates.

## Gate B: Canonicality gate

- Decoder rejects non-minimal varints.
- Decoder rejects duplicate table entries after canonicalization.
- Decoder rejects unreachable table entries.
- Re-encode result equals original bytes.
- Hashes recompute exactly.

## Gate C: Determinism gate

- Same input produces same certificate bytes.
- Same input produces same export hash.
- Same input produces same axiom report hash.
- Same rejection produces same machine-readable error code and location.

## Gate D: Checker agreement gate

- Rust fast kernel and Go reference checker agree on valid fixtures.
- Rust fast kernel and Go reference checker agree on invalid fixtures.
- Any disagreement blocks release.

## Gate E: Axiom gate

- Axiom report is generated and reviewed.
- Release profile specifies allowed axioms.
- Any new axiom is a release blocker unless explicitly approved.

## Gate F: Source frontend and traceability gate

The generic VIR branch is the sole active source-frontend release gate:

- Unsupported source-language and semantic-profile features fail closed.
- The validated release-registry identity and selected frontend/toolchain
  bundle identities are recorded.
- Compiler, subordinate binary, and frontend binary identities are recorded.
- Every captured source, contract, build-manifest, and lockfile input is
  recorded through the canonical input set.
- VIR, source-map, frontend-stage manifest, certificate-stage manifest, and VC
  hashes recompute and all repeated identities agree.
- Status/exit pairs, profile tuples, target identity, limits, and manifest
  lifecycle follow their owning frozen specifications.
- Registry bytes, toolchains, compilers, frontends, inputs, VIR, maps,
  manifests' internal claims, VCs, policy output, and AI output remain
  untrusted helper artifacts.

## Gate G: Performance gate

- Full MVP fixture suite checks under target resource budgets.
- 10,000 invalid candidates reject without memory growth regressions.
- Defeq fuel exhaustion is deterministic.

## Gate H: Security gate

The unsafe-code portion of this gate is defined by `specs/UNSAFE_POLICY_V0.md`.

- Kernel crates forbid unsafe code in MVP.
- Fuzz tests cover certificate decoder.
- Malformed certificates never panic.
- Public API cannot bypass certificate verification.

## Gate I: New source-language admission gate

This gate is not part of the current Go/Rust release and does not add a Rust
v0 prerequisite. It activates only after the serial handoff in
`../docs/06_multilanguage_frontend_design-todo.md`: `RUST-07-T05` completes,
then `MLANG-00`, then `MLANG-01`, and only then C# production begins. Every
later language's entire phase waits for its predecessor's complete release
gate.

Before a new source language becomes a registered production frontend:

- its exact supported subset, rejected-feature taxonomy, semantic profile,
  contract syntax, compiler/API boundary, target model, canonical fixtures,
  limits, diagnostics, and version policy are frozen and hash-pinned;
- its frontend, subordinate compiler components, toolchain, runtime inputs,
  and release bundle are exact, registered, reproducible identities rather
  than ambient or user-selected executables;
- language selection resolves to exactly one registered frontend and one
  semantic profile before lowering; unknown identities and mismatches reject;
- the frontend emits one language-isolated VIR module; mixed-language VIR,
  cross-language calls, FFI semantics, and ABI claims remain unsupported;
- the language adds no new certificate axiom category, does not change
  Certificate v0 or either source-free checker acceptance rule, and remains
  outside the proof trust boundary;
- positive, negative, boundary, determinism, differential, adversarial, and
  bounded fuzz suites pass, and both source-free checkers accept identical
  resulting certificate bytes; and
- the language-specific review ledger is empty and the release report records
  all selected profiles, bundles, inputs, manifests, VIR, VCs, certificates,
  and recomputed hashes.
