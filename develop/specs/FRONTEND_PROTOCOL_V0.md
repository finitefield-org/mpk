# Generic Frontend CLI Protocol v0 Specification

Status: normative and frozen for implementation.

Schema "mpk.frontend.cli.v0" is the only public child-process protocol used by
the Go and Rust VIR frontends. The response is untrusted input to the generic
runner. Accepting a response does not make the frontend, VIR, source map, or
source manifest proof evidence.

## 1. Conformance language and closed objects

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
the consumer returns no VIR, source map, source manifest, VC, certificate, or
evidence derived from the response.

Every object is closed. A listed field is required unless a status branch says
that it is absent. Unknown or inapplicable fields, missing required fields,
duplicate JSON names, and null reject. Strings and enum values are
case-sensitive. Floating-point JSON numbers and integers outside
[-9007199254740991, 9007199254740991] reject.

All JSON follows the RFC 8785 JCS restrictions in VIR_V0.md: valid Unicode
scalar strings without additional normalization, compact JCS encoding, and no
BOM or insignificant bytes. A Sha256 is exactly 64 lowercase hexadecimal
characters.

## 2. Request identities

The runner validates the complete caller configuration and resolves one release
tuple before starting a frontend. A launched frontend therefore receives one
fixed source language, semantic profile, target, pointer width, limit profile,
selection, release-registry identity, frontend bundle, and toolchain bundle.
There is no frontend-selected default.

The only language/profile and semantic-parameter pairs are:

| source_language | semantic_profile | semantic_parameters |
|---|---|---|
| "go" | "mpk.go.fixed.v0" | {"target_id": TargetId, "pointer_width": 32 or 64} |
| "rust" | "mpk.rust.checked.v0" | {"target_id": TargetId, "pointer_width": 32 or 64, "overflow_mode": "checked", "panic_mode": "abort"} |

The exact selection union is selected by "source_language".

GoSelection has exactly:

~~~json
{"package":"example.com/mpk/vector","function":"example.com/mpk/vector.Identity"}
~~~

"package" is the canonical selected import path. "function" is a canonical Go
function ID from VIR_V0.md, belongs to that package, and is the exact
caller-supplied function.

RustSelection has exactly:

~~~json
{"package":"payment-policy","crate":"payment_policy","kind":"lib","function":"payment_policy::approved_reserve_cents"}
~~~

"package" is the accepted Cargo package name, "crate" is the selected library
crate name and first segment of "function", "kind" is literally "lib", and
"function" is the exact caller-supplied canonical Rust function ID. Neither
selection branch contains a path, Cargo package ID, target directory, or
compiler-local identity.

The consumer constructs this object before launch. Every envelope repeats it
member-for-member. A frontend cannot repair, derive, or substitute any member.

## 3. Envelope phases

The fixed processing order is:

1. "configuration": CLI syntax, mandatory profile and target, registry
   assertions, and caller bundle IDs;
2. "release": installed registry, tuple, bundle, executable, toolchain, host,
   runtime, and sandbox capability validation;
3. "capture": immutable input capture and structural preflight;
4. "source": language parsing and the pre-expansion/source gate;
5. "metadata": deterministic package/module metadata validation;
6. "typecheck": name, type, borrow, and compiler-language checking;
7. "subset": HIR/typed-source subset and contract resolution;
8. "lowering": SSA/MIR validation and VIR lowering;
9. "emission": canonical VIR, source-map, manifest, and envelope emission.

The first two phases are owned by the caller. "configuration" failure is the
exit-2 no-JSON case. A "release" failure is a caller-local artifact-free
frontend-error and no child envelope exists. The child owns phases "capture"
through "emission".

A phase completes before the next begins. Once a phase fixes a non-success
status, no later phase runs and no later diagnostic is mixed into the result.
Within one phase, a language profile freezes its finer code precedence.
Operational, identity, process, protocol, or internal-invariant failure always
beats a semantic classification from work that did not complete.

