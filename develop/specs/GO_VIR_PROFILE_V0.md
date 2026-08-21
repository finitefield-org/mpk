# Go VIR Profile v0 Specification

Status: normative and frozen for implementation.

This specification defines the complete source-language boundary for Go
programs lowered to `mpk.vir.v0` with semantic profile
`mpk.go.fixed.v0`. `GO_SUBSET_V0.md` and GIR remain the current pre-cutover
release contracts and are not historical while this profile is staged. At the
atomic GIR-to-VIR cutover this specification replaces their active helper-path
meaning; only then do they become historical migration records. Post-cutover
components do not accept those records as inputs or derive defaults from them.

## 1. Conformance, dependencies, and precedence

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
the frontend returns no VIR, source map, source manifest, VC, skeleton, or
partial downstream artifact. A construct not explicitly accepted here rejects
before public artifact emission. The frontend never approximates an unknown Go
construct, trusts a compiler-local identity, or emits an `Unsupported` node.

This profile depends normatively on:

- `VIR_V0.md` for the IR schema, value equations, safety checks, identifiers,
  graph rules, normalized contracts, hashes, and shared IR limits;
- `VC_V1.md` for the source-language-neutral obligation meanings produced from
  those operations, calls, returns, and loop contracts;
- `FRONTEND_PROTOCOL_V0.md` for request/response framing, phases, statuses,
  normalized issues, and transport limits;
- `RELEASE_BUNDLES_V0.md` for registered frontend, Go distribution, target
  library, native runtime, and execution-host identities;
- `SOURCE_MANIFEST_V0.md` for immutable input records and the exact
  `GoFixedConfiguration` object; and
- `SOURCE_MAP_V0.md` for total mapping and source/synthetic origins.

Those shared specifications own their serialized shapes. This document owns
which Go source and loader states may produce them. If a shared rule and a Go
source rule appear to conflict, the stricter rule applies. A change to any
accepted construct, loader input, environment entry, lowering rule, diagnostic
code, or limit requires a new Go profile ID; it cannot silently amend
`mpk.go.fixed.v0`.

## 2. Profile, request, release, and target

The only pairing owned here is:

```text
source_language  = go
semantic_profile = mpk.go.fixed.v0
```

The request contains the exact `GoSelection` from `FRONTEND_PROTOCOL_V0.md`:

```json
{"package":"example.com/mpk/vector","function":"example.com/mpk/vector.Identity"}
```

`package` is the compiler-resolved import path of the selected package.
`function` is the canonical public-artifact ID of a source-declared Go function
or value-receiver method, is a member of that package, and resolves exactly
once after type checking. Public-artifact identity does not require the Go
declaration name to be exported. Neither selection value is derived from a
filesystem path or package name.
After type checking, zero matches reject at subset with
`GO_SELECTION_FUNCTION_MISSING` and more than one match rejects there with
`GO_SELECTION_FUNCTION_AMBIGUOUS`; neither condition is repaired by a
package-name or filesystem fallback. These two non-success issues use the
already validated request's exact `selection.function` as their required
`function_id`.

`selection.package` and the unit prefix of `selection.function` cannot contain
the literal substring `...` or equal the Go command's reserved package-pattern
names `main`, `all`, `std`, or `cmd`. Although the shared Go unit grammar can
represent those bytes, `packages.Load` gives them non-literal meanings. The
structural preflight rejects such a spelling at capture with source-error
`GO_PACKAGE_AMBIGUOUS` before any loader call.

Before launch, the generic runner resolves exactly one registered release
tuple for `go`, `mpk.go.fixed.v0`, the caller's target, frontend bundle, and Go
toolchain bundle. The tuple supplies immutable `pointer_width` and limit
profile values. The frontend repeats, but does not select, the resulting
semantic parameters:

```json
{"target_id":"linux/amd64","pointer_width":64}
```

The canonical `target_id` is split at its sole slash. Its left component is
the exact `GOOS`; its right component is the exact `GOARCH`. No alias, host
default, `runtime.GOOS`, `runtime.GOARCH`, executable build target, or
environment fallback is consulted. The selected target-library descriptor
fixes `pointer_width`; `go/types.SizesFor("gc", GOARCH)` and every loaded
package's `TypesSizes` must agree with it or the frontend returns artifact-free
`frontend-error` at `typecheck` with `GO_FRONTEND_TOOLCHAIN`. Target and
pointer width remain hash inputs even though accepted source integer types have
explicit widths.

The `mpk.go.fixed.v0` target allowlist is exactly one tuple:

```text
target_id    = linux/amd64
pointer_width = 64
GOOS         = linux
GOARCH       = amd64
GOAMD64      = v1
```

A release registry may describe other Go target libraries, but this profile
does not select them. Adding a target or changing its architecture tuning is a
semantic-profile revision, not a registry-only amendment. All other Go
architecture tuning variables are absent because no other `GOARCH` is valid.

The selected toolchain's `CompilerIdentity` is the complete Go release
identity. The synthetic conformance release is `go1.25.0`; production accepts
only an exact release identity present in the installed validated registry.
`GOTOOLCHAIN=local` forbids download or automatic toolchain switching.
The selected Go frontend descriptor has no subordinate binary and has exact
`limit_profile_id = mpk.vir.limits.v0`,
`environment_profile_id = mpk.go.frontend_environment.v0`, and
`argument_profile_id = mpk.go.frontend_arguments.v0`.

### 2.1 Closed launcher argument profile

For an evidence-capable launch, the argument array after `argv[0]` has exactly
this order and shape:

```text
lower
/mpk/source
--package <selection.package>
--semantic-profile mpk.go.fixed.v0
--target linux/amd64
--function <selection.function>
--frontend-bundle-id <validated frontend bundle ID>
--frontend-sha256 <validated main executable digest>
--release-registry-id <validated registry ID>
--release-registry-sha256 <validated registry digest>
--toolchain-bundle-id <validated Go toolchain bundle ID>
--toolchain-root /mpk/toolchain
--toolchain-distribution-sha256 <validated distribution digest>
--contract <portable relative contract path> ...
```

Each displayed line after the positional source root is two argv elements.
There are zero through 128 `--contract` pairs, sorted by normalized path
bytes. Their paths are unique by bytes and ASCII case folding and use the
manifest portable-path grammar. The runner sorts caller contract options before
launch; caller order is not semantic. The sum of each post-`argv[0]` element's
UTF-8 byte length plus one terminator byte is at most 262,144 and is checked
before spawn. An execution-host profile unable to carry that fixed maximum is
sandbox-unavailable; a response file or shorter host-derived ceiling is not a
fallback. Exceeding either explicit-argument ceiling is a pre-launch
configuration error; the independent discovered-candidate ceiling in section
13 remains a capture-phase profile limit. All other flags, positional values,
aliases,
option reorderings, duplicate singleton options, defaults, environment-based
options, response files, and trailing arguments reject as a pre-launch
configuration error. Digests are exact lowercase `Sha256` values and every ID
and digest equals the already validated release tuple; the frontend repeats
but never resolves a registry or selects a bundle from these private values.
This complete shape is `mpk.go.frontend_arguments.v0`.

## 3. Immutable source-root capture

### 3.1 Root and namespace capture

The caller supplies a directory capability, not a public path string. The
launcher opens it without following a link and materializes one private,
read-only snapshot at logical `/mpk/source`. The snapshot root is the selected
module root and contains a root `go.mod`. Absolute host paths, device/inode
numbers, owners, permissions other than the regular/directory/link class,
timestamps, and directory enumeration order never enter public artifacts.

Before invoking any Go tool, the capture phase builds the exact import-closure
snapshot without executing `go list` against the caller's tree:

1. Open and capture root `go.mod`, optional root `go.sum`, and any root
   `go.work`/`go.work.sum`; parse the module path under section 3.2.
2. Derive the selected package directory by removing the module-path prefix
   from `selection.package`. Open every path component without following a
   link; no filesystem search or `./...` expansion is allowed.
3. Visit package import paths from a UTF-8-byte-ordered worklist. In each
   directory, record its complete entry-name/file-kind set, but open and
   capture only non-test `.go` files, compiler auxiliary candidates below, and
   contract candidates from section 5. Parse package/import clauses from the
   captured `.go` bytes with the selected Go parser.
4. Every normal import must be under the root module path. Derive its package
   directory by the same segment rule, add it to the worklist, and repeat.
   Special and forbidden imports receive their section 12 code before loader
   execution. An import cannot redirect directory selection.
5. Re-enumerate every visited directory and revalidate every opened file before
   sealing. A changed name/type/size/digest or path resolution rejects.

The selected package and every derived same-module package path are checked
against the registered target-library package inventory before loader
execution. Equality with a standard/toolchain package import path, or any path
beginning `cmd/`, rejects in the source phase with `GO_SUBSET_IMPORT`; a source
module cannot shadow the toolchain namespace.

That package inventory is derived rather than added as an undeclared registry
field. It is the set of slash-normalized directory paths relative to `go/src`
in the selected toolchain's already validated complete root inventory for
which at least one regular inventory entry has a basename ending in `.go`.
The set is sorted by UTF-8 bytes. Derivation reads no distribution path and
runs no Go command; the inventory bytes and its content identity are already
part of the selected toolchain identity.

Within each visited package, filename/build-constraint and forbidden-directive
checks complete before its parsed imports can extend the worklist. Therefore a
target-selected file is `GO_BUILD_CONSTRAINT`, not a diagnostic induced by an
import that would have been inactive on some host.

A non-test `.go` candidate whose basename begins with `.` or `_` rejects with
`GO_BUILD_CONSTRAINT` before parsing; the Go tool's silent ignored-file rule
cannot create an unlisted or content-dependent input class. Names ending in
`_test.go` remain governed by the explicit exclusion below. After parsing only
the package clause, a candidate declaring the Go tool's special
`package documentation` also rejects with `GO_BUILD_CONSTRAINT` before its
imports are examined.

The candidate suffixes are `.go` and the Go compiler auxiliary suffixes `.c`,
`.cc`, `.cpp`, `.cxx`, `.m`, `.h`, `.hh`, `.hpp`, `.hxx`, `.f`, `.F`, `.for`,
`.f90`, `.s`, `.S`, `.sx`, `.swig`, `.swigcxx`, and `.syso`. Contract
candidates use section 5.
All name tests are ASCII byte tests. A visited directory or opened candidate
that is a symbolic link, junction, reparse point, socket, device, FIFO, or
other non-regular object rejects with `GO_CAPTURE_FILE_KIND`.

Every visited directory component and every opened candidate has one
source-root-relative path satisfying the portable path grammar in
`SOURCE_MANIFEST_V0.md`. Opened candidate paths are unique by bytes and by
ASCII case folding. A nonportable path, case-fold collision, or path that
cannot be represented without rewriting rejects during capture with
`GO_CAPTURE_PATH`; the frontend never sanitizes a source identity.

The snapshot copies each opened byte sequence once into sealed storage,
computes its manifest digest from that buffer, and never rereads the caller's
tree. A namespace or content change during capture rejects with
`GO_CAPTURE_CHANGED`. Hard-linked regular files are independent captured path
entries; link identity is not semantic. On success, every captured file is
exactly one manifest input from section 5; there is no private unlisted source
file.

A `go.mod` entry below the root is a nested-module boundary. If the import
worklist would cross it, metadata rejects with `GO_MODULE_POLICY`; otherwise
neither that file nor its subtree is opened and it cannot affect the result. No
ancestor search is performed outside the root capability. An unrelated
directory is never visited merely because it is below the module root.

Files outside the visited candidate set are absent from the private loader
view and cannot affect loading. A `//go:embed`, `//go:generate`, or any other
`//go:` source directive not explicitly used for a rejected build constraint
is itself rejected with `GO_SOURCE_DIRECTIVE`; the loader never opens an
embedded asset. Go `//line` and `/*line ...*/` directives also reject with
that code so source identities and byte spans cannot be rewritten by source
text. Names ending in `_test.go` are excluded by name before open, capture,
parsing, candidate counting, or snapshot construction and cannot affect the
result because `Tests` is exactly false.

### 3.2 Module and workspace policy

The selected root `go.mod` is a required `build_manifest` input. It must be
valid under the selected Go parser and contain exactly one `module` directive
and exactly one `go 1.23` directive. The module path obeys the Go unit-ID
grammar in `VIR_V0.md`, is a prefix of `selection.package` at a segment
boundary, and is not rewritten. Comments are permitted and remain input bytes.

`require`, `replace`, `exclude`, `retract`, `toolchain`, `tool`, and `godebug`
directives reject with `GO_MODULE_POLICY`. A root `vendor` directory rejects,
as do vendoring metadata and an import resolved outside the root module. This
profile has no external module dependency.

A root `go.sum` is optional. When present it is a `lockfile` input and must be
zero bytes; a nonempty file rejects with `GO_MODULE_DEPENDENCY`. Root
`go.work` and `go.work.sum` always reject with `GO_WORKSPACE_FORBIDDEN`.
`GOWORK=off` additionally makes any host or ancestor workspace invisible.

Module resolution is read-only. The source snapshot is read-only,
`BuildFlags` contains `-mod=readonly`, the module cache is a sealed empty
directory, and network access is absent. A successful run reads no module-cache
byte. Any attempted module lookup, checksum lookup, VCS command, proxy access,
or source/update write rejects; it cannot fall back to an ambient cache.

### 3.3 Complete external-byte inventory

The following table is exhaustive for bytes and namespace facts that can
affect successful package loading.

| Source | Treatment |
|---|---|
| selected root and every visited directory's complete entry-name/file-kind set, plus loader-relevant candidates | namespace sets are counted and revalidated; candidates are captured once; every opened successful file is a manifest input |
| root `go.mod` and optional empty `go.sum` | captured `build_manifest`/`lockfile` inputs |
| every package `.go` file in the selected same-module import closure | captured `source` inputs |
| used `mpk.go.contract.v0` files | captured `contract` inputs |
| frontend executable and its linked `golang.org/x/tools/go/packages` implementation | complete selected frontend-bundle inventory and binary digest |
| directly invoked `go` executable, `GOROOT` support data, release metadata, compiler tools, and target standard-library/export/source bytes | complete selected toolchain distribution, component, and target-library inventories |
| dynamic loader/shared libraries, if any | selected native-runtime inventory and execution-host profile |
| target, pointer width, package/function selection, registry assertions, and bundle IDs | prevalidated request/release identities repeated in artifacts |
| environment, argv, logical paths, cache initial state, filesystem capabilities, and network denial | exact sections 2.1 and 3.4 through 3.6 |

There are no other successful inputs. In particular, host environment values,
host Go installations, `PATH` search outside the sandbox, user/project Go
configuration, credentials, VCS configuration, DNS, proxy responses, clocks,
locale databases, timezone databases, random values, host module/build caches,
test files, unrelated assets, compiler stderr, and prior invocation outputs
cannot affect a successful hash. If the selected Go executable reads a
distribution file, that byte is already covered by the registered complete
toolchain inventory; an undeclared read is a sandbox failure, not a new input.

### 3.4 Exact child environment

The launcher constructs the Go frontend process environment from an empty
array by adding the following complete base key/value array, sorted by key.
The frontend passes that same array as `packages.Config.Env`:

