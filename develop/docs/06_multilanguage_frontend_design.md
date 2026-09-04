# Post-Rust Multi-Language Frontend Expansion Design

Status: Gate B, `MLANG-00`, Gate C, `MLANG-01`, and Gate D are complete after
the completed `RUST-07-T05` entry gate. `CSHARP-02-T01` through
`CSHARP-02-T20` completed serially; `JAVA-03-T01` through T10 are also
implemented. The checked-in successor Go/Rust/C#/Java release is active. T01 completed the Java profile/vector/toolchain
freeze and disposable compiler/JVM compatibility probes. T02 completed the
inactive offline build candidate; T03 completed the inactive registry and
artifact validators. T04 completed internal capture, the public compiler
adapter and bounded diagnostics. T05 completed source admission, inert
initialization, acyclic call closure and typed sidecars. T06 completed private
CFG/lowering and deterministic artifact emission. T07 completed the private
candidate/JVM runner and native x86-64 Linux acceptance. T08 completed private
verification integration, T09 completed conformance and local release
rehearsal, and T10 implemented atomic activation.
`07_java_frontend_design.md` and the Java implementation ledger record
the implementation boundary. Registry revision 3 and all five tuples are
installed; the native x86-64 T10 gate passed twice on 2026-09-03, so
CSHARP-03-T01-W01 through W10 completed the entry audit, feasibility and
runtime probes, foundation/freeze work, and normative package publication.
The package is inactive, production behavior is unchanged,
CSHARP-03-T02-W01/W02/W03/W04/W05/W06 are complete, and CSHARP-03-T02-W07 is ready.

Prepared: 2026-08-21

Updated: 2026-09-05 (`JAVA-03-T10` complete with native x86-64 release
receipt; post-Java `CSHARP-03-T01-W01` through W10 and T02-W01/W02/W03/W04/W05/W06
complete with the practical C# normative freeze package published and
inactive; T02-W07 ready)

## 1. Decision summary

MPK records the future C#, Java, Dart, TypeScript, and Python expansion now,
but finishes the existing Go-to-VIR migration and scoped Rust v0 implementation
before starting any multi-language design, feasibility, specification, or
production task.

The sequencing decision is:

1. `RUST-07-T05` first completes the Rust v0 determinism, fuzzing,
   compiler-upgrade, CI, and release gates.
2. `MLANG-00` then performs the semantic comparison and compiler-API
   feasibility work for all five candidate languages.
3. `MLANG-01` then audits the completed Go/Rust implementation and freezes the
   successor extension mechanism and first C# specification package.
4. New production frontends are then added serially. C# and Java are followed
   by the published `CSHARP-03` practical C# expansion before Dart, TypeScript,
   and Python.
5. “Rust v0 complete” means the deliberately restricted Rust profile in
   `05_rust_frontend_design.md` satisfies its release gates. It does not mean
   waiting for full-language Rust support.

The language comparison and official references below are non-normative
roadmap rationale recorded to choose this queue. They are not completion of an
`MLANG-00` deliverable. `MLANG-00` revalidates them against the completed Rust
implementation and then produces the reviewed semantic and feasibility
records.

That revalidation's first deliverable is
`mlang-00-semantic-comparison-matrix.md`. It closes the T01 candidate-operation
inventory and dispositions while leaving every compiler, target, profile, and
foundation choice that belongs to later work explicitly blocked.

The second deliverable is
`mlang-00-compiler-integration-feasibility.md`. It validates the minimum
supported public analysis boundaries and deterministic input closures, then
gives every matrix row an integration `GO` or `NO-GO` verdict without choosing
a production pin or activating a profile.

The third deliverable is
`mlang-00-go-rust-shared-boundary-audit.md`. It audits the implemented Go/Rust
language/profile branches, process launches, release executable closure, and
compiler callback surfaces. Its one blocking finding—ambient source/PATH
execution of the required Go reference checker—was remediated serially with a
deterministically rebuilt embedded executable and same-byte checker input. The
repeated audit has no open finding and added no future-language production or
accepted schema value.

