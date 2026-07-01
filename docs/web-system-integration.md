# Web System Integration Guide

This guide shows the production integration pattern for MPK alpha artifacts in a
Go web system. The short version is: keep web and storage effects in ordinary
Go, extract the deterministic decision into a small pure package, and make MPK
check the contract for that pure package.

The concrete example lives in `examples/order_policy`.

## Recommended Layout

```text
internal/orderpolicy/
  policy.go              # pure deterministic rules, no imports
  policy_contract.json   # MPK contract sidecar
  gir.json               # checked-in lowered GIR
  vc.json                # checked-in generated obligations
  vc_skeleton.json       # checked-in theorem declaration skeletons

internal/httpapi/
  handler.go             # HTTP parsing, auth, persistence, side effects
  handler_test.go        # normal web tests
```

The policy package should be boring Go: fixed-width integers, booleans,
structs, fixed arrays, local variables, branches, and returns. Avoid imports,
pointers, goroutines, reflection, maps, channels, package-level mutable state,
and I/O in the verified-boundary function.

## Request Flow

1. The HTTP handler authenticates the user and decodes JSON.
2. The handler validates preconditions that appear in `policy_contract.json`.
3. The handler fetches current state from storage or another service.
4. The handler calls the pure policy function.
5. The handler performs the side effect only if the policy result allows it.
6. Tests cover handler behavior, while MPK covers the pure policy contract.

In `examples/order_policy/webapp`, the handler follows that pattern:

```go
balanceCents, err := h.Wallet.AvailableCents(r.Context(), request.AccountID)
if err != nil {
	return
}

approvedCents := orderpolicy.ApprovedReserveCents(balanceCents, request.RequestedCents)
if approvedCents != request.RequestedCents {
	return
}

_ = h.Ledger.Reserve(r.Context(), request.AccountID, request.OrderID, approvedCents)
```

The ledger call is outside MPK. The contract tells you what is true about
`approvedCents` if the inputs satisfy the preconditions.

## Preconditions At The Boundary

Every `requires` clause must be enforced before calling the pure function, or
must be guaranteed by a trusted upstream invariant. For the order policy:

```json
{
  "op": "signed_ge",
  "lhs": { "var": "requestedCents" },
  "rhs": { "int": { "value": "0", "width": 64, "signed": true } }
}
```

The web handler rejects negative `requested_cents` before calling the policy.
It also treats a negative wallet balance as an upstream invariant violation.

## Build And Verification Commands

Run these from the repository root:

```sh
(cd examples/order_policy && go test ./...)
(cd examples/order_policy/webapp && go test ./...)
(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)
(cd examples/order_policy && ../../target/debug/go2gir . | jq '.gir' > gir.json)
cargo test -p mpk-vc --test max64_example
```

When intentionally regenerating checked-in VC artifacts:

```sh
MPK_UPDATE_ORDER_POLICY_EXAMPLE=1 cargo test -p mpk-vc --test max64_example
```

For the ProofOps product-facing path, run the current policy engine commands
against the pure package:

```sh
mkdir -p target/proof-ops

cargo run --quiet -p mpk-cli -- policy scan examples/order_policy \
  --function example.com/orderpolicy.ApprovedReserveCents \
  --contract examples/order_policy/policy_contract.json \
  --json-out target/proof-ops/order-policy.scan.json \
  --go2gir target/debug/go2gir

cargo run --quiet -p mpk-cli -- policy verify examples/order_policy \
  --function example.com/orderpolicy.ApprovedReserveCents \
  --contract examples/order_policy/policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json target/proof-ops/order-policy.evidence.json \
  --evidence-md target/proof-ops/order-policy.evidence.md \
  --go2gir target/debug/go2gir
```

Treat `mpk.policy.scan.v0` as helper analysis. In
`mpk.policy.evidence.v0`, only `trusted_evidence` can support an
`mpk_verified` claim. `helper_artifacts`, call-site precondition output, and
Markdown text can explain the integration but do not prove a property.

## CI Checklist

- Run normal Go tests for the web package.
- Run normal Go tests for the pure policy package.
- Run `mpk policy scan` and keep the scan JSON as helper-analysis review
  evidence.
- Run `mpk policy verify` and keep the evidence JSON as the product source of
  truth.
- Re-run `go2gir` and fail if `gir.json` changes unexpectedly.
- Run the VC fixture test and fail if `vc.json` or `vc_skeleton.json` changes
  unexpectedly.
- Keep generated GIR, VC, and skeleton files in review so contract drift is
  visible.
- Do not treat GIR, VC JSON, skeleton JSON, HTTP logs, or frontend traces as
  proof evidence.

## Review Checklist

For every web integration PR:

- The verified function has no side effects and stays inside the Go subset.
- Every handler call site checks the policy preconditions before calling the
  verified function.
- Money, counts, and IDs use explicit fixed-width types at the boundary.
- Handler tests cover success, rejection, malformed input, and upstream failure.
- The contract sidecar changed when the business rule changed.
- Regenerated artifacts are either intentionally committed or the PR explains
  why they did not change.

## Current Alpha Limit

The current alpha pipeline generates candidate theorem obligations for the pure
Go policy. Those artifacts are useful for development review and regression
testing, but they are not proof evidence by themselves. Acceptance still comes
from canonical `.mpcert` artifacts checked source-free by the Rust kernel and,
when configured, the Go reference checker.
