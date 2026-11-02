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

## Gate I: Go/Rust successor release gate

The active release contains only Go and Rust. Provision the frozen Rust
build-input cache explicitly before entering networkless verification:

```sh
sudo ./scripts/build-release-bundles.sh --provision-build-inputs rust
```

On a reviewed native x86-64 Linux host with root, strace and writable global
cgroup v2, run the local gates from the root README:

```sh
sudo ./scripts/check-go-rust-frontend.sh
sudo ./scripts/check-all.sh
```

The frontend gate validates the frozen build-input cache, rebuilds the
registered Go/Rust release, exercises both Rust target libraries and both
frontends, and repeats the installed release pass twice. It covers actual
source artifacts, paired source-free checking, determinism, limits, fuzz smoke
and obsolete-interface review. Old receipts apply only to their original
registry and bundle identities; revision 4 needs fresh native release evidence.

Verification uses reviewed local tools, digest-pinned images and complete
registered inventories. Missing caches, tools or images fail closed; the gate
does not download or upgrade dependencies. Privileged setup creates the
required cgroup and fixed tmpfs boundary. Compiler, frontend and program work
runs without host privileges inside the release sandbox. Use an externally
egress-denied disposable runner for privileged release verification. GitHub
Actions and workflow files are not used.

Compiler and build-closure changes follow
[the Rust toolchain upgrade procedure](../docs/rust-frontend-toolchain-upgrade.md).
Gate reports and provenance remain untrusted; canonical certificate bytes and
checked theory certificates determine proof acceptance.

## Gate J: Go/Rust capability-profile expansion

The [language scope](../specs/LANGUAGE_SUPPORT_SCOPE_V1.md) permits Go and Rust
only. The former other-language admission queue and release commands are
retired. Practical Go follows its
[work plan](../docs/09_go_practical_subset_design-todo.md) and remains inactive
until all required implementation and release gates pass.

Before activating a widened Go/Rust profile:

- freeze its exact supported subset, rejection taxonomy, semantic profile,
  contract syntax, compiler/API boundary, target, fixtures, limits and version
  policy;
- register reproducible frontend/toolchain identities and resolve one exact
  language/profile tuple before lowering; reject unknown or crossed contexts;
- bind every affected producer and consumer, shared artifact, manifest, policy,
  recipe and release inventory to the same current identities;
- preserve language-isolated VIR, Certificate v0, axiom categories and both
  source-free checker acceptance rules;
- pass the required positive, negative, boundary, differential, adversarial,
  determinism and fuzz checks with actual certificate bytes; and
- close the phase's review ledger and pass the complete native Linux release
  gate before claiming release acceptance.

During intermediate W items, run only affected local tests, lint/format and
review. Run `./scripts/check-fast.sh` over cumulative T changes in its final W;
that W and T remain incomplete until it passes. No language-specific skip mode
or substitute aggregate gate is part of this cadence.
