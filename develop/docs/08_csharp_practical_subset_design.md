# C# Practical Subset Expansion Design

Status: proposed governance and implementation design. This document does not
change the active `mpk.csharp.scalar.v0` profile, register a new profile, or
authorize a practical-profile public route. The active release is registry
revision 3 with Go, Rust, C# scalar, and Java scalar support. `JAVA-03-T10`
and native x86-64 Linux gate completed on 2026-09-03. The CSHARP-03 entry gate
is satisfied, but this proposal does not itself start specification or
implementation work.

Prepared: 2026-09-02. Revised: 2026-09-03.

## 1. Decision

Create a new, immutable C# profile for deterministic business-domain logic.
The working name in this design is `mpk.csharp.practical.v1`. The name is not
an accepted schema value until a later specification-freeze task assigns exact
identities, canonical vectors, hashes, compiler observations, and limits.

Do not widen or reinterpret `mpk.csharp.scalar.v0`. Its source semantics,
profile-payload meanings, diagnostic meanings, and historical artifact bytes
remain frozen. The practical profile is an additional semantic entry; the
common contract/artifact migration assigns successor identities without
mutating any old ID or byte meaning.

Every source form admitted by `mpk.csharp.scalar.v0` remains admitted under the
new practical context with the same C# value, evaluation-order, integer,
conversion, and checked/unchecked-overflow semantics. This is semantic
preservation, not artifact compatibility: the selected profile and registry
identity are context-bound, so scalar-profile and practical-profile sidecars,
VIR, maps, manifests, VCs, evidence, and hashes are never interchangeable.

The default and normative integration path is source-dependency-free. Selected
application source does not import an MPK namespace, reference an MPK assembly
or package, implement an MPK interface, carry an MPK attribute, or compile
generated MPK source. MPK is an internal capture, validation, lowering, and
checking tool. Application-owned concrete types and exact compiler-recognized
source forms are mapped to MPK-owned semantic definitions by the frontend and
sidecars, then specialized before VIR emission. No MPK runtime component is
linked into or deployed with the application.

In this document, notation such as `option<T>`, `result<T,E>`,
`bounded_sequence<T>`, or `transition<S,E,R>` names an internal semantic
template. It is not a C# type name and does not authorize a source reference to
an `Mpk.*` type. The registered practical foundation bundle owns these
templates, their operations, and their expansion rules; each accepted
verification artifact set contains only the required closed, monomorphic
instances. The application source and build output contain none of them. That
bundle is the MPK standard library for verification purposes; it is not a
distributable .NET application library.

“MPK standard library” in this document never means the .NET Base Class
Library. The former is a versioned verification-foundation input expanded to
ordinary monomorphic definitions; the latter remains outside proof and is
available to selected source only through the exact framework-symbol and
operation allowlists stated below.

The requested capability set is:

| Area | Practical-profile boundary |
| --- | --- |
| Concise syntax | expression-bodied members, locally inferred `var`, and name-resolution-only ordinary namespace `using` directives when they normalize to an otherwise admitted form |
| Domain data | source-defined enums, immutable structs, and sealed immutable classes with fields, properties, constructors, `init`/`required`, object initializers, and pure instance methods |
| Collections | bounded arrays plus internal immutable-sequence and canonical ordered-map/set semantics; source lookup, aggregation, and construction use arrays, application-owned closed types, and admitted loops |
| Text | bounded UTF-16 strings and ordinal source operations, plus exact culture-free parse/format relations at the canonical boundary |
| Numbers | existing integers plus exact `float`, `double`, and .NET `decimal` semantics |
| Absence | nullable value types, nullable string/array/class references, and boundary-only missing/null/value presence with explicit proof obligations |
| Business values | exact date, time-of-day, duration, GUID, plus application-owned instant/money values bound to internal semantic definitions; structural equality is available, while canonical ordering follows the closed section 8.5 matrix |
| Domain outcomes | application-owned closed outcome types mapped to internal `option<T>`, `lookup<T>`, `result<T,E>`, and accumulating `validation<T,E>` semantics |
| Control flow | `while`, `do`, `for`, `foreach`, `break`, `continue`, switch statements/expressions, and closed patterns |
| Failure | explicit throws, built-in operation exceptions, typed catch, pure filters, and `finally` under a closed exception model |
| Integration footprint | no MPK source, package, assembly, attribute, generated-code, or runtime dependency in the application |
| Integration | versioned verification-overlay canonical boundary values and pure state/command-to-new-state/events/response transitions; serializers, clocks, databases, and transports remain outside proof |

“Supported” means source capture, pinned Roslyn analysis, profile validation,
lowering, source mapping, VIR import, VC generation, both-checker certificate
acceptance, policy/evidence, installed execution, and deterministic replay all
agree. Merely parsing a syntax form does not satisfy this definition.

## 2. Authority and serial scheduling

The normative authority remains, in order:

1. Certificate v0 and the source-free checker specifications;
2. the active successor artifact and semantic-registry specifications;
3. `CSHARP_PROFILE_V0.md` for the existing scalar profile; and
4. a future frozen practical-profile specification and owned vectors.

This document is subordinate to the first three. A conflict is resolved by
changing this design or minting a new artifact version, never by silently
changing an existing meaning.

The proposed serial production order is:

```text
JAVA-03-T10 -> CSHARP-03 practical profile -> DART-04
```

Preparing this design was a governance amendment, not the start of CSHARP-03
specification or implementation. The `JAVA-03-T10` code predecessor is
complete with its native receipt. W01/W02/W03/W04/W05/W06/W07 have now closed the
entry audit, consumer inventory, private frontend/toolchain closure proof, and
Roslyn data/construction/control/exception/pattern/dependency/generic/iterator/
async-rejection plus primitive/string/numeric/codec runtime measurements
without a normative identity or production change, and W08 is ready. DART-04
waits for the complete CSHARP-03 release gate. This
insertion records the user value of making the already released C# frontend
useful for business-domain logic before adding another language; it does not
authorize parallel language work.

## 3. Goals and non-goals

### 3.1 Goals

- Let users express ordinary immutable domain values rather than flatten every
  value to unrelated scalar parameters.
- Cover deterministic validation, pricing, eligibility, aggregation, and
  transformation code over bounded data.
- Cover bounded variable-length results, key-based lookup, accumulating
  validation, business dates and identifiers, and optimistic state transitions
  without admitting ambient services or requiring a source-facing MPK library.
- Preserve C# evaluation order, overflow, rounding, null, and exception
  behavior for every admitted form.
- Verify captured application source directly while keeping MPK contracts,
  semantic templates, generated artifacts, and tooling in an internal
  verification overlay.
- Keep compiler and runtime observations untrusted and independently
  revalidated by MPK-owned representations and both source-free checkers.
- Keep all input graphs, values, CFGs, proof terms, diagnostics, and processes
  deterministically bounded.
- Add no axiom, theory certificate, C# semantics primitive, or new
  proof-authority path to an accepted program certificate.
- Retain identical semantics for every active Go, Rust, C# scalar, and Java
  profile during the required shared-artifact migration.

### 3.2 Non-goals

The practical profile is not arbitrary C# or arbitrary .NET. It does not
accept:

- mutable shared object graphs, observable object identity, weak references,
  finalizers, or unsafe/ref-like storage;
- inheritance or virtual/interface dispatch, except the closed exception
  hierarchy and exact compiler-recognized source forms described below;
- user-defined operators/conversions, delegates, lambdas, expression trees,
  LINQ, reflection, `dynamic`, records, record structs, `with` expressions,
  every user-defined generic type or method (including a closed use), type
  parameters, constraints, variance, generic method inference, or runtime code
  generation;
- general `List<T>`, `Dictionary<K,V>`, `System.Collections.Immutable`, spans,
  caller comparers, arbitrary framework collections, array covariance,
  multidimensional arrays, or jagged arrays; section 9 admits arrays and
  application-owned non-generic concrete representations rather than a
  source-facing MPK collection API;
- culture-sensitive text, normalization, regular expressions, general-purpose
  source formatting/parsing, resources, globalization, or ambient locale;
  section 10's exact canonical grammars exist only at the boundary;
- filesystem, database, network, clock, random, environment, console, process,
  synchronization, thread, scheduler, cancellation, or other external effects;
- iterator methods, `yield`, non-generic and generic `IEnumerable`/
  `IEnumerator`, `IAsyncEnumerable<T>`, `IAsyncEnumerator<T>`, `async`/`await`,
  `Task`, `Task<T>`, `ValueTask`, `ValueTask<T>`, custom awaiters, task races,
  parallel execution, or an assertion that an external asynchronous operation
  is correct;
- catchable resource exhaustion such as `OutOfMemoryException` or
  `StackOverflowException`;
- project/NuGet discovery, analyzers, generators, MSBuild behavior, an ambient
  reference assembly, or any application reference to an MPK package,
  assembly, namespace, attribute, interface, base type, generated source, or
  runtime component; and
- every source-written attribute. Exact compiler-synthesized metadata markers
  required by admitted `init`/`required` shapes are frozen observations, not an
  attribute-syntax exception.

Application adapters may use those features outside the verified source root.
Their outputs must enter the verified core as explicit values satisfying
checked preconditions. MPK does not turn an adapter contract into proof of the
adapter, service, database, or network.

Likewise, the profile does not verify a JSON serializer, HTTP stack, identity
provider, database transaction, system clock, or timezone database. Section 18
defines how their bounded outputs enter a verified pure core and which claims
remain outside proof.

## 4. Trust and processing boundary

The expanded path remains certificate-first:

```text
captured C# + sidecars
        |
        v
pinned Roslyn syntax / symbols / IOperation / CFG      (untrusted observation)
        |
        v
source-dependency and semantic-binding validation       (untrusted producer)
        |
        v
closed-instance collection and monomorphic expansion   (untrusted producer)
        |
        v
csharp2vir lowering; no generic value crosses this bar  (untrusted producer)
        |
        v
VIR v2 -> VC/skeleton -> canonical certificate bytes   (untrusted helpers)
                                      |
                                      v
                         Rust checker + reference checker
                                      |
                                      v
                              proof acceptance
```

Roslyn success, nullable warnings, runtime differential results, semantic
binding, specialization, and frontend success are never proof evidence. They
establish only that the untrusted producer followed the frozen adapter
contract. Proof acceptance still comes from identical certificate bytes
accepted by both source-free checkers.

The capture request supplies source bytes and sidecars, not an application
project mutation. MPK writes no source or project file, and an accepted
application compilation has no assembly reference whose identity belongs to
the practical foundation bundle. A verification-only wrapper may convert
boundary data and invoke the same selected method, but it may not reimplement
business logic and claim that the original method was proved. Evidence binds
the exact selected source bytes, semantic bindings, and compilation context.

The validated semantic context and registered release tuple select one exact
foundation-bundle descriptor and content hash. A request cannot supply a
bundle path, body, template table, operation set, or replacement hash. The
frontend and VIR importer each read an immutable registered snapshot, recompute
its content hash, and require the same identity in the source manifest and
reproduction evidence before using an expansion.

The frontend continues to execute only in the registered Linux x86-64 release
sandbox with the exact SDK, Roslyn packages, reference assemblies, runtime,
native closure, arguments, environment, cgroup, filesystem, process, and
network restrictions. The new profile cannot accept a caller-selected
compiler, runtime, assembly, culture, or feature flag.

## 5. Version and migration boundary

### 5.1 Why a new shared artifact revision is required

The current VIR can represent fixed-size arrays, structural values, and cyclic
CFGs with loop contracts. It cannot faithfully represent all of the following
without changing a closed meaning:

- run-time-length arrays, immutable sequences, linear sequence-construction
  state, canonical ordered maps/sets, and UTF-16 strings;
- nullable/optional values;
- tagged option/lookup/result/validation/boundary-presence/transition values,
  business-date/time/GUID values, and construction-state tracking;
- application-owned concrete-type bindings and their closed specialization
  identities;
- IEEE binary32/binary64 and .NET decimal operations;
- normal and exceptional successors from one source operation;
- exception propagation and handlers;
- the contract expressions needed for sequence, field, null, decimal,
  floating-point, ordered-collection, parse/format, boundary, and exceptional
  postconditions.

Encoding these as profile-specific strings inside existing fields is
forbidden. The practical profile therefore requires a successor shared
artifact family. Working names are:

| Role | Working successor identity |
| --- | --- |
| Semantic profile | `mpk.csharp.practical.v1` |
| Parameters | `mpk.semantic_parameters.csharp_practical.v1` |
| Selection | `mpk.selection.csharp_members.v1` |
| Method/type contracts | `mpk.csharp.contract.v1`, `mpk.csharp.type_contract.v1` |
| Application semantic bindings | `mpk.csharp.semantic_binding.v1` |
| Closed semantic-instance set | `mpk.csharp.closed_instances.v1` |
| Business boundary/transition contracts | `mpk.csharp.boundary.v1`, `mpk.csharp.transition.v1` |
| Internal semantic-template bundle | `mpk.csharp.practical.foundation.v1` plus a successor descriptor and content-hash domain |
| Operation/check profiles | `mpk.csharp.operations.v1`, `mpk.csharp.required_checks.v1` |
| Limits | `mpk.csharp.limits.v1` |
| Semantic registry | successor schema/root/entry family after `mpk.semantic_profile.registry.v1` |
| Shared program IR | `mpk.vir.v2` with a new hash domain |
| Frontend protocol | successor request, success, and diagnostic schemas |
| Source map | successor schema and hash domain |
| Source manifest | successor frontend/certificate-stage schema and hash domain |
| VC and skeleton | successor versions after the current `mpk.vc.v2` family, including a new VC hash domain |
| Release registry | successor root, tuple, descriptor, candidate, and receipt schemas plus a new root hash domain |
| Policy/evidence | successor scan, evidence, reproduction, and receipt schemas |
| Program assembly | successor assembly profile consuming the new context; Certificate v0 remains unchanged |
| AI/API | successor request, session, report, and route contracts that repeat the new semantic context |