The fourth deliverable is
`mlang-01-go-rust-csharp-gap-audit.md`. It uses the completed implementation to
classify every C# semantic and cross-contract gap. All 18 feasible candidate
rows reuse the implemented VIR/VC core as profile-only work; the remaining
rows are one excluded VIR-operation gap, two excluded checked-foundation gaps,
and 13 unsupported behaviors. Four disjoint successor-shape gaps are assigned
to `MLANG-01-T02`; exact C# profile records remain assigned to
`MLANG-01-T03`. No C# production or accepted schema value was added.

The fifth deliverable is
`../specs/SEMANTIC_PROFILE_REGISTRY_V1.md` with
`../specs/vectors/semantic-profile-registry-v1.json`. It chooses and freezes a
closed, hash-pinned semantic-profile registry whose entries can select only
finite validators compiled into the pinned release. Its revision-1 baseline
contains only the existing Go/Rust profiles; at that deliverable's freeze point
the atomic successor schema remained inactive. Exact C# content and the
revision-2 root remained assigned to `MLANG-01-T03`.

The sixth deliverable is `../specs/CSHARP_PROFILE_V0.md` with
`../specs/vectors/csharp-profile-v0.json` and
`../specs/vectors/semantic-profile-registry-v2.json`. It freezes the complete
C# scalar subset, toolchain/reference closure, operation/check mapping,
contracts, diagnostics, limits, nine compiled-profile payloads, and immutable
revision-2 root without activating any production path.

The seventh deliverable is
`csharp-02-implementation-traceability-ledger.md`. It decomposes CSHARP-02 into
20 strictly serial tasks, assigns every frozen requirement and vector field to
one primary implementation owner, and reserves production activation for the
final atomic cutover in T20. It is an execution plan and does not amend either
normative specification.

The eighth design and freeze-package record is
`08_csharp_practical_subset_design.md`. It defines a post-Java practical C#
profile for immutable domain types and construction, bounded collections,
canonical text codecs, business values and outcomes, richer numeric and control
flow, closed exceptions, explicit integration boundaries, and pure business
state transitions. Application source and runtime remain independent of MPK;
user-defined generics, iterators, and async source are explicit exclusions,
while enumerated internal semantic templates come from one registered,
content-hash-pinned foundation bundle and are monomorphized before VIR
emission. Its canonical document is an MPK verification-overlay transport,
not a required application runtime protocol. T01-W10 published
`CSHARP_PRACTICAL_PROFILE_V1.md`,
`CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md`, and the manifested 700-vector
package without widening the active scalar profile or authorizing an installed
bundle or public route. The Java T10 receipt is accepted, CSHARP-03 T01 and
T02-W01/W02/W03/W04/W05/W06 are complete, and T02-W07 is ready.

The order may change only through a reviewed governance amendment that records
the user value, semantic risk, compiler integration quality, and effect on the
critical path. It must not be changed merely to start several frontend
implementations in parallel.

## 2. Authority and relationship to the active program

Before `RUST-07-T05` completed, the implementation authority remained:

- `05_rust_frontend_design.md`;
- `05_rust_frontend_design-todo.md`;
- the frozen VIR-00 specifications and vectors; and
- the dependency order `VIR-01 -> GO-VIR-02 -> RUST-03..RUST-07`.

This document is subordinate to those contracts. In particular, it does not:

- add language identifiers to `mpk.vir.v0`;
- add selection branches to `mpk.frontend.cli.v0`;
- add release tuples or bundles for future languages;
- change Go or Rust semantic profiles;
- create a second public VIR input during the Go cutover;
- permit mixed-language VIR modules; or
- change Certificate v0, checker inputs, trust, or axiom categories.

If this document conflicts with a frozen v0/v1 specification or with the
current Rust task graph, the frozen specification and Rust task graph win until
a separately reviewed successor specification is activated.

## 3. Goals

The multi-language expansion should:

- reuse the same certificate, checker, VC, policy, evidence, source-map,
  source-manifest, and registered-frontend architecture;
- make each language's runtime semantics explicit in a distinct semantic
  profile rather than infer semantics from syntax or a frontend name;
- keep every compiler, SDK, analyzer, frontend, source file, contract, IR,
  source map, manifest claim, and VC outside the proof trust boundary;
- pin every compiler API, target, standard-library/reference input, option,
  subordinate executable, and release bundle needed for deterministic
  lowering;
