# Generic Source Manifest v0 Specification

Status: normative and frozen for implementation.

Schema "mpk.source_manifest.v0" records the reproducibility inputs and selected
release identities for one successful VIR lowering. It is an untrusted
traceability artifact. It may be embedded as the opaque certificate
SourceManifest payload, but a source-free checker does not interpret it for
proof acceptance.

## 1. Conformance language and validation order

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
no validated manifest or downstream artifact derived from it is returned.

Every object is closed. Unknown or inapplicable fields, missing required fields,
duplicate JSON names, and null reject. Strings and enums are case-sensitive.
Floating-point numbers and integers outside
[-9007199254740991, 9007199254740991] reject. A Sha256 is exactly 64 lowercase
hexadecimal characters.

RegistryId, BundleId, ProfileId, ComponentName, ExecutableName, Version, Rust
compiler commit values, and their length/character rules are exactly those in
RELEASE_BUNDLES_V0.md. This specification projects those validated values and
does not introduce aliases or looser scalar forms.

Frontend-stage validation uses this first-error phase order:

1. "transport": enclosing byte and collection-count limits, UTF-8, JSON
   nesting, string limit, duplicate-name detection, and number syntax;
2. "shape": closed objects/unions, schema, and absence of "vc_hash";
3. "scalar": IDs, versions, hashes, input sizes, and portable paths;
4. "order": component, subordinate, unit, input, feature, cfg, and path
   ordering/uniqueness;
5. "semantic": language/profile/parameters/target/configuration/selection and
   VIR unit linkage;
6. "release": exact projection of the selected registry and bundle
   descriptors;
7. "inputs": input-kind profile, captured byte sizes/digests, and
   "input_set_hash";
8. "artifacts": VIR and source-map repeated identity/hash linkage;
9. "canonical_size": complete-root JCS byte limit;
10. "hash": frontend-stage "source_manifest_hash".

Certificate-stage validation uses the same order, requires "vc_hash" at phase
2, and inserts a "vc_linkage" phase between "artifacts" and "canonical_size".
The first failing phase owns the stable code. A later phase cannot replace it.

## 2. Root schema and lifecycle branches

The frontend-stage root has exactly:

| Field | Type and rule |
|---|---|
| "schema" | exactly "mpk.source_manifest.v0" |
| "source_language" | "go" or "rust" |
| "semantic_profile" | exact language pairing |
| "semantic_parameters" | exact VIR profile object |
| "selection" | exact FRONTEND_PROTOCOL_V0.md union |
| "limit_profile" | selected release tuple limit profile ID |
| "release_registry" | ReleaseRegistryIdentity |
| "toolchain" | ToolchainIdentity |
| "frontend" | FrontendIdentity |
| "units" | canonical nonempty ManifestUnit array |
| "target" | TargetIdentity |
| "inputs" | canonical nonempty InputEntry array |
| "input_set_hash" | hash of "inputs" only |
| "vir_hash" | validated VIR hash |
| "source_map_hash" | validated source-map hash |
| "source_manifest_hash" | lifecycle self-hash |

The certificate-stage root has exactly the same fields and order-independent
object semantics plus required "vc_hash". No other branch or nullable form
exists. Presence of "vc_hash" discriminates the serialized union; stage is not
a user-supplied JSON field. Each boundary also supplies the expected lifecycle:
the frontend envelope accepts only the frontend branch, certificate assembly
consumes only that branch and produces the certificate branch, and certificate
import accepts only the certificate branch. A branch unequal to that boundary
context is "SOURCE_MANIFEST_STAGE".

The manifest hash is:

~~~text
SHA256(
  UTF8("MPK-SOURCE-MANIFEST-0.1") || 0x00 ||
  JCS(SourceManifest with only source_manifest_hash removed)
)
~~~

The domain is exactly the 23 ASCII bytes "MPK-SOURCE-MANIFEST-0.1". Only the
root self-hash member is removed. Frontend-stage preimages omit "vc_hash"
because that field is absent. Certificate-stage preimages include "vc_hash".
The compact JCS preimage contains no LF.

The hash can be recomputed from canonical manifest bytes without original
source. Input byte validation is a separate frontend-boundary operation.

## 3. Repeated semantic identities

The only language/profile pairs and SemanticParameters are those in VIR_V0.md
and FRONTEND_PROTOCOL_V0.md. These fields are member-for-member equal across
the request, frontend envelope, VIR, every VIR contract, manifest, VC when
attached, and later policy payloads.

"target" has exactly:

| Field | Rule |
|---|---|
| "id" | exact selected target ID |
| "pointer_width" | 32 or 64 |
| "language_configuration" | exact union from section 7 |

"target.id" and "target.pointer_width" equal
"semantic_parameters.target_id" and "semantic_parameters.pointer_width".
Changing either is a semantic identity change.

"selection" equals the caller-constructed envelope selection. Its function
resolves exactly once in VIR. For Rust, "selection.crate" equals the sole unit
identity and "selection.package" equals that unit name. For Go,
"selection.package" equals the unit containing the selected function.

## 4. Release registry and bundle projections

ReleaseRegistryIdentity has exactly:

~~~json
{"schema":"mpk.release.bundle_registry.v0","id":"mpk.release.registry.v0","registry_sha256":"47f80ab09e8cde24af73ddc198aef254ff1dbd18c1423a2e7e0ebb69f8c787a7"}
~~~

It is member-for-member equal to the validated installed registry root
projection. A frontend cannot replace it with another registry that happens to
select equal bundle bytes.

FrontendIdentity has exactly:

| Field | Rule |
|---|---|
| "bundle_id" | selected frontend descriptor ID |
| "name" | selected descriptor name |
| "version" | selected descriptor version |
| "binary_sha256" | snapshotted main executable digest |
| "subordinate_binaries" | canonical SubordinateIdentity array |

SubordinateIdentity has exactly "name", "version", and "binary_sha256".
The array is strictly increasing by "name" UTF-8 bytes and names are unique.
It is the path-free projection of the selected descriptor's complete
subordinate set. Go v0 has an empty array. Rust v0 has exactly the registered
"rust2vir-driver". The main and subordinate paths, runtime loader, and
inventory paths never enter the manifest.

ToolchainIdentity has exactly:

| Field | Rule |
|---|---|
| "bundle_id" | selected toolchain descriptor ID |
| "distribution_sha256" | descriptor whole-distribution digest |
| "components" | canonical ComponentIdentity array |

ComponentIdentity is selected by "kind":

~~~json
{"kind":"executable","name":"go","release":"go1.25.0","binary_sha256":"306c6ca7407560340797866e077e053627ad409277d1b9da58106fce4cf717cb"}
~~~

The Rust compiler executable branch additionally has required "commit_hash":

~~~json
{"kind":"executable","name":"rustc","release":"1.89.0-nightly","commit_hash":"0000000000000000000000000000000000000000","binary_sha256":"306c6ca7407560340797866e077e053627ad409277d1b9da58106fce4cf717cb"}
~~~

~~~json
{"kind":"content","name":"go-stdlib","release":"go1.25.0","content_sha256":"d0e474c3d4c5604aa8d233021e24e742322ff0cea7cf55c1ce7f2c87dfefa469"}
~~~

Executable components require "binary_sha256" and forbid "content_sha256".
Content components require "content_sha256" and forbid "binary_sha256".
"commit_hash" is required only on the Rust executable component named "rustc"
and forbidden on every other component. For Go, the "go" executable release
equals the release descriptor compiler release. For Rust, "rustc" release and
"commit_hash" equal the descriptor compiler release and "rustc_commit".
Components are nonempty, strictly increasing by "name" UTF-8 bytes, and names
are unique. They equal the complete selected descriptor component projection;
a frontend cannot omit support content, add a local tool, or reinterpret the
whole-distribution digest as a component hash.

"limit_profile" equals both the resolved release tuple and selected frontend
descriptor "limit_profile_id". The manifest does not record execution host,
runtime mount, bundle path, executable path, component inventory, or installation
root; those were validated by the launcher and their identities are committed
through the registry and bundle hashes.

## 5. Units

ManifestUnit has exactly "identity", "name", and "kind".

The array is nonempty, strictly increasing by "identity" UTF-8 bytes, and
identities are unique. It equals the complete VIR unit set:

- "identity" equals VirUnit "id";
- "name" equals VirUnit "name";
- Go "kind" is exactly "package";
- Rust "kind" is exactly "lib".

Rust v0 has one unit. Go may have multiple units under its profile. Cargo opaque
package IDs and raw package-loader records are forbidden.

## 6. Input entries and input-set hash

InputEntry has exactly:

| Field | Rule |
|---|---|
| "kind" | "source", "contract", "build_manifest", or "lockfile" |
| "normalized_path" | portable source-root-relative path |
| "size_bytes" | exact captured regular-file byte count |
| "sha256" | SHA-256 of those exact bytes |