The new common value, operation, exception, and contract vocabulary makes
these successor surfaces mandatory under
`SEMANTIC_PROFILE_REGISTRY_V1.md` section 11; they are not conditional on an
implementation happening to fit a new field into an old transport. Relevant
private runner/build transports also receive successor identities wherever
their closed preimage or compiled contract changes.

These working names reserve no public values. T01 must inventory every active
producer, consumer, repeated context member, and serialized root, then settle
the exact names. Every self-hashed root whose preimage changes receives a new
domain, including contract, registry, foundation-bundle, VIR, source-map,
manifest, release-root, semantic-binding, closed-instance, and VC hashes. A
Certificate v0,
declaration, axiom-report, or input-set domain may remain only when its exact
preimage and meaning remain unchanged. Old parsers must reject every new
family, and new parsers must reject old-family bytes wherever parallel
acceptance would create ambiguity.

### 5.2 Atomic migration

One release must:

1. retain every existing profile identity and language semantics, migrating
   only entry envelopes and compiled bindings required by the successor
   registry schema;
2. add the practical C# entry only after its specification and vectors are
   complete;
3. migrate every active producer and consumer to the one successor shared
   artifact family;
4. regenerate the registered foundation bundle and all context-bound
   contracts, semantic bindings, closed-instance sets, transports, manifests,
   examples, fixtures, VCs, reports, receipts, and hashes;
5. prove predecessor Go/Rust/C# scalar/Java source behavior and obligations are
   unchanged; and
6. atomically install the new registry, release registry, binaries, bundles,
   policy routes, API routes, and documentation.

The installed binary must not publicly import both VIR revisions. Existing
canonical certificates remain independently checkable because source-helper
artifacts do not participate in Certificate v0 checking. Rollback replaces the
whole installed image.

The practical frontend candidate may also implement the scalar profile, but
it must dispatch only from a validated semantic context and pass byte-level
determinism gates plus source-verdict, semantic, and obligation equivalence
gates. Expected successor schema, context, and hash differences are recorded,
not suppressed. Do not retain an unnecessary second executable or ambient
fallback merely for compatibility.

## 6. Source closure and declaration model

The captured root remains closed over selected `src/**/*.cs` and exact JSON
sidecars. Project files, binaries, resources, generated sources, editor or
analyzer configuration, and unselected files reject before Roslyn runs. The
practical profile retains the scalar profile's namespace and static-helper
class forms and adds only the declarations closed below. The selected files
are the application's original source, not a rewritten MPK-flavored mirror.

The new selection envelope names:

- one compilation ID;
- all source paths;
- selected root methods or constructors;
- every type, method/constructor, semantic-binding, boundary, and transition
  sidecar; and
- no executable, reference, package, registry, or toolchain path.

The closure begins at the selected roots and includes every source-declared
type, constructor, property getter, and instance/static method reachable
through admitted calls and field/property types. Type and call graphs are
finite and bounded, and the call graph remains acyclic; direct or mutual
recursion rejects. Every ordinary declaration in a selected source file must
belong to that closed compilation and satisfy the profile; an unrelated,
unreachable, or unselected type/member does not become ignored or trusted and
rejects.

Partial declarations, source generators, ambient metadata user types, every
source-defined generic declaration, every nested type declaration, and
conditional compilation remain rejected. The only admitted metadata-backed
types beyond the pinned scalar value surface are the exact predefined text/
floating/decimal types, compiler-owned implicit `System.Object`,
`System.ValueType`, and `System.Enum` bases for the exact admitted declaration
forms, the closed exception hierarchy, `StringComparison.Ordinal` and the
closed allowlisted `System.MidpointRounding` values in their exact intrinsic
argument positions, admitted date/time/GUID types, the exact closed
`System.DayOfWeek` enum, the exact value-type-nullable source form described in
section 12.1, and the exact compiler-owned modifier/attribute identities emitted
for otherwise admitted `init`/`required` declarations. The implicit bases,
their inherited members, and those synthesized markers are opaque declaration
metadata, not admitted source values or calls; source access to a marker symbol
rejects. Compiler-synthesized members and markers may be observed only for an
exact frozen pattern and are never used as the semantic definition of an
admitted source feature.

### 6.1 Application-source dependency boundary

Neither the selected files nor any source-declared reachable type may resolve
to an assembly, namespace, attribute, interface, base type, or generated member
owned by MPK. The application compilation therefore remains buildable and
deployable without MPK. Fully qualifying an `Mpk.*` name does not avoid this
rule because assembly identity, rather than a `using` directive, determines the
dependency. The frontend verifies the complete metadata identity of every
admitted framework symbol and rejects namespace or type-name spoofing.

Contracts and semantic-binding sidecars live in the verification overlay. They
may name original application declarations by canonical source identity, but
they cannot add a member, change overload resolution, provide an assembly,
replace a method body, or assert that an unverified wrapper is equivalent to
the application. A sidecar or generated artifact is never copied into the
application project or included in its deployment output.

If an opt-in source-facing MPK library is ever required, it belongs to a
separate integration profile with separate identities, artifacts, dependency
evidence, and compatibility rules. It cannot be enabled as a flag or silent
widening of `mpk.csharp.practical.v1`.

Source-dependency-free does not mean that an arbitrary application method is
admitted unchanged. A method that uses a framework collection, task, iterator,
external service, or another unsupported value remains outside the selected
root. Its adapter must materialize admitted arrays and application-owned
concrete values before calling the pure method, or the application must refactor
that logic into such a method. MPK does not obtain a proof by translating a
different reimplementation.

### 6.2 Closed source types and internal specialization

Application declarations are non-generic. Generic type declarations, generic
methods, type parameters, constraints, variance, explicit generic method type
arguments, and inferred generic method calls reject even when a particular use
would be closed. Arbitrary constructed CLR/BCL types also reject. The sole
initial constructed-generic source exception is nullable value-type syntax
`T?`, validated as the exact compiler-owned `System.Nullable<T>` construction
and immediately mapped as described in section 12.1; the explicit spelling
`System.Nullable<T>` remains rejected. Arrays are not generic CLR types and
remain admitted under section 9.1.

An exact allowlisted framework value is an opaque profile intrinsic. Incidental
generic interfaces, base metadata, or helper signatures implemented by
`string`, admitted arrays, decimal/numeric types, dates/times, GUID, or another
such runtime type do not enter the captured source closure and do not make
those interfaces admitted. Conversely, any source-visible interface type,
interface conversion or constraint, member call, `foreach` protocol, pattern,
or dispatch that resolves through that metadata rejects unless this design
names one exact compiler-recognized intrinsic source form. The frontend
validates the selected symbol identity and allowlisted operation directly; it
never accepts a transitive metadata surface.

The exact registered foundation bundle instead contains this closed initial
registry of internal semantic templates. Its descriptor binds the bundle
schema and semantic-context ID, canonical member inventory, content hash,
template identities and versions, operation sets, dependency rules, expansion
definitions, and applicable counters. These are working semantic names, not
source or schema IDs:

| Template | Arity and direct generated dependencies | Derivation source |
| --- | --- | --- |
| `bounded_sequence<T>` | 1; none | admitted array/wrapper, string code units, validation errors, events, contract/boundary field |
| `sequence_construction<T>` | 1; `bounded_sequence<T>` | fresh-array or bounded string/sequence result construction; never a published value |
| `ordered_entry<K,V>` | 2; none | dependency-only product for an ordered map |
| `ordered_map<K,V>` | 2; `ordered_entry<K,V>`, `bounded_sequence<ordered_entry<K,V>>`, `lookup<V>` | bound ordered-entry array/wrapper or boundary/contract use |
| `ordered_set<T>` | 1; `bounded_sequence<T>` | bound ordered element array/wrapper or boundary/contract use |
| `option<T>` | 1; none | value-type/reference nullability or a bound option type |
| `lookup<T>` | 1; none | bound lookup type or ordered-map dependency |
| `result<T,E>` | 2; none | bound result type, codec result, money/instant outcome, or transition result |
| `validation<T,E>` | 2; `bounded_sequence<E>` | bound accumulating-validation type |
| `boundary_field<T>` | 1; none | bound missing/null/value source type |
| `transition<S,E,R>` | 3; `bounded_sequence<E>` | bound new-state/events/response source type |
| `money<C>` | 1; none | bound decimal amount plus closed currency carrier |

No other template name or arity belongs to the initial profile. A T01
feasibility result may require a smaller registry or operation set, but that
change must first revise and re-review this design; no implementation task may
silently add, remove, or change a template. The templates are not C# metadata
references and are not accepted program declarations. The profile freezes each
template identity, arity, operation set, dependency rule, and ordinary-core
expansion. Non-template semantic values such as `unit`, `parse_error`, the
internal instant, and the closed exception sum are registered separately. When
one is an argument of a template—for example `parse_error` in
`result<T,parse_error>`—the resulting concrete instance is still derived,
enumerated, bounded, and expanded by the same rules below.

For each compilation, the frontend derives a finite closed-instance set from
the admitted source types, semantic-binding sidecars, boundary schemas, and
contracts. It must:

1. resolve each template by its registered semantic identity;
2. require every argument to be an admitted closed concrete type;
3. recursively add dependencies, such as the error sequence used by a
   validation or the lookup result used by an ordered map;
4. derive the instance identity from the template identity and recursively
   canonical argument identities;
5. sort by canonical identity and emit one instance for duplicates; and
6. enforce instance-count, nesting, expanded-declaration, operation, and term
   limits before retaining VIR output.

A registered template alone authorizes no use. Every permitted concrete
instantiation appears exactly once in this derived per-compilation table; an
instance absent from the table rejects even when its template and argument
types are individually registered.

The source manifest records the exact foundation descriptor ID and content
hash, the complete derived instance table, and its source-binding provenance.
The importer independently validates the registered bundle bytes and
recomputes their hash, then recomputes the instance closure, identities, order,
and limits. Expansion converts every instance into concrete struct, tagged-sum,
sequence, map/set, or operation definitions before VIR emission. A type
parameter, generic definition, constructed generic type, or generic call
remaining at the VIR boundary is a deterministic internal consistency
rejection. VIR and VC gain no source-generic or semantic-template node;
certificates contain only existing ordinary core terms over concrete types,
and neither checker gains a C#-generic-specific representation or reduction
rule.

The closed-instance-set root repeats the foundation descriptor ID and content
hash. Each entry contains the semantic-context and template-version IDs, arity,
ordered concrete argument IDs, sorted direct dependency IDs, derived instance
ID, ordered expanded type/operation IDs, contributing source-binding IDs, and
checked expansion counters. The table is sorted by instance ID and has no
duplicate or unreachable entry. Callers cannot submit a second allowlist or
request an instance that is not derived from the captured closure.

### 6.3 Application-owned semantic bindings

A semantic-binding sidecar may classify an admitted application-owned concrete
type as a closed option, lookup, result, validation, boundary-presence,
transition, instant, money, sequence wrapper, ordered-entry, or ordered
collection representation. The binding names exact tag, payload, carrier, and
field/property identities as applicable. It is identification data, not a
semantic assertion: the frontend still checks that the type is source-owned,
non-generic, immutable after construction, acyclic, and bounded, and the VC
layer proves its constructor, active-arm, invariant, ordering, and operation
obligations.

Without a binding, a source type has only its ordinary structural semantics;
the frontend never infers a role from its name or shape. Any contract, boundary,
transition, or closed-instance request that treats an application type as one
of these roles requires exactly one reachable binding. The only role mappings
that need no application-type binding are exact value-type `T?`, nullable
reference forms, direct admitted arrays and strings, the admitted framework
date/time/GUID values, and a raw instant carrier explicitly classified by a
boundary or transition field. Money and every application-owned option,
lookup, result, validation, boundary-presence, transition, instant wrapper,
sequence wrapper, entry, map, or set always require one binding.

Each canonical binding entry contains exactly:

- schema, semantic-context, compilation, source-type, and source-hash
  identities;
- one registered representation-role and semantic-definition-version identity
  (the template version for a templated role, or the concrete definition
  version for a non-template role such as an instant wrapper);
- ordered carrier, tag, payload, element, key, value, state, event, and
  response member IDs required by that role, with all inapplicable positions
  absent;
- closed type-argument IDs inferred from those source members and repeated for
  cross-checking, never caller-selected independently;
- the complete source tag-value to internal-arm mapping, including the source
  behavior for an unknown tag;
- the actual `default(T)` arm or an explicit `default_ineligible` marker;
- applicable length, nesting, canonical-order, uniqueness, scale, and range
  bounds; and
- the sorted direct semantic-instance dependencies and binding hash.

One source type has at most one representation role in one semantic context.
Structural equality and ordering eligibility remain type-contract facts rather
than additional roles. Duplicate entries, unused members, missing members,
unknown fields, unknown role IDs, argument/member disagreement, dependency
cycles, and a binding not reachable from the selected roots reject before
lowering.

No field or invalid arm may become trusted merely because a sidecar omits it.
Every constructor, getter, and helper used by selected logic remains in the
source closure and is lowered from its original body. A binding cannot replace
a source helper with an internal operation unless the source form is an exact
profile intrinsic. Names are immaterial: no application namespace, product
term, status name, or member spelling is built into the profile. This keeps the
MPK documentation and foundation bundle use-case-neutral.

