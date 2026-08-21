# MPK AI API v1 Specification

Status: normative and frozen for implementation.

This specification defines the local, untrusted helper API profile
`mpk.ai.api.v1`. It is the active source-program API profile after the atomic
VIR cutover. It replaces the source import route from AI API v0 while retaining
the session, term, proof, certificate-module, and non-import VC operations
whose meaning is unchanged.

AI API v1 is intentionally incompatible with the GIR import boundary. There
is no GIR parser, route alias, request adapter, or hash-field fallback in this
profile.

## 1. Scope, conformance, and transport

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
the operation returns one structured error, publishes no output artifact, and
does not mutate any session table, counter, candidate, proof target, module, or
accepted-state marker.

Route matching uses the exact uppercase HTTP method and exact case-sensitive
path listed in section 3. A trailing slash, path normalization, percent-decoded
alternate spelling, method override, query-selected operation, or unlisted
path is not equivalent. In particular, `POST /gir/import` is an unknown route;
it is not a deprecated request and is never forwarded to `POST /vir/import`.

The v1 router's JSON request and response objects are closed. Unknown fields,
missing fields, duplicate object names, any JSON `null`, invalid UTF-8, a BOM,
and trailing bytes reject. Optional `ApiError` and `RepairDiagnostic` fields
are omitted as specified in section 7; they are never encoded as `null`.
Strings and enum values are case-sensitive. A `Sha256` is exactly 64 lowercase
hexadecimal ASCII characters. Integral values are in the JSON
interoperable range `[-9007199254740991, 9007199254740991]` unless a smaller
request type applies; floating-point values are absent from API-owned models.

The HTTP-like method/path notation is the exact router key. The staged v1
router is an in-process typed service and does not itself open a network
listener; every operation still receives the request model specified here.
Any later HTTP server binding must preserve the same method/path, canonical
model bytes, validation order, and errors and may not infer identity from
ambient process state.

The service is constructed with a read-only `ValidatedFrontendArtifactStore`.
Its only insertion path consumes a successful `mpk.frontend.cli.v0` result and
runs the complete VIR, source-map, source-manifest, release, and captured-input
validation before storing the exact canonical VIR, source-map, and manifest
bytes plus the closed validated input/release identities under the
frontend-stage manifest self-hash. Raw captured input bytes need not remain
after that constructor succeeds. API request JSON cannot insert, replace, or
deserialize a "validated" record. A later HTTP binding must use this same store
contract; having or guessing a manifest hash is not a validation capability.
Records are immutable for the service lifetime. Reinserting the same hash is
idempotent only when every retained canonical byte and validation identity is
equal; an unequal record under the same hash aborts store construction and is
never resolved by last-writer wins.

An operation that carries a JSON body uses RFC 8785 JCS followed by one ASCII
LF:

```text
JCS(request) || 0x0a
```

Responses use the same framing. Object-key order in an in-memory conformance
vector is not semantic; array order remains semantic. The import wrapper's
complete input is bounded by checked addition of the embedded VIR
`input_json_bytes` limit and 1 MiB of API-owned envelope overhead. The embedded
VIR's own `canonical_json_bytes`, nesting, string, node, and collection limits
still apply independently. The `/vc/generate` success response is bounded by
checked addition of the VC v1 `canonical_json_bytes` limit and 1 MiB of
API-owned envelope overhead. All unchanged endpoints retain their existing
operation-specific limits; v1 does not silently enlarge them.

The import-request and generate-response wrapper parsers permit exactly 257
JSON object/array levels: the referenced VIR/VC limit of 256 plus the one
API-owned enclosing object. Extracted standalone artifacts still reject above
256. Checked addition defines this relationship so a future artifact version
cannot inherit 257 accidentally. API-owned request fragments remain within the
unchanged endpoint limits and cannot use the extra wrapper level to deepen a
term, proof, or diagnostic value.

The router validates in this first-error order:

1. exact method/path resolution;
2. transport bytes, UTF-8, duplicate names, JSON depth, and numeric syntax;
3. closed request shape and required fields;
4. scalar IDs, hashes, profiles, and enum values;
5. session existence, lifecycle state, and repeated session/source identity;
6. complete artifact validation and repeated-identity linkage;
7. operation-specific context, ordering, and candidate checks;
8. canonical request transport equality;
9. one atomic state commit.

A later hash mismatch cannot hide an unknown route, malformed shape, stale
session, or invalid artifact. Validation may use private scratch state, but the
live session is changed only at step 9.

## 2. Trust and acceptance boundary

Everything constructed or returned by this API is helper data, including:

- session and interned term/proof IDs;
- imported VIR and generated VC identities;
- proof targets, attached candidates, strategy results, diagnostics, and
  `ok: true` candidate verdicts;
- frozen module state and exported canonical certificate bytes.

None of those values marks a theorem, declaration, module, package, or release
as accepted. Acceptance requires canonical `.mpcert` bytes to pass every
source-free checker required by the active policy, with the exact certificate,
checker, axiom, package, and release identities checked outside this helper
API. No endpoint may set, return, or imply a trusted `accepted`,
`mpk_verified`, or `proof_evidence: true` state.

`POST /module/import` imports a canonical checked-module/certificate interface;
it is not a source-IR import alias. `POST /module/export-certificate` only
serializes a candidate certificate. Neither endpoint bypasses checker
verification.

When an exported module contains declarations built from a session VC,
certificate assembly must consume that session's exact retained
frontend-stage manifest and VC. It follows `SOURCE_MANIFEST_V0.md`: validate
both again, add only the recomputed `vc_hash`, recompute the certificate-stage
manifest hash, and embed those exact opaque bytes. A request cannot supply or
replace a manifest at export time. This linkage makes the candidate
reproducible but does not make it accepted.

## 3. Exact route registry

The v1 registry contains exactly the routes below.

### 3.1 Session and module operations

```http
POST /module/new
POST /module/import
POST /module/freeze
POST /module/export-certificate
```

Their lifecycle and certificate semantics are unchanged from v0. A new module
creates one isolated session; imported checked interfaces, term/proof arenas,
VIR/VC bindings, and candidates never cross session IDs. Freeze prevents every
later mutating operation but still permits read-only list and export operations
that were already valid at the freeze point.

### 3.2 Term operations

```http
POST /term/sort
POST /term/var
POST /term/const
POST /term/app
POST /term/lam
POST /term/pi
POST /term/let
POST /term/check
POST /term/infer
POST /term/defeq
```

Interned IDs remain session-local. A term ID created by another session or a
failed request is unknown. Check, inference, and definitional-equality results
are deterministic diagnostics, not acceptance verdicts.

### 3.3 Proof operations

```http
POST /proof/exact
POST /proof/apply
POST /proof/intro
POST /proof/refl
POST /proof/let
POST /proof/rewrite
POST /proof/eq-rec
POST /proof/constructor
POST /proof/recursor
POST /proof/conv
POST /proof/theory
POST /proof/check-node
POST /proof/check-decl
```

`exact`, `apply`, `intro`, `refl`, and `conv` are the `core-bootstrap`
operations. `let`, `rewrite`, `eq-rec`, `constructor`, and `recursor` require a
compatible `mvp-structural` or `mvp-strict` checker profile. `theory` requires
the applicable checked theory-certificate contract. A `split` strategy hint
may expand to constructor or introduction nodes but is not a certificate proof
node or API route.

### 3.4 VIR and VC operations

```http
POST /vir/import
POST /vc/generate
GET  /vc/list
POST /vc/start-proof
POST /vc/attach-candidate
POST /vc/check-candidate
```

The five non-import VC operations preserve their v0 workflow meaning but are
bound exclusively to `mpk.vir.v0`, `mpk.vc.v1`,
`source_ir_schema`/`source_ir_hash`, and `source_vc_schema`/`vc_hash` identities
as defined below. There is no `source_gir_hash`, `gir_hash`, or unversioned VC
identity in v1 state or messages.

