# Rust Verification Frontend and Unified VIR Migration Design

Status: active Rust migration reference; the Go cutover is complete.

GIR_CUTOVER_STATUS: complete; RUST_PHASES: active; RETAINED_GIR_TERMINOLOGY: historical

This design never reinterprets the `mpk.gir.v0` schema identifier. It introduces
`mpk.vir.v0` as the only post-cutover source-program IR, migrates the Go path to
that schema, and then retires GIR v0 and its Go-specific frontend, VC, hash, and
policy interfaces plus the GIR-bound AI helper API route. Certificate v0
encoding and the mathematical trust boundary remain unchanged, although their
Go-specific documentation examples will be amended to describe VIR and
multiple source languages.

Prepared: 2026-08-20

## 1. Decision summary

MPK will support Rust by making a single breaking transition from the current
Go-only GIR pipeline to a shared, language-neutral Verification IR (VIR), then
adding an untrusted, fail-closed `rust2vir` frontend that obtains type-checked
Rust MIR from a pinned compiler toolchain.

In this document, “no compatibility” means that the post-cutover release does
not read old GIR, preserve old JSON or hash bytes, keep retired flags, or ship a
runtime adapter. The Go baseline is still compared at the semantic level so a
breaking format migration cannot silently change what an existing proof is
intended to mean.

The implementation will make the following architectural changes:

1. Add `mpk.vir.v0` as the sole serialized program input to VC generation.
2. Replace `go2gir` with `go2vir`, migrate every accepted Go program to VIR,
   and remove the production GIR importer rather than maintaining an adapter.
3. Replace Go-specific IR, VC, source-manifest, frontend, policy, and AI helper
   API boundaries with versioned language-neutral contracts used by both
   frontends.
4. Regenerate Go fixtures and hashes under the new schema; semantic behavior
   and rejection coverage must be preserved, but byte compatibility is not a
   goal.
5. Add the untrusted `rust-tools/rust2vir` tool outside the root Cargo workspace
   because it must use a separately pinned `rustc_private` toolchain.
6. Extract MIR after type checking and borrow checking, before MIR
   optimization, using `rustc_driver` callbacks.
7. Fix Rust v0 semantics to checked integer arithmetic, an explicit compilation
   target, `panic=abort`, no loops, and an acyclic static-call graph.
8. Encode source-language differences as explicit semantic profiles and
   required check sets; VC generation never infers semantics from a language
   name or frontend identity.
9. Generate runtime-safety VCs for every accepted operation that can panic.
10. Keep both source frontends, compilers, source, contracts, VIR, source maps,
    and VC output outside the trusted proof boundary.
11. Reuse the existing canonical `.mpcert`, Rust kernel, and independent Go
   source-free checker without adding a Rust-specific checker.

Rust support is complete only when a Rust source function can be lowered,
generate property and safety obligations, produce a canonical certificate, and
pass both source-free checkers. The migration is complete only when Go reaches
the same shared path and no production component consumes or emits GIR v0.
Producing VIR alone is not source-program verification.

C#, Java, Dart, TypeScript, and Python are planned only as post-Rust expansion.
No multi-language design, feasibility, specification, or production milestone
starts while this program runs. After the Rust hardening/release gate
`RUST-07-T05`, the follow-on program starts its own design phase and then
continues one phase at a time. The governing follow-on documents are
`06_multilanguage_frontend_design.md` and
`06_multilanguage_frontend_design-todo.md`.

## 2. Context and current constraints

The current implementation already separates its source frontend from proof
acceptance, but several helper-layer interfaces remain Go-specific.

| Current component | Go-specific constraint | Replacement design |
|---|---|---|
| `develop/specs/GIR_V0.md` | GIR means Go Verification IR and is frozen. | Keep its frozen Go-specific meaning, introduce VIR, migrate all producers and consumers, then retire GIR. |
| `develop/specs/GO_SUBSET_V0.md` | Its fail-closed boundary is phrased in terms of GIR emission. | Preserve its accepted/rejected behavior in a new `GO_VIR_PROFILE_V0.md`; mark the GIR-bound document historical after cutover. |
| `crates/mpk-vc/src/type_encode.rs` | Types encode to `Std.Go.Base.*`. | Replace it with a `Std.Program.Base.*` encoder used by both semantic profiles. |
| `crates/mpk-cli/src/policy_scan.rs` | The runner accepts only `mpk.go2gir.cli.v0` and `--go2gir`. | Replace the route with the generic frontend protocol and policy schemas v1. |
| `crates/mpk-cli/src/policy_evidence.rs`, `policy_report.rs`, and `ai_explain.rs` | Evidence targets, helper kinds, renderers, and the optional `mpk explain` path are typed around `package_path`, `gir_hash`, `GoSource`/`Gir`, and `mpk.policy.evidence.v0`. | Migrate every producer and consumer to the language-neutral evidence v1 model in the atomic cutover, including the explainer's validation, redaction, tests, and documentation. |
| `develop/specs/AI_API_V0.md` | The frozen VC helper API exposes `POST /gir/import`. | Replace the active profile with `AI_API_V1.md` and `POST /vir/import`; keep v0 only as a historical record after cutover. |
| `mpk.gir.v0` | The schema lacks source semantic context and complete checked-operation metadata. | VIR hashes the semantic profile and target parameters and carries canonical safety checks. |
| `mpk-vc` safety generation | Go-specific rules are derived from instruction names; ordinary arithmetic is wrapping. | Make every profile's required check set explicit, including both existing Go behavior and Rust checked arithmetic. |
| `mpk-vc` control-flow paths | Acyclic WP and Go loop handling are split across GIR-specific modules. | One VIR engine handles acyclic CFGs and contract-delimited loop cutpoints; Rust rejects cyclic CFGs. |
| `mpk-vc` static calls | `CallStatic` exists in the data model but is not accepted by the current WP path. | Add contract-based call VC generation shared by both profiles. |
| Certificate source manifest | The binary payload is opaque but its specification example is Go/GIR-specific. | Define a generic manifest and amend the example; no certificate encoding change is required. |
| Axiom policy v0 | It has `GoSemanticsAxiom` but no Rust category. | Add no Rust semantic axiom category; use checked definitions and existing checked theory interfaces. |

## 3. Goals

The Rust v0 work must provide:

- source-to-certificate verification for pure Rust library functions;
- compiler-resolved names, types, constants, control flow, and target width;
- exact, recorded compiler and Cargo versions;
- deterministic VIR, VC, source-manifest, and evidence hashes;
- one post-cutover VIR importer, validator, canonicalizer, and VC pipeline for Go
  and Rust;
- checked semantics for `bool`, fixed-width integers, `usize`/`isize` on an
  explicit 32- or 64-bit target, fixed arrays, and simple by-value structs;
- straight-line code, local mutation, branches, early returns, and acyclic
  calls to other accepted functions in the same crate;
- proof obligations for integer overflow, division/remainder failure, invalid
  shifts, and array bounds;
- deterministic rejected-feature diagnostics;
- preservation of the accepted Go subset's value, runtime-safety, contract, and
  loop semantics through regenerated VIR/VC/certificate fixtures;
- removal of active `mpk.gir.v0`, `mpk.go2gir.cli.v0`, `source_gir_hash`,
  `--go2gir`, the AI API v0 `/gir/import` route, and policy v0 interfaces at the
  atomic cutover;
- unchanged source-free certificate checking and checker agreement.

## 4. Non-goals

Rust v0 will not support:

- full Rust language verification;
- source-to-VIR translation inside the trusted base;
- references, raw pointers, aliasing, lifetimes as proof objects, or borrow
  reasoning;
- `unsafe` code, FFI, inline assembly, or layout-dependent operations;
- heap allocation, `Box`, `Vec`, `String`, slices, or trait objects;
- async functions, generators, threads, atomics, I/O, or nondeterminism;
- traits, trait method dispatch, closures, function pointers, or dynamic calls;
- generics, const generics, associated types, or monomorphization claims;
- enums, pattern matching, `Option`, `Result`, tuples, or destructuring;
- loops or recursion;
- explicit panic, `assert!`, `unreachable!`, indexing implemented through a
  user `Index` implementation, or unwinding behavior;
- procedural macros, build scripts, external Cargo dependencies, or user macro
  expansion;
- Cargo workspaces or workspace-inherited package configuration;
- floating point, `i128`, or `u128`;
- verification of ABI, physical struct layout, object code, or LLVM IR;
- runtime backward compatibility for GIR v0, `go2gir`, old VC JSON, policy
  scan/evidence v0, or AI helper API v0 after the cutover;
- changing certificate v0 binary encoding or adding source artifacts to the
  trusted checker inputs;
- cross-language calls inside one VIR module in v0;
- multi-language design, feasibility, specification, or production milestones
  before Rust v0 completes its final release gate; or
- placeholder future-language IDs, selection branches, semantic profiles,
  release tuples, or bundles in the frozen Go/Rust contracts.

These features require later semantic profiles. An unsupported feature must not
be approximated, erased, or interpreted as an unconstrained value.

## 5. Trust boundary

### 5.1 Trusted proof evidence

The unified VIR migration and Rust support do not add a new acceptance path.
Trusted evidence remains:

- canonical `.mpcert` bytes;
- checked theory certificates;
- the Rust kernel verdict;
- the independent Go source-free checker verdict;
- recomputed export, certificate, and axiom-report hashes;
- recomputed axiom dependencies permitted by policy.

### 5.2 Untrusted helper inputs and outputs

The following remain untrusted for proof acceptance:

- Go or Rust source and build manifests;
- the release bundle registry, descriptors, inventories, and registered bundle
  bytes;
- `go2vir`, `rust2vir`, and their rejected-feature detectors;
- Go tooling, `Cargo.lock`, the selected target, rustc, Cargo, compiler
  metadata, HIR, and MIR;
- contract sidecars;
- VIR, source maps, and VIR hashes;
- VC JSON, proof-search output, solver answers, and report prose.

The kernel must not read source programs, build metadata, SSA, MIR, VIR, source
maps, or compiler diagnostics. A successful frontend or compiler invocation is
not proof evidence.

### 5.3 Meaning of a source-program verification claim

At trust level 0, MPK proves only the theorem encoded in the certificate.

At the artifact-traceability level, MPK may report that a theorem was generated
from hash-pinned Go or Rust helper artifacts under a recorded release registry,
frontend, semantic profile, target, and toolchain. Because both frontends and
VIR are untrusted, this is not a mathematical proof that the theorem exactly
matches the source program.

User-facing evidence must keep these claims separate. `mpk_verified` continues
to require checked declaration or checked theory-certificate evidence.

## 6. High-level architecture

```text
Go source + Go contract
  -> go2vir package/SSA validation --------------------┐
                                                       |
Rust crate + Rust contract                             |
  -> rust2vir preflight                                |
       cargo metadata --locked --offline               |
       reject executable build inputs                  |
  -> pinned rust2vir-driver                            |
       pre-expansion source/AST gate                   |
       HIR validation + pre-optimization MIR lowering  |
                                                       v
                   mpk.frontend.cli.v0
                  canonical mpk.vir.v0
              + source map + generic source manifest
                              |
                              v
                  one mpk-vc VIR validator
                  profile-aware WP and safety
                              |
                              v
                property, call, loop, and safety VCs
                              |
                              v
                    canonical .mpcert
                              |
                              v
          Rust fast kernel + independent Go checker
```

`go2vir` replaces `go2gir`; it reuses the Go loader, SSA analysis, subset
detector, and contract parser but emits the generic frontend envelope and VIR.
There is no production GIR-to-VIR adapter after cutover. Development may use a
one-off comparison tool to audit fixture migration, but that tool is not an
accepted input path and is deleted or archived before the migration gate closes.

`rust2vir` is split into two executables in one isolated Cargo project:

- `rust2vir`: stable orchestration, Cargo preflight, hashing, contract loading,
  canonical emission, and CLI output;
- `rust2vir-driver`: the pinned `rustc_driver` integration invoked only for the
  selected library target.

The frontend process boundaries isolate compiler failures from the main `mpk`
process and make it possible to distinguish source rejection from frontend
bugs. Both frontends terminate at the same protocol and hash-verification
boundary.

## 7. Why a source gate, HIR, and MIR

No single compiler representation is sufficient. In particular, `cfg` can
remove an item before HIR exists, and macro expansion can erase the source form
that the subset policy needs to reject.

The pre-expansion source gate is responsible for:

- validating every selected crate source file before rustc expands it;
- rejecting `cfg`, `cfg_attr`, `path`, derive, macro definitions, macro
  invocations, `include!`, and other expansion-affecting constructs;
- recording the exact bytes and normalized path of every file returned to
  rustc;
- rejecting a source read outside the normalized source root.

The same source gate runs first during deterministic module-closure discovery
and again at the compiler boundary. Before rustc starts, `rust2vir` parses the
allowlisted library-root setting from `Cargo.toml` (or uses Cargo's documented
default), reads that root once, and recursively resolves only ordinary
out-of-line `mod name;` declarations using the pinned profile's default Rust
module-path rules. Inline modules are traversed in their containing file.
`cfg`, `cfg_attr`, `path`, macros, and every other construct that could alter
module discovery reject before a child path is followed. Both `name.rs` and
`name/mod.rs` existing for one declaration, duplicate normalized paths,
case-fold collisions, cycles, a missing module, or a path outside the source
root also reject deterministically. The walker does not glob the tree or scan
unrelated `.rs` files.

Expansion-affecting constructs are valid Rust outside this profile and return
`rejected` with `RUST_SUBSET_*`. Ordinary parse/name failures such as a missing
or ambiguous module return `source-error` with `RUST_SOURCE_*`. Symlink, root-
escape, path-normalization, and case-collision failures return `rejected` with
`RUST_PREFLIGHT_*`. The normative Rust subset specification freezes the exact
codes within those families.

Each discovered file is opened without following symlinks, read exactly once
into an immutable buffer, validated, and then copied from that buffer into the
private analysis snapshot. The driver installs a custom rustc `FileLoader`
that can return only those snapshotted bytes. On every Rust source read, the
loader lexes/parses and validates the file before returning it to the compiler;
the crate-root parsing callback performs the same validation on the root AST.
After compilation, the root callback plus loader inventory must equal the
preflight-discovered set exactly. An unexpected or unread discovered file is a
`frontend-error`, because it means the pinned compiler and discovery profile
disagree; it never causes a broader filesystem scan or an unsnapshotted read.

Expansion-affecting source-gate rules are crate-wide for every file compiled
into the selected library. Type, purity, and control-flow subset checks are
applied to the selected function dependency closure. The explicit module-path
and import restrictions in section 9.1 are also crate-wide because they affect
name resolution. This asymmetry is intentional: an item removed by `cfg` or a
scope-changing import cannot safely be assigned to a dependency closure only
after expansion and resolution.

HIR validation is responsible for constructs whose identity is erased or
desugared before MIR, including:

- `unsafe`, async, closures, loops, match, and destructuring;
- generics, traits, methods, impls, and remaining disallowed attributes;
- rejecting any selected item whose span still reports expansion provenance;
- statics, references, raw pointers, and disallowed type declarations;
- the source-level function and parameter names used by contracts.

MIR validation and lowering are responsible for:

- compiler-resolved primitive operations and constants;
- actual basic blocks and branch successors;
- local assignments and place projections;
- compiler-inserted overflow, division, shift, and bounds assertions;
- direct-call resolution;
- detecting any unsupported statement, rvalue, projection, or terminator left
  after HIR filtering.

The frontend will request the MIR stage after borrow checking and drop
elaboration but before optimization. For the pinned compiler, the profile
specification names the exact query and callback point, initially
`mir_drops_elaborated_and_const_checked`, and accesses it before a later query
can steal the body. If that query, access sequence, or MIR dialect changes, the
toolchain upgrade fails its compatibility gate and requires a reviewed adapter
change. Optimized MIR is not used because optimization may remove or reshape
source-level safety checks and make hashes more sensitive to optimizer changes.

## 8. Toolchain and Cargo execution

### 8.1 Pinned compiler

`rustc_driver` and `rustc_interface` are unstable compiler-internal APIs. The
frontend therefore has its own exact `rust-toolchain.toml` and records the
rustc commit hash it was built against.

The isolated Cargo package has one non-installable internal library target named
`rust2vir_internal`, used by its two binaries, unit/integration tests, and the
separately frozen fuzz package. The release frontend bundle contains only the
`rust2vir` main and `rust2vir-driver` executables; no rlib, dylib, test, example,
fuzz binary, or build artifact can enter an installed inventory.

The pinned build/test toolchain must include at least:

- an exact dated nightly toolchain, its host `rustc`, Cargo, and host standard
  library;
- `rustc-dev`;
- `llvm-tools`, whose contents are unstable and therefore pinned and inventoried
  at the selected nightly just like compiler libraries;
- `rust-src` only if the selected integration proves it is required;
- the exact nightly `rustfmt` and Clippy components used by repository gates;
- the exact `cargo-fuzz` executable used by the bounded hardening gate, built
  from its frozen source/manifest/lock dependency graph, plus its pinned C/C++
  compiler and libFuzzer/sanitizer build/runtime closure;
- the target standard library for every release-tested target.

`RUST_SUBSET_V0.md` freezes two related but distinct closed inventories. The
build/test materialization contains every component needed to build, format,
lint, link, and test `rust2vir`. In addition to the Rust components, it includes
the exact host linker/archiver and every other specification-allowlisted native
build-tool binary, startup objects, native development sysroot and libraries,
linker configuration, and runtime closure required by those tools; no ambient
`cc`, `ld`, SDK, or host development directory is used. It also includes the
checksum-verified dependency-source closure selected by the committed
`Cargo.lock`, plus separately frozen fuzz-manifest/lock and cargo-fuzz-tool
manifest/lock source closures, in an assembler-owned offline Cargo cache.

The reviewed production descriptor is tracked at the repository-derived path
`release/build-inputs/rust/build-inputs.json`; no environment or CLI value may
relocate it. The tracked transport is one compact RFC 8785 object followed by
one LF; the LF counts toward the descriptor byte limit but is excluded from
the hash preimage. Its canonical schema is `mpk.rust.build_inputs.v0`, and its
`build_inputs_sha256` field is:

```text
SHA256(
  "MPK-RUST-BUILD-INPUTS-0.1" || 0x00 ||
  canonical_build_inputs_without_build_inputs_sha256
)
```

The closed descriptor fields are: schema/profile/recipe and execution-host
profile IDs; Rust distribution, commit, components, targets, distribution-
archive digests, and inventoried tool-source digests; native linker/archiver/
tool/sysroot/runtime identities and origins;
the registry plus exact manifest/lock raw hashes and parsed package graphs for
the frontend, fuzz package, and cargo-fuzz tool; the cargo-fuzz source identity,
build recipe, and resulting executable digest; component provenance and
license/notice references; sorted component inventories whose file entries are
portable relative path, executable bit, byte length, and raw SHA-256; and
`build_inputs_sha256`. Unknown fields, duplicate IDs/paths, a machine-local
path, or an inventory/graph disagreement reject.
Before RUST-07-T03 creates the fuzz package, its manifest/lock fields refer to
the byte-exact spec-owned template hashes and graph; RUST-07-T03 must materialize
those template bytes unchanged before cargo-fuzz may run.

`develop/specs/vectors/rust-build-inputs-v0.json` owns synthetic byte-exact
schema/hash vectors and invalid mutations; the production descriptor is a
generated, reviewed instance of that frozen contract. It inventories every
build-only regular file exactly once and cannot list itself. Directories are
implicit; symlinks, hard-link aliases, devices, sockets, and unlisted entries
reject. The ignored materialized content lives only at
`release/build-input-cache/rust/<build_inputs_sha256>/`, whose final component
is the descriptor's recomputed lowercase hex digest. That cache subtree has
exactly `toolchain/`, `tool-sources/`, `native-sysroot/`, `native-runtime/`,
`vendor/`, `cargo-home-seed/`, and `notices/` as top-level entries.
Every reader enforces the section 18 descriptor, graph, path, per-file, and
aggregate-cache limits with checked counters before full JSON allocation or
unbounded file reads. A declared length outside the profile, an arithmetic
overflow, or an actual byte count that differs rejects before any cache content
is mounted or executed.

