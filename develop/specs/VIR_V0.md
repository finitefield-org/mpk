# Verification IR v0 Specification

Status: normative and frozen for implementation.

VIR is the untrusted, language-neutral program input to verification-condition
generation. A valid VIR document is not proof evidence. Only a certificate
accepted by both configured MPK checkers is proof evidence.

The only schema specified here is `mpk.vir.v0`. Removed predecessor documents
never parse as VIR, and no component may reinterpret a legacy schema
discriminator as VIR.

## 1. Conformance language

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
no VIR, VC, certificate skeleton, or partial trusted artifact is returned.

Every object in this specification is closed: the listed fields are the exact
fields, every listed field is required unless explicitly described otherwise,
`null` is never a substitute for an absent or required value, and an unknown or
inapplicable field rejects. Every tagged union uses the exact, case-sensitive
`kind` value shown here. Duplicate JSON object names reject before typed
deserialization or canonicalization.

All input is UTF-8 JSON. Floating-point JSON numbers reject. Integral JSON
numbers are restricted to `[-9007199254740991, 9007199254740991]`; mathematical
integers outside that interval use the decimal-string encodings declared below.
Strings contain only Unicode scalar values and receive no Unicode
normalization.

Validation uses this deterministic phase order. The first failing phase owns
the diagnostic code used by the conformance vectors:

1. byte, UTF-8, duplicate-name, JSON-depth, and per-string byte limits;
2. object shape, tagged-union shape, and schema discriminator;
3. language/profile/semantic-parameter pairing;
4. shared collection-count, identifier, and aggregate-depth limits;
5. identifier grammar, uniqueness, declaration order, and reference closure;
6. type, operation, instruction, and terminator rules;
7. CFG, call-graph, loop-cutpoint, contract, and non-check canonical-order
   rules;
8. exact `safety_checks` contents, uniqueness, profile-required set, and order;
9. canonical-root JCS byte limit, then `contract_hash` and `vir_hash`
   recomputation.

An implementation may collect more than one finding inside a phase, but it
MUST NOT report a later-phase failure as the primary failure.

## 2. Module and semantic profiles

### 2.1 `VirModule`

The root object has exactly these fields:

| Field | Type | Rule |
|---|---|---|
| `schema` | string | exactly `mpk.vir.v0` |
| `source_language` | string | exactly `go` or `rust` |
| `semantic_profile` | string | one of the two profiles below |
| `semantic_parameters` | closed object | exact profile-specific shape below |
| `units` | array of `VirUnit` | canonical unit order; nonempty |
| `vir_hash` | string | 64 lowercase hexadecimal characters |

One module contains one source language and one semantic profile. All repeated
profile and semantic-parameter objects in its contracts MUST be member-for-member
equal as JSON values to the module fields. Source object-key order is immaterial;
JCS fixes the hashed order. Mixed-language units and calls to a different module
reject.

The only initial pairings are:

| `source_language` | `semantic_profile` | Semantic-parameter shape |
|---|---|---|
| `go` | `mpk.go.fixed.v0` | `GoFixed` |
| `rust` | `mpk.rust.checked.v0` | `RustChecked` |

No other pairing is inferred or accepted. Value-operation semantics never
branch on `source_language`; only `semantic_profile` selects the required
failure checks and the accepted CFG/source-operation subset.

`GoFixed` has exactly the following fields; the values shown are one accepted
registered-target instance:

```json
{"target_id":"linux/amd64","pointer_width":64}
```

`target_id` is the canonical lowercase `<goos>/<goarch>` pair selected by the
registered Go frontend and matches
`[a-z0-9_]+/[a-z0-9_]+`. `pointer_width` is the JSON integer `32` or `64` and
must match that registered target. Go v0 program types remain explicit-width;
the target is nevertheless semantic context and always participates in hashes.

`RustChecked` has exactly the following fields; the target and width shown are
one accepted registered-target instance:

```json
{"target_id":"x86_64-unknown-linux-gnu","pointer_width":64,"overflow_mode":"checked","panic_mode":"abort"}
```

`target_id` is the exact registered built-in Rust target triple, uses only
ASCII lowercase letters, digits, `_`, `.`, and `-`, starts and ends with an
ASCII lowercase letter or digit, and is 1 through 255 UTF-8 bytes.
`pointer_width` is `32` or `64` and must match the registered target.
`overflow_mode` and `panic_mode` are the literal strings shown. Custom JSON
targets and aliases reject.

Changing a target identifier or pointer width changes the semantic context and
therefore changes VIR, VC, manifest, and evidence hashes even when no program
type uses the pointer width.

### 2.2 `VirUnit`

A unit has exactly:

| Field | Type | Rule |
|---|---|---|
| `id` | string | canonical Go import path or Rust crate name |
| `name` | string | validated Go package identifier or Rust Cargo package name |
| `type_decls` | array of `StructDecl` | declaration-dependency order |
| `const_decls` | array of `ConstDecl` | increasing `id` UTF-8 byte order |
| `functions` | array of `VirFunction` | callee-first order from section 5.4 |

Units are sorted by `id` in increasing UTF-8 byte order. Duplicate unit IDs
reject; distinct Go units may have the same package `name`. Rust v0 has exactly
one unit: `id` is its library crate name and `name` is its exact Cargo package
name, which may differ; Go v0 may have more than one same-module unit, but each
call remains within `units`.

## 3. Identifiers and canonical renaming

### 3.1 Lexical classes

`AsciiIdent` matches `[A-Za-z_][A-Za-z0-9_]*`, is not `_`, and is at most 255
UTF-8 bytes. Rust crate, module, item, function, struct, constant, field, and
source binding names are `AsciiIdent`. Raw and non-ASCII Rust identifiers
reject before VIR emission. A Rust unit `name` is instead the accepted Cargo
package name and matches `[A-Za-z][A-Za-z0-9_-]*` within the global 1,024-byte
identifier limit.

