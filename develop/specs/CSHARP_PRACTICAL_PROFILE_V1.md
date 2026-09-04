# C# Practical Profile v1

Status: normative, frozen, and inactive. Published by
`CSHARP-03-T01-W10` on 2026-09-04. Publication does not install a registry
entry, expose a CLI or API route, or change production acceptance. The only
next executable work item is `CSHARP-03-T02-W01`; activation remains owned by
`CSHARP-03-T08-W10`.

## 1. Authority and package

This specification defines `mpk.csharp.practical.v1`. Its exhaustive
machine-readable package is
`develop/specs/vectors/csharp-practical-profile-v1.json`, schema
`mpk.csharp.practical.profile.conformance.v1`. The primary specification test
owner is
`crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10`.

The package contains, without semantic rewriting:

- the complete W09 `frozen_contract`, including all names, strict shapes,
  diagnostic precedence, transition rules, termination rules, and limits;
- the same 700 sorted conformance rows as the W09 private handoff;
- raw hashes for every canonical W01-W09 evidence record and all three
  specification members;
- one exact primary test owner for every freeze requirement and every T02-T08
  work item, together with each downstream item's exact `Owns`, exit-gate, and
  verification contract from the reviewed task plan;
- a flattened owner inventory for every identity, hash domain, schema shape,
  diagnostic family, and limit; and
- the exclusion/upgrade matrix and future release-gate decision.

The published package is the authority for exact strings, field order, field
types, hash domains and preimages, numeric ceilings, producer/consumer work
items, expected vector results, and production-test owner pairs. This prose is
the authority for how those rows compose. A conflict is a specification defect:
reject the candidate and return to T01 rather than choosing one representation.
The detailed source rules incorporated from sections 6 through 23 of
`develop/docs/08_csharp_practical_subset_design.md` remain binding; their exact
publication-time projection hash is recorded in `incorporated_design`.

The companion
`develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md` defines the mandatory
successor shared-artifact migration. The registered foundation and its closed
specialization semantics are defined by
`develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md`. Certificate v0 and both
source-free checker specifications retain higher authority and are unchanged.

## 2. Trust and application boundary

Application source is ordinary C#. It MUST NOT import an MPK namespace,
reference an MPK package or assembly, implement an MPK interface, inherit an
MPK base type, use an MPK attribute, compile generated MPK source, or deploy an
MPK runtime component. Fully qualifying such a symbol does not bypass this
rule. MPK-owned contracts, semantic bindings, boundary documents, descriptors,
and proof artifacts are separate verification-overlay inputs or outputs.

Roslyn and the .NET runtime are untrusted capture and observation mechanisms.
Acceptance requires deterministic validation and lowering, ordinary
Certificate v0 generation, and agreement of both source-free checkers. No
compiler result, runtime result, serializer, database, clock, identity
provider, transport, or filesystem effect becomes a proof premise merely by
being observed.

Every accepted request MUST select one immutable semantic-registry entry by a
complete validated semantic context. Digests never substitute for repeated
typed values. Every emitted practical artifact MUST bind the same context,
selection, foundation descriptor, and closed-instance set. A mixed artifact
family, projected context, ambient flag, fallback, or unregistered bundle
MUST reject before VIR emission.

## 3. Fixed compilation context

The practical parameter document is a strict object with these exact values:

| Parameter | Required value |
| --- | --- |
| `language_version` | `14.0` |
| `nullable_context` | `enable` |
| `check_overflow_default` | `true` |
| `optimization` | `release` |
| `platform` | `x64` |
| `pointer_width` | `64` |
| `source_kind` | `regular` |
| `documentation_mode` | `none` |
| `preprocessor_symbols` | empty array |
| `unsafe` | `false` |
| `target_framework` | `net10.0` |
| `target_id` | `linux-x64` |

The root schema is `mpk.semantic_parameters.csharp_practical.v1`. Unknown,
missing, duplicate, differently typed, or differently valued members reject.
The selection schema is `mpk.selection.csharp_members.v1`: schema,
compilation ID, sorted nonempty unique source paths, sorted nonempty unique
selected method/constructor IDs, sorted unique sidecar paths, and the final
selection hash. Project files, packages, binaries, references, toolchains,
environment overrides, fallback inputs, and unselected source are forbidden.

## 4. Closed source language