| Key | Exact value |
|---|---|
| `CGO_ENABLED` | `0` |
| `GO111MODULE` | `on` |
| `GOAMD64` | `v1` |
| `GOARCH` | selected target right component |
| `GOCACHE` | `/mpk/cache/go-build` |
| `GODEBUG` | empty string |
| `GOENV` | `off` |
| `GOEXPERIMENT` | empty string |
| `GOFLAGS` | empty string |
| `GOMAXPROCS` | `1` |
| `GOMODCACHE` | `/mpk/cache/go-mod` |
| `GONOPROXY` | empty string |
| `GONOSUMDB` | empty string |
| `GOOS` | selected target left component |
| `GOPACKAGESDRIVER` | `off` |
| `GOPATH` | `/mpk/gopath` |
| `GOPRIVATE` | empty string |
| `GOPROXY` | `off` |
| `GOROOT` | `/mpk/toolchain/go` |
| `GOSUMDB` | `off` |
| `GOTELEMETRY` | `off` |
| `GOTOOLCHAIN` | `local` |
| `GOVCS` | `*:off` |
| `GOWORK` | `off` |
| `HOME` | `/mpk/empty/home` |
| `LANG` | `C` |
| `LC_ALL` | `C` |
| `PATH` | `/mpk/toolchain/go/bin` |
| `TMPDIR` | `/mpk/tmp` |
| `TZ` | `UTC` |

No duplicate base key is permitted. Empty means the serialized environment
entry ends immediately after `=`. With `packages.Config.Dir = /mpk/source`,
the pinned `go/packages` runner appends exactly one final
`PWD=/mpk/source` entry after that base array. Except for the fixed probe
below, every Go child observes the base entries in the table followed by that
fixed `PWD`; `PWD` is not included in `packages.Config.Env` and is never
inherited from the host. The runner's sole environment exception is its
toolchain release-tag probe, whose argv is exactly
`["go","list","-e","-f","{{context.ReleaseTags}}","--","unsafe"]`: it
clones the base array, appends `GO111MODULE=off`, and then appends the fixed
`PWD`. The final
duplicate `GO111MODULE` value is effective only for this module-independent
toolchain probe. Source-module loading and target-size discovery retain the
base `GO111MODULE=on`. No other child adds, removes, replaces, or duplicates
an environment entry.
`OLDPWD`, `USER`, `LOGNAME`, `SHELL`, `TERM`, `XDG_*`, CI variables,
credential variables, and all other inherited entries are absent.
`CGO_ENABLED=0`, explicit `GOOS`, explicit `GOARCH`, the fixed child `PWD`,
and construction from an empty inherited environment are invariants, not
caller options.

### 3.5 Logical filesystem and caches

`/mpk/source`, `/mpk/frontend`, `/mpk/toolchain`, `/mpk/native-runtime`,
`/mpk/cache/go-mod`, `/mpk/gopath`, and `/mpk/empty/home` are read-only. Every
private interpreter or library destination mounted from the selected
`NativeRuntimeLayoutProfile` is also read-only and resolves only bytes from
`/mpk/native-runtime`; no host runtime path remains visible. The module cache,
GOPATH, and home directories start empty. `/mpk/cache/go-build` and `/mpk/tmp`
are fresh empty private writable directories for each invocation, are not
shared, and are discarded. Only those two directories may gain files. Network,
process inspection, host mounts, and writes elsewhere are denied. The
registered executable is invoked by its already-open capability; the fixed
`PATH` exists only for its registered Go child tools.

Cache contents and private logical paths never enter VIR, source maps,
manifests, issues, or hashes. Repeating a request with empty versus populated
host caches is therefore observationally identical. A Go child attempting to
write `go.mod`, `go.sum`, the module cache, home, or toolchain causes
artifact-free `frontend-error` `GO_FRONTEND_SANDBOX`.

### 3.6 Exact package-loader configuration

The frontend uses the `go/packages` implementation linked into the registered
frontend binary. Its public configuration is exactly:

```text
Dir        = /mpk/source
Env        = section 3.4, in key order
Mode       = NeedName | NeedFiles | NeedCompiledGoFiles | NeedImports |
             NeedDeps | NeedModule | NeedTypes | NeedSyntax |
             NeedTypesInfo | NeedTypesSizes
BuildFlags = ["-mod=readonly"]
Tests      = false
Overlay    = nil
ParseFile  = nil
Logf       = nil
```

The sole `packages.Load` pattern is the exact `selection.package`; `./...`,
wildcards, file lists, multiple patterns, and command-line build tags are
forbidden. The frontend independently validates that exactly one root package
matches and that its `PkgPath` equals the selection. Package-loader IDs,
absolute filenames, and map iteration order are never public identities.

For the selected Go release, the package query's effective fixed flags are
`compiled=true`, `test=false`, `export=false`, `deps=true`, `find=false`,
`pgo=off`, and module mode `readonly`; `-mod=readonly` is the sole build flag.
The runner's fixed release-tag and target-size probes address only `unsafe` and
cannot add a source package to the closure. `GOPACKAGESDRIVER=off`, `Overlay =
nil`, the one literal load pattern, and the registered frontend implementation
close all other query/argument paths.

For every package admitted to the closure, `GoFiles` and `CompiledGoFiles`
must be equal after resolving both to captured regular files and sorting by
normalized path. Every non-test `.go` candidate in that package directory must
appear in both lists. A mismatch, `IgnoredFiles`, target/build filename suffix,
`//go:build`, legacy `// +build`, cgo-generated file, external file, overlay,
or compiler-synthetic source rejects with `GO_BUILD_CONSTRAINT`. Auxiliary
compiler files in section 3.1 reject with `GO_CGO_OR_AUX_SOURCE`.

Loader findings are collected only after the fixed call returns. Compiler
diagnostics are normalized as section 12 specifies; raw `go` text is not
returned. Package records, files, imports, declarations, and findings are
sorted by the canonical rules below before any public construction.

## 4. Package and function closure

The emitted package closure is deliberately conservative to preserve the
historical package-wide fail-closed boundary:

1. Start with the exact selected package and compute the complete same-module
   syntactic import closure by section 3.1.
2. Include every source-declared function and value-receiver method with a body
   in every closure package. Each included package must supply at least one
   function, as required by the shared VIR unit lower bound.
3. Resolve every direct call in every included body using `go/types`; every
   callee must already belong to the import closure.
4. Include every referenced accepted type and constant declaration required by
   those functions when it belongs to the function's own package. Reject an
   unresolved declaration or dependency cycle.

Thus an unsupported declaration in an included package still rejects even if
the selected function cannot reach it. A package outside this closure is not a
VIR unit. Standard-library imports, external-module imports, blank imports,
dot imports, and import cycles reject. `C`, `unsafe`, and `reflect` retain the
specific cgo, unsafe, and reflection rejections. A normal import alias must be
an `AsciiIdent`; aliases remain source-only.

The module call graph must be acyclic. Each `CallStatic` resolves inside the
closed VIR module, targets a function with exactly one result, repeats its
recomputed contract hash, and passes exact typed arguments. A value-receiver
method call is direct, remains inside its receiver's unit, and supplies the
receiver as `arg0`; method expressions, method values, cross-unit methods,
pointer receivers, interfaces, and dynamic dispatch reject.

Because a VIR `struct` type resolves in its containing unit, a cross-unit call
signature may contain only bool/BV types and arrays transitively composed of
those types. Using an imported package's type or constant, or passing/returning
an imported struct across a call, rejects with `GO_SUBSET_IMPORT`. A package
import is therefore accepted only for direct static free-function calls. Each
included package may still use its own structs and constants internally.

