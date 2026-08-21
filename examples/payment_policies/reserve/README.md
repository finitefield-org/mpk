# Reserve Policy Example

`ApprovedReserveCents` caps a reserve request to the available balance.

Helper artifacts:

- Go source in `policy.go`;
- contract sidecar in `policy_contract.json`;
- generated `vir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`.

Policy evidence v1 is generated into caller-selected output paths and is not
checked into this source directory. Follow
`../../../docs/proof-ops-policy-ci.md` for the complete registered-bundle
invocation.
