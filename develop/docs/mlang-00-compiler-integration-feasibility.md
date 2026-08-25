# MLANG-00-T02 Official Compiler Integration Feasibility

Status: complete non-normative feasibility record for `MLANG-00-T02`.

Prepared: 2026-08-25.

Research snapshot: Roslyn's documented compiler API, JDK 25, Dart analyzer
14.1.0, the TypeScript compiler API documented for 6.0 and earlier, and
CPython 3.14. These are research anchors, not production toolchain selections.

## 1. Scope and authority

This record validates the supported compiler or analyzer integration boundary
for C#, Java, Dart, TypeScript, and Python. It builds on the closed 34-row
inventory in `mlang-00-semantic-comparison-matrix.md` and records:

- the minimum public APIs needed for resolved syntax, symbols, types, control
  flow, source positions, diagnostics, and target/options;
- the complete classes of compiler, SDK, reference, standard-library, package,
  source, configuration, and host inputs that a deterministic invocation must
  pin;
- unstable, internal, target-specific, or ambient-state-dependent surfaces
  that an implementation must not treat as authority; and
- an explicit integration `GO` or `NO-GO` result for every matrix row in every
  language.

This is Gate B design evidence under
`06_multilanguage_frontend_design.md`. It does not choose a production
compiler version, language profile, target, runtime, contract grammar, or
successor schema. It adds no frontend code, release bundle, registry entry,
language identifier, VIR producer, checker input, hash domain, or axiom
category. No experiment is connected to `mpk-cli` or retained as production
evidence.

The trust boundary in `../specs/TRUST_BOUNDARY_V0.md` continues to apply. A
compiler, analyzer, typed tree, symbol, control-flow graph, diagnostic, or
MPK-owned frontend analysis is untrusted helper evidence. Only canonical
certificate checking can justify proof acceptance.

## 2. Verdict contract

Each language in sections 4 through 8 has two exhaustive row lists:

| Verdict | Meaning |
| --- | --- |
| `GO` | The supported public boundary exposes enough information to recognize the row and either lower it exactly or perform a finite, exhaustive MPK-owned derivation over a later-frozen source subset. Every condition in this record and the T01 required-check catalog still applies. |
| `NO-GO` | The row was already a T01 foundation/rejection, or the supported public boundary cannot establish a required runtime fact without a new semantic mechanism. The row remains excluded from the candidate initial subset. |

`GO` is an integration-feasibility result, not source acceptance. It does not
resolve a T01 blocked question, activate a language, or authorize current VIR
emission. A later specification may choose fewer `GO` rows. It may not admit a
`NO-GO` row without a reviewed semantic-matrix and feasibility revision.

Every T01 `F` or `R` cell is necessarily `NO-GO`; an analyzer API cannot turn
an absent checked foundation into an existing representation. A T01 `E` or
`C` cell may still become `NO-GO` here when static compiler facts do not prove
the required runtime boundary. If a source form uses several rows, every row
must be `GO`, every required parameter must be frozen, and every check must be
discharged. Otherwise the whole form rejects before VIR publication.

Some `GO` rows are deliberately structural. In particular, control-flow,
local-assignment, pin, and contract-attachment feasibility does not by itself
provide a usable runtime value carrier. This distinction matters for the
TypeScript and Python results.

## 3. Common integration boundary

### 3.1 Minimum fact contract

Each frontend integration needs the following facts. A language with no
supported public control-flow API may construct its own graph only from an
exhaustively matched public syntax tree for the frozen subset.

| Fact | Minimum requirement |
| --- | --- |
| Resolved syntax | Exact source bytes parsed at the pinned language version; every accepted node and token kind matched; malformed, recovered, preview, unknown, or omitted forms rejected. |
| Symbols | Declaration/use identity, scope, selected call target, and source-versus-library provenance after semantic analysis; ambiguous, dynamic, error, synthetic, or unavailable symbols rejected. |
| Types and conversions | Exact operand/result types, built-in versus user-defined operation, promotions/conversions, nullability mode where relevant, and error/unknown types. Static type facts never substitute for a runtime guarantee in an erased or dynamic language. |
| Control flow | Evaluation order, branches, joins, returns, reachability, and dominating local assignment. An MPK-owned graph must reject every syntax kind it does not model and remains untrusted. |
| Source positions | Stable offsets plus line/column mapping from the exact input bytes, with a frozen path-normalization and encoding convention for source maps and diagnostics. |
| Diagnostics | Complete syntax, declaration, option, and semantic diagnostics before lowering. Normalize and sort by stable path/span/severity/code fields; human messages are informational and version/locale pinned. Any diagnostic category not explicitly permitted rejects. |
| Target and options | Exact language level, feature/preview switches, target/runtime identity, checking modes, defines, module/package resolution, standard-library/reference set, and every option that can alter syntax, resolution, typing, lowering, or runtime behavior. Missing or ambient defaults reject. |

