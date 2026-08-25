# Release Registry and Bundle Installation v0 Specification

Status: normative and frozen for implementation.

This specification defines the untrusted release registry, bundle descriptors,
installed release tree, deterministic release assembler, and the boundary by
which a frontend/toolchain pair may be selected. Registry metadata and bundle
bytes are reproducibility inputs. They are never proof evidence.

The three public descriptor schemas are exactly:

- `mpk.release.bundle_registry.v0`;
- `mpk.release.frontend_bundle.v0`;
- `mpk.release.toolchain_bundle.v0`.

The canonical inventory schema is `mpk.release.bundle_inventory.v0`. The
source-only Rust candidate envelope is
`mpk.release.bundle_candidate.v0`; it is never an installed schema.

## 1. Conformance language and validation order

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
no frontend process starts and no VIR, source map, manifest, VC, certificate,
evidence, partial registry, or partially published release tree is returned.

Every object is closed. The tables in this document list all and only its
fields. Every listed field is required unless a tagged-union branch says
otherwise. Unknown fields, missing fields, duplicate JSON object names, and
`null` reject. A field forbidden by the selected tagged-union branch rejects.
Strings and enum values are case-sensitive. Floating-point numbers and JSON
integers outside `[-9007199254740991, 9007199254740991]` reject.

Registry validation uses this first-error phase order:

1. `transport`: transport byte count, UTF-8, JSON nesting, per-string limit,
   duplicate-name detection, and integer syntax;
2. `shape`: exact object and tagged-union shapes and schema discriminators;
3. `scalar`: scalar grammars, individual closed enum values, count limits, and
   portable paths;
4. `order`: canonical array order, uniqueness, and reference closure;
5. `invariant`: language/profile pairing, inventory subset/partition rules,
   executable records, host/runtime-layout consistency, and tuple
   compatibility;
6. `content_hash`: inventory content hashes and `bundle_sha256`,
   `distribution_sha256`, and `content_sha256` fields;
7. `registry_hash`: canonical registry byte count and `registry_sha256`;
8. `canonical_transport`: exact JCS-plus-LF transport comparison.

Those eight strings are the complete registry `phase` vocabulary. A vector and
owner test use the token verbatim; a numeric phase or alias rejects.

An implementation may report additional findings from the same phase, but a
later phase cannot replace the primary diagnostic. Installed-tree validation
starts only after all eight registry phases succeed.

## 2. Canonical JSON, scalar forms, and limits

### 2.1 Canonical transport

RFC 8785 JSON Canonicalization Scheme (JCS) is used throughout. A tracked or
installed registry is exactly `JCS(Registry) || 0x0a`: one JSON object, no BOM,
followed by exactly one LF and no other byte. Candidate transport has the same
JCS-plus-LF rule. The LF is not part of a hash preimage.

Registry producers serialize only after complete semantic validation.
Consumers parse while enforcing phase-1 limits, reserialize the validated
value with JCS, append one LF, and compare every byte with the input. Pretty
printing, CRLF, missing/further newlines, alternate escape spelling, or
noncanonical object-member order therefore rejects even if it denotes the same
JSON value.

A `Sha256` is exactly 64 lowercase hexadecimal ASCII characters. All digest
comparisons decode to 32 bytes and compare those bytes; uppercase hexadecimal,
base64, and a `sha256:` prefix reject.

### 2.2 Identifiers

`RegistryId`, `BundleId`, `ProfileId`, and `ComponentName` are 1 through 128
ASCII bytes and match:

```text
[a-z0-9]+([._-][a-z0-9]+)*
```

`ExecutableName` has the same grammar and is at most 64 bytes. An ID has no
normalization or alias. The bootstrap registry ID is exactly
`mpk.release.registry.v0`.

`source_language` is exactly `go` or `rust`. Its only semantic-profile pairing
is respectively `mpk.go.fixed.v0` or `mpk.rust.checked.v0`.
`pointer_width` is the JSON integer `32` or `64`. Go target IDs match
`[a-z0-9_]+/[a-z0-9_]+`. Rust target IDs use the `target_id` grammar from
`VIR_V0.md`. A tuple's target and width must be a target registered by its
toolchain descriptor; no host-derived target alias is permitted.

