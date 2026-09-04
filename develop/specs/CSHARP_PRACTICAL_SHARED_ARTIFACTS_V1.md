# C# Practical Successor Shared Artifacts v1

Status: normative, frozen, and inactive. Published by
`CSHARP-03-T01-W10` on 2026-09-04. This specification defines the one successor
shared-artifact family required by `mpk.csharp.practical.v1`; it does not
install that family.

## 1. Authority and exact inventory

The exhaustive name, hash-domain, shape, producer, consumer, migration, and
rollback inventory is the `frozen_contract` and `name_owner_inventory` in
`develop/specs/vectors/csharp-practical-profile-v1.json`, schema
`mpk.csharp.practical.profile.conformance.v1`. Its primary specification test
owner is
`crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10`.

The package contains 17 identity families, 102 globally unique successor
identities, 42 globally unique successor hash domains, 88 retained identities,
11 retained hash domains, 15 strict root schemas, 20 strict nested records,
three closed tagged unions, and the 33-variant contract-expression union. The
machine rows, not abbreviations in this prose, are the exact inventory. No
alias, alternate spelling, implicit version, or unlisted tag/member exists.

Certificate v0, its declaration/term/proof/module/certificate domains, its
empty-proof-node/empty-theory-certificate rules, and both checker acceptance
rules remain unchanged. Retained input-set and Rust source-inventory domains
also remain unchanged because their preimages do. Every changed preimage or
meaning uses the successor identity and domain in the inventory.

## 2. Required successor family

The migration contains all of these inseparable families:

| Family | Principal successor surface |
| --- | --- |
| Semantic registry/context | registry v2, entry v2, limits v2, context v2, validated request v2 |
| Practical parameters/selection | C# practical parameters v1 and selected-members v1 |
| Profile contracts | method/type/expression, operations/checks/limits, bindings, canonical boundary, and transition v1 roots |
| Foundation | registered descriptor/definitions/expansion, semantic bindings, and closed instances v1 |
| Source artifacts/VIR | source-artifacts v2 and VIR v2 |
| Frontend protocol | CLI, request, success, and diagnostic v2 |
| Mapping/manifests | source-map v2 and frontend/certificate manifests v2 |
| Verification | VC v3, certificate skeleton v3, and ordinary-context program assembly v2 |
| Release/policy | bundle/release successor roots, evidence/receipt roots, policy v3 roots |
| AI/API | explanation and API v3 roots |

The exact identity and hash-domain list for each row is
`frozen_contract.identity_families`. Implementations MUST consume those names
verbatim. A parser MUST reject an unknown or later version. A producer MUST NOT
write a retained identity with a successor meaning or a successor identity
with a retained preimage.

## 3. Strict data model

Every root in `frozen_contract.schemas` is a strict JSON object. Its exact
member order is `ordered_fields`; every member is required; `optional_fields`
is empty; the value type is given by `field_types`; unknown members reject;
duplicate keys reject before object construction; and later versions reject.
If a root is self-hashed, its hash field is last and its preimage is exactly
`hash_preimage_fields` encoded canonically under the listed domain.

The same closed rules apply to each nested record in
`schema_type_system.nested_records`, each tagged union in
`schema_type_system.tagged_unions`, and every expression arm in
`expression_union.variants`. A tagged value has one recognized tag and exactly
the fields declared for that arm. Unknown tags, inactive payloads, omitted
required payloads, extra members, duplicate keys, or incorrect field types
reject before downstream artifact construction.

Hash preimages are ASCII domain bytes, one zero byte, then canonical JSON bytes
without a trailing newline. Hashes are lowercase SHA-256. Implementations MUST
hash the complete typed preimage and independently recompute every referenced
hash; a caller-provided hash does not establish validity.

## 4. Semantic context and frontend linkage

`profile_entry_sha256` resolves exactly one entry in one referenced immutable
registry revision. Source language, semantic profile, semantic-parameter
schema, selection schema, and registered foundation descriptor MUST equal that
entry. Every repeated semantic context is compared by field-complete typed
equality. A context digest or projection is never a substitute.

The shared validated request interprets selection through the schema named by
the resolved profile entry. For `mpk.csharp.practical.v1` it is exactly
`mpk.selection.csharp_members.v1`; other profiles retain their own registered
selection rules. A frontend success repeats the complete request context and
selection linkage and contains the complete source-artifact root. A diagnostic
always binds the raw request hash and byte size. Its linkage is `unvalidated`
only before strict request/context validation and `validated` afterward.
Failure MUST contain no partial artifact root.