VIR units are sorted by canonical import path. Functions use the module-wide
callee-first Kahn order and ID tie-break from `VIR_V0.md`. Package discovery,
import maps, type maps, and filesystem order cannot change that result.
Each `VirUnit.id` is the exact compiler-resolved import path and each
`VirUnit.name` is the single type-checked package-clause name shared by its
captured source files; neither is derived from the final directory segment.

## 5. Source and contract manifest inventory

On success, the source manifest's `inputs` array is exactly:

- root `go.mod` as `build_manifest`;
- root `go.sum` as `lockfile` if and only if the empty file exists;
- every `GoFiles`/`CompiledGoFiles` member of every included package as
  `source`; and
- every contract sidecar attached to an included function as `contract`.

No test, ignored, nested-module, auxiliary, unused asset, cache, toolchain, or
generated file appears. Paths are `/mpk/source`-relative portable paths and
are sorted under `SOURCE_MANIFEST_V0.md`. Each entry's size and digest come
from its single immutable buffer. The module and all source files are nonempty;
the profile explicitly permits only `go.sum` to be an empty non-source input.

Contract discovery scans the included package directories in normalized-path
order and matches regular filenames case-insensitively against exactly:

```text
contract.json
*.contract.json
*_contract.json
```

A path matching more than one suffix is one candidate. Every candidate must
appear exactly once in the normalized `--contract` path set from section 2.1,
and every listed path must be one of those discovered candidates. Set mismatch,
an out-of-directory/nonmatching listed path, or an unused candidate rejects in
the subset phase with `GO_CONTRACT_FUNCTION`; option order cannot select or
hide a file. Every matched candidate must parse and attach to exactly one
included function; ambiguous or duplicate attachments reject. A function with
no candidate receives the default contract in section 10.4 and therefore has
no contract input entry.

The manifest's language configuration is member-for-member equal to:

```json
{"kind":"go","compiler":"gc","cgo_enabled":false,"go111module":"on","module_mode":"readonly","workspace_mode":"off","tests":false,"build_tags":[],"environment_profile_id":"mpk.go.frontend_environment.v0","argument_profile_id":"mpk.go.frontend_arguments.v0"}
```

Manifest `selection`, `units`, `target`, release identities, profile,
parameters, hashes, and limit profile follow `SOURCE_MANIFEST_V0.md`; the
manifest limit profile is exactly `mpk.vir.limits.v0`. `GOOS`
and `GOARCH` are not repeated in this configuration because they are exactly
recoverable from `target.id` and are already fixed by sections 2 and 3.4.

## 6. Accepted declarations and types

### 6.1 Declarations

An included package accepts only:

- top-level `const` declarations whose fully resolved values have `bool` or an
  explicit accepted fixed-width integer type;
- named, non-alias struct type declarations with accepted fields;
- top-level functions with bodies whose name is not `init`; and
- methods on a named non-pointer struct receiver.

Package variables, `init`, declarations without bodies, type aliases, named
Boolean/integer/array types, embedded fields, generic declarations or
instantiations, and all other declaration forms reject. Struct tags may be
present; they are source bytes but have no meaning because reflection is
forbidden and do not enter VIR. All public identities, package names,
declaration names, receiver type names, field names, and source bindings must
obey the relevant ASCII identifier grammar in `VIR_V0.md`.

An accepted `const` value specification has exactly one nonblank name, an
explicit `bool` or accepted fixed-width integer type, and exactly one
initializer. Its compile-time expression is composed only of accepted
Boolean/integer literals, same-package accepted constants, parentheses, the
applicable section 9 unary/binary operators, and exact BV conversions. It must
resolve to a fitting value of the declared type. Go constant evaluation is
arbitrary precision until that final type check; it emits one canonical
`VirConstDecl` literal and no program instruction or safety check. `iota`,
implicit expression repetition, an omitted type, and multi-name specifications
reject.

The canonical public identities are:

```text
top-level function  UNIT_ID "." FUNCTION_NAME
value method        UNIT_ID "." RECEIVER_TYPE "." METHOD_NAME
struct or constant  UNIT_ID "." DECLARATION_NAME
```

A receiver, parameter, or result name may be absent or the Go blank identifier;
it then creates no source-name binding and is not contract-visible, while its
positional `argN` or `resultN` still exists. Every present nonblank source
binding obeys `AsciiIdent`. The blank identifier is otherwise accepted only in
the explicit `_ = expression` statement from section 7; it cannot name a
local, field, type, constant, function, or public identity.

### 6.2 Types

Accepted source value types are exactly:

- `bool`;
- `int8`, `int16`, `int32`, and `int64`;
- `uint8`, `uint16`, `uint32`, and `uint64`;
- the predeclared aliases `byte` and `rune`, normalized respectively as
  `uint8` and `int32`;
- `[N]T`, where `0 <= N <= 256` and `T` is accepted; and
- a named struct whose zero through 64 fields have accepted types and whose
  aggregate nesting is at most 16.

They map to `bool`, exact signed/unsigned `bv`, structural `array`, and nominal
`struct` VIR types. `int`, `uint`, and `uintptr` reject even when their host or
selected-target width happens to be known. Named primitive and named array
types reject because VIR v0 would erase their Go nominal assignment identity.

Pointers, `unsafe.Pointer`, slices, maps, strings, floating-point, complex,
interfaces, type parameters, channels, functions as values, tuples outside
signatures, and `nil` reject.
Recursive aggregate types cannot be formed without a rejected pointer and
also fail the shared declaration-dependency rules.

Go functions may have zero through 16 results. Parameters and results use
accepted types. A called function must have exactly one result; an uncalled
selected-closure function may have any accepted result count. Variadic
parameters reject. Named results are represented by source locals initialized
to their Go zero values and every return is made explicit before VIR emission.

## 7. Accepted statements and control flow

The accepted statement set is closed:

- a local `var` declaration with one name and zero or one initializer;
- `x := expression` introducing exactly one local;
- `x = expression` assigning exactly one previously declared local;
- `_ = expression` for an otherwise accepted pure expression;
- a block;
- `if` with an optional single accepted initialization statement, a Boolean
  condition, and optional block `else`;
- an explicit `return` with the exact result count;
- a static-call expression statement;
- the contracted `for` form below; and
- an empty statement.

Assignments to parameters, fields, array elements, constants, package values,
or multiple targets reject. Parallel assignment, tuple assignment, compound
assignment, `++`, `--`, and naked return reject. A declaration without an
initializer materializes the recursive Go zero value with explicit VIR
instructions before any read; source-language zero initialization is never an
undefined VIR local. All locals are renamed in source declaration order, not
alphabetically. Shadowed source names remain distinct compiler-resolved
bindings.

`else if`, `switch`, type switch, `select`, `range`, labeled statements,
`goto`, `break`, `continue`, `fallthrough`, `defer`, `go`, send, receive, and
explicit `panic`/`recover` reject. An equivalent nested `if` must be written as
an ordinary block and independently satisfy the rules.

A loop is exactly `for init; condition; post { body }`, where `condition` is
present, Boolean, and lowers without short-circuit control flow. `init` is
empty, one `x := expression` introducing exactly one local, or one accepted
assignment; `post` is empty or one accepted assignment. The body may contain
accepted statements but no nested loop, return, or edge leaving the loop other
than the header's false edge. Its emitted CFG must satisfy the single-header,
disjoint natural-loop rules in `VIR_V0.md`. Infinite loops, condition-only
loops, range loops, nested/overlapping loops, irreducible flow, and any loop
not matched by exactly one sidecar loop contract reject.

## 8. Accepted expressions and aggregate construction

Accepted expressions are typed identifiers, accepted constants/literals,
parentheses, the exact operations in section 9, direct static calls, direct
struct field reads, fixed-array reads, complete aggregate literals, and exact
BV conversions.

