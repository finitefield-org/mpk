# C# Practical Subset Expansion Design

Status: proposed governance and implementation design. This document does not
change the active `mpk.csharp.scalar.v0` profile, register a new profile, or
authorize a public route. The active release remains registry revision 2 with
Go, Rust, and C# scalar support; Java activation remains owned by
`JAVA-03-T10`.

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
| Domain data | source-defined enums, immutable structs, and sealed immutable classes with fields, properties, constructors, and pure instance methods |
| Collections | bounded one-dimensional arrays, reads, owned local construction/update, `Length`, and `foreach` |
| Text | bounded UTF-16 strings and an exact ordinal-only operation allowlist |
| Numbers | existing integers plus exact `float`, `double`, and .NET `decimal` semantics |
| Absence | nullable value types and nullable string/array/class references with explicit proof obligations |
| Control flow | `while`, `do`, `for`, `foreach`, `break`, `continue`, switch statements/expressions, and closed patterns |
| Failure | explicit throws, built-in operation exceptions, typed catch, pure filters, and `finally` under a closed exception model |
| Lazy flow | source iterators whose values do not escape the admitted closure |
| Async flow | sequential `Task`/`Task<T>` and closed async iterators whose scheduling and external effects are unobservable |

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

Preparing this design is a governance amendment, not the start of CSHARP-03
specification or implementation. No normative freeze, production parser,
registry entry, bundle, or public activation may start before `JAVA-03-T10`
completes. DART-04 then waits for the complete CSHARP-03 release gate. This
insertion records the user value of making the already released C# frontend
useful for business-domain logic before adding another language; it does not
authorize parallel language work.

## 3. Goals and non-goals

### 3.1 Goals

- Let users express ordinary immutable domain values rather than flatten every
  value to unrelated scalar parameters.
- Cover deterministic validation, pricing, eligibility, aggregation, and
  transformation code over bounded data.
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
  hierarchy and compiler-recognized `Task<T>` and iterator protocols described
  below;
- user-defined operators/conversions, delegates, lambdas, expression trees,
  LINQ, reflection, `dynamic`, records, record structs, `with` expressions,
  open generics, or runtime code generation;
- `List<T>`, dictionaries, spans, arbitrary framework collections, array
  covariance, multidimensional arrays, or jagged arrays;
- culture-sensitive text, normalization, regular expressions, formatting,
  resources, globalization, or ambient locale;
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

- run-time-length arrays and UTF-16 strings;
- nullable/optional values;
- IEEE binary32/binary64 and .NET decimal operations;
- normal and exceptional successors from one source operation;
- exception propagation and handlers;
- iterator yield/suspension state; and
- the contract expressions needed for sequence, field, null, decimal,
  floating-point, and exceptional postconditions.

Encoding these as profile-specific strings inside existing fields is
forbidden. The practical profile therefore requires a successor shared
artifact family. Working names are:

| Role | Working successor identity |
| --- | --- |
| Semantic profile | `mpk.csharp.practical.v1` |
| Parameters | `mpk.semantic_parameters.csharp_practical.v1` |
| Selection | `mpk.selection.csharp_members.v1` |
| Method/type contracts | `mpk.csharp.contract.v1`, `mpk.csharp.type_contract.v1` |
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
- every method and type-contract sidecar; and
- no executable, reference, package, registry, or toolchain path.

The closure begins at the selected roots and includes every source-declared
type, constructor, property getter, instance/static method, iterator, and async
method reachable through admitted calls and field/property types. Type and call
graphs are finite and bounded, and the call graph remains acyclic; direct or
mutual recursion rejects. Every ordinary declaration in a selected source file
must belong to that closed compilation and satisfy the profile; an
unrelated, unreachable, or unselected type/member does not become ignored or
trusted and rejects.

