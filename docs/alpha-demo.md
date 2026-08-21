# Alpha Demo Guide

This guide reproduces the active Go/VIR path and the source-free proof checks.
Run commands from the repository root unless noted otherwise.

## Trust boundary

Go source, contracts, frontend envelopes, VIR, source maps, manifests, VC JSON,
skeletons, policy reports, and AI output are untrusted helper artifacts. Proof
acceptance comes only from canonical certificate bytes or checked theory
certificates accepted by the configured source-free checkers.

## 1. Run the Go source corpus

```sh
(cd fixtures/go-alpha && go test ./...)
(cd go-tools/go2vir && go test -count=1 ./...)
```

The frontend corpus covers all 100 alpha functions, the positive examples, and
the reviewed negative cases. Unsupported source features fail closed.

## 2. Check the installed Go/VIR artifacts

```sh
./scripts/regenerate-go-vir-corpus.sh --check
cargo test -p mpk-vc --test go_vir_corpus
```

The test performs two clean generations, imports VIR and source maps, rebuilds
VC v1 and grouped skeletons, and compares the result with `fixtures/vir-go`,
`fixtures/vc-alpha`, and the active example files. Use `--update` only for an
intentional fixture regeneration, then review every changed hash and byte.

The inspectable example outputs include:

- `examples/max64/{vir,source-map,source-manifest.frontend,vc,vc_skeleton}.json`;
- `examples/order_policy/{vir,source-map,source-manifest.frontend,vc,vc_skeleton}.json`;
- the same artifact set for each positive payment-policy example.

## 3. Run the registered policy path

Production `mpk policy scan` and `mpk policy verify` resolve a frontend and
toolchain from the release registry installed beside `bin/mpk`. They require
the full language/profile/registry/bundle/target/selection tuple and reject raw
executable, toolchain, and registry paths.

Use the complete reserve commands in
[`proof-ops-policy-ci.md`](proof-ops-policy-ci.md). A successful scan prints
`ok policy scan status=ready`; a successful verification prints
`ok policy verify status=complete`. The canonical policy evidence v1 report,
not the success line, records individual property statuses and trusted links.

## 4. Check canonical proof evidence

```sh
cargo run --quiet -p mpk-cli -- check fixtures/cert-basic/one-theorem.hex
cargo run --quiet -p mpk-cli -- axiom-report fixtures/cert-basic/one-theorem.hex
cargo run --quiet -p mpk-cli -- package verify-certs \
  fixtures/package-manifest/valid/basic-package.json
./scripts/checker-agreement.sh
```

The Rust checker and independent Go checker must agree on certificate hashes,
exports, axiom reports, and acceptance. Helper artifact success cannot replace
these checks.

## 5. Run the complete gate

```sh
./scripts/check-fast.sh
./scripts/check-all.sh
```

The complete gate includes the strict removed-interface scan, release-bundle
checks, Go frontend tests, policy/AI/API v1 owners, and the full Rust workspace.
