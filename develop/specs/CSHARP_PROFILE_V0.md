# C# Scalar Profile v0 Specification

Status: normative, frozen, and active. `CSHARP-02-T20` atomically activated the
profile on 2026-08-30; `JAVA-03-T10` retained it unchanged in the installed
revision-3 Go/Rust/C#/Java release on 2026-09-03. Historical future-tense gate
text below records the required freeze-to-activation path and does not mean the
profile is currently inactive.

This specification is the frozen `MLANG-01-T03` C# package. It consumes the
closed successor mechanism in `SEMANTIC_PROFILE_REGISTRY_V1.md` without
changing that mechanism. `CSHARP-02` implemented this package and performed
the whole-release atomic migration defined by the registry specification.

## 1. Scope, trust, and profile identity

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means no
VIR, source map, source manifest, VC, certificate candidate, policy evidence,
or AI request is published from the rejected source context.

C# source, contracts, Roslyn, .NET, `csharp2vir`, registry bytes, VIR, maps,
manifests, VCs, policy/evidence documents, differential runs, and AI output are
untrusted helper data. Certificate v0, both source-free checker inputs and
acceptance rules, and the four axiom categories remain unchanged. There is no
C# semantics axiom and no compiler result is proof evidence.

The exact profile identity is:

| Identity | Exact value |
| --- | --- |
| source language | `csharp` |
| semantic profile | `mpk.csharp.scalar.v0` |
| parameter schema | `mpk.semantic_parameters.csharp_scalar.v0` |
| selection schema | `mpk.selection.csharp_methods.v0` |
| contract sidecar schema | `mpk.csharp.contract.v0` |
| toolchain-input schema | `mpk.csharp.toolchain_inputs.v0` |
| limit profile | `mpk.csharp.limits.v0` |
| execution host | `mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0` |
| runtime layout | `mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0` |

Unknown aliases, case variants, additional parameter members, another target,
and another language/profile pairing reject. This profile owns no language
value other than `csharp` and does not authorize Java or any later language.

### 1.1 Registry revision 2

The C# entry has the exact nine contract IDs below:

| Contract field | Contract ID |
| --- | --- |
| `ai` | `mpk.profile.ai.csharp_scalar.v0` |
| `evidence` | `mpk.profile.evidence.csharp_scalar.v0` |
| `frontend` | `mpk.profile.frontend.csharp_scalar.v0` |
| `manifest` | `mpk.profile.manifest.csharp_scalar.v0` |
| `policy` | `mpk.profile.policy.csharp_scalar.v0` |
| `release` | `mpk.profile.release.csharp_scalar.v0` |
| `source_map` | `mpk.profile.source_map.csharp_scalar.v0` |
| `vc` | `mpk.profile.vc.csharp_scalar.v0` |
| `vir` | `mpk.profile.vir.csharp_scalar.v0` |

Its immutable entry hash is `d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac`.

Registry revision 2 contains exactly the new C# entry followed by the
byte-for-byte revision-1 Go and Rust entries, sorted by `(source_language,
semantic_profile)`. Its root hash is `6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75`. Its complete
identity, canonical sizes, hashes, predecessor proof, and old/new rejection
vectors are frozen in
`develop/specs/vectors/semantic-profile-registry-v2.json`.

Revision 2 is a design input for `CSHARP-02`, not an active installed root.
Membership alone does not activate C#; all section-14 gates are required.

## 2. Semantic parameters and source selection

### 2.1 Exact semantic parameters

The complete `SemanticParametersEnvelope` is:

```json
{"schema":"mpk.semantic_parameters.csharp_scalar.v0","value":{"check_overflow_default":false,"documentation_mode":"none","language_version":"14.0","nullable_context":"disable","optimization":"release","platform":"x64","pointer_width":64,"preprocessor_symbols":[],"source_kind":"regular","target_framework":"net10.0","target_id":"linux-x64","unsafe":false}}
```

The false default is a compiler-session assertion, not permission to use an
implicit unchecked context. Section 5.4 requires explicit `checked` or
`unchecked` syntax for every admitted source arithmetic or nonidentity explicit
conversion whose behavior depends on overflow context. Source outside an
explicit context rejects even though the compiler default is pinned false.

`preprocessor_symbols` is exactly empty. No symbol, target, nullable, language,
optimization, documentation, unsafe, or overflow option comes from source,
environment, response file, project file, editor configuration, or host state.

### 2.2 Exact selection envelope

`SelectionEnvelope.schema` is exactly `mpk.selection.csharp_methods.v0`.
Its `value` has exactly:

| Field | Rule |
| --- | --- |
| `compilation` | 1..64 ASCII bytes, `[a-z][a-z0-9]*([._-][a-z0-9]+)*` |
| `sources` | 1..256 strictly increasing unique portable paths under `src/`, each ending `.cs` |
| `contracts` | 1..128 strictly increasing unique portable paths under `contracts/`, each ending `.json` |
| `methods` | 1..32 strictly increasing unique canonical method IDs, each 1..1,024 ASCII bytes |

The immutable capture contains exactly the listed regular files and their
implied directories. An unlisted entry, listed missing file, link, hard-link
alias, reparse point, special file, case-fold collision, or normalized-path
collision rejects. There is no project/solution, glob, directory default,
current-directory discovery, response file, or implicit contract path.

A canonical method ID is:

```text
Namespace.StaticType::Method(Type,...)->Type
```

Namespace, type, and method components are nonempty ASCII C# identifiers whose
first byte is `[A-Za-z_]` and remaining bytes are `[A-Za-z0-9_]`; namespace
components are dot-separated. The type tokens are exactly `bool`, `i32`,
`u32`, `i64`, and `u64`. There is no whitespace, alias spelling, nullable
suffix, generic arity, assembly qualification, return-type inference, or
Unicode normalization. The signature is included even when the source has no
overload. Every selected method resolves exactly once.

The vector fixture's exact selection is the normative worked example.

## 3. Hermetic Roslyn and .NET input closure

### 3.1 Frozen versions

