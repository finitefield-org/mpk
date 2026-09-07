# CSHARP-03-T04-W06 control emission evidence

The private `mpk.csharp_practical.t04_w06.control_lowering.v1` route combines
captured data facts with validated loop, pattern and exception control. Native
emission produces ordinary typed VIR operations, SSA phis, explicit branches,
closed exception values and construction cleanup. The ordinary importer checks
dominance, types, regions, handler order, call closure and ownership. It also
reconstructs source correspondence without invoking Roslyn. That reconstruction
shares the source emitter; it is not a second independent compiler. Runtime
comparisons use the separate ordinary-VIR observer in test support.

The data-plus-control root set, including pending return carriers and contract
expression roots, uses the existing T02 specialization engine. The importer
recomputes the closed instances and operation linkage. Source artifacts retain
the frozen declaration/member source-map vocabulary; control anchors additionally
bind each emitted evaluation to its original UTF-8 operation span.

## Retained inputs

- `source-cases.json`: all 165 T04 source cases, with 111 accepted captures and
  54 artifact-free rejections. The pinned frontend captures each accepted
  selection twice and requires identical bytes.
- `loop-responses.json`: 27 loop selections with actual sidecars; 23 executable
  `Run` roots have 533 original CLR runs. Four map/set wrapper roots have no
  `Run` entry; their actual algorithms are covered separately.
- `cross-loop-responses.json`: four pattern/exception/handler loop combinations.
- `collection-sources.json`, `collection-capture.json`,
  `collection-responses.json`: six deferred real collection algorithms and
  159 CLR runs, including the two-pass array and ordered map/set matrices.
- `construction-sources.json`, `construction-capture.json`,
  `construction-responses.json`: 22 construction/publication selections,
  comprising 16 accepted cases with 96 CLR runs and six artifact-free rejections.
  Source-only captures supply the exact
  method identities used to generate sidecars; responses retain the actual
  subsequent capture with those sidecars and original CLR execution.
- `fuzz-seeds.json`: 37 deterministic, retained CFG mutations across source
  ordinals, value definitions, normal targets and exception targets.
- `control-emission-inputs.json`: exact current compiler harness/build inputs.
  Older stage conformance receipts remain historical; their live `*-inputs.json`
  manifests track the updated common producer files.

Ordinary-VIR comparisons also cover all 29 W05 handler cases and their original
CLR traces, including cross-call filter-before-finally search, filter throws,
rethrow payload identity and replacement exceptions from finally. The observer
suspends an escaping callee before unwind while its caller searches filters.

A unique, unfinished array/object/sequence state is discarded before entering
catch/finally or escaping the function. A published immutable alias may be read
there. A constructor's unfinished receiver is live throughout the
constructor, so source try/catch/finally in that constructor rejects with
`CSHARP_PRACTICAL_OBJECT/unfinished_receiver_handler` for classes and structs.
Looping constructors retain their receiver/fields in SSA until normal return.
Proof discharge, installed frontend activation and public profile activation
remain with their existing later owners.

## Reproduction

Run `cargo test -p mpk-cli --test csharp_practical_control` and
`./scripts/check-fast.sh` locally. Native tests need no Roslyn process: they
import retained real-source facts, regenerate complete artifacts, compare
ordinary-VIR execution with retained CLR outcomes, and reject malformed inputs.

The pinned local Linux frontend actions are:

```sh
./scripts/build-csharp-practical-frontend.sh --test-control-emission
./scripts/build-csharp-practical-frontend.sh --test-control-emission-requests < requests.json
./scripts/build-csharp-practical-frontend.sh --test-data-phase-replay
python3 -B scripts/csharp_practical_build_inputs.py check-build-inputs
```

The request action accepts at most 32 MiB of JSON. Generate request files by
setting `MPK_CSHARP_CONTROL_REQUESTS_OUT`,
`MPK_CSHARP_CROSS_CONTROL_REQUESTS_OUT`,
`MPK_CSHARP_COLLECTION_REQUESTS_OUT`, or
`MPK_CSHARP_CONSTRUCTION_REQUESTS_OUT` while running the matching W06 test.
Each variable names an output file. Run the generated requests using the pinned
local Linux toolchain; retain the returned bytes, not modified handoffs.
`verification.json` records the commands, results and reviewed file hashes.