## 4. Exact status-tagged envelopes

Every envelope begins with these seven exact common fields:

| Field | Rule |
|---|---|
| "schema" | exactly "mpk.frontend.cli.v0" |
| "status" | one status below |
| "phase" | deepest started child phase that owns the result |
| "source_language" | exact request language |
| "semantic_profile" | exact request profile |
| "semantic_parameters" | exact request object |
| "selection" | exact language-specific request selection |

### 4.1 "ir-lowered"

The exact fields are the seven common fields plus "ir", "source_manifest",
"source_map", "rejected_features", and "diagnostics". "phase" is exactly
"emission". Exit status is exactly 0.

"ir" has exactly:

| Field | Rule |
|---|---|
| "schema" | exactly "mpk.vir.v0" |
| "sha256" | equals the enclosed VIR "vir_hash" |
| "value" | one complete validated VirModule |

"source_map" is one complete "mpk.source_map.v0" value.
"source_manifest" is one frontend-stage "mpk.source_manifest.v0" value and
therefore has no "vc_hash". "rejected_features" is empty. "diagnostics" may
contain normalized non-fatal diagnostics. No debug, SSA, HIR, MIR, compiler,
binary wrapper, partial hash, or compatibility object is permitted.

### 4.2 "rejected"

The exact fields are only the seven common fields plus "rejected_features" and
"diagnostics". Exit status is exactly 3. "phase" is one of "capture",
"source", "metadata", "subset", "lowering", or "emission".

The concatenation of the two issue arrays is nonempty. This status means valid
language input or structurally captured input used a feature outside the closed
profile, failed a contract rule, or exceeded a deterministic source/IR profile
limit. It contains no "ir", "source_map", "source_manifest", "sha256", or
other partial artifact field.

### 4.3 "source-error"

The exact fields are only the seven common fields plus "rejected_features" and
"diagnostics". Exit status is exactly 4. "phase" is one of "capture", "source",
"metadata", or "typecheck". "rejected_features" is empty and "diagnostics" is
nonempty. "capture" is used for an ordinary parse/name failure discovered by
deterministic module-closure capture, such as a missing or ambiguous module;
symlink/root-escape policy failures remain "rejected".

This status is restricted to malformed or ill-typed language input rejected by
the loader or compiler. A syntactically valid but unsupported construct is
"rejected"; a compiler crash or protocol failure is "frontend-error".

### 4.4 "frontend-error"

The exact fields are only the seven common fields plus "rejected_features" and
"diagnostics". Exit status is exactly 1. "phase" is any child-owned phase from
"capture" through "emission". "rejected_features" is empty and "diagnostics"
is nonempty.

This status covers a frontend-detected process failure, compiler crash,
toolchain or repeated-identity mismatch, sandbox failure, private protocol
failure, canonicalization failure, and internal invariant violation. A
frontend that can still report the failure emits this envelope. The consumer
never requires a child to synthesize an envelope after the child can no longer
respond.

### 4.5 CLI configuration error

Exit status 2 writes zero bytes to stdout and no envelope. Help output is not a
"lower" response and is outside this protocol. A missing, malformed, unknown,
or wrong-language semantic profile; missing target or selection; unknown bundle
ID; registry assertion mismatch; incompatible registered tuple; and invalid
launcher-only option are configuration errors detected before child launch.

If a supposedly prevalidated child nevertheless exits 2, the generic runner
classifies it locally as "frontend-error" with
"FRONTEND_PROTOCOL_UNEXPECTED_USAGE"; it does not expose a second
configuration result.

## 5. Normalized issues

"rejected_features" and "diagnostics" contain the same closed Issue shape:

| Field | Presence and rule |
|---|---|
| "code" | required stable uppercase MPK code |
| "message" | required normalized concise UTF-8 message |
| "function_id" | optional; exact canonical VIR function ID |
| "span" | optional SourceSpan |

