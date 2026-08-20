# Verification Condition v1 Specification

Status: normative and frozen for implementation.

This specification defines the untrusted verification-condition document
`mpk.vc.v1` and its untrusted grouped theorem-declaration skeleton
`mpk.vc.cert_skeleton.v1`. A valid document or skeleton is not proof evidence.
Only the resulting certificate declarations accepted by both configured MPK
checkers are proof evidence.

The two v1 schemas are intentionally incompatible with the pre-cutover GIR VC
model. There is no adapter at either public boundary.

## 1. Conformance language and validation order

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
the stage returns no VC, certificate skeleton, partial certificate, policy
evidence, or proof-pending artifact.

Every object and tagged union is closed. Unknown or inapplicable fields,
missing required fields, duplicate JSON object names, and `null` reject.
Strings and enum values are case-sensitive. A `Sha256` is exactly 64 lowercase
hexadecimal characters. JSON strings contain only Unicode scalar values and
receive no Unicode normalization. Floating-point JSON numbers and integral
numbers outside `[-9007199254740991, 9007199254740991]` reject.

Serialized VC and skeleton artifacts are RFC 8785 JCS, narrowed by the JSON
rules above, with no BOM, leading or trailing whitespace, or transport LF. An
importer compares the received bytes with its JCS re-encoding and rejects a
noncanonical transport. Object-key order in an in-memory vector value is not
semantic; every ordered array rule below is semantic and JCS never sorts an
array.

VC validation uses this first-error phase order:

1. `transport`: the applicable 256 MiB artifact byte ceiling, UTF-8,
   duplicate-name detection, JSON number syntax, and trailing bytes, using an
   iterative parser whose counters do not require constructing the tree;
2. `shape`: closed objects and unions, exact `schema`, required fields, and
   exclusion of retired spellings;
3. `scalar`: hash, ID, ordinal, MPK-name, literal, and type-term scalar rules;
4. `stream_limits`: member, assumption, expression-node, and expression-depth
   counters while parsing;
5. `linkage`: exact source VIR, input-set, profile, parameters, limit profile,
   function, contract, parameter, and `requires` repetition;
6. `members`: member-ID shape/function/uniqueness, strict array order, then
   complete regenerated member equality;
7. `groups`: the exact two groups and exhaustive one-to-one member partition;
8. `dependencies`: reference closure, order, acyclicity, and the exact
   necessary-and-sufficient generated edge set;
9. `theorem_limits`: deterministic grouped theorem-type depth;
10. `canonical_size`: JCS size of the complete root including `vc_hash`;
11. `canonical_transport`: equality of received and re-encoded JCS bytes;
12. `hash`: recomputed `vc_hash`.

Within `linkage`, validate source schema/hash/input-set and manifest linkage
first (`VC_SOURCE_LINKAGE`), semantic profile/parameters and the verification
limit profile second (`VC_PROFILE_LINKAGE`), the exact function set and
callee-first order third (`VC_FUNCTION_ORDER`), and the matched functions'
contract hashes, parameters, and `requires` arrays fourth
(`VC_SOURCE_LINKAGE`).

Skeleton validation uses the same `transport`, `shape`, `scalar`, and
`stream_limits` phases, followed by:

1. `vc_linkage`: parse and validate the complete source VC bytes, recompute
   `source_vc_hash`, and compare every repeated root identity;
2. `declarations`: exact declaration count/order, group/member repetition, and
   dependency repetition;
3. `theorem_type`: exact binder and grouped proposition reconstruction;
4. `theorem_limits`, `canonical_size`, and `canonical_transport`.

The first failing phase owns the stable code in section 10. A later hash error
cannot hide a missing member, extra dependency, or limit failure.

The exact retired root names `schema_version`, `source_gir_hash`, `gir_hash`,
and `source_manifest_hash` are unknown fields and reject with the applicable
shape code. In particular, `schema_version: "mpk.vc.v1"` is not an alias for
`schema`, and `source_gir_hash` is never interpreted as `source_ir_hash`.

## 2. VC document root and repeated identities

The `mpk.vc.v1` root has exactly these fields:

| Field | Type and rule |
|---|---|
| `schema` | exactly `mpk.vc.v1` |
| `source_ir_schema` | exactly `mpk.vir.v0` |
| `source_ir_hash` | recomputed VIR `vir_hash` |
| `input_set_hash` | recomputed source-manifest input-set hash |
| `semantic_profile` | exactly `mpk.go.fixed.v0` or `mpk.rust.checked.v0` |
| `semantic_parameters` | exact profile object from `VIR_V0.md` |
| `verification_limit_profile` | exactly `mpk.verify.limits.v0` |
| `functions` | nonempty canonical `VcFunction` array |
| `vc_hash` | v1 self-hash from section 8 |

No field is optional. The root does not repeat `source_language`: the validated
VIR and the registered semantic-profile tuple own that pairing. It does not
contain a frontend or certificate-stage `source_manifest_hash`, avoiding a
manifest/VC hash cycle.