"size_bytes" is an integer from 0 through 4,294,967,296. A source file is
nonempty under both initial language profiles; other kinds may be empty only
when that profile explicitly permits the named file to be empty.

The array is strictly increasing by:

1. "normalized_path" UTF-8 bytes;
2. "kind" UTF-8 bytes.

Normalized paths are unique, so the second key is a defensive total-order
definition rather than permission to record one path twice. Every input is
opened without following links, captured exactly once, and the manifest entry
is computed from that immutable buffer. A captured input not listed, a listed
input not captured, or later reread rejects.

The v0 profile responsibilities are:

- Go records every selected package/module source, every used contract, and all
  module/workspace build-manifest and checksum/lock inputs required by
  GO_VIR_PROFILE_V0.md;
- Rust records the root "Cargo.toml", "Cargo.lock", every used contract, and
  the exact compiled module-closure source set required by RUST_SUBSET_V0.md.

A target repository toolchain file is not a source input. Toolchain and
frontend build provenance is represented by the selected release identities.
An external or compiler-synthetic source file rejects unless the language
profile names it as a compiler builtin wholly covered by toolchain identity and
requires no source-map span.

"input_set_hash" is:

~~~text
SHA256(
  UTF8("MPK-INPUT-SET-0.1") || 0x00 ||
  JCS(inputs)
)
~~~

The domain is exactly the 17 ASCII bytes "MPK-INPUT-SET-0.1". The preimage is
the complete already-sorted InputEntry array only. It excludes the root field
name, manifest, LF, and source bytes. It can be recomputed from the manifest.
Inside validation phase "inputs", the exact language input-kind/name inventory
is checked first, captured size and digest equality second, and
"input_set_hash" last. Thus a stale entry digest is not hidden by the
consequent input-set hash mismatch.

## 7. LanguageConfiguration union

The union is selected jointly by "source_language" and "semantic_profile".
Unknown fields and a branch for the wrong language reject.

### 7.1 GoFixedConfiguration

The exact object is:

~~~json
{
  "kind":"go",
  "compiler":"gc",
  "cgo_enabled":false,
  "go111module":"on",
  "module_mode":"readonly",
  "workspace_mode":"off",
  "tests":false,
  "build_tags":[],
  "environment_profile_id":"mpk.go.frontend_environment.v0",
  "argument_profile_id":"mpk.go.frontend_arguments.v0"
}
~~~

"build_tags" is a sorted unique array and is empty in Go v0. "compiler" is
"gc"; cgo and tests are disabled; module and workspace mode have no ambient
fallback. The two profile IDs equal the selected frontend descriptor and name
the complete closed environment/argv behavior frozen by GO_VIR_PROFILE_V0.md.
GOOS, GOARCH, and pointer width are represented by the containing target and
semantic parameters rather than repeated in this object.

### 7.2 RustCheckedConfiguration

The exact object has:

| Field | Exact value or rule |
|---|---|
| "kind" | "rust" |
| "edition" | "2021" |
| "crate_type" | "lib" |
| "enabled_features" | empty array |
| "prelude" | "std" or "core", derived from accepted no_std form |
| "locked" | true |
| "offline" | true |
| "default_features" | false |
| "overflow_checks" | true |
| "panic" | "abort" |
| "debug_assertions" | false |
| "rustc_opt_level" | 0 |
| "mir_opt_level" | 0 |
| "jobs" | 1 |
| "message_format" | "json" |
| "target_allowlist_id" | "mpk.rust.targets.v0" |
| "environment_profile_id" | "mpk.rust.frontend_environment.v0" |
| "argument_profile_id" | "mpk.rust.frontend_arguments.v0" |
| "cfg" | complete effective rustc cfg string set |

"enabled_features" is present and empty. "cfg" is sorted by UTF-8 bytes,
contains no duplicates, and contains only the complete normalized values
allowed by RUST_SUBSET_V0.md for the selected built-in target. Each cfg string
is 1 through 1,024 printable ASCII bytes, contains no control byte or path, and
uses the exact rustc spelling. Paths, output names, and Cargo/rustc incidental
argv are forbidden.

The environment and argument IDs equal the selected frontend descriptor. The
target allowlist and exact cfg set are independently cross-checked against the
selected toolchain target and final effective compiler session.

## 8. Portable normalized input paths

A path is 1 through 1,024 ASCII bytes, relative, slash-separated, and already
normalized. Each component is 1 through 255 bytes, contains only ASCII letters,
digits, period, underscore, or hyphen, is neither "." nor "..", does not end
in period, and is not a case-insensitive Windows device name. Paths are unique
under both byte equality and ASCII case folding.