SourceSpan has exactly "normalized_path", "start", and "end".
"normalized_path" uses SOURCE_MANIFEST_V0.md portable input paths. "start" and
"end" are JSON integers and a zero-based half-open UTF-8 byte range satisfying
0 <= start < end <= the captured input size. It names only an input permitted
for the reporting phase. Line and column numbers are display-only and never
canonical fields.

The emitting frontend enforces captured-input membership, size, and UTF-8
boundaries from its immutable buffers. For a non-success envelope, the outer
consumer has no manifest and therefore validates only path grammar, integer
ordering/range, normalization, and public-path exclusion; it does not reread
the source tree or treat the untrusted span as proof of input membership. For
"ir-lowered", the consumer additionally cross-checks every diagnostic span
against the returned validated manifest and captured source inventory.

"function_id" is required for an issue owned by phases "subset", "lowering",
or "emission" and for any earlier issue attributed to one resolved function.
It is absent for crate/package-wide capture, source, metadata, or typecheck
issues whose function has not resolved. When present it equals a function in
the successful VIR, or, on non-success, the exact canonical selected/dependency
function identity already resolved before that issue. The truncation marker is
the sole issue that is always function-free.

"code" is 1 through 128 ASCII bytes, starts with an ASCII uppercase letter,
and otherwise contains only uppercase letters, digits, and underscore.
Language-owned child codes use the families declared by that language profile.
The shared caller and importer own "FRONTEND_REGISTRY_*",
"FRONTEND_BUNDLE_*", "FRONTEND_PROCESS_*", "FRONTEND_PROTOCOL_*", and
"VIR_*". A frontend cannot emit a caller-owned protocol code to make malformed
transport self-authenticating.

Raw Cargo, rustc, go, compiler, or child stderr is forbidden. Normalization
removes ANSI and control sequences, source snippets, rendered expansion text,
child argv/environment, host suggestions, and all machine-local paths. An
accepted source locator is replaced by its normalized input path. A detail that
cannot be normalized is omitted. "message" is at most 4,096 UTF-8 bytes.

Each array is independently sorted by this ascending key:

1. span path, or the empty byte string when absent;
2. span start, or 0 when absent;
3. code;
4. message;
5. function ID, or the empty byte string;
6. span end, or 0.

String comparisons use UTF-8 bytes. Exact duplicate Issue values are retained
and count independently; because their canonical bytes are equal, their
relative order is unobservable.

### 5.1 Deterministic truncation

The two arrays together contain at most 1,024 entries and at most 2,097,152
message UTF-8 bytes. A message longer than 4,096 bytes is first reduced to the
longest scalar-boundary prefix of at most 4,084 bytes followed by the exact
ASCII suffix " [truncated]".

After normalization and independent sorting, concatenate
"rejected_features" then "diagnostics". If the combined entry and message
budgets fit, no marker is added. Otherwise let N be the candidate count. Choose
the greatest k no larger than 1,023 such that the first k candidates plus the
marker message fit the message budget. Retain exactly that prefix in its
original arrays, omit N-k candidates, and append one marker as the final
"diagnostics" entry.

The marker has no "function_id" or "span". Its code is
"GO_LIMIT_DIAGNOSTICS_TRUNCATED" for Go or
"RUST_LIMIT_DIAGNOSTICS_TRUNCATED" for Rust. Its message is exactly
"<N-k> normalized issues omitted", using canonical unsigned decimal. The
marker does not change "status" or "phase". If even the marker cannot fit, the
frontend emits "frontend-error" at "emission" with no span, the exact
"selection.function" as "function_id", message
"diagnostic budget invariant failed", and code
"GO_FRONTEND_DIAGNOSTIC_BUDGET" or
"RUST_FRONTEND_DIAGNOSTIC_BUDGET" according to the fixed source language. This
condition is unreachable for the specified positive budgets when counters are
implemented before allocation.