Each binding generates projection obligations. On values satisfying the source
public invariant, source-to-semantic projection must be total and select
exactly one arm; semantic-to-source reconstruction must establish the source
invariant; both round trips must preserve every admitted observation; distinct
active arms must not collapse; and an unknown or inactive source state must
follow its declared rejection or exception edge. Constructors, boundary
decoding, field/property reads, equality, ordering, and every selected helper
must commute with the projection. Failure of any obligation rejects the
binding and the compilation; it cannot be recorded as an assumption.

### 6.4 Canonical source identities

Newly admitted application data and exception types are top-level, nonnested,
and nonpartial. Namespace, type, member, parameter, and local identifiers retain
the scalar profile's ASCII identifier grammar; verbatim identifiers, Unicode
escapes, normalization aliases, and compiler-generated display names never
become canonical identities. A source spelling may use an admitted ordinary
namespace `using`, but identity always uses the fully resolved declaration.

T01 freezes one canonical declaration-identity encoding. At minimum it
contains the compilation ID, fully qualified namespace, declaration kind,
containing source type ID, member kind and source name, ordered exact parameter
type IDs, and exact result type ID where applicable. Constructors, fields,
properties, getters, and methods have distinct kinds; overloads differ by the
ordered parameter type IDs rather than a display signature. A separate
provenance attachment binds that logical ID to its normalized source path,
exact span, and captured file hash, so an edit changes evidence without
silently redefining the logical naming grammar. A sidecar never selects by a
short name, source display string, metadata token, generated backing-field
name, or Roslyn object identity. The frontend and importer independently
reconstruct both layers and reject an alias, collision, stale span/hash, or
disagreement before specialization.

## 7. Concise syntax

### 7.1 Expression-bodied members

Admit expression-bodied methods and pure property getters when the expression
itself is admitted and the corresponding block form would be admitted. The
frontend normalizes:

```csharp
public static int Identity(int x) => x;
```

to the same semantic return graph as:

```csharp
public static int Identity(int x)
{
    return x;
}
```

The source map retains the original arrow-expression span. Constructors,
setters, event accessors, operators, destructors, and arbitrary accessors do
not become accepted through this rule.

### 7.2 `var`

Admit `var` for a local variable with exactly one initializer, or for a
`foreach` variable with one uniquely resolved element type. In either case the
Roslyn-resolved type must be admitted and fully closed. The frontend
independently maps that type and emits the same VIR as explicit source
spelling. Anonymous types, target-typed constructs with no unique admitted
type, target-typed object creation/default and every other target-typed form
not explicitly admitted elsewhere, `dynamic`, implicit arrays requiring a
best-common-type conversion, multiple local declarators, and a `var` alias/type
named by source reject.

No semantic-profile field depends on whether a local used `var`; it is source
syntax and mapping evidence only.

### 7.3 Name-resolution-only syntax

Admit an ordinary, non-global namespace `using Namespace.Name;` directive at
compilation-unit or namespace scope. It contributes only to Roslyn name
resolution, emits no VIR node, and admits no symbol by itself. Every referenced
type and member must still resolve to an exact captured application declaration
or allowlisted framework symbol and pass the complete dependency, type, and
operation checks. Directive order, duplicate imports, and ambiguity follow the
pinned compiler; an unresolved or ambiguous source compilation fails before
subset validation.

`global using`, `using static`, alias directives, `extern alias`, using
statements/declarations for disposal, project-generated implicit/global usings,
and an imported MPK namespace reject. The exact `#nullable enable` directive is
accepted only when it occurs before the first non-directive token and remains
effective to end of file, as a redundant declaration of the profile's already
enabled nullable context; it emits no semantic node. A scoped directive,
`disable`, `restore`, annotations-only or warnings-only target, conditional
directive, pragma, line remap, or any other source directive rejects. Source
that relies on a project or generated global-using file must add an ordinary
explicit namespace `using` to the selected original file or remain outside the
profile; MPK does not synthesize one.

## 8. Domain data model

### 8.1 Enums and immutable structs

Admit source-defined enums with one exact underlying admitted integer type and
closed constant members. Duplicate numeric values are aliases of the same
runtime value; equality and patterns observe the integer carrier, while source
with duplicate switch labels fails normal compilation. Flags behavior and
`System.Enum` APIs remain excluded. An enum value originates only from a
declared member, a value already of that exact enum type, or checked canonical
boundary decoding. Numeric-to-enum casts and arithmetic reject. Its recursive
default is eligible only when carrier zero names a declared member; otherwise
publishing `default(EnumType)` rejects.

Admit non-generic `readonly struct` declarations whose complete field graph is
acyclic and contains only admitted scalar, decimal, floating-point, nullable,
string, one-dimensional array, exact admitted framework value, enum, or another
source-defined structural type that precedes it in the recomputed acyclic
dependency order. A type classified by a semantic-binding
sidecar is still checked under this same source-type rule; the internal role
does not add a source field type. Instance fields are explicitly `readonly`;
properties are getter-only or init-only; methods are pure and nonvirtual.
Custom layout, fixed buffers, ref fields, events, indexers, destructors,
operators, conversions, and boxing reject.

“User-defined value type” is structural rather than a hard-coded name or shape
allowlist: every source-defined enum or non-generic `readonly struct` satisfying
these rules is eligible. It does not mean arbitrary mutable, unsafe, generic,
explicit-layout, or runtime-provided CLR value types.

Structs lower to named structural values. Declaration and field order are
canonical source order, while type declarations are emitted in dependency
order. Every admitted source type and specialized internal semantic value records
`default_eligible`. It is true only when the specification freezes the exact
recursive zero/null value, every nested default is eligible, no member is
required, and the public type invariant holds for that value. A non-null
reference type is ineligible; its nullable form may be eligible through the
internal `none` arm. `default(T)`, an implicit zero-valued struct construction,
and any other publication of a default value reject when this fact is false.
Temporary zero state before an admitted constructor finishes is construction
state and cannot be read or published.

Source may request a default only with `default(ExactAdmittedType)`; the
target-typed `default` literal rejects. The frontend independently expands the
recursive value and checks `default_eligible`; Roslyn constant/default
classification is not the eligibility proof.

### 8.2 Sealed immutable classes

Admit ordinary non-generic `sealed class` declarations as immutable value
objects under these restrictions. Source exception declarations use the
separate closed-hierarchy rule in section 15:

- the compiler-owned implicit base is `object`, and no source base-list syntax
  appears;
- all instance state is in source-declared readonly fields, getter-only
  properties, or admitted init-only auto-properties;
- the complete reachable type and value graph is acyclic and deterministically
  bounded;
- no static mutable state, finalizer, event, indexer, virtual member,
  interface, reflection, monitor, or identity API exists;
- constructors, defaults, and any enclosing initializer together establish one
  final value for every member on every normal construction path;
- `this` does not escape during construction;
- methods do not mutate the receiver or any reachable value; and
- C# class/reference `==`/`!=`, `ReferenceEquals`, `GetHashCode`, runtime type
  inspection, and identity-sensitive collections reject; only section 8.5's
  explicit structural operation compares projected values.

With identity and mutation unobservable, a non-null instance lowers to a
structural VIR value. This is an explicit profile theorem, not an assumption
that all C# classes have value semantics. A later mutable-heap profile would
need a separate heap, alias, frame, and allocation model.

### 8.3 Fields, properties, and constructors

Allow `private`, `internal`, or `public` readonly instance fields and
getter-only or exact init-only properties. An auto-property is accepted only
when its compiler-synthesized backing-field shape is frozen and cross-checked;
an explicit getter must be a pure admitted expression or block. A custom
`init` body, ordinary `set`, lazy getter, cached value, property side effect,
or mutation after the enclosing object-creation expression rejects.
Source-declared static fields, constants, properties, events, and type
initializers reject; a stateless static pure method remains eligible under the
ordinary closed-call rules. Enum members are governed separately by section
8.1 and are not a static-storage exception.

Constructors may take admitted value parameters, delegate to one acyclic
constructor in the same type, assign members, validate arguments, and throw an
admitted exception. Base-constructor behavior is only the inert `object`
constructor. Source-declared instance field/property initializers and static
initializers remain rejected in the first profile; “default” means only the
frozen zero/null value present before constructor assignment. For a type
without init-only members, a normal constructor exit
establishes the public type invariant. For a type with init-only members, every
constructor instead establishes a closed construction invariant and every
complete object-creation expression—including one with no initializer—performs
the finalization step that proves the public invariant. Every exceptional exit
produces no object value.

The exact compiler-synthesized parameterless constructor of a class with no
source-declared constructor may be admitted only when T01's frozen Roslyn shape
has the inert `object` base call and no other operation. Its zero/null member
state must satisfy the public invariant immediately, or the construction
invariant when admitted init-only members are finalized by that same creation
expression. Any other synthesized constructor or compiler-supplied member
initialization rejects.

Before finalization, `this` may be used only as the receiver of the current
constructor's direct member assignment/read or its one admitted same-type
constructor delegation. A direct member read requires that member to be
assigned on every incoming path. Calling an instance method/getter on `this`,
passing or returning it, storing it in another value, comparing it, or capturing
it rejects even if the target method would otherwise be pure. Whole-`this`
assignment and returning a ref to a member also reject. A constructor may call
an admitted static pure helper with arguments that do not contain `this`.

Pure instance methods lower to direct functions with the receiver as the first
argument. Calls are statically resolved to one source declaration; there is no
virtual dispatch.

Every application-declared method/constructor parameter is an exact admitted
closed value passed by value. Invocation arguments are positional, have exact
arity, and use only the separately admitted conversions. Optional/default or
`params` parameters, named arguments, extension syntax, and `ref`, `in`, or
`out` parameters/arguments reject. Constructor delegation and base construction
obey the same argument rule.

An application object-creation expression spells the exact source type as
`new ExactType(...)`. Target-typed `new()`, anonymous-object creation,
reflection/activator construction, and compiler-generated factory or conversion
paths reject even when Roslyn could infer the same destination type.
For a readonly struct, `new ExactType()` invokes its exact admitted
source-declared parameterless constructor when one exists; otherwise it denotes
the recursive zero value and requires `default_eligible`. `default(ExactType)`
never invokes that constructor.

### 8.4 `init`, `required`, and object initializers

Admit `required` only on an admitted init-only auto-property. A required member
has no source declaration initializer and is not assigned by a constructor; its
enclosing object initializer assigns it exactly once. Every `new` expression is
independently checked to assign every required member and to prove non-null and
type-invariant obligations. A compiler success, nullable warning state,
`RequiredMemberAttribute`, or `SetsRequiredMembersAttribute` is not proof;
spelling either attribute in source rejects, and the frontend recomputes the
member set and assignments.

Admit an object initializer only for an admitted non-null immutable class or
readonly struct, and admit only its init-only auto-properties as assignment
targets. C# evaluates the selected constructor first, then initializer
right-hand sides and member assignments once each in source order. MPK retains
that order and treats the under-construction value as one unique local
transaction. It cannot be read, captured, passed, returned, or otherwise
observed until all initializers complete and the final invariant is proved.
Multiple writes to one member across the constructor and initializer, duplicate
initializer targets, nested object/collection initializers, indexer
initializers, and mutation of an already published value reject.

An initializer exception produces no usable result value and follows the
ordinary exceptional edge. Successful completion freezes the instance and all
reachable state. For a creation expression with no object initializer,
finalization occurs immediately after the constructor: a type with init-only
members proves its construction invariant at constructor return and its public
invariant at that finalization point; every other type proves the public
invariant at constructor return.

### 8.5 Structural equality and canonical ordering

The profile derives internal structural equality for admitted immutable
structs, classes, read-only arrays, specialized sequences, ordered maps/sets,
bound closed outcomes, boundary-presence values, and business primitives.
Fields are compared in canonical declaration order; sequence elements and map
entries are compared in canonical order; active sum arms compare their tag and
payload. A null reference is equal only to null. Floating-point equality
retains C# NaN and signed-zero behavior and therefore is not silently changed
into bit equality.

The contract language exposes typed `structural_equal` and
`canonical_compare` expressions; they are not callable `Mpk.*` C# methods.
Selected application code uses admitted primitive/string equality or a
source-defined non-generic pure helper whose field-by-field implementation is
included in the closure. `canonical_compare` is available only to contracts,
boundary canonicalization, and internal collection definitions over the frozen
totally ordered key matrix: integer and Boolean scalars, `char`, ordinal string,
enum carrier, decimal value, date/time/duration/instant, GUID, nullable values
with null first when their payload is orderable, and immutable structural
values whose fields are all orderable. `float`, `double`, their nullable forms,
and structures containing them are excluded from ordering and map/set keys
because NaN prevents the required total order.

Each internal operation is statically specialized for one admitted closed type
and lowers to shared structural terms. This does not admit virtual `Equals`,
`IEquatable<T>`, `IComparable<T>`, caller comparers, user-defined equality or
ordering operators, boxing, `GetHashCode`, or reference equality. The freeze
must assign the exact decimal, GUID, null, and lexicographic ordering vectors.

## 9. Arrays and internal bounded-collection semantics

### 9.1 Arrays and ownership

