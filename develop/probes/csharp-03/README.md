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
Primitive/string/numeric/codec behavior is W07 evidence. Foundation/data
semantics are W08-owned; the remaining freeze package is W09/W10-owned and
production activation remains T07/T08-owned.

## T01-W07 primitive, string, numeric, and codec runtime probe

`PrimitiveStringNumericCodecProbe.cs` is a disposable runtime measurement
program compiled directly by the exact W03 C# 14 compiler and 167-reference
projection, then executed on the pinned .NET 10.0.11 Linux-x64 runtime. It has
no MPK application API and does not become part of a frontend, foundation
library, profile bundle, or customer project. Its finite corpus contains 3,468
vectors grouped into 154 exact operations and 26 evidence families covering:

- UTF-16 literals, surrogate pairs and lone surrogates, code-unit length and
  indexing, ordinal equality/comparison/predicates, null receivers and
  arguments, substring ranges, null/empty behavior, and constant switches;
- the exact string/string, string/char, char/string, and two-/three-/four-
  string concatenation matrix, including null-as-empty results, rejected
  char/char and object conversion, restricted string/char interpolation, and
  rejected numeric/alignment/format holes;
- independently implemented canonical ASCII parsers and formatters for every
  signed/unsigned integer width, normalized and fixed-scale decimal, DateOnly,
  TimeOnly, duration ticks, Unix milliseconds, binary32/binary64 bits, and
  lowercase GUID N/D forms, with syntax, noncanonical, range,
  scale/precision and input-bound errors, output-bound obligations, and 96
  explicit lossless/rounded-value round-trip vectors;
- exhaustive pairwise operations over nine binary32 and nine binary64 edge
  values, including infinities, both zero signs, a subnormal, quiet NaN and
  signaling NaN payloads, plus exact intrinsics and checked conversions;
- exhaustive pairwise decimal operations/comparisons over eight small-domain
  representations, decimal sign/coefficient/scale observations, five rounding
  modes, conversions, trailing-scale and signed-zero equality, division by
  zero, and arithmetic overflow; and
- parser and sidecar multi-failure vectors that fix input-bound, syntax,
  canonicality, scale/range, unknown-codec, and unknown-rounding precedence.

General framework parsing, formatting, interpolation, object conversion, and
culture-sensitive string calls appear only in the differential side of a
vector. Profile-side codec results are produced by closed ASCII grammar code;
float/double values use exact bit strings and decimal values use sign, scale,
and a 96-bit coefficient. No raw exception message, stack, path, source text,
or ambient culture name is recorded.

Each clean build executes all vectors twice under each of three explicitly
constructed hostile current cultures (`hostile-arabic`, `hostile-comma`, and
`hostile-swap`). The second execution mutates an unlisted environment input.
The runner requires equal output across that mutation and equal profile-side
values, bit strings, error IDs, and precedence across all cultures. It records
83 intentionally culture-varying BCL differential vectors without allowing
those values to define the candidate semantics.

The canonical result is
`../../migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json`.
It is 9,318,258 bytes with raw SHA-256
`0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769`.
The normalized three-culture observation section is 6,641,752 bytes with
SHA-256
`872e6150d17476c52ee01db3530f9e710afc8c6252592daa5393f3c705e46967`;
the deterministic probe binary SHA-256 is
`7b61263a2847340902b5692dd397c458a72cdd24a7b9158a8f4b3ea2279d85ed`.

On native x86-64 Linux with the W03 archive cache already provisioned:

```sh
./develop/probes/csharp-03/run-primitive-string-numeric-codec-probe.sh --check
```

`--check` performs two fresh builds and twelve isolated runtime executions,
requires deterministic binary/raw/normalized bytes, and compares the result
byte-for-byte with the record. `--check-record` validates the canonical record
and every live predecessor/input hash without running Linux binaries.
`--self-test` mutates one recorded input per operation to check its semantic
hash binding, rejects missing/extra culture runs, and changes representative
result-bit, rejection-ID, parse-error, and precedence observations. It does
not substitute for the actual input/edge cases executed by `--check`.
`--update` applies the same full two-build gate before an atomic replacement.

The recorded and final check use the immutable local Linux-x64 gate image, no
network, a read-only root, and a fresh executable tmpfs:

```text
docker run --rm --platform linux/amd64 --privileged --network none --read-only --tmpfs /tmp:rw,nosuid,nodev,exec,size=4g --mount type=bind,source=<repository>,target=/workspace,readonly -w /workspace sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a ./develop/probes/csharp-03/run-primitive-string-numeric-codec-probe.sh --check
```

W07 remains private measurement evidence. It does not freeze the final
foundation descriptor, operation allowlist, public diagnostics, profile or
schema identity, VIR/VC semantics, source-facing library, production route,
or activation. The normative freeze package begins in T01-W08 and remains
serial through T01-W10; production implementation and atomic activation
belong to the later implementation/release milestones.

