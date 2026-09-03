# Successor Semantic Profile Registry v1 Specification

Status: normative, frozen, and active as the successor registry schema.
`CSHARP-02-T20` activated revision 2 on 2026-08-30, and `JAVA-03-T10` installed
the append-only revision-3 Go/Rust/C#/Java root on 2026-09-03. Historical
handoff and migration clauses below record the required activation path;
revision-1 and revision-2 roots are not alternate active inputs.

This specification is the `MLANG-01-T02` successor-extension decision. MPK
chooses a closed, hash-pinned semantic-profile registry rather than adding a
new source-language branch to every common tagged union. The registry carries
only immutable identities that select validators compiled into the exact
release binaries. It is not a schema language, executable registry, plugin
system, compatibility layer, or proof input.

## 1. Scope and trust

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
no frontend starts or, after a child starts, that no VIR, source map, manifest,
VC, certificate candidate, policy evidence, or AI request is published from
the invalid context.

The registry, its hashes, release descriptors, frontends, compilers, source,
VIR, VC, policy/evidence documents, and AI output remain untrusted helper data.
Certificate v0, its binary encoding and hash domains, both source-free checker
inputs, checker acceptance, and the four axiom categories do not change.

This specification closes the four shape gaps from
`../docs/mlang-01-go-rust-csharp-gap-audit.md`:

| Gap | Frozen owner in this specification |
| --- | --- |
| `S01` | `ProfileRegistryIdentity`, `SemanticContext`, registry and entry hashes |
| `S02` | entry-owned `selection_schema` and the closed `SelectionEnvelope` |
| `S03` | release-root binding, tuple/descriptor entry hashes, and compiled frontend/release contracts |
| `S04` | fixed policy, evidence, and AI contract bindings |

It does not freeze any C# profile content. Registry revision 1 contains only
the two already implemented Go/Rust profiles. `MLANG-01-T03` may define one
C# entry by creating revision 2 under section 11; it must not alter either
revision-1 entry.

## 2. Closed JSON and transport

Every registry object, entry, contract-binding object, semantic-context
object, parameter envelope, selection envelope, compiled-profile envelope,
and registry-identity object is closed. Unknown or inapplicable fields,
missing required fields, duplicate JSON names, and `null` reject. Strings and
enum values are case-sensitive.
Floating-point numbers and integers outside
`[-9007199254740991, 9007199254740991]` reject. Strings contain only Unicode
scalar values and receive no Unicode normalization. A `Sha256` is exactly 64
lowercase hexadecimal ASCII characters.

The installed registry file is exactly:

```text
JCS(SemanticProfileRegistry) || 0x0a
```

JCS means RFC 8785 under the narrowed numeric and Unicode rules above. The LF
is transport framing and is not part of either self-hash. A BOM, missing LF,
extra LF, CRLF, whitespace, alternate escaping, or noncanonical member order
rejects even when it denotes the same JSON value.

Registry validation uses this first-error phase order:

1. `transport`: byte ceiling, UTF-8, JSON depth, duplicate names, string and
   numeric syntax;
2. `shape`: exact root, entry, and contract-binding fields and schema tags;
3. `scalar`: hashes, IDs, revision, and source-language grammar;
4. `limits`: canonical root bytes and profile count;
5. `order`: profile order plus language/profile uniqueness;
6. `entry_hash`: every entry self-hash;
7. `contract_binding`: every contract ID is compiled into each required
   consumer for the selected entry;
8. `invariant`: language/profile, parameter schema, selection schema, and
   contract relationships;
9. `registry_hash`: root self-hash;
10. `embedded_identity`: exact schema/ID/revision/root-hash equality to the
    identity compiled into the release; and
11. `canonical_transport`: exact JCS-plus-LF bytes.

A later failure cannot replace the primary code from an earlier phase.
Invalid UTF-8, a BOM or other JSON syntax error, duplicate names, a float or
unsafe integer, depth 33, and an oversized byte stream fail in `transport`.
When bytes otherwise decode as one valid semantic JSON value, a missing or
extra LF, CRLF, noncanonical whitespace/member order, or alternate escape
fails only in `canonical_transport` after all intervening semantic phases.

## 3. Identifiers and common limits

`RegistryId`, `SchemaId`, `ProfileId`, and `ContractId` are 1 through 128 ASCII
bytes and match:

```text
[a-z0-9]+([._-][a-z0-9]+)*
```

`SourceLanguageId` is 1 through 64 ASCII bytes under the same grammar. There
is no case folding, alias, Unicode normalization, semantic-version comparison,
or “latest” interpretation.

