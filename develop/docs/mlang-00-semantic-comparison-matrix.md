# MLANG-00-T01 Semantic Comparison Matrix

Status: complete non-normative feasibility record for `MLANG-00-T01`.

Prepared: 2026-08-25.

## 1. Scope and authority

This record compares C#, Java, Dart, TypeScript, and Python against the
completed Go/Rust VIR path. It covers types, integers, floating point,
evaluation order, conversions, null-like values, heap behavior, exceptions,
calls, dispatch, initialization, concurrency, async behavior, and
target/runtime variability. It is design evidence under Gate B of
`06_multilanguage_frontend_design.md`; it is not a source-language
specification and does not activate a frontend.

The semantic authority used for the comparison is:

- `../specs/VIR_V0.md`, including its closed Bool/BV/array/struct carriers,
  total operations, six safety-check kinds, ordered CFG, and direct acyclic
  `CallStatic` model;
- `../specs/GO_VIR_PROFILE_V0.md` and `../specs/RUST_SUBSET_V0.md`, which show
  the two completed interpretations of those operations; and
- the official language references listed in section 9.

`mpk.vir.v0` remains closed over Go and Rust. An `E` or `C` result below says
that the operation's checked meaning is reusable in a future reviewed
successor profile. It does **not** permit a C#, Java, Dart, TypeScript, or
Python producer to emit the current schema, add a profile ID, or claim source
acceptance. No Certificate v0 encoding, checker input, hash domain, or axiom
category changes here.

## 2. Disposition contract

Every language cell in section 6 begins with exactly one of these four
dispositions:

| Code | Disposition | Exact meaning in this record |
| --- | --- | --- |
| `E` | existing VIR operation | The closed source behavior can use the named current VIR carrier, instruction, terminator, or composition without a new value foundation or source-specific runtime-safety predicate. Normal source resolution and fail-closed subset validation still apply. |
| `C` | required-check rule | The named VIR operation is exact only with every listed current safety predicate or deterministic profile-admission check. Missing, ambiguous, or unproved checks prevent acceptance. |
| `F` | proposed checked foundation | Current VIR cannot express the behavior exactly. The named foundation is only a gap label for later design; the operation remains blocked until a reviewed successor definition, encoding, checked implementation, and vectors exist. |
| `R` | explicit initial rejection | The behavior is outside the initial profile direction. It is not erased, approximated, or represented as an unconstrained value. |

Only one code may appear at the start of a cell. `F` and `R` are always
blocked. `E` and `C` are representability findings, not activated-language
acceptance. If a source form combines rows, all of its rows must be eligible;
one `F`, `R`, unknown behavior, or unresolved required parameter rejects the
whole form before VIR publication.

### 2.1 Required-check catalog

| Check ID | Kind | Required rule |
| --- | --- | --- |
| `C-OVERFLOW` | existing VIR safety checks | Emit the exact ordered `integer_no_overflow` check for each checked `add`, `sub`, `mul`, or signed `neg`. |
| `C-DIV` | existing VIR safety checks | Emit `divisor_nonzero`; where the source must exclude signed `MIN / -1` or `MIN % -1`, also emit the operation-qualified `signed_divrem_representable` check. |
| `C-CALL` | profile-admission and VC checks | Resolve one same-module, direct, non-generic, pure, one-result target; close the selected dependency graph; reject recursion/cycles; bind the exact callee contract hash; and discharge callee preconditions and panic-free dependencies. |
| `C-LOCAL` | profile-admission check | Prove source definite assignment and emit a dominating `Copy` for every local read on every incoming path. A source-observable uninitialized/default value is not inferred from VIR. |
| `C-PIN` | profile-admission check | Pin every semantic compiler/language/runtime/target/option/library input named for the language. Missing, unknown, preview, ambient, or conflicting inputs reject. |
| `C-CONTRACT` | profile-admission and VC checks | Use a frozen language-owned contract grammar, type rules, attachment rule, and pure closed operation set, then normalize it to `VirContract`. No annotation or comment is trusted merely because it parses. |

These profile-admission checks run in an untrusted frontend or validator and
are later independently constrained by the successor contract. They do not
add new Certificate v0 assumptions.

### 2.2 Proposed-foundation catalog

| Foundation ID | Gap, not an accepted design |
| --- | --- |
| `F-BINARY-FLOAT` | Checked binary32/binary64 carriers and operations, including rounding, NaN payload/canonicalization policy, infinities, signed zero, conversions, comparisons, and target reproducibility. |
| `F-CSHARP-DECIMAL` | Exact C# `decimal` carrier, scale/rounding, conversion, overflow, division, and remainder behavior. |
| `F-DART-INT` | One pinned Dart runtime target's exact `int` carrier, overflow, division/remainder, bitwise, shift, identity, and int/double interaction. |
| `F-ECMASCRIPT-NUMBER` | ECMAScript Number plus the exact coercion boundary needed by accepted TypeScript expressions, including NaN, infinities, signed zero, ToNumeric/ToInt32 effects, and host/runtime pinning. |
| `F-UNBOUNDED-INT` | Checked arbitrary-precision integer carrier and operations, including resource bounds and each language's division, remainder, shift, and conversion rules. |
| `F-CHECKED-CONVERT` | Range predicate and checked conversion semantics. Current `Convert` is total, keeps/truncates BV bits, is Go-profile-only, and carries an empty `safety_checks` array. |