The profile is an intentionally closed C# subset for deterministic business
logic, not general C# or general .NET. An implementation MUST accept only the
forms enumerated by the incorporated design and MUST reject any source,
symbol, operation, conversion, synthesized form, or metadata dependency not
explicitly admitted there.

Positive scope includes the already frozen scalar semantics plus:

- expression-bodied members, local `var`, and name-resolution-only ordinary
  namespace imports that normalize to an otherwise admitted form;
- source-defined enums, immutable structs, and sealed immutable classes with
  the frozen field/property/constructor/`init`/`required`/initializer rules;
- bounded one-dimensional arrays, linear count-then-allocate construction,
  application-owned ordered entry/map/set representations, and admitted
  loops for deterministic collection algorithms;
- bounded UTF-16 strings and the closed ordinal operation and ASCII
  parse/format sets;
- exact `float`, `double`, and .NET `decimal` operations named by the operation
  profile, including frozen NaN, signed-zero, scale, rounding, and overflow
  behavior;
- value-type nullable `T?`, nullable admitted references, and the separate
  boundary-only missing/null/value presence sum;
- exact admitted date, time-of-day, duration, GUID, application-owned instant,
  and application-owned money shapes;
- application-owned closed outcome types bound to option, lookup, result, and
  accumulating validation semantics;
- the frozen loops, switches, patterns, explicit throws, typed catches, pure
  filters, and `finally` forms; and
- the verification-overlay boundary and pure transition model.

Reachability begins at selected roots and closes over every source-declared
type and callable reached through admitted members and types. The call graph
MUST be finite and acyclic. Every ordinary declaration in a selected source
file MUST belong to and satisfy the closed compilation; unreachable or
unselected declarations are not ignored or trusted.

## 5. Generics and MPK-owned standard foundation

Every user-defined generic type declaration, generic method, type parameter,
constraint, variance form, generic inference use, and closed use of a
user-defined generic MUST reject. Constructed framework types also reject
except for exact value-type `T?` source syntax over an admitted closed payload.
Explicit `System.Nullable<T>`, an open argument, a reference-type payload, an
unsupported value payload, or any residual generic value MUST reject.

Names such as `option<T>`, `result<T,E>`, and `transition<S,E,R>` denote only
MPK-owned semantic templates in the registered verification foundation. They
are not C# types and create no application dependency. Only the twelve
templates and four non-template definitions frozen by
`CSHARP_PRACTICAL_FOUNDATION_V1.md` exist. A caller cannot add a template,
operation, instance, or allowlist entry.

For each compilation, the frontend derives the transitive set of required
closed instances from validated source/sidecar provenance, canonicalizes and
deduplicates it, enforces all frozen limits, and expands every instance to
ordinary concrete definitions before VIR emission. The VIR/importer boundary
MUST reject a template, type parameter, generic application, open type, or
unresolved operation. The final Boolean-cube lookup and static
concrete-transformer recipes use only ordinary checked core definitions; they
do not require a new core construct, proof-node kind, theory certificate, or
axiom.

## 6. Values, equality, ordering, and operations

All admitted source operations use the exact C# evaluation order, null checks,
conversion, checked/unchecked overflow, floating-point bit behavior, decimal
behavior, and exception precedence fixed by the package vectors. No ambient
culture, comparer, locale, timezone database, resource, regex engine, general
parser/formatter, reflection, dynamic dispatch, or runtime code generation is
part of the profile.

Structural equality is recursive, field-complete, and defined only for frozen
admitted shapes. Canonical ordering is available only for the closed total-key
matrix. Reference identity, mutable aliasing, user equality/operator overloads,
caller comparers, array covariance, multidimensional or jagged arrays, and
unordered framework collection semantics reject. A source binding MUST prove
field-complete projection and reconstruction, distinct arms, operation
commutation, source-content linkage, and absence of observable identity. A
binding obligation is not itself a proof.

## 7. Contracts, control, exceptions, and termination

Method, type, loop, boundary, and transition contracts use only the 33 closed
expression variants listed in `frozen_contract.expression_union`. Unknown
tags, wrong/missing/duplicate fields, ill-typed expressions, unbounded
quantifiers, or an expression outside the selected context reject.

Every admitted loop requires an invariant and well-founded decreases proof.
Every admitted total route—boundary, transition, checked-in example, and
public practical-profile route—MUST have a finite acyclic call graph and may
not reach a partial callee. Bounded quantifier ranges are evaluated before
body traversal. Generated static networks check their finite counts before
generation and use bounded balanced composition.

