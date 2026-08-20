# Rust Verification Frontend and Unified VIR Migration Design

Status: proposed breaking migration for implementation. The current release
continues to follow the frozen GIR v0, Go subset v0, certificate v0, and
trust-boundary v0 specifications until the cutover described here is complete.

This design never reinterprets the `mpk.gir.v0` schema identifier. It introduces
`mpk.vir.v0` as the only post-cutover source-program IR, migrates the Go path to
that schema, and then retires GIR v0 and its Go-specific frontend, VC, hash, and
policy interfaces. Certificate v0 encoding and the mathematical trust boundary
remain unchanged, although their Go-specific documentation examples will be
amended to describe VIR and multiple source languages.

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
3. Replace Go-specific IR, VC, source-manifest, frontend, and policy field names
   with one versioned language-neutral contract used by both frontends.
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

## 2. Context and current constraints

The current implementation already separates its source frontend from proof
acceptance, but several helper-layer interfaces remain Go-specific.

| Current component | Go-specific constraint | Replacement design |
|---|---|---|
| `develop/specs/GIR_V0.md` | GIR means Go Verification IR and is frozen. | Keep its historical meaning, introduce VIR, migrate all producers and consumers, then retire GIR. |
| `develop/specs/GO_SUBSET_V0.md` | Its fail-closed boundary is phrased in terms of GIR emission. | Preserve its accepted/rejected behavior in a new `GO_VIR_PROFILE_V0.md`; mark the GIR-bound document historical after cutover. |
| `crates/mpk-vc/src/type_encode.rs` | Types encode to `Std.Go.Base.*`. | Replace it with a `Std.Program.Base.*` encoder used by both semantic profiles. |
| `crates/mpk-cli/src/policy_scan.rs` | The runner accepts only `mpk.go2gir.cli.v0` and `--go2gir`. | Replace the route with the generic frontend protocol and policy schemas v1. |
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
  `--go2gir`, and policy v0 interfaces at the atomic cutover;
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
- runtime backward compatibility for GIR v0, `go2gir`, old VC JSON, or policy
  scan/evidence v0 after the cutover;
- changing certificate v0 binary encoding or adding source artifacts to the
  trusted checker inputs;
- cross-language calls inside one VIR module in v0.

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
from hash-pinned Go or Rust helper artifacts under a recorded frontend,
semantic profile, target, and toolchain. Because both frontends and VIR are
untrusted, this is not a mathematical proof that the theorem exactly matches
the source program.

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
                + generic source manifest
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

The driver installs a custom rustc `FileLoader`. On every Rust source read, the
loader lexes/parses and validates the file before returning its bytes to the
compiler. The crate-root parsing callback performs the same validation on the
root AST. This covers submodules parsed during expansion while avoiding a scan
of unrelated `.rs` files that are not in the selected crate.

Expansion-affecting source-gate rules are crate-wide for every file compiled
into the selected library. Type, purity, and control-flow subset checks are
applied to the selected function dependency closure. This asymmetry is
intentional: an item removed by `cfg` cannot safely be assigned to a dependency
closure after expansion.

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

The pinned toolchain must include at least:

- an exact dated nightly toolchain;
- `rustc-dev`;
- `rust-src` if required by the selected integration;
- the target standard library for every release-tested target.

`rust2vir-driver` must refuse to run if the invoked rustc commit differs from
the commit embedded at frontend build time. A toolchain update is an explicit
reviewed change that regenerates every MIR/VIR golden fixture.

The ordinary root workspace remains buildable with its current stable Rust
toolchain. `rust-tools/rust2vir` is not a root workspace member.

### 8.2 Cargo preflight

Rust v0 requires `source-root` to be one self-contained Cargo package root with
one selected library target. A structural TOML/filesystem preflight first
rejects Cargo workspaces, workspace-inherited fields, symlinks, `.cargo`
configuration, build scripts, and dependencies. It then creates a private
analysis snapshot containing the exact accepted manifests, lockfile, contract
files, and Rust source bytes.

From that snapshot, before invoking rustc, `rust2vir` runs:

```text
cargo metadata --format-version 1 --no-deps --locked --offline \
  --no-default-features
```

The preflight selects exactly one library target and rejects:

- a missing or stale `Cargo.lock`;
- a package-name mismatch or more than one matching library target;
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
- a manifest outside the normalized source root;
- a non-UTF-8, nonportable, or case-fold-colliding relative input path;
- any symlink in the copied input set or any path that resolves outside the
  source root.

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

It uses an isolated empty `CARGO_HOME`, offline mode, and these semantic flags:

```text
-C overflow-checks=yes
-C panic=abort
-C debug-assertions=no
-C opt-level=0
-Z mir-opt-level=0
--remap-path-prefix=<analysis-snapshot-root>=.
```

Cargo's working directory is a controlled launcher root, not the original
repository. Every ancestor visible to Cargo's hierarchical configuration
search must be part of the sandbox and contain no `.cargo` configuration. If
the platform cannot establish that condition, verification stops with
`frontend-error`; it does not fall back to an unsandboxed Cargo invocation.

The compile command is equivalent to:

```text
cargo check --lib --package <package> --target <target> \
  --locked --offline --no-default-features
```

The selected target triple is mandatory for verification. It has no implicit
host default in release evidence. Rust v0 accepts only built-in target triples
from a versioned MPK release allowlist and only when their pointer width is 32
or 64 bits. Custom target JSON and target paths reject. Adding a target requires
its pinned standard-library component and the complete target corpus.

In hosted or CI execution, Cargo and the compiler run in an OS sandbox that can
read only the pinned toolchain and analysis snapshot, can write only its
dedicated temporary target/output directories, has no network, credentials, or
user-home access, and has explicit process and memory limits. Sandbox failure
is `frontend-error`; it is never a reason to retry with broader access.

The driver is installed as `RUSTC_WORKSPACE_WRAPPER` and filters on the selected
primary package, library crate name, crate type, and manifest identity. It must
emit exactly one raw frontend artifact. Zero or multiple matching artifacts are
a deterministic frontend error. Cargo treats a single-package project as a
workspace for this wrapper mechanism.

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

### 9.1 Crate and item scope

Accepted:

- a selected Cargo library target using Rust edition 2021;
- ordinary file and inline modules under the source root;
- free functions with explicit parameter types and exactly one explicit,
  accepted, non-unit return type;
- primitive `const` items used by accepted functions whose initializers are a
  boolean literal, an in-range integer literal, or a leading-unary-minus signed
  integer literal whose final typed value is in range;
- named structs used by value and containing only accepted fields;
- private helper functions in the selected function's static call closure.

Crate, module, function, parameter, local, constant, struct, and field names in
the selected dependency closure must use ASCII Rust identifiers matching
`[A-Za-z_][A-Za-z0-9_]*` and must not be the discard name `_`. Raw identifiers
and non-ASCII identifiers reject so contract IDs and canonical JSON do not need
a second identifier-normalization rule in v0.

Rejected:

- binaries as verification targets;
- `static` and `static mut` items;
- `impl`, trait, associated, extern, async, const, unsafe, or variadic
  functions;
- generic parameters or `where` clauses;
- custom linkage, FFI, exported ABI, or `no_mangle` behavior;
- semantic attributes such as `cfg`, `cfg_attr`, `path`, `repr`, derive, test,
  target features, and inline assembly;
- expanded user macros in the selected call closure.

The crate-level `no_std` attribute and documentation and lint-level attributes
may be accepted from an explicit allowlist. An unknown attribute rejects.

### 9.2 Types

Accepted source types and VIR encodings:

| Rust type | VIR type |
|---|---|
| `bool` | `{"kind":"bool"}` |
| `i8`, `i16`, `i32`, `i64` | signed BV8/BV16/BV32/BV64 |
| `u8`, `u16`, `u32`, `u64` | unsigned BV8/BV16/BV32/BV64 |
| `isize`, `usize` | signed/unsigned BV32 or BV64 from the explicit target |
| `[T; N]` | fixed array, when `T` is accepted and `N` is a literal or accepted primitive constant within limits |
| named struct | nominal VIR struct with fields in declaration order |

Rejected types include:

- `i128`, `u128`, `f32`, `f64`, `char`, strings, and `str`;
- references, raw pointers, slices, vectors, boxes, and trait objects;
- tuples, enums, unions, function types, closures, and never type;
- generic or dynamically sized types;
- any type for which rustc reports that drop glue is required.

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