Admit zero-based, one-dimensional `T[]` where `T` is an admitted immutable
non-construction-state value and the run-time length is within the profile
maximum. An array initializer proves every supplied element's public invariant.
A length-only allocation whose length is nonzero and whose element type is not
`default_eligible` creates a unique construction array with every element
marked uninitialized; the CLR's temporary zero-filled cells are semantically
unobservable. Each cell has exactly one first write that changes its state from
uninitialized to initialized, and that cell cannot be read before this write.
When the element type is `default_eligible`, a length-only allocation instead
creates an immediately complete array whose cells contain that type's frozen
recursive default. A zero-length allocation is immediately complete for every
admitted element type because it has no element invariant to establish.
Until all cells are initialized, only the unique local owner may initialize a
cell or read an already initialized cell; aliasing, calls, storage, return,
wrapper construction, and every other publication reject. A loop may establish
this through a proved initialized-prefix invariant. After complete
initialization, an indexed write is an ordinary rewrite rather than a second
initialization and is permitted only while unique ownership remains; this is
the state in which a proved in-place canonical sort may operate. An abrupt exit
before publication discards the array, and no catch/finally path may observe a
partially initialized value.

The exact admitted creation forms are `new T[length]` with an exact `int`
length, `new T[] { ... }`, a local declaration initializer `T[] x = { ... }`,
and `new[] { ... }` only when Roslyn resolves every element directly to one
identical admitted element type without a nonidentity conversion. Initializer
expressions evaluate once each from left to right. Target-typed collection
expressions, `Array.Empty<T>()`, omitted or non-`int` run-time lengths,
stack allocation, and implicit best-common-type conversion reject. An index is
also exact `int`; `System.Index`, ranges, from-end indices, and implicit index
conversions reject.

This prevents zero-filled arrays of non-null class values, required-member
structs, or invariant-bearing values such as the money template from
publishing invalid elements while still permitting exact-size two-pass
construction. The array itself is a reference at the C# boundary, so
nullability is modeled separately. Multidimensional, jagged, covariant,
`System.Array`, `Span<T>`, `Memory<T>`, and arbitrary collection/interface
conversions reject.

Allowed operations are:

- the exact array-creation and local-initializer forms above;
- `Length`;
- indexed reads of initialized elements with exact `IndexOutOfRangeException`
  behavior;
- `foreach` in element order;
- equality with `null`, but no array reference equality; and
- indexed writes only while a fresh local array has unique ownership.

Parameters, fields, property values, captured arrays, and arrays passed to
another method are read-only. A newly allocated local array begins `unique` and
tracks an exact initialized-element set. The frontend runs a conservative
ownership and initialization analysis: assignment, capture, passing, storage
in another value, or return is allowed only after complete initialization and
then freezes or transfers the array; any later write through a possible alias
rejects. Returning a uniquely owned, completely initialized array transfers
ownership to the result. `ref`, `out`, slices, pointers, and element references
reject.

An active array `foreach` holds a read-only borrow of that array. An indexed
write to the same array during the enumeration rejects even when the local was
unique before the loop; this removes mutation/enumerator-order ambiguity.

VIR models arrays as bounded variable-length sequences, not as unbounded heap
objects. Sequence length and element access are explicit terms; update is a
functional sequence update. Index checks, length bounds, and ownership-state
transitions are regenerated by the importer and VC layer.

Negative or otherwise invalid C# array lengths retain the exact pinned C#
exception edge. The profile maximum is different: an initializer or constant
allocation above it is a deterministic source/limit rejection, while a
symbolic input, allocation, or result length carries a checked bound predicate
and VC. Failure to prove that predicate blocks verified acceptance; it is not
reinterpreted as a C# exception.

### 9.2 Bounded immutable sequences and construction

The internal `bounded_sequence<T>` template is specialized only after `T` is a
closed admitted value. It supplies length, indexed read, structural equality,
lexicographic ordering when `T` is orderable, and the construction-state
relations needed by contracts and VCs. It is not a source type and exposes no
C# assembly or builder API.

The initial source representation is an admitted one-dimensional `T[]`, or an
application-owned non-generic immutable wrapper containing exactly one such
array plus admitted scalar metadata. A wrapper must be selected as an ordinary
source type and may be classified by a semantic-binding sidecar; all of its
constructors, getters, invariants, and helper methods remain verified source.
The binding does not turn the wrapper into an MPK dependency or trust a hidden
implementation.

A fresh local array is the only initial source construction buffer. A
variable-length filtered result uses a deterministic two-pass form: the first
pass proves the exact output count, allocation creates an array of that count,
and the second pass fills each index exactly once in source order before the
array freezes or transfers. The loop records prove count agreement, index
bounds, element invariants, and complete initialization. Single-pass dynamic
growth, `List<T>`, `ImmutableArray<T>`, collection initializers, callbacks,
lambdas, and a source-visible generic builder remain rejected. An
application-owned wrapper remains immutable and does not create a second
mutable-container exception; its array is built under section 9.1 before the
wrapper is published.

Internally, the frontend represents unique allocation, indexed fill, and
freeze as a monomorphic linear sequence-construction state. That state is an
untrusted lowering device, cannot appear in source or at a public boundary,
and is eliminated into concrete sequence operations before certificate
generation.
Failure to prove the profile bound or complete initialization blocks verified
acceptance and never yields a partial sequence.

### 9.3 Canonical ordered maps and sets

The internal `ordered_map<K,V>` and `ordered_set<T>` templates are specialized
only for closed types satisfying section 8.5's total-order matrix. A map is a
bounded sequence of key/value entries in strictly increasing key order; a set
is a bounded sequence of elements in strictly increasing order. Both are
duplicate-free and expose internal count, membership, lookup, and canonical
enumeration relations to contracts and VC generation.

Source represents a set with an admitted element array and represents a map
with an array of an application-owned non-generic immutable entry type. An
optional non-generic immutable wrapper may carry either array. A
semantic-binding sidecar names the exact element or key/value members and
canonical-order invariant; the frontend checks the complete source shape and
the VC layer proves sortedness, uniqueness, bounds, and invariant preservation
at every publication point. Lookup in selected code is an admitted loop or a
source-defined non-generic pure helper whose body is in the closure. A missing
lookup result uses a bound application-owned closed lookup type. An admitted
nullable representation may stand for missing only when the stored value type
is non-null and no admitted value maps to the absent representation; a map that
stores nullable values must use the explicit `lookup<option<T>>` composition
and preserve missing-key versus stored-null arms.

Construction must produce the canonical sorted, duplicate-free array directly
or prove a source-defined closed construction algorithm. No internal template
changes the observable order of a C# collection. `Dictionary<K,V>`,
`HashSet<T>`, `SortedDictionary<K,V>`, framework enumeration, runtime hash
codes, randomized hashing, ambient `Comparer<T>.Default`, caller-provided
comparers, insertion-order assumptions, and generic key/value entry types
remain rejected. The importer recomputes the internal map/set projection and
the VC layer proves every operation over that projection.

## 10. Strings and characters

### 10.1 Ordinal string operations

Model `string` as an immutable bounded sequence of UTF-16 code units, matching
the C# value model. Add `char` as an unsigned 16-bit code unit, not as a Unicode
scalar value. Preserve lone surrogates created by admitted literals or indexing;
do not normalize text.

The first practical profile admits only this closed string operation set:

- ordinary/verbatim literals and escapes whose decoded UTF-16 sequence is
  independently reproduced;
- `Length` and indexing;
- `==`, `!=`, and exact ordinal equality;
- ordinal `Compare`, `StartsWith`, `EndsWith`, and `Contains` overloads with an
  explicit `StringComparison.Ordinal` argument;
- bounded C# `string + string`, `string + char`, and `char + string`, plus the
  exact two-, three-, and four-string `String.Concat` overloads;
- bounded `Substring` with explicit start and length;
- `IsNullOrEmpty`; and
- switch constant matching under exact ordinal, case-sensitive semantics.

Interpolation is accepted only when every interpolation is already string or
char, has no alignment/format component, and normalizes to bounded
concatenation. Direct BCL numeric/date formatting or parsing, case conversion,
trimming, culture, normalization, comparison without an exact ordinal
overload, interning, identity, and arbitrary `System.String` methods reject;
section 10.2's exact codecs belong to the canonical boundary rather than a
source-visible conversion library.

`StringComparison.Ordinal` is one profile-recognized intrinsic constant, not
admission of the framework enum or its other members. It cannot be stored,
returned, converted, or passed anywhere except the exact allowlisted argument
position.

For the admitted `Concat` and `+` forms, a null string operand contributes the
empty sequence and the result is non-null. At least one `+` operand must be a
string; `char + char`, object-converting concatenation, boxing, and implicit
`ToString` calls are not admitted string operations. String equality
distinguishes null from empty and uses the exact C# ordinal value rules;
ordinal `Compare` also preserves the defined null ordering. An instance call
on a null receiver has its
`NullReferenceException` edge. Where an exact chosen overload rejects a null
argument or invalid index/range, it has the corresponding allowlisted C#
exception subtype; overloads defined to accept null retain that behavior. A
result longer than the semantic maximum is a bound obligation, not an invented
runtime exception. Catchable resource exhaustion remains outside the profile.
Limits count UTF-16 units and encoded artifact bytes separately.

### 10.2 Exact boundary parse and format profiles

Add profile-owned, statically resolved parse/format relations used by boundary
sidecars and canonical reproduction. Their working values are internal bounded
text and closed `result<T,parse_error>` instances. They expose no `Mpk.*` C#
API. Parsing produces either a value or a closed parse-error arm rather than
using application `out` parameters or exception-driven control. Formatting
produces bounded non-null text and carries an output-length obligation. These
relations are ordinary checked foundation definitions; calling a BCL
parse/format implementation is only differential evidence.

The first profile freezes exactly these culture-free ASCII grammars:

- signed and unsigned base-10 integers, with `-` only for signed negative
  values and no `+`, whitespace, separators, exponent, or redundant leading
  zero;
- decimal fixed-point text with optional `-` and a canonical integer part. A
  normalized codec omits a zero fractional part and redundant trailing zeros;
  a fixed-scale codec takes an explicit scale 0..28, requires exactly that many
  fractional digits, and formats with an explicit midpoint-rounding mode from
  a closed allowlist;
- `DateOnly` as `yyyy-MM-dd` and `TimeOnly` as one exact 24-hour form with
  exactly seven fractional-second digits, preserving its full 100-nanosecond
  tick range;
- duration and Unix instant as canonical signed integral tick/millisecond
  forms respectively;
- binary32 as exactly 8 and binary64 as exactly 16 lowercase hexadecimal
  IEEE-bit digits without a prefix, preserving signed zero, infinities, and
  every admitted NaN bit pattern; no JSON-number or ambient decimal rendering
  is an exact floating codec; and
- separately named GUID `N` and `D` codecs, each accepting and emitting only
  its exact lowercase canonical spelling.

Each parser distinguishes syntax, noncanonical spelling, range,
scale/precision, and input-bound errors through the closed internal
`parse_error` kind.
Invalid calendar dates, overflow, leading/trailing whitespace, culture-specific
digits, group separators, currency symbols, and alternate calendars return an
error without a partial value. Unknown codec IDs or rounding-mode values reject
during closed sidecar validation and never reach a parser. A formatter
emits one canonical spelling and carries the output-bound obligation described
above; it does not return `parse_error`. Every lossless codec satisfies
`parse(format(value)) = value`; fixed-scale decimal formatting instead parses
to the value produced by its explicit scale and rounding mode. Every
successfully parsed input reformats byte-identically under its matching codec.

General `Parse`, `TryParse`, `ToString`, composite formatting, numeric
interpolation, `IFormatProvider`, `CultureInfo`, custom format strings, and
ambient current culture remain rejected in selected source. An
application-owned parsing or formatting method is accepted only when its body
is in the source closure and uses ordinary admitted operations; a sidecar
cannot replace it with a boundary codec. Boundary codecs in section 18 reuse
the exact relations above rather than define a second conversion meaning.

## 11. Floating-point and decimal numbers

### 11.1 `float` and `double`

Admit `float` and `double` only after a feasibility package freezes exact
binary32/binary64 behavior on the pinned .NET/Linux target. The model includes
normal values, subnormals, infinities, NaNs, signed zero, round-to-nearest
ties-to-even, comparison behavior, arithmetic, and conversions. It does not
replace these values with mathematical real numbers.

Allowed operations are literals, unary sign, `+`, `-`, `*`, `/`, `%`, ordered
comparisons, equality/inequality, explicit admitted numeric conversions, and
exact intrinsics for `IsNaN`, `IsInfinity`, `IsFinite`, `Abs`, `Min`, and
`Max` only after each overload is frozen. No ambient hardware fast-math,
fused operation, extended precision, `Math` transcendental function, generic
math interface, bit reinterpretation, or floating-point/decimal mixing is
allowed.

The VIR value is exact IEEE bits. Signed zero, NaN payload, and the pinned
quieting/result behavior are preserved exactly; canonicalization is forbidden
because section 10.2's bit codec makes every admitted representation
observable. The VC encoder produces ordinary checked core/BV definitions and
proof terms within the current program-certificate alpha acceptance profile.
It emits no theory certificate, floating-point axiom, or checker primitive.

### 11.2 `decimal`

Model .NET decimal as sign, a 96-bit coefficient, and scale 0..28, together
with exact value equivalence, rounding, overflow, division, remainder, and
conversion rules from the pinned runtime. Operations calculate the specified
exact intermediate and round to the nearest representable result, including
the required midpoint rule. Overflow and zero division produce explicit
exceptional edges.

Admit:

- canonical decimal literals;
- unary sign, `+`, `-`, `*`, `/`, `%`, comparison and equality;
- integral-to-decimal and decimal-to-integral conversions;
- exact allowlisted `System.Decimal.Round`, `Truncate`, `Floor`, and `Ceiling`
  overloads; and