Naming a foundation does not authorize a schema value, theory constant,
checker branch, implementation, or axiom. Later work may reject the proposal
instead.

## 3. Semantic facts by language

The facts below describe the language behavior that an initial profile must
either preserve or reject. “Candidate” means only that section 6 found an
exact representation under its stated restrictions.

### 3.1 C#

| Dimension | Recorded behavior and initial boundary |
| --- | --- |
| Types | `bool`; fixed signed/unsigned 8/16/32/64-bit integral value types; native-sized `nint`/`nuint`; `char`; binary floating types; `decimal`; nullable value types; and reference types. Only `bool` and an explicitly frozen fixed-integer set are carrier candidates. `nint`, `nuint`, references, nullable values, and aggregates initially reject. |
| Integer and float behavior | Fixed integers use two's complement. Checked `+`, `-`, `*`, and signed negation fail on overflow; unchecked forms discard high bits. Integer division truncates toward zero and fails on zero. Signed `MIN / -1` is checked-overflow and is implementation-defined in an unchecked context, so the candidate mapping excludes it with `C-DIV`. Shift counts use the low five or six bits. `float`/`double` and `decimal` require new foundations. |
| Evaluation order | Operands and invocation arguments evaluate left to right. `&&`, `||`, `?:`, and null-aware constructs conditionally evaluate operands. Ordered VIR instructions and branch graphs preserve the pure, non-throwing candidate cases. |
| Conversions | Built-in implicit/explicit numeric conversions depend on source/destination types and checked context; user-defined conversions can execute code. Exact or unchecked BV conversions can reuse `Convert`; checked range conversion needs `F-CHECKED-CONVERT`; user-defined, reference, boxing, nullable, and dynamic conversions reject. |
| Null | Reference `null` and nullable value types exist. Nullable-reference annotations are flow-analysis warnings, not a different runtime reference carrier. All null-bearing values and dereferences initially reject. |
| Heap | Classes, arrays, delegates, closures, boxed values, reference identity, aliasing, mutation, properties, and allocation are heap/runtime behavior. They initially reject; value structs are also held out with aggregates. |
| Exceptions | Explicit `throw`, handlers/finally, and implicit arithmetic, cast, indexing, null, and initialization exceptions are abrupt completion not represented by VIR. Initial candidates prove the modeled arithmetic failure conditions impossible; all exception flow rejects. |
| Calls | A resolved pure non-generic static method can map to `CallStatic` under `C-CALL`. External, delegate, reflection, P/Invoke, recursive, multi-result, and effectful calls reject. |
| Dispatch | Overload resolution is compile-time, but virtual/interface/delegate/dynamic calls and user-defined operators/conversions can select or execute runtime code. Only predefined operators and direct static targets are candidates. |
| Initialization | Locals are subject to definite assignment. Fields have defaults; type initialization and static constructors can run once, throw, synchronize, and be triggered by use. Only explicit candidate-local initialization maps through `C-LOCAL`; field/type initialization rejects. |
| Concurrency | Threads, locks, volatile/interlocked behavior, shared heap state, and the .NET memory model are outside the initial profile. |
| Async | `async` methods, task-like builders, suspension, continuations, iterators, and async iterators are outside the initial profile. |
| Target/runtime variability | Language version, default checked option, nullable context, preprocessor symbols, architecture for native integers, target framework/reference assemblies, runtime, and compiler inputs affect analysis or execution. No default is chosen; `C-PIN` is mandatory. |

### 3.2 Java

| Dimension | Recorded behavior and initial boundary |
| --- | --- |
| Types | Primitive `boolean`; signed two's-complement `byte`, `short`, `int`, and `long`; unsigned `char`; binary `float`/`double`; reference types; arrays; and the null type. Candidate carriers are `boolean` and a frozen primitive integral set after Java numeric promotion. References and aggregates initially reject. |
| Integer and float behavior | Integer operators promote smaller operands to `int` and never signal overflow. Division truncates toward zero and throws on zero; `MIN / -1` yields `MIN` and remainder zero. Shift counts are masked to five or six low bits. `float` and `double` are IEEE binary32/binary64 language values but still need `F-BINARY-FLOAT` in MPK. |
| Evaluation order | Operand and argument evaluation is specified left to right, including completion of both integer-division operands before the divide-by-zero exception. `&&`, `||`, and `?:` short-circuit. Ordered VIR instructions and branches preserve accepted pure cases. |
| Conversions | Widening/narrowing primitive conversion, numeric promotion, constant narrowing, boxing/unboxing, and checked reference casts are distinct. Primitive BV extension/truncation can reuse `Convert`; boxing, unboxing, references, and runtime casts initially reject. |
| Null | The null reference is assignable/castable to reference types; dereference and unboxing can throw. References and null initially reject. |
| Heap | Objects and arrays are dynamically created and referenced, with identity, fields, aliasing, mutation, monitors, and garbage collection. All initially reject. |
| Exceptions | Checked and unchecked exceptions, `throw`, `try`/`catch`/`finally`, implicit arithmetic/null/cast/bounds failures, and abrupt completion are not represented. Candidate arithmetic checks exclude failure; exception flow rejects. |
| Calls | Compile-time overload selection followed by a resolved pure static target can map through `C-CALL`. Constructors, external/native calls, recursion, and effectful/library calls reject. |
| Dispatch | Instance methods use runtime overriding/interface dispatch after compile-time method selection. Method handles, lambdas, dynamic linkage, and overloaded behavior outside predefined primitives reject. |
| Initialization | Locals require definite assignment; fields receive default values. Class/interface initialization is synchronized, can recurse or fail, and runs static initializers in specified order when triggered. Only explicit local initialization is a candidate. |
| Concurrency | Threads, monitors, volatile fields, wait sets, and the Java memory model permit behaviors absent from VIR. They initially reject. |
| Async | Java has no core `async` expression, but threads, futures, completion stages, reactive APIs, and callbacks are runtime/library concurrency. They initially reject. |
| Target/runtime variability | JDK/compiler build, source language level, exact `--release`, system modules, class/module/source paths, processors, compiler options, runtime, and library inputs must be pinned. The processor set begins empty; T01 chooses no JDK default. |

