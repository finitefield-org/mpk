# Rust payment-policy example

This dependency-free library is the Rust product-gate example for the
`payment-policy-rust-alpha` strategy. `approved_reserve_cents` returns the
payment approval bit unchanged. Its helper requires the reserve-request bit,
so the positive contract supplies that precondition and stays entirely within
the structural program-certificate subset.

The sibling contract in `contracts/insufficient-precondition.json` omits the
caller precondition without changing the accepted Rust source. It therefore
has no certificate candidate: non-strict verification records proof-pending
evidence, while strict verification records the same evidence and returns
`POLICY_PROOF_PENDING`.

## Trust boundary

The Rust source, contracts, registered frontend and toolchain identities,
frontend envelope, VIR, source map, manifests, VC, grouped skeleton, policy
scan, evidence JSON, and Markdown provide deterministic traceability. They are
not proof evidence and do not become trusted because they are checked in.

The trusted result is the exact `artifacts/program.mpcert` byte sequence after
both source-free checkers accept it. Its checked declarations bind the
certificate-stage source manifest and the structural member proofs. The
recomputed `artifacts/axiom-report.json` records zero axioms; the certificate's
import, proof-node, and theory-certificate tables are empty. The package and
release gates independently bind those bytes and reports to the
`mvp-strict` checker profile and `mvp-theory` axiom allowlist.

## Reproduction

Run the library test with:

```sh
cargo test --locked
```

From the repository root, regenerate every frontend, certificate, evidence,
package, and release-report fixture explicitly with:

```sh
scripts/regenerate-rust-payment-policy.sh --update
```

Check all frozen bytes without updating them with:

```sh
scripts/regenerate-rust-payment-policy.sh --check
```

The canonical scan and evidence reproduction command arrays are stored in
`artifacts/evidence.json`; they use only repository-relative inputs and
registered bundle identities. A normal test run generates twice in clean
temporary directories, requires byte equality, and rejects local-path leaks.
Fixture writes occur only when `MPK_UPDATE_RUST_PAYMENT_POLICY=1` is present.