An integer literal or constant must resolve from Go type information to an
accepted fixed-width type. Its normalized VIR value uses canonical decimal and
must fit that type. A syntactic unary `+` or `-` applied to a literal/constant
that `go/types` resolves as one constant is normalized as one typed literal;
in particular, `-128` in an `int8` context is not an overflowing `UnaryOp`.
Unary sign on a nonconstant value follows section 9. The two special contexts
are:

- an otherwise untyped array index literal/constant is normalized as signed
  64-bit if it fits; and
- an otherwise untyped shift-count literal/constant is normalized as unsigned
  64-bit and must be nonnegative and fit.

An untyped integer expression with no such context, including `1 == 2`, would
default to rejected `int` and rejects with `GO_LOWER_UNTYPED_INTEGER`. Boolean
literals are accepted. String, floating, imaginary, character, and `nil`
literals reject.

A struct literal names an accepted struct, uses either declaration-order
unkeyed elements or field-keyed elements, and explicitly supplies every field
exactly once. Mixed forms, promoted/embedded fields, and implicit omitted-field
zeroes reject. A fixed-array literal supplies every element in order; keyed or
ellipsis-length array literals reject. These restrictions preserve the current
lowering boundary rather than treating every Go composite-literal shorthand as
implicitly supported.

Only direct field reads and fixed-array reads are accepted. Slicing, address
taking, dereference, type assertions, function literals, function/method
values, selector-based package state, builtins, allocation, and mutation of an
aggregate component reject. The type-conversion syntax is the sole accepted
call-like expression that is not `CallStatic`.

A source `&&`/`||` tree is accepted only when the complete tree is the
condition of an `if`, the complete expression of a `return`, or the complete
RHS of one accepted local declaration/assignment. Its operands may recursively
use `&&`/`||` and otherwise accepted Boolean expressions. Embedding a
short-circuit tree inside a call argument, aggregate element, equality, unary
operation, conversion, index, or other value context rejects with
`GO_LOWER_PATTERN`. This closed context set permits direct branch/return or
branch/copy lowering without a compiler-temporary phi.

## 9. Operations, total values, and required checks

`VIR_V0.md` section 8 is the only value interpretation. In particular, Go
addition, subtraction, multiplication, and negation are fixed-width wrapping
operations, never unbounded arithmetic and never checked-overflow operations.
The exact source-to-VIR table is:

| Go source | VIR | Operand rule | Exact `safety_checks` |
|---|---|---|---|
| `!x` | `UnaryOp not` | bool | `[]` |
| `+x` | no instruction | BV | none |
| `-x` | `UnaryOp bv_neg` | signed or unsigned BV | `[]` |
| `^x` | `UnaryOp bv_not` | BV | `[]` |
| `+`, `-`, `*` | `BinOp bv_add/bv_sub/bv_mul` | equal BV types | `[]` |
| signed `/`, `%` | `BinOp bv_sdiv/bv_srem` | equal signed BV | `[divisor_nonzero]` |
| unsigned `/`, `%` | `BinOp bv_udiv/bv_urem` | equal unsigned BV | `[divisor_nonzero]` |
| `&`, `|`, `^` | `BinOp bv_and/bv_or/bv_xor` | equal BV types | `[]` |
| `<<` | `BinOp bv_shl` | BV LHS, any accepted BV RHS | signed RHS: `[shift_count_nonnegative]`; unsigned RHS: `[]` |
| signed `>>` | `BinOp bv_ashr` | signed BV LHS, any accepted BV RHS | signed RHS: `[shift_count_nonnegative]`; unsigned RHS: `[]` |
| unsigned `>>` | `BinOp bv_lshr` | unsigned BV LHS, any accepted BV RHS | signed RHS: `[shift_count_nonnegative]`; unsigned RHS: `[]` |
| `==`, `!=` | `BinOp eq/not_eq` | equal bool or BV types | `[]` |
| `<`, `<=`, `>`, `>=` | signed/unsigned comparison op | matching BV signedness/type | `[]` |
| `&&`, `||` | `Branch` graph | bool, short-circuit | no eager operation/check |
| `T(x)` | `Convert` | accepted BV to accepted BV | `[]` |
| `a[i]` | `Index` | fixed array and signed/unsigned BV index | `[index_in_bounds]` |

Go's `&^` is not represented and rejects. Source aggregate equality rejects
even though VIR can validate aggregate equality; accepting it would widen the
historical source expression boundary without a corpus or source-contract
decision. Every other omitted Go operator rejects.

The exact semantic consequences are:

- overflow of `bv_add`, `bv_sub`, `bv_mul`, or `bv_neg` wraps modulo `2^w` and
  adds no proof check;
- every division/remainder proves only a nonzero divisor; Go's signed
  `MIN / -1` returns `MIN` and `MIN % -1` returns zero, with no representability
  check;
- a signed shift count proves nonnegativity, while an unsigned count adds no
  check;
- a nonnegative count greater than or equal to the LHS width is valid: left
  and logical-right shifts return zero and arithmetic-right shift returns all
  sign bits as defined by VIR; no `shift_count_less_than_width` check exists;
- the shift uses the RHS's complete width and never truncates the count to the
  LHS width;
- one `index_in_bounds` record denotes the complete signed predicate
  `0 <= index && index < length` or unsigned predicate `index < length`; and
- conversion widens a signed source by sign extension, widens an unsigned
  source by zero extension, truncates to low bits when narrowing, and then
  applies the target signed interpretation. It never adds a runtime check.

`&&` and `||` always lower to branches so operations/calls in the RHS inherit
the left guard. Eager Boolean `BinOp and/or` is invalid program VIR.

## 10. Go source contracts

### 10.1 Continued `mpk.go.contract.v0` input

The source-side schema remains exactly `mpk.go.contract.v0`; this migration
does not mutate it in place. The root object has only `schema`, `function`, and
optional `requires`, `ensures`, `modifies`, and `loops`. Duplicate names,
unknown fields, nulls, a second JSON value, malformed UTF-8/JSON, or a schema
mismatch reject. `requires`, `modifies`, and `loops` default to empty when
absent. If a sidecar exists, `ensures` must be present and nonempty and
`modifies` must be absent or empty.

Malformed UTF-8/JSON, a duplicate object member name at any depth, or a second
JSON value uses `GO_CONTRACT_JSON`. A missing/wrong `schema`, an unknown field,
any explicit null, or a wrong JSON scalar/container type for a root or loop
field uses `GO_CONTRACT_SCHEMA`. After that structural gate, a missing/empty
function uses `GO_CONTRACT_FUNCTION`, a missing/empty `ensures` uses
`GO_CONTRACT_ENSURES`, and malformed semantic loop members use
`GO_CONTRACT_LOOP`; expression shapes use the exact codes below.

`function` is trimmed and resolves either the exact canonical VIR function ID
or the historical top-level spelling `<package-name>.<function-name>`. The
historical spelling is accepted only when it resolves exactly once across the
included closure; it is normalized to the canonical import-path ID. Methods
must use the canonical ID. Ambiguous, unknown, duplicate, and unused sidecars
reject. Before lookup, the trimmed string must match either the closed Go public
declaration-ID grammar or exactly two `AsciiIdent` components separated by one
period; a lexical failure uses `GO_CONTRACT_FUNCTION` without echoing the raw
string.

The exact accepted expression forms are the Go branches of `VIR_V0.md`
section 10: variables, results, Boolean/typed-integer atoms, `not`, variadic
`and`/`or`, equality, signed/unsigned comparisons, BV arithmetic/bitwise/
shift/division/remainder operators, unary `bv_neg`/`bv_not`, and BV-to-BV
`convert`. The source-side encoding retains its historical aliases exactly:
`not`, `bv_neg`, and `bv_not` contain either `value` or a one-element `args`;
a binary operator contains either both `lhs` and `rhs` or a two-element
`args`; `and` and `or` contain an `args` array of two through 64 elements; and
`convert` contains either `value` or a one-element `args` plus the exact BV
target `type`. The alternatives are exclusive, and an explicitly present
unused operator field rejects even when its array is empty. Atoms contain
exactly one of `var`, `result`, `bool`, or `int` and none of `op`, `args`,
`lhs`, `rhs`, `value`, or `type`. Normalization rewrites the aliases to the
single VIR expression shape.