## 4. Session source and VC state

Each session has exactly these source-program states:

```text
empty
  -> vir_imported(source_ir_schema, source_ir_hash, source_language,
                  semantic_profile, semantic_parameters, vir_bytes)
  -> vc_generated(source_manifest_schema, frontend_source_manifest_hash,
                  input_set_hash, source_vc_schema, vc_hash, vc_bytes,
                  proof_targets: target_id -> target + candidates)
```

Starting a proof inserts one child target under `vc_generated`; it does not
replace the session source state. Candidate attachment and checking are child
records under that target and do not advance a trusted lifecycle. Multiple VC
members/groups may therefore have independent proof targets in one session. A
session may import exactly one VIR and generate exactly one VC value from it.
Repeating either artifact operation with the same bytes is not an idempotent
alias: it rejects as an invalid session transition. A caller that needs another
source or generation creates another session.

Store population is a host operation that precedes these routes and never
creates or changes an API session. Importing a valid standalone VIR succeeds
even when no matching store record exists; that session may use ordinary
term/proof helpers, but `/vc/generate` deterministically returns
`AI_API_SOURCE_CONTEXT_UNKNOWN` when the service was not constructed with an
exact matching validated frontend record. Store lookup never implicitly
imports or replaces the session VIR.

The implementation retains the exact canonical VIR, frontend-stage source
manifest, and VC bytes, not only caller-supplied hashes. Every later VC
operation repeats and compares the session ID, schema, and hash before reading
or changing target/candidate state. Stale or cross-session context rejects
before mutation.

## 5. `POST /vir/import`

The request object has exactly:

| Field | Rule |
|---|---|
| `session_id` | existing active session in `empty` source state |
| `source_ir_schema` | exactly `mpk.vir.v0` |
| `source_ir_hash` | exact embedded VIR `vir_hash` |
| `vir` | complete `mpk.vir.v0` JSON value |

`vir` is not a path, URI, artifact ID, hash-only promise, GIR object, or lossy
projection. Because the wrapper itself is JCS, extracting and JCS-encoding the
`vir` value yields the exact canonical standalone VIR bytes. The importer runs
the complete `VIR_V0.md` validation, including profile/parameter pairing,
closed shapes, deterministic limits, canonical size, and the
`MPK-VIR-0.1` self-hash. It then requires both repeated fields to equal the
validated artifact.

Validation accepts exactly these baseline source tuples:

| `source_language` | `semantic_profile` | `semantic_parameters` shape |
|---|---|---|
| `go` | `mpk.go.fixed.v0` | `target_id`, `pointer_width` |
| `rust` | `mpk.rust.checked.v0` | `target_id`, `pointer_width`, `overflow_mode = checked`, `panic_mode = abort` |

The parameter values must satisfy the selected registered target as specified
by `VIR_V0.md`; this table is not a second, looser registry.

The success response has exactly:

| Field | Rule |
|---|---|
| `session_id` | request session |
| `source_ir_schema` | `mpk.vir.v0` |
| `source_ir_hash` | recomputed VIR self-hash |
| `source_language` | validated VIR value |
| `semantic_profile` | validated VIR value |
| `semantic_parameters` | complete validated profile object |
| `unit_count` | checked count of VIR units |
| `function_count` | checked aggregate count of VIR functions |
| `helper_only` | exactly `true` |

Counts are derived locally and do not replace artifact validation. The
response deliberately contains no source selection, filesystem path, trusted
verdict, or certificate-acceptance flag.

## 6. VC context operations

### 6.1 Generate and list

`POST /vc/generate` requires exactly `session_id`, `source_ir_schema`,
`source_ir_hash`, `source_manifest_schema`,
`frontend_source_manifest_hash`, and `input_set_hash`.
`source_manifest_schema` is exactly `mpk.source_manifest.v0`; the other two
manifest fields repeat the selected validated frontend artifact's recomputed
`source_manifest_hash` and `input_set_hash`.