Leading or trailing slash, empty component, backslash, colon, NUL/control byte,
URI form, drive/UNC prefix, percent-decoded reinterpretation, and the private
"/mpk/" namespace reject. The grammar intentionally rejects source trees whose
file identities cannot be represented portably; a frontend never sanitizes or
renames an accepted source path.

No other manifest string may contain an absolute machine path, hostname,
timestamp, random ID, source root, workspace, home, temporary, installation,
toolchain, executable, sandbox, or output locator.

## 9. VIR and source-map linkage

At frontend stage:

- manifest language/profile/parameters equal the envelope and VIR;
- "vir_hash" equals recomputed VIR "vir_hash";
- "source_map_hash" equals recomputed source-map hash;
- source-map "source_ir_schema" is VIR "schema";
- source-map "source_ir_hash" is manifest "vir_hash";
- manifest units equal VIR units;
- selection resolves exactly once in those units;
- every source-map SourceOrigin resolves a source-kind input and its immutable
  bytes;
- input-set hash recomputes from all captured inputs.

The manifest itself does not contain VIR or source-map bytes. The three hashes
bind the independently carried values without a cycle.

## 10. Certificate-stage attachment

Final assembly accepts validated canonical frontend-stage manifest bytes, not a
caller-constructed object or manifest hash alone. It:

1. parses and byte-reencodes the frontend-stage bytes and requires equality;
2. recomputes VIR, source-map, input-set, and frontend manifest hashes;
3. recomputes the candidate VC hash;
4. requires VC "input_set_hash", "source_ir_schema", "source_ir_hash",
   "semantic_profile", and "semantic_parameters" to equal the manifest and
   VIR;
5. copies every existing manifest member other than
   "source_manifest_hash" without changing its JSON value;
6. adds only "vc_hash" with the recomputed value;
7. recomputes "source_manifest_hash" over the certificate-stage form.

The resulting hash differs from the frontend-stage hash. The assembler records
both lifecycle hashes in policy evidence using the later POLICY_V1.md names;
it never compares them as if they were the same payload. Attempting to remove,
reorder an ordered array, or mutate any other value is
"SOURCE_MANIFEST_LIFECYCLE_MUTATION". A mismatched VC is
"SOURCE_MANIFEST_VC_LINKAGE".

VC does not contain "source_manifest_hash". It contains "input_set_hash",
"source_ir_schema", "source_ir_hash", semantic profile/parameters, and its own
"vc_hash", so there is no manifest/VC hash cycle. The retired
"source_legacy_hash" name is invalid.

The certificate-stage canonical JSON bytes may be placed unchanged in the
existing opaque certificate SourceManifest payload. Source-free checkers
preserve those bytes under the whole-certificate hash and do not parse this
schema for proof acceptance.

## 11. Limits

The intrinsic manifest limit profile is "mpk.source_manifest.limits.v0".

| Limit ID | Inclusive maximum |
|---|---:|
| "canonical_json_bytes" | 4,194,304 (4 MiB) |
| "json_nesting" | 256 levels |
| "string_bytes" | 1,048,576 per decoded string |
| "normalized_path_bytes" | 1,024 |
| "inputs" | 32,768 for Go; 512 for Rust |
| "units" | 256 |
| "toolchain_components" | 8,192 |
| "frontend_subordinates" | 0 for Go; 1 for Rust |
| "cfg_entries" | 16,384 |

Rust v0 additionally limits snapshot inputs to 512 and compiled sources to 256.
Go v0 uses the exact 32,768-entry ceiling above; its 4 MiB canonical byte limit
normally binds first. A language profile may impose a smaller registered limit
but cannot exceed these schema ceilings. All counters use checked unsigned
arithmetic and are enforced before allocating the complete collection.

"canonical_json_bytes" counts JCS of the complete root including
"source_manifest_hash" and certificate-stage "vc_hash" when present. The
manifest is nested in a separately bounded frontend transport and has no
independent LF. Exactly 4,194,304 bytes is accepted; one byte above rejects.

## 12. Stable shared codes