### 3.3 Dart

| Dimension | Recorded behavior and initial boundary |
| --- | --- |
| Types | Values are objects; core scalar types include `bool`, `int`, `double`, `num`, and nullable forms under sound null safety. Exact `bool` values are representable. `int` is not frozen until a runtime target is selected; object/nullable/aggregate values initially reject. |
| Integer and float behavior | Native `int` is normally signed 64-bit two's-complement and wraps; web `int` uses JavaScript Number with 53 bits of integer precision. Bitwise/shift behavior and int/double identity also differ on web. `double` is binary64. All `int` operations remain under `F-DART-INT`; floating point remains under `F-BINARY-FLOAT`. |
| Evaluation order | The language defines expression evaluation, short-circuit Boolean operators, conditionals, and argument processing, but most operators are method invocations and can dispatch. Ordered VIR control flow is exact only after proving exact built-in operands and rejecting effectful/operator-dispatched cases. |
| Conversions | Assignability/subtyping, runtime `as` checks, `is`, null assertions, `dynamic`, and methods such as numeric conversion can fail or dispatch. No dynamic/runtime conversion is an initial candidate. |
| Null | Sound null safety separates nullable and non-nullable static types, but nullable variables can hold `null`, `!` can fail, and legacy/version boundaries matter. Nullable values and null assertions initially reject. |
| Heap | Numbers, functions, collections, class instances, closures, identity, fields, and mutation are object behavior. Allocation, aliasing, and object identity initially reject. |
| Exceptions | Dart exceptions are unchecked and can be thrown by explicit code, dispatch, casts, null assertions, late initialization, or libraries. Abrupt completion initially rejects. |
| Calls | A resolved pure top-level or static function is representable through `C-CALL`. Closures, tear-offs, instance methods, extension ambiguity, callable objects, and dynamic invocation reject. |
| Dispatch | Instance operators and methods dispatch on objects; `dynamic`, `noSuchMethod`, extensions, and runtime types complicate target resolution. Only exact built-in scalar operations and closed direct calls can be candidates. |
| Initialization | Non-nullable locals must be initialized before use. Top-level and static variables are lazy; `late` initialization can defer work and fail at runtime. Only explicit local initialization maps through `C-LOCAL`. |
| Concurrency | Isolates have separate memory and communicate by messages; each has an event loop. Native and web concurrency capabilities differ. Isolates and message/event behavior initially reject. |
| Async | `Future`, `Stream`, `async`, `await`, and async generators suspend and resume through an event loop. They initially reject. |
| Target/runtime variability | SDK/language version, experiments, native versus web target, runtime/compiler mode, platform libraries, package configuration, and analysis options are semantic inputs. No target is selected by T01; numeric rows remain blocked. |

### 3.4 TypeScript