- define a small fail-closed subset before accepting any source program;
- reuse VIR operations only where the source-language behavior is exact;
- add ordinary checked definitions and proof terms accepted by the current
  program-certificate profile when a new value model is required; any theory
  support is a separate prior foundation/checker project, not authority granted
  by a language phase;
- preserve one source language and one semantic profile per VIR module; and
- make language additions repeatable without creating an unreviewed plugin or
  raw-executable escape hatch.

## 4. Non-goals

This plan does not aim to:

- run a multi-language milestone while the Rust program is active;
- design or implement multiple future-language phases concurrently;
- add placeholder language enum values, empty selection variants, dummy
  semantic profiles, or registry entries without complete semantics;
- define one “universal” semantic profile for superficially similar syntax;
- trust compiler ASTs, typed trees, control-flow graphs, bytecode, IL, or
  analyzer results as proof evidence;
- support a full source language in its first profile;
- accept heap allocation, object identity, dynamic dispatch, reflection,
  exceptions, async execution, threads, FFI, floating point, or runtime code
  generation merely because a compiler API exposes them;
- load third-party frontend plugins into `mpk-cli` or a checker;
- accept a user-selected compiler or frontend path on an evidence route;
- add `CSharpSemanticsAxiom`, `JavaSemanticsAxiom`, `DartSemanticsAxiom`,
  `TypeScriptSemanticsAxiom`, `PythonSemanticsAxiom`, or another Certificate v0
  axiom category; or
- model cross-language calls inside one VIR document.

Cross-language composition remains a certificate/package concern: separately
verified modules may interact only through declared, hash-pinned certificate
imports whose exported theorem interfaces are checked by the existing
checkers. Source-language FFI behavior is outside the initial expansion.

## 5. Invariants every language must preserve

### 5.1 Proof boundary

The only proof-acceptance route remains canonical `.mpcert` checking. A pinned
compiler or byte-identical frontend build improves traceability and
reproducibility; it does not establish that lowering matches the source.

Each report must continue to distinguish:

- a mathematical certificate/checker claim; and
- an untrusted source-to-certificate traceability claim.

### 5.2 Fail-closed semantics

For every accepted source operation, the owning language profile specifies:

- source operand and result types;
- value semantics and evaluation order;
- normal and abrupt completion behavior;
- every required runtime-safety check;
- target- or option-dependent parameters;
- the exact VIR operation and contract encoding; and
- the first stable rejection class for unsupported or ambiguous forms.

Unknown syntax, types, compiler nodes, implicit conversions, dispatch targets,
runtime behavior, or profile parameters reject. They are never erased,
approximated, modeled as unconstrained values, or accepted under another
language's profile.

### 5.3 Registered, isolated frontends

Each language uses a separate untrusted registered frontend bundle. A bundle
contains only the exact main executable, permitted subordinate tools, pinned
compiler/analyzer/runtime inputs, and redistribution metadata frozen by its
own specification.

The generic runner resolves bundles internally, uses immutable snapshots,
denies ambient credentials and network access, enforces resource limits, and
accepts no raw executable path. A language may use an in-process compiler API
inside its own frontend process; that API is never linked into a checker.

### 5.4 One module, one semantic profile

A VIR module continues to contain exactly one source language, one semantic
profile, one parameter object, and one target identity. A frontend cannot
self-select or silently default those values. The caller-selected registered
release tuple must agree with the request, frontend response, VIR, source map,
source manifest, VC, policy evidence, and reproduction recipe.

### 5.5 No speculative shared abstraction

Shared implementation code may be introduced only when at least two completed
frontends require the same behavior and their specifications prove the
behavior identical. Ordinary Rust milestone work may make an already-required
Go/Rust boundary language-neutral because those two active profiles need it;
that work is not a multi-language phase and must not add a plugin framework or
unused abstraction solely for the five future languages.

## 6. Versioning and extension strategy

`mpk.vir.v0` and `mpk.frontend.cli.v0` are closed over Go and Rust. Adding only
a string such as `"csharp"` would change their accepted language/selection
sets without defining corresponding semantics, so placeholder additions are
forbidden.

