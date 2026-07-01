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