The build host, execution host, and analyzed target are Linux x86-64. Build
and execution select the existing exact GNU glibc 2.27 host/runtime-layout IDs
in section 1, minimum kernel ABI `6.4.0`, interpreter
`/lib64/ld-linux-x86-64.so.2`, and only the .NET archive root plus
`/lib/x86_64-linux-gnu` as native-library roots. The frozen versions are:

| Input | Exact identity |
| --- | --- |
| .NET SDK build closure | `10.0.400`, Linux x64 |
| .NET runtime execution closure | `10.0.11`, `linux-x64` |
| target framework | `net10.0` |
| reference pack | `Microsoft.NETCore.App.Ref` `10.0.11` |
| Roslyn C# package | `Microsoft.CodeAnalysis.CSharp` `5.6.0` |
| Roslyn common package | `Microsoft.CodeAnalysis.Common` `5.6.0` |
| Roslyn analyzer dependency | `Microsoft.CodeAnalysis.Analyzers` `5.3.0`, build-only and never executed |
| Roslyn source commit | `c0573ed0a7dc3e3b4d2e70da47f97cc51a35524f` |
| C# language version | exactly `14.0`, never `default`, `latest`, or `preview` |

The complete `mpk.csharp.toolchain_inputs.v0` descriptor, archive URLs, byte
sizes, SHA-256 values, selected managed-assembly hashes, and reference-set hash
are frozen in `vectors/csharp-profile-v0.json`. Its self-hash under
`MPK-CSHARP-TOOLCHAIN-INPUTS-0.1` is
`d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f`.

The SDK and runtime tarballs are extracted into fresh empty roots with numeric
owner/group ignored and mtime ignored. Their pinned archives contain exactly
4,907 and 193 regular files, respectively, plus 724 and 7 directories; neither
contains a link or another entry type. Extraction rejects absolute, parent,
backslash, duplicate, case-fold-colliding, link, device, or unknown-type
entries. Low nine permission bits are preserved: the SDK has 4,075 mode-0644,
811 mode-0744, and 21 mode-0755 files plus 724 mode-0755 directories; the
runtime has 177 mode-0644 and 16 mode-0755 files plus 7 mode-0755 directories.
Set-ID and sticky bits reject. The four NuGet packages are exact ZIP byte
streams; extraction rejects
absolute, parent, backslash, duplicate, case-fold-colliding, encrypted, link,
device, or unsupported-compression entries. An archive hash is checked before
extraction and every retained non-directory entry is regular. ZIP external
mode bits are not authority; retained directories are installed mode 0755 and
retained files mode 0644.

The production managed Roslyn projection contains only:

- `lib/net10.0/Microsoft.CodeAnalysis.dll` from the common package; and
- `lib/net10.0/Microsoft.CodeAnalysis.CSharp.dll` from the C# package.

Their vector `runtime_path` values are relative to `/mpk`, so both assemblies
reside in the read-only `/mpk/frontend` snapshot beside `csharp2vir`.

Workspace, features, scripting, Visual Basic, MSBuild, NuGet client, analyzer,
source-generator, compiler-server, and interactive assemblies are absent. The
analyzer package is retained only as a checked build-graph input required by
the package metadata; none of its DLLs or rules may load or run.

The target reference projection is every regular `.dll` directly under
`ref/net10.0/` in the pinned reference-pack archive, sorted by path. It contains
exactly 167 assemblies totaling 6,046,008 bytes. The canonical array of
`path`, `size_bytes`, and raw `sha256` records hashes under
`MPK-CSHARP-REFERENCE-INVENTORY-0.1` to
`30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad`.
The archive projection is installed read-only at
`/mpk/toolchain/reference-pack`; each metadata-reference path is that root plus
the exact inventory path.
No runtime implementation assembly, trusted-platform-assembly list, GAC,
Windows targeting pack, ASP.NET pack, or host reference substitutes.

### 3.2 Compiler API boundary

`csharp2vir` is one managed frontend process. It loads the two pinned Roslyn
assemblies in-process and uses only these public API families:

- `SourceText.From` over the already strict-decoded source;
- `CSharpSyntaxTree.ParseText` with explicit `CSharpParseOptions`;
- `SyntaxTree.GetDiagnostics` before compilation diagnostics;
- `CSharpCompilation.Create`, `CSharpCompilationOptions`, and explicit
  `MetadataReference.CreateFromFile` over validated immutable paths in the
  sealed reference projection;
- `Compilation.GetDiagnostics`, `GetSemanticModel`, `GetDeclaredSymbol`,
  `GetSymbolInfo`, `GetTypeInfo`, `ClassifyConversion`, and `GetOperation`;
- public `ISymbol`, `IMethodSymbol`, `ITypeSymbol`, `IOperation`, operation
  interfaces, and
  `ControlFlowGraph.Create(IMethodBodyOperation, CancellationToken.None)`; and
- `SyntaxTree`, `Location`, and exact `TextSpan` values over the captured
  `SourceText`.

There is no private driver or subordinate `csc` process. No assembly emit,
module initializer, analyzer, generator, attribute constructor, or candidate
code executes during lowering. `MSBuildWorkspace`, command-line
project loading, build servers, compiler internals, bound trees, reflection
over nonpublic members, and serialized compiler caches are forbidden.
Every listed public call that accepts a cancellation token receives
`CancellationToken.None`; `GetSemanticModel` uses
`ignoreAccessibility=false`. No speculative semantic model or nullable-flow
snapshot is an alternate source of facts.

### 3.3 Exact parse and compilation session

Parse options are exactly `LanguageVersion.CSharp14`, regular source kind,
documentation mode none, an empty symbol array, and an empty feature map.
`SourceText.From(string, Encoding, SourceHashAlgorithm)` receives the exact
decoded string, `new UTF8Encoding(false, true)`, and `Sha256`.
`CSharpSyntaxTree.ParseText(SourceText, CSharpParseOptions, string,
CancellationToken)` receives that text, the exact options, the unique
selection path, and `None`. Trees are passed to `CSharpCompilation.Create` in
the exact stored `selection.sources` order; contracts never become syntax
trees.