A Rust public item ID is `<crate>(::<AsciiIdent>)+`, has no empty segment, and
is at most 1,024 bytes.
A function ID ends in its `name`; a type or constant ID ends in its source
declaration name.

A Go unit ID is a nonempty relative slash-separated ASCII import path. Each
segment matches `[A-Za-z0-9_][A-Za-z0-9._~-]*`; `.` and `..` segments, empty
segments, backslashes, a leading slash, a trailing slash, and URI schemes
reject. A Go public declaration ID is `<unit-id>.<AsciiIdent>` for a function,
type, or constant, or `<unit-id>.<AsciiIdent>.<AsciiIdent>` for a value-receiver
method. It is at most 1,024 bytes. This deliberately narrows post-cutover Go
public identities to deterministic ASCII; existing frozen corpus identities
all satisfy the rule.

Canonical internal IDs are exact:

- arguments: `arg0`, `arg1`, ... in source signature order;
- results: `result0`, `result1`, ... in source result order;
- user locals: `local0`, `local1`, ... in accepted source declaration order;
- compiler temporaries and value-producing instruction IDs: `t0`, `t1`, ...
  in canonical block traversal and instruction order;
- blocks: `bb0`, `bb1`, ... in the breadth-first order below.

No leading zero is allowed except in index zero itself. An ID sequence is
dense: skipping or repeating an index rejects. Public source names and spans
belong in the source map and MUST NOT be copied into these internal IDs.

Go unit `name` and struct field `name` values are `AsciiIdent`. Field names
remain validated source identifiers because field selection and declaration
order are semantic. Block parameters use `p0`, `p1`, ... densely across the
function in canonical block and parameter order. They are function-wide unique.

### 3.2 Block traversal

Traversal starts with the entry block, which is `bb0`. It is breadth-first.
A `Jump` enqueues its `label`; a `Branch` enqueues `else_label` before
`then_label`. An already discovered successor is not enqueued again. The
`blocks` array is exactly this traversal, contains every reachable block once,
and contains no unreachable block.

Instructions retain frontend-lowering order within each traversed block.
Instruction result IDs are then assigned densely across the whole function.
Renaming is identical for both profiles. Compiler-local indices, HIR/MIR/SSA
debug spellings, source spans, absolute paths, timestamps, hostnames, and
temporary paths never enter a VIR identity.

## 4. Types, declarations, bindings, and values

### 4.1 `VirType`

`VirType` is the following exact tagged union:

| `kind` | Exact fields | Meaning |
|---|---|---|
| `bool` | `kind` | Boolean |
| `bv` | `kind`, `width`, `signed` | fixed bitvector |
| `array` | `kind`, `length`, `element` | structural fixed array |
| `struct` | `kind`, `id` | reference to a nominal `StructDecl` |

For `bv`, `width` is exactly `8`, `16`, `32`, or `64`; `signed` is a JSON
boolean. Width and signedness are both part of the exact type even for
operations whose bit-level result is independent of signedness.

For `array`, `length` is a JSON integer from `0` through `256`, and `element`
is a `VirType`. Zero-length arrays are valid. Array type equality is structural.

For `struct`, `id` resolves to exactly one declaration in the same unit.
Struct type equality is nominal by declaration ID. Inline `name` or `fields`
inside a type use rejects. Recursive aggregate types and aggregate nesting
deeper than 16 array/struct edges reject.

### 4.2 `StructDecl` and `ConstDecl`

A struct declaration has exactly:

```json
{"id":"vector::Pair","name":"Pair","fields":[{"name":"left","type":{"kind":"bv","width":8,"signed":true}}]}
```

`id` and `name` obey section 3. Each `fields` element has exactly `name` and
`type`; fields may be empty, are unique by name, and remain in source declaration
order. Type declarations are ordered so every referenced struct appears before
its user. Canonical order is Kahn topological order: repeatedly emit the
smallest declaration ID by UTF-8 bytes among declarations whose dependencies
have already been emitted. A dependency cycle rejects.

A constant declaration has exactly:

```json
{"id":"vector::LIMIT","name":"LIMIT","type":{"kind":"bv","width":8,"signed":false},"value":{"int":{"value":"7","width":8,"signed":false}}}
```

Its type is `bool` or `bv`, never an aggregate. Its `value` is the matching
literal form from section 4.4. Constants are immutable and pure. Declaration
IDs are unique across types, constants, and functions in a unit.

### 4.3 Bindings

A binding has exactly `id` and `type`. Function parameter, result, and local
binding arrays use the dense IDs from section 3. A block parameter binding also
uses exactly `id` and `type`, with its function-wide dense `pN` ID.

Rust v0 functions have exactly one result. Go v0 functions have zero through
16 results. A `CallStatic` target must have exactly one result in both v0
profiles.
Parameters, results, locals, and block parameters are distinct namespaces.
Result bindings declare only the return signature; program `VirValue.var`
objects cannot reference them. A named Go result is lowered as a local plus an
explicit `Return` value.

Locals are initially undefined. Every local read must be dominated on all
incoming paths by a matching `Copy`; frontend knowledge of a source-language
zero value is not implicit VIR state. Arguments are defined on entry. Successor
arguments define non-entry target block parameters.

### 4.4 `VirValue`

`VirValue` is an exact one-field union:

| Form | Meaning |
|---|---|
| `{"var":"arg0"}` | closed reference to an argument, local, block parameter, or preceding instruction result |
| `{"const":"vector::LIMIT"}` | closed reference to a constant declaration in the function's unit |
| `{"bool":true}` | Boolean literal |
| `{"int":{"value":"-1","width":8,"signed":true}}` | typed bitvector literal |

