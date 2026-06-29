# Order Policy Usage Example

This example shows how MPK fits into a normal Go system. The application keeps
its handlers, storage calls, logging, and payment gateway code in ordinary Go.
Only the small deterministic business rule is lowered to GIR and turned into
verification obligations.

For the broader checklist and CI pattern, see
`../../docs/web-system-integration.md`.

## Verified Boundary

`policy.go` contains the pure function the system calls:

```go
approved := orderpolicy.ApprovedReserveCents(balanceCents, requestedCents)
```

`policy_contract.json` states the contract for that function:

- inputs are non-negative cent amounts;
- the approved reserve is non-negative;
- the approved reserve never exceeds the available balance;
- the approved reserve never exceeds the requested amount;
- the approved reserve is exactly either the balance or the request.

The current alpha pipeline generates obligations for those claims. It does not
verify the surrounding service code, database state, authentication, retries, or
network effects.

## Application Shape

In a real service, keep side effects outside the verified function. The
compilable example is in `webapp/handler.go`; its core flow is:

```go
func (s *OrderService) ReserveOrder(ctx context.Context, orderID string, requestedCents int64) error {
	balanceCents, err := s.wallet.AvailableCents(ctx, orderID)
	if err != nil {
		return err
	}

	approvedCents := orderpolicy.ApprovedReserveCents(balanceCents, requestedCents)
	if approvedCents != requestedCents {
		return ErrInsufficientBalance
	}

	return s.ledger.Reserve(ctx, orderID, approvedCents)
}
```

Only `ApprovedReserveCents` is in the MPK acceptance path. The handler is still
tested and reviewed using normal application practices.

Run the web adapter tests with:

```sh
(cd examples/order_policy/webapp && go test ./...)
```

The adapter enforces the policy preconditions at the HTTP boundary, calls
`ApprovedReserveCents`, and only then performs the ledger side effect.

## Rebuild GIR

From the repository root:

```sh
(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)
(cd examples/order_policy && ../../target/debug/go2gir . | jq '.gir' > gir.json)
```

## Rebuild VC Outputs

The checked-in `vc.json` and `vc_skeleton.json` are generated from `gir.json`
by the `mpk-vc` example regression test:

```sh
MPK_UPDATE_ORDER_POLICY_EXAMPLE=1 cargo test -p mpk-vc --test max64_example
cargo test -p mpk-vc --test max64_example
```

`vc.json` contains eight branch path obligations:

- four `then` obligations under the `requestedCents > balanceCents` path
  condition;
- four `else` obligations under the negated path condition.

`vc_skeleton.json` maps those obligations to core-shaped theorem declaration
skeletons under `VC.Obligation.*`. No proof body is present at this milestone.

## Trust Boundary

These files are candidate theorem-obligation artifacts only. The example is not
accepted as a proof until a later `.mpcert` checks in the Rust kernel and the Go
reference checker with hash and axiom-report recomputation.
