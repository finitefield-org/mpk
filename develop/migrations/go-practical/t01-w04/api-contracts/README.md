# Common API v4 contract design

This is an inactive T01-W04 design. It closes the request, response and session
wire shapes for all 33 routes declared by the existing successor API. It does
not register a profile or claim that all declared endpoints have native
implementations. Practical C#'s reserved v3 identities remain separate.

The [catalogue](schema.json) and [validator](../../../../probes/go-practical/api_contracts.py)
give every route exact required argument/output fields and scalar types.
Unknown fields, omitted nullable fields, Boolean IDs, foreign versions and
cross-context selections reject. The structural proof routes retain their
checker-profile requirements. Theory requests reference a session-local theory
certificate; its format and applicability must still be checked by the actual
executor. Route declaration or request shape validation is not execution.
The HTTP method/path must equal the envelope's method/path before dispatch;
`GET /vc/list` retains the canonical request body convention. VIR import
returns unit/function counts as well as exact refs. For Go, the unit count is
the number of distinct captured package identities owning verified function
bodies; function count covers that complete verification closure. Other
profiles retain their actual VIR unit/function counts. VC generation returns
the whole independently validated common VC alongside its ref and ordinary
status. A reference alone cannot replace that existing usable output. The
[initial response coverage review](verification/initial-response-coverage/diagnostic.json)
preserves the preceding shape and the exact repair scope.

## Identity and transport

The roots are `mpk.ai.api.request.v4`, `mpk.ai.api.response.v4` and
`mpk.ai.api.session.v4`, under `mpk.ai.api.v4`. Their references are raw SHA256
of the **whole canonical root without LF**. There is no own hash field or new
API hash domain. Wire messages have exactly one LF after canonical narrow JSON.

The API envelope allows 269,484,032 canonical bytes, one additional LF, 772
container levels and 1,048,576 decoded bytes per string. This preserves the
existing 1MiB envelope allowance and adds four levels around the common
artifact budget. Embedded source/VC artifacts retain their own 256MiB,
768-container and semantic limits and are independently validated before
dispatch. The [copied iterative JSON algorithm](../../../../probes/go-practical/api_json.py)
changes only those two envelope constants. The unchanged native hash observer
checks the 24 small complete documents here; it is not evidence that API v4's
new full envelope limits are implemented natively.

Certificate chunks use lowercase hex, 32KiB raw chunks except the final
nonempty chunk, and the enclosing byte budget. This gives one segmentation.
The candidate hash uses the existing Certificate v0 domain over exact binary
bytes. Module import requires the independent checkers, exact exported
interface/axiom hashes, policy-required axiom restrictions and collision-free
interface import before committing any arena change. Module export embeds the
session's locally regenerated certificate manifest; a request supplies none.
Exported bytes remain a candidate even after local helper checks.

## State and transactions

The [state model](../../../../probes/go-practical/api_protocol.py) holds the full
source authority separately from requests. It revalidates the existing source,
frontend, manifest, VC and retained skeleton before creating a context. State
snapshots are helper data. A client cannot install a session by supplying a
snapshot, revision, hash, or readiness flag.

Context, selection and the supplied VIR compare as complete canonical bytes.
Python value equality would conflate integer `0` with Boolean `false`: an
initial review reproduced that private model defect with an unchanged claimed
VIR hash. The [exact original input and source snapshot](verification/initial-source-equality/diagnostic.json)
are preserved. The repaired model rejects it before changing source state.

Every session has an opaque server ID, an incarnation unique to the server's
lifetime, a safe unsigned revision, full context/selection, proof profile,
module name, lifecycle and source-state union. Production must obtain an
unpredictable incarnation before serving and must never reuse it after state
loss. The deterministic incarnation and `model-s1` in these vectors identify
only this model. A request after module creation must match the exact current
server snapshot. Successful mutating requests advance the revision once;
read-only calls and failures leave it unchanged. Exhaustion rejects. A stale
mutation cannot be replayed, including after server restart with the same ID.

Source states are `empty -> vir_imported -> vc_generated`. The last state keeps
exact source refs, VC ref, ordinary status and all unelaborated request IDs.
The Go bundles retain all 169 pending demands and list no invented ordinary
members. Candidate start/attach/check and source certificate export reject
while ordinary elaboration is pending. Ordinary term/proof helpers can be
available independently; they cannot clear source obligations or supply a
source theorem. The design model returns a distinct executor-required error
for those unimplemented operations instead of fabricating their results.
For retained profiles, `ready` means an ordinary VC can be targeted; it does
not mean any member has been proved.