Compiler-provided flow information is never sufficient on its own. The
frontend still performs the language-owned subset checks, verifies that every
operation is the predefined accepted operation, closes direct-call graphs,
and rejects any effect or abrupt completion outside the profile.

### 3.2 Deterministic input closure

A future invocation is isolated only if its registered bundle or immutable
snapshot identifies and hashes all of these input classes:

1. The exact compiler/analyzer package, its transitive code dependencies, the
   runtime used to host it, and the MPK frontend executable.
2. Every source byte string, logical path, encoding, source order where an API
   observes order, generated-source prohibition, and language-version marker.
3. Every compiler/analyzer option and target identity, including defaults
   rewritten as explicit values rather than inherited from a project tool.
4. Every SDK, reference assembly, system module, platform library, standard-
   library declaration/source file, and their transitive identities.
5. Every package/module dependency, lock/config file, declaration or class
   artifact, and source needed by resolution. A global package cache is not an
   input boundary.
6. The virtual filesystem's case rule, canonical logical root, path mapping,
   current-directory behavior, newline/encoding policy, locale, and diagnostic
   normalization. Host absolute paths must not enter canonical artifacts.
7. The explicitly empty or fully pinned set of analyzers, generators,
   annotation processors, plugins, transforms, and subordinate executables.
   The candidate boundary begins with all such extension points disabled.
8. Resource limits and the no-network, no-credential, no package-manager, no
   project-discovery, and no runtime-code-execution policy.

The frontend must provide a finite filesystem or file-manager view containing
only the declared snapshot. File lookup outside that view fails. Environment
variables, user profiles, parent directories, working-directory package
files, globally installed tools, registry settings, IDE state, and network
resolution are not implicit inputs.

### 3.3 Disallowed shortcuts

The minimum boundary excludes IDE/workspace/project-system layers when a
lower-level compilation API exists. It also excludes private fields, internal
compiler packages, serialized compiler trees, undocumented flow nodes, daemon
or incremental cache state, and compiler plugins. A later frontend must never:

- infer runtime behavior from a compiler type in TypeScript or an annotation
  in Python;
- use an analyzer's success as proof that lowering is correct;
- silently fall back from a missing reference, package, SDK, or option to a
  host default; or
- keep processing an unknown syntax, symbol, type, conversion, operation,
  target, or diagnostic as though it were a supported case.

## 4. C# / Roslyn feasibility record

### 4.1 Supported public boundary

Verdict: feasible through the Roslyn compiler layer. The workspace/MSBuild
layer is unnecessary and would add ambient project and filesystem state.

| Fact | Minimum supported API |
| --- | --- |
| Resolved syntax | `CSharpSyntaxTree.ParseText` with explicit `CSharpParseOptions`; `SyntaxTree`, `SyntaxNode`, and the public C# syntax node hierarchy. |
| Symbols | `CSharpCompilation.GetSemanticModel`, `SemanticModel.GetDeclaredSymbol`, `GetSymbolInfo`, and public `ISymbol`/`IMethodSymbol` interfaces. |
| Types and conversions | `SemanticModel.GetTypeInfo`, `ClassifyConversion`, and `GetOperation`; public `ITypeSymbol`, `IConversionOperation`, and built-in `IOperation` nodes distinguish selected operators/conversions from source spellings. |
| Control flow | Public `ControlFlowGraph.Create` over the supported method/body operation, plus public operation/basic-block/branch interfaces. An unavailable graph or unrecognized operation rejects. |
| Source positions | `SyntaxTree.GetLineSpan` or `Location.GetLineSpan` over exact `TextSpan` values. |
| Diagnostics | `Compilation.GetDiagnostics`, retaining `Diagnostic.Id`, severity, location, and `GetMessage` output under the pinned locale. |
| Target and options | Explicit `CSharpParseOptions`, `CSharpCompilationOptions`, `MetadataReference` values, and compilation creation; no project loader or default references. |

### 4.2 Required input closure

The pin set contains the exact `Microsoft.CodeAnalysis.CSharp` package and all
managed dependencies, the hosting .NET runtime, architecture, frontend bytes,
all source bytes, and all reference-assembly bytes. Reference assemblies come
from one exact target-framework reference pack; runtime implementation
assemblies found on the host are not substitutes.

