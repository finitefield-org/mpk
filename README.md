# MPK: Machine Proof Kernel

MPK is a certificate-first proof kernel and program-verification toolchain for
AI-assisted proof workflows. The toolchain accepts restricted Go and Rust
programs through one registered successor pipeline and produces deterministic
verification artifacts.

The trusted boundary is deliberately small. Source, contracts, compilers,
frontends, VIR, VC JSON, policy reports, AI requests, Markdown, CI status, and
release reports are untrusted helper artifacts. Proof acceptance comes only
from canonical `.mpcert` bytes or checked theory certificates accepted by the
source-free Rust checker and, where required, the independent reference
checker.

## Current status

MPK supports Go and Rust only. The installed semantic-profile registry is
revision 4, with the Go fixed and Rust checked profiles. The canonical
`mpk.release.bundle_registry.v1` contains two frontend/toolchain pairs and
three tuples: Go `linux/amd64`, Rust `i686-unknown-linux-gnu`, and Rust
`x86_64-unknown-linux-gnu`. Removed and unknown languages fail closed.
See the [language scope](develop/specs/LANGUAGE_SUPPORT_SCOPE_V1.md).

Practical Go expansion continues in the [Go work plan](develop/docs/09_go_practical_subset_design-todo.md)
and [implementation ledger](develop/docs/go-practical-implementation-traceability-ledger.md).
T02-W01 remains in progress; the practical Go profile is not installed.
The new revision-4 release metadata requires its own native Linux release
validation. Previous release receipts apply to their original registry bytes.
Certificate v0 and both source-free checking paths retain their existing
formats and acceptance rules.

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

Each request supplies only a source root, a revision-4 semantic-context
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
[the Rust example](examples/rust-payment-policy/README.md).

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
materializes one installed release, runs both registered frontends,
checks both source-free verifiers, and repeats the installed-release pass
twice.

For development organized into stages (`Tnn`) and work items (`Wnn`), run
targeted tests for the changed code and affected consumers, appropriate
lint/format checks, and review for each W. Run the following workspace gate
only in the final W of a T, covering all changes in that T:

```sh
./scripts/check-fast.sh
```

Do not reproduce the full gate by running all its components separately for
each W. Record targeted results separately from the full gate deferred to T
completion. The final W and its T cannot be marked complete, and the next T
cannot begin, until the gate passes after any required fixes. An explicit user
request to run the full gate takes precedence over this cadence. This cadence
does not change the release checks below.

The fast gate fails if `.github/workflows/` contains any entry.

Prepare the frozen build-input caches explicitly before entering the
networkless gate:

```sh
sudo ./scripts/build-release-bundles.sh --provision-build-inputs rust
```

Then run the full local release gate on a reviewed Linux host with writable
cgroup v2:

```sh
sudo ./scripts/check-go-rust-frontend.sh
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
cargo test -p mpk-vc --test go_vir_corpus
(cd go-tools/go2vir && go test -count=1 ./...)
```

## Repository layout

```text
crates/                 kernel, certificates, VC, API, and installed CLI
go-tools/go2vir/        restricted Go successor frontend
rust-tools/rust2vir/    restricted Rust successor frontend and private driver
release/                frozen build inputs and active successor registries
docs/                   user-facing operation and integration guides
develop/                normative specs, vectors, roadmaps, and migration logs
fixtures/               certificate and frontend regression corpora
scripts/                generation and verification gates
```

## Documentation

- [Alpha Demo Guide](docs/alpha-demo.md)
- [Go Practical Expansion Design](develop/docs/09_go_practical_subset_design.md)
- [Go Practical Expansion Work Plan](develop/docs/09_go_practical_subset_design-todo.md)
- [ProofOps Policy Local Verification](docs/proof-ops-policy-ci.md)
- [ProofOps Engine Support Design](docs/proof-ops-engine-design.md)
- [Web System Integration Guide](docs/web-system-integration.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Development Documentation](develop/README.md)

Normative scope and registry contracts are in
[`LANGUAGE_SUPPORT_SCOPE_V1.md`](develop/specs/LANGUAGE_SUPPORT_SCOPE_V1.md) and
[`SEMANTIC_PROFILE_REGISTRY_V1.md`](develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md).

## License

MPK is licensed under the [Apache License 2.0](LICENSE).

Copyright 2026 [Finite Field, K.K.](https://finitefield.org/en/). See
[NOTICE](NOTICE).