Partial declarations, source generators, metadata user types, nested generic
types, and conditional compilation remain rejected. Compiler-synthesized
members may be observed only for an exact frozen pattern and are never used as
the semantic definition of an admitted source feature.

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
`System.Enum` APIs remain excluded.

Admit non-generic `readonly struct` declarations whose complete field graph is
acyclic and contains only admitted scalar, decimal, floating-point, nullable,
string, sequence, enum, or earlier structural types. Instance fields are
explicitly `readonly`; properties are getter-only; methods are pure and
nonvirtual. Custom layout, fixed buffers, ref fields, events, indexers,
destructors, operators, conversions, and boxing reject.

“User-defined value type” is structural rather than a hard-coded name or shape
allowlist: every source-defined enum or non-generic `readonly struct` satisfying
these rules is eligible. It does not mean arbitrary mutable, unsafe, generic,
explicit-layout, or runtime-provided CLR value types.

Structs lower to named structural values. Declaration and field order are
canonical source order, while type declarations are emitted in dependency
order. Default values are admitted only after the specification freezes the
exact zero/null default for every reachable field.

### 8.2 Sealed immutable classes

Admit ordinary non-generic `sealed class` declarations as immutable value
objects under these restrictions. Source exception declarations use the
separate closed-hierarchy rule in section 15:

- the only base type is `object`;
- all instance state is in source-declared readonly fields or getter-only
  properties;
- the complete reachable type and value graph is acyclic and deterministically
  bounded;
- no static mutable state, finalizer, event, indexer, virtual member,
  interface, reflection, monitor, or identity API exists;
- constructors establish every member exactly once on every normal path;
- `this` does not escape during construction;
- methods do not mutate the receiver or any reachable value; and
- class/reference equality, `ReferenceEquals`, `GetHashCode`, runtime type
  inspection, and identity-sensitive collections reject.

With identity and mutation unobservable, a non-null instance lowers to a
structural VIR value. This is an explicit profile theorem, not an assumption
that all C# classes have value semantics. A later mutable-heap profile would
need a separate heap, alias, frame, and allocation model.

### 8.3 Fields, properties, and constructors

Allow `private`, `internal`, or `public` readonly instance fields and
getter-only properties. An auto-property is accepted only when its exact
compiler-synthesized backing-field shape is frozen and cross-checked; an
explicit getter must be a pure admitted expression or block. `set`, `init`,
lazy getters, cached values, and property side effects reject.

Constructors may take admitted value parameters, delegate to one acyclic
constructor in the same type, assign members, validate arguments, and throw an
admitted exception. Base-constructor behavior is only the inert `object`
constructor. Every normal constructor exit must establish the type invariant;
every exceptional exit produces no object value. Object initializers reject.

Pure instance methods lower to direct functions with the receiver as the first
argument. Calls are statically resolved to one source declaration; there is no
virtual dispatch.

## 9. Arrays and ownership

Admit zero-based, one-dimensional `T[]` where `T` is an admitted value and the
run-time length is within the profile maximum. The array itself is a reference
at the C# boundary, so nullability is modeled separately. Multidimensional,
jagged, covariant, `System.Array`, `Span<T>`, `Memory<T>`, and arbitrary
collection/interface conversions reject.

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

## 10. Strings and characters

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
- bounded `Concat`/`+` over string and char operands;
- bounded `Substring` with explicit start and length;
- `IsNullOrEmpty`; and
- switch constant matching under exact ordinal, case-sensitive semantics.

Interpolation is accepted only when every interpolation is already string or
char, has no alignment/format component, and normalizes to bounded
concatenation. Numeric/date formatting, parsing, case conversion, trimming,
culture, normalization, comparison without an exact ordinal overload,
interning, identity, and arbitrary `System.String` methods reject.

`StringComparison.Ordinal` is one profile-recognized intrinsic constant, not
admission of the framework enum or its other members. It cannot be stored,
returned, converted, or passed anywhere except the exact allowlisted argument
position.