Successful generation receives validated canonical VIR bytes and validated
canonical frontend-stage source-manifest bytes, not hashes alone. It requires:

- `source_ir_schema` and `source_ir_hash` to equal the validated VIR;
- `input_set_hash` to equal the manifest;
- `semantic_profile` and `semantic_parameters` to be member-for-member equal
  across VIR, every VIR contract, manifest, and VC;
- the manifest `vir_hash` to equal `source_ir_hash`; and
- `verification_limit_profile` to be selected by the registered VC/checker
  profile and to equal the literal above, never a user flag.

Changing any repeated identity changes `vc_hash`. Certificate-stage source
manifest attachment recomputes this exact hash and copies it to the manifest's
`vc_hash` field as specified by `SOURCE_MANIFEST_V0.md`.

## 3. Canonical type and expression terms

VC v1 stores unresolved, name-based MPK terms. Certificate assembly resolves
the names only against checked imports and declarations. A solver builtin,
host integer, GIR-era `Std.Go.Base.*` name, or caller-supplied global numeric ID
is not an alternate encoding.

### 3.1 `VcTypeTerm`

`VcTypeTerm` is this exact `kind`-tagged union:

| `kind` | Other exact fields | Rule |
|---|---|---|
| `constant` | `name` | valid fully qualified MPK name |
| `apply` | `function`, `args` | MPK name and ordered `VcTypeTerm` array |
| `nat_literal` | `value` | safe nonnegative JSON integer |
| `string_literal` | `value` | Unicode scalar string |

The canonical VIR-to-type mapping is:

| VIR type | `VcTypeTerm` |
|---|---|
| bool | `constant("Std.Program.Base.Bool")` |
| signed BV width 8/16/32/64 | `constant("Std.Program.Base.Int8")`, `Int16`, `Int32`, or `Int64` in that namespace |
| unsigned BV width 8/16/32/64 | `constant("Std.Program.Base.Uint8")`, `Uint16`, `Uint32`, or `Uint64` in that namespace |
| array `[T; N]` | `apply("Std.Program.Base.Array", [encode(T), apply("Std.Program.Base.Array.Length", [nat_literal(N)])])` |
| nominal struct D | `apply("Std.Program.Base.Struct.Value", [Shape(D)])` |

No other bit width or alias is inferred. `Shape(D)` is
`apply("Std.Program.Base.Struct.Shape", args)`. Its first argument is
`string_literal(D.id)` and each following argument is, in declaration field
order,
`apply("Std.Program.Base.Struct.Field", [string_literal(field.name),
apply("Std.Program.Base.Struct.FieldType", [encode(field.type)])])`.
This includes the fully qualified nominal declaration ID and therefore cannot
merge equal-layout structs.

### 3.2 `VcTerm`

`VcTerm` is this exact `kind`-tagged union:

| `kind` | Other exact fields | Rule |
|---|---|---|
| `var` | `name` | one containing function parameter ID |
| `bound` | `index` | safe nonnegative JSON integer; zero-based de Bruijn reference to an enclosing inline `forall` or member-local binder |
| `constant` | `name` | valid checked MPK name |
| `bit_vec_literal` | `value`, `width`, `signed` | `value` is the canonical base-10 string from `VIR_V0.md` section 4.4, `width` is 8/16/32/64, `signed` is Boolean, and the value fits that signed or unsigned width |
| `apply` | `function`, `args` | checked MPK name and ordered `VcTerm` array |
| `convert` | `value`, `target` | one `VcTerm` and one `VcTypeTerm` |
| `forall` | `binder_type`, `body` | inline anonymous Pi binder and Boolean body |

Objects of the old `result` kind reject. WP substitutes results, instruction
results, and every acyclic local/block-parameter value before member
serialization; remaining loop-cutpoint state is converted to the anonymous
member-local binders defined below. Every `var` therefore closes over exactly
one function parameter binder. A relational
`CallStatic` fresh result is instead one anonymous inline `forall` binder at the
exact WP point where it is introduced; occurrences use `bound`, where index
zero is the nearest enclosing binder. Member-local loop-state binders are
declared separately by `VcMember.local_binders` below and share this de Bruijn
namespace. An out-of-range index rejects. Source binder names, compiler
temporary names, and alpha-renaming never enter this form. After resolving
outer parameter binders, member-local binders, and inline binders, every
serialized member is closed.

When lowering to the core's one de Bruijn namespace, member-local binders are
first installed in their specified outer-to-inner order. Entering an inline
`forall` then pushes its anonymous binder at index zero and shifts existing
member-local and outer-parameter indices by one. A `bound` index addresses the
nearest inline binder first and then the member-local stack; a `var` name
addresses only the function parameter table and is shifted by the complete
member-local plus inline-binder depth. With `p` function parameters, parameter
array element `j` has base body index `p - 1 - j` before that shift. The two
serialized forms cannot alias.