- no user-defined conversion, float/double conversion, direct BCL formatting/
  parsing, currency, culture, or representation inspection; section 10.2's
  exact decimal codecs remain available at the canonical boundary.

Trailing-zero scale is normalized only if every admitted observation is proven
value-based. `decimal.GetBits`, formatting, hash codes, and APIs that expose
representation remain rejected. MPK-owned arithmetic is differentially tested
against the exact pinned .NET runtime but is not defined by accepting the
runtime's answer.

An allowlisted `System.MidpointRounding` value is a profile-recognized intrinsic
only in the exact rounding-argument position of an admitted `decimal.Round`
call. Like `StringComparison.Ordinal`, it cannot be stored, returned, converted,
accepted as an application method parameter, or used to admit the framework
enum generally. Application-owned money or codec modes use a source-defined
closed enum and an exhaustive switch whose selected arm invokes one exact
allowlisted rounding operation.

## 12. Nullable, domain outcomes, and business primitives

### 12.1 Nullable values and references

The practical compiler session fixes nullable annotations and warnings to
enabled. Apart from section 7.3's exact redundant file-wide enable form, source
directives that select or change either setting reject. Roslyn nullable flow
analysis remains a diagnostic observation, not proof. The compilation retains
the scalar profile's fail-closed policy: every active compiler error or warning
ends the source/metadata phase with no artifacts, while informational and
hidden records follow one frozen allow/ignore table. Warning-free compilation
is necessary but never substitutes for MPK's independently generated null and
invariant obligations.

Represent internally:

- `T?` for an admitted non-nullable value type as a specialized `option<T>`;
- every string, array, and admitted class reference as an optional structural
  value at the semantic boundary; and
- source annotations as intent that determines required contracts and
  diagnostics, not as a runtime guarantee.

A non-nullable reference parameter must have an explicit `not_null`
precondition. A non-nullable result or field must be proven non-null on every
normal exit. Nullable references may use `is null`, `is not null`, `== null`,
and `!= null`. Nullable value types may use `HasValue`, `Value`,
`GetValueOrDefault`, lifted equality/comparison/arithmetic in a frozen closed
matrix, and matching patterns. Parameterless `GetValueOrDefault()` is admitted
only when the payload type is `default_eligible`; the overload with an explicit
fallback instead proves that fallback's public invariant.

Conditional access is limited to `receiver?.Member`, where `receiver` is one
nullable admitted application-class reference and `Member` is one directly
resolved source field or total pure parameterless getter returning a
non-nullable admitted reference type `U`. The receiver evaluates once; the
absent branch produces the nullable-reference form `U?`, and the present branch
performs the same member read as an explicit null test. A value-type or already-
nullable member result, chained conditional access, conditional invocation,
array/indexer access, or extension access rejects. Null coalescing is limited
to `left ?? right`, where `left` has the exact value- or reference-nullable
form `T?`, `right` has the exact non-nullable payload type `T` without a
conversion, and the result is `T`; it evaluates `left` once and evaluates
`right` only on the absent branch. `??=`, a throw operand, a user-defined
conversion, and every other lifted result shape reject.

The initial nullable-value construction forms are exactly `null` or
`default(T?)` for `none`, and the built-in implicit conversion from one exact
`T` expression to `T?` for `some(T)`. `new T?(...)`, target-typed `default`, an
explicit nullable cast used only as a construction shortcut, and every
reflection or helper construction reject. Separately admitted lifted operators
and conversions remain governed by their frozen matrix; this rule does not
authorize another constructed-generic source form.

Dereference or `Value` access without a proven present branch emits the exact
null/invalid-operation exceptional edge. The null-forgiving operator `!` is
accepted only if MPK's own dataflow already proves non-null and therefore it
has no semantic effect; otherwise it rejects. The frontend accepts only the
`T?` source spelling frozen in section 6.2, verifies the exact compiler symbol,
and specializes it before VIR emission. Repeated nullable encodings and
`option<option<T>>` reject; `lookup<option<T>>` is the one intentionally
admitted lookup-versus-null composition described in section 12.2. Other
tagged-sum nesting follows the explicit depth limit. Nullable by-reference
storage rejects.

### 12.2 Closed outcomes and accumulating validation

The foundation bundle defines internal `option<T>`, `lookup<T>`,
`result<T,E>`, and `validation<T,E>` templates. Source does not name those
templates. Apart from value-type nullable, an application that requests one of
these outcome roles uses a non-generic immutable struct or sealed class and a
mandatory semantic-binding sidecar naming its exact tag and payload members.
The same source shape may remain an unbound ordinary structural value when no
outcome role is requested. Payloads must be admitted closed immutable values,
subject to section 12.1's nested-option exclusion. The frontend validates the
source declaration and constructors and then emits one closed specialization
under section 6.2.

`option<T>` is `none` or `some(T)`. `lookup<T>` is `missing_key` or `found(T)`;
unlike nested option, `lookup<option<T>>` is admitted so a map can distinguish
a missing key from a stored nullable value. `result<T,E>` is `ok(T)` or
`error(E)`. `validation<T,E>` is `valid(T)` or
`invalid(bounded_sequence<E>)`, where the error sequence is nonempty, bounded,
and kept in source/evaluation order. Application tag values and payload fields
map one-to-one to these internal arms. Construction, tag tests, guarded
active-payload reads, structural equality, property/tag patterns, and
exhaustive switch are admitted from the original source.

Reading a bound inactive payload in selected code has the exact
`InvalidOperationException` edge required by the application declaration, or
rejects when the declaration exposes no such operation. Constructing a bound
invalid-validation arm has the checked normal condition `error_count > 0`; its
source constructor must either enforce the declared `ArgumentException` edge
or accept only callers that prove the condition. A sidecar cannot invent that
check, exception, or source path.

The frozen recursive defaults for internal option and lookup are `none` and
`missing_key`; a bound source type is `default_eligible` only if its actual
default value maps to that arm and satisfies its public invariant. Result and
validation are not `default_eligible` in this profile and require explicit
construction. A future profile may choose a different mapping only under a new
reviewed identity; it cannot widen this one. No inactive source payload becomes
unobservable or satisfies an invariant merely because a binding classifies
the type.

The profile supplies no source-visible lambda-based `Map`, `Bind`, query
syntax, implicit conversion, exception coercion, or hidden short circuit. Code
combines application-owned outcomes with ordinary branches and switches.
Validation accumulation uses admitted arrays and loops, appends left errors
before right errors, and proves the combined bound; it never drops, sorts, or
deduplicates errors implicitly. Expected business rejection uses these closed
application values, while exceptions remain for the exceptional paths declared
in section 15.

### 12.3 Date, time, duration, instant, GUID, and money

Admit a closed operation subset over pinned `System.DateOnly`,
`System.TimeOnly`, `System.TimeSpan`, and `System.Guid`, plus an internal
Unix-millisecond instant semantic value. The framework metadata types are
explicit practical-foundation intrinsics: their runtime implementations remain
untrusted observations and do not authorize other framework members. Source
represents an instant either as an application-owned non-generic immutable type
bound to that carrier, or as an exact signed 64-bit field that a boundary or
transition sidecar explicitly classifies for transport and contracts. An
unclassified integer remains an ordinary integer. The raw-carrier form gains
no source-visible instant operation; selected logic that needs instant-specific
addition, subtraction, or error outcomes uses a bound application type. Neither
form references an MPK type.

- `DateOnly` uses the proleptic Gregorian calendar and the exact pinned .NET
  range. Construction, year/month/day/day-number access, comparison, day-of-
  week through the exact closed `System.DayOfWeek` enum, and bounded
  `AddDays`/`AddMonths`/`AddYears` are admitted with exact range exceptions.
  That enum has exactly the seven pinned named values and the corresponding
  carriers; numeric casts, arithmetic, flags behavior, and other `System.Enum`
  APIs remain rejected.
- `TimeOnly` is a time of day represented by 100-nanosecond ticks in one day.
  Components, comparison, subtraction to duration, and the exact wrapping
  behavior of the value-returning, no-`out` duration-addition overload are
  admitted. Overloads that report day carry through `out`, and every `ref` or
  `out` form, reject.
- `TimeSpan` is the signed 64-bit 100-nanosecond duration carrier. Exact
  construction, components, comparison, and checked add/subtract/negate are
  allowed; scaling requires a separately frozen exact profile operation.
- The internal instant value is a signed 64-bit Unix-millisecond UTC instant.
  A bound source type must expose the exact signed carrier and prove its
  invariant. It admits comparison. Duration addition/subtraction returns an
  application-owned closed outcome containing either an instant or an error
  when the duration has non-millisecond ticks or the result is out of range.
  Instant difference likewise returns an application-owned closed outcome:
  its success arm contains the exact duration, and its error arm is selected
  when the millisecond difference cannot be represented as signed 64-bit
  100-nanosecond ticks. It cannot create a sub-millisecond remainder because
  both operands are millisecond instants. The freeze must fix both outcome
  bindings, the internal error arms, and precedence. It has no local-time,
  timezone, calendar, leap-second, or clock lookup behavior.
- `Guid` is an exact 128-bit identifier with `Empty`, equality, the frozen
  .NET comparison order, and the `N`/`D` codecs in section 10. It has no
  `NewGuid`, random source, byte-layout reinterpretation, or ambient generator.

Business money is an internal checked template, not a magical scalar or an MPK
C# class. Source uses an application-owned non-generic `readonly struct`
containing a decimal amount and an exact currency enum or validated ordinal
code. Its binding is not `default_eligible`; an application-owned closed create
outcome validates currency and scale. The source implementation exposes
amount/currency access, checked same-currency addition and subtraction,
multiplication or division by an explicit decimal quantity/rate with target
scale and rounding mode, and a same-currency amount comparison. Every fallible
operation returns a bound application-owned closed error outcome rather than
hiding an expected business failure in an exception. The freeze fixes the
internal template, required source-shape obligations, error arms, and
precedence, including invalid currency/scale, currency mismatch, division by
zero, and decimal overflow.

Structural equality observes currency and decimal value. The internal
canonical storage order places currency code first and amount second; it is not
an economic comparison across currencies and is not exposed as an `Mpk.*`
source call. Implicit rounding, ambient currency metadata, exchange-rate
lookup, and culture formatting reject. The runnable pricing example must use
and prove an application-owned concrete type against this template without an
MPK source or binary reference.

`DateTime.Now`, `DateTime.UtcNow`, local timezone conversion, daylight-saving
rules, `TimeZoneInfo`, non-Gregorian calendars, NTP/system-clock claims, and
database-generated timestamps remain outside the profile. A caller supplies
the effective date/instant explicitly through the boundary contract, and proof
establishes only logic over that value—not its real-world authenticity.

## 13. Loops and loop contracts

Admit `while`, `do`, `for`, and `foreach`, plus structured `break` and
`continue`. `goto`, labels, unsafe jumps, and irreducible control flow remain
rejected. `foreach` accepts admitted arrays and the exact compiler-recognized
string enumeration form. Application-owned wrappers, framework collection
interfaces, maps/sets, generic key/value entries, and source iterators are not
enumerable in the initial profile; selected code traverses their underlying
admitted arrays explicitly.

Every loop receives a canonical ID by method ID and source-order ordinal, for
example `MethodId#loop#0000`. Its method sidecar supplies:

- one or more invariants;
- the values modified by the loop;
- zero or more decreases expressions;
- whether the containing function claims partial or total termination.

The frontend emits a reducible cyclic CFG with one canonical header per loop.
The importer recomputes loop headers, backedges, modified variables, and
contract attachment. VC generation proves invariant initialization,
preservation, each structured exit, and the method postcondition. For total
termination it also proves a well-founded lexicographic decrease and
non-negativity. A partial function may omit decreases but its evidence must
remain visibly partial; it cannot support a total-termination property.

Simple counter/length invariants may be generated as candidates for user
convenience, but generated clauses are untrusted and become obligations exactly
like user clauses. MPK never assumes a compiler or AI-generated invariant.

Every boundary or transition entrypoint, every runnable example entrypoint,
and every method advertised by an installed practical-profile route claims
total termination. Its complete reachable call closure must also be total, and
each cyclic CFG region must discharge the frozen decreases obligations; a
partial method cannot be called on any such path. Partial correctness remains
available only for an explicitly analysis-only selected root whose evidence and
API result are labeled partial and which is ineligible for boundary,
transition, example, or production-route registration.

The contract expression language gains bounded sequence quantifiers so useful
properties such as “all elements before index `i` satisfy P” can be stated.
Quantifiers range only over an explicit bounded sequence interval; arbitrary
unbounded quantification and triggers are not admitted.

## 14. Switch and pattern matching

Admit switch statements and switch expressions over admitted scalar, enum,
string, nullable, bound closed outcome/boundary-presence, business-primitive,
and immutable structural values. Evaluate the governing expression once and
preserve first-applicable-arm and guard order.

The closed initial pattern set is:

- constant, discard, and `var` patterns;
- `null` / `not null`;
- relational and parenthesized patterns;
- `and`, `or`, and `not` logical patterns;
- declaration/type patterns over the finite admitted sealed type set;
- exact tag/property patterns for application-owned types bound to internal
  option/lookup/result/validation/boundary-presence sums;
- property patterns whose getters are total and pure; and
- bounded list patterns over admitted arrays after exact sequence semantics are
  frozen, excluding slice (`..`) subpatterns.