Compilation options are exactly DLL output, x64 platform, Release optimization,
overflow checking false, nullable disabled, unsafe false, deterministic true,
concurrent build false, public metadata import, warning level 4, general
diagnostic option error, suppressed-diagnostic reporting false, empty
specific-diagnostic options, lower-version supersession false, no global
usings, default assembly-identity comparison, and null source, metadata, XML,
strong-name, and syntax-tree-options resolvers/providers. Module/main names and
signing inputs are null/empty, `scriptClassName` is the inert API default
`Script`, and public/delay signing are false/null. Every metadata reference has
kind assembly, no alias, `embedInteropTypes=false`, and no documentation
provider. The complete getter-level object is frozen by `compiler_session` in
the vector. The assembly name equals `selection.compilation`. References are
the complete sorted 167-assembly projection and no compilation reference.

The caller's release phase requires Roslyn package/assembly version and commit
markers and the reference inventory to match the frozen descriptor before the
child starts. Any changed public enum case,
operation shape, CFG region/branch shape, implicit conversion, symbol origin,
or diagnostic behavior is `CSHARP_TOOLCHAIN_ADAPTER`; it is not accepted by
falling back to syntax text.

### 3.4 Build and execution isolation

The SDK is build-only. Candidate analysis runs the release-inventory native
`dotnet` host against the fixed `csharp2vir.dll`, `.deps.json`,
`.runtimeconfig.json`, Roslyn DLLs, reference pack, and runtime 10.0.11 tree.
The runtime configuration names `net10.0`, framework
`Microsoft.NETCore.App` version `10.0.11`, and `rollForward = Disable`.

The exact process, working directory, runtime/deps arguments, ordered frontend
argument template, repeated selection expansion, standard streams, and closed
environment are `launcher_contract` in the vector. The program is exactly
`/mpk/toolchain/dotnet/dotnet`; it runs `exec` with the frozen deps/runtime
configuration and `--fx-version 10.0.11`, then
`/mpk/frontend/csharp2vir.dll`. Selection arrays expand in stored order and
all bundle/hash placeholders are values from an already validated successor
release tuple. They are never caller paths. Expanded argv including NUL bytes
is checked against `frontend_argument_bytes` before launch.

The environment is closed by `mpk.csharp.frontend_environment.v0`:

```text
DOTNET_ROOT=/mpk/toolchain/dotnet
DOTNET_MULTILEVEL_LOOKUP=0
DOTNET_NOLOGO=1
DOTNET_CLI_TELEMETRY_OPTOUT=1
DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1
DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1
COMPlus_ReadyToRun=0
DOTNET_TieredCompilation=0
DOTNET_TieredPGO=0
HOME=/mpk/empty-home
NUGET_PACKAGES=/mpk/empty-nuget
NUGET_HTTP_CACHE_PATH=/mpk/empty-nuget-http
NUGET_PLUGINS_CACHE_PATH=/mpk/empty-nuget-plugins
TMPDIR=/mpk/tmp
LANG=C.UTF-8
LC_ALL=C.UTF-8
TZ=UTC
PATH=/nonexistent
```

No other variable is inherited. `/mpk/source` is read-only captured input;
`/mpk/toolchain` and the frontend snapshot are read-only; `/mpk/tmp` is the
bounded private tmpfs. Network, credentials, user/profile directories,
machine-wide .NET, NuGet caches, first-run mutation, restore, probing outside
the .NET archive root or selected native runtime layout, and ambient dynamic
native-library search are unavailable. The release phase validates every
native dependency against those two inventoried roots before launch.

## 4. Source transport and compilation closure

Every source is nonempty strict UTF-8 without BOM, NUL, CR, lone surrogate,
or noncharacter. Lines end only LF; a final LF is required. Unicode is allowed
in comments, but every namespace/type/method/parameter/local identifier in the
accepted closure is ASCII under section 2.2. Its source token spelling must
equal the normalized identifier: verbatim `@` spelling and Unicode escapes in
an identifier reject. String, character, interpolated, raw-string,
UTF-8-string, and `nameof` expressions reject.

The source tree contains only the selected `src/**/*.cs` and
`contracts/**/*.json` files. `.csproj`, `.sln`, `.slnx`, `global.json`,
`Directory.Build.*`, `Directory.Packages.props`, `packages.lock.json`,
`.editorconfig`, `.globalconfig`, response files, generated files, resources,
additional files, and analyzer configuration reject before Roslyn starts.

All selected source trees enter one compilation. The conservative method
closure starts at `selection.methods` and follows every resolved source static
call in each accepted body, including a source-dead branch. Every closure
method is subset-, purity-, initialization-, and contract-checked, and every
ordinary method declared in a selected source must belong to that closure. An
unrelated or unreachable source method rejects rather than becoming ignored
compilation content. Recursion and cycles reject. Reachable `CallStatic` edges
alone determine successor VIR declaration dependencies; all closure members still
emit as standalone functions in the successor VIR's canonical callee-first
order.

## 5. Closed C# source and semantic subset

### 5.1 Declarations

Accepted compilation members are block-scoped or file-scoped namespace
declarations, nonnested nonpartial `static class` declarations, and ordinary
static methods. A class is explicitly `public` or `internal` and has no other
modifier. A selected root is `public static`; a dependency may be `private
static` or `internal static`. Every method is nongeneric, nonextension,
nonpartial, nonextern, nonsynthetic, nonasync, noniterator, nonunsafe, has a
block body, explicit accepted parameter/result types, plain value parameters,
and one non-void result. Parameters are immutable in the profile.

An accepted static class has no field, constant, property, event, constructor,
operator, conversion, indexer, nested type, attribute, interface, or base-list
syntax. Its Roslyn static-constructor and module-initializer sets are empty.
This is the complete inert initialization proof; `beforefieldinit` or absence
of a currently observed initializer is not used as a shortcut.

Using directives, extern aliases, global statements, records, structs, enums,
interfaces, delegates, classes with instances, top-level programs, attributes,
directives, aliases, and generated syntax reject.

### 5.2 Types, literals, and values

The complete source type mapping is:

