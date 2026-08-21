# Max64 End-to-End Example

This example demonstrates the current untrusted pipeline boundary:

1. `go2vir` lowers `max64.go` and `max64_contract.json` to VIR.
2. `mpk-vc` generates branch verification conditions from the VIR.
3. `mpk-vc` emits theorem-obligation skeletons for later certificate emission.

The generated artifacts are checked into this directory so the end-to-end shape
is inspectable without treating any untrusted artifact as proof evidence.

## Source

- `max64.go`: Go subset v0 source.
- `max64_contract.json`: `mpk.go.contract.v0` sidecar for `example.Max64`.

`Max64` returns a selected `int64` maximum. The contract states that the result
is greater than or equal to both inputs and equals one of them.

## Rebuild VIR

From the repository root:

```sh
(cd go-tools/go2vir && go build -o ../../target/debug/go2vir .)
(cd examples/max64 && ../../target/debug/go2vir . | jq '.vir' > vir.json)
```

## Rebuild VC Outputs

The checked-in frontend, manifest, VC, and skeleton artifacts are owned by the
shared Go/VIR corpus:

```sh
./scripts/regenerate-go-vir-corpus.sh --update
./scripts/regenerate-go-vir-corpus.sh --check
```

`vc.json` contains six branch path obligations:

- three `then` obligations under the `b > a` path condition;
- three `else` obligations under the negated path condition.

`vc_skeleton.json` maps those obligations to core-shaped theorem declaration
skeletons under `VC.Obligation.*`. No proof body is present at this milestone.

## Trust Boundary

These files are candidate theorem-obligation artifacts only. The example is not
accepted as a proof until a later `.mpcert` checks in the Rust kernel and the Go
reference checker with hash and axiom-report recomputation.