Parse pins include language version, source kind, documentation mode, and
preprocessor symbols. Compilation pins include output kind, platform,
optimization, overflow-checking mode, nullable context, warning/diagnostic
options, deterministic setting, metadata import behavior, and explicit source
and metadata resolvers. The analyzer, source-generator, additional-file, and
script-reference sets begin empty. No `MSBuildWorkspace`, solution/project
discovery, NuGet restore, default trusted-platform-assembly list, or host
current directory participates.

### 4.3 Instability and semantic gaps

Roslyn ships as versioned packages and has documented API-breaking changes;
the exact package graph must be pinned and requalified on upgrade. Only public
`Microsoft.CodeAnalysis` and `Microsoft.CodeAnalysis.CSharp` contracts are
eligible. Workspace services, compiler implementation types, reflection over
private members, and internal bound trees are not.

Roslyn can resolve checked context, predefined operators, conversions, direct
method symbols, source positions, and flow. It cannot make native-sized
integers target-independent or supply missing floating, decimal, checked-
conversion, heap, or exception foundations. For `M27`, feasibility is limited
to a closed acyclic graph of source-declared, non-generic static methods whose
containing-type initialization closure is proved inert; static constructors,
nonconstant static initialization, metadata-only bodies, external calls, and
unresolved initialization effects reject. Purity is established by exhaustive
MPK-owned body analysis, not by an attribute or analyzer claim.

`M34` is feasible only as source-position-backed attachment to a later-frozen
MPK contract grammar. Roslyn attributes or comments are not contracts by
default.

### 4.4 Explicit row verdicts

`GO` (18): `M01`, `M02`, `M07`, `M08`, `M09`, `M10`, `M11`, `M12`, `M13`,
`M14`, `M16`, `M18`, `M19`, `M21`, `M27`, `M29`, `M33`, `M34`.

`NO-GO` (16): `M03`, `M04`, `M05`, `M06`, `M15`, `M17`, `M20`, `M22`,
`M23`, `M24`, `M25`, `M26`, `M28`, `M30`, `M31`, `M32`.

All 18 T01 `E`/`C` candidates are API-feasible under these restrictions. The
16 T01 `F`/`R` rows remain blocked. Exact integer types, checked-context policy,
target framework, and call sequencing remain later specification choices.

## 5. Java / `javac` feasibility record

### 5.1 Supported public boundary

Verdict: feasible through the standard Java Compiler API and exported Compiler
Tree API. The `com.sun.source.*` packages are supported exports of the
`jdk.compiler` module; `com.sun.tools.javac.*` packages are internal and are
not part of this boundary.

| Fact | Minimum supported API |
| --- | --- |
| Resolved syntax | `JavaCompiler.getTask`, downcast to `JavacTask`, then `parse()` and `analyze()`; public `CompilationUnitTree` and `com.sun.source.tree` nodes. |
| Symbols | `Trees.instance(task).getElement(TreePath)`, `JavacTask.getElements()`, and public `javax.lang.model.element` interfaces. |
| Types and conversions | `Trees.getTypeMirror(TreePath)`, `JavacTask.getTypes()`, and public `TypeMirror`, `Types`, and `Elements` APIs after analysis. Predefined primitive operations are checked from resolved trees and types. |
| Control flow | No supported public `javac` CFG API. Build an MPK-owned graph by exhaustive traversal of the accepted public `Tree` kinds; do not use `com.sun.tools.javac.comp.Flow` or internal tree fields. |
| Source positions | `Trees.getSourcePositions()` and `CompilationUnitTree.getLineMap()` over the exact `JavaFileObject` content. |
| Diagnostics | `DiagnosticCollector<JavaFileObject>` with diagnostic kind, code, source, start/end/line/column, and locale-pinned message. |
| Target and options | Explicit compiler option list plus a controlled `StandardJavaFileManager`/forwarding file manager with every location set; `--release`, encoding, processing policy, and module/class/source paths are never defaults. |

### 5.2 Required input closure

The pin set contains one exact JDK image and build, including the compiler and
language-model modules, its `ct.sym`/system-module/reference content, the Java
launcher runtime, frontend bytes, all source `JavaFileObject` bytes, and every
class/module dependency byte. It records the exact `--release` and language
feature policy, UTF-8 source encoding, diagnostic locale, module graph, root
modules, and all compiler options.