A non-object expression, unknown expression field, wrong JSON field type,
atom-count error, or atom/operator field mixture uses `GO_CONTRACT_SCHEMA`. A
lexically valid unknown operator, wrong recognized-operator alternative, or
wrong arity uses `GO_CONTRACT_OPERATOR`. A malformed typed integer/target,
ill-typed operand, signedness/width mismatch, or non-Boolean clause uses
`GO_CONTRACT_TYPE`. Source integer strings retain the historical base-0 parser
and exact width/signed tags, must fit, and normalize to canonical decimal.
Normalization preserves clause and expression order and never flattens,
reassociates, commutes, sorts, deduplicates, or constant-folds.

An `op` string is 1 through 64 ASCII bytes and matches
`[a-z][a-z0-9_]*` before operator lookup. A lexical failure uses
`GO_CONTRACT_OPERATOR` with fixed non-echoing prose; a well-formed unknown name
may appear in the normalized message, as frozen by the focused vector.

Contract BV expressions use VIR's total logical equations and never create
program `safety_checks`: contract division/remainder by zero and a negative
contract shift are logical values, not source execution. Runtime-safety
members arise only from program instructions in section 9.

Before canonical renaming, a requires expression may reference parameters; an
ensures expression may reference parameters and result indexes; a loop
invariant/decreases expression may additionally reference source locals live
at its header. Other visibility rejects. Source names resolve through type
information; source-side `var` strings are trimmed before exact resolution and
normalize to `argN`, `resultN`, `localN`, or header `pN` as required by VIR.
An expression cannot reference a temporary.

### 10.2 Loop sidecars and normalized contracts

A source loop entry has the continued fields `block_id`, optional `location`,
`invariants`, and optional `decreases`. `block_id` must equal the loop's final
canonical `bbN` header after trimming; `invariants` is nonempty. `location`,
when present, is trimmed traceability text, does not enter normalized VIR, and
cannot replace source-map coverage. Loop entries follow canonical header order
and match the complete loop set exactly.

All loops in one function are either partial, with every `decreases` empty, or
total, with every `decreases` nonempty. Mixing modes rejects. The normalized
contract uses `termination:"partial"` for the first case and `"total"` for the
second. Acyclic functions are total and have no loop entries. Invariants,
decreases, and exit semantics are exactly those in `VIR_V0.md` and generate
the source-language-neutral members in `VC_V1.md` section 4.

### 10.3 Contract identity and calls

Every included function receives exactly one normalized `VirContract`. Its
unit/function IDs, profile, semantic parameters, panic policy, termination,
clauses, loops, and `contract_hash` obey `VIR_V0.md`. Raw sidecar whitespace,
key order, aliases, and source variable spellings affect the manifest input
digest but not the normalized contract or VIR hash. Each static call repeats
the resolved callee's recomputed contract hash.

### 10.4 Deterministic default contract

Historical Go frontend acceptance did not require a sidecar for an acyclic
function, while VIR requires nonempty `ensures`. To preserve that accepted
frontend boundary without inventing a property, an acyclic function with no
sidecar receives exactly:

```text
requires    = []
ensures     = [{"bool":true}]
modifies    = []
panic       = forbidden
termination = total
loops       = []
```

All repeated IDs/profile/parameters and the contract hash are filled normally.
This default is not applied when any sidecar targets the function, and it
cannot make a loop acceptable because loops require explicit invariants. A
present sidecar with missing/empty `ensures` still rejects. The synthesized
`true` contract has no source-map entry because v0 maps only functions,
instructions, and terminators.

## 11. Canonical lowering, identifiers, and source maps

Lowering consumes only compiler-resolved typed syntax/SSA derived from the
captured buffers. It recognizes the source forms in this document and then
constructs VIR directly; it does not serialize or re-import GIR. Any SSA
instruction, implicit effect, or control-flow shape not explained by one
accepted source form rejects.

Canonical identities and order are:

- unit, type, constant, function, field, and method IDs follow sections 4 and
  6 plus `VIR_V0.md`;
- receiver then source parameters become dense `argN`;
- results become dense `resultN`;
- user locals, including explicit named-result storage, become dense `localN`
  in lexical source declaration order;
- phi values use dense block parameters `pN`, never a `Phi` instruction;
- reachable blocks become `bbN` by breadth-first traversal, enqueuing `else`
  before `then`; and
- value-producing instructions become `tN` in block/instruction order.

A non-entry block receives a parameter exactly when an accepted source local
(including named-result storage) is live there with distinct incoming values.
Within a block these parameters follow `VirFunction.locals` order; dense `pN`
assignment then follows canonical `bbN` order. Each predecessor supplies the
current exact-typed value in that order. A loop-carried local therefore becomes
a header parameter. A source read in that block uses the parameter until an
accepted `Copy` creates a newer value. An SSA phi that cannot be attributed to
one source local rejects with `GO_LOWER_PATTERN`; the short-circuit context
restriction above prevents compiler-expression phis. A loop contract local
normalizes to its header `pN` when carried and otherwise to its `localN`.

Source names, Go object pointers, token positions, SSA values, loader IDs,
absolute paths, and map iteration order never enter VIR. `features_used` is
derived exactly by `VIR_V0.md` rather than copied from historical descriptive
feature lists.

The source map contains exactly one entry for every function, instruction,
and reachable terminator. Function declarations and all nodes with a faithful
accepted syntax origin use the smallest complete UTF-8 source range that
caused that node: expression for value operations/calls, assignment or
declaration for copies/zero initialization, and `if`, short-circuit operator,
`for`, or explicit return for source control flow. Thus instructions that
materialize an omitted local initializer use the source `var` declaration;
they are not synthetic merely because Go supplied the zero value. Shared
source ranges are allowed.

The only Go synthetic reasons and applicable references are:

| Reason | Permitted reference |
|---|---|
| `go.control_flow_join` | `Jump` terminator inserted only to join an accepted `if` |
| `go.loop_backedge` | `Jump` terminator inserted only for an accepted loop post/backedge |
| `go.implicit_return` | `Return` terminator for source fallthrough of a zero-result function |

No function can be synthetic. All other instructions and terminators require a
source origin. A compiler node without one of these exact explanations rejects
with `GO_SOURCE_MAP_ORIGIN`. Contract location text, a nearest token, or a
compiler-generated file is not a substitute.

## 12. Rejections, statuses, and diagnostics

### 12.1 Closed rejected-feature matrix

The following table fixes the principal code for historical and adjacent
unsupported source classes. More specific parsing/type errors remain
`source-error`; a syntactically/type-correct use of one of these classes is
`rejected`.