Generation runs only from `vir_imported`. It performs complete
`SOURCE_MANIFEST_V0.md` frontend-stage artifact revalidation over the exact
canonical manifest, source map, VIR, and closed validated input/release
identities retrieved from `ValidatedFrontendArtifactStore`, including absence
of `vc_hash`, release/input/artifact linkage, canonical size, and self-hash.
The non-serializable store record proves that captured-input bytes were checked
by its sole constructor; generation neither rereads paths nor treats a JSON
receipt as that proof. The stored VIR bytes and every repeated request identity
must equal the retained session VIR.
A caller-supplied matching hash without a corresponding validated store record
is insufficient and returns `AI_API_SOURCE_CONTEXT_UNKNOWN`. The operation then
generates the complete canonical `mpk.vc.v1` value under `VC_V1.md`, validates
it again, and commits only after the VC repeats the session's source IR,
manifest `input_set_hash`, semantic profile, semantic parameters, and
registered `mpk.verify.limits.v0` identity.

The success response has exactly `session_id`, `source_ir_schema`,
`source_ir_hash`, `source_manifest_schema`, `frontend_source_manifest_hash`,
`input_set_hash`, `source_vc_schema`, `vc_hash`, `function_count`,
`member_count`, `group_count`, `helper_only`, and `vc`. The two VC identities
are exactly `mpk.vc.v1` and the recomputed `MPK-VC-1.0` self-hash; all three
counts are checked counts over the embedded complete canonical VC;
`helper_only` is exactly `true`; and `vc` is that complete value. Extracting
and JCS-encoding `vc` yields the retained canonical standalone VC bytes. No
optional byte-returning branch or hash-only success response exists.

`GET /vc/list` is read-only. Its typed request has exactly six fields in this
logical order: `session_id`, `source_ir_schema`, `source_ir_hash`,
`input_set_hash`, `source_vc_schema`, and `vc_hash`. All six must equal retained
session state. Its response has exactly those six repeated fields, `members`,
and `helper_only = true`. `members` returns every VC member exactly once in
canonical VC function and member order. Each row has exactly `member_id`,
`function_id`, `kind`, and `group_id` copied from the validated VC. The response
never synthesizes source spans or proof status.

### 6.2 Proof targets and candidates

Every request in this subsection begins with the same six exact identity
fields as list: `session_id`, `source_ir_schema`, `source_ir_hash`,
`input_set_hash`, `source_vc_schema`, and `vc_hash`.

`POST /vc/start-proof` additionally has one `target` object, exactly
`{"kind":"member","id":MEMBER_ID}` or
`{"kind":"group","id":GROUP_ID}`. The ID must resolve exactly once in the
retained VC. For the containing function `F`, resolved member `M`, and resolved
group `G`, the canonical target types are exactly:

```text
MemberTarget(F, M) =
  ForallMany([P.type for P in F.parameters], MemberType(M))

GroupTarget(F, G) =
  ForallMany([P.type for P in F.parameters], GroupBody(F, G))
```

`ForallMany`, `MemberType`, and `GroupBody` are the exact constructions in
`VC_V1.md` section 7. `GroupTarget` is therefore the term represented by that
group's canonical `GroupedTheoremType`; `MemberTarget` uses the same outer
function binders around the one member proposition. The handler materializes
the selected target into the session term arena deterministically, and the
assigned `target_id` refers to that exact term rather than a caller-supplied
type. The response has exactly the six repeated identities, the exact request
`target`, a monotonically assigned session-local `target_id`, and
`helper_only = true`; assignment occurs only at atomic commit.