| Code | Meaning |
|---|---|
| "SOURCE_MANIFEST_JSON_DUPLICATE_KEY" | duplicate object name |
| "SOURCE_MANIFEST_JSON_INVALID" | invalid UTF-8/JSON/number |
| "SOURCE_MANIFEST_SCHEMA" | wrong schema discriminator |
| "SOURCE_MANIFEST_SHAPE" | missing, unknown, or wrong-union field |
| "SOURCE_MANIFEST_STAGE" | wrong "vc_hash" presence for requested stage |
| "SOURCE_MANIFEST_PATH" | nonportable, duplicate, or forbidden path |
| "SOURCE_MANIFEST_ORDER" | noncanonical array order or duplicate ID |
| "SOURCE_MANIFEST_PROFILE" | language/profile/parameters/config mismatch |
| "SOURCE_MANIFEST_SELECTION" | selection or selected-function mismatch |
| "SOURCE_MANIFEST_UNITS" | unit projection mismatch |
| "SOURCE_MANIFEST_RELEASE" | registry/bundle/limit projection mismatch |
| "SOURCE_MANIFEST_INPUT_KIND" | language-profile input-kind mismatch |
| "SOURCE_MANIFEST_INPUT_BYTES" | captured size or digest mismatch |
| "SOURCE_MANIFEST_INPUT_SET_HASH" | input-set hash mismatch |
| "SOURCE_MANIFEST_IR_LINKAGE" | VIR identity/hash mismatch |
| "SOURCE_MANIFEST_SOURCE_MAP_LINKAGE" | source-map identity/hash mismatch |
| "SOURCE_MANIFEST_VC_LINKAGE" | VC field or self-hash mismatch |
| "SOURCE_MANIFEST_LIFECYCLE_MUTATION" | finalization changed more than two hashes |
| "SOURCE_MANIFEST_LIMIT" | count or complete JCS exceeds its limit |
| "SOURCE_MANIFEST_HASH" | lifecycle self-hash mismatch |

## 13. Conformance vectors and ownership

"develop/specs/vectors/source-manifest-v0.json" has schema
"mpk.source_manifest.conformance.v0". Its exact top-level fields are "schema",
"spec_schema", "dependencies", "owner_tests", "fixture_inputs",
"manifest_cases", "configuration_cases", "release_cases", "input_cases",
"lifecycle_cases", "path_cases", "hash_cases", and "limit_cases".

"fixture_inputs" contains exact base64 bytes and their expected kind/path.
The accepted frontend-stage Go manifest names the valid release registry,
VIR, and source-map dependency cases. A test hashes the decoded bytes, not a
host file. A case contains exactly one of "input", "json_text", or
"construction", except a fragment case may use "input_from" to select the
input member of an earlier case. Root and fragment inputs are passed without
implicit repair. A normal construction names an earlier "base", optional JSON
Pointer, and ordered RFC 6901/RFC 6902 add-remove-replace patches. The
"swap_inputs" fixture swaps exactly the named two input indices without
changing any hash.

"attach_vc" construction consumes the canonical bytes of the named
frontend-stage case plus an exact synthetic VC identity record, adds only
"vc_hash", and recomputes the manifest hash. "mutate_during_attach" performs
the named additional patch and must reject before final hash acceptance.
"vc_from" selects the "vc" record embedded in the named earlier lifecycle
construction; "vc_patches" applies only to that record.

"canonical_size" feeds exactly "count" bytes from an already canonical,
semantically valid model root to the isolated complete-root size counter. It
tests the inclusive comparison without checking in a multi-megabyte fixture.
"profile_input_count" similarly feeds exactly "count" accepted entries to the
named language's checked collection counter; semantic construction and the
4 MiB limit are tested independently. These isolated-counter cases carry
"context.validator" naming the counter and do not claim that every combination
of independent maxima can coexist in one manifest.

"normalized_path" starts from "manifest.valid_go_frontend_stage", replaces the
"identity.go" input with an exact-length portable path, updates its fixture and
every source-map link, and recomputes all affected hashes. A configuration case has
exactly "id", language/profile context, one input source, and "expect".
"context.validator" is either "language_configuration_shape", which validates
closed branch shape, scalar grammar, and local order without claiming a
compiler-specific complete cfg set, or "language_configuration_profile", which
also requires the language-profile fixture.
"context.validator = component_identity" applies the release/compiler context
only to one ComponentIdentity fragment and freezes the Rust "commit_hash"
presence rule without requiring an installed Rust registry fixture.
"expect" has "outcome" and, for rejection, exact "phase" and "code"; accepted
hash/lifecycle cases may additionally freeze canonical lengths and digests.

The required root "owner_tests" array is exactly, in order:

1. "crates/mpk-vc/tests/source_manifest.rs";
2. "go-tools/go2vir/bundle_candidate_test.go"; and
3. "rust-tools/rust2vir/tests/frontend_envelope.rs".

The owning tests MUST load every case, reject unknown vector/case fields,
verify unique IDs across all arrays, and prove no case is skipped.