| Class | Stable code |
|---|---|
| unsafe package or unsafe pointer | `GO_SUBSET_UNSAFE` |
| cgo or import `C` | `GO_SUBSET_CGO` |
| native/assembly auxiliary source | `GO_CGO_OR_AUX_SOURCE` |
| pointer, address-taking, dereference, pointer receiver | `GO_SUBSET_POINTER` |
| `new`, `make`, or other heap allocation | `GO_SUBSET_HEAP` |
| reflection | `GO_SUBSET_REFLECTION` |
| interface, assertion, method value, dynamic dispatch | `GO_SUBSET_INTERFACE` |
| goroutine | `GO_SUBSET_GOROUTINE` |
| channel/select/send/receive | `GO_SUBSET_CHANNEL` |
| defer | `GO_SUBSET_DEFER` |
| panic or recover | `GO_SUBSET_PANIC` |
| map | `GO_SUBSET_MAPS` |
| slice, slicing, append, copy | `GO_SUBSET_SLICES` |
| string | `GO_SUBSET_STRING` |
| floating-point | `GO_SUBSET_FLOAT` |
| complex | `GO_SUBSET_COMPLEX` |
| generic declaration/instantiation/type parameter | `GO_SUBSET_GENERICS` |
| package mutable variable or init | `GO_SUBSET_GLOBAL_STATE` |
| range or other nondeterministic iteration | `GO_SUBSET_ITERATION` |
| build tag/constraint/target-selected source | `GO_BUILD_CONSTRAINT` |
| machine-width `int`, `uint`, or `uintptr` | `GO_SUBSET_MACHINE_INT` |
| closure or function value | `GO_SUBSET_FUNCTION_VALUE` |
| runtime I/O, including `print`/`println` | `GO_SUBSET_IO` |
| external/standard-library import or package side effect | `GO_SUBSET_IMPORT` |
| unsupported declaration/statement/expression/operator | `GO_SUBSET_SYNTAX` |
| unsupported assignment target/form | `GO_SUBSET_ASSIGNMENT` |
| unsupported loop/CFG shape or missing loop contract | `GO_SUBSET_LOOP` |
| unresolved/cyclic/multi-result static call | `GO_SUBSET_CALL` |
| malformed/unsupported aggregate | `GO_SUBSET_AGGREGATE` |

The first failure phase follows `FRONTEND_PROTOCOL_V0.md`. Within a Go phase,
codes are ordered by normalized path, start offset, code, message, function
ID, and end offset after all same-phase findings have been normalized. Capture
file-kind/path/change/limit failures come first; source parser, directive, build,
and forbidden-import findings precede metadata module/workspace findings;
metadata completes before loader type checking; subset precedes lowering; and
lowering precedes source-map/manifest/hash emission. Preflight may collect the
facts needed to construct the snapshot, but it does not report a later phase
ahead of an earlier-phase finding.

### 12.2 Additional exact code families

The profile owns these additional codes:

| Code | Phase/status meaning |
|---|---|
| `GO_CAPTURE_FILE_KIND` | capture/rejected link or special file |
| `GO_CAPTURE_PATH` | capture/rejected nonportable, colliding, or rewritten path |
| `GO_CAPTURE_CHANGED` | capture/rejected namespace changed during capture |
| `GO_LIMIT_INPUTS` | capture/rejected candidate, manifest-input, directory, or directory-entry limit |
| `GO_LIMIT_INPUT_BYTES` | capture/rejected per-candidate or total captured-byte limit |
| `GO_LIMIT_SYNTAX` | source/rejected typed-syntax-node limit |
| `GO_MODULE_MISSING`, `GO_MODULE_INVALID`, `GO_PACKAGE_MISSING`, `GO_PACKAGE_AMBIGUOUS` | capture/`source-error` structural preflight failure |
| `GO_SELECTION_FUNCTION_MISSING`, `GO_SELECTION_FUNCTION_AMBIGUOUS` | subset/rejected selected function does not resolve exactly once |
| `GO_MODULE_POLICY`, `GO_MODULE_DEPENDENCY`, `GO_WORKSPACE_FORBIDDEN` | metadata/rejected closed module policy |
| `GO_CGO_OR_AUX_SOURCE`, `GO_SOURCE_DIRECTIVE` | source/rejected forbidden source-selection input |
| `GO_SOURCE_PARSE`, `GO_TYPECHECK` | source/typecheck `source-error` normalized Go error |
| `GO_LOWER_UNTYPED_INTEGER`, `GO_LOWER_PATTERN`, `GO_SOURCE_MAP_ORIGIN` | lowering/rejected unsupported exact lowering |
| `GO_CONTRACT_JSON`, `GO_CONTRACT_SCHEMA`, `GO_CONTRACT_FUNCTION`, `GO_CONTRACT_DUPLICATE`, `GO_CONTRACT_ENSURES`, `GO_CONTRACT_MODIFIES`, `GO_CONTRACT_OPERATOR`, `GO_CONTRACT_TYPE`, `GO_CONTRACT_LOOP` | subset/rejected contract failures |
| `GO_FRONTEND_SANDBOX`, `GO_FRONTEND_TOOLCHAIN`, `GO_FRONTEND_INTERNAL` | relevant phase/artifact-free `frontend-error` |
| `GO_LIMIT_DIAGNOSTICS_TRUNCATED`, `GO_FRONTEND_DIAGNOSTIC_BUDGET` | exact shared truncation behavior |

For `rejected`, every non-marker Go-owned issue is in `rejected_features` and
`diagnostics` is empty before shared truncation; the shared truncation marker,
when needed, is appended only to `diagnostics`. For `source-error` and
`frontend-error`, `rejected_features` is empty and every issue is in
`diagnostics`, as required by the shared protocol.

A subset/lowering issue uniquely owned by a resolved function uses that
canonical function ID. A package-wide declaration or contract-set issue with
no unique function owner uses the already resolved `selection.function`.
The two selection-resolution codes are the sole case that use the validated
requested function ID before a declaration match exists, as fixed in section
2. The truncation marker remains function-free.

Messages are concise normalized English and contain no GIR names, absolute
paths, source snippets, raw compiler prose, environment, argv, or sandbox
locators. The focused migration messages and codes are frozen by the
conformance vector. Each `GO_SOURCE_PARSE` message is exactly
`invalid Go source syntax`; each `GO_TYPECHECK` message is exactly
`Go type checking failed`. Their normalized spans, rather than compiler prose,
distinguish multiple findings. Rejected-feature and diagnostic arrays use the
shared sort/truncation rules. A successful Go v0 `ir-lowered` result has empty
`rejected_features` and empty `diagnostics`; this profile declares no warning
family. Child stderr is private transport and is never promoted to a warning.
Adding a successful-output diagnostic requires a new profile ID.

## 13. Deterministic limits

Shared serialized VIR, source-map, source-manifest, issue, and transport limits
apply unchanged. The Go capture/lowering boundary additionally enforces:

| Limit | Inclusive maximum |
|---|---:|
| loader-relevant candidate entries | 32,768 |
| manifest inputs | 32,768 |
| bytes per captured candidate | 16,777,216 |
| total captured candidate bytes | 268,435,456 |
| contract candidates / explicit contract paths | 128 |
| bytes per contract candidate | 1,048,576 |
| total contract candidate bytes | 8,388,608 |
| directories visited before nested-module pruning | 32,768 |
| directory entries examined | 131,072 |
| typed syntax nodes across included packages | 1,000,000 |

One loader-relevant candidate is each file selected for opening by section 3.1:
root module/workspace metadata, a crossed nested-module marker, a non-test
`.go` candidate, an auxiliary-suffix candidate, or a contract candidate. Test
files and entries in an unrelated unvisited directory do not count. The
per-candidate and total-byte counters cover exactly those same opened buffers,
including a buffer that later causes rejection. The manifest-input counter
covers the prospective section 5 input set; an auxiliary, workspace, or
nested-module marker is not prospective merely because capture opened it.
The three contract counters are stricter independent subsets of those generic
candidate/path and byte counters; both the discovered and explicit sets use the
same 128-member ceiling.

A visited directory is each unique directory opened and enumerated while
resolving the root, selected-package/import path components, or a possible
nested-module boundary; it counts before a boundary is pruned. Directory-entry
counting uses each visited directory's first complete enumeration. The final
unchanged-namespace re-enumeration checks the same bounded set and does not
increment either semantic counter. A typed syntax node is one non-nil
`go/ast.Node` reached by `ast.Inspect` over each included source file in
`packages.Package.Syntax`; each syntax tree is traversed once.

