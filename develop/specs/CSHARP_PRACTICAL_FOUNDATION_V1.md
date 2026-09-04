# C# practical foundation v1

Status: CSHARP-03-T01-W08 **candidate semantic freeze**. This document does not
activate a profile, install a descriptor, widen the scalar profile, implement a
frontend, or certify a program. The design authority is
`develop/docs/08_csharp_practical_subset_design.md`, sections 6, 8–12 and 15.
T01-W09 owns the containing sidecar unions, complete successor context and
diagnostic/limit registry; T01-W10 owns the assembled freeze package. They must
consume, not silently redefine, the foundation-local rules below.

The conformance schema is `mpk.csharp.practical.foundation.conformance.v1` at
`develop/specs/vectors/csharp-practical-foundation-v1.json`. Its primary owner is
`crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W08`.
`develop/probes/csharp-03/foundation_model.py` and `foundation_package.py` are
disposable, executable specification models, never production dependencies.
Model success and finite runtime agreement are not proofs of source invariants.

## 1. Closed candidate and trust boundary

There is exactly one candidate, `mpk.csharp.practical.foundation.v1`, version 1,
for `mpk.csharp.practical.v1`. Its descriptor is
`develop/migrations/csharp-03/foundation/foundation-descriptor.json`; its closed
definition inventory is
`develop/migrations/csharp-03/foundation/foundation-definitions.json`.
The descriptor binds both that inventory and this normative document by raw
SHA-256 and byte length. The inventory describes monomorphization and checked
definition recipes, not imported axioms or source-callable runtime code.
Later release registration binds this exact semantic descriptor to the compiled
checked definitions and executable closure; release bytes are not part of a
self-referential semantic descriptor hash.

Applications keep ordinary non-generic C# declarations, arrays, methods, and
sidecar contracts. There is no `Mpk.*` import, base class, attribute, package,
generated facade, builder, library generic, extension method or marker API.
Explicit `System.Nullable<T>`, generic methods, generic source types, open
parameters, constraints, variance and arbitrary constructed types reject.
Only the exact admitted `T?` spelling, independently resolved as the pinned
nullable symbol, introduces a compiler-owned option specialization. A name
such as `Option`, `Money`, `Entries` or `Events` has no classification authority.

No caller chooses a template, version, operation allowlist, expansion equation,
bound or content hash. A binding requests one of the closed semantic roles;
all other data is independently derived or verified. Source helper calls stay
source helper calls. A matching binding is not permission to replace a helper
body by a trusted intrinsic. Unknown keys/tags, repeated JSON keys, duplicate
entries and later schema versions reject rather than being ignored.

## 2. Identities, bytes and descriptor schema

Canonical JSON here is UTF-8, no BOM, sorted object keys, no insignificant
whitespace, no ASCII escaping of Unicode, shortest decimal integers, lowercase
`true`/`false`/`null`, no NaN/infinity/floating JSON numbers and exactly one final
LF for files. Object keys and strings contain Unicode scalar values; unpaired
surrogates in *program strings* use W07's UTF-16 encoding, not raw JSON text.
Hashes of canonical **values** exclude the final LF. Arrays retain order.
JSON integer tokens are restricted to the shared safe range
-(2^53-1)..(2^53-1). Wider semantic integer payloads are canonical decimal
strings. Enum carriers and tag-arm carriers always use canonical decimal
strings, even for small values; their exact underlying width/signedness is
retained separately. Structural counts and versions are unsigned integers,
never Booleans. Every
hash is 64 lowercase hexadecimal digits.

`H(domain,x) = SHA256(ASCII(domain) || 00 || canonical(x))`. Domains are:

| Subject | Exact domain |
| --- | --- |
| descriptor | `MPK-CSHARP-PRACTICAL-FOUNDATION-1.0` |
| closed instance | `MPK-CSHARP-SEMANTIC-INSTANCE-1.0` |
| closed instance set | `MPK-CSHARP-CLOSED-INSTANCES-1.0` |
| binding | `MPK-CSHARP-SEMANTIC-BINDING-1.0` |
| declaration | `MPK-CSHARP-DECLARATION-1.0` |
| stored member | `MPK-CSHARP-FOUNDATION-MEMBER-1.0` |
| source provenance | `MPK-CSHARP-DECLARATION-PROVENANCE-1.0` |

The descriptor has exactly `schema`, `id`, `version`, `semantic_profile`,
`members`, `template_ids`, `non_template_ids`, `hash_domains`,
`structural_limits`, `value_bounds`, `source_callable_members`,
`caller_extension_points`, `activation`, `content_sha256`.
`schema=mpk.csharp.foundation_descriptor.v1`, `activation=candidate_only`, and
both source/extension arrays are empty. `members` is path-sorted and contains
exactly the two files above, each as `{path,schema,sha256,size_bytes}`.
Its own transport is not a member. The descriptor hash is `H(descriptor,object
without content_sha256)`; member digests hash **raw file bytes**. Recompute the
complete fixed object as well as its hash: recomputing a hash cannot authorize
a changed template or an extra member. Missing, extra, reordered, stale or
symlinked members reject. Paths are repository-relative, portable, non-escaping.

Declaration identity preimage fields are exactly `kind`, `namespace`, `owner`,
`name`, `parameter_type_ids`, `result_type_id`. Types use kind `type`, empty
owner for top-level, empty parameter list and empty result; methods/constructors
use the exact declaring type's canonical ID as owner, ordered parameter IDs, and exact
result (constructed owner for a constructor). Prefix the declaration-domain
digest with `mpk.csharp.source.`. Overload identity includes types, not parameter
names. Generic arity must be zero before constructing an ID. Namespace/name
spelling is exact resolved identifier text, not display strings or aliases.

A stored member ID is `mpk.csharp.member.` plus the member-domain hash of
`{owner,name,type,storage}`, where `type` is the closed type expression below
and storage is `readonly_field`, `get_auto`, or `init_auto`. Declaration order
is recorded separately as a contiguous zero-based ordinal. Explicit computed
getters are methods, not duplicated stored state. File hash, source span,
syntax kind and source-map identity are provenance, not declaration identity;
the source manifest binds all of them. Provenance preimages contain exact
declaration ID, captured path, source SHA-256, byte start and byte length;
relocating a file preserves declaration IDs but changes provenance. Changes to
a source body invalidate bindings and proof obligations even when IDs survive.