The file manager receives explicit empty or finite source, class, processor,
module, upgrade-module, patch-module, and generated-output locations. The
initial processor and plugin sets are empty (`-proc:none`), implicit source
discovery is disabled, and the task stops after analysis rather than
generation. `CLASSPATH`, `JDK_JAVAC_OPTIONS`, parent directories, the current
directory, installed extension mechanisms, and service-loaded processors are
not permitted to fill a missing location. Although `JDK_JAVAC_OPTIONS` is not
used by the API task, it is still removed from the launch environment.

### 5.3 Instability and semantic gaps

The exported compiler/tree APIs are the supported boundary, but their exact
JDK version still requires a migration corpus. Internal `javac` classes and
module-export overrides such as `--add-exports` or `--add-opens` are forbidden.
The absence of a public CFG API is acceptable only for a tiny closed syntax
subset with an exhaustive MPK-owned graph builder; it is not permission to
copy or serialize `javac`'s internal flow graph.

The public APIs expose resolved primitive operations, promotions, direct
method symbols, diagnostics, and source positions. They do not remove Java
class/interface initialization. `M27` is therefore feasible only when the
selected target is a source-declared static method and analysis proves the
complete initialization closure inert: no static initializer blocks, no
nonconstant static field initialization, and no unresolved superclass or
default-method-superinterface initialization. A constant variable is treated
as constant only when the language model supplies its compile-time value via
`VariableElement.getConstantValue()`. Metadata-only initialization or any
uncertain closure rejects. Method purity and call acyclicity still require
exhaustive source analysis. A source interface with no nonconstant fields is
the minimal non-empty target shape; a class target is eligible only when its
superclass/default-method-interface initialization closure is equally
available and inert.

`M34` uses source positions and resolved declarations to attach a later-frozen
MPK contract; Java annotations are not trusted contracts merely because the
compiler resolves them.

### 5.4 Explicit row verdicts

`GO` (17): `M01`, `M02`, `M07`, `M08`, `M09`, `M10`, `M11`, `M12`, `M13`,
`M16`, `M18`, `M19`, `M21`, `M27`, `M29`, `M33`, `M34`.

`NO-GO` (17): `M03`, `M04`, `M05`, `M06`, `M14`, `M15`, `M17`, `M20`,
`M22`, `M23`, `M24`, `M25`, `M26`, `M28`, `M30`, `M31`, `M32`.

All 17 T01 `E`/`C` candidates are API-feasible under these restrictions. The
17 T01 `F`/`R` rows remain blocked. The exact primitive set, `--release`, and
whether calls enter the first Java profile remain later decisions.

## 6. Dart analyzer feasibility record

### 6.1 Supported public boundary

Verdict: feasible for the T01 structural and exact-Boolean candidates through
the public analyzer package. The analyzer does not establish one target's
numeric runtime semantics.

| Fact | Minimum supported API |
| --- | --- |
| Resolved syntax | `AnalysisContextCollection`, `AnalysisContext.currentSession`, `AnalysisSession.getResolvedUnit`, successful `ResolvedUnitResult`, and its public `CompilationUnit` AST. |
| Symbols | Resolved public AST identifier `element` references and public `Element`/fragment APIs from the pinned analyzer element model. Null, multiply defined, dynamic, synthetic, or non-source targets reject. |
| Types and conversions | Public `Expression.staticType`, element types, `ResolvedUnitResult.typeProvider`, and `typeSystem`. The frontend still recognizes only exact built-in operations and rejects dispatch/coercion. |
| Control flow | No supported public analyzer CFG contract for this use. Build an exhaustive MPK-owned graph from the accepted public AST nodes; never import `package:analyzer/src/...`. |
| Source positions | Public AST `offset`/`end` values and `ResolvedUnitResult.lineInfo` for the exact content. |
| Diagnostics | `ResolvedUnitResult.diagnostics`, preserving diagnostic code/severity/span and using messages only under the pinned analyzer/locale. |
| Target and options | Explicit collection roots, `ResourceProvider`, `sdkPath`, package configuration, analysis-options bytes and includes, language versions, experiments, and a separately declared runtime/compiler target. Analyzer success does not infer native versus web behavior. |

### 6.2 Required input closure

The pin set contains the exact `analyzer` package version and its complete
locked transitive package graph, the exact Dart SDK and runtime hosting the
analyzer, SDK platform-library bytes, frontend bytes, every analyzed source,
and every dependency source. It also contains `pubspec.yaml`, the lockfile,
the generated package configuration, per-package language versions, and every
analysis-options file or included file as explicit bytes.