- simple, initialized, non-shadowing identifier `let` bindings;
- local mutable-variable assignment with plain `=`;
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
  indexing;
- struct construction with every field explicitly initialized and direct field
  read;
- simple local values and accepted constants;
- direct calls to accepted same-crate free functions.

Rejected:

- overloaded operators or comparisons;
- integer `as` casts in v0;
- wrapping, checked, overflowing, or saturating library methods in v0;
- user method calls and indexing through `Index`/`IndexMut`;
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

For each call, the VC generator must:

1. prove the callee's `requires` clauses in the current path state;
2. introduce a fresh result value;
3. assume the callee's checked `ensures` clauses for subsequent obligations;
4. depend on the callee's checked panic-free theorem;
5. reject signature, type, semantic-profile, or contract-hash mismatch.

Each accepted function exposes two logical evidence groups under stable names:

- `<function>.contract`, covering its checked postconditions; and
- `<function>.panic_free`, covering every checked runtime-safety obligation
  under the same preconditions.

An implementation may encode a group as one conjunction theorem or as a
canonical hash-pinned declaration bundle. Calls must reference the exact checked
declaration hashes from both groups; report text or a successful callee scan is
not sufficient.

Functions and certificates are emitted in topological call order. Dynamic
dispatch, function values, recursion, and external calls reject.

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

- `not`, variadic `and`, and variadic `or` over booleans;
- `eq` and `not_eq` over two values of the exact same accepted type;
- `signed_lt`, `signed_le`, `signed_gt`, `signed_ge`, and their `unsigned_*`
  counterparts over matching bitvectors;
- total logical `bv_add`, `bv_sub`, `bv_mul`, `bv_and`, `bv_or`, `bv_xor`,
  `bv_neg`, `bv_not`, `bv_shl`, `bv_ashr`, and `bv_lshr` operations.

Aggregate values may appear only as exact-typed variables or results in
`eq`/`not_eq`; field selection, indexing, aggregate literals, conversion,
division, and remainder are not contract operators in Rust v0. Contract
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
  operands
  safety_checks

VirBlock:
  label
  parameters
  instructions
  terminator
```

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

For Go, a nonnegative shift count greater than or equal to the value width is
valid and uses the total bitvector shift result. For Rust, the same count is a
panic condition. For signed `MIN / -1` and `MIN % -1`, the total bitvector value
operation supplies Go's result while the Rust profile requires the
representability check. These differences are therefore visible in VIR and do
not rely on frontend identity or build mode.

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
- reachable blocks to `bb0`, `bb1`, ... using entry-first traversal and defined
  successor ordering.

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
  "source_map": {},
  "rejected_features": [],
  "diagnostics": []
}
```

`selection` has an exact language-specific shape: Rust records Cargo package,
crate, library-target kind, and canonical function ID; Go records canonical
import path and function ID. It contains identities, not filesystem paths, and
unknown or inapplicable fields reject.

Statuses and exits:

| Status | Exit | Meaning |
|---|---:|---|
| `ir-lowered` | 0 | Complete VIR and manifest emitted. |
| `rejected` | 3 | Valid input used a feature outside the selected profile. |
| `source-error` | 4 | The language loader or compiler rejected malformed or ill-typed source input. |
| no JSON | 2 | CLI usage error. |
| `frontend-error` | 1 | Compiler crash, protocol failure, toolchain mismatch, or internal error. |

The protocol consumer treats the complete response as untrusted. It accepts
only an exact status/exit pairing and one size-bounded JSON value, rejects
duplicate keys, unknown fields, and trailing stdout, recomputes
the canonical VIR and source-manifest hashes, and verifies all repeated
language, profile, semantic-parameter, compilation-target, source-selection,
and artifact-hash fields for equality wherever they recur. The selected
function must resolve uniquely inside the returned VIR. The launcher snapshots
and hashes each configured frontend or driver binary before starting it; a
response cannot redirect execution to another helper path. Any mismatch is
`frontend-error`, not `rejected` and never a partially ready scan.

For `rejected` and `source-error`, no partial VIR, VIR hash, or source manifest
is emitted. Rejected features and normalized source diagnostics are sorted by
normalized path, start position, code, and message.