| Dimension | Recorded behavior and initial boundary |
| --- | --- |
| Types | TypeScript annotations and most type constructs erase; runtime values follow ECMAScript's Undefined, Null, Boolean, String, Symbol, Number, BigInt, and Object types. Actual Boolean primitives are representable, but an annotation alone cannot prove that a runtime argument is Boolean. |
| Integer and float behavior | Ordinary `number` is ECMAScript Number (binary64 with NaN, infinities, signed zero, and precision loss), not a fixed integer. Bitwise operations coerce to 32-bit forms. `bigint` is arbitrary precision and does not implicitly mix with Number. Number needs `F-ECMASCRIPT-NUMBER`; BigInt needs `F-UNBOUNDED-INT`. |
| Evaluation order | ECMAScript evaluation algorithms order operand and argument effects; `&&` and `||` short-circuit but return an operand, while `?:` and `??` conditionally evaluate. Exact-Boolean closed cases map to VIR branches; general truthiness and operand-returning behavior reject. |
| Conversions | ECMAScript performs ToPrimitive, ToBoolean, ToNumeric, ToNumber, and other coercions; object conversion can invoke user code and throw. Type assertions generally emit no runtime check. General coercion and assertions-as-proof initially reject. |
| Null | Runtime has distinct `null` and `undefined`; `strictNullChecks` changes static checking, not their runtime existence. Both values and optional/nullish operations initially reject. |
| Heap | Objects, arrays, functions, closures, prototypes, property descriptors, getters/setters, proxies, identity, and mutation are heap/dynamic behavior. They initially reject. |
| Exceptions | Throw completions can arise from explicit `throw`, failed conversion, property access, calls, and host/runtime behavior. Exception flow initially rejects. |
| Calls | A same-module lexically fixed pure function may map through `C-CALL` only if analysis proves no rebinding, closure capture, module effect, getter, or dynamic call. General JavaScript callability rejects. |
| Dispatch | Property lookup follows prototypes and can invoke accessors/proxies; operators may coerce operands; method/function values are dynamic. These forms initially reject. |
| Initialization | `let`/`const`, temporal dead zones, class fields, static blocks, module instantiation/evaluation, import side effects, and downlevel emit all matter. Only proven explicit local initialization is a candidate. |
| Concurrency | Agents, workers, shared memory, atomics, and host event sources are ECMAScript/host behavior outside VIR. They initially reject. |
| Async | Async functions return promises; `await` and promise handlers schedule jobs whose host scheduling environment matters. Promises, jobs, generators, and callbacks initially reject. |
| Target/runtime variability | TypeScript version/options, `target`, `module`, module resolution, strictness, `lib` declarations, emit transforms, package graph, JavaScript runtime/host, and runtime flags must be pinned. `ESNext` is not a stable semantic pin. |

### 3.5 Python

| Dimension | Recorded behavior and initial boundary |
| --- | --- |
| Types | Runtime names refer to objects. `bool` has two instances and is a subclass of `int`; `int` has unlimited precision; `float` is usually implemented with a C double; `None` is a singleton. Annotations do not enforce runtime values. Exact built-in Boolean values are representable; ordinary integers and floats need foundations. |
| Integer and float behavior | Integer arithmetic is arbitrary precision; `//` floors and, for a nonzero divisor, `%` satisfies `x == (x // y) * y + (x % y)` with the divisor's sign. Shifts are unbounded and negative counts fail. Operators can invoke special methods. Float representation exposes implementation/platform details. Integer and float operations remain blocked under `F-UNBOUNDED-INT` or `F-BINARY-FLOAT`. |
| Evaluation order | Expressions evaluate left to right; assignment evaluates the right side before targets. `and`/`or` short-circuit and return an operand, comparisons can chain, and many operations may complete abruptly. Exact built-in Boolean, pure, non-throwing cases can use ordered VIR branches. |
| Conversions | Constructors and protocols such as `__int__`, `__index__`, `__bool__`, numeric promotion, and special methods can run arbitrary code or fail. Annotations and casts from typing helpers do not enforce runtime conversion. General conversions initially reject. |
| Null | `None` is an object singleton and is false in truth testing. None-bearing values, identity tests used as absence, and optional annotations initially reject. |
| Heap | Practically all values are objects with identity/lifetime; containers and user objects can be mutable and aliased; attribute access can dispatch. Heap state and mutation initially reject. |
| Exceptions | Explicit raises and ordinary name lookup, calls, arithmetic, indexing, conversion, imports, descriptors, and uninitialized locals can raise. Exception handling and abrupt completion initially reject. |
| Calls | A same-module function-name call could use `C-CALL` only after MPK-owned analysis proves exact binding, no decorator/rebinding/closure/global effect, exact built-ins, and a closed acyclic graph. General callable objects and builtins reject. |
| Dispatch | Operators use special-method lookup; attribute access can invoke descriptors, `__getattribute__`, or `__getattr__`; metaclasses and monkey patching alter behavior. Dynamic dispatch initially rejects. |
| Initialization | Reading an unbound local raises; module import creates and executes a module; decorators and default expressions run at definition; class bodies execute. Only explicit, statically proven local initialization maps through `C-LOCAL`. |
| Concurrency | Threads, processes, interpreters, shared resources, implementation locks, signals, and extension code vary by implementation and platform. They initially reject. |
| Async | Coroutines, `await`, async iterators/generators, and event-loop libraries suspend and resume dynamically. They initially reject. |
| Target/runtime variability | Python implementation/version, platform, float details, optimization flags, import roots/hooks, standard-library/native-extension set, builtins, environment, and annotation evaluation policy must be pinned. T01 does not default to CPython or a version. |

## 4. Closed candidate-operation inventory

The operation IDs below are the complete T01 inventory. A language phase may
later choose a smaller initial subset, but it may not silently treat an
unlisted operation as admitted. Adding an operation requires a reviewed
revision of this record before that operation is considered by successor
specification work.

A row groups several spellings only when they share one disposition for the
operand/result forms described by that cell. A source overload or promoted
form that does not have the named operand/result types is not admitted by the
group; exact source type resolution rejects it instead of borrowing the row's
mapping.