For a call returning type `R` with encoded callee postconditions `ensures` and
continuation `K`, the result-binding fragment is exactly
`forall(encode(R), Imp(Conjoin(ensures), K))`. Nested calls nest `forall` at
their WP introduction points; they are never hoisted, commuted, or replaced by
an existential, uninterpreted function, Hilbert choice, or implementation-body
substitution. Certificate assembly maps `forall` to a checked core Pi. Program
constant references are replaced by their normalized bool/BV literal values
before serialization; `constant` is reserved for checked MPK and generated
declaration names.

The exact logical constructors used by grouping are:

```text
True       = {"kind":"constant","name":"Std.Bool.true"}
And(x, y)  = {"kind":"apply","function":"Std.Bool.and","args":[x,y]}
Imp(x, y)  = {"kind":"apply","function":"Std.Logic.Imp","args":[x,y]}
```

`Std.Bool.true`, `Std.Bool.and`, and `Std.Logic.Imp` resolve to checked
declarations. No alternate identity, Boolean constant spelling, variadic
`and`, or omitted implication is equivalent for serialization.

For both type and expression terms, structural node count includes the root and
every repeated child occurrence. The expression-node limit counts only
`VcTerm` nodes: `convert.target` and `forall.binder_type` are validated type
trees but do not add expression nodes. Depth is one for a leaf and
`1 + max(child depths)` for a nonleaf; type children of `convert` and `forall`
do participate in depth. Function-name and literal payloads are not child
nodes. Validation type-checks every term using the source VIR signature and the
checked foundation interfaces.

## 4. Functions and source-language-neutral members

### 4.1 `VcFunction`

Each `functions` element has exactly:

| Field | Type and rule |
|---|---|
| `function_id` | exact VIR function ID |
| `contract_hash` | recomputed hash of that function's normalized VIR contract |
| `parameters` | ordered `VcBinder` array |
| `requires` | ordered encoded function `requires` terms |
| `members` | complete canonical `VcMember` array |
| `groups` | exactly two `VcGroup` values |

`VcBinder` has exactly `id` and `type`. IDs are exactly the VIR `arg0`,
`arg1`, ... sequence and `type` is the section 3.1 encoding. Parameters retain
signature order. Results, locals, block parameters, and temporaries never
become outer declaration binders. Free loop-cutpoint locals and header
parameters become only the anonymous member-local binders in section 4.2.

Functions are the exact VIR function set in its module-wide callee-first Kahn
topological order. When more than one function is ready, the smallest function
ID by UTF-8 bytes is chosen. A source-closure member that has no reachable VIR
call remains a standalone VC function. No function may be missing or added.

`requires` is member-for-member equal to the encoded normalized VIR contract
array and retains its order. Function `requires` appear only in the outer group
implication in section 7; a generator MUST NOT copy them into each member's
`assumptions`.

### 4.2 `VcMember`

A member has exactly:

| Field | Type and rule |
|---|---|
| `id` | canonical obligation ID below |
| `function_id` | exact containing `VcFunction.function_id` |
| `kind` | one closed kind from the table below |
| `local_binders` | ordered anonymous `VcTypeTerm` array; may be empty |
| `assumptions` | ordered path-specific `VcTerm` array |
| `conclusion` | one Boolean `VcTerm` |
| `group_id` | exact containing group ID |

The closed kind set and exhaustive group mapping are:

| Kind | Meaning and ordinal origin order | Required group |
|---|---|---|
| `postcondition` | return block order, then `ensures` clause order | `contract` |
| `callee_precondition` | block, instruction, then callee `requires` clause order | `contract` |
| `loop_initialization` | loop-header then invariant order | `contract` |
| `loop_preservation` | loop-header, backedge-source block, then invariant order | `contract` |
| `loop_exit` | loop-header then function `ensures` clause order | `contract` |
| `loop_decreases` | loop-header, decreases-expression order; for each expression, its one signed-nonnegative check first when signed, then strict-decrease checks by backedge-source block order | `contract` |
| `operation_safety` | block, instruction, then VIR `safety_checks` order | `panic_free` |
| `callee_panic_free` | block then `CallStatic` instruction order | `panic_free` |

Block, instruction, header, and backedge order above are their canonical VIR
array orders. Ordinals are assigned independently for each `(function_id,
kind)` after this origin ordering and are dense from zero. Every ID is exactly:

```text
FUNCTION_ID "#" KIND "#" ORDINAL6
```

`ORDINAL6` is six ASCII decimal digits from `000000` through `999999`.
Examples are `demo::f#postcondition#000000` and
`example.com/p.F#operation_safety#000003`. VIR function-ID grammars exclude
`#`, making the split unambiguous. Source block labels, source file paths,
source-language names for obligation categories, and diagnostic prose do not
enter the ID.

Each distinct required check creates one member, including both checks for a
Rust signed division/remainder operation. Each reachable call creates one
`callee_panic_free` member and one `callee_precondition` member per callee
`requires` clause. Multiple call sites remain multiple members; declaration
dependencies deduplicate callees separately.