Suggested Rust invocation:

```text
rust2vir lower <source-root>
  --manifest-path <relative-Cargo.toml>
  --package <package-name>
  --target <target-triple>
  --function <canonical-function-id>
  --contract <relative-contract-path> ...
```

The migrated Go frontend uses the same selection model:

```text
go2vir lower <source-root>
  --package <import-path>
  --target <goos>/<goarch>
  --function <canonical-function-id>
  --contract <relative-contract-path> ...
```

All filesystem arguments are resolved against `source-root`. Output paths are
relative and normalized after symlink resolution.

## 14. Generic source manifest

Schema: `mpk.source_manifest.v0`.

Every frontend emits the same top-level shape:

```text
schema
source_language
semantic_profile
semantic_parameters
limit_profile
toolchain:
  distribution_sha256
  components[]:
    name
    release
    commit_hash?
    binary_sha256?
frontend:
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
  pointer_width?
  language_configuration
inputs[]:
  kind
  normalized_path
  sha256
input_set_hash
vir_hash
vc_hash, when attached to a certificate
source_manifest_hash
```

The manifest's `source_language`, `semantic_profile`, `semantic_parameters`,
target, and `vir_hash` must exactly match VIR and the frontend request. Rust's
toolchain components include rustc, Cargo, and LLVM identities; Go records the
Go toolchain identity. `binary_sha256` is required for every directly invoked
toolchain executable; a non-executable bundled component uses its exact release
and commit identity plus the approved toolchain-distribution digest defined by
the profile specification. A subordinate compiler driver is listed as another
binary rather than as a Rust-only top-level field. `language_configuration`
records the normalized effective compiler flags and, for Rust, the complete
sorted rustc `cfg` set.

Each input entry has a versioned `kind`, a normalized relative path, and a
SHA-256 digest. Rust inputs include the selected `Cargo.toml`, `Cargo.lock`,
contract files, applicable toolchain request file, and compiled Rust sources.
The Go migration specification must similarly enumerate module/workspace
files, contract files, and every source file used by package loading; it may not
fall back to the old source-only manifest behavior.

The compiled source-file set comes from the frontend's compiler or package
loader inventory and is cross-checked against the normalized source root and
the pre-compilation snapshot. Synthetic or external source files reject unless
they are a documented compiler builtin covered by the recorded toolchain
identity.

Each unit entry uses a canonical Go import path or Rust package/crate identity
and, when applicable, a separate normalized manifest-relative path. Cargo's
opaque package ID and raw `cargo metadata` output are not embedded because they
may contain absolute paths. Cargo workspace manifests and inherited
configuration are rejected in Rust v0.

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
without changing certificate v0 encoding. The checker preserves and hashes it
as opaque traceability data and never interprets it for acceptance.

VC payloads contain `input_set_hash`, `source_ir_schema`, `source_ir_hash`,
`semantic_profile`, and the canonical semantic parameters, but not
`source_manifest_hash`. This permits the final manifest to attach `vc_hash`
without creating a manifest/VC hash cycle. `source_gir_hash` is removed rather
than aliased.

The frontend-stage manifest omits `vc_hash` and hashes that form. Certificate
assembly attaches the final `vc_hash` and recomputes `source_manifest_hash`;
the two lifecycle stages are never compared as if they were the same manifest
payload.

For identical input bytes, semantic context, and compiler identity, VIR must be
host-independent. Source-manifest and evidence hashes are reproducible for the
same approved frontend binaries; different frontend binaries have distinct
recorded manifest hashes by design and must not change VIR when they implement
the same schema and semantics.

## 15. CLI, policy, and evidence integration

### 15.1 Atomic migration and removal

The repository remains on its current Go/GIR interfaces until the shared VIR
path passes its migration gate. The cutover then lands atomically: producers,
consumers, fixtures, examples, CI, and user-facing ProofOps documentation move
together. The post-cutover release removes rather than aliases:

- `mpk.go2gir.cli.v0` and the `go2gir` executable;
- `mpk.gir.v0`, its importer, canonical binary wrapper, and `gir_hash` fields;
- `source_gir_hash` in VC, certificate-skeleton, policy, and fixture payloads;
- `--go2gir` and the Go-only policy runner;
- `mpk.go.source_manifest.v0`;
- `mpk.policy.scan.v0` and `mpk.policy.evidence.v0`.