The `AnalysisContextCollection` is built over one finite
`ResourceProvider`-backed snapshot with explicit included roots and SDK path.
Package and options discovery is confined to that snapshot. No `dart pub`,
global pub cache, parent-directory package config, host SDK fallback, analysis
server, persistent driver state, generated source, macro/plugin execution,
network access, or user analysis options participate. The initial package set
may be empty beyond the SDK, but the SDK/platform libraries still remain
hashed inputs.

The invocation also pins the intended language/runtime/compiler tuple, such
as native versus web, even though T02 chooses neither. If the declared target
cannot be matched to the registered bundle and later semantic profile, the
frontend rejects.

### 6.3 Instability and semantic gaps

The analyzer package is a versioned tool API rather than a language standard.
Its changelog records substantial AST and element-model migrations. An exact
version and dependency lock are mandatory, every accepted node/element kind
must be exhausted, and each upgrade requires requalification. Internal
`package:analyzer/src` APIs, analysis-driver internals, analysis-server
protocols, cached summaries from undeclared sources, and plugin results are
outside the boundary.

Resolved AST types and elements are sufficient to recognize exact `bool`
operations, local flow, and a source-declared top-level/static call. For
`M27`, the call graph must be closed over exact source bodies, with no instance
or dynamic dispatch, closure capture, global/top-level/static-variable read,
optional default behavior, external body, import effect, or unresolved target.

The analyzer cannot make Dart `int` semantics target-independent. Native and
web representations differ, and the language documentation treats details as
target behavior. Consequently every numeric foundation row stays `NO-GO`
until a later phase selects one exact SDK/compiler/runtime target and freezes
its checked semantics. `M34` is only position-backed MPK contract attachment;
Dart metadata is not an accepted contract language.

### 6.4 Explicit row verdicts

`GO` (10): `M01`, `M07`, `M08`, `M10`, `M11`, `M12`, `M27`, `M29`, `M33`,
`M34`.

`NO-GO` (24): `M02`, `M03`, `M04`, `M05`, `M06`, `M09`, `M13`, `M14`,
`M15`, `M16`, `M17`, `M18`, `M19`, `M20`, `M21`, `M22`, `M23`, `M24`,
`M25`, `M26`, `M28`, `M30`, `M31`, `M32`.

All 10 T01 `E`/`C` candidates are API-feasible under these restrictions. The
24 T01 `F`/`R` rows remain blocked. Runtime target and numeric semantics remain
later choices.

## 7. TypeScript compiler feasibility record

### 7.1 Supported public boundary

Verdict: feasible for syntax, resolution, types, positions, diagnostics, and
MPK-owned structural flow, but not for proving erased runtime primitive or
call boundaries from TypeScript metadata alone.

| Fact | Minimum supported API |
| --- | --- |
| Resolved syntax | `createProgram` with explicit roots/options and a finite custom `CompilerHost`; public `Program`, `SourceFile`, `Node`, `SyntaxKind`, and `forEachChild`. |
| Symbols | `Program.getTypeChecker`, `TypeChecker.getSymbolAtLocation`, resolved signatures/aliases as needed, and public `Symbol`/declaration APIs. Error, external, merged, mutable, or ambiguous bindings reject. |
| Types and conversions | `TypeChecker.getTypeAtLocation`, signatures and type flags from the exact package. These are static TypeScript facts only; annotations and assertions erase and cannot prove a runtime value. |
| Control flow | No supported public CFG API. Build an exhaustive MPK-owned graph from accepted public AST nodes. Private `flowNode` fields, internal checker methods, and binder internals are forbidden. |
| Source positions | `Node.getStart`/`getEnd` and `SourceFile.getLineAndCharacterOfPosition` over exact source text. |
| Diagnostics | `getPreEmitDiagnostics` or the explicit option/global/syntactic/semantic diagnostic sets; normalize numeric code, category, file, and span. No emit is needed for feasibility analysis. |
| Target and options | An explicit `CompilerOptions` object, root list, custom host, exact declaration libraries, module-resolution results, package graph, emit target/module mode, and separately pinned ECMAScript runtime/host. |

### 7.2 Required input closure

The pin set contains the exact `typescript` npm package bytes, its package
metadata and dependency graph, exact Node.js host runtime, frontend bytes,
source bytes, compiler options, and every loaded declaration file. The exact
`lib.*.d.ts` set, `types` set, type roots, JSX mode, strictness flags, target,
module kind, module resolution mode, path/base/root mappings, package exports
conditions, package manifests/locks, JavaScript allowance/checking mode, and
runtime/host identity are explicit.

