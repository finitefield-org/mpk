# C# Practical Subset Expansion Design

Status: proposed governance and implementation design. This document does not
change the active `mpk.csharp.scalar.v0` profile, register a new profile, or
authorize a practical-profile public route. The active release is registry
revision 3 with Go, Rust, C# scalar, and Java scalar support. `JAVA-03-T10`
implementation landed on 2026-09-03, but its native x86-64 Linux gate receipt
is still required before CSHARP-03 entry; this proposal does not itself start
specification or implementation work.

Prepared: 2026-09-02.

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

The requested capability set is:

| Area | Practical-profile boundary |
| --- | --- |
| Concise syntax | expression-bodied members and locally inferred `var` when they normalize to an otherwise admitted form |
| Domain data | source-defined enums, immutable structs, and sealed immutable classes with fields, properties, constructors, `init`/`required`, object initializers, and pure instance methods |
| Collections | bounded arrays, MPK-owned sequence builders, immutable sequences, canonical ordered maps/sets, lookup, aggregation, and `foreach` |
| Text | bounded UTF-16 strings, ordinal operations, and exact culture-free parse/format grammars |
| Numbers | existing integers plus exact `float`, `double`, and .NET `decimal` semantics |
| Absence | nullable value types, nullable string/array/class references, and boundary-only missing/null/value presence with explicit proof obligations |
| Business values | exact date, time-of-day, duration, Unix instant, GUID, money-template, structural equality, and canonical ordering semantics |
| Domain outcomes | closed `Option<T>`, `Lookup<T>`, `Result<T,E>`, and accumulating `Validation<T,E>` values without general user-defined generics |
| Control flow | `while`, `do`, `for`, `foreach`, `break`, `continue`, switch statements/expressions, and closed patterns |
| Failure | explicit throws, built-in operation exceptions, typed catch, pure filters, and `finally` under a closed exception model |
| Lazy flow | source iterators whose values do not escape the admitted closure |
| Async flow | sequential `Task`/`Task<T>` and closed async iterators whose scheduling and external effects are unobservable |
| Integration | versioned canonical boundary values and pure state/command-to-new-state/events/response transitions; serializers, clocks, databases, and transports remain outside proof |

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
implemented, but W01 remains blocked until its native release receipt is
recorded; no normative identity or production change has started here. DART-04
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
- Cover practical construction of variable-length results, key-based lookup,
  accumulating validation, business dates and identifiers, and optimistic
  state transitions without admitting ambient services.
- Preserve C# evaluation order, overflow, rounding, null, exception, iterator,
  and immediate-await behavior for every admitted form.
- Keep compiler and runtime observations untrusted and independently
  revalidated by MPK-owned representations and both source-free checkers.
- Keep all input graphs, values, CFGs, proof terms, diagnostics, and processes
  deterministically bounded.
- Add no axiom, theory certificate, C# semantics primitive, or new
  proof-authority path to an accepted program certificate.
- Retain identical semantics for every active Go, Rust, C# scalar, and, after
  T10, Java profile during the required shared-artifact migration.

### 3.2 Non-goals

The practical profile is not arbitrary C# or arbitrary .NET. It does not
accept:

- mutable shared object graphs, observable object identity, weak references,
  finalizers, or unsafe/ref-like storage;
- inheritance or virtual/interface dispatch, except the closed exception
  hierarchy and compiler/profile-recognized generic protocols described below;
- user-defined operators/conversions, delegates, lambdas, expression trees,
  LINQ, reflection, `dynamic`, records, record structs, `with` expressions,
  arbitrary user-defined/open generics, or runtime code generation;
- general `List<T>`, `Dictionary<K,V>`, `System.Collections.Immutable`, spans,
  caller comparers, arbitrary framework collections, array covariance,
  multidimensional arrays, or jagged arrays; only the closed MPK-owned
  collection surface in section 9 is admitted;
- culture-sensitive text, normalization, regular expressions, general-purpose
  formatting/parsing, resources, globalization, or ambient locale; only the
  exact canonical grammars in section 10 are admitted;
- filesystem, database, network, clock, random, environment, console, process,
  synchronization, thread, scheduler, cancellation, or other external effects;
- custom awaiters, arbitrary tasks, task races, parallel execution, or an
  assertion that an external asynchronous operation is correct;
- catchable resource exhaustion such as `OutOfMemoryException` or
  `StackOverflowException`; and
- project/NuGet discovery, analyzers, generators, MSBuild behavior, or an
  ambient reference assembly.

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
csharp2vir validation and lowering                     (untrusted producer)
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

Roslyn success, nullable warnings, generated state machines, runtime
differential results, and frontend success are never proof evidence. They
establish only that the untrusted producer followed the frozen adapter
contract. Proof acceptance still comes from identical certificate bytes
accepted by both source-free checkers.

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

- run-time-length arrays, immutable sequences, linear builders, canonical
  ordered maps/sets, and UTF-16 strings;
- nullable/optional values;
- tagged option/lookup/result/validation/boundary-presence/transition values,
  business-date/time/GUID values, and construction-state tracking;
- IEEE binary32/binary64 and .NET decimal operations;
- normal and exceptional successors from one source operation;
- exception propagation and handlers;
- iterator yield/suspension state; and
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
| Business boundary/transition contracts | `mpk.csharp.boundary.v1`, `mpk.csharp.transition.v1` |
| Profile-owned foundation surface | `mpk.csharp.practical.foundation.v1` |
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
domain, including contract, registry, VIR, source-map, manifest, release-root,
and VC hashes. A Certificate v0, declaration, axiom-report, or input-set domain
may remain only when its exact preimage and meaning remain unchanged. Old
parsers must reject every new family, and new parsers must reject old-family
bytes wherever parallel acceptance would create ambiguity.

### 5.2 Atomic migration

One release must:

1. retain every existing profile identity and language semantics, migrating
   only entry envelopes and compiled bindings required by the successor
   registry schema;
2. add the practical C# entry only after its specification and vectors are
   complete;
3. migrate every active producer and consumer to the one successor shared
   artifact family;
4. regenerate all context-bound contracts, transports, manifests, examples,
   fixtures, VCs, reports, receipts, and hashes;
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
class forms and adds only the declarations closed below.

The new selection envelope names:

- one compilation ID;
- all source paths;
- selected root methods or constructors;
- every type, method/constructor, boundary, and transition sidecar; and
- no executable, reference, package, registry, or toolchain path.

