# MPK: Machine Proof Kernel

MPK is a certificate-first proof kernel and program-verification toolchain for
AI-assisted proof workflows. The current implementation focuses on canonical
proof certificates, source-free checking, a restricted Go frontend, VC
generation, and product-facing payment-policy evidence for ProofOps.

The project is designed around a small trusted base. Go source, contracts, VIR,
VC JSON, AI output, solver answers, Markdown reports, and CI status are useful
engineering artifacts, but they are not proof evidence. The objects that matter
are canonical `.mpcert` bytes, checked theory certificates, deterministic
hashes, axiom reports, and checker verdicts.

```text
untrusted / helper analysis:
  Go source / contract JSON / go2vir / VIR / VC JSON
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
- an untrusted `go2vir` frontend for a restricted Go subset;
- VC generation for supported VIR functions;
- `mpk policy scan` with schema `mpk.policy.scan.v1`;
- `mpk policy verify` with schema `mpk.policy.evidence.v1`;
- payment-policy examples for reserve, refund, discount, fee, and points.

The ProofOps-facing policy path currently distinguishes:

- `strategy_profile`: product workflow selection such as
  `payment-policy-alpha`;
- `checker_profile`: MPK checker mode such as `mvp-strict`;
- `axiom_profile`: axiom policy allowlist such as `zero-axiom`.

These fields must remain separate in product reports and integrations.

## Continuous Integration

This repository does not use GitHub Actions and will not add GitHub Actions
workflows. Project checks are run explicitly with the repository's local
commands and scripts.

## Build From Source

Install Rust and Go, then build the CLI:

```sh
cargo build -p mpk-cli
```

Run the binary from the repository build output:

```sh
target/debug/mpk --help
```

Build the Go frontend for development and corpus checks:

```sh
(cd go-tools/go2vir && go build -o ../../target/debug/go2vir .)
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
without expanding MPK's trusted boundary. Production policy commands resolve
the frontend and toolchain only from the registry installed beside `bin/mpk`;
they do not accept executable or registry paths.

Select the installed Go release tuple:

```sh
mkdir -p target/proof-ops
export MPK_RELEASE_REGISTRY_ID=mpk.release.registry.v0
export MPK_RELEASE_REGISTRY_SHA256=29e4d26c223b90a94684c02246779ab03da6807a78608ef34be628b7c989cf20
export MPK_GO_FRONTEND_BUNDLE=frontend.go.go2vir.v0
export MPK_GO_TOOLCHAIN_BUNDLE=toolchain.go.go1.25.0.linux-amd64.v0
```

Scan the reserve policy:

```sh
cargo run --quiet -p mpk-cli -- policy scan examples/payment_policies/reserve \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$MPK_RELEASE_REGISTRY_ID" \
  --require-release-registry-sha256 "$MPK_RELEASE_REGISTRY_SHA256" \
  --frontend-bundle "$MPK_GO_FRONTEND_BUNDLE" \
  --toolchain-bundle "$MPK_GO_TOOLCHAIN_BUNDLE" \
  --target linux/amd64 \
  --package example.com/payment/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json
```

Verify the reserve policy and write product evidence:

```sh
cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$MPK_RELEASE_REGISTRY_ID" \
  --require-release-registry-sha256 "$MPK_RELEASE_REGISTRY_SHA256" \
  --frontend-bundle "$MPK_GO_FRONTEND_BUNDLE" \
  --toolchain-bundle "$MPK_GO_TOOLCHAIN_BUNDLE" \
  --target linux/amd64 \
  --package example.com/payment/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --axiom-profile zero-axiom \
  --evidence-json target/proof-ops/reserve.evidence.json \
  --evidence-md target/proof-ops/reserve.evidence.md
```

The scan JSON is helper analysis. The evidence JSON is the product API. A
property is `mpk_verified` only when it references checked declaration evidence
or checked theory-certificate evidence under `trusted_evidence`; VIR, VC JSON,
Markdown, CI status, and Gemini output remain helper analysis.

## Optional Vertex AI Explanation

The opt-in `mpk explain` command is available only in a build with the
`vertex-ai` feature:

```sh
cargo build -p mpk-cli --features vertex-ai
export GOOGLE_CLOUD_PROJECT="your-lowercase-project-id"
export GOOGLE_CLOUD_LOCATION="global"
gcloud services enable aiplatform.googleapis.com \
  --project "$GOOGLE_CLOUD_PROJECT"
gcloud auth application-default login
gcloud auth application-default set-quota-project "$GOOGLE_CLOUD_PROJECT"
mkdir -p target/proof-ops
```

Use a dedicated billed non-production project with approved quotas and budget
alerts. A cloud administrator must enable the Vertex AI API. The calling
identity needs `roles/aiplatform.user`; when user ADC assigns a quota project,
it also needs `serviceusage.services.use` on that project, normally through
`roles/serviceusage.serviceUsageConsumer`. Grant these roles only at the
narrowest practical scope. Do not approve live use until the `GEMINI-AUX-05`
English and Japanese release gate in the design document has been completed.

Before sending anything, inspect the exact credential-free request body with
the offline dry run:

```sh
target/debug/mpk explain target/proof-ops/reserve.evidence.json \
  --provider vertex-ai \
  --language en \
  --dry-run \
  --request-json-out target/proof-ops/reserve.explain-request.json
```

Normal mode obtains a short-lived token from local ADC through
`gcloud auth application-default print-access-token --quiet`, then sends the
allowlisted, redacted payload to Vertex AI. It writes separate JSON and
Markdown helper reports:

```sh
target/debug/mpk explain target/proof-ops/reserve.evidence.json \
  --provider vertex-ai \
  --output-json target/proof-ops/reserve.ai-explanation.json \
  --output-md target/proof-ops/reserve.ai-explanation.md
```

Every report is explicitly untrusted helper analysis. The JSON carries
`proof_evidence: false`, and the Markdown warning is inserted locally before
any generated text. MPK restores property IDs and statuses only from the
validated local evidence; AI prose cannot change proof results. Remote
processing, project metadata, source-evidence hashes, and provider retention
or abuse-monitoring policies still require customer consent and a current
Google Cloud data-governance review. Do not treat this feature as a zero-
retention guarantee, and never commit real request previews or reports.

To remove local ADC after use, run `gcloud auth application-default revoke`.

## Repository Layout

```text
.
+-- crates/
|   +-- mpk-core/      trusted core terms, names, environments, and checking
|   +-- mpk-cert/      canonical certificate encoding and hash material
|   +-- mpk-kernel/    certificate verifier and JSON verdict output
|   +-- mpk-theory/    checked theory-certificate implementations
|   +-- mpk-vc/        VIR import, VC generation, and policy classification
|   +-- mpk-api/       untrusted API and strategy orchestration
|   +-- mpk-cli/       installed `mpk` command
+-- go-tools/
|   +-- go2vir/        untrusted Go subset frontend
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
- [VIR v0](develop/specs/VIR_V0.md)
- [Go VIR Profile v0](develop/specs/GO_VIR_PROFILE_V0.md)
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
cargo test -p mpk-cli --test policy_cli
cargo test -p mpk-cli --test policy_report
cargo test -p mpk-vc --test go_vir_corpus
(cd go-tools/go2vir && go test -count=1 ./...)
```

## License

MPK is licensed under the [Apache License 2.0](LICENSE).

Copyright 2026 [Finite Field, K.K.](https://finitefield.org/en/). See
[NOTICE](NOTICE).
