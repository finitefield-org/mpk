# Payment Policy Corpus

This corpus contains small pure Go payment-policy functions used by the
ProofOps workflow. Each positive example is intentionally limited to the
current Go subset v0 and has:

- `policy.go`: the pure function;
- `policy_contract.json`: helper contract input;
- `gir.json`: generated helper GIR;
- `vc.json`: generated helper verification conditions;
- `vc_skeleton.json`: generated helper theorem-obligation skeletons.

These files are helper artifacts only. Trusted proof evidence starts only when
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

(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)

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

Expected `policy verify` result for the supported positive corpus is
`status=verified verified=8 proof_pending=0 unsupported=0`. The repository test
`policy_verify_positive_payment_corpus_has_expected_counts` checks the same
strict result for `reserve`, `refund`, `discount`, `fee`, and `points`.

Review these helper artifacts in PRs when a policy changes:

- `policy.go`;
- `policy_contract.json`;
- generated `gir.json`;
- generated `vc.json`;
- generated `vc_skeleton.json`;
- generated scan JSON from `mpk policy scan`;
- generated evidence JSON and Markdown from `mpk policy verify`.

Treat GIR, VC JSON, scan JSON, Markdown reports, and CI status as helper
artifacts only. They are useful drift signals, but they are not proof evidence.
Trusted proof evidence is limited to checked certificates, checked theory
certificates, checker verdicts, and the corresponding axiom reports recorded in
`mpk.policy.evidence.v0`.