For the internal build launcher, `cargo-home-seed/` contains exactly one regular
file, `config.toml`, whose
byte-exact specification-owned contents replace the one frozen registry source
with a named Cargo directory source at fixed `/mpk/vendor` and set offline
operation. The seed contains no credentials, registry index/cache, executable,
link, or alternate configuration. No other source, registry, network,
credential-provider, alias, or external command is configured. Every directory
source package has the exact inventoried `.cargo-checksum.json`; the assembler
cross-checks those file hashes, the lockfile checksum, the parsed dependency
graph, and the descriptor before Cargo starts. A seed/config mutation, missing
checksum file, unlisted vendor child, or Cargo attempt to resolve outside
`/mpk/vendor` rejects.
After copying the seed, the launcher binds `config.toml` as a read-only,
no-replace file. `RUST_SUBSET_V0.md` freezes the exact other path shapes that
the pinned Cargo may create in the fresh private Cargo home; another config,
credential, source, executable, or unlisted entry rejects at the post-run
inventory check. A dependency custom-build, proc-macro, or native-tool child
cannot launch a nested Cargo process; only the separately frozen top-level
cargo-fuzz child graph may contain its exact nested Cargo invocation.

The launcher never executes or compiles directly from the ignored cache or a
mutable checkout. For each invocation it first enumerates the exact
specification-allowed frontend project paths, opens every source/cache input
without following links, and streams each opened regular file once into a
fresh invocation-owned materialization while hashing those copied bytes. Cache
entries must match the descriptor; descriptor-bound frontend manifest,
lockfile, toolchain file, and fuzz-template bytes must also match, while the
remaining current frontend sources form one captured invocation inventory.
Those current `.rs`, test, fixture, and fuzz-harness source bytes are not cache
entries or production-descriptor fields, so an ordinary source-only edit does
not rotate `build_inputs_sha256`.
The launcher normalizes the specified executable bits and metadata, seals the
completed frontend/vendor/toolchain/sysroot/runtime views read-only, and binds
only those private copies at the fixed `/mpk/*` paths. It never reopens an
original cache or checkout path after capture. The release assembler
re-enumerates and rehashes the current frontend source closure before candidate
or registered publication and requires equality with the captured build
inventory. A concurrent change, path-set disagreement, short read, or hash/
length mismatch rejects; `--check-build-inputs rust` is not authorization to
skip this invocation-local capture.

The assembler's exact `--update-build-inputs rust` maintainer mode is the sole
writer of the tracked descriptor. It fetches only frozen origins/digests,
builds cargo-fuzz twice from the inventoried tool source and dependency closure
inside separate empty sandboxes, and requires byte-identical output. It then
stages the complete cache at a fresh private temporary path, emits and validates
the descriptor from those staged bytes, computes the descriptor hash and final
cache key, publishes the cache to that no-replace path, and only then atomically
replaces the tracked descriptor as the commit point. An unused fully validated
cache published before a failed descriptor commit is harmless because no
descriptor selects it. The separate `--provision-build-inputs rust` mode
recreates only the ignored cache from an unchanged tracked descriptor for clean
machines: it stages the complete bytes privately, validates them against that
descriptor, and publishes only to its already fixed cache key. Both modes use
no-replace cache publication: an existing path is reused only after full byte
equality, while a malformed or unequal occupant fails without overwrite or
repair.
`--check-build-inputs rust` and `run-rust2vir-toolchain.sh` never fetch, write,
or repair either location. They strictly validate the tracked descriptor, path
key, and complete cache inventory before mounting any content. Before a routine
build, the launcher copies validated `cargo-home-seed/` into a fresh private
`/mpk/cargo-home`, without links or metadata drift, and discards all resulting
writes after the gate. Neither the descriptor nor cache is copied into a
candidate or installed release or accepted as a release-bundle root. Clean CI
may restore a cache only as untrusted input to the same complete check.

The frontend, fuzz, and cargo-fuzz-tool lockfiles may select only the one
specification-frozen registry and registry packages with nonempty lockfile
checksums. Git dependencies, alternate registries, `[patch]`, and `[replace]`
are rejected. The release frontend and cargo-fuzz-tool manifests have no path
dependency. The separate fuzz manifest may contain exactly one non-release path
edge to the parent `rust2vir` package at its fixed sandbox location and must
import only its `rust2vir_internal` library target; package identity,
library-target identity, and the complete already-inventoried frontend source
root must match. Every other path dependency or escape rejects.
Every vendored regular file, package checksum, source origin, license, and
required notice is inventoried, as are the provenance, license, and notices for
the linker and native development sysroot. Any dependency custom-build target
or procedural macro that executes while building/testing the frontend,
cargo-fuzz tool, or fuzz harness must be named by an exact package/version/
target/source-hash allowlist in
`RUST_SUBSET_V0.md`; an unlisted executable target fails before Cargo starts.
These build-only dependencies are distinct from analyzed Rust input, whose
dependencies, build scripts, and procedural macros remain entirely forbidden.

The evidence-execution toolchain bundle contains only Cargo, rustc, the
host/target standard libraries, and the complete compiler/LLVM runtime-file
closure needed to launch the registered driver. For the initial Linux host
this execution closure also includes the exact ELF interpreter and native
shared-library closure required by the staged Cargo, rustc, main, and driver
executables. The launcher constructs a private runtime root in which their
frozen interpreter paths resolve to those inventoried bytes; it never discovers
or mounts the host's `/lib`, `/lib64`, or `/usr/lib` as a fallback.
Developer-only `rustfmt`, Clippy, `rust-src`, dependency sources, build linker/
sysroot content, and caches are excluded unless the pinned integration proves a
file is part of the execution closure. The deterministic assembler may fetch
the exact pinned distribution and locked dependency sources only in
`--update-build-inputs rust` or `--provision-build-inputs rust`. Its
`--check-build-inputs rust`, bundle check modes, and evidence routes use the
already materialized bytes with network disabled, never install a component,
and never read or modify ambient rustup, Cargo, or native-library state.

All frontend and toolchain descriptor authority comes from one closed release
registry with schema `mpk.release.bundle_registry.v0`. It contains a unique
registry ID, every language/profile/target/bundle tuple, the complete frontend
and toolchain descriptors, and their canonical bundle inventories. Its
`registry_sha256` is:

```text
SHA256(
  "MPK-BUNDLE-REGISTRY-0.1" || 0x00 ||
  canonical_registry_json_without_registry_sha256
)
```

The generic runner is built with the exact expected registry ID and hash. At
startup it size-bounds, strictly parses, canonicalizes, and hashes the installed
registry bytes before resolving an entry; environment, project, command-line,
or adjacent installation files cannot override or extend it. The runner then
retains those validated immutable bytes for the execution rather than reopening
the registry path. A missing, noncanonical, unknown-field, duplicate-entry, or
hash-mismatched registry is `frontend-error` before any source frontend runs.
Changing any descriptor or inventory requires a new reviewed registry hash and
MPK release. The registry is reproducibility metadata and does not become
trusted proof evidence.

`RELEASE_BUNDLES_V0.md` freezes the assembler's stateful lifecycle. Before Go's
first registration, Go tests construct an in-process candidate afresh and no
tracked candidate or candidate assembler mode exists. After that registration,
`--update go`/`--check go` are the registered Go-only pair. Rust first uses the
tracked, non-installable `release/bundles/candidates/rust` projection:
`--update-candidate rust` is its sole writer and `--check-candidate rust` is
write-free. The first `--update all` must rebuild bytes equal to that current
candidate, register every Go/Rust tuple, and remove the candidate in the same
atomic publication. Thereafter both candidate modes reject without writes, and
`--update all`/`--check all` are the only complete registered Go/Rust rotation
pair. Every Rust candidate build and every registered `all` build first passes
`--check-build-inputs rust`; Rust candidate/registered update and all check
modes are network-disabled and cannot create or repair build inputs. Candidate
content is never an installation source or evidence selection.

Each release profile contains a closed toolchain-bundle descriptor with its
bundle ID, distribution digest, component names and releases, rustc commit,
directly invoked executable digests, and allowlisted target-library digests.
When an execution profile needs dynamically loaded native support, the same
descriptor also names its closed execution-host/runtime-layout profile and a
content component whose inventory contains the interpreter and native shared-
library closure. Those files are ordinary hash-checked bundle content, not an
ambient operating-system prerequisite.
The profile freezes the host OS/architecture/ABI, minimum kernel ABI, exact
interpreter mount locations, and every required isolation/file-publication
primitive, including read-only/no-exec bindings, network isolation, no-follow
opens, and atomic no-replace rename. The runner performs the profile's bounded
capability probes before exposing source or starting a frontend; a version or
capability mismatch is sandbox-unavailable, with no weaker fallback.
Its exact schema is `mpk.release.toolchain_bundle.v0`.
The evidence-producing caller names a registered bundle ID; language, semantic
profile, and target must match its registry tuple, with no default or
latest-version selection. Before launching a frontend, the generic runner
resolves that descriptor from the validated release registry, opens and
hash-checks the installed read-only bundle and its Cargo and rustc executables,
and constructs a private immutable toolchain view for the sandbox. Validation
and execution use the same
pinned file identities and never reopen a mutable installation path. The target
repository, ambient rustup state, and frontend response cannot select the
bundle. An unknown or incompatible ID is a pre-launch configuration error; a
registered file missing from the installation, extra executable request, or
digest mismatch is `frontend-error`.

Distribution and multi-file component digests are not hashes of host archive
metadata. `RELEASE_BUNDLES_V0.md` defines them as:

```text
SHA256(
  "MPK-BUNDLE-CONTENT-0.1" || 0x00 ||
  canonical_bundle_inventory
)
```

The inventory contains its distribution-or-component scope and every regular
file's portable relative path, executable bit, byte length, and raw-file
SHA-256, sorted by path. Directories are implicit; symlinks, hard-link aliases,
device files, sockets, and unlisted files reject. The launcher validates the
complete inventory before exposing the immutable view, so a changed library or
compiler support file cannot hide behind unchanged executable digests.

`rust2vir-driver` must refuse to run if the invoked rustc commit differs from
the commit embedded at frontend build time. A toolchain update is an explicit
reviewed change that regenerates every MIR/VIR golden fixture.

The ordinary root workspace remains buildable with its current stable Rust
toolchain. `rust-tools/rust2vir` is not a root workspace member.
Repository build, format-check, lint, test, and run gates for that isolated
project invoke a single internal launcher that directly selects the validated
pinned build/test materialization, native build sysroot, and private runtime
root. Its build namespace exposes the frontend source, vendored sources,
toolchain, and native sysroot read-only at specification-fixed paths, with
writes only to fresh home/Cargo-home/temp/target directories. The frontend
source is mounted at `/mpk/frontend`, vendor at `/mpk/vendor`, toolchain at
`/mpk/toolchain`, native development sysroot at `/mpk/native-sysroot`, and the
validated native-runtime view at `/mpk/native-runtime`; Cargo's working
directory is `/mpk/frontend`. It starts from a closed environment, disables
network and credentials, freezes linker selection and path-remapping flags,
normalizes input metadata, and fixes locale, timezone, hostname, job count, and
`SOURCE_DATE_EPOCH`. It constructs a toolchain-only `PATH` plus the fresh
offline `CARGO_HOME` copied from the validated seed. Cargo can therefore resolve
only inventoried rustfmt/Clippy, linker, build-script/proc-macro, and dependency-
source bytes; it cannot read an ambient SDK, Cargo configuration, host library
directory, or user file.
The launcher accepts only specification-frozen argv shapes for clean release
builds, format checks, Clippy, unit/integration tests, the version probe, and
the bounded fuzz-smoke targets; an arbitrary Cargo subcommand, target, feature,
profile, package, or trailing argument rejects before Cargo starts.
Fuzz-smoke copies an enumerated read-only seed corpus into a fresh private
writable corpus and uses a separate fixed private artifact directory, so
libFuzzer never mutates the checkout or chooses an output path.
`RUST_SUBSET_V0.md` also freezes the selected cargo-fuzz version's complete
bounded-smoke child-process graph: exact Cargo/rustc/native-tool argv, target,
engine/sanitizer/profile settings, and every environment addition, removal, or
replacement (including any rustflags or Cargo profile variable). The launcher
validates those children against the fixed sandbox paths and input inventories;
an unknown executable, argument, variable transformation, nested Cargo shape,
engine, or output locator rejects. No cargo-fuzz default may vary with an
ambient environment or a later tool release.
The initial dynamic-loader environment and every directory the pinned Cargo
version adds for a child process are frozen by the build profile; such additions
may resolve only inside the fresh private target directory or the validated
toolchain view, never to a host directory.
Process, memory, stdout/stderr, temporary-file, and output-size limits apply to
Cargo and every build-time child. Unavailability of that sandbox is a gate
failure. The release-bundle assembler uses the same materialization and
requires two separately empty clean release builds to produce byte-identical
main and driver files before it updates or checks an inventory. The launcher
never invokes a rustup proxy, installs a
component, or accepts an arbitrary toolchain path. Direct developer `cargo`
invocation is not an evidence or release gate.

### 8.2 Cargo preflight

Rust v0 requires `source-root` to be one self-contained Cargo package root with
one selected library target. A structural TOML/filesystem preflight first
rejects Cargo workspaces, workspace-inherited fields, symlinks, `.cargo`
configuration, target-repository `rust-toolchain`/`rust-toolchain.toml` files,
build scripts, and dependencies. The target repository cannot select or cause
rustup to install a toolchain; only the separately reviewed frontend toolchain
pin from section 8.1 is effective. Preflight then performs the module-closure
discovery from section 7 and creates a private analysis snapshot from the
already captured immutable buffers for the exact accepted manifest, lockfile,
contract files, and discovered Rust sources. Snapshot creation never rereads an
original input path.

From that snapshot, before invoking rustc, `rust2vir` runs:

```text
cargo metadata \
  --manifest-path /mpk/input/Cargo.toml \
  --format-version 1 --no-deps --locked --offline --no-default-features \
  --color never
```

This metadata command uses the same exact pinned Cargo binary, constructed
environment, isolated `CARGO_HOME`, controlled working directory, network
denial, filesystem sandbox, and output limits as the later `cargo check`. It
never runs from or reads the original source tree. Supplying an explicit
snapshot manifest path prevents Cargo from selecting a manifest by walking the
working directory's ancestors.

The preflight selects exactly one library target and rejects:

- a missing or stale `Cargo.lock`;
- a package-name mismatch or more than one matching library target;
- an explicit `crate-type` manifest field or any effective crate type other
  than Cargo's default Rust library (`lib`/rlib) form;
- a `[workspace]` table or inherited workspace field;
- any normal, build, development, or target-specific external dependency in the
  selected package;
- any build script or procedural-macro target in the selected package's
  compilation closure;
- enabled Cargo features in Rust v0;
- any build-affecting manifest table or field outside a versioned allowlist;
  descriptive package metadata may be ignored only from a separate explicit
  allowlist;
- project `.cargo/config` or `.cargo/config.toml` files;
- project `rust-toolchain` or `rust-toolchain.toml` files;
- a manifest outside the normalized source root;
- a non-UTF-8, nonportable, or case-fold-colliding relative input path;
- any non-regular captured input, symlink or filesystem reparse point in the
  copied input set, or any path that resolves outside the source root.

For Rust v0, a portable relative path uses `/` separators and nonempty ASCII
components containing only `[A-Za-z0-9._-]`. Components `.` and `..`, a
trailing dot, case-insensitive Windows device names (`CON`, `PRN`, `AUX`,
`NUL`, `COM1`-`COM9`, and `LPT1`-`LPT9`, with or without an extension), and two
paths equal under ASCII case folding reject. A component is at most 255 bytes
and the complete normalized path is subject to the section 18 limit. The
normative subset specification freezes this grammar so “nonportable” is not a
host-dependent filesystem judgment.

Other non-selected targets in the same package may exist, but they are
explicitly outside the claim scope and are not included in VIR. The evidence
identifies only the selected package and library target.

Cargo and rustc run only against the private snapshot. Source-manifest hashes
describe the bytes in that snapshot. This prevents parent-directory Cargo
configuration and concurrent changes to the original tree from changing the
analyzed input after hashing. Snapshot creation and cleanup use a dedicated,
validated temporary directory and never follow source symlinks.

### 8.3 Sanitized compilation

The frontend invokes the exact pinned Cargo and compiler from the analysis
snapshot, not a target-repository toolchain override. It clears or replaces
environment inputs that can affect compilation, including:

- `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS`;
- `RUSTC`, `RUSTC_WRAPPER`, and `RUSTC_WORKSPACE_WRAPPER`;
- `RUSTC_BOOTSTRAP`;
- incremental-compilation settings;
- target and profile overrides.

The launcher constructs the child environment from a versioned allowlist
rather than inheriting arbitrary process variables. All `CARGO_*`, `RUST*`,
target-specific compiler, linker, runner, registry, proxy, and credential
variables are absent unless the profile explicitly sets a deterministic value.
The wrapper and target directory variables required by this design are then
added with validated paths inside the analysis sandbox.

The Rust v0 environment profile initially passed to Cargo sets a private empty
`HOME`, isolated `CARGO_HOME`, private `TMPDIR` and `CARGO_TARGET_DIR`, a
toolchain-only `PATH`,
`LC_ALL=C`, `LANG=C`, `TZ=UTC`, `TERM=dumb`, `CARGO_TERM_COLOR=never`,
`CARGO_NET_OFFLINE=true`, `CARGO_INCREMENTAL=0`, and `RUST_BACKTRACE=0`.
For the exact initial Linux host triple/ABI frozen by `RUST_SUBSET_V0.md`, it
also sets `LD_LIBRARY_PATH` to the exact
nonempty, colon-separated, specification-ordered directories in the validated
private toolchain view that contain the pinned compiler and LLVM runtime
libraries. The value has no inherited suffix or empty element; every directory
and regular file reachable through it belongs to the validated toolchain
inventory. Other loader-path variables remain absent. A non-Linux host or a
toolchain layout that does not match the frozen loader-directory list is
sandbox-unavailable rather than an ambient-loader fallback.

The pinned Cargo version may extend the loader path it passes to an allowlisted
rustc child. `RUST_SUBSET_V0.md` freezes that exact per-invocation
transformation and directory order. The result may add only the applicable
compiler-created directories beneath the initially empty private `/mpk/target`
and validated sysroot directories beneath `/mpk/toolchain`; it may not add an
empty element, a source-controlled directory, or any host path. The wrapper
validates the transformed value before classifying a probe or primary
compilation. Before that wrapper process starts, the empty target is writable
only by the selected Cargo/rustc processes, and analyzed build scripts,
procedural macros, and user executables are forbidden. A different Cargo
transformation rejects as a pinned-toolchain mismatch.

Inside each fresh sandbox namespace, machine-local paths are mounted only at
these fixed virtual locations: the immutable source snapshot at `/mpk/input`,
the immutable selected toolchain view at `/mpk/toolchain`, the controlled
frontend view at `/mpk/frontend`, working directory at `/mpk/work`, the empty
home at `/mpk/home`, Cargo home at `/mpk/cargo-home`, temporary files at
`/mpk/tmp`, Cargo target output at `/mpk/target`, and driver output at
`/mpk/driver-output`, whose only final file is
`/mpk/driver-output/result.json`. The read-only private driver request is
mounted at `/mpk/driver-request.json`, and the validated read-only runtime is
mounted at `/mpk/native-runtime`. Thus `PATH` is exactly
`/mpk/toolchain/bin`, `RUSTC` is
`/mpk/toolchain/bin/rustc`, and `RUSTC_WORKSPACE_WRAPPER` is
`/mpk/frontend/rust2vir-driver`. The private native runtime root supplies its
frozen interpreter paths separately. Each individual fixed path value just
listed contains no `:`, `=`, or byte `0x1f`; the separately constructed
`LD_LIBRARY_PATH` contains only its required `:` separators between validated
path elements. No original install, workspace, source, home, or temporary
locator is forwarded into the namespace as an argv or environment value. A
platform unable to create these exact bindings is sandbox-unavailable.
The input, toolchain, frontend, work directory, request, and native-runtime
views are read-only; the work directory is freshly empty. Execution is enabled
only for inventoried toolchain/frontend/runtime executables, never for the input,
request, or work views. Only the freshly empty home, Cargo home, temporary,
target, and driver-output directories are writable, are owned by this
invocation, and cannot alias one another or any read-only view.

After digest and boundary validation, it sets `RUSTC` to the snapshotted rustc,
`RUSTC_WORKSPACE_WRAPPER` to the snapshotted driver, and
`CARGO_ENCODED_RUSTFLAGS` to the unit-separator encoding of the semantic flag
argv below. `RUSTFLAGS` remains absent. The value is the UTF-8 encoding of
these individual argv elements joined by one byte `0x1f`, with no leading or
trailing separator:

```text
["-C", "overflow-checks=yes",
 "-C", "panic=abort",
 "-C", "debug-assertions=no",
 "-C", "opt-level=0",
 "-Z", "mir-opt-level=0",
 "--remap-path-prefix=/mpk/input=."]
```

The final element is one argument. No element contains an incidental separating
space. The environment-profile specification freezes this exact byte
construction and the loader-directory construction above. No other variable is
present unless that profile gives its exact name and value.

Cargo's working directory is a controlled launcher root, not the original
repository. Every ancestor visible to Cargo's hierarchical configuration
search must be part of the sandbox and contain no `.cargo` configuration. If
the platform cannot establish that condition, verification stops with
`frontend-error`; it does not fall back to an unsandboxed Cargo invocation.

The compile command is equivalent to:

```text
cargo check --lib --package <package> --target <target> \
  --manifest-path /mpk/input/Cargo.toml \
  --locked --offline --no-default-features --jobs 1 \
  --message-format json --color never
```

The selected target triple is mandatory for verification. It has no implicit
host default in release evidence. The initial allowlist
`mpk.rust.targets.v0` contains exactly `i686-unknown-linux-gnu` and
`x86_64-unknown-linux-gnu`, providing reviewed 32- and 64-bit targets. Custom
target JSON and target paths reject. Adding a target requires a new registered
target-allowlist ID, its pinned standard-library component, and the complete
target corpus; an implementation cannot silently broaden v0.

In every evidence-producing execution, including local `policy scan` and
`policy verify`, Cargo and the compiler run in an OS sandbox that can read only
the pinned toolchain and analysis snapshot, can write only its dedicated
temporary target/output directories, has no network, credentials, or user-home
access, and has explicit process and memory limits. Sandbox failure is
`frontend-error`; it is never a reason to retry with broader access.

The driver is installed as `RUSTC_WORKSPACE_WRAPPER` and filters on the selected
primary package, library crate name, crate type, and manifest identity. It must
emit exactly one raw frontend artifact. Zero or multiple matching artifacts are
a deterministic frontend error. Cargo treats a single-package project as a
workspace for this wrapper mechanism.

Before Cargo starts, `rust2vir` atomically creates one bounded canonical
JCS+LF private request with schema `mpk.rust.driver.request.v0`, then exposes
those immutable bytes to the wrapper only at `/mpk/driver-request.json`. The
request contains the normalized source inventory and input-set hash, language/
profile/semantic parameters, target and selection, limit/environment/argument/
target-allowlist profile IDs, release-registry identity, expected main/driver
digests, toolchain bundle/distribution/component identities, compiler commit,
and its own `request_fingerprint`. It contains no executable, installation,
source-root, snapshot, output, home, or temporary path. No `MPK_*` environment
variable is used to transmit request state or an output locator; the output
file is the fixed `/mpk/driver-output/result.json` profile path.

`request_fingerprint` is recomputed as:

```text
SHA256(
  "MPK-RUST-DRIVER-REQUEST-0.1" || 0x00 ||
  canonical_driver_request_without_request_fingerprint
)
```

The driver strictly parses and byte-reencodes the request before classifying
an invocation. Missing, mutable, noncanonical, duplicate-key, oversized, or
identity-mismatched request bytes are a private protocol frontend error and
produce no output artifact.

The wrapper contract also freezes every Cargo compiler-probe invocation for the
pinned Cargo version, including its initial `rustc -vV` probe. An allowlisted
probe is delegated to the already selected rustc with the exact validated argv,
exit status, bounded stdout, and bounded stderr and emits no driver artifact.
The same no-artifact rule applies to any other explicitly allowlisted
non-primary invocation. An unknown probe or non-primary invocation rejects; it
is not silently passed through. Only the exact selected library compilation may
create the one private driver artifact.

The corresponding output boundary uses the versioned schema
`mpk.rust.driver.v0`; neither private schema is VIR or appears in a certificate
or evidence report. The main frontend provides a fresh output directory. A
matching driver creates the exact regular
temporary file `/mpk/driver-output/result.json.partial` with no-follow and
exclusive-create semantics, writes and bounds the complete bytes, then
atomically renames it without replacement to the exact regular file
`/mpk/driver-output/result.json`. After the child exits, the directory must
contain only `result.json`; a missing/remaining partial, unexpected entry,
second writer, replacement attempt, link, or oversized file is
`RUST_FRONTEND_DRIVER_PROTOCOL_*`. The artifact serialization is compact JCS,
as defined in section 12.5, followed by one LF. It has an exact tagged
status, and every status carries the deterministic request
fingerprint, expected driver digest, compiler/toolchain identity, and requested
package, crate, and function, plus `source_inventory_hash`. Success additionally
carries the normalized source inventory with hashes and raw lowered program/
source-map data. A non-success driver status carries bounded normalized
diagnostics but no partial source inventory, lowered program, source map, or
payload hash.

The always-present output `source_inventory_hash` and success-only
`payload_hash` are:

```text
SHA256(
  "MPK-RUST-SOURCE-INVENTORY-0.1" || 0x00 ||
  canonical_normalized_source_inventory
)

SHA256(
  "MPK-RUST-DRIVER-PAYLOAD-0.1" || 0x00 ||
  canonical_driver_success_payload_without_payload_hash
)
```

For all three private hash domains, each `canonical_*` operand is the exact RFC
8785 UTF-8 byte sequence frozen by `RUST_DRIVER_PROTOCOL_V0.md`, with the named
self-hash field omitted where applicable and without a trailing LF. The single
LF belongs only to the request/output file transport and is appended after the
complete object has been encoded; it is never part of a hash preimage.

The request carries the expected `source_inventory_hash`; every output repeats
the `request_fingerprint` and `source_inventory_hash`. The main recomputes the
request and source-inventory domains for every status and, on success, the
payload domain before it interprets the raw lowered payload.

The request fingerprint covers the normalized source-inventory/input-set
identities, canonical profile, target, selection, option-profile IDs, validated
release-registry identity/hash, and expected binary/compiler/toolchain digests;
it excludes executable, installation, snapshot, and output paths.

`rust2vir` strictly parses this artifact, rejects duplicate or unknown fields,
recomputes its inventory and payload hashes, and compares every repeated
request, binary, toolchain, selection, and present source identity with values
captured outside the driver before it creates the public frontend envelope. A
missing artifact after a compiler failure is locally classified as
`frontend-error`; stale or partial output is never reused.

Before public emission, the main frontend converts raw driver spans only to the
captured input paths and UTF-8 byte ranges allowed by `mpk.source_map.v0`. An
expansion span, external file, invalid byte boundary, unknown VIR reference, or
profile-required mapping that escaped the earlier gates is `frontend-error`
with `RUST_FRONTEND_SOURCE_MAP_*`; compiler-internal span data is never leaked.

The driver validates the final effective rustc session options after Cargo has
composed profile and command-line settings. Target, edition, panic strategy,
overflow checks, debug assertions, MIR optimization level, features, and `cfg`
values must exactly match the recorded request. Supplying the desired flags is
not sufficient if a conflicting setting remains effective. Before creating a
compiler session, the wrapper also rejects any rustc argument outside a
versioned allowlist, except explicitly normalized input and output paths. This
prevents Cargo configuration from injecting an unreviewed `-C` or `-Z` option
whose effect is not covered by the smaller semantic-option fingerprint.

## 9. Rust subset v0

The normative profile name is `mpk.rust.checked.v0`.

The accepted lists in this section are closed. A source, HIR, or MIR construct,
item category, type, operator/type combination, coercion, adjustment, or
compiler-generated form not explicitly accepted by the normative subset
specification rejects with a stable code; the frontend never treats an omitted
case as harmless or lowers it by analogy.

### 9.1 Crate and item scope

Accepted:

- a selected Cargo library target using Rust edition 2021;
- inherited/private or bare `pub` visibility on an accepted module, function,
  constant, struct, or struct field; visibility is type-checked by rustc but is
  not a VIR semantic field;
- ordinary file and inline modules under the source root;
- free functions whose parameters are plain immutable identifier patterns with
  explicit types and which have exactly one explicit, accepted, non-unit return
  type;
- primitive `const` items used by accepted functions whose initializers are a
  boolean literal, an in-range integer literal, or a leading-unary-minus signed
  integer literal whose final typed value is in range;
- named structs used by value and containing only accepted fields;
- source paths made only from `crate`, `self`, `super`, and ordinary module/item
  identifier segments, resolving to an accepted item in the selected crate;
- same-crate helper functions in the selected function's static call closure,
  using either inherited/private or bare `pub` visibility as allowed above.

Crate, module, function, parameter, local, constant, struct, and field names in
the selected dependency closure must use ASCII Rust identifiers matching
`[A-Za-z_][A-Za-z0-9_]*` and must not be the discard name `_`. Raw identifiers
and non-ASCII identifiers reject so contract IDs and canonical JSON do not need
a second identifier-normalization rule in v0.

The selected Cargo package name must match
`[A-Za-z][A-Za-z0-9_-]*` and is compared byte-for-byte with `--package`; it is
not rewritten from the library crate name. The selected crate name follows the
Rust identifier rule above and must match the first segment of `--function`.

Rejected:

- binaries as verification targets;
- `static` and `static mut` items;
- `impl`, trait, associated, extern, async, const, unsafe, or variadic
  functions;
- generic parameters or `where` clauses;
- custom linkage, FFI, exported ABI, or `no_mangle` behavior;
- `extern crate`, `use`, glob imports, import aliases, and re-exports; accepted
  cross-module references use the explicit same-crate path forms above;
- restricted visibility forms such as `pub(crate)`, `pub(super)`, and
  `pub(in path)`;
- semantic attributes such as `cfg`, `cfg_attr`, `path`, `repr`, derive, test,
  target features, and inline assembly;
- expanded user macros in the selected call closure.

The Rust v0 attribute allowlist is exactly crate-level `#![no_std]` and inert
`doc = <string-literal>` attributes, including the equivalent forms generated
from line/block documentation comments. Other `doc` meta forms and macro-valued
documentation reject. Lint-level attributes reject; in particular, source
cannot lower or cap overflow, unconditional-panic, unsafe-code, or other
compiler diagnostics that affect acceptance. The driver also rejects
unapproved effective `-A`, `-W`, `-D`, `-F`, or `--cap-lints` arguments. Any
other attribute rejects.

### 9.2 Types

Accepted source types and VIR encodings:

| Rust type | VIR type |
|---|---|
| `bool` | `{"kind":"bool"}` |
| `i8`, `i16`, `i32`, `i64` | signed BV8/BV16/BV32/BV64 |
| `u8`, `u16`, `u32`, `u64` | unsigned BV8/BV16/BV32/BV64 |
| `isize`, `usize` | signed/unsigned BV32 or BV64 from the explicit target |
| `[T; N]` | fixed array, when `T` is accepted and `N` is an in-range literal or accepted primitive constant whose compiler-resolved type is exactly target-width `usize` |
| named struct | nominal VIR struct with fields in declaration order |

Rejected types include:

- `i128`, `u128`, `f32`, `f64`, `char`, strings, and `str`;
- references, raw pointers, slices, vectors, boxes, and trait objects;
- tuples, enums, unions, function types, closures, and never type;
- generic or dynamically sized types;
- any type for which rustc reports that drop glue is required.

Array length constants are not generalized from their numeric value. A named
constant of `u8`, `u32`, `isize`, or any type other than `usize` does not become
a valid length merely because its value fits; ordinary rustc type checking
rejects it before lowering. The accepted length is then converted to the
schema-bounded mathematical array arity, while its source constant declaration
retains target-width `usize` type and target-sensitive hashes.

VIR models struct fields nominally and never relies on Rust's physical layout.
Endianness, padding, ABI, and niche optimizations are outside the profile.

After borrow checking, a whole-place MIR `Move` of an accepted no-drop value is
lowered as a pure value transfer, using the same VIR value read as `Copy`.
This ownership erasure is valid for the profile because references, aliasing,
interior mutation, and drop glue are rejected, while rustc has already rejected
use-after-move. A projected move or partial move rejects in v0; a projection is
accepted only when rustc classifies the projected operand as `Copy`.

### 9.3 Statements and control flow

Accepted:

- simple, initialized, non-shadowing `let [mut] name [: accepted type] = ...`
  identifier bindings;
- assignment to a `let mut` local with plain `=`;
- expression statements whose result is an accepted value;
- blocks, `if`/`else`, early `return`, and a final return expression;
- direct calls to accepted functions in the same crate.

Rejected:

- field or array-element mutation;
- uninitialized bindings, shadowing, and compound assignment;
- destructuring assignments and patterns;
- `loop`, `while`, `for`, `break`, and `continue`;
- `match`, `if let`, and `let else`;
- recursion or any cycle in the selected static call graph;
- `panic!`, `assert!`, `debug_assert!`, `unreachable!`, and abort calls;
- the `?` operator and any unwind or cleanup edge.

Because loops and recursion are rejected, termination of accepted control flow
is structural. Later loop support requires invariants and decreases clauses and
will use a new or amended semantic profile.

### 9.4 Expressions and operations

Accepted:

- boolean literals and `!`, `&&`, `||`;
- in-range integer literals whose type is resolved by rustc, including an
  accepted leading-unary-minus literal normalized as one constant;
- integer `+`, `-`, `*`, `/`, `%`, unary `-`;
- integer bitwise `&`, `|`, `^`, `!`, `<<`, `>>`;
- equality for booleans and accepted integers, and signed/unsigned ordered
  comparisons for accepted integers;
- fixed-array construction with an explicit full element list and read-only
  indexing by a compiler-resolved `usize` expression;
- struct construction with every field explicitly initialized and direct field
  read;
- simple local values and accepted constants;
- compiler-resolved same-crate paths to accepted constants, structs, and free
  functions using the source forms from section 9.1;
- direct calls to accepted same-crate free functions.

Rejected:

- overloaded operators or comparisons;
- integer `as` casts in v0;
- wrapping, checked, overflowing, or saturating library methods in v0;
- user method calls and indexing through `Index`/`IndexMut`;
- an array index whose compiler-resolved type is not `usize`;
- ranges, closures, allocation, references, dereference, and raw-address
  operations;
- repeated array expressions such as `[value; length]`;
- struct update syntax;
- inline constants whose evaluated type or value is outside the accepted model.

Rejecting casts and integer helper methods avoids assigning semantics to library
calls before call and conversion theory support is complete. They can be added
independently in a later profile.

### 9.5 Purity rule

An accepted function is pure only when it:

- reads parameters, constants, and locals only;
- writes locals only;
- constructs values without allocation;
- calls only accepted pure functions;
- accesses no static, thread-local, environment, clock, random, I/O, atomic,
  volatile, or foreign state;
- contains no reference, interior mutability, drop glue, panic operation, or
  unknown compiler intrinsic.

The borrow checker accepting a function does not establish this MPK purity
rule. The HIR and MIR validators enforce it independently and fail closed over
the selected functions and every referenced constant and type declaration.

## 10. Rust semantic profile

### 10.1 Integer carriers

Accepted integer values use the existing fixed-width bitvector carriers.
Signedness is an explicit interpretation for comparison, division, remainder,
shift, and overflow predicates.

Signed comparison uses the two's-complement signed interpretation. Signed
division truncates toward zero, signed remainder has the dividend's sign,
signed right shift is arithmetic, and unsigned right shift is logical. Bitwise
operations and the value component of checked arithmetic use the fixed-width
bitvector result. These rules are encoded explicitly; they are not inferred
from an MPK type name.

Both migrated Go VCs and new Rust VCs encode types through
`Std.Program.Base.*`, not `Std.Go.Base.*`. `Std.Program.Base` provides checked
aliases over existing Bool, BitVec, fixed-array, and struct foundations. The
aliases introduce no new axioms. The migration regenerates declarations and
certificate hashes that previously named `Std.Go.Base.*`; no compatibility
alias remains on the post-cutover VC path.

### 10.2 Checked arithmetic

Rust v0 fixes `overflow-checks=yes`. Normal arithmetic therefore has two
effects in VIR:

1. compute the bitvector result; and
2. require the matching no-panic safety predicate.

VIR bitvector value operations are total and language-neutral. Whether a
source operation must prove a failure condition is represented solely by its
canonical `safety_checks` set and validated against the module's semantic
profile. Thus the same `bv_add` value operation has no overflow check under the
`mpk.go.fixed.v0` profile and has `integer_no_overflow(add)` under the
`mpk.rust.checked.v0` profile.

| Rust operation | VIR value operation | Required safety check |
|---|---|---|
| signed/unsigned `+` | `bv_add` | `integer_no_overflow(add)` |
| signed/unsigned `-` | `bv_sub` | `integer_no_overflow(sub)` |
| signed/unsigned `*` | `bv_mul` | `integer_no_overflow(mul)` |
| signed unary `-` on a nonconstant value | `bv_neg` | `integer_no_overflow(neg)` |
| unsigned unary `-` | rejected by rustc | none |
| `/` | signed/unsigned division | divisor nonzero; signed `MIN / -1` excluded |
| `%` | signed/unsigned remainder | divisor nonzero; signed `MIN % -1` excluded |
| `<<`, `>>` | signed/unsigned shift operation | RHS is nonnegative when signed and less than LHS width |
| array index | fixed-array read | index is less than array length |

A leading-unary-minus integer literal that rustc accepts, including the
canonical minimum-value spelling such as `-128_i8`, lowers as one typed
`Const`; it does not emit `bv_neg` or an overflow check. The source/HIR gate
must distinguish that literal form before MIR constant folding. Every accepted
nonconstant signed negation lowers as `bv_neg` with
`integer_no_overflow(neg)`.

The frontend must recognize the exact MIR checked-operation/assertion pattern.
It consumes a compiler assertion only when it can map it to one of the required
VIR safety checks. An orphan assertion, an assertion with an unknown message,
or a checked result whose overflow flag is otherwise used rejects.

The VIR validator independently checks that `mpk.rust.checked.v0` instructions
carry the complete safety-check set required for their operation and type. This
does not make the untrusted frontend sound, but it prevents accidental omission
inside the helper pipeline.

### 10.3 Panic policy

The compilation strategy is `panic=abort`, but accepted theorems prove that no
modeled panic condition is reachable under the function preconditions.

Compiler-inserted assertions become runtime-safety obligations. Explicit panic
operations reject. A MIR unwind, cleanup, resume, terminate, or drop path that
cannot be consumed as a recognized safety check rejects.

### 10.4 `usize` and `isize`

`usize` and `isize` take their width from the mandatory target triple. The
target identifier and pointer width are hashed VIR semantic parameters; the
complete sorted rustc `cfg` set is also recorded in the source manifest.

Rust v0 rejects `cfg`-dependent source, references, pointers, and layout
operations. Consequently, target dependence in accepted VIR is limited to the
width of `usize` and `isize`. Re-running with another target or pointer width
changes the VIR semantic context and therefore the VIR, VC, manifest, and
evidence hashes even when the selected function does not mention a
target-sized integer.

### 10.5 Function calls

Only statically resolved same-crate calls are accepted. The selected call graph
must be acyclic, and every function in the closure must have a matching
contract.

The call closure is discovered from compiler-resolved HIR before MIR
reachability filtering. Every direct call written in an accepted function body
participates in the closure, even when it occurs in a source-dead branch that
rustc later removes. MIR simplification therefore cannot silently remove a
callee from subset, purity, cycle, or contract validation.

That conservative HIR closure is the exact function set lowered into the VIR
module and the set against which a contract file is considered used. Every
member therefore remains a standalone `VirFunction` even if its only incoming
source call disappears from reachable MIR. The inter-function semantic and
declaration-dependency graph is separately reconstructed from reachable
`CallStatic` instructions. A HIR-only dead call creates no call-site obligation
or declaration edge, but its callee is still validated, hashed, and certified
as a standalone closure member. Function ordering uses the reachable VIR call
graph and the canonical-ID tie-break below, so the frontend cannot smuggle HIR
edge order into canonical artifacts.

For each call, the VC generator must:

1. prove the callee's `requires` clauses in the current path state;
2. introduce a fresh result value;
3. assume the callee's checked `ensures` clauses for subsequent obligations;
4. depend on the callee's checked panic-free theorem;
5. reject signature, type, semantic-profile, or contract-hash mismatch.

Each accepted function exposes two logical evidence groups under stable names:

- `<function>.contract`, covering postconditions, call-site callee
  preconditions, and, for Go, loop initialization, preservation, exit, and
  decreases obligations; and
- `<function>.panic_free`, covering operation runtime-safety obligations and
  dependencies on each called function's checked panic-free declaration under
  the same preconditions.

The semantic profiles fix that exhaustive membership partition. VC v1 retains
the ordered member obligation IDs and the certificate skeleton emits each group
as exactly one theorem declaration. Its body is the canonical conjunction of
members in stable obligation-ID order; an empty panic-free set uses the
canonical checked `True` proposition. Generated caller declarations reference
the exact checked declaration hashes required by the dependency rules below; a
declaration bundle, individually checked report row, or successful callee scan
is not an alternate representation.

The declaration type has one form: canonical function-parameter binders,
followed by an implication from the canonical conjunction of `requires`
clauses (empty means checked `True`) to the group conjunction. Each member is
an implication from its ordered path-specific assumptions (empty also means
checked `True`) to its conclusion, wrapped by that member's canonical anonymous
loop-state binders. Those binders are empty outside a loop cutpoint; otherwise
they are exactly the free post-substitution function locals followed by header
block parameters in VIR order, encoded by type and referenced with de Bruijn
indices. This keeps loop preservation, exit, decreases, safety, and call
members closed without promoting compiler-local names to declaration-level
parameters. Common preconditions are never distributed into the members, and
member implications or their local binder scopes are never flattened or
reassociated. For ordered terms, `Conjoin([]) = True`, `Conjoin([x]) = x`, and
for `n >= 2`, `Conjoin(xs) = And(Conjoin(xs[..floor(n/2)]),
Conjoin(xs[floor(n/2)..]))`. This order-preserving balanced split avoids linear
term depth. The outer and member implications are emitted even when their
antecedent is `True`. VC v1 freezes the exact `True`/`And`/`Imp` serialization
markers and both outer and member-local binder order. Certificate assembly then
applies the contextual checked-proposition lowering in
`PROGRAM_CERTIFICATE_ALPHA_V0.md`, so logically equivalent alternate syntax
cannot change declaration hashes.

Within one function, the contract declaration is emitted first. Among generated
function-group declarations, the dependency set is exact: the caller's
contract declaration references the callee contract declaration for each
distinct callee appearing at one or more `CallStatic` sites; the caller's
panic-free declaration references its own contract declaration and both the
callee contract and callee panic-free declarations for each such callee. Every
referenced generated declaration appears once, and the dependency list is
sorted by referenced declaration name in UTF-8 byte order. The own-contract
edge permits reuse of its checked callee-precondition conjuncts;
callee-contract edges permit use of postconditions for later values and
safety; callee-panic-free edges discharge the call-safety members. The caller
contract never references its own panic-free declaration, no callee declaration
references a caller declaration, and no additional generated group edge is
permitted. Dependencies on fixed checked foundations remain governed by
contextual lowering and the active certificate-assembly profile. Thus the
fixed contract-then-panic-free order plus topological callee order cannot
introduce a declaration cycle or duplicate an obligation.

Policy v1 may classify and display individual members, but every member record
references its containing declaration name and hash. A property is checked only
when that whole declaration is present and accepted; evidence cannot treat a
subset of conjuncts as an independently checked theorem.
`mpk_verified` for a selected function requires both of its declarations and
every transitive callee declaration dependency to be accepted under the active
checker and axiom profiles.

Functions and certificates are emitted in topological call order; when more
than one function is ready, the smallest canonical function ID by UTF-8 byte
order is chosen. Each function's contract declaration precedes its panic-free
declaration. Dynamic dispatch, function values, recursion, and external calls
reject.

## 11. Rust contract sidecar

Rust v0 uses an untrusted JSON sidecar rather than attributes or procedural
macros. This avoids executing contract code during compilation and keeps
contracts hashable independently of the source compiler.

Schema: `mpk.rust.contract.v0`.

```json
{
  "schema": "mpk.rust.contract.v0",
  "semantic_profile": "mpk.rust.checked.v0",
  "target_pointer_width": 64,
  "function": "payment_policy::approved_reserve_cents",
  "requires": [
    {
      "op": "signed_ge",
      "lhs": {"var": "amount"},
      "rhs": {"int": {"value": "0", "width": 64, "signed": true}}
    }
  ],
  "ensures": [
    {
      "op": "signed_ge",
      "lhs": {"result": 0},
      "rhs": {"int": {"value": "0", "width": 64, "signed": true}}
    }
  ],
  "modifies": [],
  "panic": "forbidden",
  "termination": "total",
  "loops": []
}
```

Rules:

- `function` is `<crate_name>::(<module>::)*<function>` using the
  compiler-resolved crate and zero or more module segments; rustc internal
  `DefId` and crate-disambiguator values never enter the public identifier;
- the target must resolve to exactly one accepted free function;
- `semantic_profile` must equal the frontend request;
- `target_pointer_width` must equal the selected target's recorded width;
- `requires` must be present and may be empty;
- `ensures` must be present and nonempty;
- `modifies` and `loops` must be present and empty;
- `panic` must be `forbidden`;
- `termination` must be `total`; loops and recursion are rejected by the
  profile rather than accepted on an unchecked termination assertion;
- contract variables resolve to source parameter names and are normalized to
  VIR argument IDs before hashing;
- local variables are not contract-visible in v0;
- integer literals carry explicit width and signedness;
- unknown fields, operators, names, result indexes, and type mismatches reject;
- every function in the static-call closure has its own contract and hash;
- `--contract` is repeatable, contracts are indexed by canonical function ID,
  and duplicate or unused contract files reject.

Rust v0 contract atoms are typed parameter variables, `result` index `0`,
boolean literals, and explicitly typed bitvector integer literals. Its operators
are exactly:

- unary `not`, plus `and` and `or` with between two and 64 boolean operands;
- `eq` and `not_eq` over two values of the exact same accepted type;
- `signed_lt`, `signed_le`, `signed_gt`, `signed_ge`, and their `unsigned_*`
  counterparts over matching bitvectors whose signedness matches the operator;
- total logical `bv_add`, `bv_sub`, `bv_mul`, `bv_and`, `bv_or`, `bv_xor`,
  `bv_neg`, `bv_not`, `bv_shl`, `bv_ashr`, and `bv_lshr` operations.

Operand arrays retain source sidecar order; the parser does not flatten,
reassociate, commute, or deduplicate expressions. All comparison and binary
bitvector operators have arity two, and `bv_neg`/`bv_not` have arity one.

After name/type resolution and argument renaming, both frontends encode each
function's contract as the same language-neutral VIR contract object. It
contains the canonical function/unit IDs, semantic profile and parameters,
ordered normalized clauses, modifies/panic/termination/loop fields, and a
`contract_hash` computed as:

```text
SHA256(
  "MPK-CONTRACT-0.1" || 0x00 ||
  canonical_vir_contract_json_without_contract_hash
)
```

The VIR validator recomputes it, and `CallStatic` repeats the expected callee
contract hash. The raw sidecar's manifest input SHA remains distinct: changing
only JSON whitespace changes source traceability but not the normalized
contract or VIR hash.

Binary contract bitvector operands and results have the same exact type except
for shifts: a shift's LHS and result match, while its RHS may be any accepted
integer type and follows the full-count rule in section 12.2. Unary operations
preserve their operand type.

Aggregate values may appear only as exact-typed variables or results in
`eq`/`not_eq`; field selection, indexing, aggregate literals, conversion,
division, and remainder are not contract operators in Rust v0. Contract
array equality is fixed-length componentwise equality, and struct equality
requires the same nominal VIR type and compares fields componentwise in
declaration order; `not_eq` is the checked negation of that equality. Contract
bitvector operations are total logical operations and never create runtime
safety obligations. Safety obligations arise from the source program only.
`RUST_SUBSET_V0.md` copies this closed list and must not inherit operators
implicitly from the Go profile.

## 12. Verification IR v0

### 12.1 Purpose

VIR is an untrusted, language-neutral program IR for VC generation. It is not a
proof artifact. Its schema is separate from frozen GIR v0.

Schema: `mpk.vir.v0`.

```text
VirModule:
  schema = mpk.vir.v0
  source_language
  semantic_profile
  semantic_parameters
  units
  vir_hash

VirSemanticParameters ::=
  GoFixed:
    target_id
    pointer_width
  | RustChecked:
    target_id
    pointer_width
    overflow_mode = checked
    panic_mode = abort

VirUnit:
  id
  name
  type_decls
  const_decls
  functions

VirFunction:
  id
  unit_id
  name
  params
  results
  locals
  blocks
  contracts
  features_used

VirInstruction:
  id
  kind
  type
  kind-specific fields, flattened into the tagged instruction object
  safety_checks

VirBlock:
  label
  parameters
  instructions
  terminator
```

`VirInstruction` is an exact tagged union, not a record with a generic
`operands` bag. `VIR_V0.md` freezes the required and forbidden fields for every
instruction kind; for example, `BinOp` has the flattened `op`, `lhs`, and `rhs`
fields shown below. Unknown, missing, or inapplicable kind-specific fields
reject.

One VIR module contains units from exactly one source language and semantic
profile. A unit is a Go package or Rust crate; mixed-language modules and
cross-language calls reject in v0. `semantic_parameters` has an exact
profile-specific schema with unknown fields denied. For the initial profiles:

- `mpk.go.fixed.v0` records the canonical Go target identifier and pointer width
  needed by any accepted target-sized frontend construct;
- `mpk.rust.checked.v0` records the Rust target triple, pointer width,
  `overflow_mode = checked`, and `panic_mode = abort`.

The profile and all semantic parameters are part of canonical VIR. Toolchain
versions and frontend binary identities remain in the source manifest because
they are traceability inputs, not program semantics.

VIR v0 replaces the existing GIR value operations with their language-neutral
equivalents and adds explicit `safety_checks`. It retains every operation and
control-flow feature needed by the accepted Go subset, including conversions,
phi/block parameters, and loop contracts, while adding no heap, reference,
exception, or physical-layout semantics. Rejected-feature diagnostics belong
to the frontend envelope, not a successfully lowered VIR module.

Example checked addition:

```json
{
  "id": "t0",
  "kind": "BinOp",
  "op": "bv_add",
  "type": {"kind": "bv", "width": 64, "signed": true},
  "lhs": {"var": "arg0"},
  "rhs": {"var": "arg1"},
  "safety_checks": [
    {"kind": "integer_no_overflow", "operation": "add", "signed": true}
  ]
}
```

Allowed safety-check kinds in VIR v0 are:

- `integer_no_overflow` with `add`, `sub`, `mul`, or `neg`;
- `divisor_nonzero`;
- `signed_divrem_representable` with `div` or `rem`;
- `shift_count_nonnegative`;
- `shift_count_less_than_width`;
- `index_in_bounds`.

When an instruction requires more than one check, its `safety_checks` array is
sorted by the kind order above and then by the declared operation order
`add`, `sub`, `mul`, `neg`, `div`, `rem`; duplicate checks reject.

Checks reference the owning instruction operands; they do not carry an
arbitrary frontend-supplied proposition. `signed_divrem_representable` means
`lhs != MIN || rhs != -1`. `index_in_bounds` means `0 <= index < length` for a
signed index and `index < length` for an unsigned index. Shift checks compare
the count against zero using its signed view and against the left operand width
after a nonnegative count is established.

`callee_precondition` and `callee_panic_free` are call-site obligation kinds,
not instruction `safety_checks`; the VC generator derives them from
`CallStatic` and the resolved callee contract.

Unknown kinds reject. Required checks are profile-validated; extra checks also
reject so that hashes and obligations remain canonical.

### 12.2 Profile-explicit value and failure semantics

The `source_language` field is descriptive and constrains the allowed profile;
the VC generator never branches on it. A total VIR value operation has the same
meaning in every module. The semantic profile determines the exact checks that
must accompany each source operation:

Both initial source profiles lower `&&` and `||` to `Branch` control flow with
the right operand evaluated only on the language-defined path. They never emit
an eager Boolean `BinOp` for source short-circuit syntax. Consequently, a
runtime-safety or call obligation in the right operand carries the left
operand's path assumption. Boolean `!` remains a total unary value operation;
contract-side `and` and `or` are total logical operators, not source execution.

| Operation | `mpk.go.fixed.v0` required checks | `mpk.rust.checked.v0` required checks |
|---|---|---|
| signed/unsigned add, subtract, multiply | none; fixed-width result wraps | matching `integer_no_overflow` |
| signed negate of a nonconstant value | none; fixed-width result wraps | `integer_no_overflow(neg)` |
| unsigned negate | none; fixed-width result wraps | source form rejects |
| signed divide or remainder | `divisor_nonzero` | `divisor_nonzero` and matching `signed_divrem_representable` |
| unsigned divide or remainder | `divisor_nonzero` | `divisor_nonzero` |
| shift with signed count | `shift_count_nonnegative` | `shift_count_nonnegative` and `shift_count_less_than_width` |
| shift with unsigned count | none | `shift_count_less_than_width` |
| fixed-array index | `index_in_bounds` | `index_in_bounds` |

The shared `index_in_bounds` predicate supports signed and unsigned VIR index
types because the Go profile needs both. Rust source array indexing is narrower:
the source/HIR/MIR validators require the compiler-resolved primitive array
index to be exactly target-width `usize`, so a Rust `Index` instruction always
uses the unsigned bound. An `isize`, fixed-width integer, cast, or user
`Index`/`IndexMut` route is rejected before Rust lowering and is not justified
by the shared predicate's wider type support.

VIR shift instructions permit an accepted integer RHS whose width and
signedness differ from the LHS. The total value operation interprets the RHS
bit pattern as an unsigned mathematical count without truncating it to the LHS
width. Counts at least the LHS width yield zero for left/logical-right shift and
the repeated sign bit for arithmetic-right shift. A signed RHS is additionally
viewed as signed only by `shift_count_nonnegative`; once that check holds, its
unsigned count is the same nonnegative value. `shift_count_less_than_width`
compares that full count with the LHS width.

For Go, a nonnegative shift count greater than or equal to the value width is
valid and uses the total bitvector shift result. For Rust, the same count is a
panic condition. For signed `MIN / -1` and `MIN % -1`, the total bitvector value
operation supplies Go's result while the Rust profile requires the
representability check. These differences are therefore visible in VIR and do
not rely on frontend identity or build mode.

Except for the explicitly cross-width shift-count rule above, VIR v0 copies the
exact total fixed-size bitvector equations from the pinned SMT-LIB 2.7
`FixedSizeBitVectors` theory and standard BV logic extensions into `VIR_V0.md`;
it does not incorporate a moving external document by reference. This fixes
division and remainder even when the divisor is zero, although both initial
source profiles require `divisor_nonzero`, and fixes signed overflow results
such as `MIN / -1` even when one profile also requires a safety check. Checked
MPK definitions and vectors implement those copied equations; a solver's
built-in interpretation is not trusted evidence.

### 12.3 Source lowering to VIR

`go2vir` constructs VIR directly from the existing type-checked Go/SSA
pipeline. No serialized GIR is produced or consumed. Existing GIR concepts map
mechanically to their VIR replacements, but the migrated Go frontend must set
`mpk.go.fixed.v0`, emit the profile-required check set, preserve loop
contracts, and retain current fail-closed behavior.

The Go migration also removes implicit host build context. `go2vir` loads an
immutable input snapshot with a versioned environment allowlist, the recorded
Go toolchain, explicit `GOOS`/`GOARCH`, `CGO_ENABLED=0`, and read-only module
resolution. `GO_VIR_PROFILE_V0.md` freezes the exact loader flags, module and
workspace policy, source-file inventory, and treatment of standard-library or
module-cache inputs. An inherited environment, implicit host target, unrecorded
external file, or build constraint outside that policy rejects.

The Rust MIR mapping is:

| MIR form | VIR lowering | Rust v0 behavior |
|---|---|---|
| local `Assign` from `Use` or constant | `Const` or `Copy` | accept `Copy` or a whole-place no-drop `Move`; reject projected moves |
| primitive `BinaryOp` | `BinOp` | accept whitelisted operator/type pair |
| `CheckedBinaryOp` plus matching `Assert` | `BinOp` plus safety check | accept exact recognized pattern |
| primitive `UnaryOp` | `UnaryOp` | accept bool not, bit not, checked signed neg |
| array/struct aggregate | `MakeArray` / `MakeStruct` | accept complete by-value construction |
| field or fixed-array projection | `Field` / `Index` | accept read-only projection |
| `Goto` | `Jump` | accept |
| boolean `SwitchInt` | `Branch` | accept exactly two boolean successors |
| `Return` | `Return` | accept one modeled result |
| direct local `Call` | `CallStatic`, then `Jump` | accept contracted acyclic callee |
| compiler safety `Assert` | consumed into safety check | reject if not consumed exactly once |
| `Drop`, `Unwind`, `Yield`, `InlineAsm`, unknown form | none | reject |
| dereference, downcast, subslice, opaque cast | none | reject |

Storage markers and other semantically empty MIR statements may be ignored only
from a version-specific explicit allowlist. A new MIR enum variant or changed
shape rejects until reviewed.

### 12.4 Stable identifiers

Compiler-local indices are not public identifiers. The emitter deterministically
renames:

- arguments to `arg0`, `arg1`, ... in source signature order;
- return values to `result0`;
- user locals to `local0`, `local1`, ... in accepted HIR declaration order;
- compiler temporaries to `t0`, `t1`, ... in canonical block traversal order;
- reachable blocks to `bb0`, `bb1`, ... using breadth-first traversal from the
  entry block, enqueuing a `Jump` target directly and a `Branch` false target
  before its true target.

Instructions and compiler temporaries within a block retain accepted MIR
statement order. A successor already discovered is not enqueued again. VIR v0
uses the same traversal for both profiles and defines the successor order of
every future terminator before that terminator can be accepted.

Source names and spans live in a separate untrusted source map. Contracts are
resolved before renaming. Unreachable MIR blocks are omitted only after HIR
validation has checked their source constructs; reachable unknown blocks reject.

Go identifiers continue to use their compiler-resolved import path and function
identity after validation. Rust identifiers follow section 11. A `VirUnit.id`
is the canonical Go import path or Rust crate name. Because a module has one
source language, these namespaces cannot collide across languages. Compiler or
SSA-local IDs never enter the public identifier space.

### 12.5 Validation and canonical hash

VIR validation must check at least:

- exact schema, language/profile pairing, and profile-specific semantic
  parameters;
- unique unit, type, constant, function, block, instruction, local, and value
  IDs;
- closed value references and valid successor labels;
- operand and result type agreement;
- complete instruction shape with unknown fields denied;
- exactly one entry block and a valid terminator on every reachable block;
- an acyclic reachable CFG for Rust v0;
- an acyclic resolved call graph for both initial profiles; recursion requires a
  future induction and termination design;
- valid loop cutpoints, invariants, and profile-appropriate termination
  metadata for cyclic Go CFGs;
- resolved same-module callees and no cross-language call;
- complete profile-required safety checks;
- nonempty postconditions for every function, plus empty
  modifies/loops for Rust v0;
- deterministic size limits.

`vir_hash` is SHA-256 over a domain-separated canonical JSON payload that
excludes the `vir_hash` field itself:

```text
SHA256("MPK-VIR-0.1" || 0x00 || canonical_vir_json_without_hash)
```

Object keys and all unordered collections are sorted. Numeric values that may
exceed JSON's interoperable integer range are decimal strings. Absolute paths,
compiler-local IDs, timestamps, hostnames, and temporary paths never enter VIR.
Duplicate object keys reject before canonicalization.
The semantic profile and parameters are always hash inputs; changing only a
target identifier or pointer width changes `vir_hash`.

“Canonical JSON” in every hash-bearing or protocol JSON defined here—including
VIR, source maps, frontend envelopes, contracts after parsing, source manifests,
VC and certificate-skeleton documents, policy v1 payloads, and AI v1
request/output payloads—means RFC 8785 JSON Canonicalization Scheme (JCS),
narrowed to valid Unicode strings, booleans, null, arrays, objects, and integral
JSON numbers in the inclusive range
`[-9007199254740991, 9007199254740991]`. Schemas reject floating-point numbers;
larger mathematical integers use the declared decimal-string forms.
Duplicate object names reject before JCS processing, and no additional Unicode
normalization is performed. Schema-defined ordered arrays retain their order;
only collections explicitly declared unordered are sorted before JCS encoding.
Every hash formula over canonical JSON uses the compact JCS bytes with no BOM,
trailing whitespace, or LF. The single LF required by a CLI transport is not
part of a nested artifact, manifest payload, or hash preimage.