The model's captured source view has exactly `id`, `identity`, `kind`,
`members`, `enum_values`, `enum_underlying`, `actual_default`, `public_default`,
`identity_sensitive`, `source_sha256`. Each member is exactly
`{id,name,type,storage,ordinal,required}`. This is a test view of independently
reconstructed source facts, not a caller-attested type-invariant certificate.
`public_default` and `identity_sensitive` must be established by the later
frontend/VC pipeline, never trusted from a sidecar. Enum aliases retain their
carrier values. `enum_underlying` is exactly i8/u8/i16/u16/i32/u32/i64/u64 for
an enum (null for a non-enum), and every declared carrier fits that range.
The member graph is acyclic, including through arrays/options.

## 3. Exact template and specialization registry

The twelve template IDs are `mpk.csharp.semantic.<name>.v1`. Names, arities,
ordered dependency recipes and operations are the exact arrays in the
descriptor's definition member (`mpk.csharp.foundation_definitions.v1`):

| Name | Arity | Direct dependencies, in recipe order | Root derivation |
| --- | --- | --- | --- |
| bounded_sequence | 1 | none | array, string, binding, contract, boundary, transition; dependency |
| sequence_construction | 1 | bounded_sequence(T) | source construction only |
| ordered_entry | 2 | none | exact entry binding; map dependency |
| ordered_map | 2 | ordered_entry(K,V), bounded_sequence(ordered_entry(K,V)), lookup(V) | binding |
| ordered_set | 1 | bounded_sequence(T) | binding |
| option | 1 | none | nullable, binding, contract, boundary; dependency |
| lookup | 1 | none | binding; map dependency |
| result | 2 | none | binding, codec-result root |
| validation | 2 | bounded_sequence(E) | binding |
| boundary_field | 1 | none | binding, boundary |
| transition | 3 | bounded_sequence(E) | binding, transition |
| money | 1 | none | binding |

`ordered_entry` is an internal product template, not permission to name a
generic key/value type. A standalone root requires an actual entry binding.
Primitive strings derive a char sequence; their construction/publication role
retains the string bound, not the smaller ordinary sequence bound. A raw
instant carrier derives the non-template instant classification, not a hidden
source method. Money/instant fallible operations have normal/error relations;
the application's concrete result and error enum are separately derived roots.
Do not add an undeclared `E` to `money<C>` or silently instantiate a result with
an invented source error type.

### 3.1 Four non-template definitions

The exact IDs are `mpk.csharp.value.unit.v1`, `.parse_error.v1`, `.instant.v1`
and `.exception.v1` (the latter three replace `unit` in the full prefix).

`unit` has one value, equality always true and comparison zero. `parse_error`
has tags 0–4 in this order: `input_bound`, `syntax`, `noncanonical`,
`scale_precision`, `range`; it is not default-eligible. Its equality/order are
tag equality/order. These are W07 parser outcomes, not CLR exceptions.
`instant` is the signed 64-bit millisecond value specified in section 9.

The exception definition's first nine tags are, in order:
`System.DivideByZeroException`, `System.OverflowException`,
`System.IndexOutOfRangeException`, `System.ArgumentException`,
`System.ArgumentOutOfRangeException`, `System.ArgumentNullException`,
`System.InvalidOperationException`, `System.NullReferenceException`,
`System.Runtime.CompilerServices.SwitchExpressionException`.
They have no observable payload. Append reachable admitted sealed direct
`System.Exception` subtypes in ascending canonical source-ID order, with their
exact immutable payload members. This is one closed per-compilation sum, not a
generic template or an extensible exception registry. Catch ancestry includes
ArgumentNull/ArgumentOutOfRange -> ArgumentException -> SystemException ->
Exception; the other builtins -> SystemException -> Exception except the switch
exception -> InvalidOperationException -> SystemException -> Exception.
Source subtypes have only Exception as their source-visible base. No allocation
identity, message, stack, inner exception or runtime resource failure is encoded.
Payload read requires the exact matching source arm. Construction, type test
and payload read use that closed tag table; no external exception type enters.
Filter/search/finally order is W05 evidence and T04/T06 implementation scope.

### 3.2 Closed types and closure algorithm

An input type expression is exactly one of:

```text
{kind:primitive,id:<fixed primitive name>}
{kind:source,id:<verified source declaration ID>}
{kind:instance,template:<one of the twelve names>,arguments:[closed types...]}
```

The primitive names are the model's exact `PRIMITIVES` set: bool, signed and
unsigned 8/16/32/64-bit integers, char, f32, f64, decimal, string, date, time,
duration, guid, day_of_week, and the four non-template names. This intermediate
notation is never source generic syntax. An array-valued source member maps to
its concrete sequence type before binding inference. A reference's nullable
carrier maps to option; its non-null contract applies independently.

Arguments must be fully closed admitted immutable values. Unknown source IDs,
cyclic source graphs, open parameters, wrong arity, construction-token payloads,
exception-as-ordinary-data payloads, and `option<option<T>>` reject.
`lookup<option<T>>` is valid. Other outcome nesting follows the depth bound.
Map/set keys must satisfy section 7's total-order predicate recursively.
Currency is exactly string or a source enum; no integer or framework enum
substitution. Primitive ID = `mpk.csharp.value.<name>.v1`; source ID is unchanged.
An instance preimage is `{template:<full template ID>,version:1,
arguments:[ordered concrete argument IDs]}`; prefix its instance-domain digest
with `mpk.csharp.instance.`. Provenance is not part of this ID, so identical
instances share definitions. Foundation content is bound by the containing
closed-set/context, not by adding a circular field to each instance ID.