The intrinsic common limit profile is
`mpk.semantic_profile.registry.limits.v1`. It is selected by this registry
schema and is not a file field or command-line option.

| Limit ID | Inclusive maximum |
| --- | ---: |
| `registry_canonical_bytes` | 524,288 |
| `registry_transport_bytes` | 524,289 |
| `json_nesting` | 32 object/array levels |
| `identifier_bytes` | 128 |
| `source_language_bytes` | 64 |
| `profiles` | 256 |
| `semantic_parameters_canonical_bytes` | 65,536 |
| `selection_canonical_bytes` | 65,536 |
| `compiled_profile_payload_canonical_bytes` | 1,048,576 |
| `revision` | 9,007,199,254,740,991 |

`registry_canonical_bytes` counts the complete root including
`registry_sha256`; `registry_transport_bytes` additionally counts the required
LF. Parameter and selection ceilings count JCS of the complete envelope,
including `schema` and `value`. The compiled-profile ceiling counts the
complete envelope in section 5.4. Each compiled profile contract may impose a
smaller bound but never a larger one. All counters use checked unsigned
arithmetic and reject before an allocation or output would exceed the bound.

Existing VIR, source-map, manifest, VC, policy, and AI collection limits remain
independent. A successor root's new semantic context consumes its existing
canonical artifact-byte budget; it does not enlarge that budget.

## 4. Registry root and hashes

### 4.1 `SemanticProfileRegistry`

The root has exactly:

| Field | Type and rule |
| --- | --- |
| `schema` | exactly `mpk.semantic_profile.registry.v1` |
| `id` | exactly `mpk.semantic_profile.registry.v1` |
| `revision` | integer in `1..revision` limit |
| `profiles` | nonempty canonical `SemanticProfileEntry` array |
| `registry_sha256` | root self-hash below |

`profiles` is strictly increasing by
`(source_language, semantic_profile)` using UTF-8 byte order. A
`semantic_profile` is globally unique, so the same profile ID cannot appear
under two languages. A language may own multiple distinct profiles.

`registry_sha256` is:

```text
SHA256(
  UTF8("MPK-SEMANTIC-PROFILE-REGISTRY-1.0") || 0x00 ||
  JCS(SemanticProfileRegistry with only registry_sha256 removed)
)
```

The domain is exactly the 33 ASCII bytes
`MPK-SEMANTIC-PROFILE-REGISTRY-1.0`. Only the root
`registry_sha256` field is removed.

### 4.2 `SemanticProfileEntry`

An entry has exactly:

| Field | Type and rule |
| --- | --- |
| `schema` | exactly `mpk.semantic_profile.entry.v1` |
| `source_language` | `SourceLanguageId` |
| `semantic_profile` | globally unique `ProfileId` |
| `semantic_parameters_schema` | entry-owned `SchemaId` |
| `selection_schema` | entry-owned `SchemaId` |
| `contracts` | exact `CompiledContracts` object below |
| `entry_sha256` | entry self-hash below |

`CompiledContracts` has exactly these nine required `ContractId` fields in
JSON object form: `ai`, `evidence`, `frontend`, `manifest`, `policy`,
`release`, `source_map`, `vc`, and `vir`. Object member order is not semantic.
The frontend contract owns protocol diagnostics, runner arguments,
environment, resource, and sandbox rules. The release contract owns compiler,
toolchain, runtime, bundle, and tuple rules. No tenth or optional consumer can
be introduced by registry data.

`entry_sha256` is:

```text
SHA256(
  UTF8("MPK-SEMANTIC-PROFILE-ENTRY-1.0") || 0x00 ||
  JCS(SemanticProfileEntry with only entry_sha256 removed)
)
```

The domain is exactly the 30 ASCII bytes
`MPK-SEMANTIC-PROFILE-ENTRY-1.0`. Only `entry_sha256` is removed. The root hash
therefore commits both to every full entry and to every repeated entry hash.

### 4.3 Frozen revision-1 entries

Revision 1 contains exactly these two immutable entries:

| Language/profile | Parameter schema | Selection schema | Entry hash |
| --- | --- | --- | --- |
| `go` / `mpk.go.fixed.v0` | `mpk.semantic_parameters.go_fixed.v0` | `mpk.selection.go_function.v0` | `b10ec338d1f2b3fefc015e4d46c27def43e92ff3d87341624b48c93db951ca96` |
| `rust` / `mpk.rust.checked.v0` | `mpk.semantic_parameters.rust_checked.v0` | `mpk.selection.rust_function.v0` | `1cee9716bb21d07e07b8bc1de59ecaf83437549a4d595039486312260816f057` |