| C# source type | VIR type |
| --- | --- |
| `bool` | `{"kind":"bool"}` |
| `int` | `{"kind":"bv","width":32,"signed":true}` |
| `uint` | `{"kind":"bv","width":32,"signed":false}` |
| `long` | `{"kind":"bv","width":64,"signed":true}` |
| `ulong` | `{"kind":"bv","width":64,"signed":false}` |

The source type syntax is exactly the keyword token in the first column.
Framework names and aliases such as `System.Int32`, `global::System.Int32`, or
`Int32` reject even when Roslyn resolves them to the same special type.

`sbyte`, `byte`, `short`, `ushort`, and `char` reject in v0 so implicit numeric
promotion cannot silently change an admitted carrier. `nint`, `nuint`,
`Int128`, `UInt128`, arbitrary integers, floats, `decimal`, enums, tuples,
arrays, structs, references, nullable values, pointers, function pointers,
`dynamic`, and every user type reject as values.

Accepted literals are `true`, `false`, and base-10 integer tokens without
separators. An `int` token has no suffix, `uint` uses uppercase `U`, `long`
uses uppercase `L`, and `ulong` uses uppercase `UL`. Leading `+`, lowercase or
reordered suffixes, hexadecimal/binary tokens, separators, and redundant
suffixes reject. A leading `-` is accepted only for an in-range `int`/`long`
literal under Roslyn's exact resolved constant value; it emits one `Const`, not
`bv_neg`. Composite constant-valued arithmetic and converted constant operands
reject so lowering never depends on compiler folding.

### 5.3 Statements and control flow

Accepted statements are non-const explicit-type local declarations with
exactly one variable declarator and one initializer, simple assignment to a
previously declared local, expression statements whose value operation is
otherwise accepted, blocks, `checked`/`unchecked` blocks, `if`/`else`, early
`return`, and one final return. An `if` may omit `else`, as in the early-return
vector. Local names are unique in a method; shadowing,
`var`, `const`, multiple declarators, deconstruction, uninitialized locals,
parameter assignment, and compound/increment assignment reject.

Accepted expressions are literals, parameter/local references, parentheses,
the exact operators in section 5.4, explicit checked/unchecked expressions,
accepted casts, `?:` with exact same-type branches, and accepted direct static
calls. `&&` and `||` lower to branch graphs and evaluate the right operand only
on the C# path. Operands and call arguments lower left-to-right exactly once.

Loops, switch/pattern flow, `goto`, labels, `break`, `continue`, `yield`, local
functions, lambdas, query expressions, object/collection expressions, member
access other than an accepted static call target, conditional access, null
coalescing, throw expressions, and unknown syntax reject.

Definite-assignment success from Roslyn is necessary but not sufficient. The
adapter proves one dominating emitted definition for every local read on every
incoming MPK CFG edge. Phi-like joins become explicit block parameters and
edge arguments. An unprovable or compiler-created default rejects.

### 5.4 Operators and explicit overflow policy

The complete operator mapping is:

| C# form | Resolved types | VIR operation/rule |
| --- | --- | --- |
| `!x` | `bool` | `bool_not` |
| `x == y`, `x != y` | identical accepted type | `eq`, `not_eq` |
| `<`, `<=`, `>`, `>=` | identical accepted integer type | signed/unsigned comparison from type |
| `checked(x + y)`, `-`, `*` | identical accepted integer type | `bv_add/sub/mul` plus exact `integer_no_overflow` |
| `unchecked(x + y)`, `-`, `*` | identical accepted integer type | same BV operation, no check |
| `checked(-x)` / `unchecked(-x)` | `int` or `long` | `bv_neg`, with/without `integer_no_overflow(neg,signed=true)` |
| `x / y`, `x % y` inside either explicit context | identical accepted integer type | signed/unsigned div/rem checks below |
| `~x`, `x & y`, `x \| y`, `x ^ y` | identical accepted integer type | `bv_not/and/or/xor` |
| `x << n` | accepted integer `x`, exact `int` count | mask count with `31`/`63`, then `bv_shl` |
| `x >> n` | accepted integer `x`, exact `int` count | same mask, then `bv_ashr` or `bv_lshr` |

Arithmetic operands must already have identical accepted resolved types with no
implicit binary numeric promotion. Thus mixed-width/mixed-signed operations
reject even when C# defines a conversion. Unary plus, Boolean `&`/`|`/`^`,
`>>>`, compound operators, increments, lifted/nullable operators, dynamic or
user-defined operators, and methods such as `Math.*` reject.

Every admitted `+`, `-`, `*`, signed negation, division, remainder, and
nonidentity explicit conversion is enclosed by source-written `checked` or
`unchecked` syntax. The nearest enclosing context determines the profile rule
where C# exposes one and is still required when the value rule is context-
independent. Under the pinned Roslyn API, `IsChecked` is true for checked
add/subtract/multiply, signed negate, and divide, and false for their unchecked
forms. Remainder reports `IsChecked=false` in both contexts, so its explicit
context is proven from syntax ancestry rather than inferred from that flag.
The 12 `roslyn_checked_state_cases` freeze these results. A default-context
operation, another API result, or a context inferred only from compilation
options rejects.

Signed and unsigned division/remainder always require `divisor_nonzero`.
Signed forms additionally require operation-qualified
`signed_divrem_representable`, including in `unchecked` context, so the profile
never depends on the C# implementation-defined `MIN / -1` result. Shift counts
are masked and therefore require no shift safety check. Checks are ordered by
the successor VIR rule:

```text
integer_no_overflow, divisor_nonzero, signed_divrem_representable
```

Within overflow, operation order is add, sub, mul, neg; within representable it
is div, rem. Missing, extra, duplicate, or reordered checks reject.

### 5.5 Conversions

Identity conversions among the five accepted types emit no instruction. The
only accepted implicit nonidentity conversions are `int -> long`, `uint ->
long`, and `uint -> ulong`; each emits `Convert`.

Every source/destination pair among `int`, `uint`, `long`, and `ulong` has a
built-in explicit conversion. A nonidentity explicit conversion is accepted
only in a source-written `unchecked` context and emits `Convert`. Widening uses
source signedness for sign/zero extension, narrowing retains low bits, and an
equal-width signedness change preserves all bits before assigning the declared
destination type.