Positional patterns, arbitrary `Deconstruct`, recursive open hierarchies,
extension access, dynamic patterns, and user-defined equality reject. Pattern
variables become explicit block parameters. An expression switch must be
exhaustive or have an explicit modeled exception path. Statement fall-through
remains prohibited; `goto case` and `goto default` reject.

Because the C# specification does not define an observable property-access
order for every pattern form, admitted property getters must be total, pure,
and immutable. The frontend nevertheless emits one canonical decision graph
and freezes Roslyn pattern/CFG observations in conformance vectors.

## 15. Exceptions and abrupt completion

### 15.1 Exception value model

Introduce a closed exception sum containing:

- `DivideByZeroException`;
- `OverflowException`;
- `IndexOutOfRangeException`;
- `ArgumentException` and `ArgumentOutOfRangeException`;
- `ArgumentNullException`;
- `InvalidOperationException`;
- `NullReferenceException`;
- the non-exhaustive switch exception chosen by the pinned target; and
- source-declared sealed exception types admitted under the same immutable
  payload rules as section 8.

An admitted exception exposes only its exact type and explicitly declared
immutable payload. Message text, stack trace, target site, data dictionary,
inner exception, runtime-generated wording, identity, serialization, and
mutable properties are unobservable. Resource-exhaustion and process/runtime
exceptions are outside the semantic model and cannot be caught in admitted
source.

### 15.2 Throw, catch, and finally

Admit a standalone `throw new ExactException(...)` statement, propagation from
an admitted operation/call, ordered typed catches, a pure Boolean filter, a
bare `throw;` rethrow directly within its active catch, and `finally`. Throw
expressions, `throw null`, a stored/reused exception object, and rethrow outside
that exact catch context reject. Catch matching uses the closed exception
hierarchy. A catch variable may read only admitted immutable payload.

A source-declared exception is sealed and derives directly from
`System.Exception`; only profile-declared constructors and immutable payload
are observable. Its base-constructor call is exactly the parameterless
`System.Exception()` call, implicit or explicit. An explicitly thrown built-in
exception likewise uses only the exact parameterless constructor in the
initial profile. Message-, parameter-name-, inner-exception-, serialization-,
and runtime-state constructor overloads reject because their state is outside
the closed exception value. Handler search evaluates typed catches and filters
in lexical order before unwinding the selected path. An outer filter can
therefore run before an intervening inner `finally`. If evaluation of a filter
throws an admitted exception, that filter is false, the original exception
remains the search subject, and search continues. The filter exception is
represented in the decision graph and cannot disappear from conformance
evidence.

The CFG has explicit abrupt outcomes:

```text
normal | return(value) | break(loop) | continue(loop) | throw(exception)
```

Every potentially throwing instruction has distinct normal and exceptional
successors. After selecting a handler, the runtime unwinds from the throw site
to that handler and executes the `finally` regions of frames it exits,
inner-to-outer, before entering the catch. A `finally` belonging to the
selected try/catch statement instead runs when that statement later completes.
Normal completion of `finally` preserves the incoming completion, while a
thrown exception replaces it. Source control that would leave a `finally` with
`return`, outward `break`/`continue`, or `goto` rejects. The importer
reconstructs region nesting, filter search, unwind, and handler order rather
than trusting compiler CFG or lowering output as proof.

The existing scalar `panic = forbidden` meaning is unchanged. Practical
contracts instead declare an exact `throws` set with exceptional
postconditions. An empty set requires every throw path to be caught or proven
unreachable. Evidence reports normal postconditions, exceptional
postconditions, exception freedom, and termination separately.

## 16. Iterator exclusion

The initial practical profile rejects iterator declarations, `yield return`,
`yield break`, non-generic and generic `IEnumerable`/`IEnumerator`,
`IAsyncEnumerable<T>`, `IAsyncEnumerator<T>`, manual enumerators, and iterator
state machines. These forms require source-visible framework enumeration
protocols and lazy suspension/disposal semantics that are unnecessary for the
selected pure-core boundary.

This exclusion does not remove ordinary `foreach` over admitted arrays or the
exact string enumeration form in section 13. A source transformation such as
filtering or projection uses admitted loops and an exactly sized output array;
it does not require a lazy producer. An application adapter may use iterators
outside the selected root and materialize a bounded canonical array before
invoking verified logic.

A future iterator profile would require a separately frozen source-generic
exception, producer lifetime model, disposal semantics, contracts, artifacts,
limits, and upgrade vectors. It cannot be enabled by reinterpreting this
profile or by treating framework metadata as an internal semantic template.

## 17. Async exclusion

The initial practical profile rejects `async`, `await`, `async void`, `Task`,
`Task<T>`, `ValueTask`, `ValueTask<T>`, task factories, custom awaiters,
continuations, cancellation, scheduler observation, and generated async state
machines inside the selected source closure. Non-generic `Task` is also
excluded: retaining it would preserve nearly all suspension complexity while
providing no result-bearing benefit to a pure synchronous core.

Database, network, filesystem, timer, and other orchestration code remains in
an unverified application adapter. The adapter awaits external work, converts
its outputs into bounded canonical application values, calls a synchronous
selected method, and resumes ordinary application processing. The certificate
proves only that synchronous method; it does not assert that the adapter called
it, awaited the correct operation, or preserved an external effect.

A future async profile requires a separate semantic identity and explicit
generic-source, effect, suspension, exception, cancellation, and temporal
design. No task wrapper, task-result erasure, await-site record, scheduler
claim, or async state enters this profile's VIR, contracts, manifests, VCs, or
certificates.

## 18. Business boundary, serialization, and state transitions

### 18.1 Versioned boundary values

Define a closed `mpk.csharp.boundary.v1` sidecar for each public verified-core
entrypoint. It names the semantic context, schema/version, selected method,
ordered input/output fields, admitted value types, maximum document/value
sizes, and the exact parse/format profile. Missing and explicit null are
different states. Duplicate names, unknown fields, unknown enum values,
noncanonical number/text spellings, excess depth/count/bytes, and a schema or
method mismatch reject before the value reaches the verified method.

Where business logic must distinguish omission from explicit null, admit an
application-owned non-generic closed type bound to internal
`boundary_field<T>` semantics with exactly `missing`, `null`, and `value(T)`
arms. `T` is a non-null admitted immutable payload. A required field rejects
`missing`, and a non-null target rejects `null`. An optional field either
exposes all three arms or declares one exact typed value used only for
`missing`. That value is captured canonically in the sidecar and must satisfy
the target invariant. `value(T)` always supplies its payload; explicit `null`
remains distinct and either maps to nullable `none` or rejects for a non-null
target. No implicit missing/null collapse is allowed, and this boundary-
specific sum does not open general nested option types. Construction, tag
tests, active-payload access, matching, and inactive-payload exceptions follow
the closed sum rules in section 12.2. The source type is `default_eligible`
only if its actual default maps to `missing` and satisfies its invariant.

Each MPK verification or reproduction run starts from one canonical boundary
document. This document is a verification-overlay transport, not a required
application API, production message format, serializer, stored type, or
deployment file. MPK captures its exact bytes, independently parses it into
the application-owned typed value, hashes both the byte identity and canonical
value, and binds them into the manifest/evidence chain before the internal
verification wrapper invokes the original selected method.

An MPK-side capture adapter may start from some other JSON/media bytes, but it
must produce and record the canonical document before the verified invocation.
The original byte/provenance identity remains separate, and the certificate
does not prove that this untrusted translation preserved external meaning.
Supplying an adapter-created object while bypassing MPK's canonical byte parser
rejects from the verification route. Output follows the reverse rule: the
verification overlay encodes the returned application value, reparses the
canonical bytes, and checks that they denote exactly that value before
retaining reproduction evidence. The external company's existing production
adapter may remain unchanged and completely unaware of MPK; it stays outside
the certificate unless a separate profile later verifies that adapter itself.

The initial canonical JSON projection uses UTF-8, rejects duplicate/unknown
members, fixes member order in output, and reuses section 10 codecs for decimal,
binary floating-point, date/time, duration, instant, and GUID values. A map is
an ordered array of typed key/value entries rather than a JSON object with
string-coerced keys. String encoding operates on UTF-16 units: a lone surrogate
must use a canonical `\uXXXX` escape so parsing reconstructs the exact admitted
string. The freeze fixes escape case and all other escaping choices as well as
missing/null handling, enum spelling, integer range, string unit count, option/
lookup/result/validation/boundary-presence/transition tags, and each scalar's
JSON token kind. Reflection-driven
serialization, arbitrary attributes, runtime type names, culture, host paths,
credentials, and serializer exception text do not enter proof or public
evidence.

Authentication claims, authorization roles, tenant IDs, database rows,
optimistic-concurrency versions, and current time may enter only as explicit
typed fields with checked preconditions and recorded provenance identities.
The certificate proves the function of those values. It does not prove that an
identity provider, database, transport, clock, or operator supplied truthful or
fresh data.

### 18.2 Pure business state transitions

Standardize the practical state-machine shape as a pure selected method:

```text
Apply(State, Command, Context)
  -> ApplicationOwnedApplyResult
```

The result is an application-owned non-generic immutable type bound to
`result<transition<State,Event,Response>,DomainError>`. Its success payload is
another application-owned concrete type bound to internal transition semantics
and contains exactly the new state, an immutable bounded event array, and the
response. `State`, `Command`, `Context`, `Event`, `Response`, and `DomainError`
are admitted application-owned immutable values. No source signature names an
MPK type. The contract names the state invariant, exact accepted command cases,
expected state version, command/idempotency identifier, explicit effective
date/instant, normal transition postconditions, ordered emitted events,
response relation, and every business-error result. A newly applied successful
command proves the new invariant, the frozen version-increment rule,
event/response correspondence, and all collection bounds. A rejected command
leaves the input state unchanged.

Idempotency is an optional claim. It is admitted only when the explicit input
state contains a bounded processed-command record with the key, complete
application-owned `Command` and `Context` snapshots, and the response. A
source-defined, field-complete equality helper for those snapshots remains in
the selected closure and must be proved equivalent to equality of their exact
canonical boundary encodings. The transition contract cannot select a smaller
caller-provided projection. This ensures that an effective date, tenant,
authorization fact, or any other explicit context value cannot change
unnoticed under a reused key.

Every snapshot field must have reflexive admitted equality; a `float`,
`double`, or recursively containing type is therefore ineligible for the
initial idempotency claim because C# NaN equality is not reflexive. Missing and
null remain distinct through their bound arms. After ordinary boundary
preconditions, a retained key is checked first: field-complete equal snapshots
return the current unchanged state, no new events, and the stored response;
different snapshots return an explicit idempotency-conflict error. The MPK
overlay records their canonical encodings for evidence, but an encoding digest
never substitutes for source equality or adds a collision-resistance
assumption. A project that does not have this processed-record shape may still
verify the transition without claiming idempotent replay. A new key then checks
`expected_version`; a mismatch is an explicit optimistic-concurrency error
rather than an inference from a future database write. The initial profile
performs no implicit history eviction: a new command at full history returns a
specified capacity error.
Event ordering is source order and deterministic. Hidden current time, random
IDs, mutable aggregate identity, ambient tenant context, and implicit retry
behavior reject.

An adapter may load and atomically persist state/events outside the verified
root, but transaction isolation, durability, locks, retries, message delivery,
and exactly-once effects are not certificate claims. The release documentation
must show this boundary so application teams do not mistake a proved pure
transition for proof of the surrounding infrastructure.

## 19. Contract design

Use separate closed sidecars for types and callable members.

A type contract contains exactly its schema/profile/type identity, ordered
field/property identities, frozen recursive default, `default_eligible` fact,
required/init membership, construction invariant when applicable, public
invariants, and structural equality/ordering eligibility. A constructor for a
type without init-only members proves the public invariant on normal return. A
constructor for a type with init-only members proves the construction invariant;
its enclosing creation expression proves the public invariant at finalization,
immediately after the constructor or after the last initializer assignment.
Every instance method may assume the public receiver invariant and must preserve
it because receiver mutation is forbidden.

A method/constructor contract contains:

- ordered `requires` and normal `ensures`;
- ordered exceptional cases with an exact exception type, path condition, and
  exceptional postconditions;
- an empty externally visible `modifies` set; owned object construction and
  local array construction updates are represented in construction/loop
  records but cannot escape under an alias;
- partial/total termination;
- ordered loop records with invariants, modifies, and decreases.

The expression union adds typed field/property access, sequence length/index,
map/set lookup and membership, string/char, float, decimal, enum,
date/time/duration/instant/GUID, specialized internal
option/lookup/result/validation/boundary-field/transition construction and
tests, source-binding identity, structural equality/order, parse-error kind,
exception kind and payload, and bounded `forall`/`exists`. Every expression is
closed, explicitly typed, depth/count bounded, and free of source method calls.
There is no arbitrary C# expression evaluator inside a contract parser and no
contract expression can make an internal template callable from application
source.

A boundary contract relates captured input identity to one canonical typed
value but makes no service-truth claim. A transition contract additionally
names its state invariant, version/idempotency relation, result arms, response
relation, and event sequence relation. These clauses generate ordinary VCs;
neither sidecar can assert serializer correctness, authentication, persistence,
or delivery as an assumption hidden from the axiom report.

Contracts cannot hide an external service assumption. A future effect-summary
mechanism would need an explicit axiom/theory policy and is outside this
profile.

## 20. VIR and VC design

### 20.1 Required value vocabulary

The successor VIR needs closed values for:

- Bool and existing fixed-width bit vectors;
- IEEE binary32/binary64 bits;
- .NET decimal value representation;
- option/presence and map lookup presence;
- closed tagged result and validation sums;
- closed missing/null/value boundary presence;
- bounded variable-length sequence;
- canonical ordered map/set values represented by sorted duplicate-free
  entries;