The nested `int` object has exactly `value`, `width`, and `signed`. `width` and
`signed` obey the `bv` rules. `value` is canonical base-10: `0` or a nonzero
digit followed by digits, optionally prefixed by one `-`; leading `+`, leading
zeroes, `-0`, whitespace, and nondecimal forms reject. A signed literal lies in
`[-2^(w-1), 2^(w-1)-1]`; an unsigned literal lies in `[0, 2^w-1]`.

Literal and constant types must equal the operand's required type. A `var`
never refers forward within a block. A block may refer to arguments, constants,
locals proven initialized, its own parameters, and results of earlier
instructions; it may not directly refer to another block's parameter or
instruction result.

## 5. Functions, blocks, and graphs

### 5.1 `VirFunction`

A function has exactly:

| Field | Type |
|---|---|
| `id` | public function ID |
| `unit_id` | ID of the containing unit |
| `name` | source function `AsciiIdent` |
| `params` | ordered binding array |
| `results` | ordered binding array; profile-specific cardinality from section 4.3 |
| `locals` | ordered binding array |
| `blocks` | canonical nonempty `VirBlock` array |
| `contracts` | one `VirContract` |
| `features_used` | canonical derived string set |

`unit_id` exactly equals the containing unit's ID. Function IDs are unique
module-wide. `features_used` is present, contains no duplicates, and is sorted
by UTF-8 bytes. Its closed vocabulary and exact derivation are:

| Feature | Present exactly when |
|---|---|
| `array` | an array type, `MakeArray`, or `Index` occurs |
| `branch` | a `Branch` occurs |
| `call_static` | a `CallStatic` occurs |
| `constant_decl` | the function has a program `VirValue.const` reference |
| `conversion` | `Convert` occurs |
| `cyclic_cfg` | the reachable CFG contains a cycle |
| `mutable_local` | the function has a local or a `Copy` occurs |
| `struct` | a struct type, `MakeStruct`, or `Field` occurs |

No descriptive or source-only feature may be added to this array.

### 5.2 `VirBlock`

A block has exactly `label`, `parameters`, `instructions`, and `terminator`.
The entry block has no parameters. Other blocks may have parameters. Every
incoming edge supplies exactly one argument of the exact type for each target
parameter. A non-entry block with no incoming edge is unreachable and rejects.

The legacy `Phi` instruction is not a VIR instruction. Block parameters and
successor arguments are the sole canonical phi representation. The old
`Unsupported` instruction and `PanicUnsupported` terminator likewise reject;
rejected-source diagnostics live only in the frontend protocol. Language
profile specifications fix the exact source dependency closure represented by
type and constant declarations; VIR validation independently checks their
shape, order, uniqueness, and every use but cannot infer source reachability.
This block-parameter encoding is the single `phi/block parameters` capability
named by the consolidated design, not permission for two serialized forms.

### 5.3 CFG and loops

Rust CFGs are acyclic. Go CFGs may be acyclic or may use validated loop
cutpoints. A Go cyclic CFG is valid only when all of these hold:

- every cyclic strongly connected component has exactly one contracted header;
- the header ends in `Branch`; `then_label` enters the component and
  `else_label` leaves it;
- every edge entering the component from outside targets the header;
- every cyclic edge targeting the header is a `Jump`, and removing all such
  backedges makes the whole CFG acyclic;
- no edge leaves the component except the header's false edge;
- exactly one loop contract names that header and every loop contract names
  such a header.

Contracted cyclic components MUST be pairwise vertex-disjoint. These rules admit
disjoint natural loops and deliberately reject nested or overlapping loops,
irreducible cycles, multiple-entry loops, uncontracted cycles, ambiguous
cutpoints, and a loop contract on an acyclic block. This is the complete Go v0
loop shape. Rust requires `loops: []` and rejects the same cyclic graph before
VC generation.

### 5.4 Call graph and function order

Every `CallStatic.function` resolves within the module. Calls are direct and
the module call graph is acyclic for both profiles. Compute one module-wide
callee-first topological order; when multiple functions are ready, choose the
smallest function ID by UTF-8 byte order. Each unit's `functions` array is the
subsequence of that global order belonging to the unit, while units themselves
remain sorted by unit ID. A function that belongs to the frontend-selected
conservative source call closure remains in the module even if reachable VIR
contains no call to it; such independent functions use the same ID tie-break.

At a `CallStatic`, VC generation proves the callee's ordered `requires` in the
current path state, introduces one fresh result of the declared type, assumes
the callee's ordered `ensures` for later obligations, and depends on the
callee's checked panic-free declaration. Call preconditions and panic freedom
are not value operations or `safety_checks`.

Dynamic calls, external calls, cross-language calls, recursion, and a call to a
multi-result function reject.

## 6. Instructions

Every instruction object contains exactly the four common fields `id`, `kind`,
`type`, and `safety_checks`, plus the fields required by its variant. Fields
appear in any source JSON key order, but JCS determines hash bytes.
`safety_checks` is always present, including when empty.

Except for `Copy`, `id` defines a value of `type`. `Copy` both writes its
`target` local and defines `id` as the same post-write value. IDs are the dense
`tN` sequence. The exact instruction union is:

| `kind` | Additional required fields | Required shape |
|---|---|---|
| `Const` | `value` | `value` is a `bool` or `int` literal exactly matching `type` |
| `Copy` | `target`, `value` | `target` is a local of `type`; `value` has `type` |
| `BinOp` | `op`, `lhs`, `rhs` | operation matrix in section 8 |
| `UnaryOp` | `op`, `value` | operation matrix in section 8 |
| `Convert` | `value` | Go-only BV conversion; source type differs from or equals `type` |
| `Field` | `base`, `field` | base is nominal struct; field exists; `type` is field type |
| `Index` | `base`, `index` | base is array; index is BV; `type` is element type |
| `MakeStruct` | `fields` | exact nominal field list and values |
| `MakeArray` | `elements` | exact structural element list and values |
| `CallStatic` | `function`, `contract_hash`, `args` | exact callee signature and hash |