`POST /vc/attach-candidate` additionally has `target_id`, caller-supplied
target-local unique `candidate_id`, and one complete session-local `proof_root`
ID. Candidate identity is the tuple `(session_id, target_id, candidate_id)`;
the same caller string may occur under another target but never aliases it. The
operation creates the binding but performs no acceptance mutation. Missing
proof nodes, duplicate tuple IDs, mixed session/VC context, or frozen state
reject without reserving an ID. The response has exactly the six repeated
identities, `target_id`, `candidate_id`, `proof_root`, and
`helper_only = true`.

`POST /vc/check-candidate` additionally has `target_id`, exact
`mode = fail_fast_per_candidate`, and a nonempty `candidates` array. Each
candidate row has exactly one previously attached `candidate_id` and its exact
`proof_root`; candidate IDs are unique in request order, and both fields must
equal the retained target-local binding. Each receives one deterministic
structured helper verdict in the same order, and one rejected candidate does
not prevent independent candidates from being checked. A check does not
insert, delete, replace, commit, or mark a candidate as accepted module state.
Export remains a separate explicit operation.

The handler validates the complete candidate array and every binding before it
checks any proof root. An unknown, duplicate, stale, or cross-target binding is
therefore one operation rejection with no partial results. Only a proof-checker
failure for an otherwise valid binding becomes that candidate's `invalid`
helper result.

The check response has exactly the six repeated identities, `target_id`,
`mode`, `results`, and `helper_only = true`. `results` preserves request order
and is a closed union selected by `helper_status`: a valid helper candidate has
exactly `candidate_id`, `proof_root`, and `helper_status = valid`; an invalid
helper candidate additionally has `diagnostic`, the complete section 7
`RepairDiagnostic`, and `helper_status = invalid`. The words `valid` and
`invalid` describe only the local candidate check and are never certificate
acceptance.
`valid` requires the complete session-local proof DAG to check and its inferred
type to be definitionally equal to the canonical member or grouped theorem
type reconstructed from the retained VC target; checking a well-typed proof of
another proposition is `invalid`.

V1 adds no implicit candidate-commit route. A caller that wants a validated
candidate in an exportable module must use the unchanged declaration-checking
and module-freeze workflow; `/vc/check-candidate` never performs that step on
the caller's behalf.

## 7. Diagnostics and failure atomicity

The existing operation-level `ApiError` and proof-level `RepairDiagnostic` are
distinct contracts and remain distinct in v1. An operation rejection uses the
existing closed `ApiError` shape: required `code` and `message`, followed by
optional `field` and `detail`. Absent optional fields are omitted, never JSON
`null`. A new source-boundary rejection example is exactly:

```json
{
  "code": "AI_API_VIR_HASH",
  "message": "AI API v1 request rejected",
  "field": "source_ir_hash"
}
```

The `message` for every new code below is exactly the same ASCII string shown
in the example and never interpolates request or artifact values. `field`, when
present, is one fixed request field name owned by that error site. `detail`,
when an embedded validator caused the rejection, is only its exact stable
code; it never contains implementation prose or serialized JSON. Existing
session, term, and proof `ApiError` codes, messages, field selection, and
detail behavior are unchanged.

A proof-check failure, including an `invalid` candidate result, uses the
existing `RepairDiagnostic` shape. It requires `ok = false`, `error_code`,
numeric `node_id`, `context_summary`, and `repair_hints`; the four
expected/actual type/head fields are present only when known and otherwise are
omitted. `context_summary` contains only session-local numeric term IDs, and
`repair_hints` contains only the compiled closed hint enum. A successful proof
diagnostic uses `ok = true`, omits `error_code` and every expected/actual field,
and retains `node_id` plus the two empty arrays. No operation error is wrapped
in a repair diagnostic, and no `ApiError` is inserted into candidate results.

```json
{
  "ok": false,
  "error_code": "DEF_EQ_HEAD_MISMATCH",
  "node_id": 481,
  "expected_type_id": 921,
  "actual_type_id": 877,
  "expected_head": "Core.And",
  "actual_head": "Core.Or",
  "context_summary": [31, 44, 45],
  "repair_hints": ["split", "apply", "rewrite"]
}
```