One context-dispatched `csharp2vir` executable and bundle may serve both C#
scalar and practical profiles. Dispatch is solely by the already validated
semantic profile. An ambient flag, fallback, mixed family, or profile inferred
from source text rejects. The scalar route must retain byte-identical source
verdicts, obligations, and Certificate v0 bytes.

## 5. Foundation, monomorphization, and VIR boundary

The practical registry entry binds exactly one registered
`mpk.csharp.practical.foundation.v1` descriptor and its independently
recomputed content hash. The descriptor owns the closed semantic-template
registry; callers cannot provide a bundle, template, operation, or allowlist.
Only instances transitively reachable from validated source and sidecar roots
are derived. Instance identities, dependency closure, provenance, sorting,
deduplication, depth, counts, and expansion are canonical.

Every derived template is expanded to a concrete monomorphic type and concrete
operations before VIR. `mpk.vir.v2` MUST NOT contain a template identifier,
generic parameter/application, unresolved operation, caller type metadata, or
framework object. The importer independently recomputes the complete closed
set and rejects missing, extra, unreachable, stale, colliding, or differently
expanded entries.

## 6. Producer, consumer, and test ownership

For each strict root, the sole producer and all immediate consumers are exact
in `frozen_contract.schemas`. For each nested record and tagged union, the
producer is exact in its row. Contract-expression variants are produced by
`CSHARP-03-T06-W01`. Identity-family implementation owners are exact in
`frozen_contract.identity_families`.

`name_owner_inventory` flattens these requirements and attaches the primary
test owner routed by the reviewed task plan. `downstream_work_item_owners`
contains exactly one primary path-plus-work-item owner for each of the 63
T02-T08 work items, plus that item's exact title, `Owns`, exit-gate,
verification, and plan anchor text. This binds every implementation surface
and release criterion to its task and primary test. A later implementation
MUST use that pair; moving an owner or changing the frozen task contract
requires a new reviewed freeze. Every conformance vector likewise names one
implementation work item and one exact production-test owner.

## 7. Atomic migration and rollback

The migration set is
`csharp-practical-successor-whole-release`. T02-W08 owns producer migration,
T02-W09 owns consumer migration and predecessor equivalence, and T08-W10 owns
activation. Private staging may build the complete successor family, but no
successor producer, consumer, registry entry, tuple, route, or compatibility
flag becomes public before the one atomic cutover.

Activation MUST install together:

- all successor producers, parsers, validators, canonicalizers, serializers,
  and hash preimages;
- the registered foundation bytes and complete closed-instance machinery;
- the semantic registry root, all entries, contexts, and compiled contracts;
- the release root, all five retained profile tuples, bundle descriptors,
  candidates, receipts, and hashes;
- CLI/API, policy/evidence/program-assembly/AI consumers; and
- fixtures, examples, reports, documentation, and the release gate.

Forbidden states include a new producer with an old consumer, an old producer
with a new consumer, mixed old/new roots or hash families, a public old/new
selector, a practical entry without complete registered foundation bytes, and
partial predecessor tuple migration.

Rollback replaces the entire installed image from the pre-cutover baseline,
including binaries, checker binaries, bundles, registries, descriptors,
contracts, fixtures, reports, and documentation. Restoring an individual
family, route, registry, hash, producer, or consumer is forbidden.

## 8. Limits, diagnostics, and rejection

Every parser and transformer applies the package's strict shape rules and its
owned structural limits before allocation or retention. Runtime value limits
become explicit VCs. Unknown schema/tag/member/identity/domain, duplicate key,
later version, over-limit value, hash mismatch, cross-context value, unowned
artifact, or mixed revision rejects without partial downstream artifacts.

Diagnostics use only the 29 practical families and exact phase precedence in
the package. Public failure records bind the correct request state and remain
bounded and sanitized. No permissive deserialization, default member,
unknown-field preservation, best-effort conversion, or legacy fallback exists.

## 9. Conformance and activation gate

The 700 published rows are minimum mandatory conformance cases. All applicable
retained predecessor vectors also run. Passing a model test without executing
the named production owner is insufficient. Every exact path/identity/domain
listed in the package is immutable for v1.

Before activation, `scripts/check-java-frontend.sh` remains the sole installed
release gate and `scripts/check-csharp-practical-release.sh` does not exist.
T07-W05 privately implements the future gate, T07-W06 records its receipt, and
T07-W05/W06 plus T08-W06/W09/W10 invoke exactly:

```text
sudo ./scripts/check-csharp-practical-release.sh
```

At T08-W10 the practical gate atomically replaces and retires the Java-named
gate. The aggregate `scripts/check-all.sh` then invokes the practical gate
exactly once. Until that cutover, these successor specifications remain
normative but inactive and production behavior remains unchanged.