Exceptions are explicit closed values with exact normal/exceptional control
edges. The source forms, built-in exceptional cases, filter purity, handler
selection, `finally` ordering, and error precedence are exactly those in the
frozen contract and vectors. Catchable resource-exhaustion behavior is not an
admitted semantic claim.

## 8. Canonical boundary and transitions

`mpk.csharp.boundary.v1` is an MPK verification-overlay transport. It is not
an application wire protocol. External serialization and transport adapters
remain untrusted unless another profile verifies them. Input MUST follow this
order: capture exact bytes/provenance; strictly parse; construct a complete
typed value; hash raw bytes and canonical value; bind both into manifest and
evidence; then invoke the selected original method. Output MUST capture the
complete returned value, canonically encode it, reparse the bytes, compare the
complete typed values, and bind all three identities. Bypass and hash-only
equivalence reject.

Canonical encoding is UTF-8 without BOM or trailing newline, with no external
whitespace, schema-declared member order, strict unknown/duplicate rejection,
closed tagged sums, array-encoded ordered maps, and the exact scalar encodings
in `frozen_contract.canonical_json`. Missing omits a field, null emits JSON
null, and value emits exactly one payload; these states are never collapsed.

A transition preserves source event order and uses the frozen error
precedence. New success increments a `u64` version by checked addition;
conflict, replay, error, and overflow behavior is exact. Idempotency is either
disabled or retains key, complete command snapshot, complete context snapshot,
and complete response. It is unavailable for incomplete snapshots or any
recursively non-reflexive field, including `float` or `double`. Digests cannot
replace snapshots and there is no eviction fallback.

## 9. Diagnostics, limits, and failure atomicity

The 29 diagnostic families and their phase precedence are closed by
`frozen_contract.diagnostics`. The frontend stops after the earliest failing
phase. Within that phase, diagnostics sort by family precedence, source-file
ordinal, start byte, end byte, and code. Public diagnostics are bounded and
MUST NOT contain customer names/member spellings, source snippets, raw compiler
or exception prose, host paths, generated type names, culture, or stack text.
Failure carries no partial source artifacts.

The package contains 35 practical limits and all 32 unchanged scalar-v0
limits. Every structural counter uses one recorded increment site, checked
addition, and rejection before allocation or retention when the candidate is
greater than the inclusive maximum. Every runtime value bound is a predicate
and VC that MUST be proved before verified acceptance. Counter overflow is a
limit rejection. The below/at/above vectors are normative; an implementation
MUST NOT replace one limit with a different aggregate counter.

## 10. Exclusions and upgrades

The twelve `upgrade_matrix.excluded_families` rows are rejection scope. In
particular, user generics, inheritance/dynamic dispatch, delegates/lambdas/
LINQ, iterators/`yield`, async/tasks, mutable identity, ambient services,
general text processing, unsupported collections, and infrastructure
correctness claims have no positive vector in this profile.

Admitting any excluded form requires a new semantic-profile identity, new or
versioned shared artifacts as applicable, regenerated specifications and
vectors, a fresh feasibility review, and an atomic whole-release gate.
`mpk.csharp.practical.v1` is never widened. The sole nullable exception remains
the exact value-type `T?` form stated above.

## 11. Conformance and release routing

An implementation conforms only when every applicable published vector runs
through the row's exact `implementation_owner` production path and passes the
exact `production_test_owner`; specification-model tests alone do not count as
production execution. Each downstream owner row freezes its full task
requirement, exit gate, verification command class, and plan anchor, so a
requirement cannot be detached from its primary work-item/test-owner pair.
Unknown values and unlisted positive behavior reject. All retained predecessor
vectors also remain mandatory.

The future candidate and release command is exactly:

```text
sudo ./scripts/check-csharp-practical-release.sh
```

`CSHARP-03-T07-W05` owns the private gate implementation,
`CSHARP-03-T07-W06` owns its receipt, and T07-W05/W06 plus T08-W06/W09/W10
invoke it as listed in the package. Before activation,
`scripts/check-java-frontend.sh` remains the sole installed-release gate and
the practical gate path MUST be absent. At `CSHARP-03-T08-W10`, activation
atomically replaces and retires the Java-named gate; `scripts/check-all.sh`
then delegates exactly once to the practical gate. It does not extend or call
both gates.

Until that atomic cutover, this specification and its vectors are normative
but inactive. They authorize no production route, installed tuple, registry
entry, compatibility selector, or partial successor artifact family.