`mpk.go.contract.v0` may remain a Go source-side input because contracts are
language-specific before lowering, but `go2vir` normalizes it into the shared
VIR contract model. Any necessary schema correction uses a new Go contract
version; it is not hidden behind the old identifier.

Historical GIR JSON is not accepted by the post-cutover `mpk-vc` importer. No
automatic converter is shipped as a production path. Checked-in generated Go
artifacts are regenerated, reviewed, and committed in the cutover change.
`GIR_V0.md` and `GO_SUBSET_V0.md` remain historical records; `VIR_V0.md` and
`GO_VIR_PROFILE_V0.md` become normative for the active Go path.

### 15.2 Unified route

The policy CLI replaces the Go-only route with a generic frontend route:

```text
mpk policy scan <source-root>
  --language rust
  --frontend <rust2vir>
  --target <target-id>
  --package <cargo-package>
  --function <function-id>
  --contract <contract.json>
  --json-out <scan.json>
```

Both source paths use `mpk.policy.scan.v1` and `mpk.policy.evidence.v1`. Their
source and helper-artifact sections use generic names such as `frontend`, `language`,
`semantic_profile`, `semantic_parameters`, `ir_schema`, and `ir_sha256`; they
do not expose fields such as `go_version`, `go2gir_sha256`, or `gir_sha256`.
The migrated Go path uses the same v1 schemas with `--language go`, `go2vir`,
and `mpk.go.fixed.v0`. Unknown v0 policy payloads reject after cutover.

The initial product strategy profile is distinct:

```text
payment-policy-rust-alpha
```

It may reuse language-neutral policy classification, but it does not inherit
Go-specific readiness text or source assumptions. The existing
`payment-policy-alpha` strategy migrates to evidence v1 for Go. Checker profile,
strategy profile, source-language profile, and axiom profile remain separate.

### 15.3 Axiom policy

Rust v0 adds no `RustSemanticsAxiom` and does not reuse
`GoSemanticsAxiom`. Migrating Go to VIR does not rename or broaden that fixed
certificate category. `Std.Program.Base` aliases are zero-axiom. Bitvector and
runtime-safety theory hooks must use checked definitions or the existing
`BuiltinTheoryAxiom` mechanism backed by a checked theory-certificate path.

The preferred alpha axiom profile is `mvp-theory` with concrete approved
identities. If a Rust-specific unchecked semantic assumption becomes necessary,
implementation stops pending a new axiom-policy and certificate-format design;
it must not be hidden as `ExternalAxiom`.

## 16. VC generation changes

The existing WP, loop, and safety layers are refactored to consume VIR directly.
There is one serialized program model and no GIR adapter or parallel legacy
input boundary after cutover.

The replacement VC document schema is `mpk.vc.v1`; the corresponding theorem
declaration skeleton is `mpk.vc.cert_skeleton.v1`. Both carry
`source_ir_schema = mpk.vir.v0` and `source_ir_hash` and reject their v0/GIR
predecessors after cutover.

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
- source-language-neutral obligation IDs and diagnostic codes;
- `source_ir_schema`, `source_ir_hash`, input-set hash, semantic profile, and
  semantic parameters in every emitted VC document;
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

Compiler diagnostic prose may be included as untrusted detail, but stable MPK
codes, not rustc wording, drive tests and product behavior.

The shared runner and importer use language-neutral `FRONTEND_PROTOCOL_*` and
`VIR_*` codes. `go2vir` retains Go-specific source/subset code families while
adopting the same status classes and exit mapping as `rust2vir`.

## 18. Resource limits

Acceptance must not depend on wall-clock time. Rust v0 defines deterministic
input and IR limits, initially:

- at most 256 compiled source files;
- at most 16 MiB total source bytes and 1 MiB per source file;
- at most 128 functions in the selected call closure;
- at most 1,024 reachable MIR blocks per function;
- at most 100,000 MIR statements per function;
- at most 256 fixed-array elements;
- at most 64 fields per struct;
- at most 16 levels of accepted aggregate-type nesting.

