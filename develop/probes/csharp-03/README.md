# CSHARP-03 private Roslyn probes

This directory contains disposable feasibility probes for the serial
`CSHARP-03-T01` freeze work. They are developer evidence, not a frontend,
profile specification, application library, installed candidate, or release
gate. Application snippets compiled by the probes contain no MPK namespace,
assembly, attribute, interface, base type, generated source, or runtime
dependency.

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
