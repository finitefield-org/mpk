# ProofOps Policy Local Verification

This guide uses the sole active Go/Rust/C#/Java successor CLI. Run policy commands
from a materialized Linux release: the executable resolves both registries,
the selected frontend/toolchain tuple, and all compiled profile contracts from
the installed tree beside `bin/mpk`.

The repository intentionally has no GitHub Actions or workflow files. Start
all policy and release checks locally from reviewed bytes. The authoritative
Go/Rust/C#/Java release command is `scripts/check-java-frontend.sh`; it is not
triggered or monitored through a hosted automation service.

## Trust boundary

Policy scan JSON, evidence metadata, source, contracts, compiler output, VIR,
VC, AI requests, automation logs, and release reports are helper artifacts. A
property may be `mpk_verified` only when its `trusted_evidence` reference
resolves to a canonical certificate declaration or checked theory certificate
accepted by the configured source-free checker path.

## Reserve example

Create an ignored output directory:

```sh
mkdir -p target/proof-ops
```

Run the readiness scan:

```sh
mpk policy scan examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json
```

Run strict verification:

```sh
mpk policy verify examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --evidence-json target/proof-ops/reserve.evidence.json
```

The scan schema is `mpk.policy.scan.v2`; the evidence schema is
`mpk.policy.evidence.v2`. Verification is always strict. Strategy, checker,
axiom, registry, bundle, and toolchain choices are profile-owned or
release-owned and are not public flags.

Go and Rust require at least one normalized `--contract` path. Repeat the flag
when the selection needs multiple sidecars. C# and Java contract paths are
part of their validated selection envelopes and therefore must not be repeated
as CLI flags.

## Sanitized explanation projection

Generate a deterministic helper request without authenticating or accessing a
network:

```sh
mpk explain examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --language en \
  --request-json-out target/proof-ops/reserve.explain-request.json
```

The public CLI stops at `mpk.ai.explain.request.v2`. It does not accept a
provider or credential and does not consume a model response. Keep any
external transmission outside the proof boundary and subject it to separate
customer-consent and data-retention controls.

## Reusable local verification block

Run ordinary source tests and deterministic corpus checks before invoking a
materialized installed release:

```sh
set -eu

(cd examples/payment_policies/reserve && go test -count=1 ./...)
./scripts/regenerate-go-vir-corpus.sh --check
cargo test -p mpk-vc --test go_vir_corpus

mkdir -p target/proof-ops
mpk policy scan examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json
mpk policy verify examples/payment_policies/reserve \
  --semantic-context examples/payment_policies/reserve/mpk-semantic-context.json \
  --selection examples/payment_policies/reserve/mpk-selection.json \
  --contract policy_contract.json \
  --evidence-json target/proof-ops/reserve.evidence.json
```

For release-facing changes, the authoritative repository check is:

```sh
./scripts/build-release-bundles.sh --check-build-inputs rust
./scripts/build-csharp-frontend.sh --check-build-inputs
./scripts/build-java-frontend.sh --check-build-inputs
./scripts/build-java-candidate.sh --check
sudo ./scripts/check-java-frontend.sh
python3 scripts/generate-release-report.py --check
```

The gate requires reviewed Linux bytes, root access to the initial cgroup-v2
namespace, pre-existing frozen Rust, C#, and Java build-input caches, and no
network access.

## Drift and proof checks

Generated frontend and VC fixtures are reviewed together:

```sh
./scripts/regenerate-go-vir-corpus.sh --check
git diff --exit-code -- fixtures/vir-go fixtures/vc-alpha examples
```

Proof checks remain separate:

```sh
mpk check proofs/paymentpolicy.mpcert
mpk axiom-report proofs/paymentpolicy.mpcert
mpk package verify-certs package-manifest.json
```

Do not use a successful frontend run, readiness status, hash match, automation
result, or explanation as a substitute for these checker verdicts.

## Review checklist

- semantic-context and selection envelopes are revision-3 registered values;
- no command accepts a caller-selected registry, bundle, executable,
  toolchain, provider, credential, or compatibility flag;
- `mpk.policy.scan.v2` reports an expected readiness state;
- `mpk.policy.evidence.v2` contains no unresolved strict properties;
- every `mpk_verified` property has a valid `trusted_evidence` link;
- frontend/VC fixture drift is absent or intentionally reviewed;
- both source-free checker paths agree for every proof claim;
- explanation requests and external AI processing remain helper-only.