An exact root is `{origin,provenance_id,type}`. Origin belongs to the closed
`ROOT_ORIGINS` set in the model and must be allowed by the table. A provenance
reference resolves to one captured source/binding/contract root; merely
inventing a string that matches the model's ID syntax is insufficient in
production. The model accepts IDs matching `[a-z][a-z0-9_.:-]{0,255}`.

1. Reconstruct *all* reachable roots from selected signatures/bodies, stored
   member types, bindings, contracts, codecs, boundary and transition overlays.
   Verify source closure and origin before specializing. Reject duplicate exact
   root records; separate provenance roots for one type are allowed.
2. Recursively collect nested instance arguments. Substitute positional
   parameters in the registry's dependency recipes, recursively collect their
   arguments/dependencies and repeat to a fixed point. No source generic body
   is instantiated. Every dependency inherits the originating root provenance.
3. On first insertion of each ID, compare the count to its cap. Identical ID
   with unequal type arguments rejects as a collision. Deduplicate identical
   instances; union and sort their provenance IDs, including provenance that
   arrives through a second path after initial discovery.
4. Replace every parameter, self/argument/dependency type reference with an
   exact concrete type ID in the type and operation definitions. References to
   `dependencyN` use the registry's recipe order, **not** sorted ID order.
   The serialized dependency ID set is separately sorted and deduplicated.
5. Remove `compare` from a concrete operation set precisely when that value
   lacks a total order. Other operation names/order remain registry order.
   Serialize entries by ascending instance ID and provenance IDs lexically.
   Topologically schedule definition dependencies for code emission; do not
   mistake lexical transport order for declaration dependency order.
6. Recompute each entry and all counts. The importer must compare the entire
   recomputed object, including operation bodies, dependencies, order,
   provenance and counters, not just the submitted set hash. Reject extra or
   unreachable instances, missing dependencies, stale bindings and any residual
   parameter/template application in VIR. The emitted VIR contains only named
   concrete types/operations and their closed context linkage.

The closed-set schema is `mpk.csharp.closed_instances.v1`, with exactly
`schema,semantic_profile,foundation_id,foundation_sha256,entries,counters,
closed_set_sha256`. Its last field hashes the object with that field omitted.
An entry has exactly `instance_id,template_id,version,semantic_profile,arity,
argument_ids,dependency_ids,provenance_ids,type_definition,
operation_definitions,counters`. A concrete operation has `id`,
`argument_type_ids`, `normal_result_type_id`, `equation`, `error_precedence`.
These signatures describe normal results; exceptional/error/VC branches retain
their distinct kind as specified below, not a phantom source return type.

### 3.3 Foundation-local counters

All structural comparisons are `count <= cap`; increment in mathematical
nonnegative integers and reject before an overflowing host allocation. Nesting
counts instance-argument edges from root depth zero; a primitive leaf at depth
16 is allowed, at 17 rejects. Source graph cycles reject independently.

| Counter | Cap | Single counting rule |
| --- | ---: | --- |
| binding_count | 128 | number of distinct reachable binding entries after duplicate rejection |
| closed_instance_count | 256 | first insertion of a distinct derived instance ID |
| closed_instance_depth | 16 | maximum argument-edge depth, including derived dependencies |
| expanded_declarations | 1,024 | one type definition per concrete instance |
| expanded_operations | 4,096 | one retained operation definition per instance |
| expanded_recipe_nodes | 262,144 | recursively 1 per JSON object/array/scalar value in type definition and operation array; keys do not add nodes |
| projection_obligations_per_binding | 64 | one emitted obligation record, including each member and mapped operation |

These are measured specification-expansion budgets, not estimates of kernel
term cost. Counters overlap intentionally: e.g. the instance cap can make a
larger declaration budget unreachable. Tests exercise actual closure at
255/256/257 instances and depth 15/16/17, plus each comparison at cap-1/cap/cap+1.
T01-W09 owns complete pre-invocation and checker-capacity counters and their
actual emitted-term probes; it must not interpret recipe nodes as proof nodes.
Value predicates are separate: array/sequence/map/set/event length <= 4,096,
string/construction length <= 16,384, validation errors <= 256, and recursively
counted live value cells <= 65,536. Zero-length nested containers count their
enclosing cell but no nonexistent elements. Symbols at run time generate VCs,
not structural rejection or invented capacity exceptions. Static artifacts
that exceed an encoding/transport counter reject before invocation.

## 4. Ordinary-core expansion calculus

All recipes lower to ordinary Certificate v0 `Sort`, `Var`, `Const`, `App`,
`Lam`, `Pi`, and `Let` terms plus the already checked Bool/Eq and required logic
definitions. There is no new kernel type former, generic checker, product/sum
inductive shape, axiom, extensionality rule, theory primitive, proof-node entry,
or theory certificate. The existing natural-number type in namespace `Std` may
remain in a containing assembly when an unchanged predecessor needs it, but no
practical-foundation recipe applies that namespace's `Nat.rec`. Existing
theory-oriented bit-vector or array axioms are **not**
usable merely because they have similar names.

The concrete value carrier is a finite-depth, binary-addressed Boolean cube:

```text
C(0)   = Bool
C(d+1) = Bool -> C(d)
Z(0)   = false
Z(d+1) = lambda b. Z(d)
addr(b0,...,bq-1) = sum(bit(bj) * 2^j)
mux(0,c,t,e)      = Bool.rec(e,t,c)
mux(d+1,c,t,e)    = lambda b. mux(d,c,t(b),e(b))
```

These are generation equations, not polymorphic or depth-indexed declarations
inside the certificate. Every generated `Bool.rec` has Bool false/true cases,
Bool major and Bool result. Selector arguments are ordered least-significant
first (`b0` first); false is address bit 0 and true is address bit 1. A fixed
block of `w` bits uses `C(ceil(log2(max(1,w))))`; every address `>= w` is false.
Bool itself uses `C(0)`. Signed values use fixed-width two's complement; char
uses 16 bits, GUID 128 N-order bits, and decimal sign/scale/coefficient uses the
W07 exact representation. No 64/96/128-bit scalar is converted to unary Nat.

