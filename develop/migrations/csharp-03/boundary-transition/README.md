# CSHARP-03-T05-W06: boundary/transition closure

This gate closes frontend attachment and emission for T05. Proof discharge is
still T06-owned; installed native release activation remains T07/T08-owned.

The production fix applies total-termination propagation to both captured data
and captured control routes. Previously `attach_data_contracts` only propagated
partial callees when a control handoff existed. A straight-line boundary root
with a total contract could therefore call a captured partial method through
the data route. The shared `derive_control_termination` walk now runs for every
route, including constructor and property-getter edges. Control-specific getter
claims, exception universes and the existing loop-contract preparation retain
their original behavior. Both the emitter and independent importer use this
same attachment barrier.

## Unified gate

`crates/mpk-cli/tests/csharp_practical_boundary_transition.rs` composes the
independently runnable W01–W05 boundary and transition suites and adds two W06
tests. The complete composed suite is executed twice. This retains the original
actual-source fixtures, CLR observations, full-snapshot idempotency coverage,
byte parsers, output reparse checks, bounds and previous attachment mutations.
The local test-only duplicate-module lint allowance permits this composition;
production source and validator implementations are not duplicated.

The W06 control cases attach a boundary and transition to the same actual
application-owned Apply. Method and transition expressions each introduce a
codec result root that is absent from the application source. Their distinct
contract-hash provenance rows survive in the cumulative root table, while the
shared T02-W02 engine deduplicates concrete instances. The test recomputes the
closed table and counters, independently imports the VIR, and rejects removal
of the added roots, bindings, source obligations or either
contract. A residual generic-shaped function identity cannot enter imported VIR.
No second closure, identity, deduplication or limit algorithm is introduced.

The combined input/output run retains the original source map and closed table,
both boundary and transition contract references, input provenance and output
capture. The returned typed fixture value corresponds to the W04 deposit example;
its output receipt is reproduction evidence, not proof that application code ran.
Actual finite CLR execution remains covered by the included W04/W05 suites.
The composed example has 33 roots and nine source types. The shared closed table
reports nine declarations, 54 operations and 554 recipe nodes.

## Rejection and fuzz evidence

`control-requests.json` and `control-responses.json` retain ten actual source
requests and their pinned Linux/amd64 capture rows. Five source cases reject
before artifacts: MPK namespace dependency, external time, generic serializer
call, async modifier and iterator constructed type. Exact owning diagnostics
are asserted. The other rejected cases have valid captured source but malformed
binding/boundary attachment or a partial root, including an actual partial loop
with a valid partial loop contract. They cannot return an EmittedDataPhase.

`data-requests.json` and `data-responses.json` retain four actual straight-line
source captures with no control handoff. Their root calls a method that invokes
a constructor and a getter. Total succeeds; marking each of those three
reachable callees partial rejects at contract attachment. Tests first assert
that each sidecar callable really exists, so a misspelled helper ID cannot
masquerade as a successful totality regression.

The deterministic bounded mutation schedule is:

| Target | Seeds | Mutation |
| --- | ---: | --- |
| Source protocol | 16 | Replace compilation identity with `mutated.N` |
| Canonical boundary input | 32 | Set byte `(N*3) % length` to invalid UTF-8 `0xff` |
| Canonical boundary output | 32 | Flip bit `0x80` at `(N*5) % length` |
| Original boundary sidecar | 32 | Set byte `(N*17) % length` to `0xff` |

All 112 seeds reject at their original-capture, byte-parser or strict-sidecar
barrier; none returns a publishable run/artifact. Further targeted import
mutations remove codec roots, contracts, projections or
source obligations. `evidence.json` retains the actual source map, cumulative
manifest, input/output receipts and engine counters; both full suite runs must
produce byte-identical evidence. Each accepted compiler input is also captured
twice by the pinned harness with a byte-determinism check.

## Local verification and explicit gate exception

`verification.json` records the two complete boundary/transition suite runs,
affected inventory and data-contract consumer checks, scoped lint/format checks,
and `./scripts/build-csharp-frontend.sh --check` in the local Linux image.
The native installed-release gate is not this task's gate.

For this W06 only, the user explicitly requested that the overall check be
skipped because it had run at T05-W04. Accordingly `./scripts/check-fast.sh`
is not run, nor replaced with a collection of workspace-wide commands. The
historical W04 result is not represented as validation of W05/W06 changes.
This exception does not alter the repository's normal T-completion cadence.
T05 completes under that explicit instruction; T06-W01 becomes ready.
