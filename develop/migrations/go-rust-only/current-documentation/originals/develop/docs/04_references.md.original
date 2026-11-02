# References

Initial references accessed: 2026-06-26.

Multi-language frontend references accessed: 2026-08-21.

## NPA

- NPA repository: https://github.com/finitefield-org/npa
- NPA core spec v0.2.0: https://github.com/finitefield-org/npa/blob/main/develop/core-spec-v0.2.0.md
- NPA proof-corpus AI workflow: https://github.com/finitefield-org/npa/blob/main/develop/proof-corpus-ai-workflow.md

Relevant points used in this plan:

- Certificate-first proof evidence.
- Small trusted base.
- Untrusted parser/elaborator/tactic/automation/AI layers.
- Canonical certificate bytes.
- Rust verifier and source-free reference checker.
- Deterministic hashes and axiom reports.
- Core term grammar using Sort, variable, Const, App, Lam, Pi, Let.
- Deterministic and fuel-limited definitional equality.
- Theorem opacity after checking.
- Canonical binary encoding.
- Explicit out-of-scope items such as eta conversion, proof irrelevance as conversion, theorem unfolding, external SMT trust, theorem graph trust, and AI search trust.

## Go

- Go specification: https://go.dev/ref/spec
- Go SSA package: https://pkg.go.dev/golang.org/x/tools/go/ssa

Relevant points used in this plan:

- Go has fixed-width integer semantics that require careful modeling.
- Unsigned integer operations wrap modulo the bit width.
- Signed integer overflow is legally defined by representation, operation, and operands and does not panic.
- Go SSA tooling is available for frontend engineering, but MPK treats frontend output as untrusted.

## C# and Roslyn

- .NET Compiler Platform SDK overview: https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/
- Work with semantics: https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-semantics
- `ControlFlowGraph` API: https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.flowanalysis.controlflowgraph
- C# checked and unchecked semantics: https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/checked-and-unchecked
- C# language specification: https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/

Relevant points used in the multi-language design:

- Roslyn exposes syntax, symbols, types, constants, conversions, operations,
  and control flow through supported semantic APIs.
- MPK can pin an exact Roslyn/.NET toolchain and lower a deliberately
  restricted C# subset without parsing diagnostic text.
- Compiler and frontend output remain untrusted and require fail-closed subset
  validation plus deterministic independent fixtures.

## Java and the JDK compiler APIs

- Java Language Specification: https://docs.oracle.com/javase/specs/jls/se25/html/
- Java Compiler API: https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/JavaCompiler.html
- `JavacTask` API: https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/JavacTask.html

Relevant points used in the multi-language design:

- A pinned JDK exposes parse, analyze, element, type, and compiler-tree APIs
  suitable for an untrusted restricted-language frontend.
- Java overflow, exceptions, initialization, references, class loading, and
  dispatch require an explicit semantic subset rather than a Go/Rust profile
  reinterpretation.
- MPK must pin the exact JDK release and module/API surface selected by the
  language-specific specification.

## Dart

- Dart language specification: https://dart.dev/resources/language/spec
- Dart analyzer package: https://pub.dev/packages/analyzer
- `AnalysisSession` API: https://pub.dev/documentation/analyzer/latest/dart_analysis_session_analysis_session/AnalysisSession-class.html

Relevant points used in the multi-language design:

- The analyzer provides resolved source-model access, but the analyzer and SDK
  version are part of the pinned untrusted frontend bundle.
- Integer, numeric, null-safety, exception, and runtime behavior must be frozen
  explicitly before a Dart semantic profile can be admitted.
- Package resolution and generated code stay outside the initial subset unless
  a later frozen specification models them exactly.

## TypeScript

- TypeScript Compiler API usage: https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API
- TypeScript handbook, erased types: https://www.typescriptlang.org/docs/handbook/2/basic-types.html#erased-types
- ECMAScript language specification: https://tc39.es/ecma262/

Relevant points used in the multi-language design:

- The compiler API exposes parsed and type-checked program structure, but
  TypeScript types are erased and do not themselves define runtime behavior.
- A sound MPK profile must model a restricted JavaScript runtime subset and
  treat TypeScript types as untrusted analysis inputs, not proof evidence.
- Dynamic property access, prototype behavior, coercion, exceptions, modules,
  and ambient declarations remain rejected until specified.

## Python

- Python `ast` module: https://docs.python.org/3/library/ast.html
- Python language reference: https://docs.python.org/3/reference/index.html

Relevant points used in the multi-language design:

- CPython exposes a supported AST interface, but name resolution, types,
  effects, dispatch, exceptions, and dynamic object behavior require an
  MPK-owned closed analysis and rejection layer.
- Type annotations are not runtime proof evidence and cannot justify treating
  Python as a statically typed source language.
- Python is sequenced after the more compiler-resolved frontends because its
  initial sound subset and differential oracle require the greatest semantic
  restriction.
