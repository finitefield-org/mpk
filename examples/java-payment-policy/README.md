# Java payment-policy example

This dependency-free source set exercises the active Java 25 scalar profile
through the installed `payment-policy-java-alpha` successor route. It chooses
one of two reserve amounts from an explicit approval flag. The example keeps
only reviewed inputs; generated VIR, VC, evidence, and certificate files are
not retained as a second fixture family.

The revision-3 semantic context and method selection are frozen in
`mpk-semantic-context.json` and `mpk-selection.json`. On the installed x86-64
Linux release, produce a scan with:

```sh
mpk policy scan examples/java-payment-policy \
  --semantic-context examples/java-payment-policy/mpk-semantic-context.json \
  --selection examples/java-payment-policy/mpk-selection.json \
  --json-out java-policy-scan.json
```

Run strict verification by replacing `scan` with `verify` and
`--json-out java-policy-scan.json` with
`--evidence-json java-policy-evidence.json`. Java contract paths are closed by
the validated selection envelope, so Java requests do not accept `--contract`.
Bundle identities, the registry digest, the JDK root, and executable paths are
resolved only from the installed release and cannot be selected by the caller.