## T01-W08 foundation/data semantic freeze and runtime differential

`develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md` freezes the candidate's twelve
templates, four non-template definitions, concrete closure/hash/ordering rules,
binding obligations, source construction and ownership semantics, total-order
matrix, nullable operations and business-value operations. The descriptor at
`develop/migrations/csharp-03/foundation/foundation-descriptor.json` binds that
document and `foundation-definitions.json`; it is not installed or activated.

The private `foundation_model.py`/`foundation_package.py` model generates 2,051
vectors at `develop/specs/vectors/csharp-practical-foundation-v1.json`. It checks
strict descriptor and binding fields, hashes, type substitution and dependency
closure, provenance merging, count/depth boundaries, missing/extra/colliding
entries, source cycles, residual generics, projection losses and operation
signature mismatch, recursive default eligibility, constructor transactions,
ownership joins/publication and ordered-collection failure precedence. Each
row names its later primary implementation/test owner; a passing model does
not complete that implementation or prove a source invariant.

`FoundationDataProbe.cs` is an observation-only program, compiled directly
under the exact W03 C# 14/167-reference closure and .NET 10.0.11 Linux-x64.
`foundation_runtime_model.py` independently constructs its inputs and expected
Gregorian/integer/Boolean/IEEE-bit/decimal-value results. There are 1,629 vectors
and 82 operation groups: date/time/duration/GUID, the closed lifted nullable
matrix, construction/evaluation order, arrays and count-then-allocate, plus
fixture-owned instant/money outcomes and explicit rounding/error precedence.
Runtime setup/parsing/instrumentation is not an application API admission.
W07's existing 3,468 vectors remain the sealed scalar/codec evidence.

The runner compiles twice, each time executes under two constructed hostile
cultures and two unlisted environment values, and requires equal binaries,
raw observations and independently calculated expectations. The final record
is `develop/migrations/csharp-03/probes/runtime-foundation-data.json` with raw
SHA-256 `6ef1194e1398d5822c676248ea6ccbbb31381b95cfd32c8b8a65e68376118064`.
It records source/oracle/runner/build-input hashes, all exact input rows,
canonical values/errors and deterministic binary hash. No raw exception prose,
machine path, stack, clock or ambient-culture formatting enters observations.

Portable local checks:

```sh
python3 develop/probes/csharp-03/foundation_package.py --check
python3 develop/probes/csharp-03/run-foundation-data-probe.py --check-record
cargo test -p mpk-vc --test csharp_practical_spec
python3 scripts/check-spec-vectors.py --check
```

The full repeatability check uses the existing immutable local Linux image and
W03 offline archive cache; `<repository>` is the absolute repository path:

```text
docker run --rm --platform linux/amd64 --privileged --network none --read-only --tmpfs /tmp:rw,nosuid,nodev,exec,size=4g --mount type=bind,source=<repository>,target=/workspace,readonly -w /workspace sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a python3 develop/probes/csharp-03/run-foundation-data-probe.py --check
```

`--check-record` re-derives every expected observation and verifies all live
input hashes without running a Linux binary. `--check` also repeats the two
builds/eight executions and compares the entire record byte-for-byte.
`--update` is an explicit generated-record operation, requiring the same runtime
agreement first and a writable repository mount. The package model likewise
has explicit `--update` for its three generated JSON files, and `--emit-patch`
for review; update the normative vector manifest hash after regeneration.
Neither check silently repairs changed data. No production route, source-facing
library, active profile or release artifact is changed by W08.

## T01-W09 feasibility, capacity, and private freeze

`recursor_feasibility.rs` and `run-recursor-probe.py` retain
`CSHARP-03-T01-W09-F01`: the unchanged checked `Std.Bool.rec` / `Std.Nat.rec`
interfaces reject W08's original cross-result applications. The amended W08
carrier uses little-endian Bool address binders, pointwise Bool selection, and
static composition of concrete state transformers; it never uses `Nat.rec` in
the replacement. The canonical result is
`develop/migrations/csharp-03/probes/recursor-feasibility.json`; details and
the resolved finding are in `develop/migrations/csharp-03/w09-recursor-stop.md`.
The 15 cases run twice through both checkers. Ten controls/replacement cases,
including a two-address cube and cross-coordinate function-valued static fold,
accept per run; the five retained old applications reject with the expected
type mismatch. Every replacement case also recomputes a zero direct
`Std.Nat.rec` application count from its term table.

The runner's explicit `--update` regenerates evidence only. F01's type-
feasibility gate is resolved without a core change. The record now links the
separate completed capacity evidence rather than claiming capacity from the
small feasibility cases.

