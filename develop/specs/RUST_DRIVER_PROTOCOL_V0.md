# Rust Private Driver Protocol v0 Specification

Status: normative and frozen for implementation.

This specification defines the only private boundary between `rust2vir` and
`rust2vir-driver`. The request schema is `mpk.rust.driver.request.v0`; the
status-tagged result schema is `mpk.rust.driver.v0`. Neither schema is a public
frontend envelope, VIR, source manifest, source map, VC, certificate, or proof
input. A private object MUST NOT be accepted at any public artifact boundary.

## 1. Conformance and validation order

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. Every object and
tagged branch is closed. Unknown or inapplicable fields, missing required
fields, duplicate JSON names, null, floating-point numbers, and integers
outside `[-9007199254740991, 9007199254740991]` reject. Strings and enum values
are case-sensitive. A Sha256 is exactly 64 lowercase hexadecimal characters.

Both transports are UTF-8 compact RFC 8785 JCS followed by exactly one byte
`0x0a`. A BOM, CRLF, leading or trailing whitespace, a second LF, or bytes after
the LF reject. The LF counts toward the serialized size limit but is excluded
from every hash preimage. Strict parsing detects duplicate names before a JSON
map can discard them, then JCS re-encoding MUST reproduce every byte before the
LF.

The main and driver use this first-error order:

1. streaming byte limit, checked-counter overflow, regular-file/no-follow
   state, and the exact fixed locator;
2. UTF-8, JSON depth/number/string limits, duplicate names, JCS, and one LF;
3. closed branch shape and schema/status discriminant;
4. scalar grammar, array order, uniqueness, and collection limits;
5. request fingerprint and source-inventory hash recomputation;
6. repeated request, binary, registry, toolchain, compiler, profile, target,
   selection, input, and invocation identity;
7. status/phase/diagnostic invariants;
8. success-only inventory, raw lowering, raw source-map, and payload hash;
9. cross-process comparison and filesystem publication invariants.

The earliest started step owns the failure. A private parse, transport,
identity, process, or invariant failure is a public `frontend-error` with the
appropriate `RUST_FRONTEND_DRIVER_PROTOCOL_*` code; it is never reported as a
source refusal. A valid driver status is translated only after every earlier
private check completes.

## 2. Shared closed values

The following values are projections of already validated objects. The private
protocol does not introduce alternate spellings or defaults.

### 2.1 Semantic and selection values

`source_language` is exactly `rust`. `semantic_profile` is exactly
`mpk.rust.checked.v0`. `semantic_parameters` has exactly `target_id`,
`pointer_width`, `overflow_mode`, and `panic_mode`. `target_id` is one member of
`mpk.rust.targets.v0`; `pointer_width` is that target's exact 32- or 64-bit
value; `overflow_mode` and `panic_mode` are exactly `checked` and `abort`. No
target is implicit. The vector fixture's x86_64 instance is:

~~~json
{"target_id":"x86_64-unknown-linux-gnu","pointer_width":64,"overflow_mode":"checked","panic_mode":"abort"}
~~~

`selection` has exactly `package`, `crate`, `kind`, and `function`. Their values
obey `FRONTEND_PROTOCOL_V0.md`; `kind` is literally `lib`, the crate is the
first function segment, and all four members equal the caller selection. The
vector fixture's instance is:

~~~json
{"package":"vector","crate":"vector","kind":"lib","function":"vector::identity"}
~~~

### 2.2 Release identities

`release_registry` is exactly the `ReleaseRegistryIdentity` from
`SOURCE_MANIFEST_V0.md`. `frontend` is exactly the corresponding
`FrontendIdentity` and therefore contains the expected main digest plus exactly
one sorted subordinate named `rust2vir-driver`. `toolchain` is exactly the
corresponding `ToolchainIdentity`. These are path-free projections of the
launcher-validated registry and snapshotted files.

`compiler` has exactly:

| Field | Exact rule |
|---|---|
| `name` | `rustc` |
| `release` | `1.89.0-nightly` |
| `commit_hash` | `4d08223c054cf5a56d9761ca925fd46ffebe7115` |
| `binary_sha256` | equals toolchain executable component `rustc` |
| `target` | equals `semantic_parameters.target_id` |