Exceeding a limit returns `rejected` with `RUST_LIMIT_*`. An operational process
kill or compiler crash returns `frontend-error` and can never be converted into
an accepted or unsupported-language verdict.

`VIR_V0.md` also defines shared envelope, nesting, identifier, unit, function,
block, and instruction limits that accommodate the migrated Go corpus. Each
frontend may impose stricter deterministic profile limits, but the limit-set
version is fixed by the profile specification, recorded in the source manifest,
and cannot vary by host.

## 19. Test strategy

### 19.1 Positive source corpus

The minimum corpus contains:

1. boolean identity and negation;
2. signed and unsigned `Max` branch functions;
3. checked addition with a precondition sufficient to prove no overflow;
4. signed division with the profile-required nonzero condition and, for Rust,
   a `MIN / -1` representability condition;
5. left and right shifts with each profile's required count conditions;
6. fixed-array read with a proved bounds condition;
7. simple struct construction, field selection, and a whole-value struct move;
8. early returns;
9. an acyclic two-function contracted call;
10. `usize` indexing on every release-tested target width.

Every positive fixture has golden frontend JSON, VIR, VC, certificate, axiom
report, fast-kernel verdict, and reference-checker verdict.

### 19.2 Negative source corpus

At minimum, fixtures must deterministically reject:

- reference, raw pointer, `unsafe`, and FFI use;
- heap allocation, vector, slice, and string use;
- trait, generic, method, closure, and function-pointer calls;
- async, loop, recursion, match, enum, and tuple use;
- float, 128-bit integer, and cast use;
- static state, explicit panic, assert macro, and drop type use;
- build script, external dependency, proc macro, macro expansion, and `cfg`;
- field/index mutation and a projected or partial move;
- malformed or unresolved contracts;
- missing target, unsupported pointer width, stale lockfile, and compiler commit
  mismatch;
- unknown MIR statement, rvalue, projection, terminator, assertion kind, and
  changed checked-operation pattern;
- source files or symlinks escaping the root;
- missing required safety checks and extra noncanonical safety checks;
- status/exit disagreement, repeated-identity mismatch, and incorrect VIR or
  source-manifest hashes in the frontend response.

### 19.3 Translation confidence

Because the frontend is untrusted, tests do not change the trust boundary, but
they reduce translation risk. The suite includes:

- two clean-run byte-for-byte determinism checks;
- clean-checkout tests with isolated Cargo home and offline mode;
- a small VIR interpreter differential corpus comparing accepted Rust function
  results and panic behavior over exhaustive small-width or generated inputs;
- compiler-upgrade snapshots requiring explicit review;
- migrated Go fixtures proving that accepted/rejected source behavior,
  contracts, loop obligations, and runtime semantics match the reviewed
  pre-cutover baseline even though bytes and hashes change;
- cross-language VIR fixtures showing deliberate semantic differences such as
  wrapping versus checked overflow and Go versus Rust shift bounds;
- VIR and contract parser fuzzing;
- both-checker agreement for every emitted certificate;
- assertions that canonical artifacts contain no absolute workspace or temp
  paths.

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
- prove `mpk.gir.v0`, `mpk.go2gir.cli.v0`, policy v0 payloads, and retired CLI
  flags reject deterministically after cutover;
- search production code, examples, CI, and user documentation for obsolete
  `go2gir`, `source_gir_hash`, and GIR-only paths.

## 20. Implementation sequence

### VIR-00: Freeze the replacement and migration contracts

Deliverables:

- `VIR_V0.md` covering the complete migrated Go subset and Rust v0 needs;
- `FRONTEND_PROTOCOL_V0.md` and `SOURCE_MANIFEST_V0.md` for both frontends;
- v1 VC, certificate-skeleton, policy scan, and policy evidence schemas using
  `source_ir_hash` and semantic parameters;
- `RUST_SUBSET_V0.md` derived from sections 8 through 11;
- `GO_VIR_PROFILE_V0.md`, derived from the accepted behavior of the historical
  `GO_SUBSET_V0.md`, with the normative profile ID `mpk.go.fixed.v0`;
- normative `mpk.go.fixed.v0` and `mpk.rust.checked.v0` operation/check
  matrices;