After `RUST-07-T05` and `MLANG-00`, `MLANG-01` audited the completed Go/Rust
system and chose the second reviewed strategy: a new schema revision with a
closed, hash-pinned semantic-profile registry that owns each allowed
language/profile/parameter/selection tuple. The exact normative design is
`../specs/SEMANTIC_PROFILE_REGISTRY_V1.md`; its current revision-1 vector is a
frozen, inactive design baseline rather than an accepted production input.

That strategy requires:

- unknown registry IDs fail closed;
- the exact registry root is embedded in the release and repeated in evidence;
- all consumers compile validators for every activated selection and parameter
  schema;
- a registry entry cannot supply executable validation or checker code;
- profile activation requires a normative specification and complete vectors;
  and
- registry updates are reviewed release changes rather than runtime plugin
  installation.

The gap audit also determined whether VIR value/control-flow operations are
sufficient. If a new source language needs a new numeric carrier, abrupt-
completion model, heap model, or call semantics, the schema and hash-domain
revision must include those semantics explicitly. Reusing an existing VIR
operation with a different meaning is forbidden.

If the successor changes a shared serialized contract, all active Go/Rust
producers, consumers, fixtures, reports, and examples migrate atomically. A
released configuration must not accept both old and successor program-IR
schemas as parallel public inputs. Certificate v0 does not change.

The first successor specification must be designed so later languages can be
added by a reviewed profile/registry revision when their semantics fit the
existing representation. It must not pre-register the remaining languages
before their profiles and vectors exist.

## 7. Common initial subset policy

A new language begins with the smallest subset that can exercise the shared
path without introducing heap or dynamic-language semantics. The candidate
baseline is:

- explicitly selected, non-generic, pure top-level or static functions;
- scalar `bool` and only those integer types whose source behavior has an exact
  checked representation;
- parameters, local values, deterministic assignments, comparisons, Boolean
  operations, conditional branches, and early return;
- explicit preconditions and postconditions normalized into the shared VIR
  contract form;
- no recursion and no cycles in the selected call graph; and
- no ambient I/O, time, randomness, locale, environment, process, reflection,
  or package initialization effect.

Loops, aggregates, calls, and additional integer operations are follow-up
profile revisions after the straight-line/branch corpus passes differential
and certificate gates. A feature is not part of the common baseline when its
source-language behavior differs; it remains language-profile work.

## 8. Language evaluation and implementation order

| Order | Language | Compiler boundary to evaluate | Initial profile direction | Principal blockers before freeze |
| --- | --- | --- | --- | --- |
| 1 | C# | Pinned Roslyn compilation, semantic model, operations, and control-flow graph | Pure static methods over `bool` and fixed-width built-in integers, locals, branches, and return | Checked/unchecked context and compiler option, implicit/user-defined conversions and operators, nullable/reference values, exceptions, virtual dispatch, target framework/reference assemblies |
| 2 | Java | Pinned JDK `JavaCompiler`/`JavacTask`, Compiler Tree API, language/type model | Pure static methods over primitives, locals, branches, and return | Class initialization, references/null, exceptions and abrupt completion, narrowing/unboxing, virtual calls, annotation processing, module/class paths, JDK release target |
| 3 | Dart | Pinned Dart SDK and analyzer resolved-unit API | Pure top-level/static functions after one runtime target and exact scalar model are selected | Target-dependent runtime behavior, integer representation, null safety, `dynamic`, operator dispatch, async, package configuration, analyzer API evolution |
| 4 | TypeScript | Pinned TypeScript `Program`, AST, symbols, and type checker plus an explicit ECMAScript runtime target | Pure functions after an exact JavaScript numeric subset is chosen | Types are erased at runtime, `number`/`bigint` semantics, coercion, objects/prototypes, getters, closures, exceptions, async, module resolution, compiler API evolution |
| 5 | Python | Pinned CPython parser/AST plus an MPK-owned name/type/subset analysis layer | Fully annotated pure functions after exact integer and operation semantics are frozen | Dynamic dispatch and mutation, arbitrary-size integers, exceptions, descriptors/operators, imports/decorators, generators/async, runtime introspection, AST evolution |

This order favors compiler integrations that expose resolved semantic and
control-flow information before languages whose static types do not determine
runtime behavior. User value may justify swapping adjacent languages, but the
entry and exit gates remain unchanged.