For `Concat` and `+`, a null string operand contributes the empty sequence and
the result is non-null. String equality distinguishes null from empty and uses
the exact C# ordinal value rules; ordinal `Compare` also preserves the defined
null ordering. An instance call on a null receiver has its
`NullReferenceException` edge. Where an exact chosen overload rejects a null
argument or invalid index/range, it has the corresponding allowlisted C#
exception subtype; overloads defined to accept null retain that behavior. A
result longer than the semantic maximum is a bound obligation, not an invented
runtime exception. Catchable resource exhaustion remains outside the profile.
Limits count UTF-16 units and encoded artifact bytes separately.

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

The VIR value is exact IEEE bits. Signed zero is preserved. NaN payload and
quieting behavior are either preserved exactly or canonicalized only after the
freeze proves that no admitted operation can observe the difference. The VC
encoder produces ordinary checked core/BV definitions and proof terms within
the current program-certificate alpha acceptance profile. It emits no theory
certificate, floating-point axiom, or checker primitive.

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
- no user-defined conversion, float/double conversion, formatting, parsing,
  currency, culture, or representation inspection.

Trailing-zero scale is normalized only if every admitted observation is proven
value-based. `decimal.GetBits`, formatting, hash codes, and APIs that expose
representation remain rejected. MPK-owned arithmetic is differentially tested
against the exact pinned .NET runtime but is not defined by accepting the
runtime's answer.

## 12. Nullable values and references

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
matrix, and matching patterns.

Dereference or `Value` access without a proven present branch emits the exact
null/invalid-operation exceptional edge. The null-forgiving operator `!` is
accepted only if MPK's own dataflow already proves non-null and therefore it
has no semantic effect; otherwise it rejects. Nested option forms and nullable
by-reference storage reject.

## 13. Loops and loop contracts

Admit `while`, `do`, `for`, and `foreach`, plus structured `break` and
`continue`. `goto`, labels, unsafe jumps, and irreducible control flow remain
rejected. `foreach` initially accepts only admitted arrays, strings, and closed
source iterators.

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
string, nullable, and immutable structural values. Evaluate the governing
expression once and preserve first-applicable-arm and guard order.

The closed initial pattern set is:

- constant, discard, and `var` patterns;
- `null` / `not null`;
- relational and parenthesized patterns;
- `and`, `or`, and `not` logical patterns;
- declaration/type patterns over the finite admitted sealed type set;
- property patterns whose getters are total and pure; and
- bounded list patterns over admitted arrays after exact sequence semantics are
  frozen.

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
- every produced task is immediately awaited or returned as the one method
  task; tasks are not stored, compared, combined, raced, or inspected;
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

## 18. Contract design

Use separate closed sidecars for types and callable members.

A type contract contains exactly its schema/profile/type identity, ordered
field/property identities, and invariants. A constructor proves the invariant
on normal return. Every instance method may assume the receiver invariant and
must preserve it because receiver mutation is forbidden.

A method/constructor contract contains:

- ordered `requires` and normal `ensures`;
- ordered exceptional cases with an exact exception type, path condition, and
  exceptional postconditions;
- an empty externally visible `modifies` set; owned local sequence updates are
  represented in the loop records but cannot escape under an alias;
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
string/char, float, decimal, enum, option construction/tests, exception kind
and payload, and bounded `forall`/`exists`. Every expression is closed,
explicitly typed, depth/count bounded, and free of source method calls. There
is no arbitrary C# expression evaluator inside a contract parser.

Contracts cannot hide an external service assumption. A future effect-summary
mechanism would need an explicit axiom/theory policy and is outside this
profile.

## 19. VIR and VC design

### 19.1 Required value vocabulary

The successor VIR needs closed values for:

- Bool and existing fixed-width bit vectors;
- IEEE binary32/binary64 bits;
- .NET decimal value representation;
- option/presence;
- bounded variable-length sequence;
- named immutable struct/class projection;
- closed exception values; and
- unit for `Task`/void-like internal continuations.

