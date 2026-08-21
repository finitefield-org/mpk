# Rust Subset v0 and Hermetic Toolchain Specification

Status: normative and frozen for implementation.

This specification owns the Rust source profile `mpk.rust.checked.v0`, the
target allowlist `mpk.rust.targets.v0`, the compiler adapter profile, and the
build-input contract `mpk.rust.build_inputs.v0`. It is read together with
`VIR_V0.md`, `FRONTEND_PROTOCOL_V0.md`, `SOURCE_MANIFEST_V0.md`,
`SOURCE_MAP_V0.md`, and `RELEASE_BUNDLES_V0.md`. A conflict is an
implementation-blocking specification error; an implementation MUST NOT choose
one interpretation locally.

## 1. Conformance, closure, and classification

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. Every accepted
list is closed. An omitted source, AST, HIR, MIR, type, operation, attribute,
target, Cargo field, compiler option, process, path, or environment value
rejects; it is never accepted by analogy.

The only language/profile pairing is `rust` / `mpk.rust.checked.v0`. Its
registered IDs are:

| Purpose | Exact ID |
|---|---|
| deterministic limits | `mpk.rust.limits.v0` |
| targets | `mpk.rust.targets.v0` |
| frontend environment | `mpk.rust.frontend_environment.v0` |
| rustc arguments | `mpk.rust.frontend_arguments.v0` |
| MIR adapter | `mpk.rust.mir.4d08223c.v0` |
| build recipe | `mpk.rust.build_recipe.nightly-2025-06-01.v0` |
| execution host | `mpk.host.linux-x86_64-gnu.glibc2_27.v0` |
| runtime layout | `mpk.runtime.linux-x86_64-gnu.glibc2_27.v0` |

Validation uses the public phase order in `FRONTEND_PROTOCOL_V0.md`. Within a
phase, validation uses this order and stops before any later row:

1. streaming byte/count limits and checked-counter overflow;
2. no-follow file type, immutable capture, path grammar, containment, case
   folding, and duplicate identity;
3. UTF-8 and language/TOML/JSON lexical parse;
4. closed structural shape and required file/field presence;
5. identity, order, graph, target, and cross-field checks;
6. language name/type/borrow checking;
7. closed source/HIR subset, call closure, purity, and contracts;
8. pinned compiler identity/session/argument checks;
9. MIR dialect, lowering, and semantic-check completeness;
10. canonical public emission and repeated-artifact linkage.

An operational, process, toolchain, or internal-invariant failure in a started
step is `frontend-error` and takes precedence over semantic work that did not
complete. A Rust parse/name/type/borrow error is `source-error`. A well-formed
form outside this profile, a contract refusal, or a deterministic source/MIR
limit is `rejected`. When a phase can collect multiple issues of its fixed
class, it collects all such issues and applies the shared issue ordering and
truncation rule. It never mixes issues from a later phase. If source parsing
cannot complete, source errors own the result and source-gate issues inferred
from an incomplete tree are not emitted.

## 2. Portable input and immutable module closure

### 2.1 Portable paths

A normalized input path is 1 through 1,024 ASCII bytes, relative, and separated
only by `/`. Each component is 1 through 255 bytes and matches
`[A-Za-z0-9._-]+`. A component MUST NOT be `.`, `..`, end in `.`, or equal
under ASCII case folding to `CON`, `PRN`, `AUX`, `NUL`, `COM1` through `COM9`,
or `LPT1` through `LPT9`, with or without an extension. Empty components,
leading/trailing slash, backslash, colon, URI/drive/UNC syntax, control bytes,
NUL, and percent-decoded reinterpretation reject. Paths are unique under byte
equality and ASCII case folding.

Every accepted input is a regular file opened from a retained source-root
directory handle with no-follow semantics. Symlinks, hard-link aliases within
the captured set, reparse points, devices, sockets, FIFOs, root escape, and a
path whose identity changes between enumeration and open reject with
`RUST_PREFLIGHT_*`. Each opened file is streamed once into an immutable buffer
while computing length and SHA-256, and the snapshot is copied only from that
buffer. No original path is reopened. Short reads, post-open replacement,
concurrent path-set drift, and copied-byte/hash disagreement reject.

### 2.2 Selected package and manifest

`source-root` is exactly one self-contained Cargo package. The manifest option
normalizes to the literal `Cargo.toml`; `Cargo.lock` is also required. Both are
captured before parsing. `Cargo.toml` is at most 1 MiB and `Cargo.lock` at most
4 MiB. A missing/stale lock, workspace, ancestor discovery, `.cargo/config`,
`.cargo/config.toml`, target-repository `rust-toolchain`, or
`rust-toolchain.toml` rejects.

The selected manifest uses Cargo's TOML semantics but only these fields:

| Table | Accepted fields |
|---|---|
| `[package]` | required `name`, `version`, `edition = "2021"`; optional `publish = false`; descriptive `authors`, `description`, `homepage`, `documentation`, `repository`, `license`, `keywords`, and `categories` are parsed but have no semantic effect |
| `[lib]` | optional `name` and optional portable `path`; `name` is otherwise Cargo's documented package-name-to-crate-name result and `path` otherwise equals `src/lib.rs` |
| `[features]` | absent, or exactly `default = []` |
| `[[bin]]`, `[[example]]`, `[[test]]`, `[[bench]]` | non-selected targets may contain only `name`, portable `path`, `test`, `bench`, `doc`, and an empty `required-features`; they never enter VIR or the compiled source inventory |
| empty dependency tables | `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` may be absent or empty only |

