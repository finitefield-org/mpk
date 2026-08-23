# Policy profile fixtures

`rust-release-policy.json` is the active Rust release selection. The product
gate pairs it with
`examples/rust-payment-policy/artifacts/package-manifest.json`; the generic
`fixtures/package-manifest/valid/rust-policy-package.json` remains the focused
package-policy validation fixture. Both manifests permit exactly the checker
and axiom profiles selected by the active release row.
