# Foundation and limits packaging rules under review

Status: private W04 draft. This document fixes packaging and references; it
does not turn an operation recipe inventory into complete kernel definitions.
The full source/operation/contract proof library remains required.

The limits record has exactly schema, id, version, semantic_profile, counters,
weights and content_sha256. There are64 independent, id-sorted counters.
Each has id, maximum, unit, category, counting_rule and exceeded. Their
maxima/counting meanings are the preceding numeric draft, including the new
practical-only4,194,304 syntax ceiling. Existing profiles keep their current
rules. Duplicate or omitted counters, caller overrides and rehashed variants
reject. Counts/versions are exact nonnegative safe JSON integers; a Boolean
does not stand for one or zero. Source words retain their full width outside
JSON metadata and are not narrowed to the JSON safe-integer range.

The source-capture, structural-artifact, capture-manifest, semantic-domain,
carrier-accounting and proof-resource categories remain distinct. Capture
and structural overflows reject configuration/model construction. Semantic
bounds and weighted live/past counts remain actual proof-domain obligations.
A proof-resource exhaustion stays resource-pending, never semantic falsity
or a Go panic. Runtime definedness and source typing are independent of every
numeric ceiling. A width below64 does not authorize an unsupported word type.

Units are byte counts, monotonic nanoseconds, bit widths, nesting depth,
abstract carrier units or the count of the exact named object. A byte-string
bound counts arbitrary Go bytes, not Unicode characters. Shared JSON string
limits count their predecessor-defined UTF-8 bytes. Resource RSS is bytes
for one bounded child. Process-tree supervision remains an implementation
requirement and these observations do not establish an aggregate OS bound.

Fixed abstract weights charge Bool/full words, all256 byte slots plus length,
slice/location paths, all array/struct fields, every global error payload,
owner/scope/packet and observation headers. Named/alias edges retain the
underlying weight while keeping nominal identity elsewhere. Zero arrays and
inactive payloads cannot hide an invalid/cyclic expanded type graph. Current,
entry and post namespaces and copied values follow the prior accounting
rules. Backing collections use store totals with per-cell checks. No caller
supplies a live set, extension equation or lighter weight interpretation.
Finite semantic-id and source/allocator correspondence proofs remain due.

The definitions record is explicitly an operation_recipe_inventory_draft.
It contains all56 logical operation records in eight component families,
all six common source/validity/evaluation/safety/value-effect/frame members,
the exact limits reference, joint core interfaces,77 mirrored checker input
hashes, source operation-table identity and six historical evidence receipts.
Those receipts prove their own retained scopes; they do not discharge a new
source function, missing specialization or foundation expansion. Both source
callable-members and caller-extension arrays are empty. Every operation still
needs ordinary zero-axiom source/model, definedness, result/effect and frame
members, and the complete generated concrete definitions require both
checkers. Placeholder references, added axioms and a definition-only check
cannot satisfy those requirements.

The joint core proposal is exactly the retained Sort0 Nat.induct and
Bool.recType extension. The former has type
`(P : Nat -> Sort0) -> P zero -> ((n : Nat) -> P n -> P (succ n)) -> (n : Nat) -> P n`.
Its zero equation returns z; its successor equation is
`induct P z s (succ n) = s n (induct P z s n)`.
Bool type selection has type `Sort0 -> Sort0 -> Bool -> Sort0`, returning
its first/second type at false/true. Only canonical generated Sort0 families
with exactly two canonical generated constructors and exact reserved
eliminator types qualify. All universe arguments reject, including in
partial/neutral uses, prior types, opaque values and theorem proofs. Later
constructors reject without state changes. Old rec/cases rules stay intact.
The four source implementations and complete mirrored source closure are
hashed; production integration still belongs to T02. No new wire opcode,
axiom allowlist or Certificate v0 format is introduced by this packaging.

The descriptor has exactly schema, id, version, semantic_profile, activation,
limits_ref, definitions_ref, members, source_callable_members,
caller_extension_points and content_sha256. Its candidate_only activation
label grants no installed route. Its path-sorted member list contains exactly
the limits JSON, definitions JSON and this document. It excludes itself,
compiled executable bytes, a release receipt and compilation instances.
Thus limits precede definitions, both precede descriptor, descriptor precedes
registry/context, and capture/specialization/program/release data follow them.
There is no reverse reference or descriptor/context/release self-hash cycle.

References have exactly schema, id and content_sha256. Raw members have
exactly path, schema (null for this Markdown), sha256 and size_bytes. Member
hashes cover original complete file bytes, including final LF; record hashes
cover ASCII domain+NUL+narrowed RFC8785 JSON without LF, omitting only the own
content_sha256 field. All other active fields enter the preimage. Registered
member paths are logical repository-relative paths. Missing/extra/reordered,
escaping, symlinked, altered-size or stale members reject. A matching hash
does not authorize a caller-chosen descriptor or semantic definition.

The earlier context protocol's incomplete fixture remains historical private
evidence. This new candidate links the exact limits/definitions/descriptor
records throughout a freshly computed private registry and Go parameters.
Mixing either generation's registry, foundation or limits references rejects.
This is draft assembly, not a public version change or production migration.
The four retained profiles keep null foundations and their actual compiled
parameter/selection policies, including Rust32 and Rust64. The inactive C#
family and all old normative identities/preimages remain unchanged.

Remaining freeze work includes complete ordinary closed definitions and
specialization/expansion/binding/type/contract/downstream schemas, independent
source validation, reviewed producer/consumer migration and all old-profile
semantic preservation gates. W04 is open. W05 owns inactive publication and
the cumulative T01 gate; the full T02–T11 objective remains required.
