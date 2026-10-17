# Release v3 metadata and readiness design

This inactive W04 design covers seven [root schemas](schema.json): frontend
bundle, toolchain bundle, bundle registry, outer release registry, candidate,
evidence and receipt. The [model](../../../../probes/go-practical/release_contracts.py)
validates metadata reconstructed from independent frozen sources. It does not
build a frontend, install an image or grant release readiness.

## Inventory, identity and preserved execution contracts

Keep the exact generic `mpk.release.bundle_inventory.v1` shape already reserved
for member data: `schema,members`, with `path,raw_sha256,size_bytes` per member.
Do not add executable bits to that version. New bundle descriptors separately
bind an exact, complete `file_modes` list. Their existing generic
`MPK-BUNDLE-CONTENT-1.0` preimage contains the **whole versioned descriptor except
its own content hash**, including inventory, modes, runtime, contracts and origin.
Frontend/toolchain schemas distinguish those preimages. Rehashing changed
permissions cannot bypass the independently expected byte/mode snapshot.

The root shapes preserve all executable/static/dynamic runtime, interpreter,
library, component/content-inventory, host-profile and native-layout fields
from the pinned existing [release models](../../../../../crates/mpk-vc/src/release_bundle.rs).
Content components retain inventory.v0 and exact file bytes/modes. When the
owning bundle ID changes, rebind the inventory's typed owner scope, recompute
its existing `MPK-BUNDLE-CONTENT-0.1` hash and update every target-library ref.
Historical old-scope hashes remain unchanged at their original source epoch.
The [initial scope review](verification/initial-component-scope/diagnostic.json)
preserves the first projection before this correction. New descriptor IDs and
hashes are distinct from old immutable identities. Metadata is never evidence
that old executable bytes support the new protocol or profile.

The fixture [graph](graph.json) projects four registered frontend descriptors,
four toolchains and five target tuples into new metadata. Every projection
has `legacy_metadata_only` origin. The new resolver rejects them until actual
materialization and protocol/ABI conformance are observed. It does not reuse
GoFixed as practical Go. The six required contexts include practical Go, whose
tuple is absent and explicitly missing. Toolchain content may be reused only
after complete independent byte/mode/target/runtime validation; frontend and
consumer builds must actually implement the new family.

Bundle/release registries use their distinct reserved v3 domains. The bundle
registry's `id` remains `mpk.release.registry.v3`; the separate outer release
root uses that string as its schema and identity. Frontend/source-artifact
provenance continues to reference the **bundle registry** schema/hash, never
the outer image manifest. Candidate/evidence use raw whole-canonical refs;
receipt uses `MPK-RELEASE-RECEIPT-3.0`. Every own-hash preimage includes the
schema and every other required field. Canonical release transports have one LF;
all canonical/domain preimages exclude it.

Keep existing release limits: 64MiB canonical bytes plus LF, 256 container
levels, 1MiB decoded strings, 1,024 descriptors, 4,096 tuples, 256 host/layout
profiles, 8,192 components, 262,144 serialized file records/unique bundle files,
1,024-byte portable paths, 4GiB per file and 32GiB declared bytes per bundle.
Count members, modes and nested component inventory records in the serialized
record budget. All portable-path/order/duplicate/prefix/type/range/runtime
constraints remain independently checked. Input byte bounds apply before
parsing. The Go execution-host/resource contract must join the remaining W04
budget closure rather than borrowing a weaker old host or inventing a pass.

## Acyclic build and image graph

Use this order: semantic registry/foundation and frozen build inputs -> built
frontends/toolchains -> bundle registry -> consumer compilation -> outer
release registry -> candidate -> observed evidence -> receipt -> activation.
The consumer embeds exact semantic-registry, bundle-registry and schema-set
references. It does **not** embed its own outer release hash. Frontends cannot
embed a bundle-registry hash that includes their own executable hash.

The outer registry binds the consumer binary/build/checker refs, compiled
registry/schema refs and complete image inventory. Required null consumer and
inventory fields represent this incomplete candidate only; release readiness
requires their complete materialized objects. Its own manifest file is omitted
from its own file inventory to avoid self hashing. Bundle inventories and
consumer/layout metadata bind every other installed file and required mode.
One image contains bin/mpk, the two registry transports and all registered
bundle trees; additional common-family metadata has explicit owned paths.