### 8.1 C# boundary

Roslyn exposes syntax and semantic models and a compiler control-flow graph,
making C# the preferred third source language. The initial profile still
rejects reference types, allocation, properties with accessors, delegates,
virtual/interface calls, exceptions, iterators, async, reflection, unsafe code,
user-defined operators/conversions, and ambiguous overflow contexts.

The proposed successor practical boundary is designed separately in
`08_csharp_practical_subset_design.md`. It uses a new semantic profile and
successor helper-artifact family; it does not reinterpret the active
`mpk.csharp.scalar.v0` profile or make any listed feature currently accepted.

The language version, nullable mode, default overflow-checking option, target
framework, reference assemblies, preprocessor symbols, analyzer/source-
generator policy, and compiler package graph are pinned inputs.

### 8.2 Java boundary

The implementation design is `07_java_frontend_design.md`. It selects Java
SE 25 without preview features, a pinned Temurin JDK, and a first scalar
profile over `boolean`, `int`, and `long` in field-free source interfaces.
Static acyclic calls are admitted only within the completely selected inert
initialization closure. Java wrapping arithmetic and masked shifts use the
existing BV foundations with Java-owned checks. `JAVA-03-T01` froze
`../specs/JAVA_PROFILE_V0.md`, Java/revision-3 vectors, the exact JDK/native
inventory and JVM requirements. The disposable compiler/JVM measurements ran
on Linux amd64 under CPU emulation. Native Linux production enforcement was
accepted separately for T07, T09 completed the full local release rehearsal,
and T10 atomically activated the reviewed release. T02 added the offline build candidate with
exact project/class/JAR inventories and two isolated builds. T03 completed
the inactive profile and source-artifact validators. T04 completed capture and
the public compiler adapter; T05 completed source admission and typed sidecars.
T06 completed private CFG/lowering, original-byte source maps and complete
artifact emission. T07 completed the registered candidate, JVM runner and
native acceptance. T08 completed private verification integration, and T09
completed conformance and release rehearsal. T10 installed revision 3 and the
Java tuple without changing predecessor entry semantics.

The supported JDK compiler APIs expose parse/analyze trees and language/type
models. The initial profile rejects allocation, reference values, instance and
virtual calls, exceptions, synchronization, lambdas, reflection, native calls,
annotation processors, static initialization with effects, and unpinned
classpath/module-path inputs.

The JDK distribution, `--release` target, compiler options, system modules,
source/class/module paths, and permitted processor set are pinned inputs. The
default processor set is empty.

### 8.3 Dart boundary

The analyzer can provide resolved syntax, elements, types, and diagnostics,
but the profile is not frozen until one execution target and its integer,
null, dispatch, and failure semantics are fixed. An analyzer result is not a
runtime-semantics oracle.

The Dart SDK, analyzer package/source graph, language version, package config,
analysis options, experiment flags, target runtime, and platform libraries are
pinned. `dynamic`, allocation, instance dispatch, extension/member ambiguity,
exceptions, async, isolates, FFI, and platform-dependent libraries initially
reject.

### 8.4 TypeScript boundary

TypeScript static types are erased, so the semantic profile must model the
selected JavaScript/ECMAScript runtime behavior rather than treat TypeScript
types as runtime proof. General `number` arithmetic is not mapped to existing
bitvectors without an explicit, reviewed numeric model.

The TypeScript compiler, Node/runtime target if used, `lib.*.d.ts` set, module
resolution mode, ECMAScript target, JSX/decorator settings, package/lock graph,
and ambient declarations are pinned. `any`, `unknown` use without refinement,
objects/prototypes, getters/setters, closures, exceptions, async, generators,
dynamic import, eval, and host APIs initially reject.

### 8.5 Python boundary

Python requires an MPK-owned closed subset analysis after parsing because an
AST alone does not resolve dynamic runtime behavior. Type annotations are
untrusted subset inputs, not proof evidence, and do not authorize an operation
whose runtime semantics are not frozen.