`local_binders` closes free loop-cutpoint state without exposing compiler or
source names. For a member generated under one validated loop header, perform
the deterministic WP substitutions first, collect every remaining referenced
non-parameter binding across all assumptions and the conclusion, and require
each to be either a function local or that header's block parameter. Retain
referenced function locals in `VirFunction.locals` order, followed by
referenced header parameters in the header's `parameters` order, encode only
their types, and replace their occurrences with `bound` indices. The first
array binder is outermost and the last is nearest the member body; with `n`
member binders, array element `i` is therefore referenced at body index
`n - 1 - i` before any inline `forall` shift.

VIR loops are disjoint, so one member cannot require state from two headers.
Members whose WP origin is outside a loop cutpoint, including loop
initialization members after preheader substitution, have exactly
`local_binders: []`. Safety and call members generated inside a loop use the
same rule as preservation, exit, and decreases members. A remaining temporary,
instruction result, unrelated block parameter, or binding from another header
rejects instead of becoming a binder.

For a member, `assumptions` contains only its ordered path-specific facts.
`conclusion` is the exact encoded goal. A `callee_panic_free` conclusion is the
callee panic-free proposition instantiated with the call arguments, and its
proof must use the named checked callee declaration dependency. No member may
be inferred from its ID alone.

The `members` array is strictly increasing by `id` in UTF-8 byte order. IDs are
unique document-wide. Linked validation deterministically regenerates all
members from the validated VIR and requires exact equality of ID, function,
kind, local binders, assumptions, conclusion, and group. Missing, duplicate,
altered, or extra members reject; an importer never repairs or silently
reorders them. Within the `members` phase, validate every ID's grammar and
containing function and document-wide uniqueness first, validate strict array
order second, and compare the complete regenerated member array third. This
order makes a duplicate ID `VC_MEMBER_ID` even though it also prevents strict
increase.

## 5. Groups, declaration names, and exact dependencies

### 5.1 `VcGroup`

Each group has exactly:

| Field | Type and rule |
|---|---|
| `id` | `FUNCTION_ID.contract` or `FUNCTION_ID.panic_free` |
| `kind` | exactly `contract` or `panic_free` |
| `declaration_name` | exact reversible name below |
| `member_ids` | strictly increasing member-ID array; may be empty |
| `dependencies` | strictly increasing generated declaration-name array |

Every function has exactly two groups in this order: `contract`, then
`panic_free`. `id` is the exact function ID followed by the shown suffix.
`member_ids` contains every and only member whose `group_id` equals the group
ID, exactly once. It is not a policy-selected subset. A contract group is
nonempty because every valid VIR contract has an `ensures`; a panic-free group
may be empty.

Within the `groups` phase, validate the two groups' structural fields and
order first (`VC_GROUP_SHAPE`), document-wide one-to-one membership second
(`VC_GROUP_PARTITION`), and each member kind's required group third
(`VC_GROUP_KIND`). The final exact `member_ids` comparison follows from those
checks and the required UTF-8 order; it does not collapse a wrong-kind case
into a partition error.

A canonical checked MPK declaration name must support Go slash paths and Rust
`::` IDs without collisions. Define `hex_id(F)` as the lowercase two-digit hex
encoding of every byte of UTF-8 function ID F, with no separator. Then:

```text
declaration_name(F, contract)   = "VC.Function.f" || hex_id(F) || ".contract"
declaration_name(F, panic_free) = "VC.Function.f" || hex_id(F) || ".panic_free"
```

The `f` prefix makes the hex component a valid MPK name component. The mapping
is reversible and distinct for every valid VIR function ID. Sanitizing invalid
characters to `_`, hashing an ID, preserving `/` or `:`, uppercase hex, or
using the old `VC.Obligation.*` namespace rejects.
`VC.Function` is reserved to the grouped emitter within the generated
certificate module; assembly rejects a checked import or preexisting local
declaration that would collide with any generated name.

### 5.2 Necessary and sufficient dependency edges

Let `C(F)` and `P(F)` be F's contract and panic-free declaration names. Let
`Direct(F)` be the set of distinct callees occurring in reachable VIR
`CallStatic` instructions in F. It is not the conservative HIR source closure.
The only generated group dependency sets are:

```text
deps(C(F)) = { C(G) | G in Direct(F) }

deps(P(F)) = { C(F) }
           union { C(G), P(G) | G in Direct(F) }
```

Every name appears once and `dependencies` is sorted by referenced declaration
name in UTF-8 byte order. These equations are both necessary and sufficient:

- `C(G)` permits use of G's checked postconditions and call preconditions;
- `P(G)` discharges F's call-safety member for G;
- `C(F)` lets F's panic-free proof reuse F's checked contract members; and
- no other generated group edge is permitted.

Thus `C(F)` never depends on `P(F)`, no callee depends on its caller, and an
HIR-only dead call adds no edge. Multiple reachable call sites to the same
callee add members but not duplicate edges. Dependencies on checked fixed
foundations are resolved from terms and are not duplicated in this generated
group list.