For `a <= d`, `pad(a,d,x)` adds `d-a` leading selector binders and returns
`x` only when every added selector is false; all other padding addresses return
`Z(a)`. `unpad` applies false for those exact binders. Both operations expand
pointwise through `mux`; neither eliminates Bool into a function result.

For a product with `n` fields, let `q=ceil(log2(max(1,n)))` and let `d` be the
maximum padded child depth. Its carrier is `C(q+d)`: the first `q` selectors
encode the field ordinal and the remaining selectors address the padded child.
Unused field ordinals are zero. Projection supplies the fixed little-endian
field-selector bits and then `unpad`s to the declared child depth. A sum is the
two-field product of a fixed-width tag and the maximum-depth payload. Inactive
or no-payload branches contain zero, and validity checks only the **active** arm
payload. Thus `none` needs no default inhabitant of T and equal payload bytes do
not collapse distinct arms.

A sequence whose role admits `0 <= length <= N` is the product of its u32
length and an element cube with `ceil(log2(max(1,N)))` leading little-endian index
selectors. Selectors at addresses `>= N` and every address `>= length` return
the zero element. A checked read first proves the signed i32 index is in
`0..length`, then applies its low index bits; it never treats truncated high bits
as an address. Maps and sets reuse that sequence layout and add their sortedness
invariant. Transition fields are state/events/response; money storage fields are
amount/currency even though order compares currency first. `unit` is `Z(0)`;
parse-error and exception tags follow section 3.1. The per-compilation exception
payload union is concrete before VIR.

Every value conditional expands as `mux(depth,c,t,e)`. Fixed-width arithmetic,
range checks, index comparison, scalar comparison, and codec character logic
are finite Boolean circuits in the W07 bit order. Carry/borrow chains run from
least- to most-significant bit; equality is the balanced conjunction of per-bit
equivalence; order uses the frozen signed/unsigned rule; shifts, multiply,
division/remainder, decimal normalization, and codec rules use their explicitly
bounded width/character networks. A generator may share repeated subterms with
`Let`, but may not substitute host evaluation, a BCL result, or a theory node.

A bounded scan over concrete state carrier `S = C(d)` does not use an inductive
recursor. Write `mux_S(c,t,e) = mux(d,c,t,e)`, and generate one closed
transformer per possible source index:

```text
step_i : S -> S
step_i = lambda s. mux_S(unsigned(i) < length, body(i,s), s)
id_S   = lambda s. s
compose_S(f,g) = lambda s. g(f(s))
scan_S = balanced_ordered_compose(step_0,...,step_(N-1))(initial)
```

`mux_S`, `id_S`, and `compose_S` are generated at the one concrete `S`;
they are not user-visible or foundation generics. The balanced composition tree
is split into contiguous halves, evaluates the lower-index half before the
higher-index half, and uses `id_S` for an empty interval. Search/early-exit state
contains an explicit found/stopped bit, so every later transformer becomes an
identity after the first terminal step. This form permits one state coordinate
to depend on any prior coordinate while every actual `Bool.rec` still returns
only Bool. The term table and `Let` bind the shared concrete transformer bodies;
the containing W09 counters, not this semantic value bound, decide whether the
expanded network is small enough for both checkers.

Generated helpers take no source type parameter: element layout, operation IDs,
width, field count, state carrier, and scan bound are fixed by the concrete
closed operation. Numeric/codec primitive behavior remains W07's exact operation
table, domains, bits and error order; no BCL Parse/Format/operation call becomes
a proof oracle. The production owners must construct and check these bodies
before a candidate can be registered.

The equation vocabulary in the definition member is closed:

- `product`, `field`, `sum`, `tag`, `active_payload` mean the above constructors,
  projections and tag-guarded payload selection. Wrong-arm payload reads have
  an InvalidOperation edge only when source declares it; otherwise that source
  operation rejects. They do not return zero inactive bytes as a T value.
- Sequence `length/read`, initialized-cell read/write/freeze and indexed
  functional update are sections 6 and 7. No unchecked read is totalized into
  an arbitrary value on an error path.
- `*_equal` recurse through declared fields, active arms and elements using
  the *semantic* scalar equality. `*_compare` use section 7's exact order and
  return -1/0/1. Never equate functions in core, assume extensionality, or replace
  float equality with bit equality. Boundary equality and structural equality
  can have different purposes; a binding proves its own relation.
- Ordered search scans from index zero until key >= requested key, records the
  first equality, and inserts before the first greater key. Add/replace rules
  below fix every boundary/error branch.
- The other equation names are direct compositions specified in sections 8–9:
  validation ordered append, option fallback, transition product, checked
  decimal money operations and explicit rounding. There is no extensible
  evaluator dispatch keyed only by an arbitrary equation string.

Ownership is analysis state, not a heap axiom. Before VIR emission each
construction access has a concrete element operation and each publication has
the required initialization/ownership/bound obligations. The linear token
itself is not a storable application value. T02/T06 must preserve obligations
while erasing analysis tokens, never erase the checks because no token remains.
Normative recipes are a specification of what to implement; this freeze does
not claim that an unimplemented recipe already has a checked certificate.

## 5. Source declarations, construction and pure calls

Accepted source data forms are top-level, non-generic source enums, readonly
structs and sealed ordinary classes. No nested/partial data declarations,
source base list (except the separate sealed exception rule), interface,
record/record struct, layout attribute, mutable field, static storage, const,
event, indexer, ref field, operator/conversion, finalizer or initializer is
admitted. Enum members alone have their closed compile-time carrier rule.
Private/internal/public readonly fields and exact getter/init auto-properties
are stored state. Auto-property backing-field provenance is cross-checked with
W04's pinned shape; do not double-count synthesized fields. Computed getters
are total pure code; expression-bodied methods/getters normalize to the same
return graph as their block form. Custom init bodies and ordinary setters reject.

Constructor parameters are exact admitted by-value types; no optional, params,
ref/out/in, dynamic dispatch or implicit user conversion. One same-type
constructor delegation is allowed when the complete delegation graph is
acyclic. Initializer/delegation arguments and all ordinary expressions evaluate
once in source order. Only the inert object base call is allowed for data
classes. The exact compiler implicit parameterless constructor is admitted
only with W04's inert shape; other synthesized behavior does not gain admission.

