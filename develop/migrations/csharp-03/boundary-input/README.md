# CSHARP-03-T05-W02 canonical input capture

`EmittedDataPhase::capture_boundary_input` is the private byte-only handoff for
one verification/reproduction run. Its inputs are an attached W01 boundary ID,
an opaque provenance ID, original media bytes and one canonical UTF-8 document.
The resulting `CapturedBoundaryRun` owns immutable copies of both byte sequences,
complete typed arguments, the input capture, frontend manifest and source-artifact
chain. There is no object-taking constructor or conversion from a T02 transport
receipt. A boundary-input-shaped sidecar also rejects rather than bypassing the
parser.

The shared practical JSON parser checks UTF-8, minimal tokens and escapes,
duplicate names and canonical spelling. It now preserves lone UTF-16 surrogates
in member names as well as values. Internal name tokens escape their own prefix
and never appear in canonical transport; the closed JSON enum and frozen CLI
source stay unchanged. Name/type lookup does not depend on member
order, but the frozen canonical document must still use boundary-contract order;
reordered documents reject. An adapter may reorder other media only by retaining
those original bytes separately and producing the canonical document.

Typed decoding reuses the contract/default decoder and W10 scalar codecs.
Wide integers use decimal strings, narrow integers use JSON integer tokens,
float values use exact bit strings, and decimal/date/time/duration/instant/GUID
values use their selected frozen codec. Maps use ordered key/value entry arrays.
Products and sum records require complete, ordered fields and exactly the active
payload. Missing, explicit null and payload values follow the W01 field plan.
Optional defaults are materialized explicitly in the typed canonical value;
therefore omission with a default and an explicit identical value have the same
value hash while their document hashes remain distinct.

Arguments retain the exact source type and, when projected, the ordinary imported
VIR reconstruction operation. Source constructors are not invented or executed
by this decoder. Pending source invariants and binding commutation obligations
remain attached for their existing T06 owner. Capture is evidence of typed input,
not proof discharge, profile activation or permission to invoke unproved source.
W03 owns encoding and reparsing returned source values.

The gate enforces 1 MiB canonical documents, 32 value edges, 16,384 UTF-16 units
per string/name, collection bounds and 65,536 aggregate typed cells. String units
count toward cells through the existing monomorphic-value validator. A bounded
lexical preflight limits parser allocation; its node budget allows schema/tag
metadata in addition to semantic cells. The canonical document byte limit is
separate from the frozen artifact transport limit, since explicit default/tag
evidence can be larger than the input document.

`import_boundary_input_run` reconstructs a run from bytes and compares all three
persisted artifacts against independently regenerated contents. In-memory replay
also checks field-complete typed values and both exact byte sequences. Changed
provenance, context, source snapshot, contract, value/hash or manifest linkage
rejects, including rehashed mutants. Adapter meaning preservation and production
adapter behavior are not proved. Public errors contain only closed error enums;
no parser exception prose, host path, culture or credential is copied into them.

## Retained evidence and reproduction

`conformance.json` retains 29 accepted inputs from 15 existing W01 source
snapshots, complete capture bytes, typed arguments and both manifest-chain hashes.
The same matrix rejects 28 malformed/ill-typed inputs and replays every accepted
input twice. W01 requests/responses remain unchanged in `../boundary-attachment/`.
`source-requests.json` and `source-responses.json` retain two additional actual
C# captures for independent aggregate-cell and document-byte boundary tests.
Those captures use the existing pinned Linux control-source producer; no new
producer or specialization implementation is introduced.

```sh
cargo test -p mpk-cli --test csharp_practical_boundary
cargo test -p mpk-vc --lib csharp_03_t05_w02
./scripts/check-fast.sh
```

Unit corpora cover scalar/text/date/time/GUID codecs, tagged sums, map/set entry
ordering and exact UTF-16/string/depth/array/parser limits. The CLI tests add
actual-source linkage, adapter/provenance/bypass mutations and limits at minus
one, exactly and plus one. Regenerate the matrix with
`MPK_W02_INPUT_EVIDENCE_OUT=/tmp/conformance.json` while running its test.
Regenerate source requests with `MPK_W02_LIMIT_REQUESTS_OUT=/tmp/requests.json`
while running the aggregate/document-limit test, then feed those requests to
`./scripts/build-csharp-practical-frontend.sh --test-control-emission-requests`
in the pinned local Linux image. Final verification and file hashes are recorded
in `verification.json`.
