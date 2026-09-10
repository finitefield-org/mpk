# W09 actual-source structural/boundary integration review (in progress)

The shared-fold fixture established definition compatibility but did not assemble
actual source-specific structural and boundary definitions together. The new
boundary generation mode uses the same owning structural/Money Builder and
appends one shared JSON environment after every existing operation and source
observation. It regenerates the BoundaryVcProgram from the same validated VIR,
binds its exact hash and emits JSON only when that program has boundary contracts.
All counters remain cumulative and no operation/deferred obligation is dropped.

Standalone generation continues through the same implementation with boundary
mode disabled. The two new Option fields are omitted from serialization when
absent, preserving the old schema, metadata and certificate. Boundary mode has a
distinct schema and retains the boundary-program hash even when contracts are
empty. In that case certificate bytes remain identical to the standalone program,
but cross-mode metadata imports still reject. Import reconstructs the complete
expected program and compares both canonical metadata and certificate bytes.

Actual-source tests cover three captured boundary request/response contexts and
the existing unit/void context without a boundary contract. Every old structural
metadata group must agree except the expected schema/count/certificate-hash fields.
Every JSON definition/shape must agree after removing cumulative count snapshots.
All declarations and their complete transitive dependencies are compared against
both standalone components, including helper types, bodies, binders and imports.
The new JSON count must equal the final program counter. Mutations cover schema,
source/foundation/boundary hashes, JSON definitions, counts, deferred obligations,
certificate bytes, cross-source and cross-mode substitution.

All four new source cases passed generation/closure/import/mutations and limits.
The three boundary cases contain respectively 96,775/65,358/53,868 terms,
1,369/1,222/767 declarations, and 13,252/12,939/4,650 transformers. Their component
closure comparisons cover 1,405/1,250/779 declarations. The no-boundary source
retains 6,647 terms, 200 declarations and 58 transformers, with exact byte equality
to the previously dual-checked unit-void-source structural pin.

The three new certificate byte vectors are distinct and all require checker
execution. The unchanged no-boundary vector reuses its earlier checker result;
its new metadata/import mode is independently exercised here. All staged pins
were certificate-domain hashed and read back byte-for-byte against source output.
Existing 44-source structural replay passed without changing any old metadata
or certificate pin (new and old source tests together 240.18s). Targeted lint
passed in 35.44s, edited-file formatting passed, and the two affected consumer
edge tests passed in 39.31s. A prior turn reported all three checker cases
passing in 741.73s. Their temporary log is no longer available after environment
recovery. The original pins are retained under previous-string-framing; changed
current certificates and their retained logs are tracked in
unit-4-json-string-framing-progress.json.

This does not yet emit the complete typed object/array/codec grammar, boundary
propositions, native/transition relations or application proof terms. Those
original W09 requirements remain open. No public route or checker rule changes,
no component-only commit, and no intermediate-W whole-project gate.