A construction transaction has a fresh receiver identity, per-member assigned
facts and no observable completed object. Direct member reads require an
assignment on every incoming path. `this` is usable only for those direct
reads/writes and admitted delegation. Getter/method calls on `this`, whole-this
assignment, capture, argument passing, comparison, storing or returning it
before finalization reject. Static pure helpers can receive already computed
values, not the unfinished receiver. Exceptions discard the transaction.

With no init members, every normal constructor exit proves the public invariant.
With init members, every constructor exit proves the closed construction
invariant and **every** complete `new` expression, even without an initializer,
proves the public invariant. Object initializers then evaluate RHSs once in
source order, assign exact directly declared init properties and finalize.
Each member is explicitly assigned at most once over the delegation/constructor/
initializer chain. Zero state before assignment is not a duplicate write.
Required members are init-auto properties only, cannot be assigned by a
constructor in this profile and must each occur once in the initializer.
Normal exits establish all public member invariants, including eligible
recursive defaults for intentionally unassigned members. Roslyn's definite
assignment/required diagnostics are necessary but not MPK proof evidence.

An instance call evaluates receiver first, then arguments left to right, and
calls one exact statically resolved source function with receiver as parameter
zero. Receiver null failure occurs at the language call point, after argument
evaluation. Direct dereference and conditional access follow their own exact
evaluation rules. Non-null annotations require contracts, not trust in nullable
warnings. Methods never mutate their receiver or reachable values; class
reference equality/identity, hashing and runtime type inspection reject.

### 5.1 Recursive default

Default eligibility is recomputed over the acyclic type graph. Scalar zero,
false, empty GUID, day/time zero and zero duration/instant are eligible. An enum
needs a declared zero carrier; aliases do not create new values. A structural
value requires every nested default eligible, no required member and proof of
its public invariant at that exact default. A non-null string/array/class
reference is not eligible; its nullable semantic form may be none. An internal
empty sequence/map/set is eligible, but a source wrapper's actual CLR default
is **not** replaced with an empty array. Check its source null/member invariant.
The same distinction applies to source classes, whose default is null.

Internal option/lookup default to none/missing_key without requiring an
inhabitant of their payload. Bound readonly structs gain default eligibility
only when actual default storage maps to that arm and satisfies the public
invariant; a declaration can instead remain default-ineligible. Result,
validation, money, boundary-field, transition, parse-error and exception
values are not implicitly default-constructed. `default(ExactType)` is the
only explicit default source form; target-typed default rejects. Temporary
constructor/array zero storage is not publication of a default-eligible value.

## 6. Arrays, construction and publication

Arrays are one-dimensional zero-based arrays with exact int length/index, no
rank covariance, cast, pointer, span, mutable view or aliased write. Negative
allocation length produces OverflowException on the pinned runtime; an invalid
index produces IndexOutOfRangeException. Allocation/resource exhaustion is not
a catchable semantic capacity error. Length/bounds at run time are predicates.

Allocation of default-eligible elements creates complete cells. Allocation of
ineligible elements creates unique uninitialized storage with an empty bitmap;
zero length is already complete. A first write requires an uninitialized cell
and marks it exactly once. A read requires initialized cell, ownership and valid
index. A rewrite requires **all** cells complete and exclusive ownership.
Branch joins intersect definitely initialized cells and must agree on owner
and lifetime state. A path-dependent alias, lost ownership or inconsistent
publication rejects. An immutable foreach borrow permits reads and forbids
write/transfer/publication until its scope ends.

Publication/return/field storage/call transfer requires all cells initialized,
public element invariants, target-role bound and aggregate live-cell bound.
Freeze permits subsequent reads but never writes. Transfer invalidates all use
through the old construction owner, including reads. Partial construction does
not escape via a call, tuple/field, option/outcome, comparison or return.
Discarding a partial buffer is allowed on an exiting path and publishes no
value. Null is an option around the immutable array, not a partially initialized
buffer. `Length`/index/member reads never smuggle ownership out of the analysis.

For count-then-allocate construction, pass one evaluates a pure predicate on an
unchanged borrowed input and proves exact count, overflow safety and target
bound. Allocate exactly that count. Pass two traverses the same input/order,
performs each selected projection once and writes the next fresh slot. Prove
`0 <= next <= count` throughout and `next=count` on publication. Predicate
stability is a proof obligation; effects or input mutation reject. Both passes
have total-termination and loop contracts. T03 supplies representations and
handoff operations, while positive loop lowering belongs to T04, proof to T06.
No generator, iterator, builder or source-visible generic collection is implied.

The definition member's construction failure labels `ownership`, `incomplete`,
`already_initialized`, `uninitialized`, `construction_bound` and
`publication_bound` are static/VC failures, **not invented CLR exceptions**.
`negative_length` and `index_range` refer to the exact exceptional edges above.

## 7. Structural equality, total order, sequences, maps and sets

Structural equality compares every stored source field in declaration order,
recursively. Computed getters are not extra storage. Ordinary structs/classes
do not inherit a runtime `==` operator: source uses an explicit pure structural
function with proved correspondence, while contract/model equality is the
field-complete relation. Primitive float equality uses IEEE equality (NaN is
non-reflexive, both zero signs equal); decimal uses numeric value equality, not
coefficient/scale identity. Enum equality compares exact same-type carriers.
Option/outcome equality compares the tag and only semantic active payloads;
source inactive fields remain subject to the binding round-trip proof.

Total-key/order types are bool (false first), exact same-type integers/char,
ordinal UTF-16 string, enum carrier, decimal value, date/day number, time/ticks,
duration/ticks, instant/milliseconds and GUID's unsigned fields in N-text order.
unit has one key; parse-error uses its declared tag order. Nullable adds none
before some, then payload order. Products compare fields lexicographically;
sequences compare first differing element then length. Outcome sums compare
tag index then active payload. Transition is state/events/response order.
Money is currency then amount; it is not an economic cross-currency comparison.
An array key is an immutable semantic sequence and must satisfy all key
element invariants and bounds. Non-reflexive f32/f64, exception objects,
construction tokens and anything recursively containing them are not total
keys. There is no fallback bit order for floats or identity order for classes.