Neither contract contains raw source, VIR/VC bytes, compiler prose, a path, a
credential, or an unbounded parser/provider error.

The v1 boundary uses these new and inherited stable codes. `UNKNOWN_SESSION`
and `UNKNOWN_PROOF` are the unchanged existing `ApiError` codes; every
`AI_API_*` row is new to this profile.

| Code | Meaning |
|---|---|
| `AI_API_ROUTE_UNKNOWN` | exact method/path pair is not registered, including `POST /gir/import` |
| `AI_API_JSON_INVALID` | malformed transport, duplicate name, invalid UTF-8/number, or limit failure |
| `AI_API_SHAPE` | missing, unknown, retired, or wrong-union field |
| `AI_API_SCALAR` | malformed session, target, candidate, hash, profile, or enum scalar |
| `UNKNOWN_SESSION` | session ID does not exist; inherited unchanged |
| `AI_API_SESSION_STATE` | operation is not valid in the current/frozen state |
| `AI_API_VIR_SCHEMA` | source schema is not exactly `mpk.vir.v0` |
| `AI_API_VIR_INVALID` | embedded VIR fails complete VIR validation |
| `AI_API_VIR_HASH` | repeated source IR hash differs from recomputed `vir_hash` |
| `AI_API_SOURCE_CONTEXT_UNKNOWN` | no validated frontend artifact exists for the requested manifest identity |
| `AI_API_SOURCE_MANIFEST_SCHEMA` | source-manifest schema is not exactly `mpk.source_manifest.v0` |
| `AI_API_SOURCE_MANIFEST_INVALID` | stored frontend-stage artifacts fail complete revalidation or VIR/source-map linkage |
| `AI_API_SOURCE_MANIFEST_HASH` | repeated input-set hash differs from the resolved validated manifest |
| `AI_API_VC_INVALID` | generated or retained VC fails complete VC v1 validation |
| `AI_API_CONTEXT_MISMATCH` | session/source/VC/target/candidate identities are mixed or stale |
| `AI_API_TARGET_UNKNOWN` | member/group proof target does not resolve exactly once |
| `UNKNOWN_PROOF` | attached `proof_root` does not exist in the session; inherited unchanged |
| `AI_API_CANDIDATE_UNKNOWN` | target-local candidate binding does not exist |
| `AI_API_CANONICAL_TRANSPORT` | valid value was not sent as required JCS plus LF |

If an embedded validator returns a more specific `VIR_*`, `SOURCE_MANIFEST_*`,
or `VC_*` code, the API returns `AI_API_VIR_INVALID`,
`AI_API_SOURCE_MANIFEST_INVALID`, or `AI_API_VC_INVALID`, respectively, as
`code` and places the specific stable code in the optional `detail` field. A
wrong repeated manifest schema returns
`AI_API_SOURCE_MANIFEST_SCHEMA`; a resolved valid manifest whose repeated
input-set hash differs instead returns `AI_API_SOURCE_MANIFEST_HASH`. The API
does not copy implementation prose into the public response.

All operations use copy/validate/commit or an equivalent transaction. Tests
snapshot the complete session summary and all counters before every negative
case and require byte-for-byte equality afterward.

## 8. Hash and schema registry

The v1 API uses these exact identities:

| Purpose | Schema/domain |
|---|---|
| API profile | `mpk.ai.api.v1` |
| imported source IR | `mpk.vir.v0`, self-hash domain `MPK-VIR-0.1` |
| source map in validated store record | `mpk.source_map.v0`, self-hash domain `MPK-SOURCE-MAP-0.1` |
| source context | `mpk.source_manifest.v0`, self-hash domain `MPK-SOURCE-MANIFEST-0.1` |
| manifest input set | `input_set_hash`, domain `MPK-INPUT-SET-0.1` over `JCS(inputs)` |
| generated VC | `mpk.vc.v1`, self-hash domain `MPK-VC-1.0` |
| grouped skeleton | `mpk.vc.cert_skeleton.v1` |
| canonical certificate | `CERT_V0.md` domains, unchanged |