- registration and vectors for `MPK-VIR-0.1`, `MPK-INPUT-SET-0.1`, and the
  existing opaque `MPK-SOURCE-MANIFEST-0.1` certificate domain;
- a complete inventory of GIR schemas, fields, flags, files, fixtures, and
  downstream documentation to remove or regenerate;
- governance-approved language-neutral amendments or successor documentation
  for the certificate source-manifest example and trust-boundary frontend/IR
  terminology, confirming that certificate encoding, trusted evidence, and
  axiom categories do not change.

Exit gate: every currently accepted Go operation and every proposed Rust
operation has one value semantics, exact required checks, VIR representation,
contract rule, and rejection rule.

### VIR-01: Build the shared IR and checked foundations

Deliverables:

- VIR data model, validator, canonical encoder, hash, and parser fuzz target;
- `Std.Program.Base` certificate and type-map fixtures replacing active
  `Std.Go.Base` references;
- profile-aware type and expression encoders;
- common acyclic WP, Go loop-cutpoint, call, and runtime-safety infrastructure;
- overflow, division/remainder, shift, and index predicates with checked theory
  support and axiom review;
- handcrafted Go and Rust VIR vectors for every semantic difference.

Exit gate: every check can be emitted and discharged through a checked path,
both profiles reject missing/extra checks, and no Rust-specific semantic axiom
exists.

### GO-VIR-02: Migrate Go and perform the atomic cutover

Deliverables:

- `go2vir` using the generic frontend protocol and source manifest;
- direct Go SSA-to-VIR lowering with `mpk.go.fixed.v0` and no serialized
  adapter;
- generic policy runner plus policy scan/evidence v1 for the existing Go
  product path;
- regenerated Go, VC, certificate, policy, example, and release fixtures;
- a reviewed old/new semantic migration report;
- updated CLI help, CI, developer docs, ProofOps docs, templates, and examples;
- removal of the production GIR importer, `go2gir`, `Std.Go.Base` VC mapping,
  `source_gir_hash`, old frontend/policy schemas, and retired CLI flags.

Exit gate: all Go positive and negative gates pass on VIR, both checkers accept
the regenerated certificate corpus, all intentional hash changes are recorded,
and targeted searches find no obsolete interface in active code or user docs.

### RUST-03: Build the pinned compiler frontend skeleton

Deliverables:

- isolated `rust-tools/rust2vir` project and toolchain pin;
- Cargo preflight and sanitized driver invocation;
- validating pre-expansion file loader and source/AST gate;
- HIR subset validator;
- MIR extraction and deterministic diagnostics;
- identity, comparison, branch, and return lowering.

Exit gate: simple positive fixtures emit deterministic VIR and all preflight
negative fixtures reject without executing user build code.

### RUST-04: Add arithmetic and runtime safety

Deliverables:

- checked MIR pattern recognizers;
- arithmetic, division/remainder, shift, and index lowering;
- safety-check completeness validation;
- runtime-safety VC generation and negative pattern fixtures.

Exit gate: changing or removing any compiler-inserted safety assertion causes
deterministic rejection or a failed golden test, never silent approximation.

### RUST-05: Add aggregates, contracts, and calls

Deliverables:

- fixed arrays and by-value structs;
- Rust contract parser and typed resolution;
- call-closure discovery and cycle rejection;
- contract-based static-call WP;
- topological theorem and certificate dependencies.

Exit gate: the complete positive corpus generates stable property and safety
VCs.

### RUST-06: Integrate policy scan and evidence

Deliverables:

- Rust routing through the already-migrated generic frontend runner;
- Rust population of the shared policy scan/evidence v1 schemas;
- `payment-policy-rust-alpha` strategy metadata and language-neutral reports;
- generic source-manifest payload attached to certificate artifacts.

Exit gate: a Rust payment-policy example passes both source-free checkers and
produces evidence that separates trusted proof evidence from helper artifacts.

### RUST-07: Harden and release-gate

Deliverables:

- positive and negative corpus completion;
- frontend and importer fuzzing;
- differential interpreter tests;
- deterministic clean-machine CI job with the pinned nightly toolchain;
- compiler-upgrade procedure and release report integration.

Exit gate: all Rust gates, all migrated Go gates, checker agreement, path
sanitization, artifact determinism, and obsolete-interface searches pass from a
clean checkout.

