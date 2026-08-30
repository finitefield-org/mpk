# Rust payment-policy example

This dependency-free library is the Rust example for the active
`payment-policy-rust-alpha` successor route. The checked-in source and
contracts are inputs; generated VIR, VC, evidence, and certificate files are
deliberately not retained as a second predecessor fixture family.

The semantic context and function selection are frozen in
`mpk-semantic-context.json` and `mpk-selection.json`. On the installed Linux
release, produce a scan with:

```sh
mpk policy scan examples/rust-payment-policy \
  --semantic-context examples/rust-payment-policy/mpk-semantic-context.json \
  --selection examples/rust-payment-policy/mpk-selection.json \
  --contract contracts/helper.json \
  --contract contracts/selected.json \
  --json-out rust-policy-scan.json
```

Run strict verification by replacing `scan` with `verify` and
`--json-out rust-policy-scan.json` with
`--evidence-json rust-policy-evidence.json`. Bundle identities, registry
digests, toolchain roots, and executable paths are resolved only from the
installed successor release and cannot be selected by the caller.

`contracts/insufficient-precondition.json` is a negative proof-planning input.
Use it instead of `contracts/selected.json` to reproduce the proof-pending
case; it is not a compatibility or fallback route.