The ordinary bounded sequence offers length, guarded index read, equality and
conditional compare. No append operation is silently available; explicit
construction/loops implement concatenation. Map representation is a bounded
sequence of bound concrete ordered-entry products. Entry fields are key/value,
not a framework KeyValuePair. Set representation is a bounded sequence of T.
Inputs prove strict ascending unique keys/elements, count and public invariants.
Lookup scans in order and returns missing_key or found(value); found(none) is
distinct from missing_key. Equal maps/sets have the same length and pairwise
equal elements; no hash table, arbitrary comparer or dictionary enumeration.

Add first validates representation, then tests duplicate, then capacity,
then inserts at lower_bound. Duplicate at a full map is duplicate_key, not
capacity. Set add similarly reports duplicate_element. Replace first validates,
then requires existing key, then changes only its value; it never inserts and
is allowed at full capacity. Map compare exists only when both key and value
are total-orderable. These are specification relations: source implements its
declared checked branch/error result, and a sidecar cannot invent a throw or
replace a different helper. A false input sortedness invariant is a VC failure,
not a new implicit application exception.

## 8. Binding, projection, nullable and closed outcomes

### 8.1 Exact binding schema

The local schema `mpk.csharp.semantic_binding.v1` has exactly:

```text
schema, source_type_id, source_content_sha256, role, member_map, tag_arms,
inferred_argument_ids, default_arm, bounds, operation_map, binding_sha256
```

`binding_sha256` hashes the object with that field omitted. The table is sorted
by source type ID, exactly one binding per reachable classified type; no
duplicate, missing, unreachable, stale, cyclic or hash-only colliding entry.
Do not require a binding for an ordinary unclassified structural value.

| Role | Exact member-map keys | Inferred arguments |
| --- | --- | --- |
| option / lookup / boundary_field | tag, value | value type |
| result | tag, value, error | value type, error type |
| validation | tag, value, errors | value type, error array element type |
| transition | state, events, response | state type, event array element type, response type |
| instant | milliseconds | none; carrier must be i64 |
| money | amount, currency | exact currency type; amount must be decimal |
| bounded_sequence | elements | array element type |
| ordered_entry | key, value | exact key type, value type |
| ordered_map | entries | bound entry-array key and value types |
| ordered_set | elements | array element type |

Member-map values are exact distinct stored-member IDs, not names. A member
must belong to that source declaration, not a sibling, inherited member or
computed getter. Mapping a computed getter instead requires a source stored
representation and separate method commutation proof; it cannot fill a stored
role. Ordered entry/map/set source types remain non-generic. Money must be a
readonly struct, not a class. The importer derives the argument IDs from actual
member types (and recursively checked entry binding) and compares the full list.

`tag_arms` keys are exactly option {none,some}, lookup {missing_key,found},
result {ok,error}, validation {valid,invalid}, boundary {missing,null,value}.
Values are canonical decimal strings for distinct carriers of one exact source enum, exhaustive over its
distinct declared values; aliases of a carrier remain the same value. Other
roles have `{}`. Semantic tag order is the listed arm order, independent of
source enum numeric order. Source tags can be nonzero/noncontiguous; default
eligibility then follows actual CLR default. `default_arm` is none for option,
missing_key for lookup, otherwise ineligible; it describes internal semantics,
not a promise that the source default is valid.

Bounds are exactly `{length:4096}` for sequence/map/set, `{errors:256}` for
validation, `{events:4096}` for transition and `{}` otherwise. An application
may prove a stricter invariant; it cannot change the foundation bound in the
binding. `operation_map` maps zero or more requested semantic operation names
to exact captured source member/method IDs. Its names belong to the role's
fixed operation inventory. Unknown/deleted/wrong-signature targets reject;
one mapping adds a normal **and** exceptional/error commutation obligation.
It is not an intrinsic allowlist. Roles do not demand that applications expose
every possible convenience operation. Unmapped operations cannot be claimed
equivalent to a source helper merely because its name resembles an operation.
For executable-spec tests the captured method view records projected
`argument_type_ids` (receiver first) and `normal_result_type_id`; both must equal
the concrete operation signature. Source outcome projection supplies the
normal type only after validating its separate result binding. This normalized
view is recomputed from source and bindings, never accepted from the caller as
an assertion that an untyped or differently typed method commutes.

### 8.2 Proof obligations, not attestation

For every binding emit these seven obligations: source invariant implies
projection defined; semantic invariant implies reconstruction defined; source
round trip over all observable members; semantic round trip over all arms;
distinct-arm disjointness; preservation of public invariants; and identity
unobservability. Add actual-default/public-invariant obligation for option and
lookup, one field-complete reconstruction obligation for **every** source stored
member (including unmapped extras), and normal/error/exception commutation for
each mapped operation. Count them as section 3.3 specifies.

Let P be projection and R reconstruction, with source invariant Is and semantic
invariant Im. Required propositions are `Is(x) -> Im(P(x))`,
`Im(y) -> Is(R(y))`, `Is(x) -> R(P(x)) =source x`, and
`Im(y) -> P(R(y)) =semantic y`, plus each operation's commuting diagram over
its declared precondition and every completion branch. Equality is exact
field-complete representation equivalence, not class/reference identity or
source operator equality: f32/f64 round trips preserve every bit (including NaN
payload and signed zero), while decimal preserves numeric value because its
representation-observing APIs are excluded. Projection
expressions and proofs are generated from captured declarations and the
complete T01-W09 sidecar expression union; no arbitrary code string or a
finite list of observed values can assert these propositions.

An unmapped cache/metadata field, an independently variable inactive payload,
a stale tag map or class identity observation breaks losslessness. The binding
does not make them unobservable. A field determined by the invariant may be
reconstructed only with proof. Guarded source getters for inactive payloads
must have exactly the declared InvalidOperationException behavior or reject;
raw source field reads do not acquire a synthetic throw from the role.
The finite model deliberately rejects an extra none-arm value with payload 17
that a payload-dropping projection cannot recover, and rejects operation/arm
collapse. Passing these finite tests does not discharge the universal VCs.