Every nonidentity checked conversion rejects with
`CSHARP_SUBSET_CHECKED_CONVERSION`, even for a statically in-range value. A
nonidentity explicit conversion in default context also rejects. Constant,
user-defined, dynamic, boxing, unboxing, nullable, reference, pointer, enum,
string, and runtime conversion forms reject. The profile adds no checked-
conversion safety kind or foundation.

### 5.6 Calls, purity, and abrupt completion

An accepted invocation resolves to one source-declared method in the captured
compilation, has exact parameter/result types, no optional/default/params/named
argument behavior, and belongs to the acyclic conservative closure. Arguments
are positional and left-to-right. The emitted `CallStatic` repeats the
recomputed normalized callee contract hash. Overload resolution must select
one exact canonical method ID; ambiguity or metadata/external target rejects.

An accepted method reads only parameters and locals, writes locals only, and
calls only accepted closure members. It has no allocation, field/property/
event access, static/global state, reflection, P/Invoke, unsafe operation,
clock, random, environment, I/O, synchronization, volatile/atomic behavior,
thread, task, dynamic dispatch, or unknown effect.

`throw`, `try`, `catch`, `finally`, filters, `lock`, `using`, `fixed`, unsafe
flow, null failure, cast failure, type initialization, and all general abrupt
completion reject. Arithmetic abrupt conditions are represented only by the
required checks above and by contracts that require `panic: forbidden` and
`termination: total`. A source compiler diagnostic, unsupported construct, or
missing/malformed required check cannot produce frontend success. A
well-formed emitted check remains a VC obligation and cannot contribute to
acceptance until the source-free proof path discharges it.

### 5.7 Closed rejected semantic rows

The profile admits the matrix rows `M01`, `M02`, `M07` through `M14`, `M16`,
`M18`, `M19`, `M21`, `M27`, `M29`, `M33`, and `M34` under this specification's
restrictions. It rejects exactly the candidate-ineligible rows `M03` through
`M06`, `M15`, `M17`, `M20`, `M22` through `M26`, `M28`, and `M30` through
`M32`. The 34-row ownership vector is closed; no row is omitted or assigned
twice.

## 6. C# contract sidecars

Each closure method has exactly one selected sidecar. The root has exactly
`schema`, `semantic_profile`, `method`, `requires`, `ensures`, `modifies`,
`abrupt_completion`, and `termination`. Values are:

- `schema = mpk.csharp.contract.v0`;
- `semantic_profile = mpk.csharp.scalar.v0`;
- `method` equals the canonical method ID;
- `requires` is ordered and may be empty;
- `ensures` is ordered and nonempty;
- the combined clause count is 1..64;
- `modifies = []`;
- `abrupt_completion = forbidden`; and
- `termination = total`.

Contracts are strict JSON and closed. Duplicate, missing, unused, wrong-method,
wrong-profile, unresolved-variable, or multiply selected sidecars reject.
Attributes, XML documentation, comments, Code Contracts, nullable annotations,
and analyzer results are not contracts.

Expressions form one closed recursive union:

| Branch | Exact JSON members |
| --- | --- |
| parameter | `{"parameter":Name}` |
| result | `{"result":0}` |
| Boolean literal | `{"bool":Boolean}` |
| integer literal | `{"int":{"decimal":CanonicalDecimal,"type":IntType}}` |
| unary | `{"op":UnaryOp,"args":[Expr]}` |
| Boolean n-ary | `{"op":"and" or "or","args":[Expr,...]}` with 2..64 operands |
| binary | `{"op":BinaryOp,"args":[Expr,Expr]}` |

`IntType` is `i32`, `u32`, `i64`, or `u64`; its literal must fit.
`CanonicalDecimal` is `0` or an optional `-` followed by a nonzero decimal
digit and zero or more decimal digits. It has no leading plus, leading zero,
negative zero, separator, radix prefix, whitespace, or suffix; an unsigned
literal cannot be negative. Parameter names resolve exactly once in the
method signature. Every `requires` and `ensures` expression must type-check as
Boolean under the successor `VirContract` rules after the source types are
mapped, and operators admit no implicit conversion or mixed operand type.
`UnaryOp` is `not`, `bv_neg`, or `bv_not`. `BinaryOp` is exactly `eq`,
`not_eq`, `signed_lt`, `signed_le`, `signed_gt`, `signed_ge`, `unsigned_lt`,
`unsigned_le`, `unsigned_gt`, `unsigned_ge`, `bv_add`, `bv_sub`, `bv_mul`,
`bv_and`, `bv_or`, `bv_xor`, `bv_shl`, `bv_ashr`, or `bv_lshr`. Their operand
and result typing is exactly the successor VIR contract union. Division,
remainder, conversion, field/index, call, source operator spelling, and
arbitrary expressions reject.

Normalization renames a parameter by declaration position to `argN`, retains
result and Boolean atoms, maps the typed integer to the VIR `int` atom, maps a
unary `args[0]` to `value`, and maps binary `args[0]/args[1]` to `lhs/rhs`.
It performs no folding, commuting, reassociation, deduplication, or implicit
conversion. The normalized successor `VirContract` uses
`unit_id = selection.compilation`, `function_id = sidecar.method`, carries the
complete `SemanticContext`, and fixes `panic = forbidden`,
`termination = total`, and `loops=[]`. Its hash uses `MPK-CONTRACT-1.0`
exactly. Raw sidecar bytes remain a distinct manifest input.

## 7. Roslyn operation and CFG adapter

`SemanticModel.GetOperation` on the exact `MethodDeclarationSyntax` must
return `IMethodBodyOperation`; that operation is the sole CFG root and the
`IMethodBodyOperation` overload with `CancellationToken.None` is the sole
allowed `ControlFlowGraph.Create` call. The adapter accepts only public
operations corresponding to method body,
block, variable declaration/initializer, literal, parameter/local reference,
simple assignment, parenthesized/conversion, unary, binary, conditional,
invocation, return, and branch flow required by section 5. Compiler-generated
flow captures/references are accepted only in the frozen conditional,
short-circuit, and argument-evaluation patterns enumerated by the vectors;
they emit no independently addressable VIR value.

