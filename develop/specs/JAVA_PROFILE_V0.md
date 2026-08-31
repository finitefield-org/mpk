# Java Scalar Profile v0 Specification

Status: normative and frozen for implementation by `JAVA-03-T01`
(2026-08-31), but inactive. The installed release remains Go/Rust/C# with
semantic registry revision 2. The disposable compiler/JVM compatibility
measurements establish the recorded T01 observations, not the complete native
Linux production isolation gate. T02's offline build candidate and T03's
inactive contract/artifact validators and T04's internal capture/compiler
adapter are complete. T05's internal source admission and typed sidecars are
complete. T06's private CFG/lowering and artifact emission are complete;
`JAVA-03-T07` (registered candidates and JVM runner) is next.
Java activation belongs only to `JAVA-03-T10`.

T04 corrected lost Unicode characters in one recorded adapter fixture from
the archived T01 probe source. Measured positions, compiler behavior and all
profile/contract identities are unchanged; the vector manifest records the
corrected raw-file digest. See the implementation ledger for the erratum.

## 1. Scope, authority, and identities

MUST, MUST NOT, REQUIRED, and REJECT are normative requirements for the
implementation. REJECT publishes no partial VIR, map, manifest, VC,
certificate candidate, evidence, or AI request from that source context.

This profile accepts a deliberately small Java SE 25 subset: scalar methods
in field-free source interfaces, exact Boolean and signed integer values,
initialized locals, acyclic branches, and direct acyclic source calls.
Java source, sidecars, javac, the JDK/JVM, frontend, registry, VIR, VCs,
differential runs, and reports remain untrusted helper data. Certificate v0,
both source-free checker inputs and acceptance rules, and the existing four
axiom categories remain unchanged. No Java semantics axiom is introduced.

The successor mechanism in `SEMANTIC_PROFILE_REGISTRY_V1.md` defines common
schemas and hash encodings. This profile only supplies finite Java-owned
validators. It does not widen legacy Go/Rust APIs or reinterpret a C# contract.

| Role | Exact ID |
| --- | --- |
| source language | `java` |
| semantic profile | `mpk.java.scalar.v0` |
| parameter schema | `mpk.semantic_parameters.java_scalar.v0` |
| selection schema | `mpk.selection.java_methods.v0` |
| sidecar schema | `mpk.java.contract.v0` |
| toolchain input schema | `mpk.java.toolchain_inputs.v0` |
| limits | `mpk.java.limits.v0` |
| operation profile | `mpk.java.operations.v0` |
| source-map profile | `mpk.java.source_map.v0` |
| frontend argument profile | `mpk.java.frontend_arguments.v0` |
| required-check profile | `mpk.java.required_checks.v0` |
| launcher | `mpk.java.jvm_launcher.v0` |
| environment | `mpk.java.frontend_environment.v0` |
| host | `mpk.host.linux-x86_64-gnu.java25.v0` |
| runtime layout | `mpk.runtime.linux-x86_64-gnu.java25.v0` |

The exact parameter envelope is:

```json
{"schema":"mpk.semantic_parameters.java_scalar.v0","value":{"annotation_processing":"none","encoding":"UTF-8","language_version":"25","preview":false,"release":"25","target_id":"linux-x64"}}
```

Unknown members, aliases, case variants, target variants, preview modes,
compiler defaults, and another profile/parameter pairing reject. Integer
widths are fixed independently of host pointer width.

The companion `vectors/java-profile-v0.json` is the exact member-level data
contract for this specification. Its `profile_identity`, `profile_contracts`,
`semantic_parameters`, `toolchain_inputs`, `compiler_session`, and
`launcher_contract` are closed values, not example configuration. A described
field cannot be extended while retaining its immutable ID. Runtime values are
not inferred from this document's prose or from the host installation.

## 2. Selection, names, and immutable capture

The selection envelope's `schema` is `mpk.selection.java_methods.v0`; `value`
has exactly these four fields:

| Field | Rule |
| --- | --- |
| `compilation` | 1..64 ASCII bytes; `[a-z][a-z0-9]*([._-][a-z0-9]+)*` |
| `sources` | 1..256 strictly increasing unique portable paths below `src/`, ending `.java` |
| `contracts` | 1..128 strictly increasing unique portable paths below `contracts/`, ending `.json` |
| `methods` | 1..32 strictly increasing unique canonical method IDs, 1..1,024 ASCII bytes each |

Sorting uses unsigned UTF-8 byte order. A portable path has nonempty slash-
separated ASCII components, no root prefix, backslash, colon, dot/dot-dot
component, trailing separator, or normalization alternative; common successor
path restrictions also apply. Each source is exactly
`src/<package segments>/<Interface>.java`, case-sensitive, and declares that
one public top-level interface. Packages contain at least one segment. The
exact names `java`, `javax`, `jdk`, `sun`, `com.sun`, and every namespace below
one of those prefixes reject; lookalikes such as `com.sunny` do not match a
forbidden prefix.

A method ID is `package.Interface::method(T1,T2)->R`, without whitespace.
Type tokens are exactly `boolean`, `int`, and `long`; zero arguments use `()`.
The return is exactly one scalar. Method IDs are resolved against declaration
symbols and exact signatures, never by string matching alone.

All source package/type/method/parameter/local identifiers use
`[A-Za-z_][A-Za-z0-9_]*`, excluding `_`, `$`, non-ASCII spelling, escapes,
identifier-ignorable characters, and every word in this closed exclusion set:

```text
abstract assert boolean break byte case catch char class const continue
 default do double else enum extends final finally float for goto if
 implements import instanceof int interface long native new package private
 protected public return short static strictfp super switch synchronized this
 throw throws transient try void volatile while _ true false null
 exports module non-sealed open opens permits provides record requires sealed
 to transitive uses var when with yield
```

