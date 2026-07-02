# Reserve Policy Example

`ApprovedReserveCents` caps a reserve request to the available balance.

Helper artifacts:

- Go source in `policy.go`;
- contract sidecar in `policy_contract.json`;
- generated `gir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- strict `evidence_alpha.json`.

`evidence_alpha.json` is the tracked strict evidence fixture for the reserve
example. It records all eight reserve obligations as `mpk_verified` under the
`mvp-strict` checker profile: six obligations are closed by checked
`mpk.linarith.v0` theory certificates, and two selected-branch equality
obligations are closed by checked `mpk.bool-normalize.v0` theory certificates.
Every property references exactly one checked theory certificate under
`trusted_evidence`.

Refresh this fixture only through `mpk policy verify --strict
--update-fixtures`, so the checked-in JSON stays aligned with the deterministic
CLI output documented in `../../../docs/payment-policy-alpha-coverage-design.md`.
