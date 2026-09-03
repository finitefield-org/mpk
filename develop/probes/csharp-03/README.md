# CSHARP-03 private Roslyn probes

This directory contains disposable feasibility probes for the serial
`CSHARP-03-T01` freeze work. They are developer evidence, not a frontend,
profile specification, application library, installed candidate, or release
gate. Application snippets compiled by the probes contain no MPK namespace,
assembly, attribute, interface, base type, generated source, or runtime
dependency when they are proposed admitted candidates. T01-W06 deliberately
compiles isolated rejected dependency forms as negative observations; no
snippet or emitted assembly is executed as application code.

## T01-W04 data and construction probe

`DataConstructionProbe.cs` measures only public Roslyn and .NET metadata APIs
using the exact toolchain and 167-reference projection frozen by T01-W03. Its
14 isolated compilation units cover 129 proposed admitted shapes and 52
rejected near misses for:

- expression bodies, local `var`, ordinary namespace imports, and the exact
  redundant file-wide nullable directive;
- enums, readonly structs, sealed classes, fields, properties, constructors,
  same-type delegation, pure instance calls, and overload selection;
- `init`, `required`, object initializers, implicit members, backing fields,
  accessors, compiler-owned modifiers, and compiler-owned attributes;
- nullable annotations, exact value-type `T?`, nullable members, conditional
  access, coalescing, defaults, and conversions;
- all proposed one-dimensional array forms and the rejected framework,
  generic-collection, range, stack, jagged, and multidimensional alternatives;
- string literals, ordinal calls, concatenation overloads, and restricted
  interpolation; and
- the proposed floating, decimal, date, time, duration, GUID, ordinal, and
  rounding intrinsics, including incidental generic metadata that must not
  widen the source surface.

For each compilation the result records exact source bytes and SHA-256,
diagnostics, syntax nodes/tokens/directives and UTF-16 spans, declared and
selected symbols, conversions, `IOperation` roots and target shapes, CFG
blocks/regions/edges where available, source symbol members, implicit flags,
and deterministic emitted metadata. The emitted assembly is never executed;
it is re-imported through public APIs and its ECMA-335 type/member/custom-
attribute rows and signature blobs record compiler-owned `init`/`required`
markers that Roslyn intentionally summarizes as symbol properties.

The canonical checked-in result is
`../../migrations/csharp-03/probes/roslyn-data-construction.json`. It is
5,925,271 bytes with raw SHA-256
`c5de8bc209331c2295497210a570ba0be32e0871b3dd2576980d6c109222142e`.
Every admitted shape has its own observation hash and distinct upgrade
mutation ID. Changing one observation without updating its measured hash fails
both the Python and Rust probe-schema validators.

The runner never downloads or restores anything. On native x86-64 Linux with
the W03 archive cache already provisioned:

```sh
./develop/probes/csharp-03/run-data-construction-probe.sh --check
```

`--check` materializes the frozen archives, compiles and runs the probe twice
in fresh empty-environment network namespaces, requires identical probe
binary and observation bytes, and compares the normalized result byte-for-byte
with the checked-in record. `--check-record` validates the record without
executing Linux binaries; `--self-test` exercises every admitted shape's
mutation and a changed-observation rejection. `--update` performs the same
two-run check before atomically replacing the record.

The recorded run on the ARM development host used the existing local Linux
x86-64 gate image by immutable ID, no network, a read-only container root, and
a fresh executable tmpfs:

```text
docker run --rm --platform linux/amd64 --privileged --network none --read-only --tmpfs /tmp:rw,nosuid,nodev,exec,size=4g --mount type=bind,source=<repository>,target=/workspace -w /workspace sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a ./develop/probes/csharp-03/run-data-construction-probe.sh --check
```

This probe does not freeze the final admitted operation set, semantic
templates, application bindings, profile identity, diagnostic IDs, or limits.
Those decisions remain with T01-W08 through T01-W10. Control, exception, and
pattern observations remain T01-W05-owned; dependency/generic/iterator/async
rejections remain T01-W06-owned; runtime semantics remain T01-W07-owned.

## T01-W05 control, exception, and pattern probe

`ControlExceptionPatternProbe.cs` uses only public Roslyn APIs to measure the
control forms proposed by T01-W05. Its 18 isolated compilation units contain
62 proposed admitted shapes and 41 rejected near misses covering:

- `while`, `do`, `for`, array/string `foreach`, `break`, and `continue`;
- switch statements and expressions, guards, constant/discard/`var`, null,
  relational, logical, type/declaration, property/tag, and bounded list
  patterns;
- exact standalone throws, source and built-in exception construction, typed
  catch ordering, immutable payload access, rethrow, and propagation;
- filters, filter failure, nested handler search, inner-to-outer unwind,
  normal `finally`, and return/break/continue/throw completion through
  `finally`; and
- compiler-successful exclusions, warning-bearing exclusions, and compiler-
  error near misses for jumps, enumeration protocols, patterns, switches,
  exception state/constructors, handlers, filters, and illegal `finally`
  exits.