`run-checker-capacity-probe.py` generates twelve self-contained, reachable,
ordinary-term Certificate v0 cases: limit-minus-one, limit, and limit-plus-one
for 256 binder depth, 8,192 successor-generated declarations, 262,144 total
terms, and 16,384 balanced concrete state transformers. It feeds identical
bytes twice to the Rust and Go checkers. All 48 invocations accept, remain
axiom-free, and contain no proof node or theory certificate. The profile keeps
the inclusive ceilings and rejects each plus-one case before checker
invocation, so the evidence demonstrates checker headroom rather than widening
checker acceptance.

The canonical capacity record is
`develop/migrations/csharp-03/probes/checker-capacity.json` (38,701 bytes, raw
SHA-256
`de040d4342e90a23e4bbe6464aeaccbfa9f2630c1423b77b716b40c805ac8a99`).
Its 73-file source inventory SHA-256 is
`e855ce008b87b4509a8af7d3b07ce5f907f9a98383b942710d362a146a2d0e38`.
Across the retained two runs, Rust checker observations range from 35 to 1,814
ms and Go checker observations from 26 to 397 ms under the per-invocation
60-second failure bound. Timing is observational and excluded from the stable
rerun comparison; certificate hashes, verdicts, exits, and output hashes must
match.

`profile_freeze.py` generates the private implementation handoff and its 700
vectors. `profile-freeze.json` contains all 17 W02 identity families, unique
successor names/domains, exact producers/consumers, 15 strict root schemas, 20
strict nested records, three closed tagged unions, the field-complete typed
expression union, canonical JSON, context/frontend/boundary linkage, transition
and idempotency precedence, one context-dispatched `csharp2vir` successor bundle,
29 diagnostic families, termination rules, 35 practical counters, and the 32
unchanged scalar-v0 limits. It is 97,316 bytes, has content SHA-256
`f292de00a79048ecd1ff2cbe52d90fad36654f1b3e74ad580b5ec3077afa28cb`
under `MPK-CSHARP-PRACTICAL-FREEZE-1.0`, and raw SHA-256
`83954067c156e58cb349dbf07da44edf60a3ec550e628e6d2f1a890889d574e3`.
`profile-freeze-vectors.json` is 230,986 bytes with raw SHA-256
`7d1de4f4d087fe0de7b32ec44ee2b17f08cbfb052e5993699137c47736c94ef3`.

Portable reproduction commands are:

```sh
python3 develop/probes/csharp-03/run-recursor-probe.py --check
python3 develop/probes/csharp-03/run-checker-capacity-probe.py --check
python3 develop/probes/csharp-03/profile_freeze.py --check
cargo test -p mpk-vc --test csharp_practical_spec --test csharp_practical_inventory
```

The freeze and vectors remain private migration evidence. W09 did not alter
production code, core/checker behavior, the installed registry/release, public
routes, or an external company's application source or build output. W10
publishes them through the separate package below.

## T01-W10 normative publication package

`profile_package.py` publishes the W09 handoff without changing a frozen value.
It produces
`develop/specs/vectors/csharp-practical-profile-v1.json`, schema
`mpk.csharp.practical.profile.conformance.v1`, owned by
`develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md` and
`crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10`. The companion
`develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md` defines the mandatory
successor migration. The vector manifest pins the package raw SHA-256.

The package copies the complete W09 contract and the same 700 sorted vectors,
then adds 16 canonical W01-W09 evidence hashes, ten freeze requirement owners,
63 downstream T02-T08 work-item/test-owner pairs with their exact owns/exit/
verification contracts, flattened name/schema/diagnostic/limit ownership,
four upgrade-observation sets, twelve excluded
families, the historical W02 inventory extension, and the future release-gate
decision. The only inventory extension is the two specifications and this
vector file; it cannot hide another added consumer from the W02 fingerprints.

Reproduce or explicitly regenerate it with:

```sh
python3 develop/probes/csharp-03/profile_package.py --check
python3 develop/probes/csharp-03/profile_package.py --update
python3 scripts/check-spec-vectors.py --check
cargo test -p mpk-vc --test csharp_practical_spec --test csharp_practical_inventory
```

`--check` is read-only and fails on any source, specification, owner, design
projection, or generated-byte drift. `--update` atomically rewrites only the
published JSON; the manifest digest must then be updated explicitly and
reviewed. Neither mode installs a profile or changes application code.

The future command is exactly
`sudo ./scripts/check-csharp-practical-release.sh`. T07-W05 creates it
privately, T07-W06 records its receipt, and T08-W10 atomically replaces and
retires `scripts/check-java-frontend.sh`. Until then the Java-named gate and
active revision-3 registries are unchanged, the practical gate path is absent,
and this normative package is inactive.