The earlier [linkage fixture](../capture-artifacts/README.md) with
`complete_bundle_registry:false` remains a historical external-reference
experiment. Its hash does not identify this full registry and rejects at the
new binding boundary. T02 must regenerate frontend requests, source manifests,
source/VC/certificate hashes and downstream evidence against a real new
registry. Do not relabel old fixture hashes or rewrite old observations to
claim registered provenance.

## Gate evidence and atomic activation

Candidate state is derived from exact required contexts, actual bundles,
consumer and separately observed inputs. Eight mandatory gate families cover
source-artifact migration, two-pass build/materialization, paired-checker
application corpus, four-profile preservation, practical-Go application
coverage, two installed replays, native Linux resource/isolation faults, and
atomic activation/rollback. Each observation binds run/command/source epoch,
release/image/IO inventories, exit status, raw stdout/stderr and host probe.
Missing evidence is `not_run`; runtime/host failure is `environment_error`.
Neither is a false source theorem or a successful release check.

`release_verified` requires complete context coverage, actual independently
captured byte/mode handles, reproducible origins, two matching complete native
gate receipts, and no missing, failed or environment-error checks. Regenerate
all evidence/receipt fields from local runner authority. Caller JSON, CI/AI
text, valid hashes and passed flags cannot certify that a command ran. The
current model has no such authority; even its valid incomplete receipt cannot
authorize activation. `proof_authority` remains `none`: an installed helper
release is not a theorem or arbitrary application/package proof.

Production gate commands retain the root README's local native Linux checks,
including the successor bundle build/replay checks, `sudo ./scripts/check-java-frontend.sh`
and `sudo ./scripts/check-all.sh`. No workflow is created or used. Current local
Linux emulation is metadata/hash evidence, not native release acceptance.

Prepare a complete immutable owner-controlled image. Validate descriptor-
relative handles from one image and compare the expected previous image and
activation generation. Fsync prepared data before one atomic image-pointer
commit. Concurrent/stale writers reject; every failure leaves the pointer
unchanged. Existing open handles retain the old complete image. Rollback
selects the whole previous image under a new generation, never old registries
with new binaries or a mixture of toolchains. No caller path, environment,
staging-tree or predecessor-schema override becomes a resolver fallback.
The activation generation/journal is local installation state outside the
self-hashed image; its ownership and compare-and-swap implementation remain
T07 native validation obligations.

## Evidence and owners

The [vectors](vectors.json) cover required/unknown fields, reserved-family
crossing, hashes, rehashed readiness promotion and canonical framing. The
[native observations](native-legacy.json) run the unchanged actual release
validator and selector over all four languages/five targets, plus wrong-schema,
unknown-field and wrong-hash controls. They run twice. Thirteen complete
versioned preimages agree with independent [Rust](rust-hashes.json) and
[Go](go-hashes.json) canonical/domain hashes, each repeated twice. Chunked
observer transport and compact results avoid placing a multi-megabyte hex
string inside a public JSON string limit. The public preimages remain exact.

Setup findings preserve the original read-only lockfile error, missing bootstrap
target and stale overwritten dummy executable. Bootstrap outputs now use a
separate cache, and the actual observer is rebuilt/queried before acceptance.
These are construction failures, not source/proof/release verdicts. The
[verification](verification/verification.json) keeps exact source snapshots,
targeted test reasons and logs. No installed registry or old helper changes.

T02 owns private release/source-contract linkage. T07 owns actual new-family
builders, resolver, byte-handle materialization, host/resource enforcement,
two-pass native gates and atomic activation/rollback. The root README's gates
remain mandatory at T07/T11. W04 still needs ordinary-program/materialized-policy
signatures, exact Go host/budget closure and exhaustive consumer/axiom-policy/
historical-source-epoch review. W05 owns inactive publication and the cumulative
T01 gate. The [current metadata checker](../../../../probes/go-practical/check_release_metadata.py)
retains the preceding AI evidence and 143 inventoried consumers. The full
T02–T11 implementation/proof/release objective remains active.