The frontend subordinate entry named `rust2vir-driver` is the expected driver
binary identity; no separate caller-selected driver object or path exists.

### 2.3 Inputs and normalized source inventory

`inputs` is the complete canonical nonempty `InputEntry` array from
`SOURCE_MANIFEST_V0.md`, before public manifest construction. It contains the
captured `source`, `contract`, `build_manifest`, and `lockfile` identities for
this invocation and is strictly sorted by that specification. `input_set_hash`
is its exact `MPK-INPUT-SET-0.1` hash.

`source_inventory` is a nonempty array containing exactly one projection for
each and only each `inputs` entry whose `kind` is `source`. A projection has
exactly `normalized_path`, `size_bytes`, and `sha256`. It retains input order,
is strictly increasing by normalized-path UTF-8 bytes, and contains no
duplicate or case-fold collision. Every length/hash equals the immutable input
buffer and every source input appears exactly once.

The inventory hash is:

~~~text
SHA256(
  UTF8("MPK-RUST-SOURCE-INVENTORY-0.1") || 0x00 ||
  JCS(source_inventory)
)
~~~

The JCS operand is the complete array, not a containing object. It has no LF.

## 3. Canonical request

The request root has exactly these fields:

| Field | Rule |
|---|---|
| `schema` | `mpk.rust.driver.request.v0` |
| `source_language` | shared exact value |
| `semantic_profile` | shared exact value |
| `semantic_parameters` | shared exact value |
| `selection` | shared exact value |
| `limit_profile` | `mpk.rust.limits.v0` |
| `target_allowlist_id` | `mpk.rust.targets.v0` |
| `environment_profile_id` | `mpk.rust.frontend_environment.v0` |
| `argument_profile_id` | `mpk.rust.frontend_arguments.v0` |
| `mir_profile_id` | `mpk.rust.mir.4d08223c.v0` |
| `release_registry` | shared validated projection |
| `frontend` | shared validated projection |
| `toolchain` | shared validated projection |
| `compiler` | shared exact object |
| `inputs` | complete canonical input array |
| `input_set_hash` | exact input-set hash |
| `source_inventory` | complete normalized source inventory |
| `source_inventory_hash` | recomputed inventory hash |
| `request_fingerprint` | self-fingerprint below |

The request fingerprint is:

~~~text
SHA256(
  UTF8("MPK-RUST-DRIVER-REQUEST-0.1") || 0x00 ||
  JCS(DriverRequest with only request_fingerprint removed)
)
~~~

Only the root `request_fingerprint` member is removed. The preimage includes
the inventory body and hash, input body and hash, all profile IDs, target,
selection, registry identity/hash, main and driver digests, toolchain
distribution/components, and compiler identity. It contains no transport LF.

The request MUST NOT contain an executable, installation, source-root,
snapshot, manifest locator, target-directory, output, home, temporary, sysroot,
runtime, loader, cache, or other runtime path. Normalized input paths inside
`inputs` and `source_inventory` are the sole path-like values. No environment
variable, stdin byte, inherited argument, or host locator supplements it.

The complete request transport maximum is 4,194,304 bytes including its LF.
The JSON portion may therefore be at most 4,194,303 bytes. The exact boundary
is accepted. Boundary plus one or checked byte-counter overflow rejects before
allocation, produces no driver artifact, and is classified locally by the main
as `RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT`.

## 4. Status-tagged driver output

Every output branch has these exact common fields:

| Field | Rule |
|---|---|
| `schema` | `mpk.rust.driver.v0` |
| `status` | branch tag below |
| `phase` | exact branch-compatible child phase |
| `source_language` | repeats request |
| `semantic_profile` | repeats request |
| `semantic_parameters` | repeats request member-for-member |
| `selection` | repeats request member-for-member |
| `limit_profile` | repeats request |
| `target_allowlist_id` | repeats request |
| `environment_profile_id` | repeats request |
| `argument_profile_id` | repeats request |
| `mir_profile_id` | repeats request |
| `release_registry` | repeats request member-for-member |
| `frontend` | repeats request member-for-member, including driver digest |
| `toolchain` | repeats request member-for-member |
| `compiler` | repeats request member-for-member |
| `input_set_hash` | repeats request |
| `source_inventory_hash` | repeats request |
| `request_fingerprint` | repeats request |
| `diagnostics` | normalized private Issue array |