The closure begins at the selected roots and includes every source-declared
type, constructor, property getter, instance/static method, iterator, and async
method reachable through admitted calls and field/property types. Type and call
graphs are finite and bounded, and the call graph remains acyclic; direct or
mutual recursion rejects. Every ordinary declaration in a selected source file
must belong to that closed compilation and satisfy the profile; an
unrelated, unreachable, or unselected type/member does not become ignored or
trusted and rejects.

Partial declarations, source generators, ambient metadata user types, nested
generic types, and conditional compilation remain rejected. The only admitted
metadata-backed types beyond the pinned scalar value surface are the exact
predefined text/floating/decimal types, inert `object` base, closed exception
hierarchy, `StringComparison.Ordinal` intrinsic position, admitted
date/time/GUID types, the exact closed `System.DayOfWeek` enum, and closed
instantiations of the profile-owned collection, option, lookup, result,
validation, boundary-presence, transition, task, and iterator protocols.
Compiler-synthesized members may be observed only for an exact frozen pattern
and are never used as the semantic definition of an admitted source feature.

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
type, `dynamic`, implicit arrays with a mixed inferred type, multiple local
declarators, and a `var` alias/type named by source reject.

No semantic-profile field depends on whether a local used `var`; it is source
syntax and mapping evidence only.

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
string, immutable collection, option/lookup/result/validation/boundary-presence,
business primitive, enum, or earlier structural types. Instance fields are
explicitly `readonly`; properties are getter-only or init-only; methods are
pure and nonvirtual.
Custom layout, fixed buffers, ref fields, events, indexers, destructors,
operators, conversions, and boxing reject.

“User-defined value type” is structural rather than a hard-coded name or shape
allowlist: every source-defined enum or non-generic `readonly struct` satisfying
these rules is eligible. It does not mean arbitrary mutable, unsafe, generic,
explicit-layout, or runtime-provided CLR value types.

Structs lower to named structural values. Declaration and field order are
canonical source order, while type declarations are emitted in dependency
order. Every admitted source or profile-owned semantic type records
`default_eligible`. It is true only when the specification freezes the exact
recursive zero/null value, every nested default is eligible, no member is
required, and the public type invariant holds for that value. A non-null
reference type is ineligible; its nullable option form may be eligible through
`None`. `default(T)`, an implicit zero-valued struct construction, and any other
publication of a default value reject when this fact is false. Temporary zero
state before an admitted constructor finishes is construction state and cannot
be read or published.

### 8.2 Sealed immutable classes

Admit ordinary non-generic `sealed class` declarations as immutable value
objects under these restrictions. Source exception declarations use the
separate closed-hierarchy rule in section 15:

- the only base type is `object`;
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

Pure instance methods lower to direct functions with the receiver as the first
argument. Calls are statically resolved to one source declaration; there is no
virtual dispatch.

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
transaction. It cannot be read, captured, passed, returned, awaited across, or
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

The profile derives structural equality for admitted immutable structs,
classes, read-only arrays, sequences, ordered maps/sets, option/lookup/result/
validation/boundary-presence values, and business primitives. Fields are
compared in canonical declaration order; sequence elements and map entries are
compared in canonical order; active sum arms compare their tag and payload. A
null reference is equal only to null. Floating-point equality retains C# NaN
and signed-zero behavior and therefore is not silently changed into bit
equality.

The provisional source surface is the exact profile-owned
`Mpk.Value.Equal<T>(T, T)` operation. `Mpk.Value.Compare<T>(T, T)` is admitted
only for the frozen totally ordered key matrix: integer and Boolean scalars,
`char`, ordinal string, enum carrier, decimal value, date/time/duration/instant,
GUID, nullable values with null first when their payload is orderable, and
immutable structural values whose fields are all orderable. `float`, `double`,
their nullable forms, and structures containing them are excluded from ordering
and map/set keys because NaN prevents the required total order.

These operations are statically specialized for one admitted closed type and
lower to shared structural terms. They do not admit virtual `Equals`,
`IEquatable<T>`, `IComparable<T>`, caller comparers, user-defined equality or
ordering operators, boxing, `GetHashCode`, or reference equality. The freeze
must assign the exact decimal, GUID, null, and lexicographic ordering vectors.

## 9. Arrays, bounded sequences, and ordered collections

### 9.1 Arrays and ownership

Admit zero-based, one-dimensional `T[]` where `T` is an admitted immutable
non-builder value and the run-time length is within the profile maximum. A
length-only allocation is admitted only when its length is proved zero or `T`
is `default_eligible`; an array initializer instead proves every supplied
element's public invariant.
This prevents zero-filled arrays of non-null class values, required-member
structs, or invariant-bearing values such as the Money template from publishing
invalid elements. The array itself is a reference at the C# boundary, so
nullability is modeled separately. Multidimensional, jagged, covariant,
`System.Array`, `Span<T>`, `Memory<T>`, and arbitrary collection/interface
conversions reject.

Allowed operations are:

- array creation from a bounded length or initializer;
- `Length`;
- indexed reads with exact `IndexOutOfRangeException` behavior;
- `foreach` in element order;
- equality with `null`, but no array reference equality; and
- indexed writes only while a fresh local array has unique ownership.

Parameters, fields, property values, captured arrays, and arrays passed to
another method are read-only. A newly allocated local array begins `unique`.
The frontend runs a conservative ownership analysis: assignment, capture,
passing, storage in another value, or return freezes or transfers the array;
any later write through a possible alias rejects. Returning a uniquely owned
array transfers ownership to the result. `ref`, `out`, slices, pointers, and
element references reject.

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

### 9.2 Bounded immutable sequences and builders

Add exact profile-owned closed instantiations of
`Mpk.BoundedSequence<T>` and `Mpk.BoundedSequence<T>.Builder`, where `T` is an
admitted immutable value. These are compiler-recognized surface contracts shipped
in the registered practical foundation bundle, not permission to resolve an
ambient assembly or arbitrary generic type.

An immutable sequence admits `Count`, indexed read, `foreach`, structural
equality, canonical lexicographic ordering when `T` is orderable, and copying
into a fresh builder. A builder admits only bounded creation, `Count`, indexed
read, `Add`, and `Freeze`. It is a unique local linear value: it cannot be null,
aliased, captured, placed in a field/array/map, passed to an arbitrary method,
returned, compared, yielded, or live across `await`. A loop may retain it only
when its ownership and count are explicit in that loop's modifies/invariant
record.