- the closed transition state/events/response product;
- named immutable struct/class projection;
- date-day, time/duration tick, Unix-millisecond instant, and GUID carriers;
- closed exception values; and
- unit for void-like internal continuations.

Enums lower to their exact integer carrier plus a declaration identity.
Strings are a distinct profile mapping over a UTF-16 sequence so arbitrary
sequence operations cannot masquerade as string semantics.

Every value and operation type in VIR is monomorphic. VIR has no node for a
type parameter, generic definition, generic arity, constraint, variance,
constructed CLR type, or generic call. A specialized internal type ID is
derived from its registered template ID and canonical concrete argument IDs;
the argument IDs are provenance for recomputation, not variables available to
VIR evaluation. Application source type IDs and semantic-binding IDs remain
distinct and are related explicitly by the source manifest.

### 20.2 Required operation/control vocabulary

Add explicit construction-state, field, source-binding projection,
option/lookup/result/validation/boundary-presence/transition, sequence
construction, ordered-map/set, structural equality/order, boundary
parse/format, business-value, floating-point, decimal, and exception
operations. Linear sequence-construction state is explicit in VIR and cannot
cross a merge without identical ownership/version state; publication lowers it
to an immutable value. Each operation has one shared meaning and a profile
allowlist; C#-specific behavior that cannot share that meaning remains in a
C#-named operation/profile rule.

Exceptional control is explicit rather than encoded as a missing safety check.
Unchecked access is never implied by the presence of a catch. For an uncaught
exception forbidden by contract, VC generation proves the exceptional edge
unreachable. For an allowed or caught exception, VC generation proves the
corresponding handler or exceptional postcondition.

Loops continue to use cyclic CFGs and explicit loop contracts. There are no
iterator, async, task, await, suspension, scheduler, or continuation states in
the source projection or VIR for this profile.

### 20.3 Certificate encoding

Encode option/lookup/result/validation/boundary-presence/transition, immutable
records, bounded sequences, ordered maps/sets, business primitives, canonical
boundary-codec relations, float, decimal, and closed exception outcomes using
ordinary checked core terms and definitions over the existing Bool/BV/array/
struct foundations. The successor program-assembly profile required by
section 5.1 must preserve the acceptance rules already carried by the installed
`mpk.program_certificate.alpha.v1` profile from
`PROGRAM_CERTIFICATE_ALPHA_V0.md`: the root retains `proof_node_table: []` and
`theory_certificates: []`, contains no `TheoryPrimitive` declaration or
`Theory` node, and has a recomputed total axiom count of zero. Untrusted
generators may optimize construction, but both checkers receive the same
complete ordinary-term Certificate v0 bytes and recompute every declaration
and axiom report.

Semantic-template specialization and application-type projection are
untrusted generation steps. Each expanded instance contributes ordinary
concrete declarations and proof obligations; neither the certificate nor a
checker may appeal to a generic theorem, runtime library body, sidecar claim,
or hidden template axiom. The VIR importer rejects a missing, extra,
misordered, duplicate, incorrectly identified, over-limit, or still-generic
instance and independently recomputes the manifest-to-VIR binding relation.

If float/decimal/collection/business-value/boundary-codec/state-transition
definitions exceed practical checker limits, the phase stops. A separate,
prior governance/checker project would be required to change this acceptance
boundary, and this profile cannot activate while such a dependency is
unresolved. It must not add a `CSharpSemanticsAxiom`, theory shortcut, trusted
primitive, runtime answer, or omitted obligation.

## 21. Provisional deterministic limits

The specification-freeze probes must replace this table with exact measured
limits. These provisional ceilings bound the intended design and are not
accepted values:

| Counter | Proposed inclusive ceiling |
| --- | ---: |
| Source-defined data/exception types per compilation | 128 |
| Fields/properties per type | 32 |
| Constructors per type | 8 |
| Structural type nesting | 16 |
| Array elements per run-time value | 4,096 |
| Sequence-construction capacity per value | 16,384 |
| Array/sequence construction states per method / simultaneously live | 32 / 8 |
| Ordered map/set entries per value | 4,096 |
| Total collection cells represented by one request | 65,536 |
| UTF-16 units per string value | 16,384 |
| Option/lookup/result/validation/boundary-presence nesting | 16 |
| Validation errors per result | 256 |
| Application semantic bindings per compilation | 128 |
| Projection obligations per semantic binding | 64 |
| Distinct closed semantic instances per compilation | 256 |
| Closed semantic-instance nesting | 16 |
| Specialized declarations / operations per compilation | 1,024 / 4,096 |
| Boundary fields / nesting / canonical bytes | 256 / 32 / 1,048,576 |
| Events emitted by one transition | 4,096 |
| Loops per method / nesting | 32 / 8 |
| Invariants plus decreases per loop | 64 |
| Switch arms per method | 256 |
| Pattern nesting | 16 |
| Catch/finally regions per method | 32 |
| Source exception types per compilation | 32 |
| Bounded-quantifier nesting | 4 |

The shared sequence-construction ceiling accommodates the largest admitted
string construction. Publication still proves the narrower value-specific
ceiling: an array or ordinary sequence cannot exceed its 4,096-element bound,
and map/set, validation-error, and transition-event values retain their own
rows above. Reaching the lower-level construction ceiling therefore never
widens a published value's semantic limit.

Existing source, closure, operation, CFG, contract, diagnostic, artifact, and
process limits remain ceilings unless the freeze provides reproducible memory,
time, and output evidence for a changed successor limit. Structural and
transport counters reject boundary-plus-one before allocating or retaining the
excess. Untrusted boundary documents and parser inputs follow their specified
pre-invocation rejection or `parse_error` paths. Within admitted source
execution, run-time array, string, sequence-construction, map/set,
validation-error, and transition-event maxima instead become explicit value
predicates and VCs; an unproved bound blocks verified acceptance and never
invents a source-language exception. Semantic bindings and their transitive
closed-instance expansion are counted and rejected before VIR retention. All
counter arithmetic is checked, and semantic value limits remain distinct from
encoded byte limits.

## 22. Diagnostics and fail-closed precedence

Do not reuse a scalar-v0 code with a broader meaning. The practical profile
needs its own closed families, provisionally:

```text
CSHARP_PRACTICAL_DECLARATION
CSHARP_PRACTICAL_TYPE
CSHARP_PRACTICAL_DEPENDENCY
CSHARP_PRACTICAL_GENERIC
CSHARP_PRACTICAL_SOURCE_BINDING
CSHARP_PRACTICAL_OBJECT
CSHARP_PRACTICAL_INITIALIZER
CSHARP_PRACTICAL_OWNERSHIP
CSHARP_PRACTICAL_ARRAY
CSHARP_PRACTICAL_COLLECTION
CSHARP_PRACTICAL_ORDER
CSHARP_PRACTICAL_STRING
CSHARP_PRACTICAL_PARSE_FORMAT
CSHARP_PRACTICAL_FLOAT
CSHARP_PRACTICAL_DECIMAL
CSHARP_PRACTICAL_NULLABLE
CSHARP_PRACTICAL_RESULT
CSHARP_PRACTICAL_BUSINESS_VALUE
CSHARP_PRACTICAL_LOOP_CONTRACT
CSHARP_PRACTICAL_SWITCH
CSHARP_PRACTICAL_PATTERN
CSHARP_PRACTICAL_EXCEPTION
CSHARP_PRACTICAL_BOUNDARY
CSHARP_PRACTICAL_TRANSITION
CSHARP_PRACTICAL_EFFECT
CSHARP_PRACTICAL_FOUNDATION
CSHARP_PRACTICAL_LOWERING
```

Capture/source/metadata/typecheck failures precede subset failures; subset and
contract validation precede lowering; lowering precedes emission. Dependency
validation precedes generic-shape validation, which precedes semantic-binding
validation and transitive closed-instance expansion. Within a method, validate
declarations/types, construction state, ownership, ordinary operations,
control contracts, exceptions, then artifact mapping. Boundary
shape/size/canonicality validation precedes typed conversion and any selected
method launch. The normative vector must freeze every ambiguous multi-failure
case.

The generic family distinguishes user generic declarations, generic methods,
non-allowlisted constructed framework types, explicit `System.Nullable<T>`, open
arguments, unsupported nullable payloads, specialization depth/count, and a
generic value surviving the VIR barrier. The dependency family distinguishes
an MPK source/package/assembly/attribute/generated-code reference from another
unsupported ambient reference. The foundation family distinguishes an unknown
descriptor/schema/member/template/operation identity, semantic-context
mismatch, noncanonical inventory, content-hash mismatch, dependency mismatch,
over-limit expansion, and an unregistered or caller-supplied bundle. The
source-binding family distinguishes unknown source identity, invalid role,
shape mismatch, tag/payload mismatch, unproved invariant, dependency-closure
mismatch, canonical-ID mismatch, and manifest/VIR disagreement. No public
diagnostic includes a customer namespace or member spelling.

Public messages remain bounded and sanitized. Raw compiler prose, source
snippets, exception messages, host paths, generated type names, culture, and
runtime stack text never enter public artifacts.

## 23. Conformance and verification strategy

The freeze and implementation gates must include:

| Capability | Required evidence |
| --- | --- |
| Expression bodies / `var` / name binding | same normalized VIR and obligations as the explicit block/type form; ordinary namespace `using` changes spelling only; exact redundant file-wide `#nullable enable`; alias/static/global/project-generated imports, other directives, malformed, and ambiguous forms reject |
| Source dependency / generics / binding | an unchanged application build with no MPK reference; rejected MPK namespace/package/assembly/attribute/generated-source dependencies; every user generic declaration/method and arbitrary constructed framework type rejects; exact `T?` exception; an allowed intrinsic remains allowed despite incidental generic metadata while any source-visible use of that metadata rejects; binding shape/tag/payload/invariant checks; total, arm-distinct, observation-preserving projection round trips and operation commutation; logical declaration identity versus source provenance, collision/staleness mutations, exact foundation descriptor/hash and tampering, transitive specialization closure, canonical identity, sorting, deduplication, limits, tampered manifest, and residual-generic rejection |
| Data types / construction | positive structural cases, exact implicit object/value-type/enum bases, admitted enum-underlying and source-exception base clauses, rejected other source base lists, compiler-owned init/required markers with rejected marker access and inherited-member calls, enum unknown/cast/zero-default cases, recursive default-eligible/ineligible cases, acyclic constructor delegation, receiver-first pure instance-call lowering, constructor-only and ordered init/required/object-initializer cases, construction/public invariant proofs, sidecar non-authority, attribute bypass, rewritten-mirror rejection, and all mutation/identity/inheritance escapes reject |
| Arrays / sequence construction | exact explicit/implicit creation forms and rejected collection/stack/range/best-common-type forms; structural rejection versus symbolic bound obligations; immediately complete default-eligible/zero-length allocation versus unobservable uninitialized construction cells; initialized-set/prefix tracking; premature read/alias/call/publication and catch/finally rejection; abrupt discard; complete non-defaultable elements; exact-`int` boundary lengths and indices; post-initialization unique rewrites; active-foreach mutation; every linear ownership/publication path; count-then-allocate filtered variable-result construction; and rejected source-visible builders |
| Ordered map/set | application-owned entry/array/wrapper bindings, read/count/membership/lookup loops, canonical ordered enumeration, key-order matrix, duplicate rejection/replacement semantics when implemented in captured source, bound preservation, and rejected float keys/comparers/hash/framework/insertion-order dependencies |
| Strings / codecs | exact string/string and string/char concat matrix, restricted interpolation equivalence and rejected alignment/format/non-string/non-char holes, rejected char/char and object conversion, null/empty concat and equality, intrinsic-only ordinal arguments, null receivers/arguments, UTF-16/surrogates, every exact parse/format grammar and noncanonical/range mutation, lossless round-trip plus fixed-scale rounded-value laws, and pinned-runtime differential corpus |
| Float / decimal | exhaustive small-domain properties plus bit/rounding/overflow/NaN/signed-zero differential vectors against the pinned runtime |
| Nullable / lookup / results / validation | exact `T?` compiler identity and lowering, reference annotations versus runtime null, exact conditional-access/coalescing evaluation and rejected lifted shapes, application-owned closed bindings, all internal option/lookup/tagged-sum transitions, nested-option rejection and the exact lookup-versus-null exception, missing-key versus stored-null lookup, guarded active/inactive payloads, actual default mapping, empty-invalid enforcement, deterministic array-based error accumulation/order/bounds, and exhaustive matching |
| Business values | calendar boundaries/leap days and exact day-of-week enum, time wrap/carry, duration/application-instant binding/precision/difference-range errors, GUID comparison/codecs/no-generation, application-owned money creation/add/subtract/rate/division, currency/scale/rounding/error precedence, and canonical-storage-versus-business comparison cases |
| Structural equality/order | every admitted recursive type, null/decimal/GUID corner, lexicographic cases, NaN preservation and rejected non-total keys |
| Loops | invariant initialization/preservation/exit, decreases, break/continue, nested loops, partial-versus-total evidence, rejection of a partial callee on a total path, and total-only boundary/transition/example/public routes |
| Switch / patterns | source-order arms and guards, exhaustiveness, null/property/list cases, Roslyn decision-graph upgrade vectors |
| Exceptions | built-in operation edges, parameterless explicit built-in construction, source-exception payload construction, rejected message/inner/runtime-state constructors, lexical filter-before-finally search, inner-to-outer unwind, filter throws, ordered catch/finally propagation, exceptional contracts, and uncaught rejection/obligations |
| Iterator / async exclusions | every iterator, `yield`, framework enumeration protocol, `async`, `await`, task/value-task type, awaiter, cancellation, and state-machine shape rejects inside the selected closure while an unselected adapter remains outside proof |
| Boundary values | duplicate/unknown and required/non-null/three-state missing/null cases, numeric/text canonicality, depth/count/byte limits, raw-input/canonical-value/output-reparse linkage, serializer/runtime mutation, verification-overlay-only transport, and an unchanged MPK-unaware production adapter |
| State transitions | invariant and version preservation, accepted/error arms, ordered bounded events and response relation, explicit-time and optimistic-conflict behavior, optional idempotency replay, field-complete snapshot equality/mismatch, non-reflexive-field rejection, history capacity, and precedence cases |