The CPython distribution, language version, standard-library input set,
optimization flags, import root, annotations policy, and allowed builtins are
pinned. Attribute access, user objects, mutation through aliases, operator
overloading, exceptions, decorators, imports with effects, comprehensions,
generators, async, reflection, native extensions, and monkey patching initially
reject.

## 9. Required specification package per language

Before implementation, each language owns a complete package containing:

- a frozen subset specification with accepted and rejected forms;
- a semantic-profile specification with exact parameter and required-check
  matrices;
- a compiler/frontend/private-driver boundary specification where applicable;
- pinned build and execution input descriptors;
- registered release bundle and tuple definitions;
- source selection, unit, path, source-map, and source-manifest rules;
- contract grammar, type checking, attachment, and stable IDs;
- deterministic phase/code/status/exit precedence;
- byte/count/depth/graph/resource limits;
- positive, negative, duplicate-key, unknown-field, boundary, mutation, and
  hash-domain vector sets;
- named implementation test owners recorded in the vector manifest; and
- an upgrade procedure that treats compiler/API changes as reviewed semantic
  changes, not dependency maintenance.

No producer or consumer for a new serialized branch merges before this package
and its vectors are frozen.

## 10. Verification strategy

Every new frontend must pass:

1. subset conformance against all accepted and rejected source families;
2. exact semantic-profile operation/check validation;
3. frontend protocol, source-map, and manifest conformance;
4. deterministic two-build and two-run byte comparisons;
5. differential execution against the pinned source runtime for bounded
   generated and handwritten cases;
6. cross-profile rejection proving another language cannot select or reuse the
   profile;
7. mutation, parser, protocol, path, resource, and compiler-output fuzzing;
8. certificate generation and identical-byte acceptance by both source-free
   checkers; and
9. an axiom report proving no new language-specific category was introduced.

Differential execution is translation-confidence evidence only. A matching
runtime result is not proof evidence and never replaces certificate checking.

## 11. Stage gates

### Gate A: complete Rust v0 before multi-language work

`RUST-07-T05` must pass before `MLANG-00` starts. Recording this queued roadmap
does not authorize further semantic research, feasibility experiments,
successor-contract work, or production frontend work during the Rust program.

### Gate B: post-Rust design and feasibility

Completion: `MLANG-00-T01` through `MLANG-00-T03` are complete. The semantic
matrix, compiler-boundary feasibility record, and repeated zero-finding
Go/Rust shared-boundary audit are the closed Gate B outputs.

Allowed work:

- semantic comparison tables;
- official compiler/API research;
- non-normative feasibility notes and disposable local experiments;
- candidate subset and target analysis; and
- documenting potential VIR gaps.

Forbidden work:

- production frontend or runner code;
- new accepted schema IDs or enum variants;
- release registry entries or bundles;
- future-language paths in policy/evidence APIs; and
- speculative shared plugin/refactor infrastructure.

### Gate C: successor specification work after `MLANG-00`

The completed Rust path and `MLANG-00` feasibility records must show which VIR,
VC, contract, source-map, manifest, runner, policy, and evidence abstractions
are genuinely shared. `MLANG-01` then freezes the successor versioning
strategy and C# specification package.

Completion: `MLANG-01-T01` produced the closed implementation-backed gap
ledger, `MLANG-01-T02` froze the then-inactive successor registry mechanism,
and `MLANG-01-T03` froze the exact then-inactive C# profile, registry revision
2, toolchain/reference inventory, semantics, and owned vectors. Those outputs
became active only through the later atomic `CSHARP-02-T20` release.

### Gate D: C# production work after `MLANG-01`

Only after `MLANG-01` completes may C# production code and release bundles
merge. The earlier `RUST-07-T05 -> MLANG-00 -> MLANG-01` dependencies are
transitive and may not be bypassed.

Completion: `CSHARP-02-T01` through `CSHARP-02-T19` built and hardened the
successor registry, source artifacts, Go/Rust/C# frontends, VC,
policy/evidence, program-certificate, AI, API, bundle, and installed-release
paths behind the recorded staging boundary. `CSHARP-02-T20` then performed the
single atomic activation: semantic-registry revision 2 and the successor
bundle registry are installed beside one `bin/mpk`; all four registered
Go/Rust/C# tuples and the nine C# compiled-profile contracts are active;
active fixtures use successor identities; predecessor and crossed identities
reject; and the executable staging tree is gone.