`MakeStruct.fields` is an array of closed `{"name":...,"value":...}` objects
in declaration order, with every field exactly once. Its `type` is a `struct`
reference. `MakeArray.elements` is in index order and its length equals the
array type length. `CallStatic.contract_hash` is 64 lowercase hexadecimal
characters and equals the resolved callee's recomputed contract hash; `args`
matches the callee parameter list exactly.

`Const`, `Copy`, `Convert`, `Field`, `MakeStruct`, `MakeArray`, and
`CallStatic` always have an empty `safety_checks` array. `BinOp`, `UnaryOp`, and
`Index` use the exact rules in sections 8 and 9. A source call's precondition
and panic-free conditions are derived VC obligations, never instruction safety
checks.

## 7. Terminators

`VirTerminator` is this exact tagged union:

| `kind` | Exact additional fields | Rule |
|---|---|---|
| `Return` | `values` | value count and types equal function results |
| `Jump` | `label`, `args` | target exists; args match its block parameters |
| `Branch` | `cond`, `then_label`, `then_args`, `else_label`, `else_args` | `cond` is bool; both targets exist; respective args match |

A `Branch`'s labels must differ; an equal-target branch must be canonicalized
to `Jump` before VIR emission. Terminators define no value and never contain
`type`, `id`, or `safety_checks`. Every block has exactly one terminator.

Source `&&` and `||` in both profiles lower to `Branch` control flow, with the
right operand evaluated only on its language-defined path. An eager Boolean
`BinOp` for source short-circuiting rejects. Runtime-safety and call obligations
in the right operand therefore inherit the left guard as a path assumption.

## 8. Total value semantics and operation matrix

### 8.1 Mathematical notation

All VIR value-producing instructions are total. Define `zero(T)` recursively:
false for bool, the all-zero bit pattern for BV, an array of `length` element
zeroes for an array, and a nominal struct whose fields are their type zeroes in
declaration order. `Const` returns its literal, `Copy` returns its input while
updating the named local, `MakeArray` and `MakeStruct` return their ordered
components, and `Field` returns the named component.

`Index` returns the selected component when `index_in_bounds` is true and
`zero(element_type)` otherwise. A signed index uses its signed mathematical
interpretation for that selection; an unsigned index uses its unsigned
interpretation. The profile-required bounds proof makes the otherwise-result
unobservable in an accepted source execution, but fixing it here keeps the
logical value operation total and solver-independent.

`CallStatic` has relational contract semantics rather than an implementation
body substitution: it returns one fresh value constrained by the resolved
callee ensures after its requires and panic-free dependencies are discharged,
as fixed in section 5.4. No other nondeterministic or uninterpreted result is a
VIR v0 value operation.

VC encoding for both profiles uses checked `Std.Program.Base.*` Bool, BV,
fixed-array, and struct foundations. The retired language-specific foundation
namespace, a solver built-in operation, or a host integer is not an alternate
post-cutover interpretation. The checked foundations implement the equations
below without new axioms.

For width `w`, let `M = 2^w`, `BV_w = {0,...,M-1}`,
`U_w(x) = x`, and:

```text
S_w(x) = x                 when x < 2^(w-1)
         x - M             otherwise
bv_w(n) = the unique r in BV_w such that r congruent to n modulo M
```

Every BV value operation below is total and returns a `BV_w` bit pattern.
Signedness controls interpretation, not the carrier. For same-width operands
`x` and `y`:

```text
bv_add(x,y) = bv_w(U_w(x) + U_w(y))
bv_sub(x,y) = bv_w(U_w(x) - U_w(y))
bv_mul(x,y) = bv_w(U_w(x) * U_w(y))
bv_neg(x)   = bv_w(-U_w(x))
bv_not(x)   = (M - 1) xor U_w(x)
bv_and/or/xor = the width-w bitwise operation
```

Unsigned division and remainder copy the fixed-size bitvector equations:

```text
bv_udiv(x,0) = M - 1
bv_udiv(x,y) = floor(U_w(x) / U_w(y))                    when y != 0
bv_urem(x,0) = x
bv_urem(x,y) = U_w(x) - U_w(y) * bv_udiv(x,y)            when y != 0
```

Let `abs_bv(x) = x` when `S_w(x) >= 0`, otherwise `bv_neg(x)`. Signed division
and remainder are the following total equations, including zero divisors and
the minimum-value case:

```text
q = bv_udiv(abs_bv(x), abs_bv(y))
bv_sdiv(x,y) = bv_neg(q)  when exactly one of S_w(x), S_w(y) is negative
               q          otherwise
r = bv_urem(abs_bv(x), abs_bv(y))
bv_srem(x,y) = bv_neg(r)  when S_w(x) is negative
               r          otherwise
```

Consequently, signed division truncates toward zero, signed remainder has the
dividend's sign, `bv_sdiv(nonnegative,0) = M-1`,
`bv_sdiv(negative,0) = 1`, `bv_srem(x,0) = x`,
`MIN / -1` has the `MIN` bit pattern, and `MIN % -1` is zero. Both profiles
still require `divisor_nonzero`; Rust additionally excludes the two signed
`MIN`/`-1` source operations through a safety predicate.

For shifts, let `k = U_v(rhs)` use the RHS's complete width `v`; it is never
truncated to the LHS width:

```text
bv_shl(x,k)  = bv_w(U_w(x) * 2^k)          when k < w; 0 otherwise
bv_lshr(x,k) = floor(U_w(x) / 2^k)         when k < w; 0 otherwise
bv_ashr(x,k) = bv_w(floor(S_w(x) / 2^k))   when k < w
                M - 1                       when k >= w and S_w(x) < 0
                0                           when k >= w and S_w(x) >= 0
```

