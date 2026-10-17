# Closed private registry, context and request protocol

Status: W04 draft; no installed family, complete Go foundation or source proof.
Entry: `4cc82980a6502fc0bb13813d6986307887613316`.

The [six root shapes](schemas.json) close required fields, nested envelopes,
profile references, typed comparisons and complete hash preimages for registry
v3, registry entry v3, semantic context v3, validated semantic request v3,
Go practical parameters v1 and Go functions selection v1. They use the
[previous candidate identities](../identity-dag/catalog.json) without changing
the reserved inactive C# family. JSON member order is canonicalized; the
listed field order describes the records rather than accepting transport
member order as authority. Extra, missing and unknown fields reject.

## Registry authority and foundation references

The [private registration fixture](registration-fixture.json) contains all
four retained profiles plus `mpk.go.practical.v1`. Entries and the root have
complete domain-separated self hashes. Validation compares the entire registry
with the compiled fixture after checking hashes. A caller cannot authorize a
new profile, contract, foundation or entry order by recomputing valid hashes.
Entry hashes bind language, profile, parameter/selection schemas, foundation
reference and all nine compiled contract names. Dispatch requires those
bindings rather than just the language string.

Every context requires a foundation field. It is null for each retained
profile and an exact registered `{schema,id,content_sha256}` reference for Go
practical. The new family does not widen the C#-specific foundation field of
its reserved v2 family. The Go fixture hash covers an explicitly incomplete
private foundation description. It is not the final registered definition
bundle, a caller bundle, a proof or a published foundation digest. The limits
hash similarly covers the preceding64-counter/weight candidate fixture;
complete normative limits/foundation schemas and definitions remain open.

Go parameters are the complete compiled table: Linux/amd64, pointer width64,
GOAMD64 v1, compiler go1.25.0, ordered effective languages go1.22–go1.25 and
the exact limits id/hash. Caller feature flags, numeric overrides, alternate
targets/compiler patches and omitted versions reject. These parameters do
not by themselves capture source bytes or identify the actual compiler image.

## Preserve actual existing parameter and selection rules

The [native observer](../../../../probes/go-practical/context-legacy/src/main.rs)
links the unchanged production `mpk-vc`, `mpk-cert` and `mpk-core` libraries.
It calls the actual revision3 registry, semantic-context and selection
validators. The private v3 wrapper first checks its own complete binding,
then projects retained-profile parameters/selections into that actual old
compiled registry. No Python subset replaces existing Rust policy.
The observer's initial registry serialization omitted the required LF and
rejected with `RegistryCanonical`; it now uses the existing production
`canonical_registry_transport` function. Production sources are unchanged.

Both Rust checked targets remain valid: i686/32 and x86_64/64. Fixed Go,
scalar C# and scalar Java retain their actual rules, including Java's path,
method and ordering checks. Thirty-one [distinct queries](legacy-queries.json)
have two identical actual native runs in the [receipt](observations.json).
The wrapper, complete linked source tree, private Cargo lock and binary are
pinned. This configuration-policy delegation is not a proof of four-language
semantic equivalence or an implemented producer/consumer migration.

## Typed request and selection linkage

A context includes its schema, full registry reference, entry hash, language,
profile, complete typed parameters and foundation reference. Repeated
contexts must have identical canonical typed values; no field projection is
used. Two individually valid Rust32/Rust64 contexts therefore cannot link.
Booleans do not equal integers. A request hash covers its schema and the
entire context and selection; only its own hash field is excluded.
All hashes use ASCII domain+NUL+narrowed RFC8785 JSON without LF.

Go selection has1–8192 explicit roots in unique canonical JSON byte order.
Each root contains package, function and a required nullable value-receiver
selector. Receiver aliases retain their raw source names at this boundary.
Unicode15 identifier rules match the pinned Go1.25 table version; keywords,
blank names, pointer receiver spellings and package `init` reject. An ordinary
method named `init` is a separate source form and is permitted by this raw
grammar. No source file is loaded or executed by this protocol probe.

Actual declaration existence, package identity, value-receiver method sets,
alias-resolved duplicates, selected source closure and capture membership
must be checked later against original typed source. Two distinct raw aliases
can still name one method. The syntax-layer candidate result grants none of
that source admission, and the method/alias fixture claims no executable body.

## Evidence and remaining boundary

The [152 vectors](vectors.json) have24 candidates and128 configuration
rejections: six contexts, eleven requests, six repeated-context links and one
registry are candidates. Cases exercise cross-language/profile/schema/entry/
foundation/limit bindings, old/future/reserved schemas, rehashed registry
attacks, stale/incomplete request hashes, ordering, nested fields and strict
duplicate/number/Unicode JSON rejection. These are configuration outcomes,
not checker verdicts, certified false claims or source execution outcomes.

Twenty targeted tests add both Rust targets, actual legacy negative-policy
results, 8192/8193 root boundaries, package/method `init`, unknown-oracle fatal
errors and complete typed linkage. Fresh replay rebuilds the native wrapper
and repeats exact outputs twice. The [verification record](verification/verification.json)
retains command results, raw logs and selection reasons. Older proof/native
matrices are not rerun; their sources, receipts and diagnostic snapshots are
checked for preservation.

Complete Go foundation and limits payloads, type/binding/call/contract/source
roots, downstream shared roots/unions and independent production import remain
required. All143 migration routes still require symbol/preimage/behavior
review and full existing-profile preservation. W04 is In progress. W05 owns
inactive publication and the cumulative T01 gate; T02–T11 remain required.

```sh
python3 -B develop/probes/go-practical/run_context_protocol.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest -v test_context_protocol
python3 -B develop/probes/go-practical/check_context_metadata.py
```