| ID | Domain | Candidate behavior |
| --- | --- | --- |
| `M01` | types | Exact Boolean literal, parameter/local value, and result carrier |
| `M02` | types/integer | Source-declared fixed-width signed or unsigned integer carrier |
| `M03` | types/integer | Native-sized, target-sized, or arbitrary-precision integer carrier |
| `M04` | types/float | Binary floating/ECMAScript Number carrier and operations |
| `M05` | types/float | C# decimal-style numeric carrier and operations |
| `M06` | types/heap | Aggregate value construction, projection, equality, or indexing |
| `M07` | evaluation order | Left-to-right pure operand and argument evaluation |
| `M08` | evaluation order | Exact-Boolean logical not and equality/inequality |
| `M09` | integer | Integer equality and signed/unsigned ordering |
| `M10` | evaluation order | Short-circuit `and`/`or` over exact Boolean operands |
| `M11` | evaluation order | Boolean conditional branch, join, early return, and result |
| `M12` | initialization | Explicit scalar local assignment and later read |
| `M13` | integer | Fixed-width wrapping add/subtract/multiply and source-defined same-width wrapping negate |
| `M14` | integer/exceptions | Overflow-trapping add/subtract/multiply and signed negate |
| `M15` | integer | Mathematical arbitrary-precision add/subtract/multiply/negate |
| `M16` | integer/exceptions | Integer division/remainder truncating toward zero |
| `M17` | integer | Integer floor division and paired modulo semantics |
| `M18` | integer | Integer bitwise not/and/or/xor |
| `M19` | integer | Fixed-width shifts with a masked count |
| `M20` | integer | Arbitrary-precision/unbounded shifts |
| `M21` | conversions | Exact extension or wrapping/truncating fixed-BV conversion |
| `M22` | conversions/exceptions | Range-checked numeric conversion |
| `M23` | conversions/dispatch | Runtime coercion, truthiness, boxing, or user conversion protocol |
| `M24` | null | Null, undefined, None, nullable carrier, or dereference behavior |
| `M25` | heap | Allocation, references, identity, aliasing, fields/properties, or mutation |
| `M26` | exceptions | General throw/raise, catch/finally, or abrupt-completion propagation |
| `M27` | calls | Direct pure same-module top-level/static one-result call |
| `M28` | dispatch | Virtual/interface/instance/function-value/operator/descriptor dispatch |
| `M29` | initialization | Definite initialization of candidate scalar locals |
| `M30` | initialization | Default, static, class, module, global, lazy, or definition-time initialization |
| `M31` | concurrency | Threads, locks, shared memory, atomics, isolates, or messages |
| `M32` | async | Async/await, futures/promises, coroutines, generators, jobs, or event loops |
| `M33` | target/runtime variability | Complete semantic compiler/language/target/runtime/options pin |
| `M34` | contracts | Pure precondition/postcondition attachment and normalization |

## 5. Reading the matrix

Mappings name the current operation or the catalog entry that owns the gap.
For example, `E — mask + shift` means an existing `bv_and` with a width-minus-
one constant followed by the appropriate existing shift; it does not assign a
new primitive meaning to the shift. A `C` cell lists all checks specific to the
row. Global fail-closed rules still reject unknown syntax, types, conversions,
dispatch, targets, effects, or compiler nodes.

## 6. Disposition matrix