Enums lower to their exact integer carrier plus a declaration identity.
Strings are a distinct profile mapping over a UTF-16 sequence so arbitrary
sequence operations cannot masquerade as string semantics.

### 19.2 Required operation/control vocabulary

Add explicit construction, field, option, sequence, string, floating-point,
decimal, and exception operations. Each operation has one shared meaning and a
profile allowlist; C#-specific behavior that cannot share that meaning remains
in a C#-named operation/profile rule.

Exceptional control is explicit rather than encoded as a missing safety check.
Unchecked access is never implied by the presence of a catch. For an uncaught
exception forbidden by contract, VC generation proves the exceptional edge
unreachable. For an allowed or caught exception, VC generation proves the
corresponding handler or exceptional postcondition.

Loops continue to use cyclic CFGs and explicit loop contracts. Iterators and
async methods add suspension states only in the source projection; after the
closed equivalence checks they lower to sequence/task-result control forms
with no ambient scheduler.

### 19.3 Certificate encoding

Encode option, immutable records, bounded sequences, float, and decimal using
ordinary checked core terms and definitions over the existing
Bool/BV/array/struct foundations. The installed
`mpk.program_certificate.alpha.v1` assembly profile preserves the
`PROGRAM_CERTIFICATE_ALPHA_V0.md` acceptance rules: the root retains
`proof_node_table: []` and `theory_certificates: []`, contains no
`TheoryPrimitive` declaration or `Theory` node, and has a recomputed total
axiom count of zero. Untrusted generators may optimize construction, but both
checkers receive the same complete ordinary-term Certificate v0 bytes and
recompute every declaration and axiom report.

If float/decimal/sequence definitions exceed practical checker limits, the
phase stops. A separate, prior governance/checker project would be required to
change this acceptance boundary, and this profile cannot activate while such a
dependency is unresolved. It must not add a `CSharpSemanticsAxiom`, theory
shortcut, trusted primitive, runtime answer, or omitted obligation.

## 20. Provisional deterministic limits

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
| Total sequence cells represented by one request | 65,536 |
| UTF-16 units per string value | 16,384 |
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
excess. Run-time array, string, and yielded-sequence maxima instead become
explicit value predicates and VCs; an unproved bound blocks verified
acceptance and never invents a source-language exception. All counter
arithmetic is checked, and semantic value limits remain distinct from encoded
byte limits.

## 21. Diagnostics and fail-closed precedence

Do not reuse a scalar-v0 code with a broader meaning. The practical profile
needs its own closed families, provisionally:

```text
CSHARP_PRACTICAL_DECLARATION
CSHARP_PRACTICAL_TYPE
CSHARP_PRACTICAL_OBJECT
CSHARP_PRACTICAL_OWNERSHIP
CSHARP_PRACTICAL_ARRAY
CSHARP_PRACTICAL_STRING
CSHARP_PRACTICAL_FLOAT
CSHARP_PRACTICAL_DECIMAL
CSHARP_PRACTICAL_NULLABLE
CSHARP_PRACTICAL_LOOP_CONTRACT
CSHARP_PRACTICAL_SWITCH
CSHARP_PRACTICAL_PATTERN
CSHARP_PRACTICAL_EXCEPTION
CSHARP_PRACTICAL_ITERATOR
CSHARP_PRACTICAL_ASYNC
CSHARP_PRACTICAL_EFFECT
CSHARP_PRACTICAL_LIBRARY
CSHARP_PRACTICAL_LOWERING
```

Capture/source/metadata/typecheck failures precede subset failures; subset and
contract validation precede lowering; lowering precedes emission. Within a
method, validate declarations/types, ownership, ordinary operations, control
contracts, exceptions, iterator/async closure, then artifact mapping. The
normative vector must freeze every ambiguous multi-failure case.