Thus signed right shift is arithmetic and unsigned right shift is logical.
A signed RHS is interpreted by `U_v` for the total value; its signed view is
used only by `shift_count_nonnegative`.

Boolean `not` is logical negation. `eq` is Boolean equality, BV bit-pattern
equality, fixed-array component equality, or same-nominal-struct field equality
in declaration order. `not_eq` is its logical negation. Ordered comparisons are
the indicated `<`, `<=`, `>`, or `>=` over `S_w` or `U_w`.

A `Convert` from BV width `w` to width `v` first sign-extends when `v > w` and
the source type is signed, zero-extends when `v > w` and the source type is
unsigned, and otherwise keeps the low `v` bits. The target `signed` flag then
selects only the interpretation of that resulting bit pattern. Conversion is
total.

### 8.2 Instruction operation/type matrix

For all rows except shifts, BV operands have exactly the same BV type. A shift
RHS may have any accepted BV width and signedness; LHS and result have the same
BV type.

| Instruction | `op` | Operand and result types | Go | Rust |
|---|---|---|---|---|
| `UnaryOp` | `not` | bool -> bool | yes | yes |
| `UnaryOp` | `bv_neg` | BV -> same BV | signed or unsigned | signed only |
| `UnaryOp` | `bv_not` | BV -> same BV | yes | yes |
| `BinOp` | `eq`, `not_eq` | equal operands -> bool | bool, BV, array, struct | bool, BV |
| `BinOp` | `bv_add`, `bv_sub`, `bv_mul` | BV,BV -> same BV | yes | yes |
| `BinOp` | `bv_sdiv`, `bv_srem` | signed BV -> same BV | yes | yes |
| `BinOp` | `bv_udiv`, `bv_urem` | unsigned BV -> same BV | yes | yes |
| `BinOp` | `bv_and`, `bv_or`, `bv_xor` | BV,BV -> same BV | yes | yes |
| `BinOp` | `bv_shl` | BV, BV -> LHS type | yes | yes |
| `BinOp` | `bv_ashr` | signed BV LHS, BV RHS -> LHS type | yes | yes |
| `BinOp` | `bv_lshr` | unsigned BV LHS, BV RHS -> LHS type | yes | yes |
| `BinOp` | `signed_lt/le/gt/ge` | matching signed BV -> bool | yes | yes |
| `BinOp` | `unsigned_lt/le/gt/ge` | matching unsigned BV -> bool | yes | yes |
| `Convert` | no `op` field | BV -> BV | yes | no |

`and` and `or` are contract-expression operators only. Any omitted instruction
operator/type combination rejects rather than being lowered by analogy.

## 9. Safety checks and profile failure semantics

### 9.1 Exact tagged union and order

`VirSafetyCheck` is:

| `kind` | Other exact fields |
|---|---|
| `integer_no_overflow` | `operation` (`add`, `sub`, `mul`, or `neg`), `signed` (boolean) |
| `divisor_nonzero` | none |
| `signed_divrem_representable` | `operation` (`div` or `rem`) |
| `shift_count_nonnegative` | none |
| `shift_count_less_than_width` | none |
| `index_in_bounds` | none |

Checks are sorted by the table order. Within `integer_no_overflow`, operation
order is `add`, `sub`, `mul`, `neg`; within
`signed_divrem_representable`, it is `div`, `rem`. Duplicate, reordered,
missing, or extra checks reject.

Checks reference only their owning instruction operands and carry no
frontend-supplied proposition. Their exact propositions are:

```text
unsigned add: U(x)+U(y) <= 2^w-1
unsigned sub: U(x) >= U(y)
unsigned mul: U(x)*U(y) <= 2^w-1
signed add/sub/mul: the mathematical S-result is in [-2^(w-1),2^(w-1)-1]
signed neg: S(x) != -2^(w-1)
divisor_nonzero: U(rhs) != 0
signed_divrem_representable: S(lhs) != -2^(w-1) or S(rhs) != -1
shift_count_nonnegative: S(rhs) >= 0
shift_count_less_than_width: U(rhs) < lhs_width
index_in_bounds, signed index: 0 <= S(index) and S(index) < array_length
index_in_bounds, unsigned index: U(index) < array_length
```

Validation checks that this exact metadata is present; it does not require the
predicate to be statically true. VC generation emits each predicate under the
instruction's path assumptions. A false or unproved predicate prevents proof
acceptance but does not turn a well-formed VIR module into a frontend rejection,
and the instruction still has the total value from section 8.

### 9.2 Exact required sets

| Operation | `mpk.go.fixed.v0` | `mpk.rust.checked.v0` |
|---|---|---|
| signed/unsigned add | `[]` | `[integer_no_overflow(add,signed=<operand signed>)]` |
| signed/unsigned subtract | `[]` | `[integer_no_overflow(sub,signed=<operand signed>)]` |
| signed/unsigned multiply | `[]` | `[integer_no_overflow(mul,signed=<operand signed>)]` |
| signed negate instruction | `[]` | `[integer_no_overflow(neg,signed=true)]` |
| unsigned negate | `[]` | instruction rejects |
| signed divide | `[divisor_nonzero]` | `[divisor_nonzero,signed_divrem_representable(div)]` |
| signed remainder | `[divisor_nonzero]` | `[divisor_nonzero,signed_divrem_representable(rem)]` |
| unsigned divide/remainder | `[divisor_nonzero]` | `[divisor_nonzero]` |
| shift with signed RHS | `[shift_count_nonnegative]` | `[shift_count_nonnegative,shift_count_less_than_width]` |
| shift with unsigned RHS | `[]` | `[shift_count_less_than_width]` |
| array index | `[index_in_bounds]` | `[index_in_bounds]` |
| every other instruction | `[]` | `[]` |