| ID | C# | Java | Dart | TypeScript | Python |
| --- | --- | --- | --- | --- | --- |
| `M01` | `E` — `bool`, `Const` | `E` — `bool`, `Const` | `E` — exact `bool`, `Const` | `E` — actual Boolean primitive, `Const` | `E` — exact built-in `bool`, `Const` |
| `M02` | `E` — fixed `bv` widths | `E` — fixed `bv` after promotion | `R` — no source fixed-width primitive | `R` — Number/BigInt are not fixed BV carriers | `R` — `int` is not fixed width |
| `M03` | `R` — `nint`/`nuint` initially excluded | `R` — no initial native/unbounded primitive | `F` — `F-DART-INT` | `F` — BigInt via `F-UNBOUNDED-INT` | `F` — `F-UNBOUNDED-INT` |
| `M04` | `F` — `F-BINARY-FLOAT` | `F` — `F-BINARY-FLOAT` | `F` — `F-BINARY-FLOAT` | `F` — `F-ECMASCRIPT-NUMBER` | `F` — `F-BINARY-FLOAT` plus implementation pin |
| `M05` | `F` — `F-CSHARP-DECIMAL` | `R` — library/object decimal excluded | `R` — no initial core decimal carrier | `R` — no initial decimal carrier | `R` — library decimal/object excluded |
| `M06` | `R` — aggregates deferred; arrays are heap | `R` — arrays/records/classes are references | `R` — collection/record/object behavior excluded | `R` — object/array behavior excluded | `R` — collection/object behavior excluded |
| `M07` | `E` — ordered instructions/CFG | `E` — ordered instructions/CFG | `E` — ordered pure built-in expressions | `E` — ordered ECMAScript evaluation after effect rejection | `E` — ordered pure built-in expressions |
| `M08` | `E` — `not`, `eq`, `not_eq` | `E` — `not`, `eq`, `not_eq` | `E` — exact-bool operations | `E` — actual-Boolean operations | `E` — exact built-in-bool operations |
| `M09` | `E` — BV equality and signed/unsigned comparisons | `E` — promoted BV equality/comparisons | `F` — `F-DART-INT` | `F` — BigInt via `F-UNBOUNDED-INT`; Number remains in `M04` | `F` — `F-UNBOUNDED-INT` |
| `M10` | `E` — `Branch` graph | `E` — `Branch` graph | `E` — exact-bool `Branch` graph | `E` — exact-Boolean `Branch` graph only | `E` — exact-bool `Branch` graph only |
| `M11` | `E` — `Branch`/`Jump`/`Return` | `E` — `Branch`/`Jump`/`Return` | `E` — exact-bool condition | `E` — exact-Boolean condition | `E` — exact-bool condition |
| `M12` | `E` — `Copy` | `E` — `Copy` | `E` — `Copy` | `E` — `Copy` | `E` — `Copy` |
| `M13` | `E` — unchecked `bv_add/sub/mul` and same-type signed `bv_neg` | `E` — promoted wrapping BV operations | `F` — `F-DART-INT` | `R` — Number does not use fixed-width wrapping arithmetic; BigInt is in `M15` | `R` — Python integer arithmetic does not wrap |
| `M14` | `C` — BV operations plus `C-OVERFLOW` | `R` — primitive operators do not trap on overflow | `R` — no target-independent trapping form | `R` — no Number/BigInt overflow trap | `R` — arbitrary integers do not overflow |
| `M15` | `R` — no arbitrary-precision primitive candidate | `R` — no arbitrary-precision primitive candidate | `R` — target `int` is not arbitrary precision | `F` — BigInt via `F-UNBOUNDED-INT` | `F` — `F-UNBOUNDED-INT` |
| `M16` | `C` — signed/unsigned div/rem plus `C-DIV`; signed always excludes `MIN/-1` | `C` — signed `bv_sdiv/srem` plus `C-DIV` with `divisor_nonzero` only | `F` — `F-DART-INT` (`~/`, `%`, and `remainder` distinguished later) | `F` — BigInt via `F-UNBOUNDED-INT`; Number remains in `M04` | `R` — `//` and `%` are floor-based; `/` is floating |
| `M17` | `R` — no primitive floor-div/mod pair | `R` — no primitive floor-div/mod pair | `R` — not accepted by analogy to Python | `R` — no initial floor-div/mod pair | `F` — `F-UNBOUNDED-INT` with Python floor/mod rules |
| `M18` | `E` — `bv_not/and/or/xor` | `E` — promoted `bv_not/and/or/xor` | `F` — `F-DART-INT` | `F` — BigInt via `F-UNBOUNDED-INT`; Number coercion remains in `M04` | `F` — `F-UNBOUNDED-INT` |
| `M19` | `E` — `bv_and(count,width-1)` then existing shift | `E` — `bv_and(count,width-1)` then existing shift | `F` — `F-DART-INT` | `F` — masked Number shifts via `F-ECMASCRIPT-NUMBER` | `R` — Python shifts are unbounded and negative counts fail |
| `M20` | `R` — fixed-width shifts only | `R` — fixed-width shifts only | `R` — target-sized behavior, not arbitrary precision | `F` — BigInt via `F-UNBOUNDED-INT` | `F` — `F-UNBOUNDED-INT` |
| `M21` | `E` — built-in exact/unchecked BV `Convert` semantics | `E` — primitive widening/narrowing BV `Convert` semantics | `R` — numeric methods/casts are not BV conversion | `R` — assertions erase; coercion is not BV conversion | `R` — protocols are not BV conversion |
| `M22` | `F` — `F-CHECKED-CONVERT` | `R` — primitive casts truncate rather than range-trap | `R` — runtime methods/casts initially excluded | `R` — runtime coercion initially excluded | `R` — runtime constructors/protocols initially excluded |
| `M23` | `R` — dynamic/user-defined/boxing/nullable conversion | `R` — boxing/unboxing/reference/string conversion | `R` — `dynamic`, runtime casts, operator methods | `R` — ToPrimitive/ToBoolean/ToNumeric and assertions-as-proof | `R` — truthiness and conversion/special-method protocols |
| `M24` | `R` — null/nullable/reference values | `R` — null/reference values | `R` — nullable values/null assertions | `R` — null/undefined/nullish behavior | `R` — `None` and optional/identity behavior |
| `M25` | `R` — allocation/references/identity/alias mutation | `R` — objects/arrays/references/monitors | `R` — object allocation/identity/mutation | `R` — objects/prototypes/accessors/proxies | `R` — objects/identity/descriptors/mutation |
| `M26` | `R` — exception/handler/finally flow | `R` — checked/unchecked exception flow | `R` — throw/catch/finally flow | `R` — abrupt completion/throw flow | `R` — raise/handler/finally flow |
| `M27` | `C` — `CallStatic` plus `C-CALL` | `C` — static `CallStatic` plus `C-CALL` | `C` — top-level/static `CallStatic` plus `C-CALL` | `C` — closed lexical `CallStatic` plus `C-CALL` | `C` — closed same-module name plus `C-CALL` |
| `M28` | `R` — virtual/interface/delegate/dynamic/operator dispatch | `R` — virtual/interface/lambda/dynamic dispatch | `R` — instance/dynamic/extension/callable dispatch | `R` — function values/prototypes/getters/proxies/coercion | `R` — callable/descriptor/special-method dispatch |
| `M29` | `C` — `Copy` plus `C-LOCAL` | `C` — `Copy` plus `C-LOCAL` | `C` — `Copy` plus `C-LOCAL` | `C` — `Copy` plus MPK proof under `C-LOCAL` | `C` — `Copy` plus MPK proof under `C-LOCAL` |
| `M30` | `R` — field defaults/type initialization/static constructors | `R` — defaults/class/interface initialization | `R` — top-level/static lazy and `late` initialization | `R` — TDZ/class/module/import initialization | `R` — module/class/decorator/default/import execution |
| `M31` | `R` — threads/locks/shared memory | `R` — threads/monitors/JMM | `R` — isolates/messages/events | `R` — agents/workers/shared memory/atomics | `R` — threads/processes/interpreters/shared resources |
| `M32` | `R` — tasks/async/iterators/continuations | `R` — futures/callbacks/runtime concurrency | `R` — Future/Stream/async/await/event loop | `R` — Promise/jobs/async/await/generators | `R` — coroutine/await/async generator/event loop |
| `M33` | `C` — `C-PIN` for C#/.NET inputs | `C` — `C-PIN` for JDK/release/module inputs | `C` — `C-PIN`; target choice remains blocked | `C` — `C-PIN`; JS runtime/host choice remains blocked | `C` — `C-PIN`; implementation/version choice remains blocked |
| `M34` | `C` — `VirContract` plus `C-CONTRACT` | `C` — `VirContract` plus `C-CONTRACT` | `C` — `VirContract` plus `C-CONTRACT` | `C` — `VirContract` plus `C-CONTRACT` | `C` — `VirContract` plus `C-CONTRACT` |