`Version` is a nonempty 1-through-128-byte printable ASCII string. It cannot
begin or end with ASCII whitespace and cannot contain `\`, `/`, control bytes,
or DEL. Version equality is byte equality; it is not semantic-version
comparison. A Rust compiler commit is exactly 40 lowercase hexadecimal
characters.

### 2.3 Portable relative paths

Every inventory path is relative and uses `/`. Each component is 1 through 255
ASCII bytes, contains only `[A-Za-z0-9._-]`, is neither `.` nor `..`, does not
end in `.`, and is not a case-insensitive Windows device name (`CON`, `PRN`,
`AUX`, `NUL`, `COM1` through `COM9`, or `LPT1` through `LPT9`, with or without
an extension). A complete path is at most 1,024 bytes. Leading/trailing `/`, an
empty component, `\`, a drive/UNC prefix, URI syntax, NUL, percent-decoded
interpretation, and two paths equal under ASCII case folding reject.

An absolute sandbox path starts with `/`, uses the same component grammar, is
already normalized, and is at most 1,024 bytes. It contains no trailing slash
except for `/`, although `/` itself is not valid as a mount target. There is no
filesystem-dependent normalization.

### 2.4 `mpk.release.limits.v0`

The release schema selects this profile intrinsically; it is not a registry
field or runtime option. All counters use checked unsigned arithmetic and are
enforced before allocating or reading the complete bounded value.

| Limit ID | Inclusive maximum |
|---|---:|
| `registry_canonical_bytes` | 67,108,864 |
| `registry_transport_bytes` | 67,108,865 |
| `json_nesting` | 256 levels |
| `string_bytes` | 1,048,576 per decoded string |
| `bundle_descriptors` | 1,024 frontend plus toolchain descriptors |
| `tuples` | 4,096 |
| `execution_host_profiles` | 256 |
| `native_runtime_layout_profiles` | 256 |
| `components` | 8,192 across toolchain descriptors |
| `serialized_inventory_entries` | 262,144 across all inventory occurrences |
| `unique_bundle_files` | 262,144 across root bundle inventories |
| `portable_path_bytes` | 1,024 |
| `bundle_file_bytes` | 4,294,967,296 per regular file |
| `bundle_declared_bytes` | 34,359,738,368 per bundle |

`registry_canonical_bytes` counts JCS of the complete registry including
`registry_sha256`. `registry_transport_bytes` additionally counts the required
LF. `serialized_inventory_entries` counts repeated component and root entries
as serialized. `unique_bundle_files` counts only complete root inventories.
Both declared and observed file-byte sums are checked against
`bundle_declared_bytes`; overflow rejects before any file is exposed or
executed.

## 3. Canonical inventory and content identity

### 3.1 `BundleInventory`

An inventory has exactly:

| Field | Type | Rule |
|---|---|---|
| `schema` | string | exactly `mpk.release.bundle_inventory.v0` |
| `scope` | `InventoryScope` | exact tagged union below |
| `files` | array of `InventoryFile` | nonempty; increasing `path` UTF-8 byte order |

`InventoryScope` is one of:

```json
{"kind":"frontend_bundle","bundle_id":"frontend.go.test.v0"}
```

```json
{"kind":"toolchain_bundle","bundle_id":"toolchain.go.test.v0"}
```

```json
{"kind":"component","bundle_id":"toolchain.go.test.v0","component_name":"go-stdlib"}
```

The shown fields are exact for each branch. The scope IDs must equal the
containing descriptor/component.

An `InventoryFile` has exactly:

| Field | Type | Rule |
|---|---|---|
| `path` | string | portable relative path |
| `executable` | boolean | exact installed executable-bit class |
| `size_bytes` | integer | `0..bundle_file_bytes` |
| `sha256` | string | raw regular-file SHA-256 |

Paths are unique, strictly sorted by UTF-8 bytes, and unique under ASCII case
folding. Directories are implicit and never listed. A component inventory is a
nonempty exact subset of its toolchain root inventory: every repeated file
entry is member-for-member equal. Toolchain components partition the complete
root inventory, so every root file belongs to exactly one component and no file
belongs to two components. An executable component contributes exactly its
`path` entry to that partition; a content component contributes every entry in
its complete inventory. A frontend root inventory is complete directly and
does not use component inventories.

The content identity of any complete inventory is:

```text
SHA256(
  UTF8("MPK-BUNDLE-CONTENT-0.1") || 0x00 || JCS(BundleInventory)
)
```

The domain is exactly the 22 ASCII bytes `MPK-BUNDLE-CONTENT-0.1`. The
inventory has no self-hash field. Frontend `bundle_sha256`, toolchain
`distribution_sha256`, and content-component `content_sha256` each use this
equation over their respective inventory. Hashing an archive, directory
metadata, path outside the inventory, or a pretty serialization is forbidden.

### 3.2 File-mode meaning

In an installed release, a declared non-executable file has mode exactly
`0444`, an executable has mode exactly `0555`, and a directory has mode exactly
`0555`. Set-ID, sticky, ACL-based executable substitution, and any write bit
reject. On a non-POSIX packaging host the installer must materialize these
classes in its portable package metadata and the Linux installed-tree validator
must observe the exact modes above.

## 4. Frontend bundle descriptor

`FrontendBundle` has exactly:

| Field | Type | Rule |
|---|---|---|
| `schema` | string | exactly `mpk.release.frontend_bundle.v0` |
| `bundle_id` | `BundleId` | unique across both descriptor arrays |
| `source_language` | string | `go` or `rust` |
| `name` | `ExecutableName` | release frontend name |
| `version` | `Version` | reviewed release version |
| `limit_profile_id` | `ProfileId` | fixed by the language/profile specification |
| `environment_profile_id` | `ProfileId` | fixed closed launcher environment profile |
| `argument_profile_id` | `ProfileId` | fixed closed launcher argv profile |
| `main` | `ExecutableRecord` | name equals descriptor `name` |
| `subordinate_binaries` | array of `ExecutableRecord` | increasing `name`, then `path`; unique names and paths |
| `inventory` | `BundleInventory` | complete frontend bundle inventory |
| `bundle_sha256` | `Sha256` | content identity of `inventory` |

The three profile IDs are closed by `source_language`:

| Language | `limit_profile_id` | `environment_profile_id` | `argument_profile_id` |
|---|---|---|---|
| `go` | `mpk.vir.limits.v0` | `mpk.go.frontend_environment.v0` | `mpk.go.frontend_arguments.v0` |
| `rust` | `mpk.vir.limits.v0` | `mpk.rust.frontend_environment.v0` | `mpk.rust.frontend_arguments.v0` |

The language profile specifications freeze the contents behind the environment
and argument IDs. A registry cannot mint another ID or alter those semantics.

An `ExecutableRecord` has exactly:

| Field | Type | Rule |
|---|---|---|
| `name` | `ExecutableName` | unique within the descriptor |
| `version` | `Version` | recorded manifest version |
| `path` | string | portable path in the containing root inventory |
| `binary_sha256` | `Sha256` | equals that inventory entry's raw digest |
| `runtime` | `ExecutableRuntime` | exact tagged union below |

The referenced inventory entry is executable and has the same digest. Every
executable entry in a frontend inventory is named by exactly one main or
subordinate record. Go v0 has no subordinate. Rust v0 has exactly one,
`rust2vir-driver`; these language-specific cardinalities are enforced when the
corresponding language profile is registered, not inferred from a filename.

`ExecutableRuntime` is either:

```json
{"kind":"static"}
```

or:

```json
{
  "kind":"dynamic",
  "interpreter_mount":"/lib64/ld-linux-x86-64.so.2",
  "libraries":[
    {
      "soname":"libc.so.6",
      "component_path":"lib/x86_64-linux-gnu/libc.so.6",
      "sha256":"0000000000000000000000000000000000000000000000000000000000000000"
    }
  ]
}
```

A `soname` is 1 through 255 printable ASCII bytes, contains no `/`, `\`, NUL,
control byte, or surrounding whitespace. Dynamic `libraries` is nonempty,
sorted by `(soname, component_path)` UTF-8 bytes, and has unique values in both
columns. It is the complete transitive native shared-library closure, including
dependencies of the listed libraries. `component_path` is relative to the
selected toolchain's native-runtime content component. The interpreter and
every library digest must resolve through that tuple's runtime-layout profile
and component. There is no loader search after validation.

## 5. Toolchain bundle descriptor and components

`ToolchainBundle` has exactly:

| Field | Type | Rule |
|---|---|---|
| `schema` | string | exactly `mpk.release.toolchain_bundle.v0` |
| `bundle_id` | `BundleId` | unique across both descriptor arrays |
| `source_language` | string | `go` or `rust` |
| `compiler` | `CompilerIdentity` | exact language-specific union |
| `execution_host_profile_id` | `ProfileId` | resolves exactly once in the registry |
| `native_runtime` | `NativeRuntimeSelection` | exact union below |
| `components` | array of `ToolchainComponent` | nonempty, increasing `name`; unique names |
| `target_libraries` | array of `TargetLibrary` | nonempty, increasing `target_id`; unique targets |
| `inventory` | `BundleInventory` | complete distribution inventory |
| `distribution_sha256` | `Sha256` | content identity of `inventory` |

`CompilerIdentity` is exactly one of:

```json
{"kind":"go","release":"go1.25.0"}
```

```json
{"kind":"rust","release":"1.89.0-nightly","rustc_commit":"0000000000000000000000000000000000000000"}
```

The union kind equals the descriptor `source_language`. A toolchain component
is one of the following exact shapes.

Executable component:

| Field | Type | Rule |
|---|---|---|
| `kind` | string | exactly `executable` |
| `name` | `ComponentName` | component ID |
| `release` | `Version` | component release |
| `path` | string | one executable root-inventory entry |
| `binary_sha256` | `Sha256` | equals that entry |
| `runtime` | `ExecutableRuntime` | static or complete dynamic closure |

Content component:

| Field | Type | Rule |
|---|---|---|
| `kind` | string | exactly `content` |
| `name` | `ComponentName` | component ID |
| `release` | `Version` | component release |
| `inventory` | `BundleInventory` | exact component partition |
| `content_sha256` | `Sha256` | content identity of `inventory` |

Every directly invoked toolchain executable is an executable component; its
raw digest therefore appears directly in the closed descriptor. An executable-
mode entry in a content component is permitted only when it is an interpreter
or shared library named by the native-runtime closure. It is never a directly
invoked component. Support data, notices, target standard libraries, and native
runtime bytes are content components.

A `TargetLibrary` has exactly `target_id`, `pointer_width`, `component_name`,
and `content_sha256`. It is sorted by `target_id`; target IDs are unique. The
named component is a content component and its digest is repeated exactly.
Every tuple target resolves one such entry of the matching width. Every target
library entry is selected by at least one tuple; a changed standard-library
file changes its component digest, the distribution digest, and the registry
hash.

`NativeRuntimeSelection` is exactly one of:

```json
{"kind":"none"}
```

```json
{"kind":"component","component_name":"native-runtime","component_root":"native-runtime","layout_profile_id":"mpk.runtime.example.v0"}
```

`none` is valid only when every toolchain executable and every frontend
executable paired with this toolchain is statically linked. The `component`
branch requires exactly one content component named `native-runtime` and the
literal `component_root` `native-runtime`. Every file in that component
inventory begins with `native-runtime/`; stripping that prefix yields the
component-relative paths used by executable/runtime-layout records. Its layout
profile must resolve and name the same execution-host profile as the toolchain.
If any paired executable has dynamic runtime, this branch is required. A
runtime component with `none`, or an unreferenced runtime component, rejects.

## 6. Execution-host and native-runtime-layout profiles

Profile IDs are closed by the validated registry: a descriptor can refer only
to a member of the corresponding sorted profile array. No environment,
platform probe, frontend response, or default/latest lookup may create a new
profile ID. Registry v0 admits only `linux`, `x86_64`, and `gnu`; each profile
still records all three values and an exact minimum kernel ABI so that no host
fact is implicit. A later platform requires a new release-registry schema.

### 6.1 `ExecutionHostProfile`

An execution-host profile has exactly:

| Field | Type | Rule |
|---|---|---|
| `id` | `ProfileId` | unique; profile array sort key |
| `os` | string | exactly `linux` |
| `architecture` | string | exactly `x86_64` |
| `abi` | string | exactly `gnu` |
| `minimum_kernel_abi` | string | exact `MAJOR.MINOR.PATCH`, no leading zero |
| `probe_profile_id` | string | one of the two closed probe IDs below |
| `required_primitives` | array of strings | exact array selected by `probe_profile_id` |

For `mpk.release.probe.linux_namespaces.v0`, the exact
`required_primitives` array, already in UTF-8 byte order, is:

```json
[
  "filesystem.atomic_no_replace",
  "filesystem.immutable_handle",
  "filesystem.no_follow_open",
  "isolation.mount_namespace",
  "isolation.network_namespace",
  "isolation.user_namespace",
  "mount.no_exec",
  "mount.read_only",
  "process.closed_environment",
  "process.no_new_privileges"
]
```

For `mpk.release.probe.linux_namespaces_cgroup2_tmpfs.v0`, the exact array is:

```json
[
  "filesystem.atomic_no_replace",
  "filesystem.immutable_handle",
  "filesystem.no_follow_open",
  "filesystem.tmpfs_allocated_blocks",
  "filesystem.tmpfs_inode_limit",
  "isolation.cgroup_v2",
  "isolation.mount_namespace",
  "isolation.network_namespace",
  "isolation.user_namespace",
  "memory.cgroup_accounting",
  "mount.no_exec",
  "mount.read_only",
  "mount.tmpfs_noswap",
  "process.cgroup_tasks",
  "process.closed_environment",
  "process.no_new_privileges",
  "process.rlimit_address_space",
  "process.rlimit_open_files",
  "process.task_tree_kill"
]
```

The cgroup2/tmpfs probe requires `minimum_kernel_abi` exactly `6.4.0`; that
kernel owns tmpfs `noswap` and every cgroup file used by the profile.

Each `minimum_kernel_abi` component is either `0` or `[1-9][0-9]*`, fits an
unsigned 32-bit integer, and the complete string contains exactly two dots.
The host tuple is the four-member value `(os, architecture, abi,
minimum_kernel_abi)`. Kernel comparison is numeric by the three components;
suffix text in the observed release is ignored only after parsing those three
leading numeric components. A missing component, overflow, or host below the
minimum is unavailable, not a version guess.

### 6.2 Bounded capability probe

`mpk.release.probe.linux_namespaces.v0` is one fixed pre-launch algorithm. It
runs after registry and bundle snapshots validate, but before source capture,
private bundle exposure, or frontend launch. The runner creates one child from
its own already-open executable, passes no user bytes, and gives it an empty
closed environment. The child may make at most 128 probe syscalls, create at
most 32 private filesystem entries, fork no child, open no network endpoint,
and emit at most 512 bytes of fixed-format result data. The parent enforces a
2-second monotonic operational deadline and kills/reaps the child on expiry.
Probe time never changes source or proof acceptance.

In one fresh private temporary directory, the child checks the numeric kernel
minimum, user/mount/network namespace creation, `no_new_privileges`, read-only
and no-exec bind behavior, no-follow rejection of a synthetic symlink, identity
retention by an already-open handle after a path replacement, and atomic
no-replace publication against an occupied destination. It verifies that the
new network namespace has no externally usable interface and never tests that
property by contacting an endpoint. Every operation is cleaned up inside the
child namespace.

Any missing primitive, denial, unexpected success of a prohibited write/exec,
malformed probe result, limit breach, timeout, or cleanup uncertainty maps to
artifact-free `frontend-error` code `FRONTEND_SANDBOX_UNAVAILABLE`. The runner
does not retry outside a namespace, weaken a mount, inherit networking, switch
to a path-based open, or use replace-on-collision publication.

`mpk.release.probe.linux_namespaces_cgroup2_tmpfs.v0` first validates Linux's
reserved initial-cgroup-namespace inode `0xeffffffb` and that the sole visible
cgroup2 mount is the writable global hierarchy root. The runner must be the
only process in an otherwise empty delegated domain. It creates an
unlimited manager child, moves itself there, enables exactly `memory` and
`pids` on the processless domain, and creates a finite sibling accounting leaf.
It writes and reads back the registered `pids.max`, `memory.max`, and
`memory.swap.max` values and creates the probe task in that leaf atomically with
`clone3(CLONE_INTO_CGROUP | CLONE_PIDFD)`. Every non-root ancestor must have
unlimited pids, memory, memory-high, and swap controls with unchanged relevant
local events.

Inside the accounting leaf, the child verifies its exact cgroup membership,
controls, rlimits, closed descriptors, and every v0 namespace primitive above.
Before entering its user namespace, it creates a first private mount namespace
and mounts a one-page `nosuid,nodev,noexec,noswap` tmpfs with a four-inode
ceiling. It verifies the exact block and inode totals, consumes the last three
non-root inodes, observes `ENOSPC` on the next creation, and cleans up the
mount. It then enters the user namespace and a second private mount namespace
for the remaining v0 checks. Mounting the fixed `noswap` backing before
`NEWUSER` is required because Linux may reject that superblock option from an
unprivileged user namespace; no source byte or ambient mount option participates.

The production bootstrap uses the same two-stage ordering. Before `NEWUSER`,
it creates a private mount namespace, bind-pins the fresh sandbox root, and
mounts the fixed 20-GiB, 262144-inode `nosuid,nodev,noswap` aggregate tmpfs at
a hidden child of the future `/mpk/tmp` mountpoint. After `NEWUSER` and the
second mount-namespace creation, it bind-mounts only a fresh directory from
that backing over `/mpk/tmp`, remounts the visible view
`nosuid,nodev,noexec`, and revalidates the exact capacity, inode ceiling, and
`noswap` state. The overmount makes the privileged backing path unreachable to
the frontend, and `no_new_privileges` is set before the frontend executable is
started.

The parent kills and reaps the task, verifies `cgroup.kill`, `memory.peak`,
`pids.events`, and `memory.events.local`, releases every pipe, namespace
descriptor, and backing object, requests cgroup-local memory reclaim, and
requires `memory.swap.current` plus the `anon`, `file`, `sock`, and `shmem`
byte gauges in `memory.stat` and any present `zswap` and `zswapped` gauges to
reach zero. It then removes the probe leaf and requires zero dying descendants.
Parsed residual
`memory.current` and `kernel` accounting are not by themselves live-resource
evidence after those task-owned gauges discharge because newer kernels retain
per-CPU stock and cgroup kernel-object charges until removal. It retains the
same unlimited manager and processless domain for exactly one fresh finite
production leaf. Final teardown permits only the removed manager's
attributable invisible dying state and permanently consumes the process-wide
Rust session. The probe is capability evidence, not a substitute for the
production leaf's independent controls and audit.

### 6.3 `NativeRuntimeLayoutProfile`

A native-runtime-layout profile has exactly:

| Field | Type | Rule |
|---|---|---|
| `id` | `ProfileId` | unique; array sort key |
| `execution_host_profile_id` | `ProfileId` | exact host-profile reference |
| `runtime_root` | string | exactly `/mpk/native-runtime` |
| `interpreter_mounts` | array of `InterpreterMount` | nonempty; sorted by `sandbox_path` |
| `library_mounts` | array of `LibraryMount` | nonempty; sorted by `sandbox_path` |
| `loader_search_paths` | array of absolute paths | nonempty, unique, specification order |
| `forbidden_host_roots` | array of strings | exact array below |

An `InterpreterMount` has exactly `component_path` and `sandbox_path`.
`component_path` is a portable regular-file path in the native-runtime
component. `sandbox_path` is an absolute path. Both columns are unique. A
`LibraryMount` has exactly `component_path` and `sandbox_path`; each names a
portable component directory and an absolute sandbox directory. Directory
prefixes cannot overlap another library mount or an interpreter file.

`loader_search_paths` consists only of exact `sandbox_path` values from
`library_mounts`, with no empty element and no inherited suffix. The exact
`forbidden_host_roots` array is:

```json
["/lib","/lib64","/usr/lib"]
```

`runtime_root` bounds component-side source material: after removing the
descriptor's literal `native-runtime/` component root, every
`component_path` is resolved beneath the sealed `/mpk/native-runtime` view and
cannot escape it. `sandbox_path` and `loader_search_paths` are destination
names inside the new sandbox namespace; they are not required to be beneath
`runtime_root`. In particular, `/lib64/ld-linux-x86-64.so.2` and
`/lib/x86_64-linux-gnu` are valid private mount destinations because the
mounted source handles come only from `/mpk/native-runtime`.

`forbidden_host_roots` restricts source lookup in the parent/host namespace. It
does not forbid those same spellings as destination names in the isolated
namespace. Before mounting private handles, the destination directories are
fresh and contain no inherited host mount; after mounting, the runner verifies
their device/inode identities equal the sealed component sources.

The native-runtime component contains exactly the union of every interpreter
file and every regular file recursively beneath the component-side library
directories. Every such entry is copied from its already-open snapshot into
the private runtime root. Every dynamic executable's `interpreter_mount` and
library path/digest resolves exactly in that union. No file from host `/lib`,
`/lib64`, `/usr/lib`, an ELF default search directory, `ld.so.cache`,
`LD_LIBRARY_PATH`, `LD_PRELOAD`, or an ABI-compatible fallback may be opened as
a source or retained from the host namespace. A missing SONAME or transitive dependency is
`FRONTEND_SANDBOX_UNAVAILABLE`, not authorization to search the host.

## 7. Release registry and tuple resolution

### 7.1 `BundleRegistry`

The registry root has exactly:

| Field | Type | Rule |
|---|---|---|
| `schema` | string | exactly `mpk.release.bundle_registry.v0` |
| `id` | `RegistryId` | exactly `mpk.release.registry.v0` for v0 |
| `execution_host_profiles` | array | increasing `id`; unique |
| `native_runtime_layout_profiles` | array | increasing `id`; unique |
| `frontend_bundles` | array of `FrontendBundle` | increasing `bundle_id`; unique |
| `toolchain_bundles` | array of `ToolchainBundle` | increasing `bundle_id`; unique |
| `tuples` | array of `ReleaseTuple` | exact key order below; unique |
| `registry_sha256` | `Sha256` | registry identity equation below |

The two bundle arrays have disjoint IDs. Every bundle descriptor is selected
by at least one tuple and every profile is selected by at least one toolchain,
except that all five arrays may be empty together in the bootstrap registry.
The bootstrap state cannot contain a proper nonempty subset. It validates for
building pre-frontend MPK, but resolves no request.

`registry_sha256` is:

```text
SHA256(
  UTF8("MPK-BUNDLE-REGISTRY-0.1") || 0x00 ||
  JCS(BundleRegistry with only registry_sha256 removed)
)
```

The domain is exactly the 23 ASCII bytes `MPK-BUNDLE-REGISTRY-0.1`. Only the
root `registry_sha256` member is removed. All descriptors, inventories,
profiles, tuples, and `id` remain. Any descriptor or inventory mutation
therefore changes the expected registry hash.

### 7.2 `ReleaseTuple`

A tuple has exactly:

| Field | Type | Rule |
|---|---|---|
| `source_language` | string | `go` or `rust` |
| `semantic_profile` | `ProfileId` | exact language pairing from section 2.2 |
| `target_id` | string | language target grammar |
| `pointer_width` | integer | `32` or `64` |
| `limit_profile_id` | `ProfileId` | equals the selected frontend descriptor |
| `frontend_bundle_id` | `BundleId` | exact descriptor reference |
| `toolchain_bundle_id` | `BundleId` | exact descriptor reference |

The canonical tuple key is:

```text
(
  source_language,
  semantic_profile,
  target_id,
  pointer_width,
  limit_profile_id,
  frontend_bundle_id,
  toolchain_bundle_id
)
```

Strings compare by UTF-8 bytes and `pointer_width` numerically. `tuples` is
strictly increasing by this key. Duplicate full keys reject. Multiple bundle
pairs for one language/profile/target are permitted only when their full keys
differ; caller-supplied bundle IDs make the selection unambiguous.

The caller selection key omits the two descriptor-derived fields and is:

```text
(
  source_language,
  semantic_profile,
  target_id,
  frontend_bundle_id,
  toolchain_bundle_id
)
```

Selection keys are also unique. Thus two tuples cannot differ only by
`pointer_width` or `limit_profile_id`; those values are outputs fixed by the
selected tuple, never caller overrides.

The tuple language equals both descriptors. Its limit profile equals the
frontend descriptor. Its target/width resolves exactly one toolchain target
library. Every dynamic executable in the selected frontend/toolchain pair
resolves the same required runtime layout and native-runtime component. Crossed
language, profile, target, width, limit, bundle, host, or runtime values reject
registry validation; they are not repaired during selection.

### 7.3 Caller selection

An evidence-producing caller supplies all of:

- registry ID and registry SHA-256 equality assertions;
- `source_language`, `semantic_profile`, and `target_id`;
- `frontend_bundle_id` and `toolchain_bundle_id`.

The assertions compare with build-embedded constants and cannot select a
different registry. The remaining fields resolve one exact selection key; its
`pointer_width` and `limit_profile_id` are then returned as immutable selected
configuration. There is no caller pointer-width/limit override, first entry,
default, latest version, compatible fallback, host target, filename, `PATH`,
project configuration, or environment selection.

An unknown bundle ID is a pre-launch configuration error
`FRONTEND_BUNDLE_UNKNOWN`. A registry ID/hash assertion unequal to the embedded
constants is the distinct pre-launch configuration error
`FRONTEND_REGISTRY_ASSERTION`. Known bundle IDs that do not form the requested
registered tuple are `FRONTEND_BUNDLE_INCOMPATIBLE`. These errors produce no
frontend protocol response. A separately installed file missing or unequal
after a tuple was selected is instead artifact-free `frontend-error`
`FRONTEND_BUNDLE_INVALID`.

## 8. Installed release layout and root derivation

### 8.1 Exact layout

The security-managed installed tree is exactly:

```text
RELEASE_ROOT/
  bin/
    mpk
  share/
    mpk/
      bundle-registry.json
  libexec/
    mpk/
      bundles/
        BUNDLE_ID/
          ... complete root inventory ...
