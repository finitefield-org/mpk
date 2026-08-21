# ProofOps Policy CI

This guide shows a CI shape for a repository that uses MPK to review one
payment-policy package. It is written as a command block first, not as a
repository workflow file, because this repository does not currently maintain a
GitHub Actions workflow convention.

CI has two separate jobs:

- helper-artifact drift checks for Go source, contract JSON, VIR, VC JSON,
  skeletons, scan JSON, Markdown reports, and generated evidence JSON;
- trusted-evidence checks for canonical `.mpcert` bytes, checked theory
  certificates, checker verdicts, and axiom reports.

CI success does not replace proof evidence. A policy property is proof evidence
only when `mpk.policy.evidence.v1` records checked declaration evidence or
checked theory-certificate evidence under the configured checker profile.

## Repository Example

Run this from the MPK repository root to exercise the reserve policy example.
The commands use the real POE-10 CLI names and the checked-in example paths.

```sh
set -euo pipefail

mkdir -p target/proof-ops
registry_id=mpk.release.registry.v0
registry_sha256=29e4d26c223b90a94684c02246779ab03da6807a78608ef34be628b7c989cf20
frontend_bundle=frontend.go.go2vir.v0
toolchain_bundle=toolchain.go.go1.25.0.linux-amd64.v0

# Helper-artifact generation for the reserve policy.
mpk policy scan examples/payment_policies/reserve \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$registry_id" \
  --require-release-registry-sha256 "$registry_sha256" \
  --frontend-bundle "$frontend_bundle" \
  --toolchain-bundle "$toolchain_bundle" \
  --target linux/amd64 \
  --package example.com/payment/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json

# Non-strict evidence generation for drift review. This writes the helper
# sections and any available trusted evidence, but it is not the strict gate.
mpk policy verify examples/payment_policies/reserve \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$registry_id" \
  --require-release-registry-sha256 "$registry_sha256" \
  --frontend-bundle "$frontend_bundle" \
  --toolchain-bundle "$toolchain_bundle" \
  --target linux/amd64 \
  --package example.com/payment/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --axiom-profile zero-axiom \
  --evidence-json target/proof-ops/reserve.evidence.json \
  --evidence-md target/proof-ops/reserve.evidence.md

# Strict product gate for the supported payment-policy alpha subset.
mpk policy verify examples/payment_policies/reserve \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$registry_id" \
  --require-release-registry-sha256 "$registry_sha256" \
  --frontend-bundle "$frontend_bundle" \
  --toolchain-bundle "$toolchain_bundle" \
  --target linux/amd64 \
  --package example.com/payment/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --axiom-profile zero-axiom \
  --evidence-json target/proof-ops/reserve.strict.evidence.json \
  --evidence-md target/proof-ops/reserve.strict.evidence.md \
  --strict

# Trusted-evidence checker smoke checks against existing repository fixtures.
mpk check fixtures/cert-basic/one-theorem.hex
mpk axiom-report fixtures/cert-basic/one-theorem.hex
mpk package verify-certs \
  fixtures/package-manifest/valid/basic-package.json
```

For the supported reserve policy, the strict `policy verify` command must report
`ok policy verify status=complete`. The evidence report carries the exact
verified/pending/unsupported counts. The non-strict
command is still useful for helper-artifact generation and drift review, but the
strict command is the product gate that fails if any property remains
`proof_pending` or `unsupported`.

The trusted-evidence smoke checks above prove the checkers are running against
existing repository fixtures. They are not proof evidence for the reserve
policy; reserve-specific trust still comes only from checked certificate or
checked theory-certificate entries in the reserve evidence JSON.

If a repository intentionally tracks generated helper fixtures, refresh them in
the CI worktree and then use `git diff --exit-code` as the drift gate. For the
current payment-policy corpus, VC and skeleton drift is checked this way:

```sh
./scripts/regenerate-go-vir-corpus.sh --update
git diff --exit-code -- examples/payment_policies
```

If the repository also tracks `vir.json`, regenerate VIR with the checked
`go2vir` binary before the diff check. For one package:

```sh
(cd examples/payment_policies/reserve && \
  ../../../target/debug/go2vir . > ../../../target/proof-ops/reserve.go2vir.json)
python3 - <<'PY'
import json
from pathlib import Path

raw = json.loads(Path("target/proof-ops/reserve.go2vir.json").read_text())
Path("examples/payment_policies/reserve/vir.json").write_text(
    json.dumps(raw["vir"], indent=2, sort_keys=False) + "\n"
)
PY
git diff --exit-code -- examples/payment_policies/reserve/vir.json
```

