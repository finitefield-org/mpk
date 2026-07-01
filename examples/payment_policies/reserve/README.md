# Reserve Policy Example

`ApprovedReserveCents` caps a reserve request to the available balance.

Helper artifacts:

- Go source in `policy.go`;
- contract sidecar in `policy_contract.json`;
- generated `gir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- representative `evidence_alpha.json`.

`evidence_alpha.json` records the first reserve nonnegativity obligation as
`mpk_verified` through a checked `mpk.linarith.v0` theory certificate under the
`mvp-strict` profile. The remaining reserve obligations are deliberately left as
`proof_pending` or `unsupported` until they have their own checked evidence.