## 6. Transport

Every JSON-bearing exit writes exactly:

~~~text
JCS(envelope) || 0x0a
~~~

The complete stdout maximum is 268,435,456 bytes (256 MiB), including the one
LF. The JSON portion is therefore at most 268,435,455 bytes. Exit 2 writes no
stdout. A JSON-bearing exit has exactly one LF; missing LF, CRLF, a second LF,
trailing whitespace, a second JSON value, a BOM, or any other stdout byte is
not accepted. Frontend stderr is captured separately up to 2,097,152 observed
bytes, is never parsed into the envelope, and never enters an artifact hash.

The consumer validates in this order:

1. streaming stdout and stderr limits;
2. spawn result and signal/kill status;
3. exit-2 empty-stdout special case;
4. response presence, existence of a complete first JSON value, and required
   terminal LF;
5. first-value UTF-8/JSON lexical form, duplicate names, and JSON nesting;
6. closed envelope/status shape, normalized issue/path rules, and exact exit
   pairing;
7. JCS re-encoding plus LF byte equality;
8. repeated request identities and release selection;
9. VIR, source-map, manifest, selected-function, and hash linkage.

The first failing step owns the local code. The complete outer result remains
"frontend-error" and contains no child artifact.

| Condition | Local code |
|---|---|
| selected executable or registered byte missing before spawn | "FRONTEND_BUNDLE_INVALID" |
| spawn fails | "FRONTEND_PROCESS_SPAWN" |
| externally killed, signaled, or resource-terminated child | "FRONTEND_PROCESS_KILLED" |
| zero stdout on an exit other than 2 | "FRONTEND_PROTOCOL_MISSING" |
| nonempty stdout lacks one complete framed JSON value | "FRONTEND_PROTOCOL_TRUNCATED" |
| invalid UTF-8/JSON, duplicate name, or forbidden JSON number | "FRONTEND_PROTOCOL_MALFORMED" |
| parseable response has a wrong/unknown status shape | "FRONTEND_PROTOCOL_SHAPE" |
| status and process exit disagree | "FRONTEND_PROTOCOL_STATUS_EXIT" |
| a launched, prevalidated child exits 2 | "FRONTEND_PROTOCOL_UNEXPECTED_USAGE" |
| parseable value is not exact JCS plus LF | "FRONTEND_PROTOCOL_NONCANONICAL" |
| stdout or stderr exceeds its streaming ceiling | "FRONTEND_PROTOCOL_LIMIT" |
| repeated language/profile/target/selection/release identity differs | "FRONTEND_PROTOCOL_IDENTITY_MISMATCH" |
| nested artifact, selected function, or hash linkage differs | "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH" |

When the consumer kills a child because a stream crossed its ceiling,
"FRONTEND_PROTOCOL_LIMIT" owns precedence over the resulting signal. A
complete parseable but whitespace-varied response is
"FRONTEND_PROTOCOL_NONCANONICAL", not "MALFORMED". A prefix that would become
valid if more bytes arrived is "TRUNCATED". No condition reuses partial stdout
or an artifact from a prior invocation.

For classification only, the consumer locates the first complete JSON value
without discarding any byte from the equality check. If that first value is
invalid, the result is "MALFORMED". Once a complete first value is parseable,
a BOM, leading/trailing whitespace, CRLF, extra LF, second value, or any other
extra stdout byte is "NONCANONICAL". A complete JSON value without its required
transport LF is "TRUNCATED". This makes every parseable non-JCS transport use
"FRONTEND_PROTOCOL_NONCANONICAL" while retaining distinct missing, truncated,
and malformed classes.

## 7. Success cross-checks

For "ir-lowered", the consumer MUST validate all of these before returning:

- envelope language/profile/parameters equal the request and enclosed VIR;
- "ir.schema", VIR "schema", and source-map "source_ir_schema" are
  "mpk.vir.v0";