“Copying into a fresh builder” and filtered projection mean ordinary bounded
creation followed by source-order `foreach`/conditional `Add` calls. They do
not add `AddRange`, `Filter`, a callback, a lambda, or another builder API.

`Add` preserves insertion order. Builder creation requires a nonnegative
capacity and has an exact `ArgumentOutOfRangeException` edge otherwise.
`Count < declared_capacity` is `Add`'s checked normal-path condition and a full
builder has an exact `InvalidOperationException` edge;
`declared_capacity <= profile_max` is a separate profile-bound obligation.
`Freeze` transfers the complete contents to one immutable sequence and
permanently invalidates the builder. An exception before freeze discards the
unpublished builder; a use after freeze rejects statically. Failure to prove
the profile bound blocks verified acceptance and never yields a partial
sequence.

This surface solves filtered projection and variable-length result assembly
without admitting general `List<T>` semantics. `ImmutableArray<T>` and its
builder are API-design references only; their package, interfaces, extension
methods, default value quirks, and full method set are not accepted wholesale.

### 9.3 Canonical ordered maps and sets

Add exact profile-owned `Mpk.OrderedMap<K,V>` and `Mpk.OrderedSet<T>` plus
unique local builders. Key/element types must satisfy the frozen total-order
matrix in section 8.5; values are any admitted immutable value. The immutable
map surface admits `Count`, `ContainsKey(K)`, `Lookup(K)` returning
`Mpk.Lookup<V>`, and `foreach` in strict canonical key order. The immutable set
surface admits `Count`, `Contains(T)`, and the same canonical enumeration. Both
expose no buckets or insertion order.

A map builder admits bounded creation, `Add` with duplicate-key
`ArgumentException`, `Put` with deterministic replacement, lookup returning
`Mpk.Lookup<V>`, `Count`, and `Freeze`. A set builder admits
bounded creation, `Add`,
whose Boolean result says whether the element was new, `Contains`, `Count`, and
`Freeze`. An operation that inserts a new entry into a full builder has the
same `InvalidOperationException` edge as a full sequence builder; `Put` may
replace an existing entry and set `Add` may report an existing element even
when full. The freeze must set duplicate-versus-capacity diagnostic/exception
precedence. Builders obey the same uniqueness, loop, abrupt-completion, bound,
and post-freeze rules as sequence builders.

Map/set values lower to sorted duplicate-free sequences. The importer
recomputes key order and duplicate absence; the VC layer proves preservation
for every builder operation. Runtime hash codes, randomized hashing, ambient
`Comparer<T>.Default`, caller-provided comparers, enumeration-order accidents,
and framework `Dictionary`/`HashSet` behavior cannot influence artifacts.

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
section 10.2's profile-owned exact codecs are the only conversion exception.

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

### 10.2 Exact parse and format profiles

Add profile-owned, statically resolved parse/format functions whose working
surface is `Mpk.Text`. Parsing returns `Mpk.Result<T, Mpk.ParseError>` rather
than using general `out` parameters or exception-driven control. Formatting
returns a bounded non-null string and carries an output-length obligation.
These functions are ordinary checked foundation definitions; calling a BCL
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
scale/precision, and input-bound errors through a closed `ParseError` enum.
Invalid calendar dates, overflow, leading/trailing whitespace, culture-specific
digits, group separators, currency symbols, and alternate calendars return an
error without a partial value. Unknown codec IDs or rounding-mode values reject
during closed source/sidecar validation and never reach a parser. A formatter
emits one canonical spelling and carries the output-bound obligation described
above; it does not return `ParseError`. Every lossless codec satisfies
`parse(format(value)) = value`; fixed-scale decimal formatting instead parses
to the value produced by its explicit scale and rounding mode. Every
successfully parsed input reformats byte-identically under its matching codec.

General `Parse`, `TryParse`, `ToString`, composite formatting, numeric
interpolation, `IFormatProvider`, `CultureInfo`, custom format strings, and
ambient current culture remain rejected. Boundary codecs in section 18 reuse
these exact grammars rather than define a second conversion meaning.

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
- exact allowlisted `Round`, `Truncate`, `Floor`, and `Ceiling` overloads; and
- no user-defined conversion, float/double conversion, direct BCL formatting/
  parsing, currency, culture, or representation inspection; section 10.2's
  profile-owned exact decimal codecs remain available.

Trailing-zero scale is normalized only if every admitted observation is proven
value-based. `decimal.GetBits`, formatting, hash codes, and APIs that expose
representation remain rejected. MPK-owned arithmetic is differentially tested
against the exact pinned .NET runtime but is not defined by accepting the
runtime's answer.

## 12. Nullable, domain outcomes, and business primitives

### 12.1 Nullable values and references

The practical compiler session fixes nullable annotations and warnings to
enabled and rejects source directives that change them. Roslyn nullable flow
analysis remains a diagnostic observation, not proof.

Represent:

- `T?` for an admitted non-nullable value type as `Option<T>`;
- every string, array, and admitted class reference as an optional structural
  value at the semantic boundary; and
- source annotations as intent that determines required contracts and
  diagnostics, not as a runtime guarantee.

A non-nullable reference parameter must have an explicit `not_null`
precondition. A non-nullable result or field must be proven non-null on every
normal exit. Nullable references may use `is null`, `is not null`, `== null`,
`!= null`, `?.`, and `??`. Nullable value types may use `HasValue`, `Value`,
`GetValueOrDefault`, lifted equality/comparison/arithmetic in a frozen closed
matrix, and matching patterns. Parameterless `GetValueOrDefault()` is admitted
only when the payload type is `default_eligible`; the overload with an explicit
fallback instead proves that fallback's public invariant.

Dereference or `Value` access without a proven present branch emits the exact
null/invalid-operation exceptional edge. The null-forgiving operator `!` is
accepted only if MPK's own dataflow already proves non-null and therefore it
has no semantic effect; otherwise it rejects. Repeated nullable encodings and
`Mpk.Option<Mpk.Option<T>>` reject; `Lookup<Option<T>>` is the one intentionally
admitted lookup-versus-null composition described in section 12.2. Other
tagged-sum nesting follows the explicit depth limit. Nullable by-reference
storage rejects.

### 12.2 Option, lookup, result, and accumulating validation

Add exact profile-owned closed instantiations of `Mpk.Option<T>`,
`Mpk.Lookup<T>`, `Mpk.Result<T,E>`, and `Mpk.Validation<T,E>`. Type arguments
must be admitted closed immutable values, subject to the nested-option
exclusion in section 12.1; user declarations do not gain generic parameters,
variance, constraints, reflection, interface dispatch, or arbitrary generic
method inference.