Every dependency resolves to an earlier declaration under the canonical
callee-first function order and contract-before-panic-free order. The validator
checks reference closure first, duplicate/UTF-8 order second, later-reference
or graph cycles third, and exact-set equality fourth. Thus a crafted cycle has
a stable cycle diagnostic even though its added edge is also extra.

## 6. Certificate skeleton schema

The `mpk.vc.cert_skeleton.v1` root has exactly:

| Field | Type and rule |
|---|---|
| `schema` | exactly `mpk.vc.cert_skeleton.v1` |
| `source_vc_schema` | exactly `mpk.vc.v1` |
| `source_vc_hash` | recomputed source VC hash |
| `source_ir_schema` | exact VC value, therefore `mpk.vir.v0` |
| `source_ir_hash` | exact VC value |
| `input_set_hash` | exact VC value |
| `semantic_profile` | exact VC value |
| `semantic_parameters` | exact VC object |
| `verification_limit_profile` | exact VC value, `mpk.verify.limits.v0` |
| `theorem_declarations` | exact `GroupedTheoremDeclaration` array |

There is deliberately no skeleton self-hash. `source_vc_hash` binds the
complete grouped input, and the assembled certificate and declaration
interfaces already use the domain-separated hashes in `CERT_V0.md`. A
`skeleton_hash`, `vc_hash`, `schema_version`, or `source_gir_hash` root member
is therefore unknown and rejects.

A `GroupedTheoremDeclaration` has exactly:

| Field | Type and rule |
|---|---|
| `name` | exact source group `declaration_name` |
| `function_id` | exact source function ID |
| `group_id` | exact source group ID |
| `group_kind` | exact source group kind |
| `member_ids` | exact source group member array |
| `dependencies` | exact source group dependency array |
| `theorem_type` | exact `GroupedTheoremType` below |

`GroupedTheoremType` has exactly `binders` and `body`. `binders` is the exact
source function `parameters` array. `body` is the canonical group proposition
from section 7. The declaration array is the source VC function order, with
each function's contract declaration immediately followed by its panic-free
declaration. Empty arrays are present, never omitted.

Skeleton emission accepts the complete canonical VC bytes, recomputes
`vc_hash`, and rejects any repeated-field, member, group, dependency, binder,
or theorem-type mismatch. It never trusts a caller-supplied source hash or
pretty-printed representation.

The skeleton carries dependency names rather than caller-supplied declaration
hashes. During certificate assembly, every name resolves to the already built
checked declaration; the assembler recomputes its `MPK-DECL-0.1` interface hash
from the candidate certificate and binds the proof reference to that exact
identity. This avoids accepting an asserted hash that was never checked.

After checker acceptance, every policy member row binds at least its obligation
ID, group ID, containing declaration name, and recomputed containing
declaration hash; `POLICY_V1.md` owns the complete closed row shape. A row with
the wrong name or hash, or a row whose complete declaration was not accepted,
is not checked evidence. Individual conjuncts are never independently accepted
declarations.

## 7. Canonical grouped theorem type

For an ordered term array `xs`, define the only permitted conjunction:

```text
Conjoin([])  = True
Conjoin([x]) = x

Conjoin(xs), n >= 2:
  k = floor(n / 2)
  And(Conjoin(xs[0..k]), Conjoin(xs[k..n]))
```

The split is order-preserving and balanced. It is not a fold. Producers and
importers MUST NOT flatten, reassociate, commute, sort, deduplicate, or
constant-fold the tree.

For an ordered type array `ts`, define `ForallMany(ts, body)` by wrapping from
the last array element back to the first, so the first array element is
outermost; the empty array returns `body`. Each wrapper is the exact `forall`
term from section 3.2. For member M:

```text
MemberType(M) =
  ForallMany(
    M.local_binders,
    Imp(Conjoin(M.assumptions), M.conclusion)
  )
```

For group G of function F, preserving `G.member_ids` order:

```text
GroupBody(F, G) =
  Imp(
    Conjoin(F.requires),
    Conjoin([MemberType(member(id)) for id in G.member_ids])
  )
```

Both the member and outer `Imp` nodes are always emitted, including when an
antecedent is `True`. An empty panic-free group therefore has body
`Imp(Conjoin(requires), True)`. A singleton group contains its one member
type directly, including its member-local binders. A logically equivalent
right fold or a proposition with common preconditions distributed into members
is not the canonical theorem type.

Finally, certificate assembly wraps `GroupBody` with one checked Pi binder per
`theorem_type.binders`, in reverse construction order so the serialized binder
array's first parameter is outermost:

```text
Pi(arg0 : T0, Pi(arg1 : T1, ... Pi(argN : TN, GroupBody) ...))
```

Outer binder names are resolution labels only; the core term uses the
corresponding de Bruijn indices. No result, local, path, proof, or implicit
*outer* binder is added. Explicit member-local binders remain inside their
member conjunct, and inline anonymous Pi binders already present as
`VcTerm.forall` for relational calls remain exactly where WP placed them. This
fixed construction ensures equivalent but differently associated input
propositions cannot change the canonical grouped declaration interface.