Counts use checked unsigned arithmetic. Capture counters are checked before
opening, reading, or retaining the next profile-owned entry, and a validated
file size is checked before its buffer allocation. Syntax traversal stops when
the first node over the ceiling is observed, before lowering or public output.
Candidate, manifest-input, directory, and directory-entry overflow use
`GO_LIMIT_INPUTS`; per-candidate and total byte overflow use
`GO_LIMIT_INPUT_BYTES`; typed-syntax overflow uses `GO_LIMIT_SYNTAX`. Exact
ceilings do not themselves change the underlying outcome; ceiling plus one
rejects with the stated code. Normalized issue count
and message budgets use only the shared truncation algorithm and preserve the
underlying status. After lowering, the shared
`mpk.vir.limits.v0` ceilings are applied independently and may bind first. Host
memory, CPU count, command-line flags, or environment cannot tighten or relax a
semantic limit. External process termination remains the caller-owned
`FRONTEND_PROCESS_KILLED`, not a profile rejection.

## 14. Historical preservation and intentional corrections

The historical accepted source rules map as follows:

| Historical rule | VIR disposition |
|---|---|
| top-level functions and value methods | `VirFunction`; receiver is `arg0` |
| top-level fixed Boolean/integer constants | `VirConstDecl` plus canonical constant references |
| bool/fixed signed and unsigned integers | `bool`/exact `bv` |
| fixed arrays and structs | structural arrays, nominal struct declarations, aggregate instructions |
| local declaration/assignment | explicit zero/value instructions and `Copy`/SSA block parameters |
| if/else and return | canonical `Branch`/`Jump`/`Return` CFG |
| contracted for loop | validated cyclic CFG plus `LoopContract` |
| Boolean/fixed arithmetic/comparison | section 9 operations and short-circuit CFG |
| struct/array construction and read | `MakeStruct`, `Field`, `MakeArray`, `Index` |
| static verified pure call | same-module `CallStatic` plus callee contract/precondition/panic-free dependencies |
| division, signed shift, array bounds | exact `divisor_nonzero`, `shift_count_nonnegative`, `index_in_bounds` checks |
| function purity | closed read/write/call rules and rejection matrix |

Every historical negative class remains rejected under section 12. GIR schema,
field, hash, CLI status, and byte equality are intentionally not preserved.
The source semantics are preserved while identities become VIR/VC v1
identities.

The following reviewed corrections/activations are explicit rather than
unexplained semantic widening:

- no-sidecar acyclic functions receive only the tautological contract in
  section 10.4 so historically accepted frontend inputs satisfy VIR shape;
- loops and same-module static calls described by `GO_SUBSET_V0.md` become
  active only in the exact contracted/closed forms above; current unimplemented
  forms do not define a second behavior;
- source `&&` and `||` use Go-required short-circuit CFG rather than the legacy
  eager descriptive operation, so RHS calls and safety checks retain their path
  guard;
- source fixed-BV conversions become `Convert`, matching the already captured
  contract-conversion intent and VIR's exact Go conversion equation;
- omitted local initializers materialize Go zero values instead of carrying an
  undefined legacy local;
- lexically shadowed locals become distinct canonical `localN` bindings instead
  of colliding through their source spelling; this activates ordinary local
  declarations without erasing binding identity;
- named primitive/array types and parameter assignment now reject because VIR
  v0 has neither Go nominal declarations for those types nor mutable argument
  bindings; erasing those distinctions would be unsound;
- top-level constants use only the explicit typed, one-name form in section
  6.1; the legacy feature scan's broader shorthand acceptance had no baseline
  coverage and did not define a stable declaration identity;
- source contract visibility is tightened to the normalized VIR visibility
  rules; no accepted baseline sidecar relies on the legacy overbroad local
  lookup;
- the explicit launcher contract paths must equal the legacy filename-based
  discovery set, preserving automatic discovery while preventing omission,
  addition, or caller ordering from changing semantics;
- source contract decoding rejects duplicate member names and explicit nulls,
  and `convert` is limited to BV-to-BV as required by VIR; the legacy parser's
  structurally accepted ambiguous encodings, package-name method aliases, and
  Boolean conversion had no collision-free or defined Go/VIR meaning and no
  accepted baseline uses them; and
- one signed `index_in_bounds` check contains both historical lower/upper
  predicates. Obligation IDs/count grouping may change, but neither predicate
  is lost.

The baseline's old hashes, GIR/VC/skeleton bytes, theorem names, and synthetic
VC-alpha counts are audit anchors, not equality targets. Payment-policy clause
order, branch-path meaning, eight postcondition intents per policy, proof
pending state until checked certificates exist, and all focused negative
classifications remain migration requirements. The release-report certificate
and source-free checker facts are independent trusted-pipeline anchors and are
not reclassified by a source profile.

The exact machine-readable mapping, including every Go-alpha function and
every focused baseline item, is in the conformance vector.

## 15. Conformance vectors and ownership

`develop/specs/vectors/go-vir-profile-v0.json` has schema
`mpk.go.vir_profile.conformance.v0`. Its exact top-level fields are `schema`,
`spec_profile`, `dependencies`, `owner_tests`, `profile_cases`,
`capture_cases`, `source_cases`, `operation_cases`, `contract_cases`,
`loop_call_cases`, `diagnostic_cases`, `limit_cases`, and
`migration_baseline`.

Case IDs are unique across all case arrays and arrays retain file order. A case
has exactly `id`, a profile-specific `construction` or `source`, and `expect`;
the vector uses compact construction recipes because the shared VIR vectors
own complete serialized model objects and hashes. `expect.outcome` is
`accepted`, `configuration-error`, `rejected`, `source-error`, or
`frontend-error`. `configuration-error` is the pre-launch exit-2/no-envelope
branch from `FRONTEND_PROTOCOL_V0.md`; other non-accepted cases carry exact
`phase` and `code`. Operation cases additionally carry exact
VIR kind/op/type/check/value facts; their owner constructs and validates a
complete model rather than comparing prose.

A case-level `source` is one complete LF-terminated Go file. Its owner places
it in the minimal module `example.com/p` with `go 1.23`, uses package selection
`example.com/p`, and selects the `expect.function_id` when present or
`example.com/p.F` otherwise. It supplies no sidecar. A `construction` instead
owns every input that it names and does not inherit this source shorthand.

`migration_baseline` names the immutable audit file and digest, lists every
old frontend/obligation disposition, every corpus group/function or case,
every behavioral anchor, and both checker anchors. The owning test loads that
file, verifies its digest and captured revision, compares complete set equality
for all named inventories, and proves no baseline array member is uncovered.
Changing the baseline or vector therefore requires an explicit reviewed
disposition rather than silently adding an accepted behavior.

The required root `owner_tests` array is exactly, in order:

1. `go-tools/go2vir/profile_v0_test.go`;
2. `go-tools/go2vir/profile_v0_vectors_test.go`; and
3. `crates/mpk-vc/tests/go_profile_vectors.rs`.

The owner tests MUST reject unknown vector/case fields, verify unique IDs,
execute every case, assert exact same-phase diagnostic precedence, and prove
that no case or migration record is skipped. Host-environment and target
determinism cases must poison every key in section 3.4 plus `PWD` and an
otherwise unknown sentinel key, then compare canonical VIR, source map, and
manifest bytes from equal registered requests. Argument-profile tests must
compare the complete launcher argv, normalize reordered caller contract
options to the same sorted path set, exercise both pre-spawn argument ceilings,
and prove that the explicit and discovered contract sets are equal before any
contract is accepted.