## 7. Unresolved-question ledger

Every item has state `blocked`. None is accepted, defaulted, or silently
resolved by the matrix. The named later owner is where a choice may be made;
it is not permission to start that work early.

| ID | Affected rows | Blocked question | Later owner | State |
| --- | --- | --- | --- | --- |
| `Q-CS-01` | `M13`, `M14`, `M16`, `M21`, `M22`, `M33` | Will the first C# profile require one overflow context, admit explicit checked and unchecked forms separately, or reject compiler-option-dependent forms? No default checked setting is inferred. | `MLANG-01-T03` | blocked |
| `Q-CS-02` | `M02`, `M09`, `M13`-`M21` | Which fixed integral source types are initial, and how are small-type/`char` promotions and result conversions delimited? | `MLANG-01-T03` | blocked |
| `Q-CS-03` | `M03`, `M27`, `M30`, `M33` | Which language version, .NET runtime, target framework, reference assemblies, architecture, nullable mode, and preprocessor set are pinned? | `MLANG-00-T02`, then `MLANG-01-T03` | blocked |
| `Q-JAVA-01` | `M02`, `M09`, `M13`, `M16`, `M18`, `M19`, `M21`, `M33` | Which primitive set and JDK `--release` enter the first profile, and which promotion/constant-narrowing forms are admitted? | Java specification phase after its entry gate | blocked |
| `Q-JAVA-02` | `M27`, `M30` | Can compiler integration prove that candidate static calls and constants trigger no class/interface initialization effect, or must all such calls wait? | `MLANG-00-T02` feasibility; later Java specification | blocked |
| `Q-DART-01` | `M03`, `M04`, `M09`, `M13`, `M16`, `M18`, `M19`, `M33` | Is one native target, one web target, or neither defensible? T01 does not choose a runtime target. | Dart phase after its entry gate | blocked |
| `Q-DART-02` | `M03`, `M09`, `M13`, `M16`, `M18`-`M21` | For the selected SDK/target, what are the exact literal, overflow, division, remainder, shift, bitwise, and int/double identity rules exposed by the runtime and analyzer? | `MLANG-00-T02` feasibility; later Dart specification | blocked |
| `Q-TS-01` | `M03`, `M04`, `M09`, `M13`, `M15`-`M20` | Does the first TypeScript profile use a checked Number subset, BigInt, a coercion-free 32-bit subset, or no numeric operations? | TypeScript phase after its entry gate | blocked |
| `Q-TS-02` | `M01`, `M07`-`M12`, `M27`, `M29` | How is an actual runtime primitive boundary established when TypeScript annotations erase? An annotation alone is never the answer. | `MLANG-00-T02` feasibility; later TypeScript specification | blocked |
| `Q-TS-03` | `M27`, `M30`, `M32`, `M33` | Which TypeScript compiler, emit target/module mode, JavaScript runtime/host, libraries, package graph, and module-initialization boundary are pinned? | TypeScript phase after its entry gate | blocked |
| `Q-PY-01` | `M03`, `M09`, `M15`-`M20` | Is Python integer support an arbitrary-precision foundation, a proved bounded embedding with an explicit width, or an initial rejection? No width is invented. | Python phase after its entry gate | blocked |
| `Q-PY-02` | `M01`, `M07`-`M12`, `M23`, `M27`-`M30` | What closed MPK-owned analysis proves exact builtins, bindings, no descriptor/operator dispatch, no rebinding/monkey patching, and no definition/import effects without trusting annotations? | `MLANG-00-T02` feasibility; later Python specification | blocked |
| `Q-PY-03` | `M04`, `M31`-`M33` | Which Python implementation/version, platform, optimization mode, standard-library/native-extension set, import policy, builtins, and float model are pinned? | Python phase after its entry gate | blocked |
| `Q-X-01` | `M22` | Is a checked numeric conversion foundation justified by at least one frozen language profile, and what exact range predicate and failure model does it use? | `MLANG-01-T01/T02` only after gap audit | blocked |
| `Q-X-02` | `M27` | Are direct calls part of each first straight-line profile or a follow-up revision? Representability through `C-CALL` does not decide sequencing. | each language specification phase | blocked |
| `Q-X-03` | `M34` | What source/sidecar contract grammar, attachment, type domain, and diagnostics does each language own? No comment, annotation, or decorator is assumed authoritative. | each language specification phase | blocked |