## 8. Canonical VC and `vc_hash`

`vc_hash` is exactly:

```text
SHA256(
  UTF8("MPK-VC-1.0") || 0x00 ||
  JCS(VcDocument with only the root vc_hash field removed)
)
```

The domain is exactly the ten ASCII bytes `MPK-VC-1.0`, followed by exactly one
zero byte. Only the root `vc_hash` member is removed. All source identities,
profile parameters, functions, contract hashes, binders, requirements,
members, groups, declaration names, and dependencies remain. The JCS preimage
has no BOM, LF, or trailing whitespace.

The following arrays have already been validated in their required order and
are hashed exactly as stored:

- functions in callee-first order;
- parameters and `requires` in semantic order;
- members and group member IDs in UTF-8 ID order;
- groups in contract-before-panic-free order; and
- dependencies in referenced declaration-name UTF-8 order.

An importer never sorts before hashing. Object-key order and insignificant
input whitespace cannot change JCS; every ordered-array mutation either
rejects earlier or changes the hash in an isolated hash-sensitivity test.

## 9. Deterministic limits and failure precedence

`verification_limit_profile = mpk.verify.limits.v0` denotes these inclusive VC
and skeleton limits:

| Limit ID | Inclusive maximum | Counting rule |
|---|---:|---|
| `members_per_function` | 100,000 | all member kinds in one function |
| `members_per_document` | 262,144 | all functions |
| `assumptions_per_member` | 4,096 | one member's array length |
| `expression_nodes_per_member` | 8,192 | assumptions plus conclusion, counting each occurrence |
| `expression_nodes_per_document` | 4,194,304 | `requires`, assumptions, and conclusions |
| `member_expression_depth` | 256 | each stored member assumption or conclusion before grouping |
| `grouped_theorem_depth` | 512 | final balanced body and complete Pi type tree |
| `generated_proof_depth` | 512 | final proof tree for any grouped declaration |
| `canonical_vc_json_bytes` | 268,435,456 (256 MiB) | complete VC JCS including `vc_hash` |
| `canonical_skeleton_json_bytes` | 268,435,456 (256 MiB) | complete skeleton JCS |
| `canonical_certificate_bytes` | 536,870,912 (512 MiB) | complete canonical generated certificate |

The VIR profile supplies the function, parameter, and type-declaration limits;
VC validation does not permit a larger repeated projection. Checked unsigned
arithmetic is used for every counter.

`expression_nodes_per_member` counts no member-local binder types or function
`requires`.
The document expression total counts every serialized `VcTerm` occurrence,
including `requires` and a term repeated in two members; `VcTypeTerm` is
governed by the source VIR type and aggregate limits instead. Generated
`True`, `And`, `Imp`, and generated Pi nodes are not stored VC expression
nodes, but they count toward `grouped_theorem_depth` and the skeleton byte
limit. This includes every member-local wrapper and every outer function
parameter wrapper. `And` and `Imp` have depth
`1 + max(left_depth, right_depth)`. If `Rest` is the remaining Pi/body tree,
one binder has depth
`1 + max(depth(encoded_binder_type), depth(Rest))`. This recurrence, applied
from the last binder back to the first, is the exact final theorem-type depth;
an implementation must not approximate it as only binder count plus body
depth.

`generated_proof_depth` uses the same leaf-one, one-plus-maximum-child
recurrence over the proof-node tree after all generated group proof structure
is present. Expected-type term references do not inline the referenced type
into that proof-depth count. The certificate byte count uses the exact
canonical binary certificate bytes accepted by `CERT_V0.md`, with no transport
wrapper. The same registered profile fixes later policy evidence JSON and
rendered Markdown to 268,435,456 bytes (256 MiB) each; `POLICY_V1.md` owns their
field-specific counters and `POLICY_LIMIT_*` codes but cannot change these
values under `mpk.verify.limits.v0`.

Counters are enforced while reading or generating, before allocating a
complete unbounded collection or tree. The canonical byte limits are checked
after otherwise valid ordered construction and include the complete root. A
limit breach is an artifact-free helper failure with the exact `VC_LIMIT_*`
code below. It occurs after a successful frontend result and never rewrites
`ir-lowered` as a source rejection, emits a partial VC/skeleton/certificate or
policy report, or becomes proof-pending.

When more than one stage could fail, precedence is:

1. an already returned frontend non-success remains authoritative;
2. all VC validation phases in the exact section 1 order;
3. all skeleton validation phases in the exact section 1 order;
4. certificate generation, including generated-proof depth and canonical
   certificate size, followed by both checker results; and
5. policy/evidence/rendering limits.

A downstream stage does not relabel an earlier failure. Wall-clock time,
available memory, thread count, host configuration, or a user option cannot
change these ceilings.

## 10. Stable codes