## 13. Generic frontend protocol

Both `go2vir` and `rust2vir` implement the generic frontend envelope schema
`mpk.frontend.cli.v0`. The old `mpk.go2gir.cli.v0` envelope is not accepted
after cutover.

```json
{
  "schema": "mpk.frontend.cli.v0",
  "status": "ir-lowered",
  "source_language": "rust",
  "semantic_profile": "mpk.rust.checked.v0",
  "semantic_parameters": {
    "target_id": "x86_64-unknown-linux-gnu",
    "pointer_width": 64,
    "overflow_mode": "checked",
    "panic_mode": "abort"
  },
  "selection": {
    "package": "payment-policy",
    "crate": "payment_policy",
    "kind": "lib",
    "function": "payment_policy::approved_reserve_cents"
  },
  "ir": {
    "schema": "mpk.vir.v0",
    "sha256": "...",
    "value": {}
  },
  "source_manifest": {},
  "source_map": {
    "schema": "mpk.source_map.v0",
    "source_ir_schema": "mpk.vir.v0",
    "source_ir_hash": "...",
    "entries": [],
    "source_map_hash": "..."
  },
  "rejected_features": [],
  "diagnostics": []
}
```

`selection` has an exact language-specific shape: Rust records Cargo package,
crate, library-target kind, and canonical function ID; Go records canonical
import path and function ID. It contains identities, not filesystem paths, and
unknown or inapplicable fields reject.

An `ir-lowered` response contains exactly one `mpk.source_map.v0` object. It is
an untrusted auxiliary artifact, not proof evidence. Its entries use a closed
tagged reference union for VIR functions, instructions, and terminators, plus a
normalized input path and zero-based half-open UTF-8 byte offsets. The path
must identify a manifest input with `kind = source`, and the range must fit its
recorded `size_bytes`, satisfy `0 <= start < end <= size_bytes`, and begin and
end on UTF-8 scalar boundaries. Entries are sorted by their canonical VIR
reference, and duplicate references reject. `SOURCE_MAP_V0.md` and each
language profile freeze the exact reference shapes and the total mapping rule
for source-derived nodes; a frontend cannot invent a span for an unknown or
synthetic form.

`source_map_hash` is:

```text
SHA256(
  "MPK-SOURCE-MAP-0.1" || 0x00 ||
  canonical_source_map_json_without_source_map_hash
)
```

The repeated source IR schema and hash must equal the successful VIR. The
consumer recomputes the source-map hash and validates every reference and
range. The map contains no raw source text, line rendering, absolute path, or
compiler-internal span identity.

Before launch, the generic runner constructs the complete expected selection
from caller-controlled values. For Rust, `package` is `--package`, `function`
is `--function`, `crate` is the first segment of that canonical function ID,
and `kind` is the profile constant `lib`; preflight must prove that this crate
is the selected Cargo library target. For Go, the import path and function ID
come directly from `--package` and `--function`. A frontend therefore cannot
invent a derived selection field that the consumer merely echoes back.

`--semantic-profile` is mandatory on every lower request. A frontend accepts
only profiles registered for its fixed source language and never substitutes a
default, even when v0 currently defines only one profile for that language.
The envelope, VIR, manifest, and policy artifacts repeat and cross-check that
caller-selected value.

A missing, syntactically invalid, unknown, or wrong-language profile is a CLI
configuration error: a standalone frontend returns exit 2 with no JSON, and
the policy CLI rejects it before launching any frontend. It is not reported as
an unsupported source feature and cannot fall back to another profile.

Statuses and exits:

| Status | Exit | Meaning |
|---|---:|---|
| `ir-lowered` | 0 | Complete VIR, source map, and manifest emitted. |
| `rejected` | 3 | Valid input used a feature outside the selected profile. |
| `source-error` | 4 | The language loader or compiler rejected malformed or ill-typed source input. |
| no JSON | 2 | CLI usage error. |
| `frontend-error` | 1 | Compiler crash, protocol failure, toolchain mismatch, or internal error. |

Classification follows one fixed phase order: CLI/profile/registry-assertion
parsing; installed release-registry and registered frontend/toolchain
validation; immutable input capture and structural preflight; source parse and
pre-expansion gate; Cargo metadata validation; rustc parse/name/type/borrow
checking; HIR subset and contract resolution; MIR validation/lowering;
canonical emission. A phase completes or fixes the
non-success class before the next phase begins. Malformed language input in the
source/compiler phases is `source-error`; a well-formed construct outside the
closed profile, contract rejection, or deterministic source/IR structural or
profile-limit refusal is `rejected`; an executed phase's process, identity,
protocol, or internal invariant failure is `frontend-error`. Diagnostics from
later phases are never mixed into an earlier failure. `FRONTEND_PROTOCOL_V0.md`
and the language profiles freeze finer same-phase precedence, collection, and
stable codes.

Exit 2 is the only protocol classification that intentionally has no JSON
response. If a child is killed, exits without a complete JSON value, or cannot
report its own internal failure, the consumer locally classifies the attempt as
`frontend-error`; it does not invent a successful envelope or reuse partial
stdout. A frontend that remains able to handle an internal failure emits the
canonical `frontend-error` envelope and exits 1.

Every JSON-bearing exit writes exactly one compact canonical UTF-8 envelope
followed by one LF. The consumer strictly parses it, re-encodes it, and requires
byte equality; a BOM, insignificant whitespace, a second value, or any other
stdout byte rejects as `FRONTEND_PROTOCOL_NONCANONICAL`. Exit 2 writes no
stdout. Frontend and child stderr are diagnostic transport only and never enter
a canonical artifact or evidence hash.

The protocol consumer treats the complete response as untrusted. It accepts
only an exact status/exit pairing and one size-bounded JSON value, rejects
duplicate keys, unknown fields, and trailing stdout, recomputes the canonical
VIR, source-map, and source-manifest hashes, and verifies all repeated
language, profile, semantic-parameter, compilation-target, source-selection,
release-registry, and artifact-hash fields for equality wherever they recur.
The selected function must resolve uniquely inside the returned VIR. The
launcher snapshots
and hashes the complete configured frontend binary set before starting it.
Language, semantic profile, target, and the caller's mandatory registered
bundle ID resolve one versioned release frontend-bundle descriptor from the
validated registry in section 8.1. The descriptor contains the bundle ID, main
identity/digest, and closed, name-keyed subordinate identity/digest set. Its
exact schema is
`mpk.release.frontend_bundle.v0`; `RELEASE_BUNDLES_V0.md` also freezes the
language/profile/target registry key and denies unknown descriptor fields. An
evidence-producing caller supplies the registered bundle ID, never a main or
helper executable path. The runner resolves the installed paths internally,
opens and snapshots those registered bytes, and then launches only them. An
unknown or incompatible caller-selected ID is a pre-launch configuration error;
a registered descriptor, executable, or digest missing from the installed
release is `frontend-error`. Registration does not move any frontend into the
proof trust boundary.

Rust v0 requires exactly the registered `rust2vir-driver`; Go v0 requires no
subordinate executable. The launcher passes only its snapshotted helper path
and expected digest to the main frontend and constrains sandbox execution to
that set plus the independently selected release toolchain bundle. It requires
`source_manifest.frontend.bundle_id` and `binary_sha256` to match the release
descriptor and snapshotted main executable, and requires
`source_manifest.frontend.subordinate_binaries` to match the configured helper
names and digests exactly. The returned toolchain identity must likewise equal
the launcher's bundle descriptor. A response cannot add a helper, choose its
own toolchain, or redirect execution to another path. Any mismatch is
`frontend-error`, not `rejected` and never a partially ready scan.

For every non-success status (`rejected`, `source-error`, or
`frontend-error`), no partial VIR, VIR hash, source map, source manifest, or
artifact hash is emitted. Rejected features and normalized source diagnostics
are sorted by normalized path, start position, code, and message. The bounded
truncation entry defined in section 18, when present, is the sole exception and
is always the final `diagnostics` entry.

Suggested Rust invocation:

```text
rust2vir lower <source-root>
  --manifest-path Cargo.toml
  --package <package-name>
  --semantic-profile mpk.rust.checked.v0
  --target <target-triple>
  --function <canonical-function-id>
  --frontend-bundle-id <launcher-selected-bundle-id>
  --frontend-sha256 <launcher-snapshotted-main-sha256>
  --release-registry-id <launcher-validated-registry-id>
  --release-registry-sha256 <launcher-validated-registry-sha256>
  --driver <launcher-snapshotted-rust2vir-driver>
  --driver-sha256 <expected-sha256>
  --toolchain-bundle-id <launcher-selected-toolchain-bundle-id>
  --toolchain-root <launcher-validated-toolchain-root>
  --toolchain-distribution-sha256 <expected-distribution-sha256>
  --contract <relative-contract-path> ...
```

The migrated Go frontend uses the same selection model:

```text
go2vir lower <source-root>
  --package <import-path>
  --semantic-profile mpk.go.fixed.v0
  --target <goos>/<goarch>
  --function <canonical-function-id>
  --frontend-bundle-id <launcher-selected-bundle-id>
  --frontend-sha256 <launcher-snapshotted-main-sha256>
  --release-registry-id <launcher-validated-registry-id>
  --release-registry-sha256 <launcher-validated-registry-sha256>
  --toolchain-bundle-id <launcher-selected-toolchain-bundle-id>
  --toolchain-root <launcher-validated-toolchain-root>
  --toolchain-distribution-sha256 <expected-distribution-sha256>
  --contract <relative-contract-path> ...
```

Source, manifest, and contract arguments are resolved against `source-root`.
Rust v0 requires `--manifest-path` to normalize to the literal `Cargo.toml` at
that root; the explicit option prevents ancestor lookup rather than permitting
a nested package. The generic policy runner supplies that fixed value for Rust;
the machine-local `source-root` locator itself is never recorded.
The registry identity/hash, frontend and toolchain bundle IDs, expected main
digest, expected toolchain distribution digest, toolchain root, and executable
arguments in the lower protocol are launcher-only values, not policy-user
flags. The toolchain distribution argument is exactly the
`distribution_sha256` from the validated release toolchain-bundle descriptor;
it is not a separate manifest hash. Executable and toolchain paths are absolute
inside validated private bundles, are opened and hashed before execution, and
never enter canonical artifacts as paths. A standalone developer invocation
not launched from that registry cannot yield accepted policy evidence. Private
snapshot, target, and driver-output paths are absolute validated sandbox paths
and are likewise excluded. User-facing policy report destinations follow the
CLI's safe-write and explicit fixture-overwrite rules but are not frontend
request fields or artifact hash inputs. Policy v1 does not copy their
machine-local spellings into evidence.

## 14. Generic source manifest

Schema: `mpk.source_manifest.v0`.

Every frontend emits the same top-level shape:

```text
schema
source_language
semantic_profile
semantic_parameters
selection
limit_profile
release_registry:
  schema
  id
  registry_sha256
toolchain:
  bundle_id
  distribution_sha256
  components[]:
    kind
    name
    release
    commit_hash?
    binary_sha256?
    content_sha256?
frontend:
  bundle_id
  name
  version
  binary_sha256
  subordinate_binaries[]:
    name
    version
    binary_sha256
units[]:
  identity
  name
  kind
target:
  id
  pointer_width
  language_configuration
inputs[]:
  kind
  normalized_path
  size_bytes
  sha256
input_set_hash
vir_hash
source_map_hash
vc_hash, when attached to a certificate
source_manifest_hash
```

The manifest's `source_language`, `semantic_profile`, `semantic_parameters`,
and `vir_hash` must exactly match VIR, the frontend request, and the successful
frontend envelope wherever the field recurs. `source_map_hash` must equal the
validated source map. Its `selection` must exactly match the request and
envelope, and its selected function must resolve uniquely in VIR. The duplicate
`target.id` and `target.pointer_width` values must equal the corresponding
`semantic_parameters.target_id` and
`semantic_parameters.pointer_width`;
`language_configuration` must equal the normalized effective configuration
prescribed by the request and selected profile. Rust's toolchain components
include rustc, Cargo, and LLVM identities; Go records the Go toolchain identity.
`toolchain.bundle_id`, `distribution_sha256`, components, and executable
digests must equal the launcher's selected release descriptor exactly.
`limit_profile` must equal the release registry's fixed value for the selected
language and semantic profile; a frontend cannot self-select it.
`release_registry` must equal the launcher's validated schema, ID, and hash
exactly. The frontend cannot substitute another registry or omit the identity
merely because its selected bundle fields happen to have the same values.

The `frontend` bundle ID, names, versions, and main/subordinate digests likewise
equal the release frontend descriptor and snapshotted executable set; paths are
never recorded.

Toolchain component `kind` is exactly `executable` or `content`.
`binary_sha256` is required and `content_sha256` forbidden for every directly
invoked executable. Every non-executable component, including each target
standard library, requires `content_sha256` and forbids `binary_sha256`; the
approved whole-distribution digest is also required. Components have unique
names and are sorted by name. Frontend subordinate binaries likewise have
unique names and are sorted by name. A subordinate compiler driver appears in
`frontend.subordinate_binaries`, not as a Rust-only top-level field.

For Rust, `language_configuration` is an exact object containing edition
`2021`, crate type `lib`, an empty enabled
feature set, the exact `std` or `core` prelude mode derived from `#![no_std]`,
`locked = true`, `offline = true`, `default_features = false`,
`overflow_checks = true`, `panic = abort`, `debug_assertions = false`, rustc and
MIR optimization levels `0`, `jobs = 1`, `message_format = json`, the versioned
target-allowlist, environment-profile, and rustc-argument-allowlist IDs, and the
complete sorted rustc `cfg` set. Paths and output filenames are excluded. The
Go profile specification defines an equally exact loader/build-configuration
object; neither frontend may add an implementation-specific setting.

Each input entry has a versioned `kind`, a normalized source-root-relative path,
byte length, and SHA-256 digest. Normalized paths are unique. Entries are sorted
by the UTF-8 bytes of `normalized_path`, then `kind`; units are sorted by
`identity`. The v0 input-kind union is exactly `source`, `contract`,
`build_manifest`, and `lockfile`; unknown kinds reject, and each language
profile fixes which filenames are required in each kind. Rust inputs include
the selected `Cargo.toml`, `Cargo.lock`, contract files, and compiled Rust
sources. A target-repository
toolchain request is rejected and therefore never appears in `inputs`; the
frontend project's own `rust-toolchain.toml` is build provenance represented by
the effective toolchain distribution, component, commit, and executable
identities above, not a source-root-relative input. The Go migration
specification must similarly enumerate module/workspace files, contract files,
and every source file used by package loading; it may not fall back to the old
source-only manifest behavior.

For Rust, the compiled source-file set is the exact-match result of the
preflight module closure and rustc inventory. For Go, it is the exact package-
loader inventory defined by `GO_VIR_PROFILE_V0.md`. Both are cross-checked
against the normalized source root and their pre-compilation snapshot.
Synthetic or external source files reject unless they are a documented
compiler builtin covered by the recorded toolchain identity.

The manifest unit set must equal the VIR unit set exactly. `units[].identity`
equals the corresponding `VirUnit.id`: a canonical Go import path or canonical
Rust crate name. For Go, `name` is the declared package name and `kind` is
`package`; for Rust, `name` is the exact accepted Cargo package name and `kind`
is `lib`. Rust `selection.crate` equals the unit identity and
`selection.package` equals the unit name. Cargo's opaque package ID and raw
`cargo metadata` output are not embedded because they may contain absolute
paths. Cargo workspace manifests and inherited configuration are rejected in
Rust v0.

`input_set_hash` covers only the sorted canonical input entries:

```text
SHA256(
  "MPK-INPUT-SET-0.1" || 0x00 ||
  canonical_input_entry_list
)
```

The complete manifest is canonical JSON. Its `source_manifest_hash` is:

```text
SHA256(
  "MPK-SOURCE-MANIFEST-0.1" || 0x00 ||
  canonical_source_manifest_json_without_source_manifest_hash
)
```

This reuses the existing certificate source-manifest hash domain. The manifest
may be placed in the existing opaque certificate `SourceManifest` payload
without changing certificate v0 encoding. The frontend consumer, assembler,
and policy helper recompute `source_manifest_hash`. A source-free checker does
not parse that JSON or independently validate its internal hash; it preserves
the opaque payload bytes, which are already committed by the canonical whole-
certificate hash, and never interprets them for proof acceptance.

VC payloads contain `input_set_hash`, `source_ir_schema`, `source_ir_hash`,
`semantic_profile`, the canonical semantic parameters, and their self-checked
`vc_hash`, but not `source_manifest_hash`. This permits the final manifest to
attach `vc_hash` without creating a manifest/VC hash cycle. `source_gir_hash`
is removed rather than aliased.

The frontend-stage manifest omits `vc_hash` and hashes that form. Certificate
assembly must preserve every other manifest field byte-for-byte in canonical
form, attach only the final `vc_hash`, and recompute `source_manifest_hash`.
Before attachment, assembly recomputes the VC hash and requires the VC's
`input_set_hash`, `source_ir_schema`, `source_ir_hash`, semantic profile, and
semantic parameters to equal the frontend manifest and VIR. Changing any other
manifest field or attaching a mismatched VC is an internal error. The two
lifecycle stages have different hashes and are never compared as if they were
the same manifest payload.

For identical input bytes, semantic context, and compiler identity, VIR must be
host-independent. Source-manifest and evidence hashes are reproducible for the
same approved release registry and frontend/toolchain bundle identities.
Changing only the registry root while retaining the selected descriptors, or
changing only a frontend binary implementation under the same compiler and
semantic contracts, produces a distinct manifest hash by design but must not
change VIR. A selected compiler-identity change instead follows the explicit
upgrade and golden-regeneration gate.

## 15. CLI, policy, and evidence integration

### 15.1 Atomic migration and removal

The repository remains on its current Go/GIR interfaces until the shared VIR
path passes its migration gate. The cutover then lands atomically: producers,
consumers, fixtures, examples, CI, and user-facing ProofOps documentation move
together. The post-cutover release removes rather than aliases:

- `mpk.go2gir.cli.v0` and the `go2gir` executable;
- `mpk.gir.v0`, `mpk.gir.emit.v0`, the `MPK_GIR_V0` canonical binary wrapper,
  and the associated importer, `gir_emit`, `gir_hash`, and `gir-lowered`
  interfaces;
- `source_gir_hash` in VC, certificate-skeleton, policy, and fixture payloads;
- the unversioned GIR-bound VC document and
  `mpk.vc.cert_skeleton.v0` payload/parser;
- `--go2gir` and the Go-only policy runner;
- `mpk.go.source_manifest.v0`;
- `mpk.policy.scan.v0` and `mpk.policy.evidence.v0`;
- policy evidence's implicit/hard-coded `allowed_axiom_profiles` list in favor
  of one explicit strategy-compatible `axiom_profile`;
- `mpk.ai.explain.request.v0`, `mpk.ai.explanation.v0`, and the
  `mpk.evidence-explainer.v0` prompt template and `minimal-v0` redaction
  profile;
- the AI API v0 `POST /gir/import` route in favor of the v1
  `POST /vir/import` route.

AI API v1 retains the session, term, proof, and non-import VC operations whose
semantics do not change. `POST /vir/import` accepts only a canonical, validated
`mpk.vir.v0` module, and subsequent VC operations emit or consume `mpk.vc.v1`
artifacts using `source_ir_schema`/`source_ir_hash`. `POST /gir/import` is an
unknown route after cutover, not an alias.

Removal of policy v0 includes every typed downstream consumer, not only the
scan producer. `PolicyEvidenceReport`, Markdown rendering, ProofOps
integrations, and `mpk explain` move to evidence v1 in the same cutover; no v0
compatibility parser remains. The explainer's sanitized helper kinds change
from `go_source`/`gir` to `source`/`verification_ir`, so its credential-free
request carries the non-sensitive `source_language`, `semantic_profile`, and
canonical non-path semantic parameters needed to distinguish wrapping,
checked, and target-width behavior, plus the validated `strategy_profile`,
`checker_profile`, and `axiom_profile`. It continues to exclude raw package,
crate, function, and filesystem-path identities, compiler prose, rendered
diagnostics, and source spans; only stable diagnostic codes and counts may
survive sanitization. Its schema and prompt template are versioned as
`mpk.ai.explain.request.v1` and `mpk.evidence-explainer.v1`, and the mandatory
non-user-selectable redaction profile becomes `minimal-v1`, rather than
silently changing their v0 canonical payload. The `mpk.ai.explanation.v0`
output schema and its parser are replaced by `mpk.ai.explanation.v1` because
`source_evidence.schema` becomes `mpk.policy.evidence.v1` and the exact
prompt-template, redaction-profile, and sanitized-request references change to
v1. `mpk.ai.explanation.response.v0` remains the model-response schema because
its exact property-alias and explanation-text shape does not change. The Vertex
assistant design, root README, ProofOps documents, tests, and golden request
hashes are updated atomically.

