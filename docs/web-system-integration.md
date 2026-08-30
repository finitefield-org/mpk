# Web System Integration Guide

MPK verifies a small deterministic policy function; it does not verify an
entire web service. Keep authentication, request parsing, storage, retries,
logging, and network effects in ordinary application code, and isolate the
decision rule in a pure package with a reviewed contract.

The runnable example is `examples/order_policy`:

```text
examples/order_policy/
  policy.go
  policy_contract.json
  mpk-semantic-context.json
  mpk-selection.json
  frontend-envelope.json
  vir.json
  source-map.json
  source-manifest.frontend.json
  vc.json
  vc_skeleton.json
  webapp/
```

All JSON files in this list are helper artifacts. Canonical certificates and
checked theory certificates become proof evidence only after source-free
checker acceptance.

## Application boundary

The web handler validates request-level preconditions, reads its upstream
state, calls `ApprovedReserveCents`, and performs a side effect only after the
pure function returns the requested amount. MPK does not infer the database,
authentication, or concurrency invariants around that call.

Run the ordinary application tests independently:

```sh
(cd examples/order_policy && go test ./...)
(cd examples/order_policy/webapp && go test ./...)
```

## Artifact drift

The active corpus owner regenerates the frontend and VC artifacts through the
same canonical pipeline used by all Go examples:

```sh
./scripts/regenerate-go-vir-corpus.sh --check
cargo test -p mpk-vc --test go_vir_corpus
```

Use `./scripts/regenerate-go-vir-corpus.sh --update` only for intentional
changes and review the resulting VIR, source map, manifest, VC, skeleton, and
hash differences together.

## Policy and local verification

Run `mpk policy scan` and `mpk policy verify` from an installed Linux release.
Both commands validate revision-2 semantic-context and selection envelopes and
select the registered Go frontend/toolchain tuple. No raw binary, registry,
bundle, toolchain, or compatibility path is accepted. The complete argument
contract and reusable local verification block are in
[`proof-ops-policy-ci.md`](proof-ops-policy-ci.md).

Store scan/evidence outputs as review artifacts. Only
`trusted_evidence` entries backed by accepted declarations or theory
certificates can support an `mpk_verified` property. VIR, VC JSON, skeletons,
source locations, diagnostics, AI prose, and HTTP logs remain helper data.

## Review checklist

- Run pure-package and web-handler Go tests.
- Run the Go/VIR corpus and release-bundle drift gates.
- Review source, contract, VIR, source-map, manifest, VC, and skeleton changes.
- Keep policy v2 property statuses separate from helper-artifact readiness.
- Run source-free certificate checkers for every proof claim.