| Code | Meaning |
|---|---|
| `VC_JSON_DUPLICATE_KEY` | duplicate VC object name |
| `VC_JSON_INVALID` | invalid UTF-8, JSON, number, nesting, string, or trailing bytes |
| `VC_SCHEMA` | wrong VC schema discriminator |
| `VC_SHAPE` | missing, unknown, retired, or wrong-union field |
| `VC_SCALAR` | malformed hash, ID, ordinal, MPK name, literal, or type term |
| `VC_SOURCE_LINKAGE` | source schema/hash, input set, contract, parameter, or requirement mismatch after function matching |
| `VC_PROFILE_LINKAGE` | semantic profile/parameters or verification-limit profile mismatch |
| `VC_FUNCTION_ORDER` | function set or callee-first order mismatch |
| `VC_MEMBER_ID` | duplicate, malformed, or wrong-function member ID |
| `VC_MEMBER_ORDER` | member array is not strictly increasing |
| `VC_MEMBER_SET` | regenerated member is missing, altered, or extra |
| `VC_GROUP_SHAPE` | group ID/name/count/kind/order is wrong |
| `VC_GROUP_PARTITION` | member is ungrouped or appears other than exactly once |
| `VC_GROUP_KIND` | a member kind is assigned to the wrong group |
| `VC_DEPENDENCY_REFERENCE` | a listed dependency resolves to no generated declaration |
| `VC_DEPENDENCY_ORDER` | dependency is duplicate or out of UTF-8 order |
| `VC_DEPENDENCY_CYCLE` | generated declaration graph is cyclic or references a later declaration |
| `VC_DEPENDENCY_SET` | necessary edge is absent or extra edge is present |
| `VC_CANONICAL_TRANSPORT` | received VC bytes are not exact JCS |
| `VC_HASH` | `MPK-VC-1.0` self-hash mismatch |
| `VC_LIMIT_MEMBERS_PER_FUNCTION` | per-function member limit exceeded |
| `VC_LIMIT_MEMBERS_PER_DOCUMENT` | document member limit exceeded |
| `VC_LIMIT_ASSUMPTIONS_PER_MEMBER` | member assumption limit exceeded |
| `VC_LIMIT_EXPRESSION_NODES_PER_MEMBER` | member expression-node limit exceeded |
| `VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT` | document expression-node limit exceeded |
| `VC_LIMIT_MEMBER_EXPRESSION_DEPTH` | stored expression depth exceeded |
| `VC_LIMIT_GROUPED_THEOREM_DEPTH` | final grouped declaration type depth exceeded |
| `VC_LIMIT_GENERATED_PROOF_DEPTH` | final grouped declaration proof depth exceeded |
| `VC_LIMIT_CANONICAL_JSON_BYTES` | complete VC JCS exceeds 256 MiB |
| `VC_SKELETON_JSON_DUPLICATE_KEY` | duplicate skeleton object name |
| `VC_SKELETON_JSON_INVALID` | invalid skeleton JSON transport |
| `VC_SKELETON_SCHEMA` | wrong skeleton schema discriminator |
| `VC_SKELETON_SHAPE` | missing, unknown, retired, or wrong-union skeleton field |
| `VC_SKELETON_VC_LINKAGE` | source VC hash/schema or repeated identity mismatch |
| `VC_SKELETON_DECLARATIONS` | declaration order/group/member/dependency repetition mismatch |
| `VC_SKELETON_THEOREM_TYPE` | binders or balanced proposition differ from reconstruction |
| `VC_SKELETON_CANONICAL_TRANSPORT` | received skeleton bytes are not exact JCS |
| `VC_LIMIT_CANONICAL_SKELETON_JSON_BYTES` | complete skeleton JCS exceeds 256 MiB |
| `VC_LIMIT_CANONICAL_CERTIFICATE_BYTES` | complete generated certificate exceeds 512 MiB |

## 11. Conformance vectors and test ownership

`develop/specs/vectors/vc-v1.json` has schema `mpk.vc.conformance.v1` and
exact top-level fields `schema`, `spec_schema`, `dependencies`, `owner_tests`,
`source_contexts`, `fixtures`, `vc_cases`, and `limit_cases`.

`source_contexts` are vector-only validated-input projections, not production
VC fields. Each freezes the source hashes, profile, canonical function order,
direct callees, parameters, requirements, contract hashes, and regenerated
members needed to test linked validation without copying a complete VIR and
source manifest into this vector. A fixture names one context and contains the
exact generated VC `input`. The owner test first validates the referenced VIR
and manifest fixtures or constructs the exact projection, then proves the VC
input matches it.

The `vc.go_loop_cutpoint` fixture is the scope-composition witness. Its loop
member has one member-local binder and an inline `forall`; inside that inline
body, `bound` index zero names the inline value, index one names the loop-state
binder, and `var: arg0` still names the outer function parameter. The owner
must type-check and reconstruct those scopes, not only compare fixture JSON.