Whitespace in the table only separates words. Primitive keywords and Boolean
literals remain admitted in their type/literal positions, not as identifiers.
Rejecting contextual words in every identifier position is an intentional
restriction beyond Java's position-dependent grammar. Raw spellings must be
checked before compiler identifier normalization.

Capture only the selected regular files and implied directories through the
shared no-follow immutable snapshot boundary. Reject missing or unlisted
entries, symlinks, hard-link aliases, special files, traversal, duplicate or
ASCII case-fold-colliding paths, and byte changes during capture. There are no
globs, build-file discovery, default contract paths, or implicit dependencies.
Sidecar file names need not equal method names; the closed `method` field
performs attachment. Source and sidecar raw-byte hashes remain separate.

## 3. Declarations, initialization, and closure

Each compilation unit has a named package, no imports/module declaration or
package annotation, and exactly one explicitly `public` top-level interface.
It has no fields, declared superinterfaces, type parameters, annotation, nested type,
initializer, or other member except accepted methods. Each method explicitly
writes both `public` and `static` exactly once, in either legal modifier order,
and no other modifier. It has scalar keyword parameter and result types, an
ordinary body, and no type parameters, receiver parameter, `throws`, default
value, annotation, varargs, or array dimensions. Parameters have unique names
and no modifiers. Count JVM descriptor parameter units explicitly: `boolean`
and `int` each use one, `long` uses two, and a static method has no implicit
receiver unit. The maximum is 255. The measured `analyze()` accepts a
256-unit signature without a diagnostic, so compiler success does not
replace this profile check; reject it as `JAVA_LIMIT_PARAMETER_SLOTS` before
lowering. Empty interfaces and unselected helper declarations reject.

Raw pre-attribution modifier records must prove explicitly written modifiers;
attribution's implicit flags cannot satisfy that requirement. Every method
name occurs once per interface: overload declarations reject even when javac
can resolve them. Class, record, enum, and annotation declarations reject.