`Option<T>` is `None` or `Some(T)`. `Lookup<T>` is `MissingKey` or `Found(T)`;
unlike nested option, `Lookup<Option<T>>` is admitted so a map can distinguish a
missing key from a stored nullable value. `Result<T,E>` is `Ok(T)` or
`Error(E)`. `Validation<T,E>` is `Valid(T)` or
`Invalid(BoundedSequence<E>)`, where the error sequence is nonempty, bounded,
and kept in source/evaluation order. Construction, tag tests, active-payload
reads, structural equality, pattern matching, and exhaustive switch are
admitted. Reading an inactive payload has an explicit
`InvalidOperationException` edge. Constructing `Invalid` has the checked normal
condition `Count > 0` and an exact `ArgumentException` edge otherwise. The
caller must prove that edge unreachable, catch it, or declare it in the method
contract; no failing path constructs a partial validation value.

The frozen recursive defaults for `Option<T>` and `Lookup<T>` are `None` and
`MissingKey`; both are `default_eligible` even when their inactive payload type
is not. `Result<T,E>` and `Validation<T,E>` are not `default_eligible` in this
profile and require explicit construction of one valid active arm. No inactive
payload bytes become observable or satisfy an invariant by assumption.

The profile supplies no lambda-based `Map`, `Bind`, query syntax, implicit
conversion, exception coercion, or hidden short circuit. Code combines results
with ordinary branches/switches. Validation accumulation appends left errors
before right errors and proves the combined bound; it never drops, sorts, or
deduplicates errors implicitly. Expected business rejection uses these values,
while exceptions remain for the exceptional paths declared in section 15.

### 12.3 Date, time, duration, instant, GUID, and money

Admit a closed operation subset over pinned `System.DateOnly`,
`System.TimeOnly`, `System.TimeSpan`, and `System.Guid`, plus the profile-owned
`Mpk.Instant`. These metadata types are explicit practical-foundation
intrinsics: their runtime implementations remain untrusted observations and do
not authorize other framework members.

- `DateOnly` uses the proleptic Gregorian calendar and the exact pinned .NET
  range. Construction, year/month/day/day-number access, comparison, day-of-
  week through the exact closed `System.DayOfWeek` enum, and bounded
  `AddDays`/`AddMonths`/`AddYears` are admitted with exact range exceptions.
  That enum has exactly the seven pinned named values and the corresponding
  carriers; numeric casts, arithmetic, flags behavior, and other `System.Enum`
  APIs remain rejected.
- `TimeOnly` is a time of day represented by 100-nanosecond ticks in one day.
  Components, comparison, subtraction to duration, and explicitly frozen
  wrap/day-carry behavior for addition are admitted.
- `TimeSpan` is the signed 64-bit 100-nanosecond duration carrier. Exact
  construction, components, comparison, and checked add/subtract/negate are
  allowed; scaling requires a separately frozen exact profile operation.
- `Mpk.Instant` is a signed 64-bit Unix-millisecond UTC instant. It admits
  comparison. Duration addition/subtraction returns an exact result containing
  either an instant or `Mpk.InstantError` when the duration has non-millisecond
  ticks or the result is out of range. Instant difference likewise returns
  either an exact duration or that error when the millisecond difference cannot
  be represented as signed 64-bit 100-nanosecond ticks; it cannot create a
  sub-millisecond remainder because both operands are millisecond instants. The
  freeze must fix the closed error enum and error precedence. It has no local-
  time, timezone, calendar, leap-second, or clock lookup behavior.
- `Guid` is an exact 128-bit identifier with `Empty`, equality, the frozen
  .NET comparison order, and the `N`/`D` codecs in section 10. It has no
  `NewGuid`, random source, byte-layout reinterpretation, or ambient generator.

Business money is supplied as a checked template, not a magical scalar:
`readonly struct Money` contains a decimal amount and an exact currency enum or
validated ordinal code. The template is not `default_eligible`; callers use a
closed `Create` operation that validates currency and scale and returns
`Mpk.Result<Money, MoneyError>`. It exposes amount/currency access, checked same-
currency addition and subtraction, multiplication or division by an explicit
decimal quantity/rate with target scale and rounding mode, and a same-currency
amount comparison. Every fallible operation returns a closed `MoneyError`
rather than hiding an expected business failure in an exception. The freeze
fixes its error arms and precedence, including invalid currency/scale, currency
mismatch, division by zero, and decimal overflow.

Structural equality observes currency and decimal value. Canonical storage
ordering, when requested through `Mpk.Value.Compare<Money>`, orders currency
code first and amount second; it is not an economic comparison across
currencies. Implicit rounding, ambient currency metadata, exchange-rate lookup,
and culture formatting reject. Projects may define similarly shaped money
types under section 8, but the runnable pricing example must use and prove this
template.

`DateTime.Now`, `DateTime.UtcNow`, local timezone conversion, daylight-saving
rules, `TimeZoneInfo`, non-Gregorian calendars, NTP/system-clock claims, and
database-generated timestamps remain outside the profile. A caller supplies
the effective date/instant explicitly through the boundary contract, and proof
establishes only logic over that value—not its real-world authenticity.

## 13. Loops and loop contracts

Admit `while`, `do`, `for`, and `foreach`, plus structured `break` and
`continue`. `goto`, labels, unsafe jumps, and irreducible control flow remain
rejected. `foreach` accepts admitted arrays, strings, immutable sequences,
ordered sets, canonical `Mpk.KeyValue<K,V>` map entries, and closed source
iterators. A builder is never itself enumerable.

Every loop receives a canonical ID by method ID and source-order ordinal, for
example `MethodId#loop#0000`. Its method sidecar supplies:

- one or more invariants;
- the values modified by the loop;
- zero or more decreases expressions;
- whether the containing function claims partial or total termination; and
- iterator/yield state when applicable.

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

The contract expression language gains bounded sequence quantifiers so useful
properties such as “all elements before index `i` satisfy P” can be stated.
Quantifiers range only over an explicit bounded sequence interval; arbitrary
unbounded quantification and triggers are not admitted.

## 14. Switch and pattern matching

Admit switch statements and switch expressions over admitted scalar, enum,
string, nullable, option/lookup/result/validation/boundary-presence, business-
primitive, and immutable structural values. Evaluate the governing expression
once and preserve first-applicable-arm and guard order.

The closed initial pattern set is:

- constant, discard, and `var` patterns;
- `null` / `not null`;
- relational and parenthesized patterns;
- `and`, `or`, and `not` logical patterns;
- declaration/type patterns over the finite admitted sealed type set;
- exact active-arm patterns for option/lookup/result/validation/boundary-
  presence tagged sums;
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

Admit `throw` of a newly constructed admitted exception, propagation from an
admitted operation/call, ordered typed catches, a pure Boolean filter, rethrow,
and `finally`. Catch matching uses the closed exception hierarchy. A catch
variable may read only admitted immutable payload.

A source-declared exception is sealed and derives directly from
`System.Exception`; only profile-declared constructors and immutable payload
are observable. Handler search evaluates typed catches and filters in lexical
order before unwinding the selected path. An outer filter can therefore run
before an intervening inner `finally`. If evaluation of a filter throws an
admitted exception, that filter is false, the original exception remains the
search subject, and search continues. The filter exception is represented in
the decision graph and cannot disappear from conformance evidence.

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
than trusting a compiler-generated state machine.

The existing scalar `panic = forbidden` meaning is unchanged. Practical
contracts instead declare an exact `throws` set with exceptional
postconditions. An empty set requires every throw path to be caught or proven
unreachable. Evidence reports normal postconditions, exceptional
postconditions, exception freedom, and termination separately.

## 16. Iterators

Admit synchronous iterator methods returning the exact pinned
`IEnumerable<T>` protocol and using `yield return` / `yield break`, where `T`
is admitted. Each invocation creates one unique, single-use producer that is
lexically paired with exactly one admitted `foreach` inside the selected
closure. Assignment or aliasing of the producer, repeated or concurrent
enumeration, escape to a field/result, interface call, LINQ operator, manual
enumerator, or external consumer rejects. A separate method invocation may
create a separate producer and be enumerated once.

Iterator execution remains lazy: argument and receiver values are captured at
creation, body execution begins on first advancement, and each yield suspends
the producer. Exceptions occur at the corresponding advancement point. The
frontend lowers source syntax to an explicit producer state relation; it does
not trust or serialize Roslyn's generated state-machine type.

Each iterator has loop/yield invariants and a decrease requirement when total
finite enumeration is claimed. A consumer may fuse with its producer only
after an equivalence obligation proves the same yield, exception, disposal,
and early-break order. Disposal is effect-free in the first profile. `yield`
inside unsupported exception/resource regions rejects.

The active consumer borrows its producer until normal completion, exception,
or early break, and then performs the exact modeled disposal transition. No
producer state is accessible afterward.

Async iterators use the same model only when every await satisfies section 17
and consumption is one closed `await foreach`. Arbitrary `IAsyncEnumerable<T>`
or external async streams remain outside the profile.

## 17. `async` / `await`

The practical profile supports async source shape without claiming to verify
network, database, timer, cancellation, scheduler, or thread behavior.

Admit `async Task` and `async Task<T>` methods only when:

- every task-producing call resolves to an admitted source method or exact
  `Task.FromResult`/completed-task intrinsic;
- every callee-produced task is immediately awaited; it cannot be stored,
  compared, combined, raced, inspected, or returned directly from an `async`
  method. The only task that escapes is the compiler-produced `Task` or
  `Task<T>` representing that method's own terminal completion;
- there is no custom awaiter, `async void`, `Task.Run`, delay, cancellation,
  synchronization context, `ConfigureAwait`, blocking wait, or continuation
  callback;
- captured values are immutable admitted values; and
- all normal and exceptional continuations remain in the closed source graph.

Under these restrictions task scheduling is unobservable. The semantic
projection erases the task wrapper to an explicit normal/exceptional result,
while the manifest records the source async signature and each await site. The
VC set includes an erasure-equivalence obligation; a task escape or observable
suspension makes the projection invalid and rejects.

An exception produced before or after an await propagates through the same
closed exception outcome. The generated async state-machine IL, builder,
execution context, and scheduler are not inputs to proof. `await foreach` is
accepted only for a closed async iterator satisfying sections 16 and 17.

This boundary lets a deterministic domain layer keep common async signatures.
Real I/O remains in an unverified adapter. Supporting arbitrary incomplete
tasks would require an effect and temporal/concurrency logic and therefore a
separate future profile.

For a selected root returning `Task` or `Task<T>`, the only public semantic
observation is terminal completion with its result or admitted exception.
Elapsed time, scheduling, task identity, intermediate status, and continuation
placement are outside the profile and any source that observes them rejects.

## 18. Business boundary, serialization, and state transitions

### 18.1 Versioned boundary values

Define a closed `mpk.csharp.boundary.v1` sidecar for each public verified-core
entrypoint. It names the semantic context, schema/version, selected method,
ordered input/output fields, admitted value types, maximum document/value
sizes, and the exact parse/format profile. Missing and explicit null are
different states. Duplicate names, unknown fields, unknown enum values,
noncanonical number/text spellings, excess depth/count/bytes, and a schema or
method mismatch reject before the value reaches the verified method.

Where business logic must distinguish omission from explicit null, admit one
closed `Mpk.BoundaryField<T>` specialization with exactly `Missing`, `Null`, and
`Value(T)` arms. `T` is a non-null admitted immutable payload. A required field
rejects `Missing`, and a non-null target rejects `Null`. An optional field
either exposes all three arms or declares one exact typed value used only for
`Missing`. That value is captured canonically in the sidecar and must satisfy
the target invariant. `Value(T)` always supplies its payload; explicit `Null`
remains distinct and either maps to nullable `None` or rejects for a non-null
target. No implicit missing/null collapse is allowed, and this boundary-
specific sum does not open general nested option types. Construction, tag
tests, active-payload access, matching, and inactive-payload exceptions follow
the closed sum rules in section 12.2; its default arm is `Missing`, so it is
`default_eligible` even when `T` is not.

The verified input is always one canonical boundary document. MPK captures its
exact bytes, independently parses it into the typed value, hashes both the byte
identity and canonical value, and binds them into the manifest/evidence chain.
An application adapter may accept some other JSON/media format, but it must
translate that input into this canonical document before invocation. The
adapter may record the original byte/provenance identity separately; the
certificate does not prove that its translation preserved the external
meaning. Supplying an adapter-created object while bypassing MPK's byte parser
rejects. Output follows the reverse rule: the verified core returns a canonical
value; an untrusted serializer emits a canonical boundary document; MPK
reparses it and checks that the bytes denote exactly that value before
retaining reproduction evidence.

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
  -> Mpk.Result<Mpk.Transition<State, Event, Response>, DomainError>