`mpk.go.contract.v0` remains the Go source-side input because its source
semantics do not change in this migration; `go2vir` normalizes it into the
shared VIR contract model. If the migration audit discovers that a field or
semantic correction is actually required, the cutover gate stops for a new Go
contract version and design amendment rather than changing v0 in place.

After the atomic cutover, historical GIR JSON is not accepted by the
post-cutover `mpk-vc` importer. No
automatic converter is shipped as a production path. Checked-in generated Go
artifacts are regenerated, reviewed, and committed in the cutover change.
Only then do `GIR_V0.md`, `GO_SUBSET_V0.md`, and `AI_API_V0.md` become
historical records; before the cutover they remain the current release
specifications. `VIR_V0.md`, `GO_VIR_PROFILE_V0.md`, and `AI_API_V1.md` become
normative for the active source-program helper path in the same cutover.

### 15.2 Unified route

The policy CLI replaces the Go-only route with a generic frontend route:

```text
mpk policy scan <source-root>
  --language rust
  --semantic-profile mpk.rust.checked.v0
  --require-release-registry-id <release-registry-id>
  --require-release-registry-sha256 <release-registry-sha256>
  --frontend-bundle <registered-bundle-id>
  --toolchain-bundle <registered-toolchain-bundle-id>
  --target <target-id>
  --package <cargo-package>
  --function <function-id>
  --contract <contract.json> ...
  --json-out <scan.json>
```

`policy verify` uses the same source selection, frontend bundle, and toolchain
bundle rather than calling a hidden Go-only scan route:

```text
mpk policy verify <source-root>
  --language rust
  --semantic-profile mpk.rust.checked.v0
  --require-release-registry-id <release-registry-id>
  --require-release-registry-sha256 <release-registry-sha256>
  --frontend-bundle <registered-bundle-id>
  --toolchain-bundle <registered-toolchain-bundle-id>
  --target <target-id>
  --package <cargo-package>
  --function <function-id>
  --contract <contract.json> ...
  --strategy-profile payment-policy-rust-alpha
  --checker-profile <checker-profile>
  --axiom-profile mvp-theory
  --evidence-json <evidence.json>
  --evidence-md <evidence.md>
  [--strict]
  [--update-fixtures]
```

`--require-release-registry-id`, `--require-release-registry-sha256`,
`--frontend-bundle`, and `--toolchain-bundle` are mandatory and have no
defaults. The two registry options are equality assertions against the runner's
embedded ID/hash; they cannot load or select another registry. The bundle
registry tuples must match the requested language, semantic profile, and
target. The frontend descriptor supplies the complete helper set: Rust v0
requires exactly `rust2vir-driver`, while Go v0 requires none.

Missing, malformed, or unequal registry assertions are pre-launch policy CLI
configuration errors and produce no frontend response. By contrast, failure to
load and validate the registry whose ID/hash the runner embeds is the
artifact-free `frontend-error` defined in section 8.1.

Evidence-producing policy routes reject raw `--frontend`, `--frontend-helper`,
`--driver`, and toolchain-path flags. The runner resolves, snapshots, and hashes
the main/helper executables and complete toolchain as described in sections 8
and 13 before it launches the frontend.

`--axiom-profile` is also mandatory on `policy verify` and has no default. It
must match the exact strategy-registry tuple and is recorded independently from
the checker, strategy, and semantic profiles. The initial Go tuple retains
`zero-axiom`; the Rust tuple uses `mvp-theory`. An unknown or crossed value
rejects before frontend launch.

The toolchain root is not a user-selected policy flag. The registered
`--toolchain-bundle` ID selects the closed descriptor from section 8.1, while
the runner resolves its root internally and independently revalidates every
installed component.

`policy verify` passes this exact validated configuration to its internal scan,
consumes the returned VIR directly, and never reconstructs a frontend path or
falls back to `go2gir`. Both evidence reproduction recipes preserve the generic
`--language`, `--semantic-profile`, registry ID/hash assertions,
`--frontend-bundle`, `--toolchain-bundle`, target, package, function, and
complete contract-set values. The verify recipe additionally preserves
checker, strategy, axiom, strictness, and fixture-update options.

Policy evidence v1 replaces the v0 free-form shell command string with a closed
`reproduction_recipes` array. Each recipe contains `label`,
`working_directory_role = source_root`, and an exact UTF-8 `argv` array. The
array begins with `mpk`, `policy`, the route, and the literal positional source
root `.`, followed by that route's canonical arguments above. The scan recipe
omits verify-only checker, strategy, axiom, strictness, and fixture-update
options. Scan uses the fixed relative output `mpk-reproduction-scan.json`;
verify uses `mpk-reproduction-evidence.json` and
`mpk-reproduction-evidence.md`. These recipe-only names replace
caller-provided report paths rather than copying them; the normal safe-write
and tracked-fixture overwrite rules still apply, and
`--update-fixtures` is retained if and only if it was part of the verified
invocation. Contract arguments are sorted as described below, all other option
order is frozen by `POLICY_V1.md`, and every argument is one array element—there
is no shell parsing or quoting in the canonical evidence. Markdown renders the
array with one specification-frozen POSIX-shell quoting algorithm, independent
of the host, and states that it runs with the source root as the working
directory; consumers on other shells use the structured array directly. Thus a
recipe contains no retired `--go2gir`, unresolved executable/source-root
placeholder, caller output path, or other machine-local path.

The generic policy CLI accepts repeatable `--contract` options. It resolves the
complete normalized relative set against `source-root`, passes that exact set
to the frontend for immutable capture, and reuses the resulting scan in
`policy verify`. Reproduction recipes emit one `--contract` argument per
normalized relative contract path in canonical order. Caller option order
cannot change VIR, evidence, or recipe bytes.

Both source paths use `mpk.policy.scan.v1` and `mpk.policy.evidence.v1`. Their
shared top-level identity fields are `source_language` and the same exact
language-specific `selection` union used by the frontend protocol; the v0
Go-only `target.package_path`/`target.function_id` shape is removed. Cross-
artifact fields are named `source_ir_schema` and `source_ir_hash`, matching VC
v1; the frontend envelope's nested `ir.schema` and `ir.sha256` values must equal
them. Other shared names include `frontend`, `semantic_profile`, and
`semantic_parameters`. Helper-artifact kinds are `source`, `contract`,
`verification_ir`, `vc`, `ai_analysis`, and `ci_status`, never `go_source` or
`gir`. The schemas do not expose fields such as `go_version`,
`go2gir_sha256`, or `gir_sha256`.

Evidence v1 additionally records the validated `strategy_profile`,
`checker_profile`, and `axiom_profile`. Scan v1 does not claim checker, strategy,
or axiom selections that its route has not used.

Each policy `contract` helper entry distinguishes the raw manifest input SHA
from the normalized `contract_hash` repeated by VIR and call sites; reports do
not label one as the other.

`mpk.policy.evidence.v1` records the one validated `axiom_profile`; it does not
derive or silently broaden a policy-evidence allowlist. Package-level manifests
continue to express their separately governed checker profile and allowed axiom
profile set. The source-free package/release gate, not `policy verify`, checks
its active profiles against that manifest and the recomputed axiom report;
release orchestration requires both active selections to equal the checker and
axiom profiles recorded in evidence. `policy verify` neither reads nor
reproduces those package-manifest policy fields, and its explicit selections
cannot override them.

Evidence v1's `trusted_evidence.certificates` is exactly empty or the singleton
candidate program certificate with `id = program`; all generated declarations
for the VC and their needed checked zero-axiom foundation closure are assembled
into that one canonical byte sequence. Under
`mpk.program_certificate.alpha.v0`, its import and theory-certificate tables are
empty. A candidate row is not itself an acceptance claim. Both source-free
checker verdicts cover the same singleton ID, and only two accepted verdicts
can support `mpk_verified`. A deterministic rejection is
written as valid untrusted evidence and makes `policy verify` fail regardless
of strict mode; a checker crash or internal failure writes neither report.

Both policy v1 payloads' `release_registry` object repeats the validated
registry schema, ID, and hash from the source manifest and runner. The policy
CLI cannot select or override it; its mandatory registry flags only assert
equality with the MPK release's embedded selection. Evidence validation rejects
any mismatch before it interprets bundle IDs.

The policy `frontend` object repeats the registered bundle ID and exact
main/subordinate names, versions, and digests from the source manifest, never
their machine-local locator paths.

The policy `toolchain` object likewise repeats the registered bundle ID,
distribution digest, and complete component identities/digests. Neither object
can be reconstructed from a mutable executable search path.

Policy v1 names the two manifest lifecycle hashes explicitly.
`mpk.policy.scan.v1` carries `frontend_source_manifest_hash` for the validated
frontend-stage payload, which omits `vc_hash`. `mpk.policy.evidence.v1`
preserves that field and adds
`certificate_source_manifest_hash` plus `vc_hash`. The internal scan result
retains the validated canonical frontend-stage manifest bytes; verify derives
the allowed final manifest from those bytes, never from a hash alone,
recomputes both hashes, and requires the certificate's opaque source-manifest
payload to match it exactly. There is no ambiguous `source_manifest_hash` field
whose stage must be guessed.

The certificate-stage manifest is derived immediately after validated VC
generation even when proof search remains pending and no candidate certificate
is emitted. Every candidate that is emitted must attach those exact bytes, so a
proof-pending evidence report can still record the lifecycle hash without
pretending that a certificate exists.

The migrated Go path uses the same v1 schemas with `--language go`,
`--semantic-profile mpk.go.fixed.v0`, and registered `go2vir` frontend and Go
toolchain bundles. `mpk explain` accepts only a valid evidence v1 report after
cutover and preserves the existing rule that AI output is untrusted helper
analysis. Its recognized strategy allowlist contains
both `payment-policy-alpha` and `payment-policy-rust-alpha`; the sanitized v1
request preserves the validated `source_language`, `semantic_profile`, and
non-path semantic parameters while continuing to map unknown future strategy
profiles to the existing unrecognized value rather than inventing semantics.
Unknown v0 policy payloads reject after cutover.

The initial product strategy profile is distinct:

```text
payment-policy-rust-alpha
```

It may reuse language-neutral policy classification, but it does not inherit
Go-specific readiness text or source assumptions. The existing
`payment-policy-alpha` strategy migrates to evidence v1 for Go. Checker profile,
strategy profile, semantic profile, and axiom profile remain separate.

The v1 strategy registry binds exact tuples:
`payment-policy-alpha` pairs with Go, `mpk.go.fixed.v0`, and `zero-axiom`, while
`payment-policy-rust-alpha` pairs with Rust, `mpk.rust.checked.v0`, and
`mvp-theory`. The CLI rejects a known strategy used with the wrong
language/semantic/axiom profile before launch, and evidence/explainer validation
rejects the same mismatch. Only an actually unknown future strategy is
represented by the sanitized explainer's unrecognized value; a crossed known
tuple is not downgraded to that value.

### 15.3 Axiom policy

Rust v0 adds no `RustSemanticsAxiom` and does not reuse
`GoSemanticsAxiom`. Migrating Go to VIR does not rename or broaden that fixed
certificate category. `Std.Program.Base` aliases are zero-axiom. Bitvector and
runtime-safety theory hooks must use checked definitions or the existing
`BuiltinTheoryAxiom` mechanism backed by a checked theory-certificate path.

The Rust alpha axiom profile is `mvp-theory` with concrete approved identities;
the migrated Go alpha tuple retains `zero-axiom`. If a Rust-specific unchecked
semantic assumption becomes necessary, implementation stops pending a new
axiom-policy and certificate-format design; it must not be hidden as
`ExternalAxiom`.

For the current program-certificate alpha release, `mvp-theory` is only the
selected axiom allowlist: it accepts the required zero-axiom result but does not
activate theory-certificate or theory-proof support. Obligations that need
those deferred hooks remain proof-pending.

### 15.4 Program-certificate alpha compatibility resolution

`PROGRAM_CERTIFICATE_ALPHA_V0.md` freezes the implementable intersection of
Certificate v0 and the two unchanged source-free checkers for RUST-06-T03. The
assembler emits one all-or-nothing, self-contained root with empty import and
proof-node/theory tables and a zero-axiom report. It reconstructs the validated
grouped skeleton, lowers stored Boolean values to propositions through typed
equality, and lowers generated `True`, implication, and balanced conjunction
markers to the checked `Std.Eq` and `Std.Logic` interfaces. It may use only
complete structural proof terms stored directly in theorem declarations,
including reflexivity, exact hypotheses, earlier generated dependencies, and
the checked grouping constructors/eliminators.

Foundation source certificates, theory classifiers, solver results, and VC
status labels remain helper inputs until the complete root certificate checks.
If any member has no such structural term, no partial candidate is emitted and
all members remain proof-pending. Import resolution, theory-payload binding,
theory primitive registration, and reference-checker theory support are
deferred to a separately governed checker task; RUST-06-T03 does not change
Certificate v0 encoding or either checker.

## 16. VC generation changes

The existing WP, loop, and safety layers are refactored to consume VIR directly.
There is one serialized program model and no GIR adapter or parallel legacy
input boundary after cutover.

The replacement VC document schema is `mpk.vc.v1`; the corresponding theorem
declaration skeleton is `mpk.vc.cert_skeleton.v1`. Both carry
`source_ir_schema = mpk.vir.v0` and `source_ir_hash` and reject their v0/GIR
predecessors after cutover. Their discriminator field is exactly `schema`;
the old skeleton's `schema_version` spelling rejects rather than aliasing.

VC v1 contains its own `vc_hash`, computed as:

```text
SHA256(
  "MPK-VC-1.0" || 0x00 || canonical_vc_json_without_vc_hash
)
```

The certificate skeleton repeats `source_vc_schema = mpk.vc.v1` and
`source_vc_hash`, in addition to the source IR, input-set, profile, and semantic
parameters plus the verification-limit profile. The skeleton emitter and
certificate assembler recompute the VC hash and reject any repeated-field or
theorem-group mismatch; neither hashes a pretty-printed form or transport LF.
The final source manifest's `vc_hash` is this same value.

Required additions:

- profile-aware program type encoding;
- deterministic weakest-precondition generation for nested branches, joins,
  mutable locals, early returns, and acyclic CFGs;
- shared loop-cutpoint processing for cyclic Go CFGs, preserving existing
  invariant, exit, and optional decreases obligations while Rust rejects all
  cycles;
- profile validation that derives the exact required check set from operation
  and operand types and rejects missing or extra checks;
- preservation of Go wrapping arithmetic, signed division/remainder, shift,
  index, conversion, and loop behavior under `mpk.go.fixed.v0`;
- no-overflow predicates for signed and unsigned add, subtract, and multiply,
  plus signed negate, at widths 8, 16, 32, and 64;
- profile-required division/remainder checks, including Rust signed
  `MIN / -1` and `MIN % -1` representability;
- profile-required shift checks for both signed and unsigned count values;
- fixed-array bounds checks for the exact VIR index type, including
  target-width Rust `usize`;
- contract-based `CallStatic` WP and safety handling;
- exhaustive assignment of postcondition/call/loop members to the contract
  declaration and operation/callee-safety members to the panic-free declaration,
  with no duplicate or ungrouped obligation;
- policy member records bound to their containing checked declaration name and
  hash;
- source-language-neutral obligation IDs and diagnostic codes;
- `source_ir_schema`, `source_ir_hash`, input-set hash, semantic profile,
  semantic parameters, and `verification_limit_profile` in every emitted VC
  document;
- v1 VC and certificate-skeleton payloads with no `source_gir_hash` field.

Safety predicates must be propositions in checked MPK modules. Solver `sat` or
`unsat` answers are never accepted directly; any solver-assisted discharge uses
an independently checked theory certificate.

## 17. Deterministic diagnostics

Every rejection includes a stable code, normalized source span when available,
function ID, and concise reason. Code families are:

- `RUST_PREFLIGHT_*`: Cargo graph, lockfile, target, dependency, or config;
- `RUST_TOOLCHAIN_*`: compiler, driver, component, or commit mismatch;
- `RUST_SOURCE_*`: malformed manifest or Rust parse, name, and type errors;
- `RUST_SUBSET_*`: disallowed HIR source feature or type;
- `RUST_MIR_*`: unknown or inconsistent MIR form;
- `RUST_CONTRACT_*`: schema, resolution, or type failure;
- `RUST_SEMANTICS_*`: missing or inconsistent safety semantics;
- `RUST_LIMIT_*`: deterministic resource limit;
- `RUST_FRONTEND_*`: internal protocol or compiler failure.

Compiler diagnostic prose may be included only after deterministic
normalization: strip ANSI/control sequences, replace accepted source paths with
normalized source-root-relative paths, omit toolchain and temporary paths,
remove host-specific suggestions, and enforce the message-byte limit from
section 18. If a detail cannot be normalized without retaining an absolute or
host-specific value, it is omitted. Raw Cargo/rustc stderr never enters the
frontend JSON, canonical artifacts, evidence hashes, or golden fixtures.
Cargo/rustc rendered diagnostics, source snippets, macro-expansion text, child
command lines, and environment dumps are never eligible prose; only the
structured diagnostic code, primary span, and normalized top-level message are
considered. A public diagnostic span uses the source map's normalized input
path convention and zero-based half-open UTF-8 byte offsets. It may name only a
captured input allowed by the reporting phase; line and column renderings are
derived display data and never canonical fields.
Stable MPK codes, not rustc wording, drive tests and product behavior.

The shared runner and importer use language-neutral `FRONTEND_REGISTRY_*`,
`FRONTEND_PROTOCOL_*`, and `VIR_*` codes. `go2vir` retains Go-specific
source/subset code families while adopting the same status classes and exit
mapping as `rust2vir`.

## 18. Resource limits

Acceptance must not depend on wall-clock time. Rust v0 defines deterministic
input and IR limits, initially:

- at most 64 MiB of canonical release-registry JSON, 1,024 bundle descriptors,
  and 262,144 regular-file inventory entries across that registry;
- for the build-only `mpk.rust.build_inputs.v0` boundary, at most 256 MiB for
  the complete JCS+LF descriptor, 1,048,576 inventoried regular files, 8,192
  package records across the three parsed dependency graphs, 1 KiB per
  portable relative inventory path, 4 GiB per cache regular file, and 32 GiB
  for the checked sum of all declared and observed cache file bytes;
- at most 1 MiB for `Cargo.toml` and 4 MiB for `Cargo.lock`;
- at most 128 contract files, 1 MiB per contract, and 8 MiB total contract
  bytes;
- at most 64 `requires` plus `ensures` clauses per function, 1,024 contract
  expression nodes per function, 8,192 nodes across the call closure, and 32
  levels of contract-expression nesting;
- at most 256 compiled source files;
- at most 16 MiB total source bytes and 1 MiB per source file;
- at most 512 total snapshot input entries and 32 MiB total snapshotted input
  bytes across manifests, lockfile, contracts, and source;
- at most 1 KiB per normalized input path;
- at most 128 functions in the selected call closure;
- at most 1,024 reachable MIR blocks per function;
- at most 8,192 reachable MIR blocks across the call closure;
- at most 100,000 MIR statements per function;
- at most 250,000 MIR statements across the call closure;
- at most 256 fixed-array elements;
- at most 64 fields per struct;
- at most 16 levels of accepted aggregate-type nesting;
- at most 1,024 entries combined across normalized `rejected_features` and
  `diagnostics`, 4 KiB per message, and 2 MiB combined normalized message
  bytes;
- at most 64 MiB total Cargo/rustc child stdout across metadata and check and
  2 MiB total captured child stderr, neither of which is embedded verbatim;
- at most 4 MiB for the canonical `mpk.rust.driver.request.v0` request,
  including its normalized input inventory;
- at most 256 MiB for the private `mpk.rust.driver.v0` artifact, leaving bounded
  space beyond the maximum lowered program and source map for its required
  inventory and protocol metadata;
