# MPK: Machine Proof Kernel

MPK is a certificate-first proof kernel and program-verification toolchain for
AI-assisted proof workflows. The active release accepts restricted Go, Rust,
and C# programs through one registered successor pipeline and produces
deterministic verification artifacts.

The trusted boundary is deliberately small. Source, contracts, compilers,
frontends, VIR, VC JSON, policy reports, AI requests, Markdown, CI status, and
release reports are untrusted helper artifacts. Proof acceptance comes only
from canonical `.mpcert` bytes or checked theory certificates accepted by the
source-free Rust checker and, where required, the independent reference
checker.

## Current status

The workspace version is `0.1.0`, and the installed CLI is `mpk`.
`CSHARP-02-T20` completed the atomic successor cutover:

- semantic-profile registry revision 2 is the only active profile registry;
- `mpk.release.registry.v1` is the only active bundle registry;
- Go, Rust, and C# resolve exact frontend/toolchain tuples from the installed
  release beside `bin/mpk`;
- policy scan and evidence use `mpk.policy.scan.v2` and
  `mpk.policy.evidence.v2`;
- `mpk explain` emits only a deterministic, sanitized
  `mpk.ai.explain.request.v2` helper request;
- callers cannot supply registry paths, executable paths, bundle identities,
  toolchain roots, provider credentials, or compatibility flags.

Certificate v0 and both source-free checking paths remain unchanged by this
frontend migration.

`JAVA-03-T01` completed the inactive [Java 25 profile freeze](develop/specs/JAVA_PROFILE_V0.md),
including conformance vectors and pinned compiler/JVM compatibility evidence.
`JAVA-03-T02` added the [offline build candidate](java-tools/README.md), with
two isolated builds and exact source/class/JAR inventories. Next is T03's
inactive profile and artifact validators in the
[Java implementation ledger](develop/docs/java-03-implementation-traceability-ledger.md).
Java source processing is not implemented or active; the installed registry remains revision 2.
The T01 Linux amd64 probe used CPU emulation and does not replace the later
native Linux release gate.

## Build from source

Install Rust, Go, and Python 3, then build the workspace CLI:

```sh
cargo build -p mpk-cli
target/debug/mpk --help
```

The source checkout is suitable for development tests. Policy commands must
run from a materialized Linux release because their frontend and toolchain
bundles are resolved relative to the installed `bin/mpk`.

## Certificate verification

Verify a canonical fixture:

```sh
cargo run --quiet -p mpk-cli -- check fixtures/cert-basic/one-theorem.hex
```

Verify a package whose certificates require both source-free checking paths:

```sh
cargo run --quiet -p mpk-cli -- package verify-certs \
  fixtures/package-manifest/valid/basic-package.json
```

## Successor policy CLI

Each request supplies only a source root, a revision-2 semantic-context
envelope, a selection envelope, normalized Go/Rust contract paths, and an
output path. The installed release selects every security-sensitive identity.

Scan the Go reserve example:

```sh
mkdir -p target/proof-ops
mpk policy scan examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json
```

Verify it and emit strict evidence:

```sh
mpk policy verify examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --evidence-json target/proof-ops/reserve.evidence.json
```

The profile-owned policy contract fixes the strategy, checker, and axiom
profiles. Verification is strict; there is no public flag that weakens it.
Only `trusted_evidence` links backed by accepted certificates or checked
theory certificates can support an `mpk_verified` property.

Rust uses the same command shape and may repeat `--contract`; see
[the Rust example](examples/rust-payment-policy/README.md). C# contract paths
come from the validated C# selection envelope, so C# requests do not accept
`--contract`.

## Sanitized explanation request

Generate an English request projection without credentials or network access:

```sh
mpk explain examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --language en \
  --request-json-out target/proof-ops/reserve.explain-request.json
```

The output is untrusted helper analysis. The public CLI does not authenticate,
select a provider, contact an AI service, consume a response, or alter policy
evidence. Any external system that sends the request is independently
responsible for consent, credentials, retention, and provider governance.

## Local verification gates

This repository intentionally does not use GitHub Actions or workflow files.
Do not create, trigger, monitor, or rely on `.github/workflows/`. All checks
must be started locally from reviewed bytes. The full gate validates frozen
inputs, uses digest-pinned images, disables network access during verification,
materializes one installed release, runs all three registered frontends,
checks both source-free verifiers, and repeats the installed-release pass
twice.

Ordinary development:

```sh
./scripts/check-fast.sh
```

The fast gate fails if `.github/workflows/` contains any entry.

Prepare the frozen build-input caches explicitly before entering the
networkless gate:

```sh
sudo ./scripts/build-release-bundles.sh --provision-build-inputs rust
./scripts/build-csharp-frontend.sh --provision-build-inputs
```

Then run the full local release gate on a reviewed Linux host with writable
cgroup v2:

```sh
sudo ./scripts/check-csharp-frontend.sh
sudo ./scripts/check-all.sh
```

Provisioning is a separate setup step; the verification gates never download
or upgrade a toolchain. Root is used by the gates only to create the release
sandbox's delegated cgroup and fixed `noswap` tmpfs; frontend and compiler work
starts after host privileges are removed. Run privileged gates only from
reviewed bytes on an externally egress-denied disposable runner.

Useful targeted checks:

```sh
cargo test --workspace
cargo test -p mpk-cli --test successor_atomic_cutover
cargo test -p mpk-cli --test csharp_policy_verify
cargo test -p mpk-vc --test go_vir_corpus
(cd go-tools/go2vir && go test -count=1 ./...)
```

## Repository layout

```text
crates/                 kernel, certificates, VC, API, and installed CLI
go-tools/go2vir/        restricted Go successor frontend
rust-tools/rust2vir/    restricted Rust successor frontend and private driver
csharp-tools/csharp2vir restricted C# successor frontend
release/                frozen build inputs and active successor registries
docs/                   user-facing operation and integration guides
develop/                normative specs, vectors, roadmaps, and migration logs
examples/               Go and Rust policy examples
fixtures/               certificate and frontend regression corpora
scripts/                generation and verification gates
```

## Documentation

- [Alpha Demo Guide](docs/alpha-demo.md)
- [ProofOps Policy Local Verification](docs/proof-ops-policy-ci.md)
- [ProofOps Engine Support Design](docs/proof-ops-engine-design.md)
- [Web System Integration Guide](docs/web-system-integration.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Development Documentation](develop/README.md)

Normative successor contracts are in
[`SEMANTIC_PROFILE_REGISTRY_V1.md`](develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md)
and [`CSHARP_PROFILE_V0.md`](develop/specs/CSHARP_PROFILE_V0.md).

## License

MPK is licensed under the [Apache License 2.0](LICENSE).

Copyright 2026 [Finite Field, K.K.](https://finitefield.org/en/). See
[NOTICE](NOTICE).