For Go, arithmetic overflow wraps, and a nonnegative shift count at least the
LHS width is valid and returns the total shift result. For Rust, those same
source operations require no-overflow or less-than-width proofs. A leading
unary-minus Rust literal accepted by rustc, including `-128_i8`, is one signed
`Const` literal and emits no `UnaryOp` or overflow check; every emitted Rust
signed `UnaryOp bv_neg`, including one whose operand is a VIR literal or
constant reference, uses the check shown in the table. Rust `Index` requires an
unsigned BV whose width equals `pointer_width`; Go accepts signed or unsigned BV
indexes. Extra checks reject even when they are logically true.

## 10. Normalized contracts

### 10.1 `VirContract`

Each function has one contract object with exactly:

| Field | Rule |
|---|---|
| `unit_id` | containing unit ID |
| `function_id` | containing function ID |
| `semantic_profile` | exact module value |
| `semantic_parameters` | exact module object |
| `requires` | ordered Boolean expression array; may be empty |
| `ensures` | ordered Boolean expression array; nonempty |
| `modifies` | present and exactly `[]` |
| `panic` | exactly `forbidden` |
| `termination` | `total` or, for cyclic Go only, `partial` |
| `loops` | ordered `LoopContract` array |
| `contract_hash` | 64 lowercase hexadecimal characters |

Rust requires `termination: "total"` and `loops: []`. Acyclic Go also requires
`total` and no loop contracts. Cyclic Go has one loop contract per validated
header. It is `total` exactly when every loop has one or more `decreases`
expressions; otherwise it is `partial`. In a partial contract, every
`decreases` array MUST be empty so one normalized function cannot mix partial
and total loop claims.

A `LoopContract` has exactly `header`, `invariants`, and `decreases`.
`header` is the canonical `bbN` cutpoint, `invariants` is ordered and nonempty,
and `decreases` is ordered. Loop contracts follow header order in `blocks`.
Source locations are excluded and live in the source map.

### 10.2 Contract expressions

Contract expressions are closed exact-shape objects:

| Form/operator | Exact fields | Type rule |
|---|---|---|
| variable atom | `var` | normalized visible binding ID |
| result atom | `result` | JSON integer in `0..results.length-1`; type is that result binding's type |
| Boolean atom | `bool` | bool |
| integer atom | `int` | section 4.4 typed literal |
| `not` | `op`, `value` | bool -> bool |
| `and`, `or` | `op`, `args` | 2 through 64 ordered bool operands -> bool |
| `eq`, `not_eq` | `op`, `lhs`, `rhs` | exact same type -> bool |
| signed/unsigned comparison | `op`, `lhs`, `rhs` | matching signedness and BV type -> bool |
| BV binary operator | `op`, `lhs`, `rhs` | program equations; exact BV types except shift RHS |
| `bv_neg`, `bv_not` | `op`, `value` | BV -> same BV |
| `convert` | `op`, `value`, `type` | Go only; BV -> declared BV type |

The exact comparison names are `signed_lt`, `signed_le`, `signed_gt`,
`signed_ge`, `unsigned_lt`, `unsigned_le`, `unsigned_gt`, and `unsigned_ge`.
The exact binary BV names are `bv_add`, `bv_sub`, `bv_mul`, `bv_and`, `bv_or`,
`bv_xor`, `bv_shl`, `bv_ashr`, and `bv_lshr`. Go contracts additionally accept
`bv_sdiv`, `bv_udiv`, `bv_srem`, and `bv_urem`; Rust contracts reject division,
remainder, and `convert`.

Contract `bv_sdiv` and `bv_srem` require signed operands; `bv_udiv` and
`bv_urem` require unsigned operands. `bv_ashr` requires a signed LHS and
`bv_lshr` an unsigned LHS. Shift RHS types follow the full-count rule and may
differ from the LHS. Other binary BV operands match exactly. Contract `bv_neg`
and `bv_not` are total for either signedness, even though Rust source-program
unsigned negation is not an accepted instruction.

Contract BV operations are total logical expressions and never add runtime
safety checks. Operand arrays and clause arrays retain normalized source order.
The normalizer MUST NOT flatten, reassociate, commute, sort, deduplicate, or
constant-fold expression trees.

Requires may reference arguments. Ensures may reference arguments and declared
result indexes. Go loop invariants and decreases may also reference locals and
the named header's block parameters. Constants used by source contracts are
resolved and replaced by typed literal atoms during normalization; there is no
contract constant-reference atom. Rust contract variables resolve only to source
parameters and are renamed to `argN`; locals are not contract-visible. An
expression may not refer to an instruction result.

Every invariant has type bool. Every decreases expression has BV type. For a
total loop, each listed expression independently generates a strict-decrease
obligation, using `S_w(after) < S_w(before)` for a signed type and
`U_w(after) < U_w(before)` for an unsigned type. A signed expression
additionally generates `S_w(before) >= 0`. Multiple expressions are not a
lexicographic tuple. At least one decreases expression per loop is sufficient
to select `termination: "total"`, and every listed expression must satisfy its
own obligations.

Aggregate values occur only as exact-typed variables or results in `eq` and
`not_eq`. Array equality is ordered component equality. Struct equality
requires the same nominal type and is ordered field equality; `not_eq` is the
checked logical negation of that aggregate equality. Field selection, indexing,
aggregate literals, and aggregate conversion are not contract expressions.

### 10.3 Contract hash and calls

`contract_hash` is recomputed as:

```text
SHA256(
  UTF8("MPK-CONTRACT-0.1") || 0x00 ||
  JCS(contract object with only contract_hash removed)
)
```

