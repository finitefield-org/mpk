# MPK: Machine Proof Kernel

MPK is a certificate-first proof kernel and program-verification toolchain for
AI-assisted proof workflows. The current implementation focuses on canonical
proof certificates, source-free checking, a restricted Go frontend, VC
generation, and product-facing payment-policy evidence for ProofOps.

The project is designed around a small trusted base. Go source, contracts, GIR,
VC JSON, AI output, solver answers, Markdown reports, and CI status are useful
engineering artifacts, but they are not proof evidence. The objects that matter
are canonical `.mpcert` bytes, checked theory certificates, deterministic
hashes, axiom reports, and checker verdicts.

```text
untrusted / helper analysis:
  Go source / contract JSON / go2gir / GIR / VC JSON
  AI traces / solver answers / report prose / CI status / web logs

trusted proof evidence:
  canonical .mpcert bytes
  checked theory certificates
  Rust kernel / verifier verdicts
  independent source-free reference checker verdicts
  deterministic export_hash, certificate_hash, and axiom_report_hash
```

MPK is not a production replacement for Lean or Rocq. It is a research and
implementation repository for a proof-certificate-centered verification
toolchain and its first Go-policy product path.

## Current Status

The workspace version is `0.1.0`. The installed CLI binary is `mpk`.

Implemented paths include:

- canonical certificate encoding and source-free verification;
- Rust kernel checks for core proof certificates;
- checked theory-certificate support for selected theory proofs;
- package manifest validation and certificate verification;
- an untrusted `go2gir` frontend for a restricted Go subset;
- VC generation for supported GIR functions;
- `mpk policy scan` with schema `mpk.policy.scan.v0`;
- `mpk policy verify` with schema `mpk.policy.evidence.v0`;
- payment-policy examples for reserve, refund, discount, fee, and points.

The ProofOps-facing policy path currently distinguishes:

- `strategy_profile`: product workflow selection such as
  `payment-policy-alpha`;
- `checker_profile`: MPK checker mode such as `mvp-strict`;
- `allowed_axiom_profiles`: axiom policy allowlist such as `zero-axiom`.

These fields must remain separate in product reports and integrations.

## Build From Source

Install Rust and Go, then build the CLI:

```sh
cargo build -p mpk-cli
```

Run the binary from the repository build output:

```sh
target/debug/mpk --help
```

Build the Go frontend used by policy scan and verify:

```sh
(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)
```

## Certificate Verification Quick Start

Verify a canonical certificate fixture:

```sh
cargo run --quiet -p mpk-cli -- check fixtures/cert-basic/one-theorem.hex
```

Expected result: JSON with `"verdict":"accepted"`.

Verify a package manifest that requires source-free checking and the independent
reference checker:

```sh
cargo run --quiet -p mpk-cli -- package verify-certs \
  fixtures/package-manifest/valid/basic-package.json
```

Expected result:

```text
ok package=Example.Basic.Package source_free=1 reference=1
```

## ProofOps Policy Quick Start

The payment-policy path is the product-facing integration surface for
`../proof-ops`. It produces deterministic helper artifacts and evidence JSON
without expanding MPK's trusted boundary.

Build `go2gir` once:

```sh
(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)
mkdir -p target/proof-ops
```

Scan the reserve policy:

```sh
cargo run --quiet -p mpk-cli -- policy scan examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json \
  --go2gir target/debug/go2gir
```

Verify the reserve policy and write product evidence:

```sh
cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json target/proof-ops/reserve.evidence.json \
  --evidence-md target/proof-ops/reserve.evidence.md \
  --go2gir target/debug/go2gir
```

The scan JSON is helper analysis. The evidence JSON is the product API. A
property is `mpk_verified` only when it references checked declaration evidence
or checked theory-certificate evidence under `trusted_evidence`; GIR, VC JSON,
Markdown, CI status, and Gemini output remain helper analysis.

## Repository Layout

```text
.
+-- crates/
|   +-- mpk-core/      trusted core terms, names, environments, and checking
|   +-- mpk-cert/      canonical certificate encoding and hash material
|   +-- mpk-kernel/    certificate verifier and JSON verdict output
|   +-- mpk-theory/    checked theory-certificate implementations
|   +-- mpk-vc/        GIR import, VC generation, and policy classification
|   +-- mpk-api/       untrusted API and strategy orchestration
|   +-- mpk-cli/       installed `mpk` command
+-- go-tools/
|   +-- go2gir/        untrusted Go subset frontend
|   +-- mpk-checker-ref/ independent source-free checker prototype
+-- docs/              user-facing ProofOps and integration documentation
+-- develop/           specs, roadmap, release gates, and internal design docs
+-- examples/          Max64, order policy, and payment-policy examples
+-- fixtures/          certificate, package, Go, and VC regression fixtures
+-- proofs/            checked proof fixtures and standard-library material
+-- fuzz/              fuzz targets for malformed proof inputs
+-- scripts/           local verification gates
```

## Documentation

Start with the current user-facing guides:

- [Alpha Demo Guide](docs/alpha-demo.md)
- [ProofOps Engine Support Design](docs/proof-ops-engine-design.md)
- [ProofOps Policy CI](docs/proof-ops-policy-ci.md)
- [Vertex AI Gemini Assistant Design](docs/vertex-ai-gemini-assistant-design.md)
- [Web System Integration Guide](docs/web-system-integration.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

Developer-facing specs, roadmap, and release gates are routed from
[Development Documentation](develop/README.md). The most important trust-boundary
references are:

- [Trust Boundary v0](develop/specs/TRUST_BOUNDARY_V0.md)
- [Certificate Format v0](develop/specs/CERT_V0.md)
- [GIR v0](develop/specs/GIR_V0.md)
- [Go Subset v0](develop/specs/GO_SUBSET_V0.md)
- [Axiom Policy v0](develop/specs/AXIOM_POLICY_V0.md)

## Local Development Gates

For ordinary development, start with the fast gate:

```sh
./scripts/check-fast.sh
```

For a fuller local release-style check:

```sh
./scripts/check-all.sh
```

Useful targeted gates:

```sh
cargo test --workspace
cargo test -p mpk-cli --test policy_scan
cargo test -p mpk-cli --test policy_verify
cargo test -p mpk-vc --test payment_policy_examples
(cd go-tools/go2gir && go test -count=1 ./...)
```

## License

MPK is licensed under the [Apache License 2.0](LICENSE).

Copyright 2026 [Finite Field, K.K.](https://finitefield.org/en/). See
[NOTICE](NOTICE).