A normal VC case contains exactly one of `input_from`, `transport_from`,
`json_text`, or `construction`. `input_from` names a fixture. A
`transport_from` has exactly `fixture` and `encoding`; it serializes that
fixture with the named encoding and submits the resulting bytes. `json_text`
is submitted as its exact UTF-8 bytes and exists for lexical invalidity and
duplicate-name cases. A `construction` has exactly `base` and ordered patches;
`base` names a fixture or earlier case that resolves to a VC value. Normal
patches are the RFC
6901/RFC 6902 `add`, `remove`, and `replace` subset. The vector-only `copy`
patch copies the JSON value at `from` and inserts it at `path`; `swap` swaps
the `first` and `second` array indices at `path`. These two operations exist
only for duplicate and reorder attacks. Hashes are never repaired implicitly.
`expect` contains outcome and, for rejection, phase and code.

Patch objects are closed: `add` and `replace` have exactly `op`, `path`, and
`value`; `remove` has exactly `op` and `path`; `copy` has exactly `op`, `from`,
and `path`; and `swap` has exactly `op`, `path`, `first`, and `second`. JSON
pointers must resolve under their operation's RFC rule. `add` and `copy` to an
array index insert before the element currently at that index; they do not
replace it. `first` and `second` are distinct in-range safe nonnegative JSON
integers.

A limit case names its exact maximum and has `below`, `at`, and `above`
constructions. Each construction has exactly `fixture` and `count`. The fixture
builder starts from a valid minimal linked VC/skeleton pair, creates the
smallest canonical terms and number of functions needed to make only that
counter exact, restores all derived IDs, groups, dependencies, theorem types,
and hashes, and keeps every other counter below its maximum. Expression-depth
fixtures use a unary `Std.Bool.not` chain; expression-node fixtures use the
shallowest balanced `Std.Bool.and` tree. Canonical-size fixtures choose the
lexicographically smallest legal padding through valid distinct member terms
and function IDs; absence of an exact-size solution fails the owner test.

`develop/specs/vectors/vc-hash-v1.json` has schema
`mpk.vc.hash_vectors.v1`. Its exact top-level fields are `schema`,
`spec_schema`, `source_vector`, `owner_test`, `domain`, `canonical_cases`,
`equivalence_cases`, `ordered_array_cases`, and `mutation_cases`. It names the
source vector and exact fixtures, freezes the `MPK-VC-1.0` domain bytes and
separator, complete-root JCS and preimage lengths, expected SHA-256 values,
root-hash exclusion, object-key/whitespace equivalence, ordered-array
sensitivity, and cross-domain separation. Its vector-only `reverse` patch
has exactly `op` and `path` and reverses the named array without repairing it;
its other patch operations use the common closed shapes above. A hash mutation
is hashed even when linked validation would reject it earlier.
The equivalence encoding
`reverse_each_object_key_order_with_two_space_indent` recursively emits object
keys in reverse JCS key order and otherwise uses the shared pretty layout below.

`develop/specs/vectors/vc-skeleton-v1.json` has schema
`mpk.vc.cert_skeleton.conformance.v1`. Its exact top-level fields are `schema`,
`spec_schema`, `source_vector`, `owner_test`, `fixture_digest_semantics`,
`construction_operations`, `emission_cases`, `mutation_cases`, and
`limit_cases`. Its `emit` constructions consume the canonical VC fixture and
independently assert the complete skeleton JCS SHA-256 plus declaration
projections and theorem-type JCS SHA-256 values. Those SHA-256 values are
fixture byte assertions, not public artifact hashes. A mutation case contains
exactly one of `construction` or `json_text`; skeleton `json_text` has the same
exact-byte meaning as above. Mutation constructions have required fields
`emit_from` and `mutations` and the optional field `transport_encoding`; they
use only the closed operations enumerated in `construction_operations`, are
applied in array order after emission, and never repair a dependent field.
When present, `transport_encoding` serializes the fully mutated skeleton and
submits those bytes. Skeleton `add`, `remove`, `replace`, `copy`, and `swap` use
the common closed shapes above; `reverse` has exactly `op` and `path`; and
`right_associate_group_members` has exactly `op` and an in-range safe
nonnegative `declaration_index`.

The two named pretty encodings retain every array order, use two ASCII spaces
per indentation level and one ASCII space after `:`, and have no final LF.
Each nonempty object or array places LF immediately after its opener, after
each comma, and immediately before its aligned closer; empty objects and arrays
remain `{}` and `[]`. `two_space_indent_no_final_lf`, used by the VC and
skeleton conformance vectors, emits object keys in JCS order; the hash-vector
encoding emits them in reverse JCS order as stated above. Both preserve the
JSON value while violating the required JCS transport.

The owning implementation tests are:

- `crates/mpk-vc/tests/vc_v1.rs` for VC model, linked validation, grouping,
  limits, canonical transport, and `vc-v1.json`;
- `crates/mpk-vc/tests/vc_hash_v1.rs` for `vc-hash-v1.json`; and
- `crates/mpk-vc/tests/vc_skeleton_v1.rs` for grouped emission, theorem types,
  dependency resolution, canonical transport, and `vc-skeleton-v1.json`.

Each owner MUST reject an unknown vector schema or case field, execute every
declared case exactly once, and fail if any case is silently skipped.