The domain text is exactly the 16 ASCII bytes shown, the separator is exactly
one zero byte, and the JCS payload has no BOM, LF, or trailing whitespace.
Every `CallStatic` repeats the resolved callee's recomputed hash. A mismatched
function ID, signature, semantic context, or hash rejects.

The raw sidecar is a source-manifest input and has its own raw input SHA-256.
Its whitespace, object-key order, and source variable spellings never enter the
normalized contract. Changing only raw JSON whitespace changes source
traceability but not `contract_hash` or `vir_hash`.

## 11. Canonical VIR and `vir_hash`

Canonical JSON is RFC 8785 JCS narrowed by section 1. A producer constructs the
following semantically unordered collections in their required sorted order,
and a validator independently computes that order and rejects when the input
array differs:

- modules' `units` by unit ID;
- independent type declarations by the dependency/ID rule;
- constant declarations by ID;
- functions by callee-first topological/ID order;
- `features_used` by UTF-8 bytes;
- safety checks by section 9.1.

All other arrays are ordered and MUST NOT be sorted. After order validation,
JCS hashes every array exactly as stored. No importer silently rewrites an
input collection before hashing.

`vir_hash` is:

```text
SHA256(
  UTF8("MPK-VIR-0.1") || 0x00 ||
  JCS(VirModule with only the root vir_hash field removed)
)
```

The domain is exactly the 11 ASCII bytes `MPK-VIR-0.1`. Only the root
`vir_hash` member is removed. Contract hashes, source language, semantic
profile, every semantic parameter, unit identities, declarations, function
order, checks, and contracts remain in the payload. The JCS bytes contain no
BOM, LF, or trailing whitespace. A CLI's single transport LF is outside this
hash preimage.

Hash comparison decodes neither case-insensitive hexadecimal nor base64: only
64 lowercase hexadecimal characters are valid. Implementations compute with
checked length counters and compare the decoded 32 bytes without using a
pretty-printed representation.

## 12. Shared deterministic limits

The implicit shared limit profile for `mpk.vir.v0` is
`mpk.vir.limits.v0`. It is selected by the schema and is not a configurable VIR
field. Counts use checked unsigned arithmetic. A shared limit breach rejects
before allocation of the complete corresponding collection and emits no
partial downstream artifact.

`input_json_bytes` counts every input byte presented to the standalone VIR
parser, including insignificant JSON whitespace and excluding any containing
frontend-protocol bytes. `canonical_json_bytes` counts JCS of the complete root
object including `vir_hash`. The standalone input ceiling is intentionally
larger than the canonical ceiling so a compact input can reach the canonical
size diagnostic; a containing frontend envelope applies its separately frozen
whole-transport ceiling. A JSON object or array opens one nesting level; the
root is level 1, and scalars do not add a level. `string_bytes` counts the UTF-8
encoding of the decoded string value, not quotes or JSON escapes.

Aggregate nesting counts each array-to-element or struct-to-field type edge;
the outermost aggregate contributes one and a primitive contributes zero.
Contract-expression nesting counts the root atom/operator as one and takes the
maximum child path. Contract node counts include every atom and operator once.
CFG edge counts include each `Jump` edge once and both `Branch` edges even when
they reach a previously discovered block. Module totals are checked sums across
units and functions.

| Limit ID | Inclusive maximum |
|---|---:|
| `canonical_json_bytes` | 201,326,592 (192 MiB) |
| `input_json_bytes` | 268,435,456 (256 MiB) |
| `json_nesting` | 256 levels |
| `string_bytes` | 1,048,576 per string |
| `identifier_bytes` | 1,024 per public or internal ID |
| `units` | 256 per module |
| `type_decls` | 4,096 per module |
| `const_decls` | 65,536 per module |
| `functions` | 8,192 per module |
| `params` | 256 per function |
| `results` | 16 per function |
| `locals` | 65,536 per function |
| `blocks_per_function` | 8,192 |
| `blocks_per_module` | 65,536 |
| `block_parameters` | 4,096 per block |
| `instructions_per_block` | 100,000 |
| `instructions_per_function` | 100,000 |
| `instructions_per_module` | 250,000 |
| `cfg_edges_per_function` | 16,000 |
| `call_args` | 256 per call |
| `array_elements` | 256 |
| `struct_fields` | 64 |
| `aggregate_type_nesting` | 16 edges |
| `contract_clauses` | 64 total requires plus ensures per function |
| `contract_expr_nodes_per_function` | 1,024 including loops |
| `contract_expr_nodes_per_module` | 8,192 |
| `contract_expr_nesting` | 32 levels |
| `loops` | 1,024 per module |
| `loop_invariants` | 64 per loop |
| `loop_decreases` | 64 per loop |

Lower bounds fixed elsewhere are also limits: a module has at least one unit,
a unit has at least one function, a function has at least one block, and a
contract has at least one ensures clause.

Language profiles may be stricter. Rust v0 additionally limits the selected
call closure to 128 functions, reachable blocks to 1,024 per function and
8,192 across the closure, source MIR statements to 100,000 per function and
250,000 across the closure, arrays to 256 elements, structs to 64 fields, and
aggregate nesting to 16. These Rust limits are enforced before or during
lowering and again where represented in VIR. An implementation cannot tighten
or relax a limit by host configuration.

## 13. Conformance vectors and test ownership

`develop/specs/vectors/vir-v0.json` has schema
`mpk.vir.conformance.v0`. Its exact top-level fields are `schema`,
`spec_schema`, `owner_tests`, `module_cases`, `type_cases`,
`instruction_cases`, `terminator_cases`, `contract_cases`,
`safety_check_cases`, `profile_cases`, and `limit_cases`. Case IDs are unique
across all arrays and arrays retain file order.