```

`Mpk.Transition<TState,TEvent,TResponse>` is another profile-owned protocol
admitted only at closed types; it contains exactly `NewState`, an immutable
`Mpk.BoundedSequence<TEvent>` named `Events`, and `Response`. `State`,
`Command`, `Context`, `Event`, `Response`, and `DomainError` are admitted
immutable values. The contract names the state invariant, exact accepted
command cases, expected state version, command/idempotency identifier, explicit
effective date/instant, normal transition postconditions, ordered emitted
events, response relation, and every business-error result. A newly applied
successful command proves the new invariant, the frozen version-increment
rule, event/response correspondence, and all collection bounds. A rejected
command leaves the input state unchanged.

Idempotency is claimed only when the explicit input state contains a bounded
processed-command record with the key, bounded canonical command-encoding
bytes, and response, and the boundary/transition contracts prove that those
bytes are the exact canonical encoding of the command. After ordinary boundary
preconditions, a retained key is checked first: byte-identical command encoding
returns the current unchanged state, no new events, and the stored response; a
different encoding returns an explicit idempotency-conflict error. This remains
reflexive even for NaN payloads and distinguishes missing from null. An evidence
digest may identify the record but never substitutes for byte equality or adds
a collision-resistance assumption. A new key then checks `expected_version`; a
mismatch is an explicit optimistic-concurrency error rather than an inference
from a future database write. The initial profile performs no implicit history
eviction: a new command at full history returns a specified capacity error.
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
  local array/builder updates are represented in construction/loop records but
  cannot escape under an alias;
- partial/total termination;
- ordered loop records with invariants, modifies, and decreases; and
- iterator/async clauses only for the corresponding source form.

An iterator contract additionally supplies an invariant over producer state
and the yielded prefix, an ordered `ensures_yield(index, value)` obligation for
each advancement, and `ensures_complete(count)` for normal exhaustion.
Exceptions are attached to the advancement that produces them; early disposal
must preserve the stated prefix invariant but does not claim completion.

An async method contract reuses one terminal normal or exceptional method
outcome after task erasure and cannot mention elapsed time, scheduling, status,
or continuation placement. An async iterator combines that terminal rule with
the iterator prefix/yield/completion clauses. All yield counts, indices,
prefixes, and async call depths are explicitly bounded.

The expression union adds typed field/property access, sequence length/index,
map/set lookup and membership, string/char, float, decimal, enum,
date/time/duration/instant/GUID, option/lookup/result/validation/boundary-field/
transition construction and tests, structural equality/order, parse-error
kind, exception kind and payload, and bounded `forall`/`exists`. Every
expression is closed, explicitly typed, depth/count bounded, and free of source
method calls. There is no arbitrary C# expression evaluator inside a contract
parser.

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
- unit for `Task`/void-like internal continuations.

Enums lower to their exact integer carrier plus a declaration identity.
Strings are a distinct profile mapping over a UTF-16 sequence so arbitrary
sequence operations cannot masquerade as string semantics.

### 20.2 Required operation/control vocabulary

Add explicit construction-state, field, option/lookup/result/validation/
boundary-presence/transition, builder, sequence, ordered-map/set, structural
equality/order, string parse/format, business-value, floating-point, decimal,
and exception operations.
Linear builder state is explicit in VIR and cannot cross a merge without
identical ownership/version state; freeze lowers it to an immutable value. Each
operation has one shared meaning and a profile allowlist; C#-specific behavior
that cannot share that meaning remains in a C#-named operation/profile rule.

Exceptional control is explicit rather than encoded as a missing safety check.
Unchecked access is never implied by the presence of a catch. For an uncaught
exception forbidden by contract, VC generation proves the exceptional edge
unreachable. For an allowed or caught exception, VC generation proves the
corresponding handler or exceptional postcondition.

Loops continue to use cyclic CFGs and explicit loop contracts. Iterators and
async methods add suspension states only in the source projection; after the
closed equivalence checks they lower to sequence/task-result control forms
with no ambient scheduler.

### 20.3 Certificate encoding

Encode option/lookup/result/validation/boundary-presence/transition, immutable
records, bounded sequences, ordered maps/sets, business primitives, canonical
boundary-codec relations, float, decimal, and closed exception outcomes using
ordinary checked core terms and definitions over the existing Bool/BV/array/
struct foundations. The installed
`mpk.program_certificate.alpha.v1` assembly profile preserves the
`PROGRAM_CERTIFICATE_ALPHA_V0.md` acceptance rules: the root retains
`proof_node_table: []` and `theory_certificates: []`, contains no
`TheoryPrimitive` declaration or `Theory` node, and has a recomputed total
axiom count of zero. Untrusted generators may optimize construction, but both
checkers receive the same complete ordinary-term Certificate v0 bytes and
recompute every declaration and axiom report.

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
| Sequence/map/set builder capacity per value | 4,096 |
| Builders per method / simultaneously live | 32 / 8 |
| Ordered map/set entries per value | 4,096 |
| Total collection cells represented by one request | 65,536 |
| UTF-16 units per string value | 16,384 |
| Option/lookup/result/validation/boundary-presence nesting | 16 |
| Validation errors per result | 256 |
| Boundary fields / nesting / canonical bytes | 256 / 32 / 1,048,576 |
| Events emitted by one transition | 4,096 |
| Loops per method / nesting | 32 / 8 |
| Invariants plus decreases per loop | 64 |
| Switch arms per method | 256 |
| Pattern nesting | 16 |
| Catch/finally regions per method | 32 |
| Source exception types per compilation | 32 |
| Yield sites per iterator | 64 |
| Await sites per method / closed async call depth | 64 / 64 |
| Bounded-quantifier nesting | 4 |

Existing source, closure, operation, CFG, contract, diagnostic, artifact, and
process limits remain ceilings unless the freeze provides reproducible memory,
time, and output evidence for a changed successor limit. Structural and
transport counters reject boundary-plus-one before allocating or retaining the
excess. Untrusted boundary documents and parser inputs follow their specified
pre-invocation rejection or `ParseError` paths. Within admitted source
execution, run-time array, string, builder, map/set, validation-error,
transition-event, and yielded-sequence maxima instead become explicit value
predicates and VCs; an unproved bound blocks verified acceptance and never
invents a source-language exception. All counter arithmetic is checked, and
semantic value limits remain distinct from encoded byte limits.

## 22. Diagnostics and fail-closed precedence

Do not reuse a scalar-v0 code with a broader meaning. The practical profile
needs its own closed families, provisionally:

```text
CSHARP_PRACTICAL_DECLARATION
CSHARP_PRACTICAL_TYPE
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
CSHARP_PRACTICAL_ITERATOR
CSHARP_PRACTICAL_ASYNC
CSHARP_PRACTICAL_BOUNDARY
CSHARP_PRACTICAL_TRANSITION
CSHARP_PRACTICAL_EFFECT
CSHARP_PRACTICAL_LIBRARY
CSHARP_PRACTICAL_LOWERING
```

Capture/source/metadata/typecheck failures precede subset failures; subset and
contract validation precede lowering; lowering precedes emission. Within a
method, validate declarations/types, construction state, ownership/builders,
ordinary operations, control contracts, exceptions, iterator/async closure,
then artifact mapping. Boundary shape/size/canonicality validation precedes
typed conversion and any selected method launch. The normative vector must
freeze every ambiguous multi-failure case.

Public messages remain bounded and sanitized. Raw compiler prose, source
snippets, exception messages, host paths, task/state-machine names, culture,
and runtime stack text never enter public artifacts.

## 23. Conformance and verification strategy

The freeze and implementation gates must include:

| Capability | Required evidence |
| --- | --- |
| Expression bodies / `var` | same normalized VIR and obligations as the explicit block/type form; malformed and ambiguous forms reject |
| Data types / construction | positive structural cases, enum unknown/cast/zero-default cases, recursive default-eligible/ineligible cases, acyclic constructor delegation, receiver-first pure instance-call lowering, constructor-only and ordered init/required/object-initializer cases, construction/public invariant proofs, attribute bypass and all mutation/identity/inheritance escapes reject |
| Arrays / builders / sequences | structural rejection versus symbolic bound obligations, default-eligible length allocation versus fully initialized non-defaultable elements, boundary lengths and indices, active-foreach mutation, every linear ownership/freeze/use-after-freeze path, filtered variable-result construction |
| Ordered map/set | exact immutable/builder API transition, read, count, contains, and ordered-enumeration matrices; key-order matrix, duplicate add/replace, lookup, bound preservation, and rejected float keys/comparers/hash/insertion-order dependencies |
| Strings / codecs | exact string/string and string/char concat matrix, restricted interpolation equivalence and rejected alignment/format/non-string/non-char holes, rejected char/char and object conversion, null/empty concat and equality, intrinsic-only ordinal arguments, null receivers/arguments, UTF-16/surrogates, every exact parse/format grammar and noncanonical/range mutation, lossless round-trip plus fixed-scale rounded-value laws, and pinned-runtime differential corpus |
| Float / decimal | exhaustive small-domain properties plus bit/rounding/overflow/NaN/signed-zero differential vectors against the pinned runtime |
| Nullable / lookup / results / validation | all option/lookup and tagged-sum transitions, nested-option rejection and the exact lookup-versus-null exception, missing-key versus stored-null lookup, active/inactive payloads and empty-invalid exception, default-ineligible fallback rejection, annotations versus runtime null, deterministic error accumulation/order/bounds, exhaustive matching |
| Business values | calendar boundaries/leap days and exact day-of-week enum, time wrap/carry, duration/instant precision and difference-range errors, GUID comparison/codecs/no-generation, Money creation/add/subtract/rate/division, currency/scale/rounding/error precedence, and canonical-storage-versus-business comparison cases |
| Structural equality/order | every admitted recursive type, null/decimal/GUID corner, lexicographic cases, NaN preservation and rejected non-total keys |
| Loops | invariant initialization/preservation/exit, decreases, break/continue, nested loops, partial-versus-total evidence |
| Switch / patterns | source-order arms and guards, exhaustiveness, null/property/list cases, Roslyn decision-graph upgrade vectors |
| Exceptions | built-in and explicit throws, lexical filter-before-finally search, inner-to-outer unwind, filter throws, ordered catch/finally propagation, exceptional contracts, uncaught rejection/obligations |
| Iterators | laziness, capture, per-yield/prefix/completion contracts, yield/break/exception/disposal order, early consumer break, rejected alias/re-enumeration/concurrent use/escape |
| Async | immediate/completed-task observation probes, rejected delayed/custom-task probes, result/exception propagation, erasure equivalence, every task/effect escape rejection |
| Boundary values | duplicate/unknown and required/non-null/three-state missing/null cases, numeric/text canonicality, depth/count/byte limits, raw-input/canonical-value/output-reparse linkage, serializer/runtime mutation |
| State transitions | invariant and version preservation, accepted/error arms, ordered bounded events and response relation, explicit-time and optimistic-conflict behavior, idempotency replay/command-encoding mismatch/history-capacity and precedence cases |

Every accepted source case runs twice from isolated builds and emits identical
canonical artifacts. Independent evaluators compare MPK VIR behavior with the
pinned C# runtime for finite test domains. Bounded fuzzing owns parser,
contract, Roslyn adapter, pattern, exception-region, collection, codec,
calendar, boundary, transition, iterator, async, and artifact protocols.
Mutation suites cross every profile/schema/context/hash boundary.

The existing `fixtures/csharp/policy/source/src/Required.cs` expression-body
mismatch must be repaired before using that fixture as frontend evidence. The
replacement fixture must run through the real installed C# frontend rather
than attach a manually constructed VIR to source bytes. Regenerate all linked
scan, evidence, certificate, source-map, manifest, and hash bytes in one
reviewed change. Before activation, a verified replacement may exist only as
private migration evidence; the tracked `fixtures/csharp/policy/` files change
together in the atomic release commit.

Add three general-facing, runnable end-to-end C# examples:

1. **Invoice pricing and tax:** immutable request/result plus `Money`, currency
   and scale checks, business/effective dates, decimal rounding, ordered line
   aggregation, and a bounded sequence builder.
2. **Order state transition:** GUID command/idempotency keys, explicit instant
   and expected version, switch/pattern state logic, `Result`, one caught
   allowlisted exception, replay-safe response, and an ordered bounded event
   sequence.
3. **Batch input validation:** canonical boundary JSON, missing versus null,
   exact parse/format, ordered map/set duplicate handling, accumulating
   `Validation`, a closed iterator, and an immediate-await wrapper.

Across the three examples, include constructor-only and
`required`/`init`/object-initializer construction, array and string operations,
loop invariants/decreases, structural equality/order, nullable data, and every
new business primitive. Each example documents what the certificate proves and
what remains an untrusted serializer, identity/time source, persistence, or
transport claim.

An example's source and artifacts may be checked in only after `mpk policy
scan` and `mpk policy verify` process its actual source through the installed
frontend, its boundary bytes round-trip through the canonical value, and both
checkers accept the same certificate bytes. Before T08-W10, “installed
frontend” means the exact privately materialized candidate image: a checked-in
example remains rehearsal-only, is absent from the active installed release
and public routing/documentation, and does not activate a production tuple.
T08-W10 atomically installs and advertises the already verified examples.

## 24. Implementation stages and gates

The implementation-sized work items, dependencies, owners, exit gates, and
verification commands are maintained in
[`08_csharp_practical_subset_design-todo.md`](08_csharp_practical_subset_design-todo.md).

CSHARP-03 is implemented serially behind private entrypoints:

```text
CSHARP-03-T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08
```

1. **CSHARP-03-T01 — Feasibility and freeze.** Pin toolchain versions; measure
   exact public Roslyn shapes and .NET behavior; freeze collection/option/
   lookup/result/validation/boundary-presence APIs, recursive default
   eligibility, construction lowering, equality/key ordering, Money operations
   and errors, calendar/GUID/codecs, boundary/state-transition schemas,
   specification, vectors, identities, limits, and a traceability ledger. Any
   unresolved compiler/runtime fact is a stop condition.
2. **CSHARP-03-T02 — Shared artifact foundation.** Implement the one successor
   registry, VIR type/operation/exception, sequence/map/set and tagged-sum
   vocabulary, business-value/codecs, three-state boundary presence, boundary
   linkage, source-map, manifest, VC/skeleton, hash, and VIR-importer boundary.
   Migrate predecessor producers/consumers privately and prove semantic
   equivalence.
3. **CSHARP-03-T03 — Data frontend.** Add concise syntax, immutable types,
   constructors, fields/properties, init/required/object initializers,
   structural equality/order, arrays/builders/sequences/maps/sets,
   strings/codecs, float/decimal, nullable/option/lookup/result/validation/
   boundary-presence, and business primitives with complete negative vectors.
4. **CSHARP-03-T04 — Control frontend.** Add loops/contracts, switch/patterns,
   and explicit exceptional CFGs and handlers.
5. **CSHARP-03-T05 — Suspension frontend.** Add iterators and the closed
   async/await projection, including laziness and erasure-equivalence proofs.
6. **CSHARP-03-T06 — Verification integration.** Add expanded contract
   expressions, construction/type/state invariants, exceptional/loop/
   collection/codec/transition obligations, canonical boundary round trips,
   ordinary core definitions and proof terms, policy/evidence/AI/API linkage,
   and same-byte dual-checker certificates.
7. **CSHARP-03-T07 — Release candidate.** Build reproducibly, register
   immutable frontend and toolchain bundles, run hostile-environment and native
   sandbox gates, and add no ambient library/effect route.
8. **CSHARP-03-T08 — Complete rehearsal and activation.** Run the complete
   predecessor and practical corpus twice, fix review findings to zero,
   generate and verify all three runnable business examples, then atomically
   install the sole successor release.

No stage creates a public compatibility flag or partial practical-profile
route. If any requested capability—including iterator or async—fails
feasibility, the profile remains inactive; the design is revised and re-frozen
rather than claiming partial completion under the same ID.

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
  calendar/GUID/codec, boundary-presence/transition, exception, iterator, and
  async encodings retain an empty proof-node table, empty theory-certificate
  table, no theory primitive/node, and a recomputed total axiom count of zero;
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

- exact Roslyn `IOperation`, CFG region, pattern, nullable, iterator, async, and
  synthesized/init/required/object-initializer shapes for the pinned compiler;
- exact .NET float/decimal result bits, scale, rounding, exception, string,
  date/calendar, time/duration, GUID comparison/codec, and task/iterator
  observations at every selected edge;
- the recursive default-eligibility matrix for every admitted semantic type;
- the final sequence/map/set/lookup/result/validation/boundary-presence source
  API, Money operation/error API, key-order matrix, string/char concatenation
  matrix, parse/format grammars, day-of-week enum mapping, instant granularity
  and difference range, construction invariants, transition error/precedence
  rules, and boundary/state-transition schemas;
- the final artifact/schema/profile names and hashes;
- the exact deterministic limits that both checkers can sustain; or
- whether any proposed intrinsic needs a smaller initial allowlist.

Each fact needs a disposable public-API/runtime probe, checked-in canonical
result, independent implementation owner, and upgrade mutation. A failed or
ambiguous probe narrows or blocks the feature; it never falls back to compiler
trust or an axiom.

## 27. Primary references

- [C# types, strings, decimal, and nullability](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types)
- [C# arrays](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/arrays)
- [C# expressions and await](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions)
- [C# statements, loops, switch, try/catch/finally](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/statements)
- [C# patterns](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/patterns)
- [C# exceptions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/exceptions)
- [C# classes and iterator semantics](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes)
- [C# `init` reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/init)
- [C# `required` reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/required)
- [.NET date/time parsing and business-date type guidance](https://learn.microsoft.com/en-us/dotnet/standard/base-types/parsing-datetime)
- [`DateOnly` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly?view=net-10.0)
- [`TimeOnly` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.timeonly?view=net-10.0)
- [`TimeSpan` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.timespan?view=net-10.0)
- [`Guid.TryParseExact`](https://learn.microsoft.com/en-us/dotnet/api/system.guid.tryparseexact?view=net-10.0)
- [`Guid.CompareTo`](https://learn.microsoft.com/en-us/dotnet/api/system.guid.compareto?view=net-10.0)
- [`ImmutableArray<T>` API reference](https://learn.microsoft.com/en-us/dotnet/api/system.collections.immutable.immutablearray-1?view=net-10.0)
- [`CSHARP_PROFILE_V0.md`](../specs/CSHARP_PROFILE_V0.md)
- [`SEMANTIC_PROFILE_REGISTRY_V1.md`](../specs/SEMANTIC_PROFILE_REGISTRY_V1.md)
- [`PROGRAM_CERTIFICATE_ALPHA_V0.md`](../specs/PROGRAM_CERTIFICATE_ALPHA_V0.md)
- [`06_multilanguage_frontend_design.md`](06_multilanguage_frontend_design.md)
- [General-facing sample/subset guide](../../docs/csharp-samples-and-subset.md)