Every accepted source case runs twice from isolated builds and emits identical
canonical artifacts. Independent evaluators compare MPK VIR behavior with the
pinned C# runtime for finite test domains. Bounded fuzzing owns parser,
contract, Roslyn adapter, pattern, exception-region, collection, codec,
calendar, boundary, transition, source-binding, specialization, and artifact
protocols. Mutation suites cross every profile/schema/context/hash boundary.

The existing `fixtures/csharp/policy/source/src/Required.cs` expression-body
mismatch must be repaired before using that fixture as frontend evidence. The
replacement fixture must run through the real installed C# frontend rather
than attach a manually constructed VIR to source bytes. Regenerate all linked
scan, evidence, certificate, source-map, manifest, and hash bytes in one
reviewed change. Before activation, a verified replacement may exist only as
private migration evidence; the tracked `fixtures/csharp/policy/` files change
together in the atomic release commit.

Add three general-facing, runnable end-to-end C# examples:

1. **Invoice pricing and tax:** immutable request/result plus an
   application-owned money type, currency and scale checks,
   business/effective dates, decimal rounding, ordered line aggregation, and
   count-then-allocate bounded array construction.
2. **Order state transition:** GUID command/idempotency keys, explicit instant
   and expected version, switch/pattern state logic, an application-owned
   closed result/transition pair, one caught allowlisted exception, replay-safe
   response, and an ordered bounded event array.
3. **Batch input validation:** canonical boundary JSON, missing versus null,
   exact boundary parse/format, application-owned ordered entry arrays,
   duplicate handling, accumulating closed validation, and synchronous
   array-based processing.

Across the three examples, include constructor-only and
`required`/`init`/object-initializer construction, array and string operations,
loop invariants/decreases, structural equality/order, nullable data, and every
new business primitive. Each example documents what the certificate proves and
what remains an untrusted serializer, identity/time source, persistence, or
transport claim. Each example must compile as an ordinary application without
an MPK namespace, package, assembly, attribute, interface, base type, generated
source file, or runtime component. The verification overlay and emitted
artifacts remain separate from the example's application files.

An example's source and artifacts may be checked in only after `mpk policy
scan` and `mpk policy verify` process its actual source through the installed
frontend, its boundary bytes round-trip through the canonical value, and both
checkers accept the same certificate bytes. Before the final activation stage,
“installed frontend” means the exact privately materialized candidate image: a
checked-in example remains rehearsal-only, is absent from the active installed
release and public routing/documentation, and does not activate a production
tuple. The final activation task atomically installs and advertises the already
verified examples.

## 24. Implementation stages and gates

The reviewed implementation decomposition is maintained at
[`08_csharp_practical_subset_design-todo.md`](08_csharp_practical_subset_design-todo.md).
It replaces the superseded source-visible-library, iterator/async, and
suspension-stage plan and is the current execution decomposition subordinate
to this design and the future T01 frozen specifications. Its `Wnn`
identifiers, dependencies, owners, acceptance gates, and primary test routing
are current. The accepted native `JAVA-03-T10` x86-64 Linux release receipt
satisfied the phase entry gate. `CSHARP-03-T01-W01/W02/W03/W04/W05/W06/W07` have
closed the entry audit, consumer inventory, private frontend/toolchain closure
proof, and Roslyn data/construction/control/exception/pattern/dependency/
generic/iterator/async-rejection plus primitive/string/numeric/codec runtime
measurements; `CSHARP-03-T01-W08` is ready,
and each later work item remains blocked until its serial predecessor and task-
local entry gate are satisfied.
No task may reintroduce a source-visible `Mpk.*` API, user-defined generic,
iterator, or async scope.

CSHARP-03 is implemented serially behind private entrypoints:

```text
CSHARP-03-T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08
```

1. **CSHARP-03-T01 — Feasibility and freeze.** Pin toolchain versions; measure
   exact public Roslyn shapes and .NET behavior; freeze the zero-dependency
   source rule, exact `T?` exception, application semantic-binding schemas,
   the exact foundation descriptor/content hash, section 6.2's closed template
   registry and generated dependencies, recursive default eligibility,
   specialization identity and expansion,
   construction lowering, equality/key ordering, application-owned money/
   instant obligations, calendar/GUID/codecs,
   boundary/state-transition schemas, specification, vectors, identities,
   limits, and a traceability ledger. Any unresolved compiler/runtime fact is
   a stop condition.
2. **CSHARP-03-T02 — Shared artifact foundation.** Implement the one successor
   registry, monomorphic VIR type/operation/exception, sequence/map/set and
   tagged-sum vocabulary, registered foundation descriptor/hash,
   semantic-template registry and derived closed-instance tables,
   application-binding linkage, business-value/codecs, three-state boundary
   presence, source-map,
   manifest, VC/skeleton, hash, and generic-free VIR-importer boundary. Migrate
   predecessor producers/consumers privately and prove semantic equivalence.
3. **CSHARP-03-T03 — Loop-free data frontend.** Add expression bodies, local
   `var`, name-resolution syntax, immutable types, constructors, fields/
   properties, init/required/object initializers,
   source-dependency and generic rejection, semantic-binding validation,
   closed-instance collection, canonical monomorphization, structural
   equality/order, arrays and direct sequence/map/set representations, strings,
   float/decimal, nullable/closed outcomes, internal boundary-presence and codec
   handoffs, and business primitives with complete negative vectors. Loop-
   dependent collection algorithms remain assigned to T04, and boundary
   invocation remains assigned to T05.
4. **CSHARP-03-T04 — Control frontend.** Add loops/contracts, including
   explicit-type/`var` `foreach` and its array read-borrow rule; complete count/
   fill, lookup, aggregation, sort, and dedup source algorithms over the T03
   collection representations; then add switch/patterns and explicit
   exceptional CFGs and handlers.
5. **CSHARP-03-T05 — Boundary and transition frontend.** Add canonical boundary
   decoding/encoding, application-owned presence/result/transition bindings,
   verification-overlay handoff contracts, ordered event arrays, and
   idempotency/version behavior without a source or runtime MPK dependency.
6. **CSHARP-03-T06 — Verification integration.** Add expanded contract
   expressions, construction/type/state invariants, exceptional/loop/
   collection/codec/transition and semantic-binding obligations, canonical
   boundary round trips, concrete specialized ordinary core definitions and
   proof terms, policy/evidence/AI/API linkage, and same-byte dual-checker
   certificates.
7. **CSHARP-03-T07 — Release candidate.** Build reproducibly, assemble
   immutable toolchain, frontend, and verification-foundation bundles plus a
   private candidate tuple, run hostile-environment and native sandbox gates,
   and add no ambient library/effect route. Active installation remains T08.
8. **CSHARP-03-T08 — Complete rehearsal and activation.** Run the complete
   predecessor and practical corpus twice, fix review findings to zero,
   generate and verify all three runnable business examples, then atomically
   install the sole successor release.

No stage creates a public compatibility flag or partial practical-profile
route. If any requested capability fails feasibility, the profile remains
inactive; the design is revised and re-frozen rather than claiming partial
completion under the same ID. Iterator and async are explicit exclusions, not
capabilities whose absence can be hidden as an implementation shortfall.

## 25. Release acceptance criteria

Activation requires all of the following:

- every requested capability in section 1 has positive, negative, boundary,
  differential, deterministic, and upgrade evidence;
- all old C# scalar accepted/rejected cases retain their source behavior,
  obligations, checker verdicts, and axiom categories;
- all active Go, Rust, and Java cases retain their source behavior,
  obligations, checker verdicts, and axiom categories;
- Certificate v0 and both checker acceptance rules are unchanged;
- float, decimal, collection, option/lookup/result/validation,
  calendar/GUID/codec, boundary-presence/transition, exception,
  source-binding, and closed-specialization encodings retain an empty
  proof-node table, empty theory-certificate table, no theory primitive/node,
  and a recomputed total axiom count of zero;
- every accepted example and captured application source builds without an MPK
  package, assembly, namespace, attribute, interface, base type, generated
  source, or runtime dependency, and the application output has no MPK assembly
  reference;
- every emitted VIR, VC, certificate, and checker input is monomorphic and the
  importer has recomputed the complete bounded specialization closure;
- the semantic context, registered release tuple, foundation descriptor,
  source manifest, evidence, and reproduction recipe all bind the same
  independently recomputed foundation-bundle content hash;
- every installed boundary, transition, example, and public practical-profile
  root plus its reachable call/loop closure has a discharged total-termination
  claim; partial evidence is absent from those routes;
- all three actual installed-source business examples, not constructed helper
  IR, pass boundary round-trip, `mpk policy scan`, `mpk policy verify`, evidence
  reproduction, and both checkers;
- the candidate passes the local native Linux assembly, image mutation,
  syscall, cgroup, resource, cleanup, and two-pass installed release gates;
- `./scripts/check-fast.sh` and the complete local release gate pass without
  GitHub Actions or workflow dependency;
- documentation states the exact remaining exclusions and does not call the
  profile full C# support; and
- review produces no findings before commit and push.

## 26. Freeze-time facts that must be measured

The design intentionally does not guess:

- exact Roslyn syntax/symbol/`IOperation` and CFG shapes for ordinary namespace
  imports, the redundant nullable directive, patterns, nullable shorthand,
  the restricted conditional-access/coalescing matrix, constructed-type
  identity, and the complete synthesized member/modifier/attribute inventory
  for init/required/object-initializer forms under the pinned compiler;
- exact .NET float/decimal result bits, scale, rounding, exception, string,
  date/calendar, time/duration, and GUID comparison/codec observations at every
  selected edge;
- the recursive default-eligibility matrix for every admitted semantic type;
- the exact foundation-bundle descriptor, member inventory, content hash and
  hash domain, plus section 6.2's closed template registry, operation sets,
  generated dependencies, and application-owned semantic-binding schemas;
- the money/instant source-shape obligations, key-order matrix, string/char
  concatenation matrix, parse/format grammars, day-of-week enum mapping,
  instant granularity and difference range, construction invariants,
  transition error/precedence rules, and boundary/state-transition schemas;
- the canonical specialization identity, dependency-closure, deduplication,
  ordering, expansion, and residual-generic rejection algorithms;
- the final artifact/schema/profile names and hashes;
- the exact deterministic limits that both checkers can sustain; or
- whether any proposed intrinsic needs a smaller initial allowlist.

Each compiler or runtime observation needs a disposable public-API/runtime
probe, a checked-in canonical result, an independent implementation owner, and
an upgrade mutation. Each schema, identity, closure, or specialization rule
instead needs canonical positive and negative vectors plus an independently
recomputed implementation check; it must not be justified by a runtime probe.
A failed or ambiguous required measurement narrows or blocks the feature; it
never falls back to compiler trust, caller-supplied metadata, or an axiom.

## 27. Primary references

- [C# types, strings, decimal, and nullability](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types)
- [C# arrays](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/arrays)
- [C# expressions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions)
- [C# statements, loops, switch, try/catch/finally](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/statements)
- [C# patterns](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/patterns)
- [C# exceptions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/exceptions)
- [C# classes](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes)
- [C# `init` reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/init)
- [C# `required` reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/required)
- [C# preprocessor and nullable directives](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/preprocessor-directives)
- [.NET date/time parsing and business-date type guidance](https://learn.microsoft.com/en-us/dotnet/standard/base-types/parsing-datetime)
- [`DateOnly` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly?view=net-10.0)
- [`TimeOnly` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.timeonly?view=net-10.0)
- [`TimeOnly.Add` overloads](https://learn.microsoft.com/en-us/dotnet/api/system.timeonly.add?view=net-10.0)
- [`TimeSpan` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.timespan?view=net-10.0)
- [`Decimal.Round` overloads](https://learn.microsoft.com/en-us/dotnet/api/system.decimal.round?view=net-10.0)
- [`MidpointRounding` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.midpointrounding?view=net-10.0)
- [`Guid.TryParseExact`](https://learn.microsoft.com/en-us/dotnet/api/system.guid.tryparseexact?view=net-10.0)
- [`Guid.CompareTo`](https://learn.microsoft.com/en-us/dotnet/api/system.guid.compareto?view=net-10.0)
- [`CSHARP_PROFILE_V0.md`](../specs/CSHARP_PROFILE_V0.md)
- [`SEMANTIC_PROFILE_REGISTRY_V1.md`](../specs/SEMANTIC_PROFILE_REGISTRY_V1.md)
- [`PROGRAM_CERTIFICATE_ALPHA_V0.md`](../specs/PROGRAM_CERTIFICATE_ALPHA_V0.md)
- [`06_multilanguage_frontend_design.md`](06_multilanguage_frontend_design.md)
- [General-facing sample/subset guide](../../docs/csharp-samples-and-subset.md)