The frontend supplies a finite `CompilerHost` whose reads, directory queries,
case rule, canonical names, current directory, newline, default-library path,
and module/type-reference resolution are deterministic over the snapshot. It
does not delegate to `ts.sys`, search parent `node_modules`, discover visible
`@types`, read a user's `tsconfig`, run npm, load transformers/plugins, watch
files, reuse incremental state, or access the network. `ESNext` is not an exact
runtime semantic pin.

If later work emits JavaScript for differential testing, the exact TypeScript
emit, JavaScript runtime, host APIs, runtime flags, module loader, and entry
wrapper become additional hashed inputs. They are not selected here.

### 7.3 Instability and semantic gaps

The official compiler-API page currently says it describes TypeScript 6.0 and
earlier and warns that TypeScript 7.1 will have a different API. The package's
API-breaking-change record also requires exact-major qualification. A later
frontend must pin one API generation and treat migration to the new generation
as a semantic compiler upgrade, not a dependency refresh.

The public checker resolves TypeScript symbols and static types, but TypeScript
erases types and does not change JavaScript runtime behavior. It therefore
cannot prove that a caller supplies a Boolean primitive or that an annotated
function call has the closed runtime binding and module-initialization behavior
required by T01. T02 does not select a source `typeof` gate, generated wrapper,
or other runtime-entry mechanism. A Boolean literal is recognizable, but T01's
atomic `M01` row also includes parameter/local/result carrier boundaries; T02
does not split that row into a literal-only substitute. Until a runtime gate is
specified and tested, `M01`, `M08`, `M10`, `M11`, and `M27` are `NO-GO` even
though their VIR shapes were representability candidates in T01.

The remaining `GO` rows are structural: ordered evaluation (`M07`), explicit
local copying (`M12`), an MPK-owned definite-assignment graph (`M29`), complete
pin enforcement (`M33`), and source-position-backed contract attachment
(`M34`). They cannot form an accepted program without a `GO` carrier and all
other used rows. TypeScript decorators, JSDoc, and annotations are not trusted
contract semantics.

### 7.4 Explicit row verdicts

`GO` (5): `M07`, `M12`, `M29`, `M33`, `M34`.

`NO-GO` (29): `M01`, `M02`, `M03`, `M04`, `M05`, `M06`, `M08`, `M09`,
`M10`, `M11`, `M13`, `M14`, `M15`, `M16`, `M17`, `M18`, `M19`, `M20`,
`M21`, `M22`, `M23`, `M24`, `M25`, `M26`, `M27`, `M28`, `M30`, `M31`,
`M32`.

Five of the 10 T01 `E`/`C` candidates remain structurally API-feasible. The
other five (`M01`, `M08`, `M10`, `M11`, and `M27`) are downgraded because the
supported compiler boundary does not establish their required runtime facts.
All 24 T01 `F`/`R` rows remain blocked.

## 8. Python standard-library feasibility record

### 8.1 Supported public boundary

Verdict: feasible for syntax, lexical scope, positions, diagnostics, and
MPK-owned structural flow, but the standard library exposes neither resolved
runtime call targets nor enforced annotation types.

| Fact | Minimum supported API |
| --- | --- |
| Resolved syntax | Exact-version `ast.parse`/`compile` and the public `ast` node hierarchy. Python supplies a parsed AST, not a compiler-resolved operation tree; every accepted node and context is matched explicitly. |
| Symbols | `symtable.symtable` and public `SymbolTable`/`Symbol` scope classifications. They identify lexical local/free/global/nonlocal/imported roles, not the runtime object bound to a name. |
| Types and conversions | No official resolved-type API. Annotations are syntax/metadata and do not enforce runtime values. Any later exact built-in fact needs an MPK-owned runtime gate and closed host contract. |
| Control flow | No public CFG API. Build an exhaustive MPK-owned graph from accepted AST nodes and cross-check local binding classifications; unknown statements, expressions, comprehensions, handlers, or abrupt forms reject. |
| Source positions | Public `lineno`, `col_offset`, `end_lineno`, and `end_col_offset`; column offsets are UTF-8 byte offsets, so the exact source bytes and decoding must remain attached. |
| Diagnostics | `SyntaxError` and related compile/symbol-table failures with class, message under pinned locale/runtime, filename, line/offset/end fields, and source text; resource-limit termination is a deterministic rejection. |
| Target and options | The exact CPython executable/build is the grammar and analysis target. Pin parser/compiler flags, optimization level, future-feature inputs, implementation/platform identity, encoding, and isolated launch policy. `feature_version` is not a substitute for running the target version. |