Public messages remain bounded and sanitized. Raw compiler prose, source
snippets, exception messages, host paths, task/state-machine names, culture,
and runtime stack text never enter public artifacts.

## 22. Conformance and verification strategy

The freeze and implementation gates must include:

| Capability | Required evidence |
| --- | --- |
| Expression bodies / `var` | same normalized VIR and obligations as the explicit block/type form; malformed and ambiguous forms reject |
| Data types / constructors | positive structural cases, initialization and invariant proofs, all mutation/identity/inheritance escapes reject |
| Arrays / strings | structural rejection versus symbolic bound obligations, boundary lengths and indices, active-foreach mutation, alias/ownership mutations, null/empty concat and equality, intrinsic-only ordinal arguments, null receivers/arguments, UTF-16/surrogate cases, ordinal runtime differential corpus |
| Float / decimal | exhaustive small-domain properties plus bit/rounding/overflow/NaN/signed-zero differential vectors against the pinned runtime |
| Nullable | all option transitions, annotations versus runtime null, dereference/`Value` exception edges, flow-merge cases |
| Loops | invariant initialization/preservation/exit, decreases, break/continue, nested loops, partial-versus-total evidence |
| Switch / patterns | source-order arms and guards, exhaustiveness, null/property/list cases, Roslyn decision-graph upgrade vectors |
| Exceptions | built-in and explicit throws, lexical filter-before-finally search, inner-to-outer unwind, filter throws, ordered catch/finally propagation, exceptional contracts, uncaught rejection/obligations |
| Iterators | laziness, capture, per-yield/prefix/completion contracts, yield/break/exception/disposal order, early consumer break, rejected alias/re-enumeration/concurrent use/escape |
| Async | immediate/completed-task observation probes, rejected delayed/custom-task probes, result/exception propagation, erasure equivalence, every task/effect escape rejection |

Every accepted source case runs twice from isolated builds and emits identical
canonical artifacts. Independent evaluators compare MPK VIR behavior with the
pinned C# runtime for finite test domains. Bounded fuzzing owns parser,
contract, Roslyn adapter, pattern, exception-region, iterator, async, and
artifact protocols. Mutation suites cross every profile/schema/context/hash
boundary.

The existing `fixtures/csharp/policy/source/src/Required.cs` expression-body
mismatch must be repaired before using that fixture as frontend evidence. The
replacement fixture must run through the real installed C# frontend rather
than attach a manually constructed VIR to source bytes. Regenerate all linked
scan, evidence, certificate, source-map, manifest, and hash bytes in one
reviewed change.

Add one general-facing, runnable C# example that exercises, at minimum:

- an immutable request/result type and constructor invariant;
- nullable string data;
- an array aggregation loop with explicit invariant and decrease;
- decimal arithmetic and rounding;
- a switch/property/null pattern;
- one caught domain exception;
- one closed iterator; and
- one immediate-await async wrapper.

The example is not published until `mpk policy scan` and `verify` process its
actual source through the installed frontend and both checkers accept the same
certificate bytes.

## 23. Implementation stages and gates

CSHARP-03 is implemented serially behind private entrypoints:

```text
CSHARP-03-T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08
```

1. **CSHARP-03-T01 — Feasibility and freeze.** Pin toolchain versions; measure
   exact public Roslyn shapes and .NET behavior; freeze specification, vectors,
   identities, limits, and a traceability ledger. Any unresolved
   compiler/runtime fact is a stop condition.
2. **CSHARP-03-T02 — Shared artifact foundation.** Implement the one successor
   registry, VIR/type/operation/exception, source-map, manifest, VC/skeleton,
   hash, and VIR-importer boundary. Migrate predecessor producers/consumers
   privately and prove semantic equivalence.
3. **CSHARP-03-T03 — Data frontend.** Add concise syntax, immutable types,
   constructors, fields/properties, arrays/ownership, strings/chars,
   float/decimal, and nullable analysis with complete negative vectors.