`source_ir_hash` is the VIR self-hash, not SHA-256 of a pretty or LF-framed API
request. `vc_hash` is the VC v1 self-hash, not a source-manifest hash. Vector
fields named `canonical_request_sha256` or `canonical_response_sha256` are
test assertions over exact API transport bytes and are not public request or
response fields.

The exact retired source-import tokens `POST /gir/import`, `/gir/import`,
`source_gir_hash`, `gir_hash`, `mpk.gir.v0`, and `mpk.vc.cert_skeleton.v0` have
no active interpretation. A closed shape rejects retired fields before any
attempt to compare their values.

## 9. Conformance vectors and ownership

`develop/specs/vectors/ai-api-v1.json` has schema
`mpk.ai.api.conformance.v1` and exact top-level fields `schema`, `api_profile`,
`dependencies`, `owner_test`, `route_registry`, `error_contract`,
`artifact_contexts`, `import_fixtures`, `generate_fixtures`, `route_cases`,
`import_cases`, and `context_cases`.

The route registry contains every section 3 method/path pair exactly once and
in section order. `error_contract` has exactly `operation_shape`,
`operation_example`, `proof_diagnostic_shape`,
`proof_diagnostic_example`, `optional_field_encoding`, and
`forbidden_dynamic_sources`. Its examples equal section 7, its optional-field
rule is `omit_when_absent`, and the forbidden-source list applies recursively
to both serializations. `api_error_v0` and `repair_diagnostic_v0` are
vector-only model labels, not serialized schema or profile identifiers. An
artifact context references either one
accepted VIR vector case or one complete locally generated VC fixture together
with its validated source context; it repeats the applicable exact schema and
self-hash identities. The VIR form has exactly `id`, `vir_case`,
`source_ir_schema`, `source_ir_hash`, `source_language`, `semantic_profile`,
and `semantic_parameters`. The VC form replaces `vir_case` with exact `vc_fixture`
and `vc_source_context` references; retains all source identity/profile fields;
and adds `source_manifest_schema`, `frontend_source_manifest_hash`,
`input_set_hash`, `source_vc_schema`, and `vc_hash`. Its `vc_fixture` resolves
only to a complete `vc` embedded in a generate fixture's expected response. Its
string is exactly `GENERATE_FIXTURE_ID.expected_response.vc`: the owner removes
the literal final `.expected_response.vc` suffix and resolves the remaining
complete string as the fixture ID, including any dots in that ID. An
import fixture always uses the VIR form and has exactly `id`,
`artifact_context`, `request`,
`canonical_request_utf8_length`,
`canonical_request_sha256`, and `expected_response`. The digest covers
`JCS(request) || 0x0a`.

A generate fixture has exactly `id`, a VIR-form `artifact_context`,
`source_manifest_case`, `request`, `canonical_request_utf8_length`,
`canonical_request_sha256`, `expected_response`,
`canonical_response_utf8_length`, and `canonical_response_sha256`.
`source_manifest_case` resolves to an accepted frontend-stage case in
`source-manifest-v0.json`; the owner follows that case's VIR, source-map,
release, and captured-input context through their owning validators and inserts
the resulting typed capability into `ValidatedFrontendArtifactStore`. It never
inserts the manifest JSON directly. The request and response digests cover
their respective `JCS(value) || 0x0a` bytes. `expected_response.vc` must pass the
complete VC v1 validator, its self-hash must equal every repeated `vc_hash`, and
its counts and all source/manifest identities must equal the referenced
validated inputs.

A route case has exactly `id`, `method`, `path`, and `expect`; a resolved case
names the exact handler, and a rejected case names route phase and code. An
import case has exactly `id`, one of `input_from`, `construction`,
`transport_from`, or `json_text`, and `expect`. Construction has exact `base`
and `operations`; operations are ordered RFC 6901/RFC 6902 `add`, `remove`, or
`replace` records and never repair a dependent field implicitly.
`transport_from` has exactly `fixture` and
`encoding = two_space_indent_with_final_lf`; the owner pretty-serializes the
fixture request with two-space indentation and one LF and submits those bytes
without normalization. `json_text` is submitted as its exact UTF-8 bytes and
permits duplicate-name/invalid-framing tests.