All other top-level tables and build-affecting fields reject, including
`workspace`, inherited workspace values, `build`, `links`, `rust-version`,
`resolver`, target-specific dependency tables, profiles, patches,
replacements, registries, badges, auto-discovery overrides, `crate-type`,
`proc-macro`, harness overrides, path-bearing `readme`/`license-file`, and a
nonempty feature. The effective selected target is exactly one default Rust
library (`lib`/rlib for Cargo's checking step); an explicit crate type or a
second matching library rejects. External
dependencies of every class, build scripts, and procedural macros in the
analyzed package reject before Cargo starts.

The selected package name matches `[A-Za-z][A-Za-z0-9_-]*`. The library crate
name and every crate/module/function/parameter/local/constant/struct/field name
in the selected closure match `[A-Za-z_][A-Za-z0-9_]*`, are not `_`, and are
not raw or non-ASCII identifiers. The package is compared byte-for-byte with
`--package`; the crate equals the first `--function` segment.

The package version is one canonical SemVer 2.0.0 value: three dot-separated
decimal core identifiers without a leading zero except literal `0`, optionally
followed by `-` and dot-separated nonempty ASCII alphanumeric/hyphen
prerelease identifiers, then optionally `+` and equivalent build identifiers.
A numeric prerelease identifier has no leading zero. No whitespace, `v`
prefix, omitted core component, or Cargo comparator is accepted.

The parsed `Cargo.lock` has `version = 4` and exactly one `[[package]]` record.
That record has only `name` and `version`, equal byte-for-byte to the selected
manifest values. It has no source, checksum, dependency, replace, metadata, or
patch record. Comments and whitespace are TOML transport only; the captured raw
bytes and parsed value are both retained. An unknown top-level key, second
package, or regenerated value unequal to the captured lock is
`RUST_PREFLIGHT_LOCKFILE`.

### 2.3 Module-closure discovery

The walker starts at `[lib].path` or `src/lib.rs`. It parses that immutable
buffer and follows only ordinary `mod name;` declarations. Inline `mod name {
... }` bodies are traversed in the containing buffer. For an out-of-line module
in `lib.rs` or `mod.rs`, candidates are sibling `name.rs` and `name/mod.rs`.
For one in non-`mod.rs` file `dir/parent.rs`, candidates are
`dir/parent/name.rs` and `dir/parent/name/mod.rs`. Inline-module segments are
appended before resolving an out-of-line child. Exactly one candidate MUST
exist. The algorithm is the Rust 2021 default rule just stated; no compiler or
filesystem fallback may widen it.

Discovery is depth-first in source byte order, but the captured inventory is
sorted by normalized path. It never globs or reads an unrelated `.rs` file.
Missing, ambiguous, duplicate, cyclic, or syntactically unresolved module edges
are `source-error` with `RUST_SOURCE_MODULE_*`. A nonportable, case-colliding,
symlinked, or root-escaping candidate is `rejected` with
`RUST_PREFLIGHT_*`. `cfg`, `cfg_attr`, `path`, macro syntax, or another
expansion-affecting form on a module rejects with `RUST_SUBSET_*` before a
candidate is followed.

The same source gate runs on every captured file in preflight, in the custom
rustc `FileLoader` before returning its immutable bytes, and on the root AST
callback. After analysis, root-callback plus loader reads MUST equal the
preflight inventory exactly. An unexpected compiler read or an unread captured
source is `frontend-error` with `RUST_FRONTEND_SOURCE_INVENTORY`; no scan or
unsnapshotted read follows.

## 3. Closed source, HIR, and item profile

Expansion-affecting and explicit path/import rules apply crate-wide. The
remaining type, operation, purity, and control-flow checks apply to the selected
function's conservative HIR call closure.

### 3.1 Accepted items and paths

The accepted item set is:

- ordinary file or inline modules;
- free functions with plain immutable identifier parameters, explicit accepted
  parameter types, and exactly one explicit accepted non-unit result;
- primitive `const` items initialized by a Boolean literal, an in-range integer
  literal, or an accepted leading-unary-minus signed literal;
- braced-field named structs passed and returned by value, with accepted fields
  in source declaration order; `struct S {}` is the accepted zero-field form,
  while unit and tuple struct declarations/constructors reject;
- compiler-resolved same-crate paths made only of `crate`, `self`, `super`, and
  ordinary identifier segments;
- inherited/private or bare `pub` visibility on accepted modules, functions,
  constants, structs, and fields; visibility is not a VIR semantic field; and
- acyclic same-crate helper functions in the conservative call closure.

The only attributes are crate-level `#![no_std]` and inert
`doc = <string-literal>`, including documentation-comment equivalents. The
prelude is `core` exactly when the accepted crate-level `no_std` is present and
otherwise `std`. Every other attribute rejects. In particular, `cfg`,
`cfg_attr`, `path`, derive, test, repr, lint levels, target features,
`no_mangle`, export/linkage, inline assembly, macro-valued documentation, and
unknown attributes reject.

The following are closed rejections: binary verification targets; statics and
thread locals; extern crates, `use`, aliases, globs, re-exports, external or
absolute non-crate paths; restricted visibility; macro definitions or
invocations, `include!`, expansion provenance; impls, traits, methods,
associated items, generics, where clauses; extern, ABI, variadic, async, const,
unsafe, or associated functions; custom linkage and FFI.

### 3.2 Types and values

| Rust source type | Exact VIR type |
|---|---|
| `bool` | `{"kind":"bool"}` |
| `i8`, `i16`, `i32`, `i64` | same-width signed `bv` |
| `u8`, `u16`, `u32`, `u64` | same-width unsigned `bv` |
| `isize`, `usize` | signed/unsigned BV32 or BV64 from the mandatory target |
| `[T; N]` | fixed array when `T` is accepted and `N` is an in-range literal or accepted `usize` constant, maximum 256 |
| named struct | nominal VIR struct, maximum 64 fields and 16 aggregate edges |

Zero-length arrays and zero-field structs are accepted. An array-length
constant's compiler-resolved type MUST be exactly target-width `usize`; numeric
fit cannot substitute for type identity. Struct layout, padding, ABI,
endianness, and niches never enter VIR.

Rejected types include unit, never, `i128`, `u128`, floats, `char`, strings,
`str`, references, raw pointers, slices, vectors, boxes, tuples, enums, unions,
function types, closures, trait objects, generic or dynamically sized types,
and any type requiring drop glue. Coercions and adjustments are rejected except
for identity reborrowing-free adjustments erased by rustc between exactly
equal accepted types. A whole-place MIR `Move` of an accepted no-drop value is
the same pure value read as `Copy`; projected/partial move rejects, and a
projection operand is accepted only when rustc classifies it as `Copy`.

An integer literal is accepted only after rustc resolves an accepted type and
the mathematical value is in range. The spelling may use Rust separators and
radix/suffix forms accepted by rustc, but lowering emits canonical decimal. A
leading unary minus joined to an integer literal is recognized before constant
folding; `-128_i8` is one `Const`, not `bv_neg`. An out-of-range literal is
`source-error`, not an unsupported feature.

### 3.3 Statements, control flow, and calls

Accepted statements are initialized, non-shadowing
`let [mut] name [: accepted type] = expression`; plain `=` assignment to a
mutable local; accepted value expression statements; blocks; `if`/`else`;
early `return`; a final return expression; and direct same-crate free-function
calls.

Rejected forms include uninitialized or shadowing bindings, mutable parameter
patterns, destructuring, compound assignment, field/index mutation, loops,
`break`, `continue`, match, `if let`, `let else`, recursion, function values,
dynamic/external calls, closures, `?`, unwind/cleanup flow, panic/assert/debug
assert/unreachable/abort operations, and any user executable path. Accepted CFG
and the conservative source call graph are acyclic.

The HIR call closure includes every compiler-resolved direct call written in an
accepted body, including a source-dead call removed before reachable MIR. Every
member is subset/purity/contract checked and remains a standalone VIR function.
Reachable `CallStatic` edges alone determine VIR declaration dependencies and
callee-first topological order, with UTF-8 function ID as tie-break. Every
function in the HIR closure has exactly one contract.

### 3.4 Expressions and operations

Accepted source families and exact lowering are:

- Boolean literals and `!`; `&&`/`||` lower to guarded `Branch` control flow;
- typed integer literals and accepted constants;
- integer `+`, `-`, `*`, `/`, `%`, unary signed `-`, `&`, `|`, `^`, `!`,
  `<<`, and `>>`, using the operation/type matrix and safety checks in
  `VIR_V0.md`;
- Boolean/integer equality and inequality, and signed/unsigned ordered integer
  comparisons;
- explicit full fixed-array construction and read-only indexing by a
  nonconstant compiler-resolved `usize` expression materialized as the pinned
  MIR `Index` local;
- struct construction with every field exactly once, and direct field read;
- plain locals and compiler-resolved same-crate constants, structs, and free
  functions; and
- direct same-crate calls with the exact callee `contract_hash`.

The primitive operator mapping is closed:

| Rust form | Accepted resolved operand type | VIR operation |
|---|---|---|
| `!x` | `bool` | `bool_not` |
| `!x` | any accepted integer | `bv_not` |
| `-x` | signed integer | `bv_neg` |
| `+`, `-`, `*` | two identical integers | `bv_add`, `bv_sub`, `bv_mul` |
| `/`, `%` | two identical signed integers | `bv_sdiv`, `bv_srem` |
| `/`, `%` | two identical unsigned integers | `bv_udiv`, `bv_urem` |
| `&`, `|`, `^` | two identical integers | `bv_and`, `bv_or`, `bv_xor` |
| `<<` | integer left and accepted integer count | `bv_shl` |
| `>>` | signed/unsigned integer left and accepted integer count | `bv_ashr` / `bv_lshr` |
| `==`, `!=` | two identical `bool` or integer values | `eq`, `not_eq` |
| `<`, `<=`, `>`, `>=` | two identical signed integers | `signed_lt`, `signed_le`, `signed_gt`, `signed_ge` |
| `<`, `<=`, `>`, `>=` | two identical unsigned integers | `unsigned_lt`, `unsigned_le`, `unsigned_gt`, `unsigned_ge` |

The shift count may have a different accepted integer width/signedness from the
left operand; all other integer binary operands are identical types. Source
`&&` and `||` are not VIR value operators: they emit `Branch` and evaluate the
right operand only on the Rust-defined path. Array and struct equality are
valid in contracts but reject in Rust source: source `==`/`!=` would select a
`PartialEq` implementation and therefore is not a primitive operation in this
trait-free profile.

Rejected families include overloaded operators/comparisons, `as` casts,
conversions, integer wrapping/checked/overflowing/saturating methods, user
methods, trait indexing, non-`usize` array indices, ranges, allocation,
reference/address/dereference operations, repeated arrays, struct update
syntax, array/struct source equality, inline constants outside the accepted
typed-literal model, a literal/constant index that becomes `ConstantIndex` or
is folded before the required bounds-check pattern, and an omitted
operator/type combination.

Rust arithmetic uses `overflow-checks=yes`. `+`, `-`, `*`, and nonconstant
signed unary minus require their exact `integer_no_overflow` checks. Division
and remainder require `divisor_nonzero`; signed forms also require
`signed_divrem_representable`. A signed shift count additionally requires
`shift_count_nonnegative`; every shift requires
`shift_count_less_than_width`. Indexing requires `index_in_bounds`. Missing,
duplicate, reordered, or extra checks reject with `RUST_SEMANTICS_*`.
Explicit panic rejects; compiler assertions are consumed only when they match
one required check. An orphan assertion, unknown assertion message, reused
overflow flag, unwind, cleanup, resume, terminate, or unconsumed drop rejects.

For one instruction, checks are strictly ordered
`integer_no_overflow`, `divisor_nonzero`,
`signed_divrem_representable`, `shift_count_nonnegative`,
`shift_count_less_than_width`, then `index_in_bounds`; operation-qualified ties
use `add`, `sub`, `mul`, `neg`, `div`, then `rem`. Only applicable entries are
present. Duplicate checks or the right set in another order reject.

### 3.5 Purity

An accepted function reads only parameters, accepted constants, and locals;
writes locals only; allocates nothing; constructs accepted values; and calls
only accepted pure closure members. It accesses no static, thread-local,
environment, clock, random, I/O, atomic, volatile, foreign, interior-mutable,
reference, pointer, drop, panic, intrinsic, or unknown state. HIR and MIR both
enforce this rule over functions and referenced constants/types. Borrow-check
success is not a purity proof.

### 3.6 Canonical identifiers and source-map origins

Arguments become `arg0`, `arg1`, ... in source signature order; the sole
result is `result0`; accepted user locals become `local0`, `local1`, ... in HIR
declaration order. Compiler temporaries become `t0`, `t1`, ... in canonical
block/instruction order. Reachable blocks become `bb0`, `bb1`, ... by
breadth-first traversal from entry: a `Jump` enqueues its target; a `Branch`
enqueues false then true; an already discovered successor is not enqueued.
Instructions retain accepted MIR order within a block. Unreachable MIR blocks
are omitted only after the HIR source closure has been validated.

The source map contains exactly the total mapping required by
`SOURCE_MAP_V0.md`. A function uses its complete declaration range. A value
instruction/call uses the smallest complete accepted expression; a `Copy` uses
its binding or assignment; a `Branch` uses its `if`, `&&`, or `||` expression;
a `Jump` uses the controlling `if`/short-circuit expression, direct call, or
checked operation whose consumed assertion owns that success edge; and a
`Return` uses the explicit return/final expression. Shared ranges are valid.
Rust v0's synthetic-reason allowlist is empty. Thus every emitted function,
instruction, and reachable terminator has a `source` origin in a captured
`kind = source` input. A missing faithful range, expansion provenance,
compiler file, nearest-token substitution, or invented control-flow reason is
`RUST_FRONTEND_SOURCE_MAP_EXTERNAL` or `_RANGE`; it never becomes synthetic.

## 4. Contract sidecars

The sidecar schema is exactly `mpk.rust.contract.v0`. It is untrusted JSON and
never executes during compilation. A root object has exactly:

| Field | Rule |
|---|---|
| `schema` | `mpk.rust.contract.v0` |
| `semantic_profile` | `mpk.rust.checked.v0` |
| `target_pointer_width` | exact selected 32 or 64 |
| `function` | canonical same-crate free-function ID |
| `requires` | present and ordered; zero or more clauses |
| `ensures` | present, ordered, and nonempty; `requires` plus `ensures` has 1 through 64 clauses |
| `modifies` | empty array |
| `panic` | `forbidden` |
| `termination` | `total` |
| `loops` | empty array |

`--contract` is repeatable. Paths are portable and source-root-relative;
contracts are indexed by canonical function ID, independent of CLI order.
Duplicate, unused, missing, unresolved, or wrong-target contracts reject.
Variables name source parameters, result index is only `0`, locals are not
visible, and integer literals carry exact width and signedness. Resolution
renames parameters to VIR argument IDs before the normalized contract hash.

A sidecar expression is exactly one branch of this recursive closed union:

| Branch | Exact JSON members |
|---|---|
| parameter | `{"parameter": Name}` |
| result | `{"result": 0}` |
| Boolean literal | `{"bool": true}` or `{"bool": false}` |
| bitvector literal | `{"bv":{"decimal": CanonicalDecimal, "width": 8, 16, 32, or 64, "signed": Boolean}}` |
| unary operator | `{"op": "not" or "bv_neg" or "bv_not", "args": [Expr]}` |
| Boolean n-ary operator | `{"op": "and" or "or", "args": [Expr, ...]}` with 2 through 64 operands |
| binary operator | `{"op": BinaryContractOp, "args": [Expr, Expr]}` |

`Name` uses the accepted source identifier grammar. `CanonicalDecimal` is `0`
or an optional `-` followed by a nonzero digit and decimal digits; it has no
leading plus, leading zero, separator, radix, or suffix. Its mathematical value
must fit the declared width/signedness, and an unsigned literal cannot be
negative. `BinaryContractOp` is exactly `eq`, `not_eq`, `signed_lt`,
`signed_le`, `signed_gt`, `signed_ge`,
`unsigned_lt`, `unsigned_le`, `unsigned_gt`, `unsigned_ge`, `bv_add`, `bv_sub`,
`bv_mul`, `bv_and`, `bv_or`, `bv_xor`, `bv_shl`, `bv_ashr`, or `bv_lshr`.
Object-member order is immaterial before JCS; array order is semantic. No atom
may carry `op`/`args`, no operator may carry an atom field, and no alternate
`lhs`/`rhs` or scalar-literal spelling is accepted.

Normalization to the `VIR_V0.md` contract expression union is structural and
closed. `parameter` resolves to `{"var":"argN"}`; `result` and `bool` retain
their one field; and `{"bv":{"decimal":D,"width":W,"signed":S}}` becomes
`{"int":{"value":D,"width":W,"signed":S}}`. A unary sidecar `args[0]`
becomes the VIR `value`; `and`/`or` retain their normalized `args`; and every
binary sidecar `args[0]`/`args[1]` becomes VIR `lhs`/`rhs`. The normalizer makes
no other shape, value, or operator rewrite.

Atoms are typed parameters, result 0, Boolean literals, and typed bitvector
integer literals. Operators are exactly unary `not`; `and`/`or` with 2 through
64 Boolean operands; binary `eq`/`not_eq` over identical accepted types;
`signed_lt/le/gt/ge` and `unsigned_lt/le/gt/ge` over matching signedness; and
`bv_add`, `bv_sub`, `bv_mul`, `bv_and`, `bv_or`, `bv_xor`, `bv_neg`, `bv_not`,
`bv_shl`, `bv_ashr`, and `bv_lshr`. Binary operations have arity two and unary
operations arity one. Non-shift BV binary operands have identical types; a
shift count may have another accepted BV width/signedness, `bv_ashr` requires a
signed left operand, and `bv_lshr` an unsigned left operand. Arrays/structs may
occur only as exact-typed variables or results in equality. Field/index/literal
aggregate selection, conversion, division, and remainder are not contract
operations.

Operand arrays retain sidecar order; no flattening, reassociation, commuting,
or deduplication occurs. Bitvector contract operations are total and create no
runtime-safety check. The normalized VIR contract and `MPK-CONTRACT-0.1` hash
are exactly those in `VIR_V0.md`; the raw sidecar input SHA remains distinct.

## 5. Compiler, target, HIR, and MIR adapter

### 5.1 Pinned Rust distribution

The sole distribution is `nightly-2025-06-01` for build host
`x86_64-unknown-linux-gnu`:

| Identity | Exact value |
|---|---|
| rustc release | `1.89.0-nightly (4d08223c0 2025-05-31)` |
| rustc commit | `4d08223c054cf5a56d9761ca925fd46ffebe7115` |
| rustc commit date | `2025-05-31` |
| rustc LLVM | `20.1.5` |
| Cargo release | `1.89.0-nightly (64a124607 2025-05-30)` |
| Cargo commit | `64a12460708cf146e16cc61f28aba5dc2463bbb4` |
| rustfmt | `1.8.0-nightly` |
| Clippy | `0.1.89-nightly` |

The required rustup-style component set is exactly `cargo`, `rustc`, host
`rust-std`, target `rust-std` for both registered targets, `rustc-dev`,
`llvm-tools-preview`, `rustfmt-preview`, and `clippy-preview`. `rust-src` is
normatively **not included**: the frontend does not use `-Z build-std`, and the
fuzz smoke uses the distributed target library. Any discovery that this exact
adapter requires `rust-src` blocks implementation and requires a reviewed spec
amendment; it is not permission to fetch it.

The byte-exact `rust-toolchain.toml` template uses rustup request spellings
`rustfmt` and `clippy`. For this dated channel those two names resolve only to
manifest components/archive names `rustfmt-preview` and `clippy-preview` above;
the assembler implements that fixed mapping without invoking rustup. A literal
or resolved component-set difference rejects.

The normative xz URLs and SHA-256 values are:

| Component/target | SHA-256 |
|---|---|
| `cargo-nightly-x86_64-unknown-linux-gnu.tar.xz` | `f8eab4d8201709489b1a51125dab3121ff49b41a928a31f459d4c680e60eb0df` |
| `rustc-nightly-x86_64-unknown-linux-gnu.tar.xz` | `5b07912c2b8c5162fab56171b4bf3323db25c5f3b94c5a570d1f5e077056f6ae` |
| `rust-std-nightly-x86_64-unknown-linux-gnu.tar.xz` | `8e365e30adeb0f7a1162fdf57dbc697a822f87d15e814e1f005dc40750d64269` |
| `rust-std-nightly-i686-unknown-linux-gnu.tar.xz` | `999fe513b92f562f90e10b555d25f62d169938e9847dc1167d592820a3f4e332` |
| `rustc-dev-nightly-x86_64-unknown-linux-gnu.tar.xz` | `89b142f4d82e6cfeebefb0a2cf646cc33e12a522be8b3e2256e4fff587b409ed` |
| `llvm-tools-nightly-x86_64-unknown-linux-gnu.tar.xz` | `08a251f5622d5b7f78a80049e3d0547ff471ce400a976db9174696140b9c7fbe` |
| `rustfmt-nightly-x86_64-unknown-linux-gnu.tar.xz` | `c4a275c7924f2a2bf89ee3369291d8a2fa2d22e30ceb8191429cc1f88db57685` |
| `clippy-nightly-x86_64-unknown-linux-gnu.tar.xz` | `e5d5446b82cc3604e239ed2ba27e4540eb7566822980e2ce47744fa614f05691` |

Each URL is the shown filename under
`https://static.rust-lang.org/dist/2025-06-01/`. The update/provision recipes
verify the dated channel manifest before an archive and install archive
components into a fresh tree without rustup.

### 5.2 Targets and effective cfg

`mpk.rust.targets.v0` contains exactly:

| target | pointer width |
|---|---:|
| `i686-unknown-linux-gnu` | 32 |
| `x86_64-unknown-linux-gnu` | 64 |

Custom target JSON, target path, implicit host target, and every other triple
reject. With the semantic flags in section 7, the exact sorted cfg sets are the
`target_cfg` fixtures in `rust-subset-v0.json`; they include
`overflow_checks`, `panic="abort"`, `relocation_model="pic"`, and the pinned
compiler's complete atomic/feature strings. Missing, extra, reordered, or
duplicate cfg values reject. A target or compiler change rotates VIR semantic
parameters, manifests, bundle registry, and every golden fixture.

### 5.3 Query and callbacks

The driver checks its embedded commit before constructing a compiler session.
It installs the immutable `FileLoader`, validates the root AST in the parse
callback, computes the conservative HIR closure in `after_analysis`, and, for
each local closure `DefId` in canonical ID order, requests exactly
`mir_drops_elaborated_and_const_checked`. The query is forced and borrowed
before any optimized-MIR query for that body. `optimized_mir`, textual MIR,
LLVM IR, and post-optimization bodies are forbidden.

The accepted dialect is post-borrow-check, post-drop-elaboration,
const-checked, unoptimized MIR for commit `4d08223c...`. The validator closes
over statement, rvalue, operand, place/projection, aggregate, terminator,
assertion-message, `SourceInfo`, cleanup, and call-instance enums. Unknown
discriminants, a changed checked-operation/assert pattern, expansion span,
projected move, cleanup edge, or query theft reject. A compiler/query layout
change is `frontend-error` with `RUST_TOOLCHAIN_MIR_ADAPTER`; a source-generated
but unsupported closed enum form is `rejected` with `RUST_MIR_*`.

### 5.4 Closed MIR-to-VIR mapping

The only accepted MIR statement kinds are `Assign`, `StorageLive`,
`StorageDead`, and `Nop`. `StorageLive`/`StorageDead` must agree with the
validated local lifetime but emit no VIR node. `Nop` is ignored only when its
`SourceInfo` belongs to an accepted source statement and it owns no value,
drop, or check. Every other statement kind is `RUST_MIR_STATEMENT`.

An `Assign` destination is one whole local. Its rvalue is exactly `Use`,
accepted primitive `BinaryOp`, `UnaryOp`, complete
`Aggregate` array/accepted ADT struct, or a read-only accepted place
projection. The sole additional rvalue is the pinned signed-shift check's
same-width signed-to-unsigned `IntToInt` cast; it has exactly one use in that
check predicate and emits no VIR value. `Len`, every other `Cast`, `Repeat`,
`Ref`, `ThreadLocalRef`, `RawPtr`, `ShallowInitBox`, `CopyForDeref`,
`Discriminant`, `NullaryOp`, `WrapUnsafeBinder`, and an omitted rvalue reject
with `RUST_MIR_RVALUE`.

Operands are only `Copy`, a whole-place no-drop `Move`, and an accepted typed
`Constant`. A place begins at an accepted local and has either no projection or
one or more read-only `Field` or `Index` projections matching the validated
aggregate type. `ConstantIndex`, `Deref`, `Downcast`, `Subslice`,
opaque/subtype casts, and every write projection reject. A fixed-array `Index`
uses the exact target-width `usize` index local from the source operation. Its
predecessor computes `Lt(index, const N_usize)`, where `N` equals the validated
array type length, and asserts that predicate with the pinned `BoundsCheck`
message carrying the same `N` and index. Those predicate assignments emit no
VIR value; the successor projection emits one `Index` with
`index_in_bounds`.

The only reachable terminators are:

| MIR terminator | Exact VIR treatment |
|---|---|
| `Goto` | `Jump` |
| Boolean `SwitchInt` with values `0`/otherwise | `Branch`, false then true successor |
| `Return` with the one return place | `Return` |
| resolved local direct `Call` with `UnwindAction::Unreachable` | `CallStatic` followed by `Jump` to its required target |
| compiler `Assert` with `UnwindAction::Unreachable` | attach its recognized predicate to the one checked operation and lower its sole success edge as `Jump` |

A call or assert without its normal target, another `SwitchInt`, cleanup block,
unwind/terminate/continue action, `Drop`, `UnwindResume`, `UnwindTerminate`,
`TailCall`, `Yield`, coroutine drop, `FalseEdge`, `FalseUnwind`, `InlineAsm`, or
`Unreachable` rejects with the matching `RUST_MIR_*` code. Reachability is
computed before lowering with checked block/edge counters.

`BinaryOp(AddWithOverflow|SubWithOverflow|MulWithOverflow)` yields one private
pair local. Field 1 is projected only into the vector's matching overflow
predicate and compiler `Assert`; field 0 is projected only into the one
whole-place value destination on the normal path. The pair local and both
fields have no other store, projection, reuse, or observation.
Division, remainder, shifts, and array indexing use the
pinned assertion-message variants and operand identity fixed by
`rust-subset-v0.json`. Every compiler-created assertion is consumed once and
every required VIR safety check has one recognized source pattern. This table,
the enum discriminant closure, and the exact assertion shapes are adapter data
for `mpk.rust.mir.4d08223c.v0`; no debug-string comparison widens them.

The adapter recognizes only an assertion chain whose normal-target path reaches
exactly one matching operation before any other value use or branch. All
assertions in that chain attach to that operation in section 3.4 order. The
assertion block remains a canonical reachable block, but its panic edge is
discarded under `panic=abort` and its normal target becomes the block's `Jump`;
the operation is not duplicated or moved across a source expression. A chain
with an intervening side effect, alternate predecessor, reused predicate,
wrong normal target, or assertion after the operation except the exact
overflowing-`BinaryOp` pair rejects as `RUST_MIR_CHECKED_PATTERN`.

The compiler-created predicate subgraph is also consumed exactly once and
emits no VIR values: overflow uses tuple field 1 with expected `false`; negation
uses `Eq(value, MIN)` with expected `false`; signed division/remainder uses the
zero predicate followed by `BitAnd(Eq(rhs, -1), Eq(lhs, MIN))` with expected
`false`; a shift uses `Lt(unsigned_count, const lhs_width)` with expected
`true`; and indexing uses the fixed-length predicate above. For a signed shift,
`unsigned_count` is the check-only same-width cast, so its one assertion
supplies both `shift_count_nonnegative` and
`shift_count_less_than_width`, in that VIR order. An unsigned shift supplies
only the latter. Operand identity, widths, signedness, Boolean temporary use,
message discriminant, expected bit, normal target, and `UnwindAction` must all
match the adapter vector; equivalent algebra or compiler prose is not accepted.

## 6. Isolated frontend package and locked source closures

The project root is `rust-tools/rust2vir` and is excluded from the stable root
workspace. Its package target set is exactly:

- package `rust2vir` version `0.1.0`;
- non-installable library `rust2vir_internal` at `src/lib.rs`;
- main binary `rust2vir` at `src/bin/rust2vir.rs`;
- driver binary `rust2vir-driver` at `src/bin/rust2vir-driver.rs`;
- auto-discovered unit/integration tests, which may depend on the library; and
- the separately materialized fuzz package's single dependency edge to the
  library.

There is no example, bench, build script, proc-macro, dylib, cdylib, or other
Cargo target. A candidate/installed frontend inventory contains the two regular
ELF binaries only. An rlib, dylib, dep-info file, test, example, fuzz binary,
incremental state, lock, manifest, source, or other Cargo artifact rejects the
release inventory.

The `manifest_rules.frontend` vector distinguishes the three named package
targets from the only two permitted auto-discovery categories: unit tests in
the library and one integration-test target per captured top-level `tests/*.rs`
file. A nested test target, test target outside that path grammar, doctest,
example, bench, or other auto-discovered Cargo target rejects. Test target
names are derived by pinned Cargo from a filename stem matching exactly
`[A-Za-z_][A-Za-z0-9_]*` and are validated before the vector's
`integration_test_mode` launch; they are never release artifacts.

`rust-build-inputs-v0.json` owns byte-exact base64 templates for frontend
`Cargo.toml`, frontend `Cargo.lock`, `rust-toolchain.toml`, future fuzz
`Cargo.toml`, future fuzz `Cargo.lock`, and Cargo-home `config.toml`. Their raw
SHA-256, byte length, lock format 4, and required final LF are normative. The
frontend has no registry or path dependency. The fuzz manifest has exactly one
path dependency, alias `rust2vir_internal`, package `rust2vir`, version
`=0.1.0`, absolute sandbox path `/mpk/frontend`; it resolves only the
`rust2vir_internal` library and the captured parent source inventory. Every
other path edge, escape, or package/target mismatch rejects.

The only registry is named `crates-io` with lock source
`registry+https://github.com/rust-lang/crates.io-index` and crate download
origin `https://static.crates.io/crates`. Every registry lock record has a
nonempty lowercase SHA-256 checksum. Alternate/sparse/custom registries, git
dependencies, `[patch]`, `[replace]`, and a package not in the vector graph
reject.

The three graphs are:

1. `frontend`: the root `rust2vir 0.1.0` only;
2. `fuzz`: roots `rust2vir-fuzz 0.0.0` and path-bound `rust2vir 0.1.0`, plus
   the exact registry graph rooted at `libfuzzer-sys =0.4.9`; and
3. `cargo-fuzz-tool`: upstream `cargo-fuzz 0.13.1` manifest and lock from tag
   commit `1b34938413a104856042376b285c8d1c1e11b098`, tree
   `343defc9ad9b09d963bca36ca8672577c680774b`.

The cargo-fuzz tag archive SHA-256 is
`3dae1ab57e738c1059635eb824062e4de79474080612f60a0ec0decf455d9e65`.
Its raw `Cargo.toml` is 647 bytes, SHA-256
`1873b2396fec2111a59f2ba72e34a4d72e7172b26dad361155e7a0eec52b51bd`;
its raw `Cargo.lock` is 18,301 bytes, SHA-256
`5499f4d6dd0dcc3b5dbaf40a72921347d5b2a14bd5bbe01ae816ef7a2566e625`.
Both end in LF. The vector's complete sorted package/checksum/feature/edge
graph is normative; RUST-03-T01 and RUST-07-T03 may neither resolve nor update
it.

Every registry `.crate` byte stream is verified against its lock checksum,
extracted no-follow into `vendor/<name>-<version>`, and inventoried including
the generated `.cargo-checksum.json`. That file's package checksum and every
regular-file digest MUST agree with the raw crate, descriptor, and observed
vendor tree before Cargo starts. The descriptor records every package's source
origin, exact Cargo-manifest license string, reviewed SPDX expression, license
files, and notice files. Manifest values `MIT/Apache-2.0` and `Unlicense/MIT`
use only the two exact `license_normalization.legacy_exact_mappings` entries in
the vector; every other value is the identity mapping. No other legacy syntax
or inferred equivalence is accepted. The package/license groups in the
build-input vector are normative. Missing, extra, renamed, rewritten, or
unnoted content rejects.

The frontend graph executes no dependency target. The complete fuzz lock graph
has 12 package records, but the frozen Linux target unit graph contains only
the nine records listed in `fuzz_smoke.linux_target_units`; `cfg-if 1.0.4`,
`getrandom 0.4.3`, and `r-efi 6.0.0` are locked inactive target closure. The
fuzz build may execute only `libfuzzer-sys
0.4.9/build-script-build` and `libc 0.2.189/build-script-build`, keyed by the
vector lock checksum. The
cargo-fuzz-tool Linux build may execute only the vector's `custom-build` list
and proc macros `clap_derive 4.0.21`, `proc-macro-error-attr 1.0.4`,
`serde_derive 1.0.193`, and `thiserror-impl 1.0.50`, each keyed by exact target
name and package checksum. A build target not in those lists rejects before
Cargo. No dependency child may launch Cargo; only the top-level cargo-fuzz
smoke graph in section 9 has nested Cargo.

## 7. Build-input materialization and native closure

### 7.1 Fixed native origins

Native material uses these immutable origins:

| Role | Origin and immutable identity |
|---|---|
| compiler/linker/archiver and sanitizer closure | LLVM 18.1.8 archive `clang+llvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04.tar.xz`, SHA-256 `54ec30358afcc9fb8aa74307db3046f5187f9fb89fb37064cdde906e062ebf36` |
| native development sysroot | Docker Official Image `buildpack-deps:bionic`, OCI index `sha256:6184955e89adc7744b2f3202aaa2f8b7ea8c6040d6da2b6a4d4643b77a24636a`, linux/amd64 manifest `sha256:816cb0d4a26fd8584b27d190bdd57ba7048be4fc20c259e60a985bec812887dc` |
| execution runtime | Docker Official Image `ubuntu:18.04`, OCI index `sha256:152dc042452c496007f07ca9127571cb9c29697f42acbfad72324b2bb2e43c98`, linux/amd64 manifest `sha256:dca176c9663a7ba4c1f0e710986f5a25e672842963d95b960191e2d9f7185ebe` |

The assembler verifies OCI manifest/blob digests and applies the vector's exact
projection algorithm. It does not run an image or package manager. It first
constructs the merged linux/amd64 OCI filesystem with standard layer and
whiteout semantics. The development projection recursively selects every
entry beneath each absolute `source_recursive_roots` member in the vector. A
selected regular file is copied beneath `output_root` under the source path
with its leading slash removed. A selected symbolic or hard-link
name is resolved only inside that immutable merged image, with bounded hops,
and materialized as an independent ordinary file containing the terminal
regular file's bytes; an escape, cycle, dangling link, non-regular terminal,
or output-path collision rejects. Directory entries themselves are implicit.
This yields the complete C/C++ header, startup-object, GNU library/linker-script,
and GCC runtime development view without retaining a link. Every
`required_output_paths` member must result from that projection with exactly
that ordinary-file name; the list explicitly guards all frozen startup objects
and linker inputs and does not limit the recursively selected output.

The runtime projection resolves vector `elf_entrypoints_ref` to the complete
`toolchain_executables` array and starts from every resulting regular file
after the Rust/LLVM archives and reproducible cargo-fuzz output have been
materialized. Each entrypoint must request the frozen interpreter. The
assembler reads ELF metadata without executing a binary and computes a
breadth-first `DT_NEEDED` closure. For each needed SONAME it searches the
immutable runtime image's `soname_source_directories` in vector order, resolves
a link with the same bounded in-image rule, and copies the terminal bytes under
`native-runtime/lib/x86_64-linux-gnu/<SONAME>`. Duplicate SONAME candidates are
allowed only when their terminal bytes are equal; an unresolved SONAME,
nonliteral loader token, `RPATH`/`RUNPATH`, interpreter change, cycle-counter
overflow, or dependency outside the resulting closure rejects. Before candidate
publication, the assembler repeats that inspection for all four exact
`post_build_entrypoints`: `rust2vir`, `rust2vir-driver`, and the
`driver_protocol` and `rust_contract` fuzz targets at the absolute paths in the
vector. Main and driver may request only the already frozen native-runtime
closure. A fuzz target may additionally request only an exact file in the
descriptor's `compiler_rt_files` inventory. Every post-build entrypoint must
retain the frozen interpreter and must not add `RPATH` or `RUNPATH`; any other
dependency stops implementation for a reviewed spec amendment rather than
widening the runtime. The interpreter is materialized at the vector's exact
`interpreter_output` path.

The development and runtime projection results are ordinary files explicitly
listed in the generated production descriptor; the selector roots, entrypoints,
and search order are not substitutes for that inventory. The runtime result
therefore contains exactly the ELF interpreter and SONAME closure for staged
Cargo, rustc, clang/LLD build tools, cargo-fuzz, main, driver, and both fuzz
targets.
All projected files and required Ubuntu, glibc, GCC runtime, LLVM,
compiler-rt/libFuzzer/sanitizer, Rust, Cargo, rustfmt, Clippy, crate, and
cargo-fuzz notices are descriptor inventory entries and reviewed bundle
content where redistribution requires them.

The only native executable names are `clang`, `clang++`, `ld.lld`, `llvm-ar`,
`llvm-ranlib`, and `llvm-strip`, materialized as ordinary files under
`/mpk/toolchain/bin`. Rust and LLVM tar extraction applies the vector's 64-hop,
archive-root-contained `archive_link_materialization` rule to a selected
symbolic or hard-link name; absolute/escaping/dangling/cyclic/non-regular
targets and output collisions reject. No link is retained. Linker is
`/mpk/toolchain/bin/clang`. Beside it, the assembler materializes the exact
43-byte non-executable `/mpk/toolchain/bin/clang.cfg` from raw template
`clang_config`; its two final-LF-terminated lines are, in order,
`--sysroot=/mpk/native-sysroot` and `-fuse-ld=lld`. Clang's default
configuration search loads `clang.cfg` before explicit argv for every host and
target link. A missing, differently located, additional, reordered, or changed
configuration file rejects before execution. C++ compiler is
`/mpk/toolchain/bin/clang++`; archiver and ranlib are the shown LLVM tools.
The AddressSanitizer/compiler-rt files are the exact LLVM 18.1.8 x86_64 files
named in `compiler_rt_files`. The libFuzzer runtime is instead the inventoried
`toolchain/lib/libfuzzer.a`, built from the 26 UTF-8-sorted C++ files in the
locked `libfuzzer-sys 0.4.9` source. The vector owns the complete `clang++`
argv template, source-to-object substitution, all 26 expansions, deterministic
`llvm-ar rcD` and `llvm-ranlib -D` arrays, and final component path. Update
builds it twice in independently empty fixed views and accepts only equal
archive bytes. The development projection must include the vector's required
`libstdc++.a`, linker-name `libstdc++.so`, and SONAME file; post-build ELF
projection must include the resulting `libstdc++.so.6` runtime dependency.
Ambient `cc`, `c++`, `ld`, `ar`, SDK, GCC installation, and host
include/library directory are forbidden.

The initial execution ABI is Linux/x86_64/GNU glibc 2.27, minimum kernel ABI
`5.10.0`. The required host probes are exactly
`mpk.release.probe.linux_namespaces.v0` from `RELEASE_BUNDLES_V0.md`. The
interpreter mount is `/lib64/ld-linux-x86-64.so.2`; native libraries mount at
`/lib/x86_64-linux-gnu`. That is the sole native-runtime SONAME directory; the
immutable-image search order used to construct it is the exact two-entry vector
array. The same unprefixed interpreter path in LLD argv is only the literal
written into the output ELF; it is not a link-time file read. Every native
startup object and native system-library search used by LLD remains under
`/mpk/native-sysroot` (the exact generated `lib/../lib64` spelling normalizes
within that root); Rust/LLVM inputs use only the inventoried toolchain paths.
Process loading additionally uses only the exact initial and child
`LD_LIBRARY_PATH` values in section 7.3. Neither rule permits a `/lib`,
`/lib64`, `/usr/lib`, or other host fallback.

### 7.2 Two closed inventories

The build/test inventory contains every regular file derived from the eight
Rust component archives, LLVM archive projection, native development sysroot,
native runtime, cargo-fuzz source and reproducible binary, the reproducible
prebuilt libFuzzer archive, the exact generated Clang configuration, all three
registry source closures, Cargo-home seed, licenses, and notices. It supports
only the launcher modes in section 9.

The smaller evidence-execution inventory contains only:

- `/mpk/toolchain/bin/cargo` and `/mpk/toolchain/bin/rustc`;
- compiler/codegen/LLVM shared libraries needed by rustc and the registered
  driver;
- host and both registered target `rust-std` content needed by Cargo/rustc;
- the registered `rust2vir` and `rust2vir-driver` in the frontend bundle; and
- the exact interpreter/native shared-library closure for those executables.

It excludes rustfmt, Clippy, cargo-fuzz, clang/LLD build tools, dependency
sources, `rustc-dev` metadata not needed at driver runtime, caches, native
development headers/startup objects, tests, and fuzz artifacts. Every retained
file is explicitly listed; "needed" is not runtime discovery authority.

### 7.3 Fixed paths and environment

Evidence execution uses these exact path constants:

`/mpk/input`, `/mpk/toolchain`, `/mpk/frontend`, `/mpk/work`, `/mpk/home`,
`/mpk/cargo-home`, `/mpk/tmp`, `/mpk/target`, `/mpk/driver-output`, read-only
`/mpk/driver-request.json`, and read-only `/mpk/native-runtime`.
`/mpk/driver-output/result.json.partial` and
`/mpk/driver-output/result.json` are the only permitted driver output names.
Build gates additionally expose read-only `/mpk/vendor` and
`/mpk/native-sysroot`. No value is substituted from a host locator. Each
individual fixed value contains no `:`, `=`, or byte `0x1f`.

Input/frontend/toolchain/vendor/native-sysroot/native-runtime/request views are
sealed read-only after copy. Input, request, and work are no-exec in evidence
mode. Only fresh private home, Cargo home, temp, target, and driver output are
writable and non-aliasing. Build mode uses working directory `/mpk/frontend`;
evidence Cargo uses the fixed launcher work root and explicit
`/mpk/input/Cargo.toml`.

The evidence Cargo environment has exactly these entries, sorted here by name:

~~~text
CARGO_ENCODED_RUSTFLAGS=<section 7.4 bytes>
CARGO_HOME=/mpk/cargo-home
CARGO_INCREMENTAL=0
CARGO_NET_OFFLINE=true
CARGO_TARGET_DIR=/mpk/target
CARGO_TERM_COLOR=never
HOME=/mpk/home
LANG=C
LC_ALL=C
LD_LIBRARY_PATH=/mpk/toolchain/lib
PATH=/mpk/toolchain/bin
RUSTC=/mpk/toolchain/bin/rustc
RUSTC_WORKSPACE_WRAPPER=/mpk/frontend/rust2vir-driver
RUST_BACKTRACE=0
TERM=dumb
TMPDIR=/mpk/tmp
TZ=UTC
~~~

The same map is `launcher.evidence_environment` in the build-input vector.
Its `CARGO_ENCODED_RUSTFLAGS` value MUST equal the
`0x1f`-join of `evidence_encoded_rustflags_elements`; the separator is one
control byte, not the six ASCII characters `\u001f`.

Build mode starts from that closed base after removing
`RUSTC_WORKSPACE_WRAPPER` and the evidence
`CARGO_ENCODED_RUSTFLAGS`. It adds exactly:

~~~text
AR=/mpk/toolchain/bin/llvm-ar
CARGO_BUILD_JOBS=1
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/mpk/toolchain/bin/clang
CC=/mpk/toolchain/bin/clang
CFLAGS=--sysroot=/mpk/native-sysroot -fdebug-prefix-map=/mpk/frontend=.
CXX=/mpk/toolchain/bin/clang++
CXXFLAGS=--sysroot=/mpk/native-sysroot -fdebug-prefix-map=/mpk/frontend=.
RANLIB=/mpk/toolchain/bin/llvm-ranlib
SOURCE_DATE_EPOCH=1748736000
~~~

These removals/additions are the exact vector
`build_environment_remove`/`build_environment_add` values. Non-fuzz build
flags are likewise the one-byte `0x1f` join of
`non_fuzz_encoded_rustflags_elements`.

Non-fuzz build, format, lint, test, and version modes additionally set
`CARGO_ENCODED_RUSTFLAGS` to the `0x1f`-joined elements
`-Clinker=/mpk/toolchain/bin/clang`,
`-Clink-arg=--sysroot=/mpk/native-sysroot`,
`-Clink-arg=-fuse-ld=lld`, and
`--remap-path-prefix=/mpk/frontend=.`. Bounded fuzz mode instead requires
`CARGO_ENCODED_RUSTFLAGS` absent and sets `RUSTFLAGS` to this exact one-line
ASCII suffix, with single spaces and no leading or trailing space:

~~~text
-Clinker=/mpk/toolchain/bin/clang -Clink-arg=--sysroot=/mpk/native-sysroot -Clink-arg=-fuse-ld=lld --remap-path-prefix=/mpk/frontend=. --remap-path-prefix=/mpk/work/fuzz-project=./fuzz
~~~

Cargo-fuzz prepends its exact instrumentation string as specified below.

The default `clang.cfg` in section 7.1 applies independently of Cargo's split
between target units and host build-script units. Target rustc invocations also
pass the same sysroot and LLD selection explicitly through the frozen link
arguments above; identical repetition is intentional and the vector freezes
the resulting single effective sysroot plus the exact backend argv. Host
build-script rustc invocations do not receive target `RUSTFLAGS`, so the default
configuration is their sole authority for both values. Clang may not search for
or execute ambient `ld`, nor resolve a native startup object or native system
library outside `/mpk/native-sysroot`.

The build namespace binds the offline vendor source. Locale is `C`, timezone
`UTC`, hostname `mpk-build`, umask `022`, and logical job count one. Proxy,
registry, credential, rustup, user Cargo config, target-specific runner,
inherited loader, `RUSTC_BOOTSTRAP`, and every unlisted variable are absent.
`LD_LIBRARY_PATH` is constructed without inheritance or empty elements.

Cargo 1.89 may transform the wrapper child's loader value only by prepending,
in order, existing compiler-created directories beneath the fresh
`/mpk/target/<target>/debug/deps`, `/mpk/target/debug/deps`, then the validated
`/mpk/toolchain/lib/rustlib/x86_64-unknown-linux-gnu/lib`, followed by the
initial `/mpk/toolchain/lib`; it omits a nonexistent candidate rather than
emitting an empty element. Probe invocations retain the initial value. Any
other addition, reorder, source-controlled directory, nonempty preexisting
target, or host path is `RUST_TOOLCHAIN_LOADER_PATH`.

An empty target directory is required. A target path that is absent only before
the launcher creates it is valid; an existing nonempty, source-controlled,
cache, home, or host directory rejects. No target output is reused.

### 7.4 Semantic rustflags and Cargo commands

`CARGO_ENCODED_RUSTFLAGS` is UTF-8 for these exact argv elements joined by one
byte `0x1f`, with no leading/trailing separator and no incidental space:

~~~text
-C
overflow-checks=yes
-C
panic=abort
-C
debug-assertions=no
-C
opt-level=0
-Z
mir-opt-level=0
--remap-path-prefix=/mpk/input=.
~~~

`RUSTFLAGS` is absent. Evidence metadata argv is exactly:

~~~text
cargo metadata --manifest-path /mpk/input/Cargo.toml --format-version 1
--no-deps --locked --offline --no-default-features --color never
~~~

Evidence check argv is exactly:

~~~text
cargo check --lib --package PACKAGE --target TARGET
--manifest-path /mpk/input/Cargo.toml --locked --offline
--no-default-features --jobs 1 --message-format json --color never
~~~

`PACKAGE` and `TARGET` are the already validated identities and remain single
argv elements. The order shown is exact.

The rustc wrapper receives the rustc executable as argv 1. For the frozen
Cargo check, wrapper invocations occur in this exact order: first `rustc -vV`;
the host crate-information probe; the selected-target crate-information probe;
a second `rustc -vV`; and the selected primary library. The optional direct
sysroot identity probe is exactly `rustc --print sysroot` and is outside that
check sequence. Each probe consumes empty stdin, produces bounded
stdout/stderr, emits no artifact, and has the exact full argv in
`rust-subset-v0.json`. No other probe is accepted.

The primary invocation is byte-for-byte the vector's
`rustc_wrapper_argv.selected_primary_lib.argv` after replacing only its five
declared variables. `CRATE_ROOT` is the captured portable `[lib].path`, or
`src/lib.rs` when omitted; `CRATE` is the validated effective library crate;
`TARGET` is the selected allowlisted triple; and `METADATA16`/`EXTRA16` are
exactly 16 lowercase hexadecimal characters produced by the pinned Cargo.
They are opaque but must repeat across two clean builds. The fixed argv also
owns the ordering and values of edition, JSON diagnostics, crate type, emit,
embed-bitcode, debug info, check-cfg, out-dir, dependency search paths, and all
semantic flags. Each path is normalized against its fixed view.

In the primary grammar, `-A`, `-W`, `-D`, `-F`, `--cap-lints`, another
`-C`/`-Z`, feature/cfg
injection, response file, stdin source, output outside target, a variable value
outside its declared grammar, reordered/duplicate/unknown argument, or any
non-primary compiler invocation rejects before a compiler session. Only the
one selected primary library may run the driver and emit one result.

## 8. Canonical `mpk.rust.build_inputs.v0`

The tracked descriptor is solely
`release/build-inputs/rust/build-inputs.json`. It is compact RFC 8785 JCS plus
one LF. The LF counts toward the 256 MiB descriptor limit but is excluded from
the self-hash preimage:

~~~text
SHA256(
  UTF8("MPK-RUST-BUILD-INPUTS-0.1") || 0x00 ||
  JCS(BuildInputs with only build_inputs_sha256 removed)
)
~~~

The root has exactly `schema`, `profile_id`, `recipe_id`,
`execution_host_profile_id`, `runtime_layout_profile_id`, `registry`,
`rust_distribution`, `native`, `graphs`, `cargo_fuzz`, `licenses`,
`components`, and `build_inputs_sha256`. `schema` is exactly
`mpk.rust.build_inputs.v0`. Every ID/path array is strictly increasing by UTF-8
bytes and unique. Every object is closed.

Descriptor `graphs`, `licenses`, and `components` sort by their `id`, `id`, and
`name` UTF-8 bytes respectively. Package records sort by `(name UTF-8 bytes,
canonical version UTF-8 bytes, source UTF-8 bytes)`; roots, feature names, and
`name@version` dependency IDs sort by their complete UTF-8 bytes. Executable
dependency targets sort first by kind rank (`custom-build`, then `proc-macro`),
then package name, canonical version, and target name by UTF-8 bytes. Component
files sort by complete portable path. License/notice references and every other
set-valued string array sort by complete UTF-8 bytes. Semantic arrays—argv,
contracts, graph child order, loader order, and MIR checks—retain the explicit
order given and are never set-sorted. The explicitly labelled fuzz
`unit_dag` node array and each `depends_on` array are the exception: they are
sets serialized by complete UTF-8 `id`, while runtime order is any valid
topological linearization.

The exact nested fields and valid synthetic instance are normative in
`rust-build-inputs-v0.json`:

- distribution date, releases, commits, components, targets, archive URLs and
  digests, tool-source digests, and the `rust_src=false` decision;
- native origin/index/platform digests, linker/archiver/tool identities,
  exact default Clang configuration bytes/options, sysroot/runtime profiles,
  interpreter, loader paths, startup objects, and notice references;
- the one registry and three manifest/lock raw sizes, SHA-256 values, final-LF
  flags, lock format, roots, package/checksum/feature/edge graphs;
- cargo-fuzz version/tag/tree/archive, build argv/environment profile, two-build
  equality result, and resulting executable SHA-256;
- deterministic libFuzzer source/object/archive recipe, complete bounded-smoke
  process DAG, target/scratch substitutions, and every Cargo/rustc/build-script/
  linker/fuzz-engine argv;
- component provenance and license/notice references; and
- each component's sorted regular files with portable relative `path`, Boolean
  `executable`, integer `size_bytes`, and raw `sha256`.

The vector's sibling `production_projection` object is frozen recipe input,
not a field of `mpk.rust.build_inputs.v0`. Its roots and ELF entrypoints MUST be
expanded by section 7.1 before descriptor emission, and its sole
`generated_regular_files` entry MUST be materialized from the referenced raw
template; only the resulting sorted ordinary-file inventory is serialized. A
directory selector, link, SONAME, or unexpanded glob is therefore never a
component `files` entry.

The vector's `valid_descriptor_cache_content` reconstructs every byte of the
synthetic cache used only for descriptor/cache conformance: four paths use
their named raw templates, including `toolchain/bin/clang.cfg`; each vendor
checksum file is compact JCS of the package's lock checksum and an empty
synthetic file map, and every other file is the UTF-8 component path with no
LF. Its metadata MUST recompute to the synthetic descriptor. These non-ELF
synthetic bytes are never executed (their Boolean executable metadata still
exercises descriptor validation) and are not a production projection; update
mode must replace the entire synthetic inventory with files produced from the
immutable origins and section 7 recipe.

Machine-local paths, credentials, timestamps other than the fixed recipe
epoch, unknown fields, duplicate records, graph disagreement, wrong ordering,
and cross-field identity disagreement reject. The descriptor does not
inventory itself. Current `.rs`, test, fixture, fuzz target, and seed bytes are
also excluded: they are invocation source inventory, not build-input identity.

The ignored cache key is exactly
`release/build-input-cache/rust/<build_inputs_sha256>`, using the recomputed
lowercase digest. It contains exactly directories `toolchain/`,
`tool-sources/`, `native-sysroot/`, `native-runtime/`, `vendor/`,
`cargo-home-seed/`, and `notices/`. Directories are implicit. Every cache
regular file appears exactly once in one component inventory; unlisted/extra
files, symlinks, hard-link aliases, devices, sockets, and FIFOs reject.

Before allocation or mount, readers enforce inclusive maxima: 268,435,456
descriptor JCS+LF bytes; 1,048,576 regular files; 8,192 package records across
the graphs; 1,024 bytes per inventory path; 4,294,967,296 bytes per regular
file; and a checked 34,359,738,368-byte declared and observed aggregate. A
counter overflow, size disagreement, or boundary+1 rejects before any cache
byte is exposed or executed.

The build-input conformance harness uses this closed internal failure set:

| Internal code | Owned validation |
|---|---|
| `RUST_BUILD_INPUTS_TRANSPORT` | JCS+LF framing, byte limit, or checked transport counter |
| `RUST_BUILD_INPUTS_SHAPE` | closed descriptor/vector object shape |
| `RUST_BUILD_INPUTS_HASH` | descriptor self-hash |
| `RUST_BUILD_INPUTS_CACHE_KEY` | recomputed cache-key path |
| `RUST_BUILD_INPUTS_INVENTORY` | component membership, size, digest, or self-entry |
| `RUST_BUILD_INPUTS_SOURCE_EXCLUSION` | invocation-source byte incorrectly included in cache identity |
| `RUST_BUILD_INPUTS_VENDOR` | vendored crate or `.cargo-checksum.json` disagreement |
| `RUST_BUILD_INPUTS_GRAPH` | lock/package/feature/edge/target/count/aggregate disagreement |
| `RUST_BUILD_INPUTS_CARGO_HOME` | seed or post-run private Cargo-home disagreement |
| `RUST_BUILD_INPUTS_PROVENANCE` | origin or immutable identity disagreement |
| `RUST_BUILD_INPUTS_LICENSE` | missing or mismatched license/notice association |
| `RUST_BUILD_INPUTS_PATH` | nonportable, unordered, escaping, or wrong-root path |
| `RUST_BUILD_INPUTS_FILE` | file-size limit, type, alias, or checked file counter |
| `RUST_BUILD_INPUTS_PUBLICATION` | staging, no-replace, or atomic commit invariant |

These names are test-harness discriminants, not public Issues and not valid in
a Rust private driver request or result. The release assembler maps every one
to the already registered outward error `BUNDLE_BUILD_INPUTS_INVALID`; it does
not expose the internal suffix. Thus the closed `RUST_*` driver code rule in
section 10 is not extended by this table.

`cargo-home-seed/` contains exactly one regular `config.toml`, whose vector
bytes replace `crates-io` with named directory source `mpk-vendor` at
`/mpk/vendor` and set offline mode. It contains no registry cache/index,
credential, provider, alias, external command, executable, link, or alternate
config. The copied file is read-only and no-replace. Cargo may leave only that
file and an optional zero-byte regular `.package-cache` lock file; every other
post-run Cargo-home entry rejects. Resolution outside `/mpk/vendor` rejects.

`--update-build-inputs rust` is the sole tracked descriptor writer and the only
mode besides provisioning that may fetch. It fetches fixed origins, stages
privately, builds cargo-fuzz and the vector-expanded libFuzzer archive twice in
separate empty sandboxes, requires equality within each pair, inventories and
validates staged content, computes the descriptor/hash/cache key from those
bytes, publishes the cache no-follow/no-replace, then atomically replaces the
descriptor as the commit point. A published unselected valid cache after
failed descriptor commit is harmless.

`--provision-build-inputs rust` starts from an unchanged valid descriptor,
recreates bytes in private staging, validates complete equality, and publishes
only to the fixed key. An equal occupant is reused after full validation; an
unequal/malformed occupant fails without repair. `--check-build-inputs rust`,
candidate/registered bundle modes, and every launcher/evidence route are
network-disabled and write neither descriptor nor cache. Descriptor and cache
are excluded from candidate/installed release roots.

Every build invocation enumerates allowed checkout roots `Cargo.toml`,
`Cargo.lock`, `rust-toolchain.toml`, `src/`, `tests/`, `testdata/`, and, when
materialized, `fuzz/`. It no-follow copies each portable regular file once into
a fresh private tree, hashes the copy, checks descriptor-bound templates, and
records every remaining source byte in the invocation inventory. Modes execute
only from sealed copies. Before candidate/registered publication the assembler
re-enumerates and rehashes the current closure and requires equality; mutation,
path drift, short read, or hash/length difference aborts publication.

## 9. Hermetic build launcher and fuzz smoke

The launcher accepts only these external argv grammars; tokens and order are
exact, and a variable token is validated against the captured manifest:

| Mode | Accepted argv after launcher |
|---|---|
| release | `cargo build --locked --offline --release --bins --target x86_64-unknown-linux-gnu --jobs 1` |
| frontend format | `cargo fmt --all -- --check` |
| fuzz format | `cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check` |
| lint | `cargo clippy --locked --all-targets -- -D warnings` |
| all tests | `cargo test --locked` |
| one integration test | `cargo test --locked --test TEST`, where `TEST` is one exact captured Cargo test target |
| version | `cargo run --locked --bin rust2vir -- --version` |
| bounded fuzz | the exact form below for each of `driver_protocol` and `rust_contract` |

No alias, reordered option, other package/target/profile/feature, additional
`--`, trailing token, arbitrary Cargo subcommand, rustup proxy, install, or
toolchain path is accepted.

The leading `cargo` in this external grammar is a required literal caller
token, not path lookup authority. After the complete array matches, the
launcher executes only the already validated
`/mpk/toolchain/bin/cargo` handle and preserves every remaining argument
byte-for-byte. It never consults ambient `PATH` for a build mode.

For fuzz smoke the launcher copies the byte-exact fuzz manifest, lock, target,
and enumerated read-only seed files to fresh `/mpk/work/fuzz-project`; the
manifest's only path edge still resolves `/mpk/frontend`. It creates distinct
writable `/mpk/work/fuzz-project/corpus/TARGET` and
`/mpk/work/fuzz-project/artifacts/TARGET` directories, copies seeds no-follow
with normalized metadata, then seals all other fuzz-project files read-only.
Neither writable path may alias or reside in the checkout/cache.

The outer command is exactly:

~~~text
cargo fuzz run --fuzz-dir /mpk/work/fuzz-project
--target x86_64-unknown-linux-gnu --target-dir /mpk/target/fuzz
--sanitizer address --jobs 1 TARGET
/mpk/work/fuzz-project/corpus/TARGET --
-runs=256 -seed=1 -max_len=1048576 -timeout=5 -rss_limit_mb=1024
~~~

Cargo first launches only the inventoried `cargo-fuzz 0.13.1` with subcommand
token `fuzz`. That tool launches, in order, its exact `rustc --version` probe,
the following nested build, another exact `rustc --version` probe, the
following nested run, and the fuzz target. Cargo-fuzz 0.13.1 does not launch a
metadata subprocess on this path.

~~~text
cargo build --manifest-path /mpk/work/fuzz-project/Cargo.toml --target x86_64-unknown-linux-gnu --release --config profile.release.debug="line-tables-only" --bin TARGET --target-dir /mpk/target/fuzz
~~~

~~~text
cargo run --manifest-path /mpk/work/fuzz-project/Cargo.toml --target x86_64-unknown-linux-gnu --release --config profile.release.debug="line-tables-only" --bin TARGET --target-dir /mpk/target/fuzz -- -artifact_prefix=/mpk/work/fuzz-project/artifacts/TARGET/ -runs=256 -seed=1 -max_len=1048576 -timeout=5 -rss_limit_mb=1024 /mpk/work/fuzz-project/corpus/TARGET
~~~

Each line is an argv array, not a shell command; quoted characters around
`line-tables-only` are part of that one `--config` argument. `TARGET` is the
same validated target name throughout. Cargo-fuzz uses literal executable
token `cargo`; the closed `PATH=/mpk/toolchain/bin` must resolve it to the
already inventoried `/mpk/toolchain/bin/cargo`, whose identity is revalidated
before each exec. Cargo itself dispatches only the inventoried cargo-fuzz file,
with the exact full argv (including its leading `fuzz` token), cwd, and child
nesting in `bounded_child_process_graph`.

That object is the complete normative process graph. Every listed argv includes
argv 0. The two cargo-fuzz compiler probes, nested Cargo programs, four
pre-unit Cargo rustc probes, eleven compile rustc nodes, two build-script
nodes, the libc build script's one nested `rustc --version`, three nested
`clang` linker nodes, their three nested `ld.lld` backend nodes, and final fuzz
executable are all present with exact program, cwd, argv, multiplicity, parent
depth, and dependency edges. Each Clang process first reads the inventoried
default configuration and then launches exactly its recorded LLD child. No
bounded-smoke `clang++`, ambient `ld`, `llvm-ar`, `llvm-ranlib`, proc-macro, or
other native child exists: `libfuzzer-sys` must take its validated custom-
archive branch.

The four pre-unit probes occur in vector order. Cargo 1.89 may choose any ready
unit even with one job, so `unit_dag` deliberately freezes a DAG rather than a
false total order: node records sort by UTF-8 `id`, each node occurs exactly
once, and the observed execution must be one dependency-respecting topological
linearization. Nested children remain ordered under their owner. The only
runtime substitutions are the selected vector row for `TARGET` and its four
recorded Cargo/codegen values, plus three distinct fresh scratch basenames
matching `rustc[A-Za-z0-9]{6}`. Each scratch value has exactly four permitted
occurrences across the owning Clang argv and its LLD child's matching argv:
once as `/mpk/tmp/<token>/symbols.o` and once as the `raw-dylibs` search path in
each array. It must resolve beneath the private temp root and is deleted with
that invocation. Substitution happens before member-for-member comparison. A
missing, repeated, reparented, dependency-violating, or unknown process
rejects.

The immediately following `cargo run` must reuse that exact successful target.
Its compiler/native-child array is empty and its sole child is the target with
the vector's full engine argv. Any fingerprint miss, new rustc, build script,
proc macro, linker, native tool, or different output path is a gate failure
rather than an allowed conditional rebuild.

Cargo-fuzz replaces the nested `RUSTFLAGS` value with its generated
instrumentation prefix, one separating space, and the exact outer suffix from
section 7.3. The resulting exact one-line ASCII value is:

~~~text
 -Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares --cfg fuzzing -Cllvm-args=-simplifycfg-branch-fold-threshold=0 -Zsanitizer=address -Cllvm-args=-sanitizer-coverage-stack-depth -Cdebug-assertions -Ccodegen-units=1 -Clinker=/mpk/toolchain/bin/clang -Clink-arg=--sysroot=/mpk/native-sysroot -Clink-arg=-fuse-ld=lld --remap-path-prefix=/mpk/frontend=. --remap-path-prefix=/mpk/work/fuzz-project=./fuzz
~~~

The value begins with one space and otherwise has single spaces between
elements, with no trailing space. Before cargo-fuzz, the launcher requires
`CUSTOM_LIBFUZZER_PATH`, `CUSTOM_LIBFUZZER_STD_CXX`, and `ASAN_OPTIONS` absent,
then sets the first two to `/mpk/toolchain/lib/libfuzzer.a` and `stdc++`.
Cargo-fuzz preserves both custom values byte-for-byte for build and run and
replaces the absent `ASAN_OPTIONS` with `detect_odr_violation=0` while replacing
`RUSTFLAGS` with the generated value above. `TSAN_OPTIONS`, `RUSTC_BOOTSTRAP`,
`CARGO_ENCODED_RUSTFLAGS`, and all other tool defaults remain absent. The custom
archive is a descriptor-inventoried regular file, is rehashed immediately
before execution, and its license reference includes the locked
`libfuzzer-sys` NCSA conjunction.

Nested Cargo otherwise inherits the closed build environment,
`CARGO_NET_OFFLINE=true`, `CARGO_BUILD_JOBS=1`, fixed target/home, and read-only
lock/vendor config. The final run receives, after cargo-fuzz's fixed
`-artifact_prefix=.../artifacts/TARGET/`, exactly the five libFuzzer options
and private corpus above. Unknown process, nested Cargo shape, argument, target,
engine, sanitizer, profile variable, environment mutation, or output locator
rejects and terminates the gate.

Build/fuzz process trees are bounded to 256 processes, 1,024 open files per
process, 16 GiB virtual and 8 GiB resident memory, 4 GiB temp bytes, 16 GiB
target bytes, 262,144 output regular files, 64 MiB aggregate child stdout, and
2 MiB aggregate child stderr, all with checked counters. Fuzz completion is
exactly 256 libFuzzer runs and seed 1; wall-clock elapsed time is not an
artifact acceptance input. A limit or unavailable isolation primitive is a
gate failure, never permission for an ambient fallback.

Two separately empty release builds over equal build-input and captured-source
inventories MUST produce byte-identical `rust2vir` and `rust2vir-driver` ELF
files. Both are rehashed against the captured source inventory immediately
before candidate or registry publication.

## 10. Stable Rust diagnostic codes

Every code below has one status and phase. Codes within one completed phase are
collected and sorted as shared Issues; table order supplies precedence when a
single first error is required.

| Code or closed prefix | Status / phase | Meaning |
|---|---|---|
| `RUST_LIMIT_INPUT_BYTES`, `RUST_LIMIT_INPUT_COUNT`, `RUST_LIMIT_PATH`, `RUST_LIMIT_CONTRACT`, `RUST_LIMIT_CALL_CLOSURE`, `RUST_LIMIT_MIR_BLOCKS`, `RUST_LIMIT_MIR_STATEMENTS`, `RUST_LIMIT_AGGREGATE`, `RUST_LIMIT_IR` | rejected / owning phase | exact deterministic limit |
| `RUST_PREFLIGHT_FILE_TYPE` | rejected / capture | non-regular/link/reparse/special input |
| `RUST_PREFLIGHT_PATH` | rejected / capture | nonportable, colliding, or escaping path |
| `RUST_SOURCE_MANIFEST_PARSE` | source-error / capture | malformed Cargo TOML |
| `RUST_PREFLIGHT_MANIFEST_FIELD` | rejected / capture | field/table outside allowlist |
| `RUST_PREFLIGHT_WORKSPACE`, `RUST_PREFLIGHT_CONFIG`, `RUST_PREFLIGHT_TOOLCHAIN_FILE` | rejected / capture | forbidden selection/config authority |
| `RUST_PREFLIGHT_DEPENDENCY`, `RUST_PREFLIGHT_BUILD_SCRIPT`, `RUST_PREFLIGHT_FEATURE` | rejected / capture | forbidden analyzed package graph |
| `RUST_PREFLIGHT_TARGET`, `RUST_PREFLIGHT_LOCKFILE` | rejected / capture | selected target/lock invariant |
| `RUST_SOURCE_MODULE_MISSING`, `RUST_SOURCE_MODULE_AMBIGUOUS`, `RUST_SOURCE_MODULE_DUPLICATE`, `RUST_SOURCE_MODULE_CYCLE` | source-error / capture | deterministic module name resolution |
| `RUST_SOURCE_PARSE` | source-error / source | Rust parse failure |
| `RUST_SUBSET_CFG`, `RUST_SUBSET_MACRO`, `RUST_SUBSET_ATTRIBUTE`, `RUST_SUBSET_IMPORT`, `RUST_SUBSET_VISIBILITY`, `RUST_SUBSET_PATH`, `RUST_SUBSET_EXPANSION` | rejected / source or subset | crate-wide source gate |
| `RUST_FRONTEND_METADATA_PROCESS`, `RUST_FRONTEND_METADATA_PROTOCOL` | frontend-error / metadata | Cargo child/process output failure |
| `RUST_PREFLIGHT_METADATA_MISMATCH` | rejected / metadata | metadata contradicts captured manifest selection |
| `RUST_SOURCE_NAME`, `RUST_SOURCE_TYPE`, `RUST_SOURCE_BORROW`, `RUST_SOURCE_LITERAL_RANGE` | source-error / typecheck | compiler language error |
| `RUST_SUBSET_IDENTIFIER`, `RUST_SUBSET_ITEM`, `RUST_SUBSET_FUNCTION_KIND`, `RUST_SUBSET_GENERIC`, `RUST_SUBSET_TRAIT`, `RUST_SUBSET_IMPL`, `RUST_SUBSET_STATIC`, `RUST_SUBSET_TYPE`, `RUST_SUBSET_DROP`, `RUST_SUBSET_PATTERN`, `RUST_SUBSET_BINDING`, `RUST_SUBSET_CONTROL_FLOW`, `RUST_SUBSET_MUTATION`, `RUST_SUBSET_OPERATION`, `RUST_SUBSET_CALL`, `RUST_SUBSET_PURITY` | rejected / subset | closed HIR/profile refusal |
| `RUST_CONTRACT_JSON`, `RUST_CONTRACT_SCHEMA`, `RUST_CONTRACT_SHAPE`, `RUST_CONTRACT_IDENTITY`, `RUST_CONTRACT_DUPLICATE`, `RUST_CONTRACT_UNUSED`, `RUST_CONTRACT_MISSING`, `RUST_CONTRACT_RESOLUTION`, `RUST_CONTRACT_PROFILE`, `RUST_CONTRACT_TYPE`, `RUST_CONTRACT_OPERATOR`, `RUST_CONTRACT_LIMIT`, `RUST_CONTRACT_HASH` | rejected / subset | contract parse, closure, or typing refusal |
| `RUST_TOOLCHAIN_COMPONENT`, `RUST_TOOLCHAIN_COMMIT`, `RUST_TOOLCHAIN_TARGET`, `RUST_TOOLCHAIN_OPTIONS`, `RUST_TOOLCHAIN_ARGUMENT`, `RUST_TOOLCHAIN_LOADER_PATH`, `RUST_TOOLCHAIN_MIR_ADAPTER` | frontend-error / release, typecheck, or lowering | pinned identity/session mismatch |
| `RUST_MIR_STATEMENT`, `RUST_MIR_RVALUE`, `RUST_MIR_OPERAND`, `RUST_MIR_PLACE`, `RUST_MIR_PROJECTION`, `RUST_MIR_TERMINATOR`, `RUST_MIR_ASSERTION`, `RUST_MIR_CHECKED_PATTERN`, `RUST_MIR_CALL`, `RUST_MIR_MOVE`, `RUST_MIR_CLEANUP` | rejected / lowering | unsupported source-generated MIR form |
| `RUST_SEMANTICS_TYPE`, `RUST_SEMANTICS_TARGET`, `RUST_SEMANTICS_CHECK_MISSING`, `RUST_SEMANTICS_CHECK_EXTRA`, `RUST_SEMANTICS_PANIC` | rejected / lowering | lowering/profile semantic mismatch |
| `RUST_FRONTEND_SOURCE_INVENTORY`, `RUST_FRONTEND_SOURCE_MAP_EXTERNAL`, `RUST_FRONTEND_SOURCE_MAP_RANGE`, `RUST_FRONTEND_SOURCE_MAP_REFERENCE` | frontend-error / source or emission | compiler inventory/span invariant |
| `RUST_FRONTEND_CHILD_OUTPUT_LIMIT`, `RUST_FRONTEND_COMPILER_CRASH`, `RUST_FRONTEND_DIAGNOSTIC_BUDGET` | frontend-error / started phase | bounded child/internal failure |
| `RUST_FRONTEND_DRIVER_PROTOCOL_*` | frontend-error / typecheck or lowering | private request/output/filesystem protocol failure |
| `RUST_LIMIT_DIAGNOSTICS_TRUNCATED` | preserves owning status | final shared truncation marker |

Unknown `RUST_*` codes reject protocol conformance. Compiler prose is optional
after shared normalization; raw stderr, source snippets, macro text, child argv,
environment dumps, host suggestions, and absolute paths are never public.

## 11. Deterministic limits

`mpk.rust.limits.v0` is exactly:

| Resource | Inclusive maximum |
|---|---:|
| `Cargo.toml` / `Cargo.lock` | 1 MiB / 4 MiB |
| contract files | 128 files, 1 MiB each, 8 MiB total |
| contract clauses/nodes/depth | 64 per function, 1,024 per function, 8,192 closure total, depth 32 |
| compiled sources | 256 files, 1 MiB each, 16 MiB total |
| complete snapshot | 512 entries, 32 MiB total |
| normalized path | 1 KiB |
| call-closure functions | 128 |
| reachable MIR blocks | 1,024/function, 8,192/closure |
| MIR statements | 100,000/function, 250,000/closure |
| array / struct / aggregate nesting | 256 elements / 64 fields / 16 levels |
| normalized public issues | 1,024 entries, 4 KiB/message, 2 MiB messages |
| Cargo/rustc output | 64 MiB stdout, 2 MiB stderr |
| private driver request | 4 MiB JCS+LF |
| private driver output | 256 MiB JCS+LF |
| VIR / source map / source manifest | 192 MiB / 32 MiB / 4 MiB JCS |
| public frontend stdout/stderr | 256 MiB JCS+LF / 2 MiB |

The serialized limits include the required transport LF, so their JSON portion
is at most one byte less. Stream limits count every observed byte. File and
aggregate counters are checked during streaming and before full allocation.
The exact boundary is accepted and boundary+1 rejects. Diagnostic budget
handling is exactly `FRONTEND_PROTOCOL_V0.md`; it does not reclassify status.

## 12. Conformance vectors and ownership

`develop/specs/vectors/rust-subset-v0.json` has schema
`mpk.rust.subset.conformance.v0` and owns accepted/rejected source families,
manifest/module/path rules, targets/cfg, MIR/semantic cases, phase precedence,
diagnostics, and every exact limit boundary.

`develop/specs/vectors/rust-build-inputs-v0.json` has schema
`mpk.rust.build_inputs.conformance.v0` and owns raw templates, graphs,
component/native projections, launcher/fuzz child graphs, valid descriptor and
hash preimage, publication state cases, and invalid mutations. Synthetic
digests are fixtures; production descriptor values are emitted only by the
frozen update recipe and first reviewed in RUST-03-T01.

An implementation MUST exercise every vector case and compare exact outcome,
phase, code, repeated identity, and hash. A compiler or Cargo update requires a
new adapter/profile ID and reviewed regeneration; it cannot mutate these v0
vectors in place merely to make a new tool pass.