4. **CSHARP-03-T04 — Control frontend.** Add loops/contracts, switch/patterns,
   and explicit exceptional CFGs and handlers.
5. **CSHARP-03-T05 — Suspension frontend.** Add iterators and the closed
   async/await projection, including laziness and erasure-equivalence proofs.
6. **CSHARP-03-T06 — Verification integration.** Add expanded contract
   expressions, type invariants, exceptional/loop/sequence obligations,
   ordinary core definitions and proof terms, policy/evidence/AI/API linkage,
   and same-byte dual-checker certificates.
7. **CSHARP-03-T07 — Release candidate.** Build reproducibly, register
   immutable frontend and toolchain bundles, run hostile-environment and native
   sandbox gates, and add no ambient library/effect route.
8. **CSHARP-03-T08 — Complete rehearsal and activation.** Run the complete
   predecessor and practical corpus twice, fix review findings to zero,
   generate the runnable sample, then atomically install the sole successor
   release.

No stage creates a public compatibility flag or partial practical-profile
route. If iterator or async feasibility fails, the profile remains inactive;
the design is revised and re-frozen rather than claiming partial completion
under the same ID.

## 24. Release acceptance criteria

Activation requires all of the following:

- every requested capability in section 1 has positive, negative, boundary,
  differential, deterministic, and upgrade evidence;
- all old C# scalar accepted/rejected cases remain semantically identical;
- all active Go, Rust, and Java cases retain their source behavior,
  obligations, checker verdicts, and axiom categories;
- Certificate v0 and both checker acceptance rules are unchanged;
- float, decimal, sequence, exception, iterator, and async encodings retain an
  empty proof-node table, empty theory-certificate table, no theory
  primitive/node, and a recomputed total axiom count of zero;
- actual installed-source samples, not constructed helper IR, pass scan,
  verification, evidence reproduction, and both checkers;
- the candidate passes the local native Linux assembly, image mutation,
  syscall, cgroup, resource, cleanup, and two-pass installed release gates;
- `./scripts/check-fast.sh` and the complete local release gate pass without
  GitHub Actions or workflow dependency;
- documentation states the exact remaining exclusions and does not call the
  profile full C# support; and
- review produces no findings before commit and push.

## 25. Freeze-time facts that must be measured

The design intentionally does not guess:

- exact Roslyn `IOperation`, CFG region, pattern, nullable, iterator, async, and
  synthesized-member shapes for the pinned compiler;
- exact .NET float/decimal result bits, scale, rounding, exception, string, and
  task/iterator observations at every selected edge;
- the final artifact/schema/profile names and hashes;
- the exact deterministic limits that both checkers can sustain; or
- whether any proposed intrinsic needs a smaller initial allowlist.

Each fact needs a disposable public-API/runtime probe, checked-in canonical
result, independent implementation owner, and upgrade mutation. A failed or
ambiguous probe narrows or blocks the feature; it never falls back to compiler
trust or an axiom.

## 26. Primary references

- [C# types, strings, decimal, and nullability](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types)
- [C# arrays](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/arrays)
- [C# expressions and await](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions)
- [C# statements, loops, switch, try/catch/finally](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/statements)
- [C# patterns](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/patterns)
- [C# exceptions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/exceptions)
- [C# classes and iterator semantics](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes)
- [`CSHARP_PROFILE_V0.md`](../specs/CSHARP_PROFILE_V0.md)
- [`SEMANTIC_PROFILE_REGISTRY_V1.md`](../specs/SEMANTIC_PROFILE_REGISTRY_V1.md)
- [`PROGRAM_CERTIFICATE_ALPHA_V0.md`](../specs/PROGRAM_CERTIFICATE_ALPHA_V0.md)
- [`06_multilanguage_frontend_design.md`](06_multilanguage_frontend_design.md)
- [General-facing sample/subset guide](../../docs/csharp-samples-and-subset.md)
