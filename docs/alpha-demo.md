# Alpha Demo Guide

This guide reproduces the active Go/Rust/C# successor paths and source-free
proof checks.
Run commands from the repository root unless noted otherwise.

## Trust boundary

Go/Rust/C# source, contracts, compilers, frontend envelopes, VIR, source maps,
manifests, VC JSON, skeletons, policy reports, CI results, and AI output are
untrusted helper artifacts. Proof
acceptance comes only from canonical certificate bytes or checked theory
certificates accepted by the configured source-free checkers.

## 1. Run the frontend source corpora

```sh
(cd fixtures/go-alpha && go test ./...)
(cd go-tools/go2vir && go test -count=1 ./...)
cargo test -p mpk-cli --test csharp_profile_vectors
./scripts/build-csharp-frontend.sh --check
```

The corpora cover the registered profiles, positive examples, and reviewed
negative cases. Unsupported source features and crossed profile identities
fail closed.

## 2. Check the installed Go/VIR artifacts

```sh
./scripts/regenerate-go-vir-corpus.sh --check
cargo test -p mpk-vc --test go_vir_corpus
```

The test imports successor VIR and source maps, rebuilds successor VC and
grouped skeletons, and compares the result with `fixtures/vir-go`,
`fixtures/vc-alpha`, and the active example files. Use `--update` only for an
intentional fixture regeneration, then review every changed hash and byte.

The inspectable example outputs include:

- `examples/max64/{vir,source-map,source-manifest.frontend,vc,vc_skeleton}.json`;
- `examples/order_policy/{vir,source-map,source-manifest.frontend,vc,vc_skeleton}.json`;
- the same artifact set for each positive payment-policy example.

## 3. Run the registered policy path

Production `mpk policy scan` and `mpk policy verify` resolve a frontend,
toolchain, registry identity, and compiled profile contracts from the release
installed beside `bin/mpk`. Callers provide only revision-2 semantic-context
and selection envelopes, Go/Rust contract paths, and an output path. Raw
executables, toolchain roots, registries, bundle identities, and compatibility
flags reject.

Use the complete reserve commands in
[`proof-ops-policy-ci.md`](proof-ops-policy-ci.md). A successful scan prints
`ok policy scan schema=mpk.policy.scan.v2`; a successful verification prints
`ok policy verify schema=mpk.policy.evidence.v2`. The canonical evidence
report, not the success line, records individual property statuses and trusted
links.

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
sudo ./scripts/check-csharp-frontend.sh
sudo ./scripts/check-all.sh
```

The successor gate validates frozen build inputs and all registered bundles,
executes Go, Rust, and C# through one installed image, checks determinism,
differential cases, limits, fuzz smoke, path sanitation, checker agreement,
axiom/profile equality, and removed predecessor interfaces. Its release report
is untrusted provenance; it cannot replace the certificate checks in section
4.
The root boundary is used only to create the release sandbox's fresh delegated
cgroup and fixed `noswap` backing; Rust source processing runs after the
sandbox removes host privileges.

The Rust example and its exact request envelopes live under
`examples/rust-payment-policy`. Verify generated release metadata with:

```sh
python3 scripts/generate-release-report.py --check
```