### 8.2 Required input closure

The pin set contains one exact CPython executable and build configuration, its
runtime shared libraries and standard-library snapshot, frontend bytes, and
all source bytes. The initial analysis subset imports and executes no module,
third-party package, native extension, site customization, annotation, default
expression, decorator, or module body. Therefore package/import roots begin
empty rather than inheriting an environment.

The host launch is isolated (`-I`, with environment/user-site/unsafe-path
effects disabled), omits site initialization (`-S`), and prevents bytecode
writes (`-B`). It pins optimization to zero, source encoding handling, future
flags, UTF-8/locale behavior, logical filenames, recursion/memory/time limits,
and diagnostic normalization. `PYTHONPATH`, current-directory imports,
user/global site packages, startup hooks, environment-driven warning/debug
options, hash/locale defaults, and the network are outside the snapshot.

Analysis invokes parsing, compilation validation, and symbol-table creation
only. It never evaluates source, imports it, resolves annotations, calls
builtins, or writes `.pyc` files. A later runtime-differential harness would be
a separately specified pinned input boundary.

### 8.3 Instability and semantic gaps

The official `ast` documentation warns that the abstract grammar can change
with Python releases and that `feature_version` parsing is best effort rather
than equivalent to another runtime. The exact interpreter minor version must
therefore be pinned, all accepted node classes/fields exhausted, and every
upgrade requalified. Parsing and AST construction also require resource
limits because adversarial nesting or size can exhaust the process.

`ast` plus `symtable` can establish lexical structure and an MPK-owned local
flow graph. They cannot prove a runtime object's type, freeze globals or
builtins, resolve descriptor/special-method dispatch, or identify one stable
call target. Python annotations do not perform runtime enforcement. T02 does
not select a `type(x) is bool` entry gate, generated wrapper, sealed builtins
environment, or equivalent runtime mechanism. `True` and `False` literals are
recognizable, but T01's atomic `M01` row also includes parameter/local/result
carrier boundaries; T02 does not replace it with a literal-only row.
Consequently `M01`, `M08`, `M10`, `M11`, and `M27` are `NO-GO` despite their
T01 representability.

The five `GO` rows are structural: ordered evaluation (`M07`), explicit local
copying (`M12`), MPK-owned definite-assignment analysis (`M29`), complete pin
enforcement (`M33`), and position-backed attachment of a later-frozen MPK
contract (`M34`). A decorator, annotation, type comment, or docstring is not a
trusted contract by default, and these rows cannot form an accepted program
without a runtime carrier.

### 8.4 Explicit row verdicts

`GO` (5): `M07`, `M12`, `M29`, `M33`, `M34`.

`NO-GO` (29): `M01`, `M02`, `M03`, `M04`, `M05`, `M06`, `M08`, `M09`,
`M10`, `M11`, `M13`, `M14`, `M15`, `M16`, `M17`, `M18`, `M19`, `M20`,
`M21`, `M22`, `M23`, `M24`, `M25`, `M26`, `M27`, `M28`, `M30`, `M31`,
`M32`.

Five of the 10 T01 `E`/`C` candidates remain structurally API-feasible. The
other five (`M01`, `M08`, `M10`, `M11`, and `M27`) are downgraded because the
official analysis boundary cannot establish their required runtime facts. All
24 T01 `F`/`R` rows remain blocked.

## 9. Exit-gate audit

The five feasibility records cover the seven required fact categories and a
complete deterministic input closure. Their row lists contain each of
`M01` through `M34` exactly once per language:

| Language | T01 `E`/`C` candidates | T02 `GO` | T02 `NO-GO` | T01 candidates downgraded |
| --- | ---: | ---: | ---: | --- |
| C# | 18 | 18 | 16 | none |
| Java | 17 | 17 | 17 | none |
| Dart | 10 | 10 | 24 | none |
| TypeScript | 10 | 5 | 29 | `M01`, `M08`, `M10`, `M11`, `M27` |
| Python | 10 | 5 | 29 | `M01`, `M08`, `M10`, `M11`, `M27` |
| **Total** | **65** | **55** | **115** | **10** |

No T01 `F` or `R` cell was upgraded. Every `NO-GO` remains excluded, every
`GO` remains subject to all-row closure and later specification, and every
toolchain/runtime/profile choice remains unactivated. Thus each language has
a reviewed feasibility record and explicit go/no-go candidate list without
starting `MLANG-00-T03`, `MLANG-01`, or a production language phase.

## 10. Official API and runtime references