There is no catch-all status. Every repeated value is compared with the
main's independently retained expected value, not merely with another field in
the untrusted output.

### 4.1 `lowered`

Success has `status = "lowered"`, `phase = "lowering"`, and the common fields
plus exactly `source_inventory`, `raw_lowering`, `raw_source_map`, and
`payload_hash`. Exit status is 0. Diagnostics may contain normalized nonfatal
items.

`source_inventory` equals the request array byte-for-byte after JCS encoding
and hashes to the repeated `source_inventory_hash`.

`raw_lowering` is a closed object with exactly these members:

| Member | Exact value or rule |
|---|---|
| `schema` | `mpk.rust.driver.lowering.v0` |
| `mir_profile_id` | `mpk.rust.mir.4d08223c.v0` |
| `vir` | one complete closed `mpk.vir.v0` module, including its `vir_hash` |

The exact valid instance is the `valid_lowered.value.raw_lowering` object in
`rust-driver-v0.json`; no shortened or schematic VIR object is valid. It is
still untrusted raw driver data. The main validates its full schema,
recomputes the hash, verifies the Rust profile, selection, contracts, and
operation/check ordering, and only then may use it in a public envelope.

`raw_source_map` has exactly `schema`, `source_ir_schema`, `source_ir_hash`, and
`entries`. `schema` is `mpk.rust.driver.raw_source_map.v0`;
`source_ir_schema` is `mpk.vir.v0`; `source_ir_hash` equals the enclosed VIR
hash. `entries` uses exactly the closed reference and origin unions and
canonical reference order in `SOURCE_MAP_V0.md`, but the object intentionally
has no public `source_map_hash`. A source origin already has literal
`input_kind = "source"`; its normalized path/range is validated against the
main's immutable input buffer. A synthetic origin uses only the Rust profile's
closed reason set. Compiler filenames, source snippets, line/column pairs,
expansion spans, and internal IDs are forbidden.

The success payload hash is:

~~~text
SHA256(
  UTF8("MPK-RUST-DRIVER-PAYLOAD-0.1") || 0x00 ||
  JCS(complete lowered output with only payload_hash removed)
)
~~~

Only the root `payload_hash` is removed; all common/repeated values,
diagnostics, source inventory, raw lowering, and raw source map remain. The
preimage has no LF. A payload hash is forbidden on every non-success branch.

### 4.2 `rejected`

`status` is `rejected`; `phase` is one of `capture`, `source`, `metadata`,
`subset`, or `lowering`; exit status is 3; and `diagnostics` is nonempty. It
represents a closed-profile refusal or deterministic source/MIR limit.

### 4.3 `source-error`

`status` is `source-error`; `phase` is one of `capture`, `source`, `metadata`,
or `typecheck`; exit status is 4; and `diagnostics` is nonempty. It represents
malformed, unresolved, ill-typed, or borrow-invalid Rust input.

### 4.4 `frontend-error`

`status` is `frontend-error`; `phase` is any started child phase from
`capture` through `lowering`; exit status is 1; and `diagnostics` is nonempty.
It represents an operational, compiler, toolchain, protocol, or internal
invariant failure the driver can still report.

Each non-success branch consists of the common fields only. In particular it
MUST NOT contain `source_inventory`, an inventory body under another name,
`raw_lowering`, VIR, VIR hash, raw or public source map, source manifest,
`payload_hash`, or any partial artifact. Its repeated `source_inventory_hash`
is recomputed from the request inventory retained by both processes, not from
an output inventory body.

The complete output transport maximum is 268,435,456 bytes including LF. The
JSON portion may therefore be at most 268,435,455 bytes. Exact boundary accepts;
boundary plus one rejects while streaming and the main returns no partial
public artifact.

## 5. Private diagnostics and precedence

