# mpk

Start with [docs/alpha-demo.md](docs/alpha-demo.md) for the current alpha
verification path and [docs/proof-ops-engine-design.md](docs/proof-ops-engine-design.md)
for the ProofOps engine contract consumed by `../proof-ops`. See
[docs/web-system-integration.md](docs/web-system-integration.md) for the
web-system integration pattern and [docs/proof-ops-policy-ci.md](docs/proof-ops-policy-ci.md)
for the CI handoff shape.

Examples:

- `examples/max64`: compact branch VC example.
- `examples/order_policy`: application-shaped Go usage example showing how a
  service can call a pure policy function that MPK lowers and turns into VC
  obligations.
