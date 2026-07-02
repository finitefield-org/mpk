# Reserve Policy Example

`ApprovedReserveCents` caps a reserve request to the available balance.

Helper artifacts:

- Go source in `policy.go`;
- contract sidecar in `policy_contract.json`;
- generated `gir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- representative `evidence_alpha.json`.

`evidence_alpha.json` is a representative schema fixture. It records the first
reserve nonnegativity obligation as `mpk_verified` through a checked
`mpk.linarith.v0` theory certificate under the `mvp-strict` profile. The
remaining reserve obligations are deliberately untrusted until they have their
own checked evidence.

The live CLI baseline for future coverage work is documented in
`../../../docs/payment-policy-alpha-coverage-design.md`; when this fixture is
refreshed, align its status counts with the deterministic `mpk policy verify`
output described there.