For each row and each contract field `X`, the exact contract ID is
`mpk.profile.X.go_fixed.v0` or `mpk.profile.X.rust_checked.v0`, respectively,
where `X` is the literal field spelling. The complete values, canonical byte
lengths, and domain-separated hashes are frozen in
`vectors/semantic-profile-registry-v1.json`.

The revision-1 root identity is:

```json
{"schema":"mpk.semantic_profile.registry.v1","id":"mpk.semantic_profile.registry.v1","revision":1,"registry_sha256":"7c9163571cda32aa47984e3e6d949c8857bf62f00110dd1b2c3958eed5e537cc"}
```

The complete root is 1,632 canonical bytes and 1,633 transport bytes. Its
JCS-without-root-hash payload is 1,547 bytes; the complete domain-separated
preimage is 1,581 bytes. The SHA-256 of its exact JCS-plus-LF transport is
`e9466a1baf1936dc82289b35eaf993856cc06e4df6bd6e6d43c943e7da5d1d03`.

This revision-1 root is a frozen design/vector baseline, not an installed or
accepted production root. The active production route recognizes the v1
schema but requires the embedded revision-3 identity, so the revision-1 ID/hash
rejects there.

## 5. Semantic context and payload ownership

### 5.1 `ProfileRegistryIdentity`

Every successor semantic context contains one exact registry projection:

```json
{"schema":"mpk.semantic_profile.registry.v1","id":"mpk.semantic_profile.registry.v1","revision":1,"registry_sha256":"7c9163571cda32aa47984e3e6d949c8857bf62f00110dd1b2c3958eed5e537cc"}
```

The four members are copied from the completely validated installed root and
compare member-for-member. A hash without its schema, ID, and revision is not a
registry identity.

### 5.2 `SemanticContext`

`SemanticContext` has exactly:

| Field | Rule |
| --- | --- |
| `profile_registry` | exact `ProfileRegistryIdentity` |
| `profile_entry_sha256` | exact selected entry hash |
| `source_language` | exact selected entry language |
| `semantic_profile` | exact selected entry profile |
| `semantic_parameters` | exact `SemanticParametersEnvelope` |

`SemanticParametersEnvelope` has exactly `schema` and `value`. `schema` equals
the selected entry's `semantic_parameters_schema`. `value` is a JSON object
validated by the corresponding compiled profile contract. The registry does
not describe that object's fields, defaults, aliases, coercions, or limits.

The revision-1 Go and Rust values retain their current exact semantics:

```json
{"schema":"mpk.semantic_parameters.go_fixed.v0","value":{"target_id":"linux/amd64","pointer_width":64}}
```

```json
{"schema":"mpk.semantic_parameters.rust_checked.v0","value":{"target_id":"x86_64-unknown-linux-gnu","pointer_width":64,"overflow_mode":"checked","panic_mode":"abort"}}
```

Changing the registry root, entry hash, language, profile, parameter schema,
or parameter value changes semantic context. No component may infer one member
from another and omit it from a successor artifact.

### 5.3 `SelectionEnvelope`

`SelectionEnvelope` has exactly `schema` and `value`. `schema` equals the
selected entry's `selection_schema`; `value` is a JSON object validated by the
compiled frontend, manifest, release, and policy contracts. Revision-1 Go and
Rust values retain the current selection members and relationships under these
new envelopes:

```json
{"schema":"mpk.selection.go_function.v0","value":{"package":"example.com/mpk/vector","function":"example.com/mpk/vector.Identity"}}
```

```json
{"schema":"mpk.selection.rust_function.v0","value":{"package":"payment-policy","crate":"payment_policy","kind":"lib","function":"payment_policy::approved_reserve_cents"}}
```

An unknown schema, known schema from another entry, invalid value, unknown
member, missing member, or crossed language/profile/selection rejects. A
frontend cannot derive or repair a selection.

### 5.4 `CompiledProfileEnvelope`

Profile-specific data on any other successor surface uses one exact envelope:

| Field | Rule |
| --- | --- |
| `profile_entry_sha256` | exact selected/member entry hash |
| `contract_id` | exact owning field of that entry's `CompiledContracts` |
| `value` | non-null JSON object accepted by that compiled contract |

The envelope is closed and its complete JCS is at most 1,048,576 bytes. Its
`value` is also closed by the finite validator selected through `contract_id`;
the compiled contract owns its exact fields, tagged unions, ordering,
relationships, and any smaller limits. A surface cannot carry an untagged
profile configuration, infer a contract from a filename, or use a contract ID
from another entry or another contract field.

