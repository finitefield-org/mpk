# ProofOps Policy CI

This guide shows a CI shape for a repository that uses MPK to review one
payment-policy package. It is written as a command block first, not as a
repository workflow file, because this repository does not currently maintain a
GitHub Actions workflow convention.

CI has two separate jobs:

- helper-artifact drift checks for Go source, contract JSON, GIR, VC JSON,
  skeletons, scan JSON, Markdown reports, and generated evidence JSON;
- trusted-evidence checks for canonical `.mpcert` bytes, checked theory
  certificates, checker verdicts, and axiom reports.

CI success does not replace proof evidence. A policy property is proof evidence
only when `mpk.policy.evidence.v0` records checked declaration evidence or
checked theory-certificate evidence under the configured checker profile.

## Repository Example

Run this from the MPK repository root to exercise the reserve policy example.
The commands use the real POE-10 CLI names and the checked-in example paths.

```sh
set -euo pipefail

mkdir -p target/proof-ops

(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)

# Helper-artifact generation for the reserve policy.
cargo run --quiet -p mpk-cli -- policy scan examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json \
  --go2gir target/debug/go2gir

# Non-strict evidence generation for drift review. This writes the helper
# sections and any available trusted evidence, but it is not the strict gate.
cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json target/proof-ops/reserve.evidence.json \
  --evidence-md target/proof-ops/reserve.evidence.md \
  --go2gir target/debug/go2gir

# Strict product gate for the supported payment-policy alpha subset.
cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json target/proof-ops/reserve.strict.evidence.json \
  --evidence-md target/proof-ops/reserve.strict.evidence.md \
  --go2gir target/debug/go2gir \
  --strict

# Trusted-evidence checker smoke checks against existing repository fixtures.
cargo run --quiet -p mpk-cli -- check fixtures/cert-basic/one-theorem.hex
cargo run --quiet -p mpk-cli -- axiom-report fixtures/cert-basic/one-theorem.hex
cargo run --quiet -p mpk-cli -- package verify-certs \
  fixtures/package-manifest/valid/basic-package.json
```

For the supported reserve policy, the strict `policy verify` command must report
`status=verified verified=8 proof_pending=0 unsupported=0`. The non-strict
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
MPK_UPDATE_PAYMENT_POLICY_EXAMPLES=1 cargo test -p mpk-vc --test payment_policy_examples
git diff --exit-code -- examples/payment_policies
```

If the repository also tracks `gir.json`, regenerate GIR with the checked
`go2gir` binary before the diff check. For one package:

```sh
(cd examples/payment_policies/reserve && \
  ../../../target/debug/go2gir . > ../../../target/proof-ops/reserve.go2gir.json)
python3 - <<'PY'
import json
from pathlib import Path

raw = json.loads(Path("target/proof-ops/reserve.go2gir.json").read_text())
Path("examples/payment_policies/reserve/gir.json").write_text(
    json.dumps(raw["gir"], indent=2, sort_keys=False) + "\n"
)
PY
git diff --exit-code -- examples/payment_policies/reserve/gir.json
```

## Customer Repository Block

For a customer repository that vendors or installs `mpk` and `go2gir` as
binaries on `PATH`, use the same command names with repository-local paths:

```sh
set -euo pipefail

mkdir -p build/proof-ops

go test -count=1 ./internal/paymentpolicy/...

mpk policy scan ./internal/paymentpolicy \
  --function example.com/customer/internal/paymentpolicy.ApprovedReserveCents \
  --contract ./internal/paymentpolicy/policy_contract.json \
  --json-out build/proof-ops/paymentpolicy.scan.json \
  --go2gir go2gir

mpk policy verify ./internal/paymentpolicy \
  --function example.com/customer/internal/paymentpolicy.ApprovedReserveCents \
  --contract ./internal/paymentpolicy/policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json build/proof-ops/paymentpolicy.evidence.json \
  --evidence-md build/proof-ops/paymentpolicy.evidence.md \
  --go2gir go2gir \
  --strict
```

Keep `build/proof-ops/*.scan.json`, `*.evidence.json`, and `*.evidence.md` as
CI artifacts for review. Commit generated GIR, VC, and skeleton files only when
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
- generated `gir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- `mpk.policy.scan.v0` output from `mpk policy scan`;
- `mpk.policy.evidence.v0` helper sections and Markdown reports from
  `mpk policy verify`.

Fail CI on unexpected helper drift with a normal diff check, for example:

```sh
git diff --exit-code -- \
  internal/paymentpolicy/gir.json \
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

Do not use GIR, VC JSON, scan JSON, Markdown, CI status, AI analysis, or web
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