These primary references establish the research boundary. Versioned research
URLs do not select the eventual production pin.

### C# / Roslyn

- [Roslyn SDK overview](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/)
- [Roslyn compiler API model](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/compiler-api-model)
- [Working with Roslyn semantics](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-semantics)
- [`SemanticModel`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.semanticmodel)
- [`ControlFlowGraph.Create`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.flowanalysis.controlflowgraph.create)
- [`CSharpParseOptions`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.csharp.csharpparseoptions.-ctor)
- [`CompilationOptions`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.compilationoptions)
- [`Compilation.GetDiagnostics`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.compilation.getdiagnostics)
- [.NET reference assemblies](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies)
- [Roslyn breaking API changes](https://github.com/dotnet/roslyn/blob/main/docs/Breaking%20API%20Changes.md)
- [C# language specification: classes and initialization](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes)

### Java / `javac`

- [`jdk.compiler` module and Compiler Tree API](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/module-summary.html)
- [`JavacTask`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/JavacTask.html)
- [`Trees`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/Trees.html)
- [`SourcePositions`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/SourcePositions.html)
- [`JavaCompiler`](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/JavaCompiler.html)
- [`StandardJavaFileManager`](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/StandardJavaFileManager.html)
- [`DiagnosticCollector`](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/DiagnosticCollector.html)
- [Language-model utilities](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/lang/model/util/package-summary.html)
- [`javac` options](https://docs.oracle.com/en/java/javase/25/docs/specs/man/javac.html)
- [JLS 25 class and interface initialization](https://docs.oracle.com/javase/specs/jls/se25/html/jls-12.html#jls-12.4.1)
- [JDK migration guide on internal APIs](https://docs.oracle.com/en/java/javase/25/migrate/migrating-jdk-8-later-jdk-releases.html)

### Dart analyzer

- [`AnalysisContextCollection` 14.1.0](https://pub.dev/documentation/analyzer/14.1.0/dart_analysis_analysis_context_collection/AnalysisContextCollection-class.html)
- [`AnalysisSession` 14.1.0](https://pub.dev/documentation/analyzer/14.1.0/dart_analysis_session/AnalysisSession-class.html)
- [`ResolvedUnitResult` 14.1.0](https://pub.dev/documentation/analyzer/14.1.0/dart_analysis_results/ResolvedUnitResult-class.html)
- [Public analyzer AST API 14.1.0](https://pub.dev/documentation/analyzer/14.1.0/dart_ast_ast/)
- [`Expression.staticType` 14.1.0](https://pub.dev/documentation/analyzer/14.1.0/dart_ast_ast/Expression/staticType.html)
- [Dart number representations by target](https://dart.dev/resources/language/number-representation)
- [Dart analysis options](https://dart.dev/tools/analysis)
- [Dart package layout and package configuration](https://dart.dev/tools/pub/package-layout)
- [Dart package versioning and lockfiles](https://dart.dev/tools/pub/versioning)
- [Analyzer changelog](https://github.com/dart-lang/sdk/blob/main/pkg/analyzer/CHANGELOG.md)

### TypeScript / ECMAScript

- [Using the TypeScript Compiler API](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API)
- [TypeScript API breaking changes](https://github.com/microsoft/TypeScript/wiki/API-Breaking-Changes)
- [TypeScript compiler binder and internal flow graph](https://github.com/microsoft/TypeScript/wiki/Codebase-Compiler-Binder)
- [Type erasure and unchanged JavaScript runtime behavior](https://www.typescriptlang.org/docs/handbook/typescript-from-scratch.html)
- [TypeScript `lib` option](https://www.typescriptlang.org/tsconfig/lib.html)
- [TypeScript `moduleResolution` option](https://www.typescriptlang.org/tsconfig/moduleResolution.html)
- [TypeScript `types` option](https://www.typescriptlang.org/tsconfig/types.html)
- [TypeScript `target` option](https://www.typescriptlang.org/tsconfig/target.html)
- [ECMAScript language specification](https://tc39.es/ecma262/)

### Python / CPython

- [Python 3.14 `ast`](https://docs.python.org/3.14/library/ast.html)
- [Python 3.14 `symtable`](https://docs.python.org/3.14/library/symtable.html)
- [Python 3.14 command-line isolation and options](https://docs.python.org/3.14/using/cmdline.html)
- [Python 3.14 execution model](https://docs.python.org/3.14/reference/executionmodel.html)
- [Python 3.14 annotations](https://docs.python.org/3.14/reference/compound_stmts.html#annotations)