- "ir.sha256", VIR "vir_hash", source-map "source_ir_hash", and manifest
  "vir_hash" are equal and independently recomputed;
- manifest language/profile/parameters/selection/target equal the envelope and
  request;
- source-map hash equals manifest "source_map_hash";
- manifest release registry, frontend, toolchain, and limit profile equal the
  validated selected release descriptors and snapshotted binaries;
- manifest unit set equals VIR units and the selected function resolves
  exactly once;
- source-map entries resolve against VIR and source-kind manifest inputs;
- input-set, source-map, and frontend-stage source-manifest hashes recompute.

Any mismatch is a local "frontend-error"; a child success cannot downgrade it
to "rejected" or "source-error".

## 8. Public path exclusion

No string in an envelope or nested canonical artifact may contain an absolute
source, workspace, installation, toolchain, executable, home, temporary,
snapshot, sandbox, output, or registry path. POSIX-rooted paths, drive-letter
paths, UNC paths, file URIs, and the private "/mpk/" namespace are forbidden.
The only public paths are SOURCE_MANIFEST_V0.md normalized relative input paths.
Diagnostic messages are checked after normalization under the same rule.

## 9. Limits

The protocol parser additionally uses:

| Limit ID | Inclusive maximum |
|---|---:|
| "stdout_transport_bytes" | 268,435,456 |
| "stderr_observed_bytes" | 2,097,152 |
| "json_nesting" | 256 levels |
| "string_bytes" | 1,048,576 per decoded string |
| "issues" | 1,024 combined after truncation |
| "issue_message_bytes" | 4,096 per normalized message |
| "combined_issue_message_bytes" | 2,097,152 |

Nested VIR, source-map, and source-manifest limits are enforced independently.
The outer stream limit is checked first and cannot be evaded by a small nested
artifact.

## 10. Conformance vectors and ownership

"develop/specs/vectors/frontend-protocol-v0.json" has schema
"mpk.frontend.protocol.conformance.v0". Its exact top-level fields are
"schema", "spec_schema", "dependencies", "owner_tests", "status_cases",
"transport_cases", "identity_cases", and "limit_cases".

A case has an ID, one input source, process facts where applicable, and an
"expect" object. An input source is exactly one of "input", "input_from", or
"construction"; the exit-2 and raw-transport cases may omit a JSON input.
"input_from" resolves the completed input value of an earlier case. A normal
construction names an earlier "base" and ordered RFC 6901/RFC 6902
add-remove-replace patches as defined by VIR_V0.md. Patches do not implicitly
repair nested hashes. The named dependency cases are loaded from the three
vector files, not silently replaced by local fixtures.
"canonical_from_dependencies" constructs the success envelope from the exact
referenced VIR, source map, and frontend-stage manifest values.

Transport constructions operate on UTF-8 bytes after envelope construction.
"canonical" means JCS plus LF; "pretty", "missing_lf", "extra_lf", "bom",
"second_value", "duplicate_root_key", and "truncate" perform only the named
mutation.
"stream_bytes" generates exactly the requested observed byte count without
storing a 256 MiB string in Git.

"process" has exactly one of integer "exit" or string "signal", plus the
applicable stdout/stderr source. A stream source is "canonical" transformation,
literal UTF-8, base64, or a construction, never more than one. "stderr_bytes"
and "stream_bytes" repeat one byte to an exact checked count.
"normalized_message" uses ASCII "x" input and applies section 5.1.
"normalized_issues" creates distinct already-sorted codes/messages, visits the
requested feature/diagnostic counts in protocol field order, supplies the
required "selection.function" identity for its "subset" base phase, and applies
the same retention algorithm. "expect" has "outcome" and, for local failure,
"code"; accepted cases may freeze status, phase, lengths, marker, and retained
counts.

The owning tests MUST load every case, reject unknown case fields, verify that
IDs are unique across arrays, and prove that no case is skipped.