The envelope transports declarative profile data, not validators. A release
contract value may name only inventory-relative records already present in
the surrounding validated descriptor and may never name a validator,
checker, dynamic library to load as validation code, plugin, or external
resource. T03 owns the exact C# values; registry bytes do not contain them.

## 6. Compiled contract closure and no-plugin proof

The registry is accepted only when every contract ID in every installed entry
exists in a finite table compiled into the exact `bin/mpk` bytes. The selected
frontend and any subordinate compiler driver also compile the entry's exact
frontend, VIR, source-map, and manifest contracts. Release descriptors repeat
the supported entry hashes, and their content-addressed executable bytes are
validated before launch.

Contract IDs are opaque equality tokens. They are never interpreted as a
pathname, URI, package/module name, symbol, dynamic-library name, executable,
command, environment variable, regular expression, JSON Schema, schema DSL,
WASM/native bytecode, proof term, checker profile, axiom profile, or callback.
The exact registry and entry shapes have no field capable of carrying any of
those values. In particular, they contain no path, URL, code blob, validator,
checker, plugin, handler, command, argument, environment, library, or import
member.

Every profile, parameter-schema, selection-schema, and contract ID denotes one
immutable normative meaning. A later release MUST NOT compile different
accepted values or behavior behind the same ID. Any validator or semantic
change requires a new ID, a new immutable entry, and the section-11 review;
the release gate proves that each pinned binary implements that exact frozen
contract.

Registry validation performs only data validation, hash recomputation, exact
compiled-ID lookup, and relationship checks. It MUST NOT open a path named by
registry bytes, dynamically link a module, reflect over a type name, evaluate
an expression, fetch a resource, start a process, or install code. A digest is
an identity, not execution authority.

The semantic-profile registry cannot:

- add a VIR type, instruction, safety-check kind, terminator, or VC term;
- reinterpret an existing operation;
- add a Certificate v0 tag, proof node, checker, trust source, or axiom
  category;
- select a raw frontend/compiler/checker executable or bundle path;
- broaden AI output into proof evidence; or
- activate an entry absent from the release tuple set.

A new common operation requires a successor VIR/VC schema and hash-domain
revision before a profile can name it. A new checker or axiom policy follows
its existing governance and is never a registry update.

These structural exclusions, compiled contract lookup, exact release binary
hashes, and fail-closed unknown-ID behavior prove that a valid registry cannot
load executable validator or checker plugins.

## 7. Release binding and activation

The successor installed tree adds exactly one immutable file:

```text
RELEASE_ROOT/share/mpk/semantic-profile-registry.json
```

`share/mpk` then contains exactly that file and
`bundle-registry.json`. The semantic registry is regular, non-executable,
link-count one, mode `0444`, opened descriptor-relative without following
links, and byte-validated under section 2. There is no CLI, environment,
project, adjacent-file, current-directory, or `PATH` registry locator.

`bin/mpk` embeds the exact `ProfileRegistryIdentity`. The successor release
bundle registry repeats that identity once at its root. Each release tuple
contains `profile_entry_sha256` and one exact semantic-parameters envelope;
each selected frontend and toolchain descriptor lists the same supported entry
hash. The release registry hash commits all of those values. The semantic
registry does not point back to the release registry, so the two hashes have
no cycle.

The caller supplies equality assertions for semantic-registry schema, ID,
revision, root hash, and selected entry hash, in addition to the release-
registry and bundle assertions. Structured reproduction recipes use the exact
options:

```text
--profile-registry-id
--profile-registry-revision
--profile-registry-sha256
--profile-entry-sha256
```

They accept values only, never a registry path. Tuple resolution returns the
complete `SemanticContext`; the caller cannot override parameters selected by
the tuple. No first entry, default profile, compatible profile, latest
revision, fallback root, or host-derived target exists.

An entry is active only when all of these are true in one release:

1. its normative language/profile specification and complete vectors exist;
2. its unchanged entry is in the embedded semantic registry;
3. every contract ID is compiled into every required consumer;
4. one release tuple selects its exact entry and parameter value;
5. selected descriptors repeat the entry hash and exact binary/content
   identities; and
6. the release gate proves all profile, cross-profile, determinism, checker,
   and axiom requirements.

Registry membership alone is not activation.

The exact successor release-shape delta is:

- `BundleRegistry` adds one required `profile_registry` equal to the installed
  `ProfileRegistryIdentity`; source-only `BundleCandidate` removes its
  `source_language` and adds the same required identity;
- `FrontendBundle` removes `source_language` and the profile-specific
  limit/environment/argument fields, and adds nonempty `profile_contracts`,
  strictly increasing by `profile_entry_sha256`, with exactly one section-5.4
  envelope per supported entry whose `contract_id` is that entry's
  `frontend` contract;
- `ToolchainBundle` removes `source_language`, `compiler`, `native_runtime`,
  and `target_libraries`, and adds the same closed `profile_contracts` array
  using each entry's `release` contract; common execution-host, component,
  inventory, and content-hash records remain outside the envelopes;
- a `ReleaseTuple` replaces `source_language`, `semantic_profile`,
  `target_id`, and `pointer_width` with one complete `semantic_context`, while
  retaining `limit_profile_id`, `frontend_bundle_id`, and
  `toolchain_bundle_id`; and
- tuple validation requires matching frontend/release envelopes for the
  context entry, and their compiled values resolve the exact limit, target,
  compiler, launcher, managed/native runtime, component, and inventory
  relationships before any process starts.

For managed execution, the surrounding common inventory still identifies all
bytes. A profile release/frontend value may select a pinned native host from
that inventory and pinned managed assemblies/content as arguments or inputs;
it cannot request ambient `.NET`, loader, assembly resolution, restore,
network, `PATH`, or host-global installation. This is the closed descriptor
and runner-dispatch mechanism whose exact C# records remain owned by T03.

## 8. Successor artifact shape and versioning

Every common successor artifact that currently carries or derives a source
language/profile replaces its separate `source_language`, `semantic_profile`,
and `semantic_parameters` root fields with one required `semantic_context`.
Every boundary that currently carries source selection replaces its untagged
Go/Rust union with one required `SelectionEnvelope`. Retaining either old flat
field beside the successor object is an unknown-field error, not a duplicate
compatibility representation.

Any profile-specific release, frontend, manifest configuration, policy,
evidence, or AI registration data not represented by `SemanticContext` or
`SelectionEnvelope` uses section 5.4 and the corresponding entry contract.
An untagged open configuration object is not a successor extension point.

The complete version transition is:

| Surface | Current active identity | Sole successor identity |
| --- | --- | --- |
| semantic registry | none | `mpk.semantic_profile.registry.v1` |
| VIR | `mpk.vir.v0` | `mpk.vir.v1` |
| frontend envelope | `mpk.frontend.cli.v0` | `mpk.frontend.cli.v1` |
| Rust private driver request | `mpk.rust.driver.request.v0` | `mpk.rust.driver.request.v1` |
| Rust private driver result | `mpk.rust.driver.v0` | `mpk.rust.driver.v1` |
| Rust private raw lowering | `mpk.rust.driver.lowering.v0` | `mpk.rust.driver.lowering.v1` |
| Rust private raw source map | `mpk.rust.driver.raw_source_map.v0` | `mpk.rust.driver.raw_source_map.v1` |
| source map | `mpk.source_map.v0` | `mpk.source_map.v1` |
| source manifest | `mpk.source_manifest.v0` | `mpk.source_manifest.v1` |
| release registry | `mpk.release.bundle_registry.v0` | `mpk.release.bundle_registry.v1` |
| release registry ID | `mpk.release.registry.v0` | `mpk.release.registry.v1` |
| frontend descriptor | `mpk.release.frontend_bundle.v0` | `mpk.release.frontend_bundle.v1` |
| toolchain descriptor | `mpk.release.toolchain_bundle.v0` | `mpk.release.toolchain_bundle.v1` |
| source-only bundle candidate | `mpk.release.bundle_candidate.v0` | `mpk.release.bundle_candidate.v1` |
| VC | `mpk.vc.v1` | `mpk.vc.v2` |
| VC skeleton | `mpk.vc.cert_skeleton.v1` | `mpk.vc.cert_skeleton.v2` |
| policy scan | `mpk.policy.scan.v1` | `mpk.policy.scan.v2` |
| policy evidence | `mpk.policy.evidence.v1` | `mpk.policy.evidence.v2` |
| program-certificate assembly profile | `mpk.program_certificate.alpha.v0` | `mpk.program_certificate.alpha.v1` |
| AI API profile | `mpk.ai.api.v1` | `mpk.ai.api.v2` |
| AI sanitized request | `mpk.ai.explain.request.v1` | `mpk.ai.explain.request.v2` |
| AI explanation report | `mpk.ai.explanation.v1` | `mpk.ai.explanation.v2` |