An `input` field is the exact JSON value passed to the named model validator.
A `json_text` field is passed as UTF-8 bytes and exists for lexical cases such
as duplicate names. A case contains exactly one of those two fields unless its
`construction` recipe derives the value. `context.validator`, when present,
selects the named declaration, expression, or contract fragment validator;
the other `context` fields supply declared types and profile and are not merged
into the fragment. `expect` has `outcome` (`accept` or `reject`) and, on
rejection, stable `code`. For value
equation cases, `result` is the signed or unsigned decimal interpretation named
by the input type, and `check_results` is one Boolean per `safety_checks` entry
in the same order.

A non-limit `construction` has exactly `base`, optional `pointer`, and
`patches`. `base` is an earlier module-case ID and resolves to that case's
`input`; absent `pointer` means its root. A pointer is RFC 6901 JSON Pointer.
`patches` is an ordered RFC 6902 subset using only `add`, `remove`, and
`replace`; each operation has exactly `op` and `path`, plus `value` except for
`remove`. Paths are relative to the selected value. Every path must resolve as
required by its operation. Hashes are not implicitly repaired, so an invalid
mutation reaches the earliest validation phase fixed in section 1.

For the two `profile_cases` using
`cfg_shape: "validated_single_header_loop"`, the profile-model fixture is one
function with bool `arg0` and one bool result. Its canonical blocks are `bb0`
through `bb3`: `bb0` jumps to header `bb1`; `bb1` branches on `arg0` with
`else_label: "bb2"` as the exit and `then_label: "bb3"` as the body; `bb2`
returns `arg0`; and `bb3` jumps back to `bb1`. All successor-argument and block-
parameter arrays are empty. `loop_contracts: "exact"` supplies one `bb1`
invariant `{"bool":true}`, no decreases, and partial termination;
`loop_contracts: "empty"` supplies no loop contract and total termination.
The fixture uses the selected profile's valid language, semantic parameters,
and IDs and recomputes both hashes.

A limit `construction` has exactly `fixture` and `count`. Size and lexical
fixtures feed exactly `count` bytes or levels to the corresponding streaming
counter. Collection fixtures start from the accepted Go identity module, replace
the collection named by `fixture` with `count` smallest well-typed elements,
assign dense internal IDs and zero-padded public-name numeric suffixes of width
`len(decimal(max(count-1,0)))`, update every dependent binding, declaration,
edge, terminator, contract, and derived `features_used` entry, restore every
canonical collection order, and recompute affected contract and VIR hashes.
Every `at`/`above` builder keeps all other limit counters within their bounds
and introduces no same-or-earlier-phase failure; `above` changes only the named
counter. The intentional `at` failures for `unknown_nested_value` and
`unknown_string` are the two stated exceptions.
Coupled module-total fixtures use the minimum number of functions and blocks
needed to stay within every per-owner limit. CFG fixtures use forward targets.
Loop fixtures use disjoint three-block header/body/exit components with one
`{"bool":true}` invariant each and distribute components across the minimum
number of functions needed to keep both blocks and contract-expression nodes
within their per-function limits. Expression fixtures use a balanced `and` tree
except `nested_not_contract`, which uses an exact unary chain.
`canonical_size` builds a valid acyclic Go module with two
zero-parameter, one-bool-result callees and the minimum number of caller
functions and blocks allowed by the per-function and per-block limits. Caller
blocks contain dense, otherwise unused `CallStatic` results. The two callee IDs
are legal same-unit IDs that differ by one trailing ASCII `a`; both have the
same signature but independently recomputed contracts. The builder enumerates
legal unit-prefix lengths, total call counts, and the number of calls to the
one-byte-longer callee in that tuple order, assigns the longer calls last in
canonical instruction order, and uses the lexicographically smallest legal
value for every other unconstrained public name and `false` for unconstrained
Boolean literals. It recomputes every contract hash and `vir_hash`, then selects
the first module whose complete-root JCS length is exactly `count`.
The fixed at-limit and above-limit counts in the vector are required to have a
solution within all other limits; absence of a solution fails the owning test.
`transport_size` instead starts from the accepted Go identity module and adds
JSON spaces immediately after the root opening brace until the input byte count
is exact. This rule makes both byte-boundary cases executable without storing
hundreds of MiB in Git.

The owning tests are:

- `crates/mpk-vc/tests/vir_model.rs` for module/type/instruction/terminator
  tagged-union shapes;
- `crates/mpk-vc/tests/vir_validation.rs` for references, graphs, contracts,
  profiles, checks, and limits;
- `crates/mpk-vc/tests/vir_hash.rs` for `vir-hash-v0.json`.

Those implementation milestones MUST load every case, reject an unknown vector
schema or case field, and prove that no vector is silently skipped.

`develop/specs/vectors/vir-hash-v0.json` has schema
`mpk.vir.hash_vectors.v0`. Its exact top-level fields are `schema`,
`spec_schema`, `source_vector`, `owner_test`, `domains`, `canonical_cases`,
`canonical_equivalence_cases`, `raw_contract_cases`, `ordered_array_cases`, and
`mutation_cases`. A `source_case` always resolves directly to that module
case's `input`, and every `source_pointer` or patch path is relative to that
value. It freezes exact canonical UTF-8 strings, domain preimage hashes,
contract normalization, root-hash exclusion, object-key and whitespace
invariance, ordered-array mutation, and every repeated semantic-context
mutation. A mutation payload is hashed as a JSON value even when the isolated
mutation would make the module fail an earlier consistency rule; these cases
test hash sensitivity, not importer precedence. In
`canonical.domain_separator_required`, `wrong_domain_text` is the exact UTF-8
domain substituted for `MPK-VIR-0.1` while retaining the `0x00` separator; its
digest is `wrong_domain_sha256`. Its lengths and lowercase SHA-256 values are
normative.