The C# cutover retained only the archived zero-finding review and semantic-
difference reports. `crates/mpk-cli/tests/successor_atomic_cutover.rs` still
owns the installed-image proof; after Java activation,
`scripts/check-java-frontend.sh` is the sole offline two-pass release gate.
Certificate v0, both checker inputs and
verdicts, the four axiom categories, and the untrusted status of every helper
artifact remain unchanged. Gate D then admitted `JAVA-03` as its next serial
phase.

`JAVA-03-T10` completed the Gate E cutover and its native x86-64 release gate
passed twice. The `CSHARP-03` practical C# phase is next; all ten T01 freeze
work items and T02-W01/W02/W03/W04/W05/W06 are complete, and T02-W07 is ready.
DART-04 remains
blocked until that entire phase passes its release gate; design preparation
does not count as phase entry.

### Gate E: serial language/profile admission

A later language or source-profile expansion's entire production phase,
including its feasibility refresh, specification freeze, implementation, and
release work, starts only after the previous production phase has:

- a frozen specification/vector package;
- an end-to-end accepted certificate corpus;
- complete negative and deterministic gates;
- a registered release bundle;
- both-checker agreement; and
- a clean review ledger.

## 12. Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Shared contracts churn while Rust is incomplete | Do not start `MLANG-00`; audit only the completed Rust release path. |
| Placeholder language support becomes accidental acceptance | Keep v0 enums/unions closed; unknown profiles and registry entries reject. |
| A universal IR operation hides different source behavior | Per-language profile matrices and exact required checks; add a new operation/version when meanings differ. |
| Five compiler ecosystems multiply supply-chain and sandbox scope | One frontend at a time, complete pinned inventories, registered bundles, no ambient package manager or network. |
| Static type information is mistaken for runtime semantics | Differential tests plus source-language specifications; TypeScript/Python annotations remain untrusted helper input. |
| New numeric or heap semantics expand the trusted base | Use ordinary checked terms under the active program-certificate profile; a theory/checker change is a separate prior governance project, never a language shortcut. |
| A compiler API changes silently | Exact compiler/SDK/library pins, exhaustive node matching, upgrade corpus, no automatic upgrades. |
| Cross-language FFI bypasses module isolation | No mixed-language VIR or FFI profile; compose only checked certificate imports. |

## 13. Completion criteria

The queued-roadmap documentation is complete when:

- this sequencing is routed from README and the roadmaps;
- current Rust milestones explicitly exclude all future-language milestone
  work;
- the semantic/API feasibility matrix has an owner and review gate;
- no frozen v0 schema contains a placeholder future-language value; and
- the release gates define how a new language is admitted.

A language or source-profile expansion is complete only when its own
specification package, frontend, deterministic registered bundle,
conformance/differential corpus, policy/evidence path, certificate corpus,
both-checker agreement, axiom review, upgrade procedure, and clean review
ledger all pass.

## 14. Primary references

- [Roslyn SDK overview](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/)
- [Roslyn semantic model](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-semantics)
- [Roslyn control-flow graph](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.flowanalysis.controlflowgraph)
- [C# checked and unchecked semantics](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/checked-and-unchecked)
- [C# language specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/)
- [JDK Compiler module and Compiler Tree API](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/module-summary.html)
- [`JavacTask` parse/analyze API](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/JavacTask.html)
- [Java language and compiler APIs](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/module-summary.html)
- [Java Language Specification](https://docs.oracle.com/javase/specs/jls/se25/html/)
- [Dart language specification](https://dart.dev/resources/language/spec)
- [Dart analyzer package](https://pub.dev/packages/analyzer)
- [Dart analyzer `AnalysisSession`](https://pub.dev/documentation/analyzer/latest/dart_analysis_session/AnalysisSession-class.html)
- [TypeScript compiler API](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API)
- [TypeScript runtime behavior and erased types](https://www.typescriptlang.org/docs/handbook/typescript-from-scratch.html)
- [ECMAScript language specification](https://tc39.es/ecma262/)
- [Python AST](https://docs.python.org/3/library/ast.html)
- [Python language reference](https://docs.python.org/3/reference/)
