# Contributing To MPK

Thank you for considering a contribution to MPK.

MPK is a certificate-first proof kernel and program-verification toolchain. The
most important contribution rule is to preserve the trust boundary: source,
frontend output, VC JSON, AI output, Markdown, and CI status are helper analysis
only. A proof claim must be backed by canonical certificate bytes, checked
theory certificates, checker verdicts, deterministic hashes, and axiom reports.

## Before You Start

Read:

- [README.md](README.md)
- [Trust Boundary v0](develop/specs/TRUST_BOUNDARY_V0.md)
- [Certificate Format v0](develop/specs/CERT_V0.md)
- [Axiom Policy v0](develop/specs/AXIOM_POLICY_V0.md)
- [ProofOps Engine Support Design](docs/proof-ops-engine-design.md), for
  policy scan and policy verify changes

## Development Setup

Install Rust, Go, and Python 3. Then build the CLI and Go frontend:

```sh
cargo build -p mpk-cli
(cd go-tools/go2vir && go build -o ../../target/debug/go2vir .)
```

## Local Gates

For ordinary changes, run:

```sh
./scripts/check-fast.sh
```

For release-facing or cross-boundary changes, run:

```sh
./scripts/check-all.sh
```

Targeted checks are acceptable while iterating:

```sh
cargo test --workspace
cargo test -p mpk-cli --test policy_cli
cargo test -p mpk-cli --test policy_report
cargo test -p mpk-vc --test go_vir_corpus
(cd go-tools/go2vir && go test -count=1 ./...)
```

Always run:

```sh
cargo fmt
git diff --check
```

## Trust-Boundary Rules

- Do not make Go source, contracts, VIR, VC JSON, Markdown, CI status, AI
  output, or solver yes/no output proof evidence.
- Do not mark a property `mpk_verified` unless it references checked
  declaration evidence or checked theory-certificate evidence under
  `trusted_evidence`.
- Keep `strategy_profile`, `checker_profile`, and `axiom_profile`
  distinct.
- Keep product-facing JSON deterministic.
- Do not add local absolute paths, secrets, or private customer code to stable
  fixtures or reports.
- Unsupported input should fail closed with deterministic diagnostics.

## Policy Engine Changes

For `mpk policy scan` and `mpk policy verify` changes, update the relevant docs
and tests together:

- [docs/proof-ops-engine-design.md](docs/proof-ops-engine-design.md)
- [docs/proof-ops-policy-ci.md](docs/proof-ops-policy-ci.md)
- [docs/alpha-demo.md](docs/alpha-demo.md)
- `crates/mpk-cli/tests/policy_cli.rs`
- `crates/mpk-cli/tests/policy_report.rs`

Keep `mpk.policy.scan.v1` as helper analysis. Keep `mpk.policy.evidence.v1` as
the product source of truth.

## Pull Requests

PR descriptions should include:

- what changed;
- which trust-boundary surface is affected;
- verification commands run;
- any remaining `proof_pending`, `helper_only`, or `unsupported` behavior;
- whether fixtures were intentionally regenerated.

Do not include private customer code or secrets in issues, PRs, fixtures, or
logs.