### 8.3 Nullable matrix and outcome operations

`T?` construction is null/default(T?) -> none or the built-in exact T-to-T?
conversion -> some. No `new T?`, target-typed default, explicit nullable
construction cast or nullable-byref storage. `HasValue`, guarded `Value` and
`GetValueOrDefault(T fallback)` are admitted. The parameterless overload needs
default-eligible T. A fallback argument is evaluated even on a present value;
`??` differs by evaluating its exact-T RHS only when absent. Conditional access
is one nullable application-class receiver and one directly resolved field or
total pure parameterless getter returning a non-null admitted **reference** U;
result is U?, receiver evaluates once. Chained/index/conditional invocation,
value-type/nullable result and `??=` reject. `!` requires independent proof of
presence and has no semantic effect; Roslyn suppression alone is insufficient.

The initial lifted arithmetic matrix includes exact int and long `+ - * / %`,
unary `+ -`, and their same-type `== != < <= > >=`; decimal/f32/f64 lift their
same-type admitted W07 arithmetic/unary/equality/comparison operations.
There is no mixed-width/promotion lift, char/small-integer lift, bitwise numeric
lift, lifted cast or user operator in this profile. Those require explicit
present branches and ordinary admitted conversions. `bool?` has exactly `!`,
`&`, `|`, `==`, `!=`, not `&&`, `||`, `^` or arithmetic. This is intentionally
a closed subset of C# nullable operators, not all language-defined lifting.

Both operands evaluate left to right before a binary lift. Arithmetic is none
if either operand is absent (no underlying division/overflow in that branch),
otherwise execute the exact underlying operation, including errors. Equality
is true for both absent, false for one absent, else underlying equality;
inequality is its negation. Ordered comparisons with an absent operand are
false. Bool conjunction: false if either false, true if both true, else none;
disjunction: true if either true, false if both false, else none. Negation
preserves absence. f32/f64 present NaN remains non-reflexive; no special nullable
equality shortcut may make it reflexive.

Option is none/some(T), lookup missing_key/found(T), result ok(T)/error(E),
validation valid(T)/invalid(sequence(E)), boundary-field missing/null/value(T).
Constructors select the exact arm, tests inspect the tag, and guarded reads
return the active payload. Fallback selects some payload or the already
evaluated argument. Validation invalid requires 1..256 errors; a source
constructor proves this precondition or explicitly enforces its declared
ArgumentException edge. Append preserves left-before-right order and all
duplicates and proves combined <=256. Transition is a state, 0..4096 ordered
events and response product; creation proves each invariant. No implicit
short-circuit, Map/Bind, lambda combinator, sorting or error coercion exists.
Transport three-state JSON and transition/idempotency protocols are W09 scope.

## 9. Business primitives and exact operations

Only the following framework symbols become operation candidates. Resolve exact
metadata owner, static/instance kind, ordered parameter types and return type
against W03's pinned reference closure, never by display name. All unlisted
overloads reject, including general Parse/Format/ToString and boxing/object
CompareTo. Private probe setup may use such APIs to manufacture/observe values;
this does not admit them in selected source.

| Type | Exact initial source surface | Representation/rule |
| --- | --- | --- |
| DateOnly | ctor(int year,int month,int day); Year/Month/Day/DayNumber/DayOfWeek; CompareTo(DateOnly); same-type six comparison operators; AddDays(int), AddMonths(int), AddYears(int) | day number 0..3652058, Gregorian 0001-01-01..9999-12-31 |
| TimeOnly | ctor(long ticks); Ticks/Hour/Minute/Second/Millisecond; CompareTo(TimeOnly); six comparisons; operator -(TimeOnly,TimeOnly); Add(TimeSpan) returning TimeOnly | ticks 0..863999999999 |
| TimeSpan | ctor(long ticks); Ticks/Days/Hours/Minutes/Seconds/Milliseconds; CompareTo(TimeSpan); six comparisons; binary +/-, unary - | signed i64 ticks, 100 ns/tick |
| Guid | Empty; CompareTo(Guid); == and != | 128 bits, unsigned first u32, next u16, next u16, then eight bytes in order |
| DayOfWeek | exactly Sunday=0 through Saturday=6 | named values only, same enum equality/order; no numeric casts/arithmetic |

Date construction validates Gregorian components (including year/month range
before indexing month lengths) and raises ArgumentOutOfRangeException on any
invalid date. Leap years divide by 4 except centuries not divisible by 400.
DayOfWeek=(DayNumber+1) mod 7. AddDays adds the mathematical offset and checks
date range. AddMonths accepts offset -120000..120000, computes absolute month,
checks year range and clamps day to destination month's last day. AddYears
accepts -10000..10000 and uses the analogous year/clamp rule. Out-of-offset or
result range raises ArgumentOutOfRangeException; no intermediate i32 overflow
changes the exception. A valid date equality/order is its day-number relation.

Time construction checks the tick range and raises ArgumentOutOfRangeException.
Hour=ticks/36000000000; minute=(ticks/600000000)%60; second=(ticks/10000000)%60;
millisecond=(ticks/10000)%1000. Time subtraction returns the **nonnegative
wrapping** duration `(left-right) mod 864000000000`, not signed difference.
Add(TimeSpan) returns `(ticks+duration) mod day` even at i64 duration endpoints;
calculate without overflowing an i64 intermediate. No day-carry/out overload,
floating AddHours/AddMinutes or extra constructor is admitted.

Duration add/subtract/negate use a mathematical signed intermediate and check
i64 range, raising OverflowException (including negation of minimum). Component
division truncates toward zero; remainder has the dividend's sign. Days is
ticks/day; hours/minutes/seconds/milliseconds are truncated quotient remainder
24/60/60/1000. No floating factories, scaling, Total* floating properties or
additional constructor overloads are admitted.