A context case has exactly `id`, `operation`, `session_state`, one of `request`,
`request_from`, or `construction`, and `expect`. `request_from` names an import
or generate fixture and selects its request. `construction` has exact `base`
and `operations`, selects such a fixture request, and applies the same ordered
closed patch records as import cases without repairing dependent fields.
`session_state` is a vector-only typed snapshot containing the session ID,
lifecycle state, and artifact-context reference, plus only the target/candidate
or explicitly modeled foreign-session fields needed by that operation. When a
candidate case needs proof state, the snapshot additionally contains exactly
`proof_recipe` and `proof_root`; neither field is permitted otherwise. The two
closed recipes are:

- `proof.go_identity_member.refl`: starting from an empty proof arena, resolve
  the Go identity fixture's postcondition member target. Construct the bound
  `arg0` reflexivity proof of `Std.Eq(arg0, arg0)` with `/proof/refl` as root
  `0`; introduce the canonical `Std.Bool.true` antecedent with `/proof/intro`
  as root `1`; then introduce the outer `Std.Program.Base.Int8` function
  parameter with `/proof/intro` as root `2`. Root `2` checks against the exact
  `MemberTarget` and produces `helper_status = valid`.
- `proof.go_identity_member.wrong_head`: starting from an empty proof arena,
  construct `/proof/refl` for `Std.Bool.true` with expected type
  `Std.Eq(Std.Bool.true, Std.Bool.true)` as root `0`. The node is independently
  well typed, but its equality head is not the identity member's outer Pi
  target; candidate checking produces `helper_status = invalid` with
  `DEF_EQ_HEAD_MISMATCH`.

The owner realizes each recipe through the unchanged term and proof operations
and checks every returned numeric ID. It must not seed a proof arena or
deserialize a recipe snapshot into production state. Apart from this recipe,
the owner constructs real service state through successful operations and
validated fixture builders; it must not deserialize `session_state` into
production state.
`expect` contains `outcome` and `mutation_count`, plus `phase` and `code` for
operation rejection. An accepted generate case additionally has
`response_from`, requiring byte equality with that fixture's expected response.
A helper-level candidate result instead contains `proof_acceptance = false`
and, for an invalid proof, exact `diagnostic_code` equal to the returned
`RepairDiagnostic.error_code`.
Every operation rejection and helper rejection has `mutation_count = 0`.

All vector objects are closed. The owning test rejects unknown fields,
duplicate case IDs, an unexecuted case, a route-order mismatch, a fixture hash
mismatch, or a case whose before/after session snapshot differs despite zero
expected mutations. The staged owner is
`crates/mpk-api/src/v1_tests.rs`; it remains private until the atomic cutover
task publishes the v1 router.

## 10. References

- `develop/specs/AI_API_V0.md` — current pre-cutover route profile; it becomes
  historical only at the atomic cutover
- `develop/specs/FRONTEND_PROTOCOL_V0.md` — validated frontend-result boundary
- `develop/specs/RELEASE_BUNDLES_V0.md` — registered release identities used by the validated store
- `develop/specs/VIR_V0.md` — canonical source IR and `MPK-VIR-0.1`
- `develop/specs/SOURCE_MAP_V0.md` — canonical source locations held by the validated store
- `develop/specs/SOURCE_MANIFEST_V0.md` — frontend source context and input-set identity
- `develop/specs/VC_V1.md` — VC/skeleton v1 and `MPK-VC-1.0`
- `develop/specs/CERT_V0.md` — unchanged certificate/checker boundary
- `develop/specs/TRUST_BOUNDARY_V0.md` — project-wide trust rules