`diagnostics` entries use the exact closed Issue shape, normalization, source
span, sort order, 4,096-byte message limit, 1,024-entry aggregate limit, 2 MiB
message budget, and truncation algorithm in `FRONTEND_PROTOCOL_V0.md`. A
private diagnostic uses only registered `RUST_*` families from
`RUST_SUBSET_V0.md`. Raw Cargo/rustc stderr, snippets, rendered macro text,
argv, environment, absolute paths, compiler-local IDs, and host suggestions
are forbidden.

Within a completed phase, this is the first-error code precedence:

1. `RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT`, `_SHAPE`, `_CANONICAL`,
   `_HASH`, `_IDENTITY`, `_FILESYSTEM`, `_PROCESS`, `_COUNT`, `_OUTPUT_LIMIT`;
2. `RUST_PREFLIGHT_*` in the table order of `RUST_SUBSET_V0.md`;
3. `RUST_SOURCE_*` in source/name/type/borrow order;
4. `RUST_SUBSET_*`, then `RUST_CONTRACT_*`;
5. `RUST_TOOLCHAIN_*`;
6. `RUST_MIR_*`, then `RUST_SEMANTICS_*`;
7. `RUST_FRONTEND_SOURCE_*` and other internal codes.

An operational error in a started step takes precedence over semantic work
that did not complete. A parser failure prevents source-gate findings inferred
from an incomplete tree; a compiler/query adapter failure prevents a claim
that MIR was semantically rejected. Collectable same-class Issues are sorted
normally and never mixed with a later phase.

## 6. Fixed files and atomic handshake

The sole request locator is the read-only regular file
`/mpk/driver-request.json`. `rust2vir` creates its bytes privately with
no-follow/exclusive semantics, completes and validates the JCS+LF file, seals
it read-only, and only then exposes it at that exact mount. The driver opens it
once no-follow, streams it into an immutable buffer while hashing, validates
file identity before/after the read, and never reopens it. Missing, mutable,
linked, short, replaced, noncanonical, or oversized request bytes cause no
output artifact. A missing locator, wrong file type/mode, link, replacement, or
identity drift is `RUST_FRONTEND_DRIVER_PROTOCOL_FILESYSTEM`; a stable regular
file whose framing is absent/CRLF/multiple-LF or whose streaming byte counter
overflows/exceeds the request limit is
`RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT`; validly framed bytes that fail
strict JSON/JCS equality are `RUST_FRONTEND_DRIVER_PROTOCOL_CANONICAL` or
`_SHAPE` according to sections 1 and 3. These classifications are fixed before
the child can publish an artifact.

The fresh private directory `/mpk/driver-output` is writable only by this
invocation and starts empty. Only a primary invocation may:

1. exclusive-create a regular no-follow mode-0600
   `/mpk/driver-output/result.json.partial`;
2. stream one bounded JCS+LF result, flush and `fsync` that file;
3. revalidate the still-empty final name and partial file identity;
4. atomically rename without replacement to
   `/mpk/driver-output/result.json`; and
5. `fsync` the output directory.

After Cargo exits, the main opens the retained directory handle and requires
exactly one regular no-follow `result.json`, no partial, and no other entry. It
streams the final once into an immutable buffer, checks stable identity, and
parses only that buffer. A missing result, remaining partial, unexpected entry,
second writer, replacement, hard-link alias, symlink, device, FIFO, socket,
short read, or size/hash drift is `RUST_FRONTEND_DRIVER_PROTOCOL_FILESYSTEM`.
It never chooses by enumeration order and never reuses output from another run.
After a stable final file has been accepted, a byte-counter overflow or output
transport above 268,435,456 bytes is
`RUST_FRONTEND_DRIVER_PROTOCOL_OUTPUT_LIMIT`; an absent/CRLF/multiple final LF
is `RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT`; invalid UTF-8/JSON, duplicate
names, or JCS byte disagreement is `RUST_FRONTEND_DRIVER_PROTOCOL_CANONICAL`;
and a closed-branch field disagreement is `_SHAPE`. A second exclusive-create
attempt is a filesystem collision even when the first writer later publishes
valid bytes.

