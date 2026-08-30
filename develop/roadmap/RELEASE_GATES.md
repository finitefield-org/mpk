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

## Gate I: Go/Rust/C# successor final release gate

The Go/Rust/C# release closes only through `scripts/check-csharp-frontend.sh` in the
same order locally and in clean Linux CI:

- validate the tracked Rust and C# build-input descriptors and unchanged
  content-addressed caches before any byte is mounted or executed;
- rebuild and validate the registered Go/Rust/C# release, install only
  registered bundle inventories, exercise both Rust target libraries and C#,
  and reject removed predecessor publication paths and commands;
- run all three frontends, the active corpora, both
  source-free checkers, artifact-path scan, two-clean-build differential suite,
  exact limits, bounded fuzz smoke, and strict obsolete-interface scan; and
- regenerate the untrusted release provenance from registry, bundle, build
  closure, manifest, VIR, VC, certificate, checker, axiom, determinism, path,
  and zero-finding review records.

A CI cache hit is not evidence: restored and newly provisioned closures run the
same `--check-build-inputs rust` and C# `--check-build-inputs` gates before use.
On a Rust cache miss, CI provisions the closure as root because the frozen
builder validates and delegates the same global cgroup-v2 hierarchy required
by the installed release gate; compiler and build containers still run as the
fixed unprivileged identity. C# archive provisioning does not use that host
boundary.
Verification is networkless and cannot invoke an implicit rustup, Cargo, Go,
container-image, or dependency download. The digest-pinned Go bundle-build and
Rust runtime images are materialized before network isolation and every
verification container uses `--pull=never` plus `--network=none`.
The privileged workflow has no pull-request trigger: it runs reviewed `main`
bytes or a write-access-controlled manual ref, makes host tool/cache roots
root-owned, and starts with an empty environment and a fixed tool path. Its
network namespace is an operational no-fetch control for that reviewed code,
not containment against hostile root code. Testing an untrusted ref requires a
separate disposable runner whose egress is denied outside the guest and which
exposes no host control socket.
The installed Rust scan requires the initial cgroup namespace, a writable
global cgroup-v2 hierarchy, and mount privileges for fixed `noswap` tmpfs
backing. CI therefore runs the aggregate gate as root inside a networkless
namespace and gives `mpk` one fresh, otherwise empty delegated cgroup domain.
The untrusted frontend, rustc, Cargo, and generated program receive none of
those host privileges: the release bootstrap enters user and execution
namespaces, exposes only fixed bind views, and sets `no_new_privileges` before
starting them. Local runs require the same root/cgroup capability boundary.
Compiler and build-closure changes follow
`../docs/rust-frontend-toolchain-upgrade.md` as one manually reviewed
transaction. CI success and every reported provenance field remain untrusted;
proof acceptance still requires canonical certificate or checked-theory bytes
accepted by the configured source-free checkers.

## Gate J: New source-language admission gate

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