## 8. Exit-gate audit

The closed inventory has 34 rows and five required language columns, for 170
language dispositions. Each section 6 cell begins with exactly one of `E`,
`C`, `F`, or `R`; there are no blank, combined, “same as”, unknown, or implicit
cells. The totals are 42 `E`, 23 `C`, 26 `F`, and 79 `R`. Sections 2.1 and 2.2
resolve every `C` and `F` label to one catalog.

All 16 unresolved questions in section 7 are explicitly `blocked`. No row,
question, language, profile, target, foundation, schema value, or operation is
marked accepted by this record. Therefore:

- every candidate initial operation has exactly one disposition per language;
- every behavior named by the task is covered by at least one closed row;
- target-dependent behavior remains blocked until an explicit pin exists;
- no unresolved question supplies a default; and
- the `MLANG-00-T01` exit gate is satisfied without starting
  `MLANG-00-T02` or any successor/language specification work.

## 9. Official references

These references support the semantic facts; they are research inputs, not
the eventual toolchain/runtime pins.

### C#

- [C# language specification: types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types)
- [C# language specification: expressions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions)
- [C# language specification: conversions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/conversions)
- [C# language specification: variables](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/variables)
- [C# language specification: classes and async functions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes)
- [C# language specification: exceptions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/exceptions)
- [C# compiler option for overflow checking](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/compiler-options/language)

### Java

- [JLS 25, Chapter 4: types, values, and variables](https://docs.oracle.com/javase/specs/jls/se25/html/jls-4.html)
- [JLS 25, Chapter 5: conversions and contexts](https://docs.oracle.com/javase/specs/jls/se25/html/jls-5.html)
- [JLS 25, Chapter 11: exceptions](https://docs.oracle.com/javase/specs/jls/se25/html/jls-11.html)
- [JLS 25, Chapter 12: execution and initialization](https://docs.oracle.com/javase/specs/jls/se25/html/jls-12.html)
- [JLS 25, Chapter 15: expressions](https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html)
- [JLS 25, Chapter 16: definite assignment](https://docs.oracle.com/javase/specs/jls/se25/html/jls-16.html)
- [JLS 25, Chapter 17: threads and locks](https://docs.oracle.com/javase/specs/jls/se25/html/jls-17.html)

### Dart

- [Dart language specification route](https://dart.dev/resources/language/spec)
- [Dart built-in types](https://dart.dev/language/built-in-types)
- [Numbers in Dart across native and web targets](https://dart.dev/resources/language/number-representation)
- [Dart variables, null safety, and initialization](https://dart.dev/language/variables)
- [Dart exceptions](https://dart.dev/language/error-handling)
- [Dart concurrency and isolates](https://dart.dev/language/concurrency)
- [Dart asynchronous programming](https://dart.dev/language/async)

### TypeScript and ECMAScript

- [TypeScript for the New Programmer: runtime behavior and erased types](https://www.typescriptlang.org/docs/handbook/typescript-from-scratch.html)
- [TypeScript Handbook: everyday types and null/undefined](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html)
- [TypeScript compiler options](https://www.typescriptlang.org/docs/handbook/compiler-options.html)
- [TypeScript `target` option](https://www.typescriptlang.org/tsconfig/target.html)
- [ECMAScript language specification](https://tc39.es/ecma262/)
- [ECMAScript data types and values](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html)
- [ECMAScript type-conversion abstract operations](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-type-conversion)
- [ECMAScript expressions and short-circuit evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html)
- [ECMAScript jobs and execution contexts](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html)

### Python

- [Python language reference](https://docs.python.org/3/reference/)
- [Python built-in types](https://docs.python.org/3/library/stdtypes.html)
- [Python expressions and evaluation order](https://docs.python.org/3/reference/expressions.html)
- [Python data model and special-method dispatch](https://docs.python.org/3/reference/datamodel.html)
- [Python execution model](https://docs.python.org/3/reference/executionmodel.html)
- [Python import system](https://docs.python.org/3/reference/import.html)
- [Python exceptions](https://docs.python.org/3/reference/executionmodel.html#exceptions)