- at most 192 MiB of canonical VIR JSON, 32 MiB of canonical source-map JSON,
  and 4 MiB of canonical source-manifest JSON;
- at most 256 MiB of frontend stdout containing the one protocol JSON value
  plus its LF, and 2 MiB of captured frontend stderr.

Each serialized request, private output, and public frontend size limit counts
the entire transported JCS+LF byte sequence; the JSON portion therefore has at
most one byte less than the stated limit. Stream limits count every observed
byte. Hash preimages still exclude the transport LF as specified above.

The shared VC/policy stages additionally allow:

- at most 100,000 VC member obligations per function and 262,144 total;
- at most 4,096 path-specific assumptions per member;
- at most 8,192 expression nodes per member, 4,194,304 across one VC document,
  and 256 levels inside any member before balanced group conjunctions are
  added;
- at most 512 levels in any final grouped theorem type or generated proof after
  outer parameter binders, member-local binders, and balanced conjunctions are
  added;
- at most 256 MiB each for canonical VC and certificate-skeleton JSON;
- at most 512 MiB for a generated canonical certificate and 256 MiB each for
  policy evidence JSON and rendered Markdown.

Release-registry byte/count ceilings are enforced by the generic runner before
source capture or frontend launch. A breach is an artifact-free
`frontend-error` with `FRONTEND_REGISTRY_LIMIT`, not a Rust/Go source rejection.
File and aggregate byte ceilings are enforced while reading, before full parse
or JSON allocation. Deterministic structural input, source-profile, and IR
limits return `rejected` with `RUST_LIMIT_*`. Diagnostic count and byte ceilings
bound reporting rather than reclassifying the underlying result: after
normalization and canonical ordering, if entries would exceed either ceiling,
the frontend reserves space for one marker, retains the longest preceding
prefix that fits both ceilings, and appends a fixed
`RUST_LIMIT_DIAGNOSTICS_TRUNCATED` entry with the omitted count. It preserves
the original `ir-lowered`, `rejected`, `source-error`, or `frontend-error`
status. Per-message overflow is normalized to the bounded prefix plus a fixed
truncation marker before that ordering step. The combined budget visits the
independently sorted `rejected_features` array first and `diagnostics` second
in protocol field order; the truncation marker is emitted as the final
`diagnostics` entry.

A response/output ceiling enforced by the outer consumer is `frontend-error`
with `FRONTEND_PROTOCOL_LIMIT`; an operational process kill or compiler crash
is also `frontend-error`. None can be converted into an accepted or
unsupported-language verdict. `RUST_SUBSET_V0.md` and
`FRONTEND_PROTOCOL_V0.md` freeze the exact counting, diagnostic-retention, and
limit-precedence rules.

A Cargo/rustc child-output ceiling is enforced inside `rust2vir` and, while the
frontend remains able to respond, yields an artifact-free `frontend-error`
with `RUST_FRONTEND_CHILD_OUTPUT_LIMIT`. The outer runner independently bounds
the frontend protocol stream and uses `FRONTEND_PROTOCOL_LIMIT`. This keeps a
compiler flood distinct from an oversized or malformed frontend response.

`VIR_V0.md` also defines shared envelope, nesting, identifier, unit, function,
block, and instruction limits that accommodate the migrated Go corpus. Each
language profile may define stricter deterministic limits, but the limit-set
version is fixed by that profile specification and recorded in the source
manifest. A frontend implementation cannot tighten or relax it without a new
registered limit-profile ID, and it cannot vary by host.

VC and policy limits are enforced by streaming counters before constructing the
complete tree or writing an output. A breach is a deterministic artifact-free
`VC_LIMIT_*` or `POLICY_LIMIT_*` helper failure after any successful frontend
result; it never rewrites `ir-lowered` as a source rejection, emits a partial
certificate/evidence report, or becomes proof-pending. Successful VC and policy
v1 payloads record `verification_limit_profile = mpk.verify.limits.v0`, which
denotes the exact downstream limits above and is selected by the VC/checker
profile registry rather than a user flag. It is hashed with the VC and evidence
and cannot vary by host or local configuration.

## 19. Test strategy

### 19.1 Positive source corpus

The minimum corpus contains:

1. boolean identity and negation, plus `&&`/`||` fixtures whose right operand
   has a path-guarded runtime-safety or call obligation;
2. signed and unsigned `Max` branch functions;
3. checked addition with a precondition sufficient to prove no overflow;
4. the canonical minimum literal `-128_i8` lowering to one `Const` without a
   safety check, paired with a nonconstant signed negation whose precondition
   proves `integer_no_overflow(neg)`;
5. signed division with the profile-required nonzero condition and, for Rust,
   a `MIN / -1` representability condition;
6. left and right shifts with narrower and wider signed/unsigned count types and
   each profile's required count conditions;
7. fixed-array read with a proved bounds condition;
8. simple struct construction, field selection, and a whole-value struct move;
9. early returns;
10. paired acyclic two-function contracted calls across ordinary modules using
    accepted explicit same-crate paths, covering inherited/private and bare
    `pub` helper visibility;
11. `usize` indexing on every release-tested target width;
12. an ordinary multi-file module closure with an unrelated `.rs` file that is
    neither read, snapshotted, nor listed in the source manifest.

Every positive fixture has golden frontend JSON, VIR, VC, certificate, axiom
report, fast-kernel verdict, and reference-checker verdict.

### 19.2 Negative and adversarial corpus

At minimum, fixtures must deterministically reject:

- reference, raw pointer, `unsafe`, and FFI use;
- heap allocation, vector, slice, and string use;
- trait, generic, method, closure, and function-pointer calls;
- `extern crate`, `use`, import alias, re-export, restricted visibility, and
  non-same-crate path use;
- async, loop, recursion, match, enum, and tuple use;
- float, 128-bit integer, and cast use;
- an out-of-range typed integer literal as `source-error`;
- static state, explicit panic, assert macro, and drop type use;
- build script, external dependency, proc macro, macro expansion, `cfg`, and
  lint-level attribute use;
- target-repository `rust-toolchain*` files, a nondefault/effective non-library
  crate type, and an unapproved lint-control rustc argument;
- missing, ambiguous, cyclic, path-attributed, and root-escaping module edges,
  nonportable/reserved or case-colliding input paths, plus a simulated
  rustc/preflight source-inventory disagreement;
- mutable-parameter patterns, field/index mutation, a non-`usize` primitive
  array index, and a projected or partial move;
- malformed or unresolved contracts and a `CallStatic` callee-contract hash
  mismatch;
- mixed preflight/source/subset/contract failures proving fixed phase precedence
  and exclusion of later-phase diagnostics;
- missing target, unsupported pointer width, stale lockfile, and compiler commit
  mismatch, plus exit-2/pre-launch handling for missing, unknown, or
  language-mismatched semantic profiles and crossed strategy/axiom profiles;
- evidence checker or axiom profiles that differ from the release gate's active
  profiles or are not permitted by the package manifest;
- unknown MIR statement, rvalue, projection, terminator, assertion kind, and
  changed checked-operation pattern;
- non-regular source inputs, symlink/reparse-point inputs, or source files
  escaping the root;
- oversized Cargo manifest, lockfile, contract bytes/counts/expression depth,
  source set, aggregate MIR, diagnostic set, VIR, source map, source manifest,
  Cargo/rustc child stdout and stderr, and frontend protocol stdout/stderr at
  their exact boundaries, including deterministic diagnostic truncation
  without status reclassification;
- oversized/noncanonical build-input descriptor bytes, inventory/graph/path/
  per-file/aggregate-cache limit breaches, declared-size overflow or mismatch,
  and any attempt to mount or execute cache content before those checks finish;
- downstream VC member/assumption/node/depth limits, VC/skeleton JSON,
  generated certificate, and policy JSON/Markdown at their exact boundaries,
  with no partial evidence or source-status reclassification;
- missing required safety checks and extra noncanonical safety checks;
- a missing, duplicated, or wrongly partitioned theorem-group member, any
  missing, reversed, or extra declaration dependency edge, and policy evidence
  that marks one conjunct checked without its containing declaration;
- a missing/unregistered frontend-bundle descriptor, main or helper bytes that
  differ from it, duplicate or unknown helper names inside a descriptor, a raw
  executable-path flag on a policy route, an omitted/unknown/incompatible
  frontend or toolchain bundle ID, a driver digest mismatch, a missing or
  mismatched release toolchain descriptor/component, an attempted executable
  outside that bundle, and a subordinate-binary manifest that differs from the
  launcher's snapshotted bundle;
- a missing, oversized, noncanonical, unknown-field, duplicate-tuple, or
  hash-mismatched release registry, a runner whose embedded registry ID/hash
  differs, an omitted or wrong policy CLI registry assertion, and any
  source-manifest or policy registry-identity mismatch;
- a v0 free-form reproduction command, a missing or duplicate scan/verify
  recipe, a wrong working-directory role, a positional source root other than
  `.`, noncanonical argv/contract order, caller-selected output paths, or an
  absolute path in any recipe argument;
- missing, mutable, oversized, noncanonical, duplicate-key, or identity-
  mismatched `mpk.rust.driver.request.v0` requests, plus missing, partial,
  duplicate, oversized, noncanonical, or identity-mismatched
  `mpk.rust.driver.v0` artifacts;
- status/exit disagreement, missing or truncated JSON on any non-usage exit,
  noncanonical JSON or extra stdout, any partial artifact on a non-success
  response, a main-frontend digest mismatch, repeated-identity mismatch,
  incorrect source selection or unit mapping, an invalid source-map
  reference/range/hash, mismatched VC/VIR/input-set or verification-limit fields,
  a final manifest mutation beyond adding `vc_hash`, a skeleton
  `source_vc_hash` mismatch, and an incorrect VIR/source-manifest hash in the
  frontend response or VC self-hash downstream.

### 19.3 Translation confidence

Because the frontend is untrusted, tests do not change the trust boundary, but
they reduce translation risk. The suite includes:

- two clean-run byte-for-byte determinism checks;
- clean-checkout tests with isolated Cargo home and offline mode;
- hostile ambient Cargo, rustup, compiler, locale, proxy, and credential
  variables proving the constructed child environment and output are
  unchanged;
- a small VIR interpreter differential corpus comparing accepted Rust function
  results and panic behavior over exhaustive small-width or generated inputs;
- compiler-upgrade snapshots requiring explicit review;
- migrated Go fixtures proving that accepted/rejected source behavior,
  contracts, loop obligations, and runtime semantics match the reviewed
  pre-cutover baseline even though bytes and hashes change;
- cross-language VIR fixtures showing deliberate semantic differences such as
  wrapping versus checked overflow and Go versus Rust shift bounds;
- handwritten VIR total-operation vectors for zero divisors, signed
  `MIN / -1`, and over-width shifts, separate from source safety acceptance;
- accepted-but-unproved Rust fixtures with insufficient overflow, divisor,
  shift, index, or callee preconditions, proving the frontend still returns
  `ir-lowered`, the required safety VCs remain present, non-strict evidence is
  proof-pending, and strict policy verification fails rather than reporting a
  source rejection;
- public frontend, private driver-request/output, VIR, source-map, and contract
  parser fuzzing;
- both-checker agreement for every emitted certificate;
- assertions that canonical artifacts contain no absolute workspace or temp
  paths, including normalized compiler diagnostics;
- policy scan/verify integration tests proving they use the same snapshotted
  frontend and toolchain bundles, preserve distinct frontend/certificate
  manifest hashes, and emit generic structured reproduction recipes;
- a multi-function call fixture proving reordered repeatable `--contract`
  options produce identical VIR/evidence bytes and canonically ordered
  reproduction-recipe arguments, with callee-first and contract-before-panic-free
  declaration dependencies;
- a source-dead direct-call fixture proving its HIR-closure callee and contract
  remain in VIR while no absent `CallStatic` dependency is invented, paired
  with a source-dead recursive cycle that still rejects;
- paired contract sidecars differing only in JSON whitespace, proving equal
  normalized contract/VIR hashes but distinct raw-input/source-manifest hashes.

### 19.4 Breaking-migration gates

Before removing GIR, the migration suite must:

- lower every current positive Go corpus entry through `go2vir` and verify the
  regenerated certificate with both source-free checkers;
- preserve every current negative Go rejection unless a separately reviewed
  source-profile change says otherwise;
- compare old and new Go obligation kinds and theorem intent with a checked-in
  migration report, allowing identifier and foundational-module renames but no
  unexplained semantic loss;
- test Go loops, conversions, runtime checks, contracts, policy classification,
  and all payment-policy examples on the shared VIR path;
- prove `mpk.gir.v0`, `mpk.gir.emit.v0`, the `MPK_GIR_V0` wrapper,
  `mpk.go2gir.cli.v0`, `gir-lowered`, policy v0 payloads, and retired CLI flags
  reject deterministically after cutover;
- verify `mpk explain` accepts evidence v1, rejects evidence v0, and produces
  `mpk.ai.explanation.v1` plus deterministic v1 sanitized/dry-run request
  fixtures, while retaining only `mpk.ai.explanation.response.v0` for the
  unchanged model-response shape and using `minimal-v1`; fixtures use only the
  generic `source` and `verification_ir` helper kinds, the correct source
  language, semantic profile/parameters, checker and axiom profiles, both
  recognized Go/Rust strategy profiles, and no raw source-selection identity,
  compiler prose, source span, or sentinel source text; crossed known
  language/semantic/axiom/strategy tuples must reject rather than map to
  unrecognized;
- verify both `mpk policy scan` and `mpk policy verify` reject `--go2gir`, use
  the registry-pinned `--frontend-bundle`/`--toolchain-bundle` route, require
  exact release-registry ID/hash assertions, reject raw
  frontend/helper/toolchain paths, and reproduce the same
  language/profile/selection/bundle configuration from a canonical structured
  argv recipe using source-root-relative inputs and outputs, without a
  machine-local path, caller output destination, or unresolved placeholder;
- search production code, examples, CI, and active user documentation for
  obsolete interfaces. Historical specifications and migration reports form
  one exact-file allowlist. Scanner metadata/implementation and focused
  positive/negative self-test fixtures form two separate, disjoint exact-file
  exclusion classes; a checked-in fixture manifest must enumerate regular
  files, not directories, and the gate must reject missing, unknown, symlinked,
  overlapping, or directory-wide exclusions before it tests the fixtures and
  scans the active tree. The obsolete set includes `go2gir`, `Go2Gir`,
  `GO2GIR`, `mpk.go2gir.cli.v0`,
  `mpk.go.source_manifest.v0`, `mpk.gir.emit.v0`, `MPK_GIR_V0`, `gir_emit`,
  `gir-lowered`,
  `source_gir_hash`, `mpk.policy.scan.v0`, `mpk.policy.evidence.v0`,
  `mpk.vc.cert_skeleton.v0`, `go_source`, `GoSource`,
  `PolicyScanTarget`, `PolicyEvidenceTarget`,
  `PolicyHelperArtifactKind::Gir`, `SanitizedArtifactKind::GoSource`,
  `SanitizedArtifactKind::Gir`, the policy-evidence
  `allowed_axiom_profiles` field, `gir_hash`, `gir_sha256`, `go2gir_sha256`,
  `mpk.ai.explain.request.v0`, `mpk.ai.explanation.v0`,
  `mpk.evidence-explainer.v0`, `minimal-v0`,
  the v0 `reproduction_commands`/`PolicyEvidenceReproductionCommand` string
  model, the AI API `POST /gir/import` route, and GIR-only paths.

## 20. Implementation sequence

### VIR-00: Freeze the replacement and migration contracts

Deliverables:

- `VIR_V0.md` covering the complete migrated Go subset and Rust v0 needs;
- `FRONTEND_PROTOCOL_V0.md`, `RELEASE_BUNDLES_V0.md`, `SOURCE_MAP_V0.md`, and
  `SOURCE_MANIFEST_V0.md` for both frontends;
- exact non-success payload, binary-bundle identity, source-selection/unit
  linkage, canonical release-registry root and equality assertions, release
  toolchain-bundle identity, two-stage manifest, diagnostic-normalization, and
  resource-limit contracts in those shared specifications;
- `VC_V1.md` for the VC/certificate-skeleton schemas, self-hash, group/member
  mapping, and deterministic limits;
- `POLICY_V1.md` for policy scan/evidence schemas, registered-bundle routing,
  manifest lifecycle, explicit checker/axiom profiles, package/release profile
  equality, and structured reproduction recipes;
- `AI_EXPLAIN_V1.md` for the sanitized request, prompt/redaction references,
  explanation output, strategy tuple validation, and unchanged provider-response
  v0 boundary;
- `AI_API_V1.md` for the helper API's `POST /vir/import` boundary and its
  language-neutral VC operations, with no v0 GIR adapter or change to the
  certificate-check acceptance boundary;
- `RUST_SUBSET_V0.md` derived from sections 7 through 11 and the Rust-specific
  diagnostic and resource-limit rules in sections 17 and 18;
- `GO_VIR_PROFILE_V0.md`, derived from the accepted behavior of the current
  pre-cutover `GO_SUBSET_V0.md`, with the normative profile ID
  `mpk.go.fixed.v0`;
- normative `mpk.go.fixed.v0` and `mpk.rust.checked.v0` operation/check
  matrices;
- registration and vectors for `MPK-VIR-0.1`, `MPK-SOURCE-MAP-0.1`,
  `MPK-CONTRACT-0.1`, `MPK-BUNDLE-REGISTRY-0.1`,
  `MPK-BUNDLE-CONTENT-0.1`, `MPK-INPUT-SET-0.1`,
  `MPK-RUST-BUILD-INPUTS-0.1`,
  `MPK-RUST-DRIVER-REQUEST-0.1`, `MPK-RUST-SOURCE-INVENTORY-0.1`,
  `MPK-RUST-DRIVER-PAYLOAD-0.1`, `MPK-VC-1.0`, and the existing opaque
  `MPK-SOURCE-MANIFEST-0.1` certificate domain;
- a complete inventory of GIR schemas, fields, flags, files, fixtures, and
  downstream documentation and typed policy/evidence consumers, including
  `mpk explain`, to remove or regenerate;
- governance-approved language-neutral amendments or successor documentation
  for the certificate source-manifest example and trust-boundary frontend/IR
  terminology, confirming that certificate encoding, trusted evidence, and
  axiom categories do not change.

Exit gate: every currently accepted Go operation and every proposed Rust
operation has one value semantics, exact required checks, VIR representation,
contract rule, and rejection rule.

### VIR-01: Build the shared IR and checked foundations

Deliverables:

- VIR and source-map data models, validators, canonical encoders, hashes, and
  parser fuzz targets;
- `Std.Program.Base` certificate and type-map fixtures replacing active
  `Std.Go.Base` references;
- profile-aware type and expression encoders;
- common acyclic WP, Go loop-cutpoint, call, and runtime-safety infrastructure;
- streaming VC/group resource counters and balanced canonical theorem grouping;
- overflow, division/remainder, shift, and index predicates with checked theory
  support and axiom review;
- handcrafted Go and Rust VIR vectors for every semantic difference.

Exit gate: every check can be emitted and discharged through a checked path,
both profiles reject missing/extra checks, and no Rust-specific semantic axiom
exists.

### GO-VIR-02: Migrate Go and perform the atomic cutover

Deliverables:

- `go2vir` using the generic frontend protocol and source manifest;
- launcher-validated release registry, Go frontend/toolchain bundles, and the
  frozen Go loader environment;
- direct Go SSA-to-VIR/source-map lowering with `mpk.go.fixed.v0` and no
  serialized adapter;
- generic `policy scan` and `policy verify` runner plus policy scan/evidence v1
  for the existing Go product path, including explicit registered bundle and
  axiom-profile selection, package/release checker/axiom-profile cross-checks,
  and generic structured reproduction recipes;
- evidence v1 report/rendering plus the migrated `mpk explain` validator,
  language-neutral redaction model, v1 sanitized request/prompt and explanation
  output, unchanged provider-response v0 schema, tests, and Vertex assistant
  documentation;
- the AI helper API v1 `POST /vir/import` route and removal of the active
  `/gir/import` route;
- regenerated Go, VC, certificate, policy, AI-explanation, example, and release
  fixtures;
- a reviewed old/new semantic migration report;
- updated CLI help, CI, developer docs, ProofOps docs, templates, and examples;
- removal of the production GIR importer, `go2gir`, `Std.Go.Base` VC mapping,
  `source_gir_hash`, the GIR-bound VC/skeleton formats, old frontend/policy
  schemas, and retired CLI flags.