If rustc fails or is killed without a complete artifact, the main produces a
local public `frontend-error`; it does not ask the driver to synthesize a stale
result. Zero or multiple primary matches are
`RUST_FRONTEND_DRIVER_PROTOCOL_COUNT`.

## 7. Wrapper invocation classification

Before classification the wrapper validates and re-encodes the request and
compares its embedded binary/compiler/toolchain identities. The wrapper argv
grammar and semantic options are the exact allowlist in `RUST_SUBSET_V0.md`.

An exact allowlisted Cargo probe (`rustc -vV`, `rustc --print sysroot`, or the
frozen Cargo crate-information probe) delegates to the already selected rustc,
preserves exact argv/exit, uses bounded stdout/stderr, and emits no artifact.
Every explicitly allowlisted non-primary invocation likewise emits none. The
sole primary match is the selected package's library crate, crate type `lib`,
edition 2021, captured crate root, exact target, manifest/input identity, and
effective semantic option profile. It alone runs `rustc_driver` and may publish
one result. An unknown probe, response file, unknown option, injected cfg/codegen
setting, unselected target, or other non-primary compiler invocation rejects;
it is not passed through.

The final rustc session MUST equal the request after Cargo composition:
target, pointer width/cfg set, edition, panic abort, overflow checks enabled,
debug assertions disabled, MIR optimization level zero, no features, remapped
input prefix, loader path, and every allowlisted argument. Merely observing the
desired flag is insufficient if another effective setting conflicts.

## 8. Cross-process responsibilities

| Boundary owner | Must construct or validate independently | Must never trust from child |
|---|---|---|
| generic runner | installed registry, tuple, main/driver/toolchain snapshots, sandbox, input capture, caller selection | private request/result, public hashes |
| `rust2vir` main | Cargo preflight, module closure, immutable inputs, all three private hashes, fixed files, repeated identities, valid VIR/map/manifest and public envelope | driver status, compiler identity, source spans, raw lowering |
| Cargo | locked offline selected lib compilation under fixed argv/env | selection/profile authority, output locator |
| wrapper/driver | canonical request, invocation grammar, embedded commit, FileLoader reads, source/HIR/MIR subset and contracts, one result publication | ambient paths/env, Cargo's claim that options are safe |
| public manifest builder | exact captured inputs/input-set hash, registry/frontend/toolchain projections, VIR/map linkage | a private inventory body without byte comparison |
| VIR validator | complete closed VIR, profile, hashes, contracts, safety checks | rustc/MIR provenance as proof |
| source-map validator | total references, immutable source ranges/boundaries, public map hash | compiler filenames, expansion spans, nearest spans |

Every comparison is member-for-member and hash-and-bytes where raw captured
bytes exist. Equality between two attacker-controlled private fields is never
sufficient. The main creates no public object until request, output, inventory,
payload, VIR, and raw-map checks all complete.

## 9. Conformance vectors and stable errors

`develop/specs/vectors/rust-driver-v0.json`, schema
`mpk.rust.driver.conformance.v0`, owns exact request/success bytes and hashes,
all non-success branches, filesystem state cases, identity/compiler changes,
hash-domain mutations, raw-map coverage, and serialized limit boundaries.

The private protocol code set is closed:

- `RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT`
- `RUST_FRONTEND_DRIVER_PROTOCOL_SHAPE`
- `RUST_FRONTEND_DRIVER_PROTOCOL_CANONICAL`
- `RUST_FRONTEND_DRIVER_PROTOCOL_HASH`
- `RUST_FRONTEND_DRIVER_PROTOCOL_IDENTITY`
- `RUST_FRONTEND_DRIVER_PROTOCOL_FILESYSTEM`
- `RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS`
- `RUST_FRONTEND_DRIVER_PROTOCOL_COUNT`
- `RUST_FRONTEND_DRIVER_PROTOCOL_OUTPUT_LIMIT`

An unknown private protocol suffix rejects as
`RUST_FRONTEND_DRIVER_PROTOCOL_SHAPE`; it cannot extend v0. Implementations
MUST execute every vector and compare exact outcome, phase, code, transport
length/hash, field presence, repeated identity, and filesystem result.