```

The top-level directory contains only `bin`, `share`, and `libexec`; every
shown intermediate directory contains only the shown children. `bundles`
contains exactly one directory for every frontend/toolchain bundle ID and no
other child. Bundle directories have disjoint IDs. The registry itself and
`bin/mpk` are not bundle-inventory entries.

### 8.2 Deriving `RELEASE_ROOT`

On the v0 Linux host, bootstrap first opens the currently executing image via
the kernel `/proc/self/exe` identity as a regular executable handle. It rejects
a deleted image. It obtains the absolute kernel-reported path, opens every
parent component without following links, and proves the final `bin/mpk`
device/inode equals the already-open executable handle. The final two path
components must be exactly `bin/mpk`; the retained parent directory handle is
`RELEASE_ROOT`. `bin/mpk` has link count one. Each directory and `bin/mpk` has
the exact mode from section 3.2; directory link counts are not compared because
POSIX subdirectories legitimately change them.

`MPK_RELEASE_ROOT`, current working directory, `argv[0]`, `PATH`, project
files, user configuration, CLI flags, a registry adjacent to another binary,
and symlink spelling do not participate. If the executable identity cannot
yield that exact retained root, the release frontend boundary is unavailable.

The runner resolves every later path with descriptor-relative operations from
the retained root descriptor. It never concatenates an unchecked absolute
path, changes roots after selection, or reopens the executable by pathname.

### 8.3 Opening and validating the registry

The runner walks `share/mpk/bundle-registry.json` from the retained root using
no-follow opens on every component. The registry must be one regular,
non-executable, link-count-one file with mode `0444`. Before allocation it
rejects a declared size above `registry_transport_bytes`; it then reads at most
that bound plus one byte from the retained handle, detects short/growing reads,
and runs section 1 validation. The validated JCS bytes and parsed model remain
immutable for the whole invocation. The path is never reopened.

Missing registry maps to `frontend-error` code
`FRONTEND_REGISTRY_MISSING`. Byte/count/depth limits map to
`FRONTEND_REGISTRY_LIMIT`. Syntax, duplicate names, noncanonical transport,
unknown fields, invariant failure, or self-hash failure map to
`FRONTEND_REGISTRY_INVALID`. A registry ID/hash unequal to the embedded values
maps to `FRONTEND_REGISTRY_MISMATCH`. Each is artifact-free and occurs before
tuple resolution or source capture.

### 8.4 Build-embedded constants

The `mpk-cli` build script locates the repository-owned source file only at
`release/bundles/bundle-registry.json` relative to its fixed workspace source
root. No environment variable or feature chooses another path. It applies the
same bounded strict validation, recomputes the content hashes and registry
hash, and requires canonical JCS-plus-LF bytes. Compilation fails on any
disagreement.

For registry selection, only the validated `id` and decoded 32-byte
`registry_sha256` are generated into the build output and embedded. Registry
bytes, descriptor paths, source paths, bundle roots, and a hand-maintained
duplicate hash are not embedded. Runtime always validates the separately
installed registry; successful build validation cannot make a missing or
changed installation valid.

The independent Go reference checker is a separate build-owned executable
payload embedded in `bin/mpk`, not a registry selection value. The release
assembler builds it as a static Linux AMD64 executable with the same
digest-pinned Go image and closed, network-free environment used for the Go
release build, with cgo disabled, paths trimmed, and the build ID removed.
`check-go`, `check-all`, and installed fixture modes rebuild the payload and
require exact byte equality with the repository asset before acceptance.

The checker payload is not installed beside `mpk`, listed in a frontend or
toolchain inventory, or supplied by a registry descriptor. Runtime may execute
only the embedded bytes through the sealed anonymous boundary in
`PROGRAM_CERTIFICATE_ALPHA_V0.md`; no caller, environment variable, feature,
adjacent file, or registry entry chooses checker code or a checker path.

## 9. Installed bundle snapshot validation

For the exact selected frontend and toolchain bundle roots, the runner performs
the following steps before capability probes or source capture:

1. Open each bundle directory from the retained `bundles` descriptor without
   following links. Reject a symlink, mount/reparse point, non-directory, wrong
   mode, wrong ID spelling, or cross-bundle alias.
2. Enumerate the complete directory tree with checked counts, depth, and byte
   sums. Reject every path not listed by the root inventory, every missing
   path, an explicit directory entry, case-fold collision, and all source-only
   names from section 11. Every implicit intermediate directory is opened
   without following links, has mode `0555`, remains on the release-root
   filesystem, and is neither a reparse/mount point nor an inode alias of
   another managed directory.
3. Open every expected leaf relative to its already-open parent with no-follow
   and require a regular file, exact mode/executable class, link count one,
   declared size, and a `(device,inode)` identity not used by any other managed
   file. Symlinks, hard-link aliases (including a link outside the tree),
   junctions/reparse points, devices, FIFOs, sockets, and filesystem magic
   links reject.
4. Stream each opened file once into an invocation-owned immutable snapshot
   while computing raw SHA-256 and observed byte count. Recheck its handle
   identity/metadata after EOF and reject concurrent change, short read,
   growth, or digest mismatch. The original path is never read again.
5. Re-enumerate the source directories through retained handles and require
   the same path/identity set. Seal snapshot files read-only; on Linux,
   executable and runtime files use sealed anonymous files or an equivalently
   private immutable mount. Hash validation and eventual `execveat`/mount use
   those same snapshot identities.
6. Recompute all component/root inventory equations and require the descriptor
   digests. Build the private frontend, toolchain, and native-runtime views
   solely from the sealed snapshot. Only then run the bounded host probe.

An immutable open descriptor alone is insufficient if another writer can
change that inode. Copy-and-seal (or a stronger proven immutable filesystem
primitive) is mandatory. A failure in these steps is
`FRONTEND_BUNDLE_INVALID`; no path-based or ambient retry is allowed.

The ordinary runner performs complete metadata enumeration for every registered
bundle root and the full snapshot/hash procedure for the selected pair. The
installer and `check-release-bundles` installed-tree validator perform all six
steps for every registered descriptor, not only one selected tuple, and require
that a shared descriptor validates identically wherever a tuple references it.
No mode treats an unselected, extra, or malformed registered bundle directory
as inert packaging data.

The complete runtime mount uses exactly the descriptor/profile union from
section 6.3. A loader request for an unlisted interpreter, SONAME, library,
target library, or ABI is invalid even if a byte-identical host file exists.

## 10. Test-only injected resolution

Unit and integration tests may construct a `TestBundleResolver` only in code
compiled with Rust `cfg(test)` or an equivalent language test build tag. The
constructor accepts already-parsed registry values and invocation-owned sealed
file objects, not filesystem paths, environment variables, or a release-root
string. It runs the same model, hash, tuple, inventory, and runtime-closure
validation before returning a selection.

The production resolver type has no injected constructor, trait-object setter,
feature flag, public path argument, environment hook, serialized resolver, or
fallback. A compile-time production API test must prove that injection symbols
are absent. Evidence-producing routes are compiled only against the production
resolver and reject candidate schema values even in a test process.

## 11. Source-only release material

Repository source metadata and build material are not installation sources:

```text
release/bundles/bundle-registry.json
release/bundles/candidates/rust/candidate.json
release/build-inputs/rust/build-inputs.json
release/build-input-cache/rust/BUILD_INPUTS_SHA256/...
```

The registry is copied separately to its exact installed `share/mpk` location.
Registered bundle bytes are rebuilt into private assembler staging and are
never inferred from an arbitrary repository subtree. `release/bundles/README.md`
is documentation only.

An installer and installed-tree validator reject any path component named
`candidates`, `build-inputs`, or `build-input-cache`, a
`mpk.release.bundle_candidate.v0` value, or build-input descriptor/cache bytes
anywhere beneath an installed release root. They also reject a repository
`release` directory presented as a bundle root. The candidate cannot be copied,
renamed, symlinked, or rewrapped into an installation. Build-input descriptors
and caches never occur in frontend/toolchain inventories.

## 12. Source-only candidate envelope

From `RUST-03-T01` until first Rust registration, the directory
`release/bundles/candidates/rust` contains exactly one file, `candidate.json`,
in canonical JCS-plus-LF transport. `BundleCandidate` has exactly:

| Field | Type | Rule |
|---|---|---|
| `schema` | string | exactly `mpk.release.bundle_candidate.v0` |
| `source_language` | string | exactly `rust` |
| `execution_host_profiles` | array | Rust projection, canonical order |
| `native_runtime_layout_profiles` | array | Rust projection, canonical order |
| `frontend_bundles` | array | nonempty Rust descriptors only |
| `toolchain_bundles` | array | nonempty Rust descriptors only |
| `tuples` | array | nonempty Rust tuples only |

The descriptor/profile/tuple rules are identical to registry rules. The
candidate repeats every host/layout profile required by its Rust descriptors;
an ID also present in the active registry must be member-for-member equal.
Merging the active registry and candidate deduplicates only such equal profile
records, and every other bundle/tuple ID must be disjoint. The projection has
no registry ID/hash of its own. Candidate identity is exact transport-byte
equality plus the descriptor and content hashes it contains.

`--update-candidate rust` rebuilds the candidate from current sources and the
checked build-input closure. It never copies a prior candidate's descriptors.

## 13. Deterministic assembler CLI

The common internal dispatcher is
`scripts/build-release-bundles.sh`. Its complete accepted argv set is:

```text
--update-build-inputs rust
--provision-build-inputs rust
--check-build-inputs rust
--update-candidate rust
--check-candidate rust
--update go
--check go
--update all
--check all
```

The mode token must be first and the target token second; there are no other
arguments, aliases, combined short flags, defaults, or bare action. Bare,
extra, reordered, repeated, partial, or unsupported mode/target input exits
without network or repository writes as `BUNDLE_ASSEMBLER_USAGE`.

Success exits 0 with empty stdout and stderr. Failure writes one fixed code plus
LF to stderr and no stdout: usage is
exit 64, invalid state/input or mismatch is 65, unavailable publication or
host capability is 69, and I/O failure is 74. The stable non-usage codes are:

- `BUNDLE_BUILD_INPUTS_NOT_CONFIGURED`;
- `BUNDLE_BUILD_INPUTS_INVALID`;
- `BUNDLE_CANDIDATE_STATE`;
- `BUNDLE_REGISTERED_STATE`;
- `BUNDLE_REPRODUCIBILITY_MISMATCH`;
- `BUNDLE_PUBLICATION_UNAVAILABLE`;
- `BUNDLE_ASSEMBLER_IO`.

Until `RUST-03-T01` installs the handler and descriptor contract frozen by
`VIR-00-T09`, all three Rust build-input modes return exit 65 and exactly
`BUNDLE_BUILD_INPUTS_NOT_CONFIGURED\n`. They perform no fetch, temporary
materialization, tracked/cache write, candidate read/write, or registry write.

For every bundle update, the assembler derives executable runtime records from
the sealed release outputs with a bounded, non-executing Linux ELF parser. It
requires the x86_64 ELF machine/ABI selected by the host profile, reads
`PT_INTERP` and the complete recursive `DT_NEEDED` closure, and compares every
interpreter, SONAME, component-relative path, and digest with the proposed
descriptor. `static` requires no interpreter or dynamic dependency. A script,
foreign executable format, malformed ELF table, unresolved dependency, or
`ldd`/host-loader-based discovery rejects before staging. Existing descriptor
runtime fields are never trusted as the source from which the closure is
generated.

### 13.1 Write and network matrix

An invocation-owned fresh private temporary directory outside the repository is
not a published write. Check modes may build there, but never write or repair a
repository, build-input cache, installation, original source, or candidate.

| Mode | Network | Only permitted committed/persistent write |
|---|---|---|
| `--update-build-inputs rust` | only fixed origins/digests | ignored hash-keyed cache, then tracked descriptor commit |
| `--provision-build-inputs rust` | only fixed origins/digests | ignored hash-keyed cache only |
| `--check-build-inputs rust` | disabled | none |
| `--update-candidate rust` | disabled | complete Rust candidate subtree |
| `--check-candidate rust` | disabled | none |
| `--update go` | disabled | complete registered Go registry state |
| `--check go` | disabled | none |
| `--update all` | disabled | complete registered all-language state; first use also removes candidate |
| `--check all` | disabled | none |

Every network-disabled mode creates a network namespace before any build child
starts and treats inability as `BUNDLE_PUBLICATION_UNAVAILABLE`; it does not
rely on an application offline flag alone. The two network-capable modes allow
only specification-fixed HTTPS origins, exact response digests, bounded bytes,
and no redirect outside the fixed origin set. Proxy, credential, alternate
mirror, DNS search, and environment locator inputs are cleared.

### 13.2 Build-input transaction

`--update-build-inputs rust` is the only tracked build-input descriptor writer.
It fetches only the frozen inputs, builds every generated tool twice in separate
empty sandboxes, requires byte equality, stages the complete cache privately,
derives the descriptor from those staged bytes, validates it, computes its hash
and final key, and publishes the cache with atomic no-replace. An occupied key
is reused only after complete byte equality; an unequal occupant rejects and is
never overwritten or repaired. Only after successful cache publication does an
atomic same-directory descriptor replacement commit the tracked descriptor.
It never writes a candidate or registry. A validated unused cache left by a
failed descriptor commit is harmless because no descriptor selects it.

`--provision-build-inputs rust` starts from an unchanged valid tracked
descriptor, recreates only its ignored cache, validates privately, and
publishes only to that already determined no-replace key. It does not rewrite
the descriptor, candidate, or registry. `--check-build-inputs rust` reads and
validates the exact descriptor, recomputed key, and complete occupied cache
without fetch, temporary repair, or writes. A missing/invalid cache fails; no
other mode provisions it implicitly.

Every Rust candidate or registered build first completes the read-only check in
the same invocation. It then captures the validated inputs again into its own
sealed build snapshot; the earlier check is not permission to reopen mutable
cache paths during build.

### 13.3 Registered/candidate state machine

The assembler classifies the fully validated current tree before doing work:

| State | Registry | Candidate |
|---|---|---|
| `bootstrap` | all registry arrays empty | absent |
| `go_registered` | one or more Go tuples, no Rust tuple | absent |
| `rust_candidate` | one or more Go tuples, no Rust tuple | valid current Rust candidate |
| `all_registered` | one or more Go and Rust tuples | absent |

Any other combination, invalid registry/candidate, an incompatible profile-ID
collision between active registry and candidate, Rust descriptor without a
Rust tuple, or candidate after Rust registration is
`BUNDLE_REGISTERED_STATE` and no writer runs.

The reviewed assembler release-language configuration is compiled into the
script, not read from the registry or environment. It is `go` after first Go
registration. `RUST-03-T01` changes it atomically to the exact ordered set
`["go","rust"]`, even though the active registry still has no Rust tuple.
`all` always means this complete set; no `--language` subset exists.

Allowed transitions are:

- `--update go`: `bootstrap -> go_registered`, or refreshes
  `go_registered`/`rust_candidate` while preserving every candidate byte. In
  `rust_candidate`, the staged prospective Go registry must remain merge-
  compatible with the preserved candidate; otherwise the update fails before
  publication. It rejects in `all_registered`.
- `--check go`: valid only in `go_registered` or `rust_candidate`; it rebuilds
  the complete registered Go projection and byte-compares it with active Go
  state without writes.
- `--update-candidate rust`: valid only after Go registration and while no Rust
  tuple exists; it atomically replaces the complete one-file candidate
  directory. It never changes the active registry.
- `--check-candidate rust`: valid only in `rust_candidate`; it rebuilds from
  current sources and byte-compares the complete candidate transport.
- first `--update all`: valid only in `rust_candidate` with release languages
  exactly `["go","rust"]`. It rebuilds both languages from current sources,
  requires the regenerated Rust projection to equal the reviewed candidate
  member-for-member, merges all profiles, descriptors, and tuples in canonical
  order, and atomically publishes the new
  registry while removing the candidate in the same transaction.
- later `--update all`: valid only in `all_registered`; it rebuilds and
  atomically replaces the complete Go/Rust registry state.
- `--check all`: valid only in `all_registered`; it rebuilds both languages and
  byte-compares complete registered state without writes.

After first Rust registration, both candidate modes return
`BUNDLE_CANDIDATE_STATE` without even creating a private build directory. Only
`all` can rotate registered bundle metadata. A command cannot accept or
publish a partial language, descriptor, inventory, tuple, target, component,
or bundle subtree.

### 13.4 Publication

Every writer stages and validates the complete value it owns before its commit
point. File publication uses same-directory atomic replacement; an absent
content-addressed destination uses atomic no-replace. Every candidate or
registered update stages the complete `release/bundles` source-state tree,
including byte-identical `README.md`, current registry, and candidate when that
state permits one. It uses a same-volume atomic directory exchange (Linux
`renameat2(RENAME_EXCHANGE)`, macOS `renamex_np(RENAME_SWAP)`, or an
implementation proved to have the same single-commit semantics), then removes
the old exchanged tree. `--update-candidate rust` is the only mode allowed to
change candidate bytes; `--update go` preserves them exactly. The first
`--update all` omits the candidate only in the staged registered state, so the
exchange commits registry addition and candidate removal together. If the
primitive is unavailable, an update fails before changing registry or
candidate. Sequential registry-write/candidate-delete is forbidden.

Staging directories are freshly created with private mode, contain no links,
are not beneath an installation root, and are re-enumerated/rehashed before
commit. A writer never mutates a published tree in place. Check commands compare
canonical bytes and complete inventories and never invoke a formatting or
repair path.

## 14. Conformance vectors and test ownership

`develop/specs/vectors/release-bundles-v0.json` has schema
`mpk.release.bundle_conformance.v0`. Its exact top-level fields are `schema`,
`spec_schemas`, `owner_tests`, `fixtures`, `registry_cases`, `inventory_cases`,
`installation_cases`, `selection_cases`, `assembler_cases`, `limit_cases`, and
`hash_cases`. Case IDs are unique across all arrays and arrays retain file
order. `fixtures` has exactly `bootstrap_registry`, `valid_registry`, and
`bundle_bytes`. `bundle_bytes` is an ordered array of objects with exact fields
`bundle_id` and `files`; each file has exact `path`, four-digit octal `mode`,
and canonical padded `base64` fields.

The required root `owner_tests` array is exactly, in order:

1. `crates/mpk-vc/tests/release_bundle.rs`; and
2. `crates/mpk-cli/tests/frontend_runner.rs`.

The `bundle_bytes` values are opaque metadata/snapshot fixtures. Their short
synthetic bytes intentionally are not Linux ELF files and are used only for
raw length/digest, mode, no-follow, alias, enumeration, and immutable-snapshot
tests. They are excluded from assembler ELF-runtime derivation and actual exec
tests. Owner tests that exercise section 13's ELF derivation must use the exact
ELF fixtures later frozen by the owning Go/Rust profile specifications; they
must not derive the fixture's dynamic runtime declarations from these opaque
bytes. Installation cases stop after registry/layout/snapshot validation.
`accept_snapshot_original` asserts the digest of the sealed handle selected for
later execution; the owner does not invoke the opaque bytes.

### 14.1 Closed case records

Every case record, nested construction, operation, mutation, request, and
expectation is closed. The allowed top-level case fields are:

| Array | Exact allowed case fields |
|---|---|
| `registry_cases` | `id`, exactly one of `construction` or `json_text`, `expect` |
| `inventory_cases` | `id`, `construction`, `expect` |
| `installation_cases` | `id`, `construction`, `expect` |
| `selection_cases` | `id`, exactly one of (`registry_fixture`, `request`) or `input`, `expect` |
| `assembler_cases` | `id`, `argv`, `expect`, and only the applicable context fields `handler_state`, `state`, `build_input_state`, `release_languages`, `prospective_candidate_compatibility`, `rebuilt_rust_equals_candidate`, or `build_result` |
| `limit_cases` | `id`, `construction`, `expect` |
| `hash_cases` | `id`, `domain`, `canonical_payload_utf8_length`, `expected_preimage_length`, `expected_sha256`, and only the applicable source/assertion fields `canonical_payload`, `canonical_payload_sha256`, `fixture`, `pointer`, `remove_pointer`, or `different_from` |

Case `id` is a unique 1-through-128-byte lowercase ASCII value matching
`[a-z0-9_]+(\.[a-z0-9_]+)+`.
The closed outcome enum is exactly `accept`, `reject`, `accept_boundary`, or
`accept_snapshot_original`. `root_source` is an expectation field, not an
outcome; its only value is `already_opened_bin_mpk`.

An ordinary accepting `expect` has exactly `outcome`, except that exact tuple
selection additionally requires `selected_pointer_width` and
`selected_limit_profile_id`, and the ambient-root acceptance additionally
requires `root_source`. A rejecting model/install/selection/limit `expect` has
exactly `outcome`, `phase`, and `code`. An immutable-snapshot acceptance has
exactly `outcome` and `executed_sha256`.

Registry and inventory `phase` uses the eight tokens from section 1.
Installation phase is exactly `installed_layout`, `installed_registry`, or
`installed_bundle`; selection phase is exactly `selection`. Limit rejection
uses `transport` or `scalar`. `code` is the exact stable code shown in the
case. Unknown outcome, phase, code, or expectation field rejects the vector
container.

### 14.2 Model constructions

A registry construction has required `fixture` and `operations`, plus optional
boolean `rehash_registry`. An inventory construction has required `fixture`
and `operations`, plus only optional `pointer`, `rehash_content`, and
`rehash_registry`. `fixture` is `bootstrap_registry` or `valid_registry`;
`pointer` is an RFC 6901 JSON pointer. `rehash_content` is an ordered array of
pointers and flags are present only when true.

Each RFC 6902 operation has one exact shape:

- `add` or `replace`: `op`, `path`, and `value`;
- `remove`: `op` and `path`;
- `copy` or `move`: `op`, `from`, and `path`.

Operations apply in listed order. `rehash_content` pointers to a frontend or
toolchain descriptor recompute only its root
`bundle_sha256`/`distribution_sha256`; a pointer to a content component
recomputes only its `content_sha256`. `rehash_registry` then recomputes only the
root registry hash. No implicit dependent hash is repaired.

### 14.3 Installation constructions

An installation construction has exact required fields `registry_fixture`,
`bundle_bytes_fixture`, and `mutations`, plus optional `ambient_inputs`.
The fixture values are exactly `valid_registry` and `bundle_bytes`.
`ambient_inputs`, when present, has exactly
`environment_mpk_release_root`, `project_registry`, `adjacent_registry`, and
`cli_release_root`; all four are hostile ignored strings.

Each installation mutation is a closed tagged union:

| `kind` | Other exact fields |
|---|---|
| `remove` | `path`, and `bundle_id` only for a bundle-relative path |
| `symlink` | `path`, `target`, and `bundle_id` only for a bundle-relative path |
| `regular_file` | `bundle_id`, `path`, `mode`, `base64` |
| `chmod` | `bundle_id`, `path`, `mode` |
| `replace_bytes` | `bundle_id`, `path`, `base64` |
| `ambient_regular_file` | absolute `path`, `matches_declared_digest` |
| `reparse_point` or `fifo` | `bundle_id`, `path` |
| `hard_link` | `bundle_id`, `path`, `target_bundle_id`, `target_path` |
| `directory` | root-relative `path` |
| `replace_after_open` | `bundle_id`, `path`, `replacement_base64` |

Mutation order is event order. A bundle-relative mutation path is relative to
`libexec/mpk/bundles/BUNDLE_ID`; a root-relative path is relative to the
fixture release root. A mutation replaces the existing node when its path is
already occupied.

### 14.4 Selection and assembler records

A normal selection request allows exactly `registry_id`, `registry_sha256`,
`source_language`, `semantic_profile`, `target_id`, `frontend_bundle_id`, and
`toolchain_bundle_id`. Missing required members remain absent in a negative
case; no other member is allowed. `registry_fixture` is exactly
`valid_registry`. The alternate `input` branch contains the exact value offered
directly to the production selection boundary and cannot coexist with a
request.

An assembler case always has exact `id`, UTF-8 `argv` array, and `expect`.
Its optional context field vocabularies are closed:

- `handler_state`: `configured` or `not_configured`;
- `state`: `bootstrap`, `go_registered`, `rust_candidate`, or
  `all_registered`;
- `build_input_state`: `cache_missing`, `cache_invalid`, or
  `cache_key_occupied_unequal`;
- `release_languages`: exactly `["go","rust"]` where present;
- `prospective_candidate_compatibility`: exactly `equal_or_disjoint_ids`;
- `rebuilt_rust_equals_candidate`: boolean;
- `build_result`: exact `go` and `rust` fields, each `complete` or `missing`.

Assembler `expect` allows exactly these fields:
`outcome`, `exit`, `stderr`, `network`, `persistent_writes`,
`persistent_removals`, `commit_point`, `prerequisite`,
`requires_candidate_equality`, `candidate_preserved`, `candidate_bytes`,
`occupant_preserved`, and `private_build_directory_created`. `network` is
`disabled` or `fixed_origins_only`; `commit_point` is
`tracked_descriptor_atomic_replace`, `cache_atomic_no_replace`, or
`release_bundles_atomic_exchange`; `prerequisite` is exactly
`--check-build-inputs rust`; and `candidate_bytes` is exactly `unchanged`.
Booleans are JSON booleans. Persistent path arrays retain specification order.
Absent output fields mean the successful defaults fixed in section 13; a
rejection always contains exact `exit` and `stderr`.

### 14.5 Limit and hash records

A limit construction has exactly `kind` and integer `value`. `kind` is one of
the 14 limit IDs from section 2.4. Each family has consecutive below, at, and
above cases. `accept_boundary` asserts only that the named limit counter admits
the value; later semantic phases need not accept its synthetic object.

A hash case uses exactly domain `MPK-BUNDLE-REGISTRY-0.1` or
`MPK-BUNDLE-CONTENT-0.1`. Raw helper cases contain `canonical_payload` and no
fixture locator. Typed cases contain `fixture` plus exactly one of `pointer` or
`remove_pointer`; they assert the payload with exactly one of literal
`canonical_payload` or `canonical_payload_sha256`. Every case requires
`canonical_payload_utf8_length`, `expected_preimage_length`, and
`expected_sha256`. `different_from` is permitted only on an `*_mutation_after`
case and is the matching before-case digest.

`json_text` is passed as exact UTF-8. Hash helper cases test empty, minimal,
normal, integer-boundary, non-ASCII-key, and mutation bytes independently of
typed acceptance. A typed normal registry case may name the exact source
fixture plus canonical payload byte length and SHA-256 instead of duplicating
its multi-kilobyte JCS string; the owner regenerates that JCS, compares both
assertions, and then hashes the actual bytes. No owner may silently skip an
unknown case ID, ignore an unknown case field, or accept a vector by rewriting
its input.

## 15. Security invariants

- A caller selects only one exact registered language/profile/target/frontend/
  toolchain tuple; runtime discovery cannot widen it.
- Changing a descriptor, inventory entry, executable, native library, or target
  standard library changes a validated registry or content identity.
- Dynamic execution occurs only through a registered execution-host and
  runtime-layout profile whose complete native closure has been snapshotted.
- Registry build constants bind ID/hash only; installed bytes remain required
  and independently validated.
- The reference checker is the byte-rebuilt executable payload embedded in
  `bin/mpk`; it is never selected from `PATH`, an adjacent file, or a registry
  executable field.
- Open-before-hash and sealed identities prevent a mutable path from changing
  what is executed after validation.
- Build-only descriptors/caches and source-only candidates never become
  installation inputs or evidence selections.
- Check modes and provisioning cannot rewrite tracked release state;
  candidate/registered modes cannot repair or create build inputs.
- First Rust registration has one atomic commit point and cannot leave a stale
  second unregistered descriptor.
- No environment, user/project file, adjacent path, host library, default,
  latest version, or compatibility fallback participates in selection.
