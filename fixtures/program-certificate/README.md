# Program-certificate fixtures

`alpha-module-calls.hex` is the canonical self-contained Certificate v0
assembled from the Rust `module-calls` VC and grouped skeleton under
`mpk.program_certificate.alpha.v0`. It contains the selected zero-axiom
foundation closure plus all six generated group theorems, with empty import,
proof-node, and theory-certificate tables. The embedded source manifest is the
separately pinned `alpha-module-calls.source-manifest.certificate.json`, after
the policy pipeline has attached the exact VC hash.

The Rust assembler test pins these exact bytes and checks the complete report
from both source-free checkers. The checker-agreement corpus independently
submits the fixture to both checker implementations. `hashes.csv` records the
recomputed report hashes.

The Rust policy end-to-end test owns all three generated fixtures and normally
asserts that they are current. Regenerate them explicitly with:

```text
MPK_UPDATE_PROGRAM_CERTIFICATE_FIXTURE=1 cargo test -p mpk-cli --test rust_policy_verify rust_module_calls_emits_one_dual_checked_zero_axiom_program_certificate -- --exact
```