Every `IOperation` has `IsInvalid=false`, no dynamic type, no error type, source
syntax in one captured tree, and only expected implicit conversions. Every
symbol is source-declared in the captured compilation or one exact predefined
type/operator from the pinned reference set. Unknown operation kinds,
implicit object creation, invalid nodes, metadata method bodies, synthesized
members, anonymous functions, and an unexpected child/order reject.

`ControlFlowGraph.Create` must succeed for every closure method. The graph has
one entry and exit, no exception/finally/filter region, no unreachable
compiler block containing a source closure member, and only regular branches,
conditional branches, and returns corresponding to accepted syntax. MPK emits
its own canonical blocks in breadth-first false-then-true successor order;
Roslyn ordinals and capture IDs are never public IDs. The source `IOperation`
and CFG views must account for the same operations exactly once.

Arguments become `arg0`, `arg1`, ...; result is `result0`; source locals become
`local0`, `local1`, ... by declaration span; compiler temporaries become `t0`,
`t1`, ... by canonical block/instruction order. Blocks become `bb0`, `bb1`, ...
by the shared traversal. Equivalent graph rewrites are not accepted by pattern
similarity; an adapter pattern change requires section-14 upgrade review.

## 8. VIR, source-map, manifest, VC, policy, evidence, and AI mapping

The successor VIR module has one `SemanticContext`, one unit whose ID/name is
the selection compilation, and profile-specific validation selected by the C#
entry hash. Every contract repeats the same context. Mixed-language units and
another entry's contract ID reject.

All functions, instructions, and terminators use faithful source origins; the
C# synthetic-reason allowlist is empty. Roslyn spans are zero-based UTF-16 code
unit offsets. The frontend decodes the exact source once, builds a checked
UTF-16-boundary-to-UTF-8-byte table, and maps both span endpoints. A boundary
inside a surrogate pair, out-of-range span, external tree, zero-length node,
or nearest-token substitution rejects. Public maps store only UTF-8 bytes as
specified by the successor source-map schema.

Line and column values are never serialized or accepted as input. When the
adapter cross-checks a Roslyn line position, the line is the zero-based count
of LF bytes before the endpoint and the column is the zero-based count of
UTF-16 code units since the preceding LF (or file start). Both endpoints must
equal Roslyn's position for the same captured `TextSpan`; a mismatch is
`CSHARP_TOOLCHAIN_ADAPTER`. The accepted source-map vectors freeze these
internal line/column results while the public origin remains path plus UTF-8
byte range only.

The six `source_map_cases` in the vector freeze ASCII, BMP, surrogate-pair,
split-surrogate, zero-length, and out-of-range endpoint behavior. They are
required implementation vectors, not explanatory examples.

The source manifest input kinds are exactly `source` and `contract`; unit kind
is `compilation`. Project, lock, generated, analyzer, additional, and resource
inputs are impossible in the selection. It repeats the complete registry,
semantic, selection, release, toolchain, runtime, reference, Roslyn, VIR, and
map identities required by the successor schemas.

The VC contract accepts only the existing Bool/BV foundation, normalized
contract operators, `CallStatic` WP, and exact safety checks in section 5. It
adds no operation, theory primitive, or axiom category. The policy contract
registers exactly strategy `payment-policy-csharp-alpha`, checker profile
`mvp-strict`, and axiom profile `mvp-theory`. Evidence requires both checkers,
retains certificate-only proof authority, and uses the structured C# recipe.

The AI contract uses display label `C#`, redaction profile `minimal-v1`, and no
source access. It may project only validated evidence/context/selection fields
allowed by the successor AI schema. `proof_authority` is false. C# source,
contract text, compiler diagnostics, and identifiers outside the already
sanitized evidence projection never enter the model request.

The nine exact `CompiledProfileEnvelope` values are frozen in
`vectors/csharp-profile-v0.json`. They contain declarative values only and no
path, executable, callback, validator, checker, plugin, URI, code, or schema
program.

## 9. Frontend arguments, phases, diagnostics, and status

`mpk.csharp.frontend_arguments.v0` permits only the generic successor release
assertions, fixed source-root mount, the complete selection envelope, and
output protocol. Contract/source paths come only from validated selection.
`--toolchain-root` occurs once with the fixed value `/mpk/toolchain`; it is not
caller-selectable. There is no caller-provided compiler, runtime, reference,
analyzer, SDK, project, response, plugin, toolchain, or registry path option.

The child phases are strictly:

1. `capture`: selection/path/type/size closure and immutable bytes;
2. `source`: strict source transport, exact parse-option construction, Roslyn
   parsing, and syntax diagnostics;
3. `metadata`: exact compilation-option/reference construction, symbols, and
   compilation diagnostics;
4. `typecheck`: exact predefined types/operators/conversions and definite
   assignment;
5. `subset`: declarations, closure, purity, initialization, and contracts;
6. `lowering`: operation/CFG adapter, VIR mapping, and required checks; and
7. `emission`: canonical successor VIR, map, manifest, and public envelope.

An earlier phase owns the result. Compiler parse/type/name errors are
`source-error`; a valid but unsupported form is `rejected`; compiler/API,
release, protocol, map, hash, or internal identity failure is
`frontend-error`. No non-success contains a partial artifact.

After transport validation, `SyntaxTree.GetDiagnostics` runs in stored tree
order and all active warning/error records are collected under the diagnostic
limits. A nonempty result terminates the source phase with
`CSHARP_SOURCE_PARSE`; compilation diagnostics are not queried for that input.
Only when all trees have no active syntax diagnostic does
`Compilation.GetDiagnostics` run in metadata. Its active
source/type/name/warning records use
`CSHARP_SOURCE_DIAGNOSTIC` and the normalization below.

The stable code registry is:

| Code/prefix | Status / phase | Meaning |
| --- | --- | --- |
| `CSHARP_LIMIT_*` | rejected / owner | deterministic profile limit |
| `CSHARP_CAPTURE_FILE_TYPE`, `_PATH`, `_INVENTORY` | rejected / capture | closed selected input failure |
| `CSHARP_SOURCE_ENCODING`, `_PARSE`, `_DIAGNOSTIC` | source-error / source or metadata | source transport/compiler error |
| `CSHARP_TOOLCHAIN_ARCHIVE`, `_RUNTIME`, `_ROSLYN`, `_REFERENCE`, `_OPTIONS`, `_ADAPTER` | frontend-error / owning release-through-emission phase | frozen toolchain mismatch |
| `CSHARP_SUBSET_DECLARATION`, `_TYPE`, `_LITERAL`, `_CONTROL_FLOW`, `_OPERATION`, `_OVERFLOW_CONTEXT`, `_CHECKED_CONVERSION`, `_CONVERSION`, `_CALL`, `_INITIALIZATION`, `_PURITY`, `_ABRUPT` | rejected / subset or lowering | closed semantic refusal |
| `CSHARP_CONTRACT_JSON`, `_SHAPE`, `_IDENTITY`, `_DUPLICATE`, `_MISSING`, `_UNUSED`, `_TYPE`, `_OPERATOR`, `_HASH` | rejected / subset | contract refusal |
| `CSHARP_LOWERING_OPERATION`, `_CFG`, `_CHECK_MISSING`, `_CHECK_EXTRA`, `_CHECK_ORDER` | rejected / lowering | exact adapter/semantic mismatch |
| `CSHARP_SOURCE_MAP_EXTERNAL`, `_RANGE`, `_UTF16` | frontend-error / emission | nonfaithful source origin |
| `CSHARP_FRONTEND_OUTPUT_LIMIT`, `_DIAGNOSTIC_BUDGET`, `_INTERNAL` | frontend-error / started phase | bounded operational failure |

Unknown `CSHARP_*` codes reject protocol conformance. Public metadata-phase
compiler issues use code `CSHARP_SOURCE_DIAGNOSTIC` and message exactly
`C# compiler diagnostic CSNNNN`; source prose, snippets, absolute paths,
arguments, environment, stack traces, and host suggestions are omitted. Every
other registered code uses only its status to select the exact message:
`C# source is invalid` for `source-error`, `C# source is outside the frozen
profile` for `rejected`, and `C# frontend failed closed` for `frontend-error`.
Every `CSHARP_LIMIT_*` rejection instead uses exactly `C# profile limit
exceeded`. No mutation text or host detail is interpolated. An active compiler
diagnostic ID must match `CS[0-9]{4}`; another ID is
`CSHARP_TOOLCHAIN_ADAPTER` rather than normalized by analogy.
Before issue construction, Roslyn records sort by normalized path, UTF-8
start/end, Roslyn ID, severity, then normalized message bytes, using empty
path and zero offsets when no usable source location exists. Severity uses
`Diagnostic.Severity` in enum order `Hidden`, `Info`, `Warning`, `Error`. A
location in a captured tree produces an optional public span only when both
endpoints map and `start < end`; a zero-length, absent, external, or unmappable
diagnostic location produces no span rather than a fabricated range. The resulting
Issues are sorted again by the common successor Issue key: path, start, code,
message, function ID, then end, with its specified absent-field sentinels. All
Roslyn warnings are compilation errors under the pinned options;
informational/hidden diagnostics are ignored only after exact ID and span
accounting in the pinned vector policy.

## 10. Deterministic limits

`mpk.csharp.limits.v0` is exactly:

| Limit ID | Inclusive maximum |
| --- | ---: |
| `source_files` | 256 |
| `source_file_bytes` | 1,048,576 |
| `source_total_bytes` | 16,777,216 |
| `contract_files` | 128 |
| `contract_file_bytes` | 1,048,576 |
| `contract_total_bytes` | 8,388,608 |
| `snapshot_entries` | 512 |
| `snapshot_total_bytes` | 33,554,432 |
| `normalized_path_bytes` | 1,024 |
| `canonical_method_id_bytes` | 1,024 |
| `selected_methods` | 32 |
| `method_closure` | 128 |
| `syntax_nodes` | 250,000 per compilation |
| `operations` | 100,000 per method, 250,000 per closure |
| `cfg_blocks` | 1,024 per method, 8,192 per closure |
| `contract_clauses` | 64 per method |
| `contract_nodes` | 1,024 per method, 8,192 per closure |
| `contract_depth` | 32 |
| `normalized_issues` | 1,024 |
| `diagnostic_message_bytes` | 4,096 each, 2,097,152 total |
| `frontend_argument_bytes` | 131,072 including every terminating NUL |
| `private_runtime_stdout` | 268,435,456 |
| `private_runtime_stderr` | 2,097,152 |
| `vir_canonical_bytes` | 201,326,592 |
| `source_map_canonical_bytes` | 33,554,432 |
| `source_manifest_canonical_bytes` | 4,194,304 |
| `frontend_stdout` | 268,435,456 including LF |
| `frontend_stderr` | 2,097,152 |

File counts use selected regular files; total byte counts sum their raw
captured lengths. `snapshot_entries` adds those files and every distinct
nonempty relative parent directory, but not the mounted root; directories add
zero snapshot bytes. Path, method-ID, argument, diagnostic, and stream limits
count UTF-8 bytes. Expanded argument bytes are the sum of every argv element
plus one terminating NUL per element.

`syntax_nodes` counts each result of
`DescendantNodesAndSelf(descendIntoTrivia:false)` once across the stored trees.
`operations` takes the reference-identity union of the method operation root,
every CFG block `Operations` root and `BranchValue`, and every recursively
visited public `ChildOperations` element. The same object counts once;
compiler-generated flow-capture operations count, while syntax wrappers with
no operation do not. CFG counts use `Blocks.Length`.
Closure totals sum the already checked per-method counts. Contract clauses are
`requires.len + ensures.len`; contract nodes count every recursive expression
object once, and a clause root has depth 1. Normalized issues and message bytes
are counted after redaction/span normalization but before public sorting.
Canonical artifact limits count their complete JCS payload; transport LF is
counted only where the table says so.