The result records complete operation roots, 40 operation-parent decision
graphs, 25 exception-region records, public CFG blocks/edges/region trees,
abrupt completions, exact source and marker spans, and one combined source-
order sequence per compilation. Catch, arm, decision-node, throw, region, and
abrupt-completion ordinals are explicit. All 65 decision graphs and exception
regions have separately named upgrade mutations bound to their exact
observation hashes.

The canonical result is
`../../migrations/csharp-03/probes/roslyn-control-exception-pattern.json`. It
is 2,331,920 bytes with raw SHA-256
`b1215ad7f4a0e08dc269834229d7158158d31c0e9475218fa0791feea5a1629a`.
The runner is offline and consumes the exact W04 commit/result and W03
toolchain closure. On native x86-64 Linux with that archive cache present:

```sh
./develop/probes/csharp-03/run-control-exception-pattern-probe.sh --check
```

`--check-record` validates canonical schema, links, spans, source order, and
observation hashes without executing Linux binaries. `--self-test` mutates
every decision/region observation and requires full-schema rejection for both
families. `--update` runs the same two-clean-build/two-clean-run equality gate
before atomically replacing the result.

The recorded and final check use the immutable local Linux x86-64 gate image,
no network, a read-only root and repository mount, and a fresh executable
tmpfs:

```text
docker run --rm --platform linux/amd64 --privileged --network none --read-only --tmpfs /tmp:rw,nosuid,nodev,exec,size=4g --mount type=bind,source=<repository>,target=/workspace,readonly -w /workspace sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a ./develop/probes/csharp-03/run-control-exception-pattern-probe.sh --check
```

W05 remains measurement evidence only. It does not freeze decision semantics,
exception lowering, diagnostic identities, profile/schema identities, or a
production route. Its dependency/generic/iterator/async successor evidence is
recorded by W06 below; runtime behavior remains W07, and specification and
activation remain W08-W10.

## T01-W06 dependency, generic, iterator, and async rejection probe

`DependencyGenericSuspensionProbe.cs` uses only public Roslyn and public .NET
metadata APIs with the exact W03 toolchain closure and W05 result fixed as its
predecessor. Its 16 isolated compilation units record 144 source shapes in 41
deterministic families: 12 narrowly admitted exception observations and 132
rejected profile forms. The corpus covers:

- MPK package, assembly, namespace, attribute, interface, base-type,
  generated-source, project, and ambient-reference dependencies;
- source-written attributes on all relevant declaration/member/parameter
  targets versus compiler-emitted `init`/`required` metadata;
- generic declarations, methods, parameters, constraints, variance,
  explicit/inferred calls, closed/open uses, constructed framework types, and
  invalid nullable payloads;
- the exact value-type `T?` exception, its immediate closed `option`
  specialization, rejected `System.Nullable<T>`/alias/construction/cast forms,
  and non-generic arrays/reference annotations;
- exact allowlisted string, array, decimal, and date operations whose observed
  framework types carry incidental generic metadata, paired with rejected
  source-visible uses of that metadata; and
- iterator declarations, `yield`, enumeration protocols, async iterators,
  `async`/`await`, task/value-task forms, factories, continuations, schedulers,
  task races, parallel execution, custom awaiters, cancellation, lambdas/local
  functions, and emitted state machines.

Every unit records exact source bytes/hash, diagnostics, full syntax and
operation roots, selected/declared/enclosing symbols, types and converted
types, generic facts, UTF-16 marker/source order, generated sources, and
deterministically emitted metadata where compilation succeeds. Three synthetic
references have fixed virtual package/project/ambient origins. The nullable
records state the concrete payload and require `residual_type_parameter=false`;
the emitted records expose `IsExternalInit`, required-member attributes,
assembly references, and iterator/async state-machine types and attributes.

The canonical result is
`../../migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json`.
It is 4,511,101 bytes with raw SHA-256
`5dadf10613f95be9b35c108008a33474c55d222bef1be987c2614c6dcc48fe96`.
Each of the 144 shapes has an exact observation hash and a distinct substantive
upgrade mutation. Rejected units include compiler-clean profile exclusions,
one warning-only exclusion, and compiler-error near misses.

On native x86-64 Linux with the frozen W03 archive cache present:

```sh
./develop/probes/csharp-03/run-dependency-generic-suspension-probe.sh --check
```

`--check` performs two fresh empty-environment builds and runs, requires equal
binary and observation bytes, normalizes both, and compares the result
byte-for-byte with the checked-in record. `--check-record` validates the
canonical record without running Linux binaries; `--self-test` changes every
shape's selected observation and requires full-document rejection; `--update`
uses the same two-run equality gate before atomic replacement.

The recorded and final checks use the immutable local Linux x86-64 gate image,
no network, a read-only container/repository for checking, and a fresh
executable tmpfs:

```text
docker run --rm --platform linux/amd64 --privileged --network none --read-only --tmpfs /tmp:rw,nosuid,nodev,exec,size=4g --mount type=bind,source=<repository>,target=/workspace,readonly -w /workspace sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a ./develop/probes/csharp-03/run-dependency-generic-suspension-probe.sh --check
```

W06 is private measurement and rejection evidence only. It introduces no
application dependency, source-facing library, generic/iterator/async
capability, production route, registered identity, or normative vector.
Runtime behavior remains W07-owned and final specification/activation remains
W08-W10-owned.
