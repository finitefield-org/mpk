# Alpha Demo Guide

This guide reproduces the current alpha path from Go source to source-free
certificate verification. Run every command from the repository root unless the
command changes directory explicitly.

The demo intentionally separates candidate artifacts from trusted evidence:

- Go source, contracts, GIR, and VC skeletons are helper artifacts.
- A strategy success is trusted only when it attaches a checked theory
  certificate under the active checker profile.
- A theorem or package is accepted only from canonical certificate bytes checked
  source-free by the Rust kernel and, when required, the independent Go
  reference checker.

## Prerequisites

Install the normal project toolchain:

- Rust with `cargo`.
- Go.
- Python 3 for the small JSON checks below.

No network access or regenerated checked-in artifacts are required.

## 1. Compile the Go alpha corpus

The ALPHA-001 corpus contains 100 pure Go subset functions. First verify that
the source corpus itself still compiles:

```sh
(cd fixtures/go-alpha && go test ./...)
```

Expected result: the `arith`, `array`, and `branch` packages all pass.

## 2. Lower the Go corpus through go2gir

`go2gir` tests execute the alpha manifest, check the recorded function count,
and require every manifest entry to lower to GIR:

```sh
(cd go-tools/go2gir && go test -count=1 ./...)
```

Expected result: the `go-tools/go2gir` package passes.

## 3. Rebuild the Max64 GIR from Go source

Build the local `go2gir` binary and lower the Max64 example:

```sh
(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)
(cd examples/max64 && ../../target/debug/go2gir . > /tmp/mpk-max64-go2gir.json)
```

Confirm that the generated output contains one lowered GIR function and matches
the checked-in Max64 GIR fixture:

```sh
python3 - <<'PY'
import json
from pathlib import Path

data = json.loads(Path("/tmp/mpk-max64-go2gir.json").read_text())
checked_gir = json.loads(Path("examples/max64/gir.json").read_text())
gir = data["gir"]
function_count = sum(len(pkg["functions"]) for pkg in gir["packages"])

print(f"status={data['status']}")
print(f"gir_hash={gir['gir_hash']}")
print(f"matches_checked_in_gir={gir == checked_gir}")
print(f"function_count={function_count}")

assert data["status"] == "gir-lowered"
assert gir == checked_gir
assert function_count == 1
PY
```

Expected result: the script prints `status=gir-lowered` and
`matches_checked_in_gir=True`.

The repository also includes an application-shaped Go example in
`examples/order_policy`. It models the usual production shape: handlers,
storage, and external effects stay in ordinary Go, while a pure order-reserve
policy function is lowered and given a contract sidecar:

```sh
(cd examples/order_policy && ../../target/debug/go2gir . > /tmp/mpk-order-policy-go2gir.json)
```

Expected result: the generated GIR contains
`example.com/orderpolicy.ApprovedReserveCents` and matches
`examples/order_policy/gir.json`.

## 4. Reproduce VC outputs

The example regression test imports the checked-in `gir.json` files for Max64
and the order-policy example, regenerates VC and skeleton output from them, and
verifies that the generated output still matches each checked-in `vc.json` and
`vc_skeleton.json`:

```sh
cargo test -p mpk-vc --test max64_example
```

The alpha corpus regression test verifies the ALPHA-002 branch VC corpus,
including the 1,056 recorded obligations:

```sh
cargo test -p mpk-vc --test alpha_corpus
```

Expected result: both tests pass.

Only run the update commands below when intentionally regenerating checked-in
fixtures for a separate change:

```sh
MPK_UPDATE_MAX64_EXAMPLE=1 cargo test -p mpk-vc --test max64_example
MPK_UPDATE_ORDER_POLICY_EXAMPLE=1 cargo test -p mpk-vc --test max64_example
MPK_UPDATE_VC_ALPHA=1 cargo test -p mpk-vc --test alpha_corpus
```

## 5. Run the ProofOps policy engine path

The payment-policy corpus exercises the ProofOps product-facing commands. The
reserve example is the strict supported policy verify path: `mpk policy scan`
checks readiness as helper analysis, and `mpk policy verify --strict` writes
deterministic policy evidence JSON and Markdown while requiring every supported
property to have checked theory-certificate evidence.

```sh
mkdir -p target/proof-ops

cargo run --quiet -p mpk-cli -- policy scan examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --json-out target/proof-ops/reserve.scan.json \
  --go2gir target/debug/go2gir

cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve \
  --function example.com/payment/reserve.ApprovedReserveCents \
  --contract examples/payment_policies/reserve/policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json target/proof-ops/reserve.evidence.json \
  --evidence-md target/proof-ops/reserve.evidence.md \
  --go2gir target/debug/go2gir \
  --strict
```

Expected result: `policy scan` reports `status=ready`, and `policy verify`
reports `status=verified` with `verified=8 proof_pending=0 unsupported=0`. The
evidence JSON keeps `helper_artifacts` such as source hashes, GIR, VC, and
Markdown report context separate from `trusted_evidence` such as checked
theory-certificate entries. It also keeps `strategy_profile`,
`checker_profile`, and `allowed_axiom_profiles` as separate policy fields: the
strategy profile selects the payment workflow, the checker profile selects the
MPK proof checker mode, and the axiom policy profile records the allowed axiom
class.

## 6. Check the Max64-shaped theory proof hook

TH-008 adds a checked theory-certificate path for strategy proofs. This command
exercises the Max64 simple VC fixture through the API strategy dispatcher and
requires the proof node to reference a checked `linarith` theory certificate:

```sh
cargo test -p mpk-api --test strategies \
  theory_strategy_proves_max64_simple_vc_through_checked_certificate
```

Expected result: the targeted test passes.

## 7. Verify source-free certificate artifacts

Finally, verify canonical certificate bytes directly:

```sh
cargo run --quiet -p mpk-cli -- check fixtures/cert-basic/one-theorem.hex
```

Expected result: the JSON verdict contains `"verdict":"accepted"` for
`Example.Basic.OneTheorem`.

Then verify a package manifest that requires both source-free checking and the
independent Go reference checker:

```sh
cargo run --quiet -p mpk-cli -- package verify-certs \
  fixtures/package-manifest/valid/basic-package.json
```

Expected result:

```text
ok package=Example.Basic.Package source_free=1 reference=1
```

## What this demo proves

When all commands above pass locally, the alpha pipeline demonstrates:

- the 100-function Go alpha corpus compiles and lowers to GIR;
- the Max64 example lowers from Go source and reproduces its documented VC
  artifacts;
- the order-policy example documents how a real Go service calls a pure
  verified-boundary policy function while keeping side effects outside MPK;
- the ALPHA-002 VC corpus still records 1,056 generated obligations;
- the reserve payment policy scans as MPK-ready and verifies strictly with
  eight checked theory-backed `mpk_verified` properties while keeping helper
  artifacts separated from trusted checked evidence;
- the TH-008 strategy hook can close a Max64-shaped simple VC only through a
  checked theory certificate;
- canonical certificate bytes are accepted by source-free verification;
- package verification can require agreement from the independent Go reference
  checker.

## What this demo does not claim

The checked-in GIR, VC, and skeleton JSON files are not proof evidence. They are
candidate theorem-obligation artifacts. Policy scan JSON, policy verify
Markdown, helper-artifact hashes, CI status, and generated policy evidence prose
are also not proof evidence. Until a later milestone emits checked `.mpcert`
certificates for the generated alpha VCs, acceptance is demonstrated by the
certificate fixtures and package verification commands in step 7, not by the Go
source, contract sidecars, GIR, VC JSON, skeleton JSON, policy scan output,
Markdown reports, or strategy logs.