All counters use checked unsigned arithmetic and stop before retaining the
first excess item/byte. Exact boundaries accept. The three diagnostic counters
use `CSHARP_FRONTEND_DIAGNOSTIC_BUDGET`; the four private/final stream counters
use `CSHARP_FRONTEND_OUTPUT_LIMIT`; each is a frontend error. Every other
boundary-plus-one is a `CSHARP_LIMIT_*` profile rejection. Shared successor
limits may be smaller and then win in their earlier owning phase; this profile
never enlarges a shared limit.

## 11. Compiled profile payloads

For the C# entry, the exact payload values are closed as follows:

| Field | Exact value contract |
| --- | --- |
| `ai` | projection `mpk.csharp.ai_projection.v0`, label `C#`, redaction `minimal-v1`, source access false, proof authority false |
| `evidence` | recipe `mpk.csharp.evidence_recipe.v0`, both checkers required, proof authority `certificate_only` |
| `frontend` | argument/environment/limit IDs above, launcher `mpk.csharp.dotnet_launcher.v0`, private driver `none` |
| `manifest` | input kinds `[contract,source]`, extension `.cs`, unit kind `compilation` |
| `policy` | strategy `payment-policy-csharp-alpha`, checker `mvp-strict`, axiom `mvp-theory` |
| `release` | host/layout/Roslyn/runtime/reference profile IDs and exact toolchain-input self-hash |
| `source_map` | UTF-8 source, UTF-8-byte public offsets, empty synthetic allowlist |
| `vc` | C# contract/check profile IDs and `mpk.verify.limits.v0` |
| `vir` | C# operation/map profile IDs and `mpk.vir.limits.v0` |

The vector contains the complete member-level JSON. A different value requires
a new contract/profile ID and registry entry; an implementation cannot add a
field while retaining these IDs.

## 12. Hashes and conformance vectors

`develop/specs/vectors/csharp-profile-v0.json` has schema
`mpk.csharp.profile.conformance.v0`. It freezes exact profile/parameter/
selection/contract values; toolchain archives and projections; compiler
options; launcher/argv/environment; all 34 semantic-row dispositions; type,
operation, Roslyn checked-state, conversion, required-check, executable
accepted-source, rejection, source-map, precedence, diagnostic, limit,
payload, hash, isolation, and upgrade cases.

For an accepted source case, `expected_profile_operations` is an ordered
required subsequence of the complete lowering, not a replacement VIR stream;
the section-5 mappings and validators still close every intervening
instruction. `expected_required_checks` is exhaustive and in canonical order.
The executor validates complete VIR/map/manifest bytes and then checks both
projections.

The hash domains introduced by this profile are:

| Object | Exact domain |
| --- | --- |
| toolchain-input descriptor | `MPK-CSHARP-TOOLCHAIN-INPUTS-0.1` |
| reference inventory | `MPK-CSHARP-REFERENCE-INVENTORY-0.1` |
| canonical selection envelope | `MPK-CSHARP-SELECTION-0.1` |
| canonical C# sidecar | `MPK-CSHARP-CONTRACT-SIDECAR-0.1` |

The normalized VIR contract uses the successor common
`MPK-CONTRACT-1.0` domain. Registry entry/root domains remain those in
`SEMANTIC_PROFILE_REGISTRY_V1.md`.

`develop/specs/vectors/semantic-profile-registry-v2.json` has schema
`mpk.semantic_profile.registry.conformance.v2`. It freezes the complete
revision-2 root, both unchanged predecessor entries, C# entry, sizes/hashes,
append-only proof, ordering, later-language absence, and inactive/atomic
activation cases.

The frozen vector-model owner remains
`crates/mpk-vc/tests/csharp_profile_spec.rs`. It is a test-only conformance and
hash model, exports no library item, and is not linked into `mpk`, a frontend,
checker, or production parser. The completed CSHARP-02 implementation adds
production validators and implementation executors without weakening this
owner; their exact ownership is recorded in the CSHARP-02 traceability ledger.

## 13. Upgrade procedure

Any SDK, runtime patch, Roslyn package/commit, reference archive/assembly,
language version, option, public API/enum, CFG pattern, diagnostic policy, or
native-host change is a semantic upgrade, not dependency maintenance. It
requires:

1. a new toolchain/compiler/adapter ID and exact before/after archive diff;
2. regeneration and review of all source, operation, CFG, diagnostic,
   differential, reference, profile-payload, and release hashes;
3. two-clean-build and two-run equality from reviewed offline inputs;
4. old/new cross-rejection with no fallback resolver or roll-forward;
5. unchanged Certificate v0/both-checker acceptance and a zero-new-category
   axiom report; and
6. a new immutable registry entry/root if accepted semantics or a compiled
   profile contract changes.

No update command, package restore, version range, floating tag, latest alias,
or in-place mutation of a released profile is permitted.

## 14. Historical implementation and activation gate

The frozen T03 handoff status was: “No current MPK binary accepted this
profile; C# remains inactive; this package adds no executable or production
route.” That sentence is retained as a conformance-model anchor and describes
only the pre-activation state.

At specification freeze, C# was required to remain inactive until one
CSHARP-02 release atomically:

1. implements the exact source capture, Roslyn adapter, contracts, lowering,
   map/manifest, runner, policy/evidence, and AI contracts above;
2. migrates Go and Rust to the successor schemas and revision-2 registry in the
   same release, with no old public VIR input;
3. provisions only the pinned offline build/runtime/package/reference bytes;
4. executes every positive, negative, boundary, mutation, hash, isolation,
   fuzz, differential, and deterministic vector;
5. emits representative C# certificates accepted from identical bytes by both
   source-free checkers; and
6. passes the complete installed release gate with an empty review ledger.

`CSHARP-02-T20` satisfied that gate on 2026-08-30. `JAVA-03-T10` subsequently
retained the exact C# entry and semantics in revision 3. The pre-activation
Go/Rust-only state and this T03 package's lack of executable, bundle, parser,
policy, evidence, AI, or proof-rule changes remain historical facts, not the
current release state.