GUID Empty is all zero. CompareTo uses the unsigned field order in the table;
normalize its result to -1/0/1 for semantic comparison. N/D codecs output lower-
case hex, with D hyphens at 8/13/18/23. No random creation, byte reinterpretation
or runtime hash. Decoding a boundary value uses the exact W07 grammar, not an
admitted source Guid constructor/Parse overload.

### 9.1 Instant and money outcomes

Instant's source bound carrier is exactly i64 milliseconds. Carrier projection
and comparison are total on this range. Add/subtract duration first checks
`ticks mod 10000 = 0`; failure is `precision`, **before** result range. Compute
`ms +/- ticks/10000` in mathematical integers and return range on i64 overflow.
Do not negate minimum i64 ticks as an intermediate. Difference computes
`(left_ms-right_ms)*10000` then checks i64 duration range; its only error is
range. Both input instants are millisecond aligned so difference has no
precision error. Success/error branches must map to the application's separate
bound result types and exhaustive error enums; clock, timezone and DateTime
range assumptions are absent.

Money currency is the exact source enum or ordinal string with an explicitly
proved application currency predicate. No external ISO/currency metadata is
consulted. Amount is a valid .NET decimal, semantically its exact rational
value. Source uses a readonly struct; no default money is publishable.
Create(amount,currency,scale) checks currency, then integer scale 0..28, then
exact representability at that scale (`amount*10^scale` integral), without
implicit rounding. Errors in order: invalid_currency, invalid_scale,
invalid_precision. The input is already a valid decimal, not an arbitrary
unbounded rational accepted as decimal by the sidecar.

Add/subtract first require same currency, then the exact W07 checked decimal
operation; errors currency_mismatch then decimal_overflow. Multiply/divide
receive explicit decimal quantity, target scale and source closed rounding
mode. Validate scale, then mode; divide then checks zero, then performs the
W07 decimal operation and explicit Round. This intentionally rounds the
**representable decimal result**, not an infinite-precision quotient directly
to the target scale. Multiplication/division overflow cannot be masked by a
later rounding step. Modes map exhaustively to ToEven, AwayFromZero, ToZero,
ToNegativeInfinity, ToPositiveInfinity; unlisted values yield invalid_rounding.
The source implementation translates expected overflow/divide rejection to its
declared closed error outcome; binding never silently swallows a source throw.
AmountCompare requires same currency before decimal comparison. Equality is
currency equality and decimal value equality; storage order is currency then
amount. Negative amounts are allowed unless the application's invariant forbids
them. Any stricter invariant is re-established on success, not assumed.

### 9.2 Codecs and runtime evidence

W07's sealed runtime record at
`develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json`
is the finite evidence for exact date `yyyy-MM-dd`, time fixed seven fractional
digits, signed duration ticks/instant milliseconds, GUID N/D, decimal and
integer codec grammars and round trips. Its raw digest is
`0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769`.
Parser precedence is input bound, syntax, canonicality, scale/precision, range;
codec-configuration validation precedes parsing as specified there. Leading
plus/zeros, uppercase GUID, whitespace, localized digits and noncanonical
negative zero are not accepted through general BCL parsing. Money has no new
magic scalar codec: boundary composition encodes its explicit currency and
decimal fields. Instant has no timezone codec.

The additional record is
`develop/migrations/csharp-03/probes/runtime-foundation-data.json`, schema
`mpk.csharp_practical.t01_w08.runtime_foundation.v0`. Inputs and expected values
come from `foundation_runtime_model.py` using independent Gregorian/integer/
ordinal algorithms, before executing `FoundationDataProbe.cs` on fixed .NET
10.0.11/C# 14/W03 references. Two clean deterministic builds each run under
two constructed hostile cultures and two unlisted environment values. Exact
exceptions, raw values, comparisons, all input rows and source/oracle/runner
hashes are retained. Probe setup APIs and instrumented trace helpers are
**observation-only**, not source-admission fixtures. W04/W06 supply the pinned
compiler-shape and rejection evidence; this record adds runtime semantics.

Every new runtime operation family has an independent model row. Nullable
absence/presence truth tables cover every admitted lifted operator for int,
long, decimal, float and double, including bit-encoded signed zero. Int boundary
behavior is an additional cross product; the larger primitive edge domains
remain W07's exact operation evidence. Instant and
money error/projection relations are MPK mathematical specification cases, not
claims of a framework Money or Instant API. Their scalar components have W07
and additional duration/runtime evidence. Finite agreement cannot replace
universal source, arithmetic, invariant or projection proofs.

## 10. Verification and downstream ownership

Run `python3 develop/probes/csharp-03/foundation_package.py --check` and
`python3 develop/probes/csharp-03/run-foundation-data-probe.py --check-record`,
then the primary Rust owner tests, vector manifest check and repository fast
gate. The runtime README records the fixed offline Linux `--check` command.
Only explicit `--update` regenerates runtime observations or generated package
artifacts; both builders have no implicit update-on-check behavior. The package
builder can also emit an apply_patch patch for inspection.

Every vector has one `implementation_owner` and exact `production_test_owner`,
in addition to this freeze's primary test. Declaration/binding/specialization
transport belongs to T02-W02/W04/W05 and VC transport to T02-W06; types/defaults
T03-W03, constructors/init T03-W04/W05, structural order T03-W06, arrays/sequence/
maps T03-W07/W08/W09, codecs/numbers T03-W10/W11, nullable/outcomes T03-W12 and
business values T03-W13. Cumulative roots/bindings belong to T03-W14. Loop
execution remains T04; projection discharge T06-W06, data/arithmetic VCs
T06-W02/W03, collection-loop proofs T06-W04, ordinary checked assembly T06-W09.
The per-row primary owner is not a substitute for these separately named
verification owners. T07/T08 alone register and atomically activate a fully
implemented, locally checked candidate.

Review must include closure and hash mutation, residual generic rejection,
default-vs-source-null distinction, unused/inactive fields, source-operation
commutation, duplicate-before-capacity precedence, ownership after publication,
non-reflexive keys, exact time wrapping and decimal rounding. Passing a
specification model cannot mark any production work item complete.