Freeze prohibits every later mutating operation. Valid read-only listing and
export remain available subject to their original preconditions. The current
Rust lifecycle implementation exposes only `Active`, while the inherited
route specification requires freeze; closing that implementation gap is a
T07 task, not an observed behavior of the old seven-operation service.

Production dispatch must validate route, transport/shape, full context,
selection, current session, scalar/profile constraints and source bindings
before executing a route. Transport/shape/context/stale-snapshot failures
produce the existing out-of-band rejection diagnostic and no typed response
artifact. Valid requests that fail during an operation produce the closed
operation-error arm. Setup, resource, encoding and implementation failures
must remain failures, not successful transactions or theorem rejections.

Preflight diagnostics retain the existing `SuccessorApiError` code/phase
vocabulary: `ApiProfile` for a wrong schema/API family, `RouteUnknown` for a
wrong method/path, `JsonInvalid` for malformed JSON or framing, `Shape` for
missing/unknown fields, `Scalar` for invalid scalar domains, `UnknownSession`
for an unknown session ID, `SessionState` for an invalid lifecycle or stale
snapshot, `ContextMismatch` for full context/selection crossings, and
`CanonicalTransport` for strict but noncanonical JSON. Preserve their current
serialized `AI_API_V2_*` code strings when adapting into the v4 transport;
those stable diagnostic strings do not select the v2 request parser. The
Python model raises validation failures rather than implementing that HTTP
diagnostic adapter. The distinct operation-error codes are in the catalogue.

Prepare all fallible changes against detached session/arena copies, validate
the output and new source bindings, serialize within limits, and atomically
commit the graph, revision and response cache last. Candidate checking remains
read-only. Failed imports, dangling/cross-session IDs, invalid nodes, duplicate
declarations, checking failures and encoding failures must leak no interned IDs
or partial state. An implementation that allocates before validation must
rollback completely. A per-session commit lock serializes competing writers;
exact snapshot comparison rejects the loser. No request/response hash is an
authentication credential.

Every response and snapshot has `helper_only:true`. No root or route output
has an `accepted`, `mpk_verified` or `proof_evidence` authority field. To import
a response, independently regenerate the entire transaction using the same
server/source authority and compare the whole canonical response. Shape and
hash consistency alone cannot validate a supplied outcome. A transport cache
can return an identical locally retained response but cannot replay its state
mutation. The model retains locally generated bytes for this comparison.

## Evidence and ownership

The [bundles](bundles.json) contain 96 model exchanges over eight source
bundles and five profiles. The [vectors](vectors.json) cover all required and
unknown fields, every route's arguments, version/context/selection/session
crossings, readiness promotion and canonical transport. There are 48 atomic
operation failures in this model.

Six [fresh native queries](native-queries.json) run the unchanged public v2
service: 102 existing operations including 72 atomic failures. Their complete
[outputs](native.json) must equal the preceding policy record byte for byte.
They preserve actual old service behavior; they do not execute API v4.
Independent [Rust](rust-hashes.json) and [Go](go-hashes.json) raw canonical
hashes cover all three new roots across the eight bundles. The Go raw hashes
consume the Go canonicalizer's own output. Each observation runs twice.

T02-W03 owns the common source-artifact importer migration. T07-W01 owns
`crates/mpk-api/src/go_practical_api.rs`, the typed route adapters, full envelope
validation, source/ordinary target binding, transaction/replay/freeze behavior,
and the affected `crates/mpk-api/src/v2_tests.rs` preservation cases plus new
`crates/mpk-api/tests/go_practical_api.rs` tests. Every route must be classified
as implemented or explicit unsupported-operation rejection at publication;
the public route declaration alone cannot advertise support. T07-W03/W04 own
installed runner, activation, rollback and native Linux release evidence.

Run the [selected tests](../../../../probes/go-practical/test_api_contracts.py)
and [current metadata check](../../../../probes/go-practical/check_api_metadata.py)
as recorded in [verification](verification/verification.json). The
[observation receipt](observations.json) keeps design, compiled v2 execution
and future v4 implementation distinct. W04 remains open for AI/release roots,
ordinary-program signatures and the final migration/design review. W05 owns
inactive normative publication and the cumulative T01 gate. Full Go source,
elaborator, operation and application proofs remain required in T02–T06;
native Linux release gates and all remaining T02–T11 work remain active.