## Customer Repository Block

For a customer repository that vendors or installs `mpk` and `go2vir` as
binaries on `PATH`, use the same command names with repository-local paths:

```sh
set -euo pipefail

mkdir -p build/proof-ops

go test -count=1 ./internal/paymentpolicy/...

registry_id=mpk.release.registry.v0
registry_sha256=29e4d26c223b90a94684c02246779ab03da6807a78608ef34be628b7c989cf20
frontend_bundle=frontend.go.go2vir.v0
toolchain_bundle=toolchain.go.go1.25.0.linux-amd64.v0

mpk policy scan ./internal/paymentpolicy \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$registry_id" \
  --require-release-registry-sha256 "$registry_sha256" \
  --frontend-bundle "$frontend_bundle" \
  --toolchain-bundle "$toolchain_bundle" \
  --target linux/amd64 \
  --package example.com/customer/internal/paymentpolicy \
  --function example.com/customer/internal/paymentpolicy.ApprovedReserveCents \
  --contract policy_contract.json \
  --json-out build/proof-ops/paymentpolicy.scan.json

mpk policy verify ./internal/paymentpolicy \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id "$registry_id" \
  --require-release-registry-sha256 "$registry_sha256" \
  --frontend-bundle "$frontend_bundle" \
  --toolchain-bundle "$toolchain_bundle" \
  --target linux/amd64 \
  --package example.com/customer/internal/paymentpolicy \
  --function example.com/customer/internal/paymentpolicy.ApprovedReserveCents \
  --contract policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --axiom-profile zero-axiom \
  --evidence-json build/proof-ops/paymentpolicy.evidence.json \
  --evidence-md build/proof-ops/paymentpolicy.evidence.md \
  --strict
```

Keep `build/proof-ops/*.scan.json`, `*.evidence.json`, and `*.evidence.md` as
CI artifacts for review. Commit generated VIR, VC, and skeleton files only when
the repository intentionally tracks them; otherwise, compare them against a
golden artifact store or upload them as CI artifacts.

For targets inside the supported alpha subset, the customer strict gate should
expect `status=verified`, zero `proof_pending` properties, and zero
`unsupported` properties. If a target is intentionally outside the supported
subset, run non-strict `policy verify` for drift review and do not claim
`mpk_verified` unless the evidence JSON contains checked declaration or checked
theory-certificate references under `trusted_evidence`.

## Drift Checks

Helper-artifact drift is useful because it shows that the function, contract, or
frontend output changed. It is not proof acceptance.

Review these helper artifacts in pull requests:

- `policy.go`, especially changes to branch conditions and returned amounts;
- `policy_contract.json`, especially `requires` and `ensures`;
- generated `vir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- `mpk.policy.scan.v1` output from `mpk policy scan`;
- `mpk.policy.evidence.v1` helper sections and Markdown reports from
  `mpk policy verify`.

Fail CI on unexpected helper drift with a normal diff check, for example:

```sh
git diff --exit-code -- \
  internal/paymentpolicy/vir.json \
  internal/paymentpolicy/vc.json \
  internal/paymentpolicy/vc_skeleton.json
```

Trusted-evidence checks are separate. Run MPK checkers against checked proof
artifacts and fail if the checker rejects them:

```sh
mpk check proofs/paymentpolicy.mpcert
mpk axiom-report proofs/paymentpolicy.mpcert
mpk package verify-certs package-manifest.json
```

Do not use VIR, VC JSON, scan JSON, Markdown, CI status, AI analysis, or web
handler traces as proof evidence. They may explain a review result, but they do
not establish `mpk_verified` by themselves.

## Review Checklist

- Go tests for the policy package and any web handler wrapper pass.
- `mpk policy scan` exits successfully and reports the target as `ready`.
- Non-strict `mpk policy verify` writes deterministic evidence JSON and
  Markdown for drift review.
- Strict `mpk policy verify --strict` passes for supported alpha policies and
  reports zero `proof_pending` and zero `unsupported` properties.
- Helper-artifact drift is either absent or intentionally reviewed.
- Trusted proof artifacts are checked by MPK checkers when the PR claims a
  property is verified.
- The PR description names any properties that remain `proof_pending`,
  `helper_only`, or `unsupported`.