## 21. Expected file and module impact

New paths:

```text
go-tools/go2vir/
  go.mod
  main.go
  {loader,features,lower,contract,emit,manifest}.go

rust-tools/rust2vir/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  src/bin/rust2vir.rs
  src/bin/rust2vir-driver.rs
  src/{preflight,source_gate,hir_check,mir_lower,contract,emit,manifest}.rs

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
```

`go-tools/go2vir` replaces the existing `go-tools/go2gir` directory rather than
coexisting with it in the post-cutover tree.

Expected modifications:

- root `Cargo.toml` workspace exclusion for the isolated compiler frontend;
- `mpk-vc` type, expression, WP, safety, obligation, and export modules;
- `mpk-cli` frontend runner, routing, policy scan, evidence, and report modules;
- Go frontend module identity and direct VIR emitter;
- every Go/VC/policy generated fixture and hash-bearing example;
- development specs, trust-boundary examples, templates, user documentation,
  CI scripts, and release reporting.

Removed at cutover:

- `go-tools/go2gir` and production GIR import/emission code;
- active `Std.Go.Base` VC mappings once `Std.Program.Base` migration passes;
- GIR-only fixtures and old frontend/policy protocol parsers;
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
| Build scripts or proc macros execute code | Metadata preflight, dependency ban, build-script/proc-macro rejection before compilation. |
| User or Cargo config changes semantics | Sanitized environment, isolated Cargo home, project-config rejection, explicit flags and target. |
| Frontend omits a panic condition | MIR assertion consumption checks plus VIR profile safety-completeness validation and differential tests. |
| Rust source claim is overstated | Keep compiler/frontend/VIR untrusted and label source linkage as traceability only. |
| New Rust axioms weaken release policy | Require zero new Rust semantic category; stop for governance review if checked foundations are insufficient. |
| Go behavior regresses during the breaking migration | Baseline every positive/negative fixture, compare obligation intent, run Go/VIR differential tests, and require a reviewed migration report. |
| Temporary dual paths drift before cutover | No released dual mode; one atomic gate removes GIR producers and consumers together. |
| ProofOps or CI consumes removed v0 fields | Inventory every downstream field/flag, update them in the cutover change, and reject old schemas deterministically. |
| Target-dependent semantics reuse a hash | Hash target and pointer width in VIR semantic parameters and exercise multi-target fixtures. |
| Compiler resource exhaustion | Deterministic structural limits; operational failures never become acceptance. |

## 24. Completion criteria

The unified migration and Rust v0 are complete only when all of the following
hold:

- the normative Go/Rust semantic profiles, Rust subset, VIR, source-manifest,
  frontend, VC, and policy v1 specs are frozen;
- Go source, contracts, loops, runtime checks, policy classification, and
  examples use the sole VIR path with reviewed regenerated artifacts;
- no production parser, CLI flag, schema, fixture, CI command, or user guide
  consumes or emits GIR v0, `go2gir`, policy v0, or `source_gir_hash`;
- every accepted Go/SSA and Rust/HIR/MIR form has an explicit semantics and
  test;
- unsupported or unknown forms fail closed with deterministic codes;
- the compiler toolchain, target, flags, source, manifests, lockfile, contracts,
  frontend binaries, VIR, and VCs are hash-pinned in traceability metadata;
- property, call-site, overflow, division/remainder, shift, and index obligations
  are generated where required;
- the Rust example produces a canonical `.mpcert` accepted by both checkers;
- the recomputed axiom report satisfies its declared policy;
- artifacts are deterministic and contain no absolute local paths;
- the migrated Go positive/negative suite preserves reviewed semantics; all
  expected hash changes are recorded rather than suppressed;
- certificate v0 encoding, source-free checking, checker agreement, and axiom
  category encoding remain unchanged;
- documentation never presents rustc, `rust2vir`, VIR, or a successful build as
  trusted proof evidence.

## 25. Primary references

- [rustc_driver and rustc_interface](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html)
- [rustc_driver callback stages](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver_impl/trait.Callbacks.html)
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
- [Cargo support for external tools](https://doc.rust-lang.org/cargo/reference/external-tools.html)
- [Cargo environment and workspace-wrapper behavior](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