Calling one of these static interface methods can initialize only its inert
field-free declaring interface. The accepted source contains no initialization
actions or inherited initialization chain. The public type model may report `java.lang.Object` as a general type-system
supertype; that is not a source `extends` declaration or an interface
initialization dependency and does not authorize an Object method call. See
[Types.directSupertypes](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/lang/model/util/Types.html#directSupertypes(javax.lang.model.type.TypeMirror)). The
frontend analyzes source and MUST NOT generate or load selected classes. The proof models ordinary scalar
execution; it does not promise immunity from environmental JVM linkage,
resource, or operating-system failures. See
[JLS 12.4](https://docs.oracle.com/javase/specs/jls/se25/html/jls-12.html#jls-12.4)
and [JLS interface methods](https://docs.oracle.com/javase/specs/jls/se25/html/jls-9.html#jls-9.4).

Analyze all captured sources as one compilation. Starting from every selected
method, traverse calls in all syntactic branches, including a constant-false
branch and the unexecuted arm of a constant conditional. Every declared
method must occur in this conservative closure; each closure method has one
and only one selected sidecar. Missing, duplicate, or unused sidecars reject.
No source member or selected file may be silently ignored. Cycles, including
self-recursion, reject before lowering.

A call is either an unqualified same-interface method name or a fully package-
qualified source interface method. Its `Trees.getElement(TreePath)` result
must be the exact captured `ExecutableElement`, declared static in the
accepted shape. Argument count and each source argument type match exactly;
there is no implicit invocation widening. Expression receivers, partially
qualified/imported names, inherited, external/library, virtual, native,
constructor, generic, varargs, lambda, reference, and dynamic calls reject.
Methods are emitted callee-first, with the smallest canonical method ID
chosen whenever several methods are ready.

## 4. Statements, expressions, and scalar types

The entire statement set is: block; one explicitly typed, initialized local
per declaration statement; a simple assignment statement to an existing local;
`if` with optional `else`; and value return. Every syntactic complete path
returns. Statements after a terminating statement reject. Both arms of a
constant condition remain validated and represented by the frozen branch
pattern. Parameter assignment, shadowing, repeated local names anywhere in a
method, `final`, `var`, uninitialized declarations, empty/labeled statements,
loops, switch, pattern matching, break/continue, assert, exception handling,
throw, synchronization, allocation, I/O, time, randomness, reflection and
host access reject.

Accepted expressions are the literals below, parameter/local reads,
parentheses, admitted unary/binary operators, casts, same-type conditionals,
and direct calls. An assignment cannot appear within an expression. A local
assignment is the only accepted expression statement. Every local read must
be dominated by initialization on every incoming CFG path, without inventing
a default value. Assignment changes only the named local.

| Java type | VIR carrier |
| --- | --- |
| `boolean` | `bool` |
| `int` | signed BV32 |
| `long` | signed BV64 |

No other scalar or aggregate source type is admitted. An internal unsigned
BV32/BV64 is permitted only inside the exact logical-right-shift pattern in
section 5; it cannot escape into declarations, contracts, calls or returns.

Integer source tokens are `0` or a nonzero decimal digit followed by decimal
digits, optionally followed by uppercase `L` for `long`. Radix prefixes,
underscores, leading zeroes, lowercase suffixes, character/string/text-block
literals, `null`, and unary plus reject. Boolean literals are `true`/`false`.
The parsed/attributed type must be exactly the expected primitive.

A minus applied directly to an integer literal admits every in-range negative
value, including `-2147483648` and `-9223372036854775808L`; the raw literal
spelling, pinned public-tree observation, and resolved type must agree. That
form emits one signed `Const`, with zero canonicalized to `0`. Parentheses
between the minus and magnitude do not admit an otherwise out-of-range
positive literal. Other unary negation emits `bv_neg`. The adapter does not
substitute javac constant values for arithmetic or conditional source trees.
Its exact permissible parse/attribution transformations are those recorded in
`compiler_session` and the public-API probe; absent or unexplained trees fail
closed.

Identity conversion emits no instruction. An implicit `int` to `long`
conversion is allowed only for local initialization, local assignment, or
return, and sign-extends through `Convert`. Explicit `(long)` on `int`
sign-extends; `(int)` on `long` retains the low 32 bits as signed. Identity
casts are admitted. Calls, binary operations, and `?:` operands already have
identical source types except shifts. No implicit constant narrowing,
boxing/unboxing, reference, user-defined, or range-checked conversion is
admitted. Mixed widths require an explicit admitted cast. See
[JLS conversions](https://docs.oracle.com/javase/specs/jls/se25/html/jls-5.html).

## 5. Operations and exact safety checks

Operands and arguments evaluate left-to-right exactly once. No pure-looking
expression or callee call may be reordered across another evaluation or a
short-circuit boundary.

| Source operation | VIR operation | Complete `safety_checks` |
| --- | --- | --- |
| Boolean `!` | `not` | `[]` |
| same-type scalar `==`, `!=` | `eq`, `not_eq` | `[]` |
| same-type integer `<`, `<=`, `>`, `>=` | `signed_lt/le/gt/ge` | `[]` |
| integer `+`, `-`, `*`, unary minus | wrapping `bv_add/sub/mul/neg` | `[]` |
| integer `~`, `&`, `\|`, `^` | `bv_not/and/or/xor` | `[]` |
| integer `/`, `%` | `bv_sdiv`, `bv_srem` | `[divisor_nonzero]` |
| integer `<<`, `>>`, `>>>` | closed masked patterns below | `[]` |
| Boolean `&&`, `\|\|`, same-type `?:` | conditional CFG | branch-local checks only |

All source binary integer operands have the identical signed width. Boolean
`&`, `|`, `^`, unsigned ordering/division/remainder, and every unlisted
operation reject. `Const`, `Copy`, `Convert`, and `CallStatic` have no
`safety_checks`; call obligations are generated by the existing call WP
mechanism, not encoded as operation checks.

Division/remainder truncates toward zero. For either width, `MIN / -1` is
`MIN` and `MIN % -1` is zero. Existing total VIR BV equations represent those
values. Zero divisor requires a normal-completion VC. The validator rejects
missing, duplicate, extra, or reordered checks; in particular,
`integer_no_overflow` and `signed_divrem_representable` are invalid for every
Java source operation. No C# check rule is inherited. See
[JLS division/remainder](https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html#jls-15.17.2).

The RHS of every shift is exactly source `int`, even when the LHS is `long`.
After both operands have evaluated, emit signed BV32 mask constant `31` for
an `int` LHS or `63` for a `long` LHS; emit signed BV32 `bv_and` of the RHS
and that constant; pass that exact result to the shift. Negative and oversized
counts are accepted. `<<` emits `bv_shl`; `>>` emits `bv_ashr`. For `>>>`,
convert the LHS to an unsigned carrier of identical width, emit `bv_lshr`
with the masked signed BV32 count, and convert the result back to signed at
the same width. Both conversions preserve every bit. The unsigned intermediate
has exactly the uses required by this pattern and cannot be reused outside
it. A correct mask elsewhere in the function does not validate an unlinked
or unmasked shift. Mask operands may not swap or change width. These generated
instructions have empty checks and the owning shift expression's origin.
See [JLS shifts](https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html#jls-15.19).

The 34-row `semantic_rows` vector admits exactly `M01`, `M02`, `M07` through
`M13`, `M16`, `M18`, `M19`, `M21`, `M27`, `M29`, `M33`, and `M34` under
these restrictions. The other 17 rows reject before VIR. In particular,
`M14` trapping primitive arithmetic is excluded, unlike the C# profile.
The historical matrix's `F` categories remain rejected; none becomes a new
foundation in this task.

## 6. Public compiler API and deterministic CFG

The only compiler entry is the pinned JDK `JavaCompiler.getTask`, yielding a
`JavacTask`, followed by `parse()` and `analyze()`. Use exported `Trees`,
`SourcePositions`, `Elements`, `Types`, public `com.sun.source.tree` interfaces,
and `javax.lang.model` interfaces. `generate()`, `CompilationTask.call()`,
private javac packages, reflection into compiler internals, plugins,
`--add-exports`, and `--add-opens` are forbidden. There is no public javac CFG
API; MPK builds its own graph from the bounded public tree inventory. See
[JavacTask](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/JavacTask.html).

Create a new task and file manager per request, with UTF-8 decoding and
`Locale.US`. Source objects are supplied in selection order and carry only
logical-path-derived internal URIs. Options are exactly `--release 25`,
`-encoding UTF-8`, `-proc:none`, `-implicit:none`, `-Xlint:none`,
`-Xmaxerrs 1025`, and `-Xmaxwarns 1025`, in the vector's frozen order.
No source/target/system override, processor, user option, response file,
preview switch, ambient project configuration, or source generation is used.
Parse completes and its diagnostics are handled before attribution starts.

The admitted public `Tree.Kind` inventory is closed:

```text
COMPILATION_UNIT PACKAGE INTERFACE MODIFIERS METHOD VARIABLE PRIMITIVE_TYPE
BLOCK EXPRESSION_STATEMENT ASSIGNMENT IF RETURN IDENTIFIER MEMBER_SELECT
PARENTHESIZED TYPE_CAST CONDITIONAL_EXPRESSION METHOD_INVOCATION
BOOLEAN_LITERAL INT_LITERAL LONG_LITERAL UNARY_MINUS LOGICAL_COMPLEMENT
BITWISE_COMPLEMENT MULTIPLY DIVIDE REMAINDER PLUS MINUS LEFT_SHIFT RIGHT_SHIFT
UNSIGNED_RIGHT_SHIFT LESS_THAN GREATER_THAN LESS_THAN_EQUAL GREATER_THAN_EQUAL
EQUAL_TO NOT_EQUAL_TO AND XOR OR CONDITIONAL_AND CONDITIONAL_OR
```

A listed kind is allowed only in the exact source position defined above.
`MEMBER_SELECT` is package spelling or a fully qualified static target;
`ASSIGNMENT` is the sole child expression of a local-assignment statement;
`VARIABLE` is a parameter or one initialized scalar declaration. Check raw
statement boundaries so javac's separate `VariableTree` nodes cannot hide a
multi-declarator source statement. Ordinary known but excluded Java kinds
produce subset rejection; an unrecognized adapter enum/state fails closed.
Empty modifier nodes can have `NOPOS` because they emit no operation; they
cannot serve as an artifact origin.

Snapshot public tree kind, relevant source children/order, spans, raw
identifiers/modifiers/literals before attribution without admitting source
forms. Attribution and its diagnostic normalization complete first: a known
source diagnostic, unknown diagnostic/provenance, compiler exception or
resource failure keeps its earlier status even when the source also has an
excluded declaration.

After diagnostic-free attribution, perform the raw source shape/admission
gates in section 9 before comparing accepted-subtree compiler transformations.
At each parent, reject a known excluded declaration/type/literal/statement/
identifier form before traversing its descendants as candidate accepted AST.
For example, a valid class is `JAVA_SUBSET_DECLARATION`, even though javac
adds an implicit constructor, expression statement and constructor invocation
with missing end positions. Those generated children are not an allowed
constructor pattern and are never interpreted, lowered or origin-mapped.
A raw `var` declaration similarly rejects as `JAVA_SUBSET_TYPE` before
requiring an explicit pre-analysis type child: the probe finds no such child
before attribution, then an inferred `PRIMITIVE_TYPE` with an absent end
position afterward. That type insertion is not an accepted conversion or
source-origin pattern. The same parent-first ordering applies to record/enum
parents, raw disallowed identifiers, and other known excluded source forms.
It does not bypass attribution diagnostics or resource accounting.

Only subtrees that survive those source gates must match the frozen
pre/post-analysis observations. Resolve their primitive types and symbols
from public APIs and the closed source rules. Unknown kinds, erroneous trees,
missing/error types, unresolved elements, synthesized declarations, or
unexplained changes within candidate accepted subtrees are adapter errors.
All syntactic branches of an admitted parent, including source-dead branches,
undergo the same source gate and transformation checks before lowering; a
constant condition never erases unsupported source contents.

The closed application file manager exposes captured source objects only.
With `--release 25`, javac obtains its platform module/reference view through
a separate internal platform manager that the public forwarding wrapper does
not intercept. The probe observes zero wrapper system-file returns even for
resolved `java.lang` and `java.nio.file` types. The complete pinned JDK input
inventory, exact options, runtime image and operating-system filesystem
boundary close that internal platform view; application wrapper coverage is
not claimed as evidence of interception. No compiler internal API is called
to replace that platform manager. It explicitly closes application class/source,
module, upgrade, patch, and processor locations. An empty path collection or
controlled empty directory is permitted; an empty-string path component is
not. Audit `list`, input-file lookup, `contains`, binary/module-name inference,
module-location lookup/enumeration, `hasLocation`, and classloader/service-loader
access. Unknown locations produce the API-defined absent result or a bounded
adapter failure and never delegate to host defaults. All output methods refuse
writes. Platform system lookups are confined to inventoried JDK bytes by that separate
closure; the frontend JAR
is not an analyzed dependency. `-implicit:none` is not a discovery boundary.
The probe must demonstrate that planted unselected sources, classes, modules,
processor services, and ambient paths are not consumed. See
[JavaFileManager](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/JavaFileManager.html).

The CFG has one entry, no back-edge or exceptional region, and only explicit
`Branch`, `Jump`, and value `Return` terminators. A statement list continues
in the current block until a terminator. Each `if` creates its false branch
before true for traversal purposes and creates a join only when an arm can
continue; no join is emitted when both arms return. A continuing arm jumps to
that join. An `if` without `else` has an empty false arm. A constant condition
uses the same shape and preserves both syntactic arms.

For `&&`, the false arm supplies `false`; the true arm evaluates the RHS.
For `||`, the false arm evaluates the RHS; the true arm supplies `true`.
For `?:`, each arm evaluates only its own value. Each continuing arm jumps
with its result into one join block parameter of the exact result type. The
result of that expression is the join parameter. Nested expression graphs
follow the same rule; no RHS instruction or safety check is hoisted. Source
local assignments use `Copy`; shared local dominance rules apply at joins.
There are no compiler-generated source locals.

Assign `argN` by parameter order, `localN` by source declaration byte position,
and exactly one result `result0`. After graph construction, traverse blocks
breadth-first from entry, false edge before true, and assign `bbN`. Number
block parameters densely as `pN` by canonical block/parameter order and all
instruction values densely as `tN` by canonical block/instruction order.
Reference rewriting occurs after numbering. Object addresses, compiler tree
numbers and hash-map order never enter artifacts. The `cfg_patterns` vector
fixes canonical representative branch/join forms without defining a new VIR.
These are symbolic golden fragments: type tokens map through `type_mappings`,
string references map to VIR variable atoms, source origins come from the
method/body expressions above, and each `CallStatic` inserts the recomputed
callee contract hash. They are not independently accepted VIR documents.

## 7. Contract sidecars and normalization

Each closure method has exactly one selected strict JSON sidecar with these
members and no others:

| Member | Exact rule |
| --- | --- |
| `schema` | `mpk.java.contract.v0` |
| `semantic_profile` | `mpk.java.scalar.v0` |
| `method` | resolved canonical method ID |
| `requires` | ordered Boolean clauses, may be empty |
| `ensures` | ordered Boolean clauses, nonempty |
| `modifies` | `[]` |
| `abrupt_completion` | `forbidden` |
| `termination` | `total` |

Combined requires/ensures count is 1..64. Comments, annotations, Javadoc,
processor output, arbitrary source expressions and inferred contracts are not
sidecars. Reject duplicate JSON keys at every level, unknown/missing members,
null, malformed Unicode, invalid numeric encodings, and all shared strict
JSON violations before expression interpretation.

| Expression branch | Exact members |
| --- | --- |
| parameter | `{"parameter":Name}` |
| result, ensures only | `{"result":0}` |
| Boolean | `{"bool":Boolean}` |
| integer | `{"int":{"decimal":CanonicalDecimal,"type":"i32" or "i64"}}` |
| unary | `{"op":Op,"args":[Expr]}` |
| Boolean n-ary | `{"op":"and" or "or","args":[Expr,...]}`, 2..64 operands |
| binary | `{"op":Op,"args":[Expr,Expr]}` |

Parameter names resolve against that method only. Locals are never contract-
visible. `result` is forbidden throughout `requires`, including nested
expressions; in `ensures` it has the exact method result type. Integer decimal
is `0` or optional minus followed by a nonzero digit and decimal digits, in
the signed type's range; no plus, leading zero, negative zero, whitespace,
suffix, separator or radix prefix is allowed.

Unary operators are `not` on Boolean, and `bv_neg`/`bv_not` on signed integer.
`and`/`or` require Boolean operands. Binary `eq`/`not_eq` require identical
accepted scalar types; `signed_lt/le/gt/ge` and `bv_add/sub/mul/and/or/xor`
require identical integer types. Each slash family names the individual VIR
operator values. No conversions, shifts, division, remainder, unsigned type,
field, call, source operator spelling or user extension is allowed.

Normalization renames parameter positions to `argN`, retains result/Boolean
atoms, maps integer literals to the shared VIR integer atom, unary `args[0]`
to `value`, and binary operands to `lhs`/`rhs`. It does not fold, commute,
reassociate, reorder, deduplicate, or insert a conversion. The normalized
contract carries complete Java `SemanticContext`, selected compilation as
`unit_id`, sidecar method as `function_id`, `panic=forbidden`,
`termination=total`, `loops=[]`, and the common `MPK-CONTRACT-1.0` hash.
Raw sidecar bytes remain separate manifest inputs.

Every method body must satisfy its own contract. At a call, prove the callee's
ordered preconditions in the calling path state, then use its exact
postcondition through the existing WP rule and its checked normal-completion
dependency. A compiler success or matching differential evaluation is not a
verification verdict. Final self-contained Certificate v0 bytes must be
accepted identically by both source-free checkers.

## 8. Source transport, origins, and artifact binding

Require nonempty strict UTF-8, no BOM, NUL, CR, encoded surrogate, or Unicode
noncharacter, and final LF. Noncharacters include U+FDD0..U+FDEF and the final
two code points of every plane. Unicode comments are allowed; identifiers
remain ASCII. Reject every raw ASCII backslash immediately followed by `u`,
including in comments and escaped-looking pairs. Do not preprocess/rewrite
source. This intentionally excludes Java Unicode-escape translation. See
[JLS lexical translation](https://docs.oracle.com/javase/specs/jls/se25/html/jls-3.html#jls-3.3).

Decode exactly once and build a checked UTF-16-boundary-to-UTF-8-byte table
for each original source. Public `SourcePositions` supply start/end offsets;
negative, out-of-range, absent, split-surrogate, reversed, and zero-length
emitted origins fail closed. Public origins contain only captured logical
path and UTF-8 byte range. Compiler URI prefixes, absolute host paths,
line/column values and nearest-token substitutions never enter artifacts.
`LineMap.getColumnNumber` expands tabs and MUST NOT compute byte offsets.

Map every function, instruction and terminator. Function origin is the method
declaration; a local `Copy` owns its declaration or assignment statement;
return owns the `return` statement; branch/join operations generated from an
expression own that expression. A continuing `if`-arm jump owns the `if`
statement. All masks/conversion helpers own their original shift/cast or
conversion-bearing source expression. Synthetic-origin allowlist is empty.
The vector's ASCII/BMP/non-BMP/tab/invalid-boundary cases are normative.

Emit one compilation unit, empty aggregate/constant declaration inventories,
the full conservative closure, exact contracts, and a complete Java context.
The manifest input kinds are exactly `source` and `contract`, with `.java`
source extension. Frontend manifests have no VC hash. Finalization may change
only the fields authorized by the successor manifest contract. Registry,
entry, selection, source, contract, compiler/toolchain, frontend/bundle, map,
VIR, VC and evidence identities are cross-validated throughout. No public
artifact may be relabeled as another language to reuse a private projection.

## 9. Phases, diagnostics, and deterministic failure

Shared frontend protocol status/exit rules apply through the successor
semantic context: `ir-lowered`/0, `rejected`/3, `source-error`/4,
`frontend-error`/1. Caller configuration failure is exit 2 with no child JSON.
Release failure is caller-local and starts no compiler. The child phases are
`capture`, `source`, `metadata`, `typecheck`, `subset`, `lowering`, `emission`.
Metadata validates compiler/options/reference/file-manager construction;
typecheck performs attribution. Source transport and parsing precede both.

Earlier phases own semantic failures. Within subset, check declarations and
names, types/literals/control/operations, initialization/purity/calls/closure,
then sidecar JSON/shape/attachment/type rules, in source order. The raw
source gates run before accepted-subtree post-attribution comparison; a known
excluded parent rejects before its children, including compiler-synthesized
children, are interpreted as supported constructs. Thus an ordinary valid
class with a synthesized constructor remains a subset rejection, while a
class with an attribution diagnostic retains the earlier diagnostic outcome.
Operational/invariant failure always overrides semantic results from work
that actually started but did not complete; an inapplicable accepted-subtree
check is not run inside an already rejected source parent. A deterministic limit rejects before the
first excess item is retained; a timeout or killed process is a frontend
resource failure, never proof of unsupported Java syntax.

`diagnostic_registry` is the exhaustive list of public `JAVA_*` codes,
statuses, phases, fixed messages and exits. No unlisted suffix is allowed.
The groups and their responsibilities are:

| Prefix | Responsibility |
| --- | --- |
| `JAVA_CAPTURE_` | selected path, file type and inventory |
| `JAVA_SOURCE_` | strict transport, parse, attributed source errors |
| `JAVA_TOOLCHAIN_` | pinned archive/runtime/compiler/reference/options/file-manager/API failures |
| `JAVA_SUBSET_` | declaration, identifier, type, literal, flow, operation, conversion, call, initialization, purity, abrupt completion |
| `JAVA_CONTRACT_` | JSON, shape, identity, duplicate, missing, unused, type, operator, hash |
| `JAVA_LOWERING_` | operation, CFG, missing/extra/order checks, shift pattern |
| `JAVA_SOURCE_MAP_` | external source, range and UTF-16 failures |
| `JAVA_FRONTEND_` | output/diagnostic/resource budgets and internal failure |
| `JAVA_LIMIT_` | exact logical counter IDs in section 10 |

Compiler diagnostics use only the exact observed code/kind combinations in
`diagnostic_normalization.compiler_code_allowlist` and
`compiler_kind_allowlist`, supported by `adapter_observations`. The six exact
codes are `compiler.err.cant.resolve.location`, `compiler.err.doesnt.exist`,
`compiler.err.int.number.too.large`, `compiler.err.premature.eof`,
`compiler.err.prob.found.req`, and
`compiler.err.var.might.not.have.been.initialized`, each with kind `ERROR`. Error, warning and mandatory-warning
callbacks reject as `source-error`; notes are admitted only by the exact note
allowlist, which is empty. Unknown codes/kinds, unknown or external source
provenance, truncation, and unexplained API states are adapter errors. The
listener stores no compiler prose. Each callback counts before normalization,
even if messages would later coincide.

A diagnostic source must be the exact captured `JavaFileObject`. Both offsets
must be valid UTF-16 boundaries; zero-length diagnostics may omit the optional
span after boundary validation, but emitted artifact origins never may.
An unavailable/NOPOS position is an adapter error, not a fabricated range.
Sort raw issues by logical path, byte start/end, javac code, kind rank, then
arrival ordinal; kind rank is ERROR, WARNING, MANDATORY_WARNING, NOTE, OTHER.
Sort public Issues by the shared key path/start/code/message/function/end and
its absent-field sentinels. Exact messages depend only on the registry:
`Java source is invalid`, `Java source is outside the frozen profile`,
`Java frontend failed closed`, or `Java profile limit exceeded`. No snippet,
compiler prose, absolute path, environment, identifier or stack trace is
interpolated. The parent normalizes nonresponding child/resource failures;
non-success never publishes partial artifacts.

## 10. Logical resource limits

Every limit is inclusive. A lower shared successor limit wins. Counters use
checked arithmetic before allocation or retention of the first excess item.
`limit_cases` fixes both exact-boundary counter acceptance and plus-one
failure; this does not promise every maximum fits simultaneously or within
hard operating-system resource limits.

| Limit | Maximum |
| --- | ---: |
| source files / bytes each / total bytes | 256 / 1,048,576 / 16,777,216 |
| contract files / bytes each / total bytes | 128 / 1,048,576 / 8,388,608 |
| snapshot entries / total file bytes | 512 / 33,554,432 |
| path / method ID bytes | 1,024 / 1,024 |
| selected / closure methods | 32 / 128 |
| descriptor parameter units per static method | 255 |
| syntax nodes / depth | 250,000 / 256 |
| instructions per method / closure | 100,000 / 250,000 |
| blocks per method / closure | 1,024 / 8,192 |
| contract clauses per method | 64 |
| contract nodes per method / closure / depth | 1,024 / 8,192 / 32 |
| diagnostic callbacks / message bytes each / total bytes | 1,024 / 4,096 / 2,097,152 |
| expanded argv bytes including NUL | 131,072 |
| frontend stdout including LF / stderr | 268,435,456 / 2,097,152 |
| canonical VIR / map / manifest bytes | 201,326,592 / 33,554,432 / 4,194,304 |

Source/sidecar byte totals use raw captured bytes. Snapshot entries include
files and distinct nonempty parent directories, excluding the root; directory
bytes count zero. Syntax counts each source public tree node once before
attribution, including structural nodes; the post-analysis inventory is
separately bounded by the same limits. Counting a compiler-generated node does
not admit it or require it to have a source origin. Transformation comparison
applies only after the raw source gates, to candidate accepted subtrees as
specified in section 6; the counter cannot turn a known excluded class into
an adapter failure merely because it has an implicit constructor. A
compilation-unit root and a contract clause root each have depth one.
Traversal checks depth iteratively before recursive lowering.

Instructions include `Const`, masks, casts, copies and branch expression
helpers. Closure totals sum checked per-method values. Contract nodes count
every expression object, without deduplication. Diagnostic callbacks and
canonical message bytes count before public sorting. Argument bytes sum each
UTF-8 argument plus its terminating NUL. Canonical artifact limits count JCS
bytes, without transport LF unless explicitly stated.

Diagnostic counters fail with `JAVA_FRONTEND_DIAGNOSTIC_BUDGET`; stream
counters fail with `JAVA_FRONTEND_OUTPUT_LIMIT`. Other plus-one counters use
the corresponding closed `JAVA_LIMIT_*` code. Source node limits apply also
to post-attribution validation but report in the deepest started protocol
phase that permits profile rejection. Hard memory/native/PID/time/tmpfs limits
are numerical requirements in `launcher_contract`, measured separately from
logical counter bounds. javac error/warning ceilings are 1,025; the listener
must abort before retaining its 1,025th all-kind callback. The public-API
probe records the exact abort behavior and prevents silent compiler
truncation from being interpreted as success.

## 11. Pinned build and JVM execution closure

`toolchain_inputs` freezes Eclipse Temurin `25.0.4.1+1` Linux x64 HotSpot JDK,
archive `OpenJDK25U-jdk_x64_linux_hotspot_25.0.4.1_1.tar.gz` (141,329,719
bytes), archive SHA-256
`dbb698396d478e7fa2b1e50f4103324b2a99b90569ee27c33f2261f9215cf41e`,
and the exact descriptor self-hash in
`toolchain_inputs.toolchain_inputs_sha256`. That vector field is the sole
source of the descriptor digest; it covers the complete inventory and ELF
metadata. The descriptor records the exact archive URL,
length, SHA-256, safe extraction/inventory policy, release metadata, complete
reference/runtime/native inventories, file modes, redistribution notices and
canonical self-hash. The archive is verified against the publisher checksum
and independently recorded digest. No placeholder digest, version range,
latest tag, host JDK, dependency manager or runtime download is accepted.
Inventory actual modules and `ct.sym`; do not assume a `jmods` directory.
Safe extraction must restore the exact recorded modes of regular files and
directories; a library extraction filter that changes `0444` into `0644` is
not sufficient. The descriptor retains archive symlink modes as source data,
but extracted symlinks are checked by exact type and target rather than host-
reported permission bits, which differ between macOS and Linux. The closed
archive policy admits only the inventoried links; this does not relax the
source snapshot's no-link rule.

T02 builds checked-in Java frontend sources with this exact JDK, without
Maven/Gradle or external libraries. It freezes source inventory, javac build
arguments, class inventory and deterministic JAR bytes (entry order, timestamp,
manifest). No `Class-Path` or service-provider JAR entry is allowed. Two
isolated offline builds must produce identical JAR and descriptor bytes.
Downloads belong only to explicit provisioning, never validation/build gates.

`launcher_contract` fixes the exact `/mpk/toolchain/jdk/bin/java` executable,
`/mpk/frontend/java2vir.jar` classpath, `mpk.java2vir.Main`, option array, module
graph, native libraries and environment. `ToolProvider.getSystemJavaCompiler`
must return the pinned compiler. No subordinate javac process runs. The
baseline is interpreter-only, CDS off, Serial GC, one reported processor,
attach/performance facilities off, explicit heap/stack/resource ceilings,
UTF-8/English/UTC, private bounded temporary/error paths. A failing baseline
cannot fall back to JIT or weaker sandbox settings.

The environment is built from the vector's allowlist, including fixed
`HOME=/mpk/empty-home`, `TMPDIR=/mpk/tmp`, `PATH=/nonexistent`, `LANG=C.UTF-8`,
`LC_ALL=C.UTF-8`, `TZ=UTC`. Never inherit `JAVA_HOME`, `CLASSPATH`,
`JAVA_TOOL_OPTIONS`, `JDK_JAVA_OPTIONS`, `JDK_JAVAC_OPTIONS`, `_JAVA_OPTIONS`,
`LD_PRELOAD`, or `LD_LIBRARY_PATH`. Credentials and host home directories are
not mounted.

The Java host/layout contract is distinct from the .NET-specific contract.
It requires x86-64 Linux, glibc 2.36, minimum kernel ABI 6.4.0, a 16 GiB
address-space limit, 1 GiB cgroup memory limit, zero cgroup swap, 128 PIDs,
1,024 open files, zero core bytes, 64 MiB private `noswap` tmpfs, and a
120-second request timeout. These are exact frozen requirements, not claims
that the disposable T01 probe exercised every enforcement/failure path.
T07 implements and tests the installed runner; T09/T10 own the complete
native Linux release enforcement gates.
T01 freezes the measured ELF interpreter, complete native linkage, libc ABI,
required files/devices/proc view, permissions and numerical budgets. T07 must
measure and validate native clone/thread behavior, syscall policy and all
installed enforcement/failure paths; T01 does not guess those constraints or
claim they were measured under native execution. The runner must require
read-only toolchain/source mounts, bounded private `noswap` tmpfs, cgroup
memory/PID enforcement, privilege drop, denied network, no writable executable
inputs and descendant cleanup. A translated/emulated compiler run
can establish public API observations only; it cannot establish an unmeasured
native Linux isolation contract. If the common successor descriptor cannot
express the measured requirements, stop and review the affected schema before
production implementation. Existing immutable host IDs cannot be broadened.

## 12. Registry, payloads, hashes, and consumers

`semantic-profile-registry-v3.json` preserves all three revision-2 entry values
byte-for-byte, inserts exactly one Java entry, sorts by language/profile tuple,
and freezes the predecessor/root/hash cases. The Java entry hash is
`0d80d13f97c45557fa9978eccc2545ffdb3fc1b93a26856b365a9be200470301`;
the revision-3 registry root is
`fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557`.
Common entry/root domains remain unchanged. Registry membership alone never
activates Java.

Java owns exactly nine IDs `mpk.profile.<field>.java_scalar.v0`, for `ai`,
`evidence`, `frontend`, `manifest`, `policy`, `release`, `source_map`, `vc`,
and `vir`. Their exact closed envelopes and canonical envelope sizes are
`profile_contracts`; the manifest's raw vector digest pins the complete
payload bytes. There is no invented per-payload hash domain or additional
payload-hash member. No implementation may guess additional payload fields.

| Field | Responsibility |
| --- | --- |
| `ai` | `mpk.java.ai_projection.v0`, label `Java`, `minimal-v1`, no source or proof authority |
| `evidence` | `mpk.java.evidence_recipe.v0`, both checkers, certificate-only authority |
| `frontend` | fixed argument/environment/limit/JVM contracts, no private driver |
| `manifest` | compilation unit, `.java`, exactly contract/source inputs |
| `policy` | `payment-policy-java-alpha`, `mvp-strict`, `mvp-theory` |
| `release` | exact toolchain self-hash, compiler/runtime/native/host/layout closure |
| `source_map` | UTF-8 source and byte offsets, no synthetic origins |
| `vc` | typed sidecar/check contracts and shared verification limits |
| `vir` | signed scalar operation/type mapping and shared VIR limits |

| Object | Hash domain |
| --- | --- |
| toolchain-input descriptor | `MPK-JAVA-TOOLCHAIN-INPUTS-0.1` |
| selection envelope | `MPK-JAVA-SELECTION-0.1` |
| canonical sidecar | `MPK-JAVA-CONTRACT-SIDECAR-0.1` |
| normalized contract | existing `MPK-CONTRACT-1.0` |

The vector self-hash rules specify excluded self-hash members before JCS
hashing. The manifest freezes raw vector-file SHA-256 separately. No hash
comparison accepts uppercase, a URI, a checksum-file spelling, or an alias.

All CLI policy/verify/evidence/explain and API routes use the same validated
selection and complete Java context. No Java-only verifier, raw executable,
`--java-home`, classpath option, registry override, runtime toggle or staging
public API is introduced. A recognized entry missing any of the nine compiled
contracts is an invalid release. Source, sidecar text, compiler diagnostics
and host details never enter provider requests. API context mismatch rejects
before state mutation.

## 13. Vector ownership, upgrades, and activation gate

`vectors/java-profile-v0.json` has schema `mpk.java.profile.conformance.v0`;
`vectors/semantic-profile-registry-v3.json` has schema
`mpk.semantic_profile.registry.conformance.v3`. T01's specification owner is
`crates/mpk-vc/tests/java_profile_spec.rs`, a test-only model with no production
Java dispatch. The implementation traceability ledger assigns every vector
family and normative requirement to exactly one primary implementation task
and concrete test file. A model/hash test does not claim Java source execution
or operating-system isolation.

Accepted source cases contain exact source bytes, selected entrypoints, all
closure sidecars, ordered required operation projections and exhaustive safety
check projections. Projections are required subsequences of full canonical
lowering, not permission to omit or accept arbitrary intervening operations.
Evaluation cases use decimal strings for integers and JSON Booleans; they are
independent semantic expectations, not proof evidence. Rejection mutations
apply to their named baseline; a planted external dependency must remain
unconsumed, and only a lookup that tries to escape the closed manager fails.
Numeric boundaries test the exact counter maximum and plus one. Production
owners must execute complete VIR/maps/manifests as well as these projections.

A compiler/runtime/reference/native/option/API/diagnostic change requires exact
before/after inventories, observed-behavior diff, source/operation/contract/
map/hash mutation corpus, differential comparison, two offline builds and
runs, complete installed release gates, unchanged axiom categories, and
zero-finding review. Changed accepted values or operation/adapter/contract
meaning requires new immutable IDs and registry admission. A byte-only
replacement still requires new pinned hashes and full upgrade evidence.
There is no in-place update, floating resolver, compatibility fallback or
partial-language rollback.

T10 alone atomically replaces the installed image: retain the existing four
Go/Rust/C# tuples, add Linux-x64 Java, migrate every context-bound producer and
consumer to revision 3, and reject revision-2 helpers under that installation.
Old entry bytes and language semantics remain unchanged; artifacts embedding
the registry root necessarily change hashes. Rollback replaces the whole
installed image, with no mixed registry/bundle generations. Certificate v0
remains source-free and requires no registry migration.

Before activation, all owning test executors, deterministic/differential/fuzz/
upgrade corpus, same-byte dual-checker examples, hostile environment tests,
measured native Linux sandbox, and composed four-language installed release
gate must pass. Ordinary checks use `./scripts/check-fast.sh`; release checks
use the README's local Linux gates. No GitHub Actions or workflow file is
created, run, monitored or relied on. This specification task adds no active
Java parser, frontend executable, installed tuple, policy route or proof rule.