Exit gate: all Go positive and negative gates pass on VIR, both checkers accept
the regenerated certificate corpus, all intentional hash changes are recorded,
and targeted searches find no obsolete interface in active code or user docs.

### RUST-03: Build the pinned compiler frontend skeleton

Deliverables:

- isolated `rust-tools/rust2vir` project with the `rust2vir_internal` library
  and exactly two installed binaries, exact toolchain pin, closed build/test
  materialization and locked dependency-source inventory, plus the internal
  offline launcher used by every frontend build, format-check, lint, test, and
  run gate;
- content-addressed `mpk.rust.build_inputs.v0` build-only descriptor and ignored
  materialization with reproducibly built cargo-fuzz, complete provenance, and
  no installation path;
- smaller evidence-execution toolchain inventory with the pinned compiler/LLVM
  files, target libraries, Linux interpreter/native-library closure, private
  runtime layout, provenance, and redistribution notices;
- tracked, non-installable pre-registration Rust bundle candidate followed by
  atomic first registration and candidate removal once the skeleton is final;
- Rust release frontend-bundle registration for the main and driver digests,
  with a reviewed new release-registry root hash;
- release toolchain-bundle descriptor resolution and pre-launch digest checks;
- `mpk.rust.targets.v0` with pinned i686/x86_64 Linux standard libraries;
- `RUST_DRIVER_PROTOCOL_V0.md` freezing the exact
  `mpk.rust.driver.request.v0` request and `mpk.rust.driver.v0` output schemas,
  all three private hash domains, limits, and cross-process identity checks;
- Rust population of `mpk.frontend.cli.v0` and the frontend-stage
  `mpk.source_manifest.v0`, with canonical VIR/source-map/manifest hash and
  repeated-identity validation;
- Cargo preflight with explicit snapshot manifest paths, target toolchain-file
  rejection, exact default library crate type, and sanitized metadata/check
  invocation;
- validating pre-expansion file loader and source/AST gate;
- fixed lint/attribute policy and deterministic pre-parse input limits;
- HIR subset validator;
- Rust contract parser, typed selected-function resolution, and normalized
  contract attachment;
- MIR extraction and deterministic diagnostics;
- constant/copy/local-assignment, Boolean, comparison, branch, and early-return
  lowering with validated source-map coverage.

Exit gate: simple single-function positive fixtures emit deterministic generic
frontend envelopes containing schema-valid VIR, source maps, frontend-stage
manifests, and normalized contracts; all preflight negative fixtures reject
without executing user build code.

### RUST-04: Add arithmetic and runtime safety

Deliverables:

- checked MIR pattern recognizers;
- arithmetic, bitwise, division/remainder, shift, and index lowering;
- golden fixtures distinguishing accepted minimum-value literals from checked
  nonconstant signed negation;
- safety-check completeness validation;
- runtime-safety VC generation and negative pattern fixtures.

Exit gate: changing or removing any compiler-inserted safety assertion causes
deterministic rejection or a failed golden test, never silent approximation.

### RUST-05: Add aggregates, contracts, and calls

Deliverables:

- fixed-array construction plus by-value struct construction, field selection,
  and whole-value moves;
- aggregate and closure-wide contract-set resolution, including duplicate and
  unused sidecar rejection plus callee contract-hash binding;
- call-closure discovery and cycle rejection;
- contract-based static-call WP;
- topological theorem and certificate dependencies.

Exit gate: the complete positive corpus generates stable property and safety
VCs.

### RUST-06: Integrate policy scan and evidence

Deliverables:

- Rust routing through the already-migrated generic frontend runner for both
  `policy scan` and `policy verify`, using the same snapshotted main/driver
  bundle and registered toolchain bundle;
- Rust population of the shared policy scan/evidence v1 schemas;
- `payment-policy-rust-alpha` strategy metadata and language-neutral reports;
- Rust example package/release policy whose checker and allowed axiom profiles
  admit the exact selections recorded by evidence;
- generic source-manifest payload attached to certificate artifacts.

Exit gate: a Rust payment-policy example passes both source-free checkers and
produces evidence that separates trusted proof evidence from helper artifacts.

### RUST-07: Harden and release-gate

Deliverables:

- positive and negative corpus completion;
- public frontend, private driver-request/output, VIR, source-map, and contract
  parser fuzzing;
- differential interpreter tests;
- diagnostic normalization and every boundary/aggregate resource-limit test;
- deterministic clean-machine CI that validates the pinned build/test closure
  separately while exposing only registered bundles to evidence routes;
- compiler-upgrade procedure and release report integration.

Exit gate: all Rust gates, all migrated Go gates, checker agreement, path
sanitization, artifact determinism, and obsolete-interface searches pass from a
clean checkout.

## 21. Expected file and module impact

New paths:

```text
develop/specs/
  VIR_V0.md
  VC_V1.md
  FRONTEND_PROTOCOL_V0.md
  RELEASE_BUNDLES_V0.md
  SOURCE_MAP_V0.md
  SOURCE_MANIFEST_V0.md
  GO_VIR_PROFILE_V0.md
  RUST_SUBSET_V0.md
  RUST_DRIVER_PROTOCOL_V0.md
  POLICY_V1.md
  AI_EXPLAIN_V1.md
  AI_API_V1.md

develop/specs/vectors/
  manifest.json

develop/migrations/
  gir-to-vir-inventory.md
  gir-to-vir-obsolete-terms.txt
  gir-to-vir-search-fixtures/manifest.json
  go-gir-semantic-baseline.json

go-tools/go2vir/
  go.mod
  main.go
  {loader,features,lower,contract,emit,source_map,manifest}.go

rust-tools/rust2vir/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  src/lib.rs
  src/bin/rust2vir.rs
  src/bin/rust2vir-driver.rs
  src/{cli,path,preflight,source_capture,module_closure,snapshot,metadata_request}.rs
  src/{environment,sandbox,cargo_metadata,cargo_check}.rs
  src/{driver_protocol,driver_process,source_gate,file_loader}.rs
  src/{rustc_driver,session,mir_access,hir_check,mir_lower}.rs
  src/{limits,stable_id,type_lower,const_lower}.rs
  src/{mir_arithmetic,mir_projection,mir_aggregate,mir_call,call_closure}.rs
  src/{contract,contract_typecheck,diagnostics,emit,source_map,manifest}.rs
  fuzz/Cargo.toml
  fuzz/Cargo.lock
  fuzz/fuzz_targets/{driver_protocol,rust_contract}.rs

crates/mpk-vc/src/
  vir.rs
  vir_canonical.rs
  semantic_profile.rs
  program_wp.rs
  call_wp.rs

proofs/program/base/
fixtures/vir-go/
fixtures/rust-basic/
examples/rust-payment-policy/

release/bundles/
  bundle-registry.json

release/build-inputs/rust/
  build-inputs.json

scripts/
  build-release-bundles.sh
  check-release-bundles.sh
  check-no-active-gir.sh
  check-spec-vectors.py
  run-rust2vir-toolchain.sh
```

`release/bundles/candidates/rust` is a tracked migration-only staging path, not
an installed release path. It exists from RUST-03-T01 through RUST-03-T11 and
is atomically removed by the first Rust registration in RUST-03-T12.
`release/build-inputs/rust/build-inputs.json` is the tracked reviewed build-only
descriptor. Its content cache at
`release/build-input-cache/rust/<build_inputs_sha256>` is ignored and is neither
a likely touched file nor an installation input.

`go-tools/go2vir` replaces the existing `go-tools/go2gir` directory rather than
coexisting with it in the post-cutover tree.

Expected modifications:

- root `Cargo.toml` workspace exclusion for the isolated compiler frontend;
- `mpk-vc` type, expression, WP, safety, obligation, and export modules;
- `mpk-cli` frontend runner, routing, package/release profile cross-checks,
  policy scan, evidence, and report modules;
- `mpk-cli` AI explainer validation/redaction models and its CLI integration
  tests;
- `mpk-api` import/VC route models and tests for the v1 VIR boundary;
- Go frontend module identity and direct VIR emitter;
- every Go/VC/policy generated fixture and hash-bearing example;
- development specs, trust-boundary examples, templates, user documentation
  including the Vertex assistant design, CI scripts, and release reporting.

Removed at cutover:

- `go-tools/go2gir` and production GIR import/emission code;
- active `Std.Go.Base` VC mappings once `Std.Program.Base` migration passes;
- GIR-only fixtures and old frontend/policy protocol parsers;
- the active AI API v0 `/gir/import` route;
- the unversioned GIR-bound VC serializer and
  `mpk.vc.cert_skeleton.v0` parser/fixtures;
- `--go2gir`, `source_gir_hash`, and other public GIR-specific fields.

No planned modification:

- core calculus or definitional equality;
- canonical certificate v0 term/proof encoding;
- independent reference checker semantics;
- certificate acceptance rules or the set of trusted proof inputs;
- axiom category encoding.

## 22. Alternatives considered

### Parse Rust with `syn`

Rejected. Syntax parsing does not provide compiler-resolved types, macro
expansion, method/operator resolution, target-specific integer widths, or the
actual compiler control flow and inserted panic checks.

### Parse textual `--emit=mir` output

Rejected as the primary design. Although rustc can emit human-readable MIR, the
text is not a versioned structured interchange format. A custom parser would
lose typed compiler APIs and create another brittle semantic layer. It may be
used only as a debug fixture.

### Lower LLVM IR

Rejected. LLVM IR loses important Rust-level function identity, contracts,
panic classification, and source types, and introduces layout and optimization
semantics outside the desired subset.

### Trust rustc or `rust2vir`

Rejected. Moving a large evolving compiler into the proof-acceptance boundary
would contradict the existing certificate-first design. Toolchain pinning is
for reproducibility and traceability, not proof trust.

### Rewrite `mpk.gir.v0` in place

Rejected even though backward compatibility is not required. Reusing the same
schema identifier for different fields or semantics would make historical
hashes and diagnostics ambiguous. A breaking migration still requires a new
schema and hash domain.

### Keep GIR and VIR as permanent parallel inputs

Rejected. It preserves two importers, validators, hash vocabularies, policy
routes, foundations, and regression matrices, while the user has explicitly
removed the compatibility requirement. A temporary dual implementation is
allowed only inside the migration branch before the atomic cutover.

### Define a language-neutral `mpk.gir.v1`

Rejected in favor of VIR. The representation could be technically equivalent,
but GIR is already documented as Go Verification IR. VIR gives the shared
contract an unambiguous name without changing the historical meaning of GIR.

### Emit current GIR v0 directly from Rust

Rejected. GIR v0 lacks hashed semantic parameters and complete checked-operation
metadata, and its standard type names and surrounding protocols are Go-specific.
Extending it under the same identifier has the versioning problem above.

### Disable Rust overflow checks and reuse wrapping GIR arithmetic

Rejected. This would make accepted semantics depend on build configuration and
would not match the desired panic-free Rust policy profile. Rust v0 instead
fixes overflow checks on and proves their safety conditions.

## 23. Risks and mitigations

| Risk | Mitigation |
|---|---|
| rustc internal API changes | Exact toolchain pin, isolated project, compiler-upgrade corpus, no automatic upgrades. |
| MIR shape changes silently | Exact pattern matching, deny unknown variants, golden MIR/VIR fixtures. |
| A stale or malformed private driver request/artifact is reused | Fixed read-only request and result paths, exclusive partial creation plus atomic no-replace publication, versioned JCS schemas, domain-separated request fingerprint, size bounds, and request/binary/toolchain/source identity checks. |
| A local installation rewrites the release registry | Runner-embedded registry ID/hash, bounded canonical registry validation before launch, no override path, and registry identity repeated in manifests and evidence. |
| A caller substitutes an unregistered frontend binary | Evidence routes accept only a registered bundle ID; resolve paths internally and require exact pre-launch main/helper digests, while retaining the untrusted-frontend label. |
| An analyzed package build script or proc macro executes code | Metadata preflight, dependency ban, and build-script/proc-macro rejection before target compilation. |
| A frontend dependency build script, proc macro, linker, or SDK reads ambient state or changes release bytes | Registry/checksum/source inventories, an exact build-time executable allowlist, fixed linker/sysroot, a credential-free OS sandbox, and byte comparison of two separately empty clean release builds. |
| An ignored build cache or frontend checkout changes after validation | No-follow open and stream-copy every input into a fresh invocation tree while hashing, execute only sealed private copies, and recheck the current source inventory before release publication. |
| User, rustup, lint, or Cargo config changes semantics | Launcher-validated release toolchain bundle, sanitized metadata/check environment, isolated Cargo home, target `rust-toolchain*` and project-config rejection, fixed lint policy, explicit snapshot manifest, flags, and target. |
| Frontend omits a panic condition | MIR assertion consumption checks plus VIR profile safety-completeness validation and differential tests. |
| Rust source claim is overstated | Keep compiler/frontend/VIR untrusted and label source linkage as traceability only. |
| New Rust axioms weaken release policy | Require zero new Rust semantic category; stop for governance review if checked foundations are insufficient. |
| Go behavior regresses during the breaking migration | Baseline every positive/negative fixture, compare obligation intent, run Go/VIR differential tests, and require a reviewed migration report. |
| Temporary dual paths drift before cutover | No released dual mode; one atomic gate removes GIR producers and consumers together. |
| `policy verify`, `mpk-api`, ProofOps, `mpk explain`, or CI consumes removed v0 fields | Inventory every downstream typed model, route, reproduction recipe, field, enum, flag, prompt input, and golden hash; update them in the cutover change and reject old schemas deterministically. |
| Scan-stage and certificate-stage manifest hashes are confused | Distinct policy v1 field names, deterministic final-manifest derivation, and exact certificate payload comparison. |
| Target-dependent semantics reuse a hash | Hash target and pointer width in VIR semantic parameters and exercise multi-target fixtures. |
| Compiler or parser resource exhaustion | Enforce per-input and aggregate byte/count limits before full parsing, bound protocol output, and ensure operational failures never become acceptance. |
| Compiler diagnostics leak host paths or vary by host | Emit stable MPK codes, normalize bounded optional detail, omit unnormalizable prose, and never embed raw Cargo/rustc stderr. |
| Native loader/library drift or incomplete redistribution metadata | Freeze the exact execution-host ABI, interpreter/shared-library closure, component provenance, hashes, and required notices; run only inside the private runtime root and block release on any missing file, host fallback, or notice. |

## 24. Completion criteria

The unified migration and Rust v0 are complete only when all of the following
hold:

- the normative Go/Rust semantic profiles, Rust subset, VIR, release-bundle,
  source-map, source-manifest, frontend, Rust-driver, VC, policy v1, AI API v1,
  and AI explanation v1 specs are frozen;
- Go source, contracts, loops, runtime checks, policy classification, and
  examples use the sole VIR path with reviewed regenerated artifacts;
- no production parser, CLI flag, schema, fixture, CI command, or user guide
  consumes or emits GIR v0, `go2gir`, the GIR-bound VC/skeleton formats, policy
  v0, the AI API v0 `/gir/import` route, or `source_gir_hash`;
- policy reports model the shared `source_language`/`selection` union;
  `mpk explain` consumes evidence v1 only, validates that union, redacts raw
  selection identity, preserves the non-sensitive semantic, strategy, checker,
  and axiom-profile context plus generic helper kinds, emits
  `mpk.ai.explanation.v1`, and rejects v0 without an adapter;
- `policy scan` and `policy verify` use the same validated generic frontend,
  release registry, and frontend/toolchain bundle configuration, repeat the
  registry identity in evidence, require matching registry assertions, and
  produce no Go-only or machine-local-path-dependent reproduction recipe;
- `policy verify` records explicit checker and strategy-compatible axiom
  profiles and never injects an unreported default allowlist; release
  orchestration requires the same active selections, and the source-free
  package gate proves that the package manifest permits them;
- every accepted Go/SSA and Rust/HIR/MIR form has an explicit semantics and
  test;
- both `mpk.rust.targets.v0` target corpora pass with the pinned component
  digests and target-width-specific golden hashes;
- unsupported or unknown forms fail closed with deterministic codes;
- the release registry and bundle inventories, compiler toolchain, target,
  flags, source, manifests, lockfile, contracts, frontend binaries, VIR, source
  map, and VCs are hash-pinned in traceability metadata;
- property, call-site, overflow, division/remainder, shift, and index obligations
  are generated where required;
- every obligation belongs to exactly one canonical contract or panic-free
  declaration, and policy member claims bind to the accepted whole declaration;
- the Rust example produces a canonical `.mpcert` accepted by both checkers;
- the recomputed axiom report satisfies its declared policy;
- source manifests bind the exact source selection, VIR unit set, and source-map
  hash, and final certificate assembly changes only `vc_hash` plus the derived
  manifest hash;
- artifacts and normalized diagnostics are deterministic and contain no
  absolute local, toolchain, or temporary paths;
- frontend, VC, skeleton, generated-certificate, and policy resource boundaries
  pass exact below/at/above-limit tests without partial trusted evidence;
- the migrated Go positive/negative suite preserves reviewed semantics; all
  expected hash changes are recorded rather than suppressed;
- certificate v0 encoding, source-free checking, checker agreement, and axiom
  category encoding remain unchanged;
- documentation never presents rustc, `rust2vir`, VIR, or a successful build as
  trusted proof evidence.

## 25. Post-Rust multi-language handoff

This design deliberately validates the shared architecture with exactly two
source languages before starting the multi-language program. No multi-language
design, feasibility, specification, or production milestone runs during this
Go/VIR/Rust program. It must not add an unused plugin framework, future-
language branch, or widened frozen parser.

After `RUST-07-T05`, `MLANG-00` may use the completed Go/Rust path for semantic
comparison and compiler-API feasibility work. `MLANG-01` then designs the
strict successor extension mechanism and freezes the C# specification package.
Production frontends then proceed serially: C#, Java, Dart, TypeScript, and
Python. Each language has its own semantic profile, pinned registered frontend/
toolchain bundle, complete conformance vectors, and both-checker release gate.

The first added language may require successor versions of VIR, frontend,
selection, manifest, release, policy, or evidence contracts because current v0
unions are closed over Go and Rust. If a shared serialized contract changes,
all active Go/Rust producers and consumers migrate atomically; no permanent
dual public IR input is introduced. Certificate v0, the trust boundary, and
axiom-category encoding remain unchanged.

Mixed-language VIR and source-language FFI remain out of scope. Independently
verified languages compose only through checked hash-pinned certificate
imports. Exact gates and language-specific risks are owned by
`06_multilanguage_frontend_design.md` and its todo.

## 26. Primary references

- [RFC 8785: JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785)
- [SMT-LIB 2.7 FixedSizeBitVectors theory](https://smt-lib.org/theories-FixedSizeBitVectors.shtml)
- [SMT-LIB standard logic extensions](https://smt-lib.org/logics-all.shtml)
- [rustc_driver and rustc_interface](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html)
- [External rustc drivers](https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html)
- [rustc_driver callback stages](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver_impl/trait.Callbacks.html)
- [rustc `FileLoader`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/trait.FileLoader.html)
- [MIR overview](https://rustc-dev-guide.rust-lang.org/mir/index.html)
- [MIR queries and passes](https://rustc-dev-guide.rust-lang.org/mir/passes.html)
- [MIR assertion kinds](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.AssertKind.html)
- [rustc command-line output, including MIR](https://doc.rust-lang.org/nightly/rustc/command-line-arguments.html)
- [Rust integer overflow rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow)
- [Rust numeric types](https://doc.rust-lang.org/reference/types/numeric.html)
- [Rust type layout and target-sized integers](https://doc.rust-lang.org/reference/type-layout.html)
- [Rust conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Rust array indexing behavior](https://doc.rust-lang.org/reference/expressions/array-expr.html#array-and-array-index-expressions)
- [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- [Cargo source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html)
- [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- [Rust procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [rustc linker codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html#linker)
- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
- [Cargo support for external tools](https://doc.rust-lang.org/cargo/reference/external-tools.html)
- [Cargo environment and workspace-wrapper behavior](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- [Cargo wrapper coverage for the initial `rustc -vV` probe](https://github.com/rust-lang/cargo/pull/13659)
- [rustup toolchain components](https://rust-lang.github.io/rustup/concepts/components.html)
- [rustup overrides and toolchain files](https://rust-lang.github.io/rustup/overrides.html)