The successor release-registry root's `id` is exactly
`mpk.release.registry.v1`; this is an equality token, not a compatibility
range. `mpk.release.bundle_inventory.v0`, the release limit and Linux probe
profile IDs, the Go/Rust frontend argument/environment profile IDs, the Rust
target/limit/MIR profile IDs, the AI provider response schema
`mpk.ai.explanation.response.v0`, `mpk.vir.limits.v0`,
`mpk.verify.limits.v0`, and the semantic profile IDs
`mpk.go.fixed.v0`/`mpk.rust.checked.v0` remain unchanged because their
meanings and shapes do not change. The successor program-certificate assembly
profile consumes the successor manifest/VC identities but emits the unchanged
Certificate v0 encoding.

The registry identity and entry hash are repeated in:

| Surface | Required binding |
| --- | --- |
| release | root `ProfileRegistryIdentity`; tuple and descriptors repeat entry hash |
| frontend request/response | complete `SemanticContext` and `SelectionEnvelope` |
| VIR and every VIR contract | complete member-equal `SemanticContext` |
| source map | complete `SemanticContext`, source IR schema/hash, profile-owned reference validation |
| source manifest | complete `SemanticContext`, `SelectionEnvelope`, and both registry roots |
| VC and skeleton | complete `SemanticContext` plus source IR/manifest/VC linkage |
| policy scan/evidence/recipes | complete `SemanticContext`, selection, release, manifest, and VC linkage |
| AI API/session/explanation | complete `SemanticContext`; profile contract selected only after validated evidence |

A member mismatch is a linkage failure even when all named hashes exist.
Mixed-profile VIR remains forbidden. Cross-language composition remains only
through checked hash-pinned certificate imports.

## 9. Successor hash domains

Every self-hashed root whose preimage changes receives a new domain. The exact
transition is:

| Hash | Current domain | Successor domain |
| --- | --- | --- |
| contract | `MPK-CONTRACT-0.1` | `MPK-CONTRACT-1.0` |
| VIR | `MPK-VIR-0.1` | `MPK-VIR-1.0` |
| source map | `MPK-SOURCE-MAP-0.1` | `MPK-SOURCE-MAP-1.0` |
| source manifest | `MPK-SOURCE-MANIFEST-0.1` | `MPK-SOURCE-MANIFEST-1.0` |
| release registry | `MPK-BUNDLE-REGISTRY-0.1` | `MPK-BUNDLE-REGISTRY-1.0` |
| Rust driver request | `MPK-RUST-DRIVER-REQUEST-0.1` | `MPK-RUST-DRIVER-REQUEST-1.0` |
| Rust driver success payload | `MPK-RUST-DRIVER-PAYLOAD-0.1` | `MPK-RUST-DRIVER-PAYLOAD-1.0` |
| VC | `MPK-VC-1.0` | `MPK-VC-2.0` |

Each uses `SHA256(UTF8(domain) || 0x00 || JCS(root with only its self-hash
removed))`. The profile entry and profile registry use section 4's domains.

`MPK-INPUT-SET-0.1`, `MPK-RUST-SOURCE-INVENTORY-0.1`,
`MPK-BUNDLE-CONTENT-0.1`, Certificate v0 domains, declaration/interface
domains, checked theory-certificate domains, and axiom-report domains remain
unchanged because their preimages and meanings do not change. Policy and AI
canonical documents have no internal self-hash domain; their existing raw
canonical-byte digests naturally change with their schema and semantic
context.

Adding an unrelated profile changes the registry root and therefore changes
every newly produced helper artifact hash under that release, including Go and
Rust hashes. That change is intentional and must be recorded; consumers MUST
NOT omit the root, substitute only the stable entry hash, or preserve an old
artifact hash through an adapter.

The source manifest remains an opaque length-prefixed Certificate v0 metadata
payload. Embedding canonical `mpk.source_manifest.v1` bytes changes candidate
certificate bytes and their existing Certificate v0 hash as ordinary opaque
metadata, but changes no certificate field, tag, hash domain, checker parser,
or proof-acceptance rule.

## 10. Identity and status precedence

After an enclosing artifact's transport and closed shape validate, semantic
identity validation always uses this order:

1. exact registry schema/ID/revision/root-hash equality to the retained
   installed registry;
2. exact language/profile lookup;
3. exact entry-hash equality and membership;
4. parameter schema then compiled parameter-value validation;
5. selection schema then compiled selection-value validation when present;
6. compiled-profile envelope entry, owning contract field, common size, then
   compiled payload validation when present;
