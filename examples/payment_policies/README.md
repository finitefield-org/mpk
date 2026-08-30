# Payment Policy Corpus

This corpus contains small pure Go payment-policy functions used by the
ProofOps workflow. Each positive example is intentionally limited to the
current Go subset v0 and has:

- `policy.go`: the pure function;
- `policy_contract.json`: helper contract input;
- `vir.json`: generated helper VIR;
- `vc.json`: generated helper verification conditions;
- `vc_skeleton.json`: generated helper theorem-obligation skeletons.

The successor corpus also stores `mpk-semantic-context.json`,
`mpk-selection.json`, `frontend-envelope.json`, `source-map.json`, and the
frontend source manifest. These files are helper artifacts only. Trusted proof
evidence starts only when
canonical `.mpcert` bytes or checked theory certificates are accepted by MPK
under the active checker profile.

The negative examples under `negative/` exercise deterministic rejection for
unsupported floats, maps, pointers, and missing contract postconditions.

## CI Usage

The CI command pattern for this corpus is documented in
`../../docs/proof-ops-policy-ci.md`. Run the reserve example from the repository
root as the current strict end-to-end path for checked theory evidence:

```sh
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

Expected CLI output identifies `mpk.policy.evidence.v2`; the generated
evidence contains the reviewed property counts and statuses. Registry, bundle,
toolchain, strategy, checker, and axiom identities come only from the installed
successor release and its compiled profile contracts.

Review these helper artifacts in PRs when a policy changes:

- `policy.go`;
- `policy_contract.json`;
- generated `vir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- generated scan JSON from `mpk policy scan`;
- generated evidence JSON from `mpk policy verify`.

Treat VIR, VC JSON, scan JSON, explanation requests, and CI status as helper
artifacts only. They are useful drift signals, but they are not proof evidence.
Trusted proof evidence is limited to checked certificates, checked theory
certificates, checker verdicts, and the corresponding axiom reports recorded in
`mpk.policy.evidence.v2`.
