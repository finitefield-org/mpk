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

The generic VIR branch of this gate is frozen now but activates only in the
atomic Go/VIR cutover. Until that cutover, the current Go/GIR gate and its
frozen specifications remain the active release requirement and are not
historical. A release must use one complete branch; mixing pre-cutover and
post-cutover schemas is forbidden.

For the post-cutover branch:

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

For the pre-cutover branch, unsupported Go features still fail closed and the
Go source, Go version, frontend binary, GIR, and VC identities are recorded as
required by the current frozen Go/GIR documents. The atomic cutover replaces
this branch; only then are those documents labeled historical.

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