7. contract-specific release/source/VIR/map/manifest/VC/policy/evidence/AI
   linkage;
8. profile operation/subset checks; and
9. enclosing canonical transport and self-hash phases.

The stable registry codes are:

| Code | Owned failure |
| --- | --- |
| `SEMANTIC_REGISTRY_TRANSPORT` | registry transport phase |
| `SEMANTIC_REGISTRY_SHAPE` | registry shape phase, including any plugin-like extra field |
| `SEMANTIC_REGISTRY_SCALAR` | registry scalar phase |
| `SEMANTIC_REGISTRY_LIMIT` | registry/common context limit |
| `SEMANTIC_REGISTRY_ORDER` | order or uniqueness |
| `SEMANTIC_REGISTRY_ENTRY_HASH` | entry self-hash |
| `SEMANTIC_REGISTRY_CONTRACT` | unknown or unavailable compiled contract ID |
| `SEMANTIC_REGISTRY_INVARIANT` | crossed entry-owned relationship |
| `SEMANTIC_REGISTRY_HASH` | root self-hash |
| `SEMANTIC_REGISTRY_CANONICAL` | noncanonical JCS-plus-LF registry transport |
| `SEMANTIC_REGISTRY_ASSERTION` | installed or caller root identity differs from embedded identity |
| `SEMANTIC_PROFILE_UNKNOWN` | no exact language/profile entry |
| `SEMANTIC_PROFILE_ENTRY` | selected entry hash differs or is not a member |
| `SEMANTIC_PARAMETERS_SCHEMA` | parameter envelope schema differs |
| `SEMANTIC_PARAMETERS_INVALID` | compiled parameter contract rejects the value |
| `SEMANTIC_SELECTION_SCHEMA` | selection envelope schema differs |
| `SEMANTIC_SELECTION_INVALID` | compiled selection contract rejects the value |
| `SEMANTIC_PROFILE_ENVELOPE` | compiled-profile envelope is malformed or has an unknown member |
| `SEMANTIC_PROFILE_CONTRACT` | compiled-profile envelope uses the wrong entry contract field or an unavailable contract |
| `SEMANTIC_PROFILE_PAYLOAD` | compiled-profile envelope exceeds its common limit or its compiled contract rejects `value` |
| `SEMANTIC_CONTEXT_LINKAGE` | repeated valid contexts differ across artifacts |

Status mapping is exact:

- malformed caller options, an assertion mismatch, unknown/crossed profile,
  or invalid caller parameter/selection/profile payload is a pre-launch
  configuration error, exit 2 with no child JSON;
- missing, noncanonical, hash-mismatched, unsupported-contract, or otherwise
  invalid installed registry data is an artifact-free release-phase
  `frontend-error`; no child starts;
- a launched child that repeats a different otherwise-valid context emits or
  is normalized to `frontend-error`, never `rejected` or `source-error`;
- policy, evidence, and AI importers return their deterministic invalid-
  artifact result and cannot emit `ready`, `mpk_verified`, or proof evidence;
  and
- language subset, source, or operation status is considered only after the
  complete identity chain succeeds.

Thus an unknown profile cannot be disguised as an unsupported source feature,
and a later source diagnostic cannot hide a registry or context failure.

## 11. Registry revision and profile admission

Within schema v1, revisions are append-only by membership, not by array-tail
position:

1. revision 1 is the exact Go/Rust base in section 4.3;
2. revision `N+1` contains every revision-`N` entry byte-for-byte and may add
   entries only for the one next serial language phase, after which the full
   array is sorted again by section 4.1; for this roadmap each phase adds
   exactly one entry, so revision 2 adds only C# even though `csharp` sorts
   before the retained Go/Rust entries;
3. changing any existing entry requires a new semantic-profile ID and a new
   entry; an old entry may cease to have a release tuple but remains in the
   registry history;
4. changing registry/entry shape, validation, common operation vocabulary, or
   a common hash meaning requires a new registry and affected artifact schema
   version, not another v1 revision; and
5. the runtime accepts only the one schema/ID/revision/hash embedded in its
   release. Revision arithmetic never grants compatibility.

A release may select only a subset of registry entries through release tuples,
which permits fail-closed deactivation without deleting history. An entry for
a future language must not be added before that language's normative profile
and vectors are complete. At the original freeze, T03 owned the first C# entry
and exact revision-2 root while Java and later languages were absent. Revision
3 subsequently appended Java without changing the retained entries; the same
admission rule applies to every later language.

Every registry update is a reviewed release change with an exact predecessor
diff, new root hash, regenerated cross-profile vectors, and all affected helper
artifact hashes. There is no runtime install/update command.

## 12. Atomic migration and no dual IR

The predecessor schemas were required to remain the sole production path until
the C# implementation/release change performed all of these steps atomically:

1. consume without alteration T03's already frozen exact C# entry, revision-2
   root, profile specification, compiled-profile payloads, and vectors;
2. compile every revision-2 contract into `mpk`, `go2vir`, `rust2vir`, the
   Rust subordinate driver, and `csharp2vir` as applicable;
3. migrate every Go/Rust/C# producer and consumer in section 8 to the sole
   successor schema;
4. update the release assembler, installed-tree closure, registry roots,
   bundle descriptors, tuples, recipes, API routes, policy/evidence/AI
   projections, examples, fixtures, and documentation in the same release;
5. regenerate every changed Go/Rust hash and prove all old flat fields and
   schema discriminators reject;
6. install only the revision-2 semantic registry and successor bundle
   registry beside the one successor `bin/mpk`; and
7. pass determinism, cross-profile, no-plugin, Certificate v0, both-checker,
   axiom, and full installed-release gates before publication.

The successor binary MUST NOT accept `mpk.vir.v0` or expose a second import
route, version flag, compatibility parser, adapter, fallback hash domain, or
parallel old release registry. Before activation, the predecessor binary was
likewise required not to accept the then-future `mpk.vir.v1`. Development may
compare separate whole binaries and fixtures, but one binary/release never
accepts both public VIR schemas.

Rollback replaces the whole installed release with the prior release image;
it never mixes old/new registries, binaries, bundles, or artifacts. Existing
canonical certificates remain independently checkable because source helper
schemas do not participate in proof checking. This is not authorization to
feed an old helper artifact into the successor source pipeline.

## 13. Conformance vectors and test ownership

`develop/specs/vectors/semantic-profile-registry-v1.json` has schema
`mpk.semantic_profile.registry.conformance.v1` and exact top-level fields
`schema`, `spec_schemas`, `owner_test`, `fixtures`, `transport_cases`,
`hash_cases`, `registry_cases`, `context_cases`, `profile_envelope_cases`,
`limit_cases`, `hash_domain_migration_cases`, and `migration_cases`.

It freezes:

- the complete revision-1 Go/Rust root and both entry hashes;
- exact raw transport acceptance plus malformed UTF-8/JSON, duplicate-name,
  unsafe-number, depth, framing, escaping, and canonicalization rejection;
- canonical payload, domain-separated preimage, complete-root, and transport
  byte lengths plus exact SHA-256 values;
- shape, scalar, limit, order, entry-hash, compiled-contract, invariant,
  root-hash, and canonical-transport precedence;
- explicit rejection of path, plugin, validator-module, checker, and
  executable fields;
- exact registry/context/entry/parameter/selection lookup precedence,
  including multi-fault cases;
- closed compiled-profile envelopes, owning-contract linkage, and rejection
  of unknown/callback payload members;
- below/at/above cases for every section-3 limit; and
- every changed hash domain plus the complete old-to-successor serialized-
  identity map with rejection in both directions and no dual-input mode.

The sole above-safe-integer `revision` limit datum is encoded as a decimal
string so that the conformance-vector container itself remains strict JSON;
the modeled registry value is still a numeric token and rejects during
transport parsing.

The sole owner is
`crates/mpk-vc/tests/semantic_profile_registry.rs`. That file is a test-only
closed conformance model and hash checker; it exports no library item and is
not linked into `mpk`, a frontend, a checker, or any production parser/emitter.
The repository vector manifest pins the vector's raw bytes.

## 14. Historical T03 handoff and current exit state

`MLANG-01-T03` used this mechanism without changing it. T03 owned only:

- the exact C# language/profile, parameter, selection, nine contract IDs, and
  corresponding compiled-profile payload values;
- the immutable C# entry hash and revision-2 root hash;
- C# semantic, Roslyn/toolchain, source-map, manifest, runner, policy,
  evidence, AI, diagnostic, limit, and rejection content; and
- every C# conformance/hash vector and implementation-test owner.

It preserved both revision-1 entries exactly and left every later language
absent. `CSHARP-02` then completed the atomic implementation in section 12.

At freeze, this design introduced no production parser/emitter, released dual
IR input, C# entry, executable registry field, or checker/trust change. The
active production routes now select only the embedded revision-3 identity and
still have no dual-IR mode, executable registry field, or checker/trust change.
